//! §10.4 Courtship mechanics — the process from attraction to pair-bond.
//!
//! Architecture §10.4: Romantic stages flow from Awareness → Attraction →
//! Flirtation → ActiveCourtship → TestingReciprocity → SocialValidation
//! → Exclusivity → Betrothal → Marriage.
//!
//! This module uses `RomanticStage` from `marriage.rs` (the unified enum)
//! and provides courtship-specific logic: eligibility checks, progression
//! probability, and reciprocity tracking.
//!
//! ```text
//! Courtship dynamics:
//!   - Notice → Appraise → Signal interest → Seek proximity
//!   - Test reciprocity → Gossip validation → Propose bond
//!   - Seek social approval → Formalize
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};
use super::marriage::RomanticStage;

/// Minimum attraction as a fraction of the trust threshold for stage advancement.
const ATTRACTION_FLOOR_RATIO: Fixed = Fixed::from_raw(6000); // 0.6

/// A courtship in progress between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Courtship {
    /// Index of agent pursuing (may be either direction).
    pub pursuer: usize,
    /// Index of agent being pursued.
    pub pursued: usize,
    /// Current romantic stage (using the unified RomanticStage enum).
    pub stage: RomanticStage,
    /// Mutual attraction level (averaged from both agents).
    pub mutual_attraction: Fixed,
    /// Social approval from the community (0–1).
    pub social_approval: Fixed,
    /// Reciprocity score — how much the pursued returns interest (0–1).
    pub reciprocity: Fixed,
    /// Number of positive interactions during courtship.
    pub positive_interactions: u32,
    /// Number of negative interactions during courtship.
    pub negative_interactions: u32,
    /// Tick when courtship began.
    pub started_tick: u64,
    /// Tick of last courtship-relevant interaction.
    pub last_interaction_tick: u64,
    /// Whether the courtship is still active.
    pub active: bool,
}

impl Courtship {
    /// Create a new courtship.
    pub fn new(pursuer: usize, pursued: usize, tick: u64) -> Self {
        Self {
            pursuer,
            pursued,
            stage: RomanticStage::Unaware,
            mutual_attraction: Fixed::ZERO,
            social_approval: Fixed::from_f64(0.5),
            reciprocity: Fixed::ZERO,
            positive_interactions: 0,
            negative_interactions: 0,
            started_tick: tick,
            last_interaction_tick: tick,
            active: true,
        }
    }

    /// Record a positive courtship interaction.
    pub fn record_positive(&mut self, tick: u64, trust: Fixed, attraction: Fixed) {
        self.positive_interactions += 1;
        self.last_interaction_tick = tick;
        self.mutual_attraction = attraction;
        self.reciprocity = (self.reciprocity + Fixed::from_f64(0.02)).clamp_01();
        // Attempt to advance stage
        self.try_advance(trust, attraction);
    }

    /// Record a negative courtship interaction (rejection, betrayal, conflict).
    pub fn record_negative(&mut self, tick: u64) {
        self.negative_interactions += 1;
        self.last_interaction_tick = tick;
        self.reciprocity = (self.reciprocity - Fixed::from_f64(0.05)).max(Fixed::ZERO);
        // May regress stage
        self.try_regress();
    }

