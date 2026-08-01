//! §10.4 Courtship mechanics — the process from attraction to pair-bond.
//!
//! Architecture §10.4: Romantic stages flow from Awareness → Attraction →
//! Flirtation → Courtship → Exclusivity → Betrothal → Marriage.
//!
//! Courtship is a scripted social process with probabilistic gates:
//!   - Each stage has entry conditions based on attraction, trust, and social context
//!   - Progression requires repeated positive interactions
//!   - Social approval modulates progression speed
//!   - Rejection or betrayal can reset or reverse stages
//!
//! ```text
//! Courtship dynamics:
//!   - Notice → Appraise → Signal interest → Seek proximity
//!   - Test reciprocity → Gossip validation → Propose bond
//!   - Seek social approval → Formalize
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Stage of courtship between two agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum CourtshipStage {
    /// No romantic awareness.
    Unaware,
    /// One or both agents are aware of each other as potential partners.
    Awareness,
    /// Initial attraction detected (may be one-sided).
    Attraction,
    /// Subtle signaling of interest (glances, proximity seeking).
    Flirtation,
    /// Active courtship — pursuing the relationship deliberately.
    ActiveCourtship,
    /// Testing reciprocity — does the other feel the same?
    TestingReciprocity,
    /// Social validation — gossip, community awareness.
    SocialValidation,
    /// Exclusivity agreed upon (private or public).
    Exclusivity,
    /// Formal betrothal / engagement.
    Betrothal,
}

impl CourtshipStage {
    /// Advance to the next stage if the courtship progresses positively.
    pub fn next(&self) -> Option<Self> {
        match self {
            Self::Unaware => Some(Self::Awareness),
            Self::Awareness => Some(Self::Attraction),
            Self::Attraction => Some(Self::Flirtation),
            Self::Flirtation => Some(Self::ActiveCourtship),
            Self::ActiveCourtship => Some(Self::TestingReciprocity),
            Self::TestingReciprocity => Some(Self::SocialValidation),
            Self::SocialValidation => Some(Self::Exclusivity),
            Self::Exclusivity => Some(Self::Betrothal),
            Self::Betrothal => None, // Courtship complete — leads to marriage
        }
    }

    /// Regression — courtship can fail and step back.
    pub fn regress(&self) -> Option<Self> {
        match self {
            Self::Betrothal => Some(Self::Exclusivity),
            Self::Exclusivity => Some(Self::ActiveCourtship),
            Self::ActiveCourtship => Some(Self::Flirtation),
            Self::Flirtation => Some(Self::Attraction),
            Self::Attraction => Some(Self::Awareness),
            Self::Awareness => None, // Back to unaware — courtship failed
            Self::TestingReciprocity => Some(Self::ActiveCourtship),
            Self::SocialValidation => Some(Self::TestingReciprocity),
            Self::Unaware => None,
        }
    }

    /// Minimum trust required to enter this stage.
    pub fn trust_threshold(&self) -> Fixed {
        match self {
            Self::Unaware => Fixed::ZERO,
            Self::Awareness => Fixed::from_f64(0.1),
            Self::Attraction => Fixed::from_f64(0.15),
            Self::Flirtation => Fixed::from_f64(0.25),
            Self::ActiveCourtship => Fixed::from_f64(0.35),
            Self::TestingReciprocity => Fixed::from_f64(0.4),
            Self::SocialValidation => Fixed::from_f64(0.45),
            Self::Exclusivity => Fixed::from_f64(0.55),
            Self::Betrothal => Fixed::from_f64(0.65),
        }
    }

    /// Minimum attraction required to enter this stage.
    pub fn attraction_threshold(&self) -> Fixed {
        match self {
            Self::Unaware => Fixed::ZERO,
            Self::Awareness => Fixed::from_f64(0.1),
            Self::Attraction => Fixed::from_f64(0.2),
            Self::Flirtation => Fixed::from_f64(0.3),
            Self::ActiveCourtship => Fixed::from_f64(0.4),
            Self::TestingReciprocity => Fixed::from_f64(0.45),
            Self::SocialValidation => Fixed::from_f64(0.5),
            Self::Exclusivity => Fixed::from_f64(0.55),
            Self::Betrothal => Fixed::from_f64(0.6),
        }
    }
}

/// A courtship in progress between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Courtship {
    /// Index of agent pursuing (may be either direction).
    pub pursuer: usize,
    /// Index of agent being pursued.
    pub pursued: usize,
    /// Current courtship stage.
    pub stage: CourtshipStage,
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
            stage: CourtshipStage::Unaware,
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
    fn try_advance(&mut self, trust: Fixed, attraction: Fixed) {
        if let Some(next) = self.stage.next() {
            if trust >= next.trust_threshold()
                && attraction >= next.attraction_threshold()
                && self.positive_interactions > self.negative_interactions
            {
                self.stage = next;
            }
        }
    }

    /// Check if stage regression conditions are met.
    fn try_regress(&mut self) {
        if self.negative_interactions > self.positive_interactions {
            if let Some(prev) = self.stage.regress() {
                self.stage = prev;
            } else {
                // Courtship failed completely
                self.active = false;
            }
        }
    }

    /// Courtship probability of advancing per tick — depends on reciprocity and approval.
    pub fn advance_probability(&self) -> Fixed {
        let base = Fixed::from_f64(0.01);
        let reciprocity_bonus = self.reciprocity * Fixed::from_f64(0.02);
        let approval_bonus = self.social_approval * Fixed::from_f64(0.01);
        (base + reciprocity_bonus + approval_bonus).clamp_01()
    }

    /// Courtship probability of regression per tick — depends on negative interactions.
    pub fn regression_probability(&self) -> Fixed {
        let negatives = Fixed::from_int(self.negative_interactions as i64);
        let positives = Fixed::from_int(self.positive_interactions as i64);
        let ratio = if negatives + positives > Fixed::ZERO {
            negatives / (negatives + positives)
        } else {
            Fixed::ZERO
        };
        ratio * Fixed::from_f64(0.03)
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
        assert!(courtship.stage >= CourtshipStage::Awareness);
    }

    #[test]
    fn courtship_stage_regresses_on_negatives() {
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = CourtshipStage::Flirtation;
        // Multiple negatives should regress
        for i in 0..5 {
            courtship.record_negative(10 + i);
        }
        assert!(courtship.stage < CourtshipStage::Flirtation);
    }

    #[test]
    fn courtship_fails_after_enough_negatives() {
        let mut courtship = Courtship::new(0, 1, 0);
        courtship.stage = CourtshipStage::Attraction;
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
}