    /// Check if stage advancement conditions are met.
    ///
    /// Requires: trust >= the next stage's threshold, attraction >= 60% of
    /// that threshold, and — once the pair has an interaction record —
    /// positive interactions must outnumber negatives. A freshly formed
    /// courtship has no record (0 positives, 0 negatives), so it may advance
    /// on trust + attraction alone (the daily pass); hostile interaction
    /// history then gates the ladder through the balance check. Advances at
    /// most one rung per call, so repeated positive contact (or a daily
    /// refresh with the pair's current trust) climbs the ladder one stage at
    /// a time, stalling at the pair's trust/attraction ceiling (Iteration
    /// 101: the daily pass feeds the pursuer's live `total_attraction()`, so
    /// the 0.6× attraction floor genuinely gates — a pair below the floor
    /// stalls even with ample trust).
    pub fn try_advance(&mut self, trust: Fixed, attraction: Fixed) {
        if let Some(next) = self.stage.next_positive() {
            // The courtship ladder concludes at Betrothal (§10.4: Awareness →
            // … → Betrothal → Marriage): Marriage and the post-marriage stages
            // belong to the pair-bond/marriage system, so a courtship record
            // must never display them.
            if next == RomanticStage::Married {
                return;
            }
            let trust_ok = trust >= next.base_trust();
            // Attraction must be at least 60% of the trust threshold
            let min_attraction = next.base_trust() * ATTRACTION_FLOOR_RATIO;
            let attraction_ok = attraction >= min_attraction;
            let no_record = self.positive_interactions == 0 && self.negative_interactions == 0;
            let balance_ok =
                no_record || self.positive_interactions > self.negative_interactions;

            if trust_ok && attraction_ok && balance_ok {
                self.stage = next;
            }
        }
    }

    /// Check if stage regression conditions are met.
    fn try_regress(&mut self) {
        if self.negative_interactions > self.positive_interactions {
            // Regress one stage back if possible
            let prev = match self.stage {
                RomanticStage::ActiveCourtship => Some(RomanticStage::Flirtation),
                RomanticStage::TestingReciprocity => Some(RomanticStage::ActiveCourtship),
                RomanticStage::SocialValidation => Some(RomanticStage::TestingReciprocity),
                RomanticStage::Exclusivity => Some(RomanticStage::ActiveCourtship),
                RomanticStage::Betrothal => Some(RomanticStage::Exclusivity),
                RomanticStage::Flirtation => Some(RomanticStage::Attraction),
                RomanticStage::Attraction => Some(RomanticStage::Awareness),
                RomanticStage::Awareness => None, // courtship failed
                _ => None,
            };
            if let Some(p) = prev {
                self.stage = p;
            } else {
                self.active = false;
            }
        }
    }

    /// Courtship probability of advancing per tick — depends on reciprocity and approval.
    ///
    /// Superseded (Iteration 76): the production advancement path is the
    /// deterministic, trust-gated `try_advance` (driven by the daily pass and
    /// the interaction wiring). A probabilistic daily roll here would draw
    /// from the Social RNG stream and shift every downstream draw, forcing a
    /// golden-baseline recalibration; the deterministic gate keeps the golden
    /// replay byte-identical. Kept as the documented probabilistic design.
    pub fn advance_probability(&self) -> Fixed {
        let base = Fixed::from_f64(0.01);
        let reciprocity_bonus = self.reciprocity * Fixed::from_f64(0.02);
        let approval_bonus = self.social_approval * Fixed::from_f64(0.01);
        (base + reciprocity_bonus + approval_bonus).clamp_01()
    }

    /// Daily update — decay reciprocity, converge social approval toward reciprocity.
    pub fn daily_update(&mut self) {
        if !self.active {
            return;
        }
        // Reciprocity slowly decays without reinforcement
        self.reciprocity = (self.reciprocity * Fixed::from_f64(0.98)).max(Fixed::ZERO);
        // Social approval tracks mutual interest: community perception follows
        // how much the pursued returns the pursuer's interest.
        self.social_approval = (self.social_approval * Fixed::from_f64(0.995) + Fixed::from_f64(0.005) * self.reciprocity).clamp_01();
    }

    /// Courtship probability of regression per tick — depends on negative interactions.
    ///
    /// Superseded (Iteration 76) for the same golden-stability reason as
    /// `advance_probability`: regression is driven deterministically by
    /// `record_negative` → `try_regress` instead of a probabilistic roll.
    pub fn regression_probability(&self) -> Fixed {
        let negatives = Fixed::from_int(self.negative_interactions as i64);
        let positives = Fixed::from_int(self.positive_interactions as i64);
        let total = negatives + positives;
        if total > Fixed::ZERO {
            (negatives / total) * Fixed::from_f64(0.03)
        } else {
            Fixed::ZERO
        }
    }
}

/// Check whether two agents are eligible to begin courtship.
///
/// Eligibility requires: both adults, not closely related, not already bonded,
/// and some minimum attraction level.
pub fn eligible_for_courtship(
    age_a: Fixed,
    age_b: Fixed,
    kinship_coefficient: Fixed,
    already_bonded: bool,
    attraction_threshold: Fixed,
    mutual_attraction: Fixed,
) -> bool {
    let adult_age = Fixed::from_f64(16.0); // developmental adulthood threshold
    let close_kin_threshold = Fixed::from_f64(0.25); // half-sibling equivalent

    age_a >= adult_age
        && age_b >= adult_age
        && kinship_coefficient < close_kin_threshold
        && !already_bonded
        && mutual_attraction >= attraction_threshold
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn courtship_stage_advances() {
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.record_positive(10, Fixed::from_f64(0.5), Fixed::from_f64(0.5));
        // Should advance from Unaware to at least Awareness or Attraction
        assert!(courtship.stage >= RomanticStage::Awareness);
    }

    #[test]
    fn courtship_stage_regresses_on_negatives() {
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = RomanticStage::Flirtation;
        // Multiple negatives should regress
        for i in 0..5 {
            courtship.record_negative(10 + i);
        }
        assert!(courtship.stage < RomanticStage::Flirtation);
    }

    #[test]
    fn courtship_fails_after_enough_negatives() {
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = RomanticStage::Attraction;
        for i in 0..10 {
            courtship.record_negative(10 + i);
        }
        assert!(!courtship.active);
    }

    #[test]
    fn eligibility_checks_adults_only() {
        assert!(!eligible_for_courtship(
            Fixed::from_f64(12.0), Fixed::from_f64(20.0),
            Fixed::ZERO, false, Fixed::from_f64(0.3), Fixed::from_f64(0.5),
        ));
        assert!(eligible_for_courtship(
            Fixed::from_f64(20.0), Fixed::from_f64(22.0),
            Fixed::ZERO, false, Fixed::from_f64(0.3), Fixed::from_f64(0.5),
        ));
    }

    #[test]
    fn eligibility_rejects_close_kin() {
        assert!(!eligible_for_courtship(
            Fixed::from_f64(20.0), Fixed::from_f64(22.0),
            Fixed::from_f64(0.5), false, Fixed::from_f64(0.3), Fixed::from_f64(0.5),
        ));
    }

    #[test]
    fn ladder_climbs_one_stage_per_positive_as_trust_grows() {
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = RomanticStage::Awareness;
        // The ladder is trust-gated: each rung requires the pair's trust to
        // have grown past the next stage's base_trust (Exclusivity 0.55,
        // Betrothal 0.65). Model the trust growth that repeated prosocial
        // contact produces in production (relationship_v2 trust climbs with
        // interaction). Seven positives climb Awareness → Attraction →
        // Flirtation → ActiveCourtship → TestingReciprocity →
        // SocialValidation → Exclusivity → Betrothal, one rung per positive.
        for i in 0..7 {
            let trust = (Fixed::from_f64(0.3) + Fixed::from_f64(0.1) * Fixed::from_int(i as i64))
                .min(Fixed::from_f64(0.7));
            courtship.record_positive(10, trust, Fixed::from_f64(0.5));
        }
        assert_eq!(courtship.stage, RomanticStage::Betrothal);
        assert!(courtship.active);
        assert_eq!(courtship.positive_interactions, 7);
        // Reciprocity accumulates with each positive (0.02 per interaction).
        assert!(courtship.reciprocity > Fixed::from_f64(0.13));
    }

    #[test]
    fn trust_threshold_gates_ladder_advancement() {
        // Trust 0.15 never clears the Attraction threshold (base 0.2), so a
        // courtship stays at Awareness no matter how often the pair interacts.
        let mut low_trust = Courtship::new(0, 1, 0);
        low_trust.stage = RomanticStage::Awareness;
        for _ in 0..20 {
            low_trust.record_positive(10, Fixed::from_f64(0.15), Fixed::from_f64(0.5));
        }
        assert_eq!(low_trust.stage, RomanticStage::Awareness);

        // Trust 0.25 clears Attraction (0.2) but not Flirtation (0.3) — the
        // ladder stalls one rung up until pair trust grows.
        let mut mid_trust = Courtship::new(0, 1, 0);
        mid_trust.stage = RomanticStage::Awareness;
        for _ in 0..20 {
            mid_trust.record_positive(10, Fixed::from_f64(0.25), Fixed::from_f64(0.5));
        }
        assert_eq!(mid_trust.stage, RomanticStage::Attraction);
    }

    #[test]
    fn direct_try_advance_is_trust_gated_and_monotonic() {
        // try_advance is now public (wired into the daily courtship pass): it
        // must advance at most one rung and never regress on its own.
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = RomanticStage::Awareness;
        // Trust below the Attraction threshold (0.2): no movement.
        courtship.try_advance(Fixed::from_f64(0.15), Fixed::from_f64(0.15));
        assert_eq!(courtship.stage, RomanticStage::Awareness);
        // Trust above it: exactly one rung per call.
        courtship.try_advance(Fixed::from_f64(0.25), Fixed::from_f64(0.25));
        assert_eq!(courtship.stage, RomanticStage::Attraction);
        courtship.try_advance(Fixed::from_f64(0.25), Fixed::from_f64(0.25));
        // Flirtation needs 0.3 — stalled at the trust ceiling.
        assert_eq!(courtship.stage, RomanticStage::Attraction);
    }

    /// §10.4 (Iteration 101): the attraction floor binds when the real
    /// attraction arg is below 0.6× the next stage's base trust — the
    /// documented simplification (daily pass passed trust as both gate and
    /// attraction) made this floor dead; the consumer now feeds the pursuer's
    /// live `total_attraction()`. These tests pin the floor as a real gate
    /// independent of trust.
    #[test]
    fn attraction_floor_gates_ladder_even_with_high_trust() {
        // Trust 0.6 clears Exclusivity (0.55) but attraction 0.2 is below its
        // floor (0.55 × 0.6 = 0.33) — the ladder must stall at the floor.
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = RomanticStage::ActiveCourtship;
        courtship.record_positive(10, Fixed::from_f64(0.6), Fixed::from_f64(0.2));
        assert_eq!(
            courtship.stage, RomanticStage::ActiveCourtship,
            "attraction below the 0.6× floor must stall advancement"
        );
    }

    #[test]
    fn attraction_at_floor_advances_with_trust() {
        // Attraction exactly at the floor (0.55 × 0.6 = 0.33) is sufficient
        // when trust clears the threshold — zero-at-zero preservation.
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = RomanticStage::ActiveCourtship;
        courtship.record_positive(10, Fixed::from_f64(0.6), Fixed::from_f64(0.33));
        assert_eq!(courtship.stage, RomanticStage::TestingReciprocity);
    }

    #[test]
    fn hostile_interactions_regress_and_can_kill_a_courtship() {
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = RomanticStage::ActiveCourtship;
        // Two hostile acts flip the balance negative at Flirtation…
        for i in 0..2 {
            courtship.record_negative(10 + i);
        }
        // First negative: positives(0) < negatives(1) → regress to Flirtation.
        // Second: 0 < 2 → regress to Attraction.
        assert_eq!(courtship.stage, RomanticStage::Attraction);
        // A courtship at Awareness that goes net-negative fails outright.
        let mut fresh = Courtship::new(0, 1, 0);
        fresh.stage = RomanticStage::Awareness;
        fresh.record_negative(10);
        assert!(!fresh.active);
    }
}
