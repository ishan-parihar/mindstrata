//! Attraction system — multi-factor mate selection model.

use mindstrata_core::event::InteractionKind;
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Attraction model for evaluating potential partners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractionModel {
    pub physical_attraction: Fixed,
    pub personality_attraction: Fixed,
    pub status_attraction: Fixed,
    pub moral_attraction: Fixed,
    pub familiarity: Fixed,
    pub reciprocity: Fixed,
    pub social_approval: Fixed,
    pub kinship_penalty: Fixed,
    pub moral_disgust: Fixed,
    pub social_cost: Fixed,
    /// §8.1.4 (Iteration 123): the envy aversion channel — coveting what the
    /// higher-status other has (`status_threat × incongruent`) poisons
    /// courtship interest: the envious mind is consumed with status
    /// comparison, so romantic salience drops. Weighs 0.2 (negative) in
    /// `total_attraction()` — the same weight tier as `moral_disgust` (both
    /// strong aversive emotions). Deterministic, no RNG, clamped. 0 in calm
    /// worlds (envy is never produced there — probe-pinned, see the sim.rs
    /// fold site). Note: at full envy the 0.2 removal can flip a borderline
    /// profile (e.g. 0.45) under the 0.4 D4 courtship gate — fully
    /// suppressing courtship in genuinely envious worlds. That is the
    /// intended "poisons courtship interest" effect, matching the disgust
    /// tier, and the const-style weight is trivially tunable.
    #[serde(default)]
    pub envy_cost: Fixed,
    /// §8.1.18/§10.4 (Iteration 165): the taboo restraint channel — the
    /// strongest taboo the agent's cultural cognition holds (`CulturalCognition::max_taboo_strength`).
    /// An agent whose culture forbids more courts more hesitantly: the
    /// internalized prohibition raises the cost of romantic pursuit for the
    /// *pursuer*, independent of any particular target. Weighs 0.1 (negative)
    /// in `total_attraction()` — the same modest tier as `social_cost` (both
    /// standing cultural/social costs rather than acute emotions). This is
    /// the RWR row-#11 `taboo_penalty` consumer: the last unwired §10.4
    /// attraction channel, and the first production consumer of the §8.1.18
    /// taboo layer (previously write-only dead code). Deterministic, no RNG,
    /// clamped; identity-at-zero for agents holding no taboos (pre-Iter-165
    /// snapshots). Sized small so the D4 courtship gate (0.4) stays
    /// reachable for low-taboo agents while genuinely differentiated profiles
    /// feel the restraint.
    #[serde(default)]
    pub taboo_penalty: Fixed,
    pub attachment_resonance: Fixed,
}

impl Default for AttractionModel {
    fn default() -> Self {
        Self {
            physical_attraction: Fixed::from_f64(0.5),
            personality_attraction: Fixed::from_f64(0.5),
            status_attraction: Fixed::ZERO,
            moral_attraction: Fixed::from_f64(0.5),
            familiarity: Fixed::ZERO,
            reciprocity: Fixed::ZERO,
            social_approval: Fixed::from_f64(0.5),
            kinship_penalty: Fixed::ZERO,
            moral_disgust: Fixed::ZERO,
            social_cost: Fixed::ZERO,
            envy_cost: Fixed::ZERO,
            taboo_penalty: Fixed::ZERO,
            attachment_resonance: Fixed::from_f64(0.5),
        }
    }
}

impl AttractionModel {
    pub fn total_attraction(&self) -> Fixed {
        let positive = self.physical_attraction * Fixed::from_f64(0.15)
            + self.personality_attraction * Fixed::from_f64(0.2)
            + self.status_attraction * Fixed::from_f64(0.1)
            + self.moral_attraction * Fixed::from_f64(0.1)
            + self.familiarity * Fixed::from_f64(0.1)
            + self.reciprocity * Fixed::from_f64(0.15)
            + self.social_approval * Fixed::from_f64(0.05)
            + self.attachment_resonance * Fixed::from_f64(0.1);
        let negative = self.kinship_penalty * Fixed::from_f64(0.3)
            + self.moral_disgust * Fixed::from_f64(0.2)
            + self.social_cost * Fixed::from_f64(0.1)
            + self.envy_cost * Fixed::from_f64(0.2)
            + self.taboo_penalty * Fixed::from_f64(0.1);
        (positive - negative).clamp_01()
    }

    pub fn update_familiarity(&mut self, interaction_quality: Fixed) {
        self.familiarity = (self.familiarity + interaction_quality * Fixed::from_f64(0.01))
            .min(Fixed::from_f64(0.8));
    }

    /// §10.4 (Iteration 77): Reflect the agent's current social standing in
    /// their attraction profile. `status_attraction` was a declared plan field
    /// with zero production writers — the last of the three attraction factors
    /// (with familiarity and reciprocity) that could push `total_attraction()`
    /// over the §8.1.16 D4 courtship gate (0.4). The agent's own composite
    /// standing (`StatusDimensions::effective_status`) contributes to the
    /// standing they bring to a match. Deterministic, no RNG, clamped [0, 1].
    pub fn update_status_attraction(&mut self, effective_status: Fixed) {
        self.status_attraction = effective_status.clamp_01();
    }

    /// §10.4 (Iteration 78): The agent's criminal notoriety is a social cost
    /// on any match — repeat offenders are costly partners. Wired from the
    /// live norm-enforcement crime records (`CrimeRecord.notoriety`); this
    /// channel was a declared plan field with zero production writers. Weighs
    /// 0.1 (negative) in `total_attraction()`. Deterministic, no RNG, clamped.
    pub fn update_social_cost(&mut self, notoriety: Fixed) {
        self.social_cost = notoriety.clamp_01();
    }

    /// §10.4 (Iteration 78): The agent's aversion to taboo/purity violations.
    /// Wired from the disgust emotion (which the §8.1.4 appraisal computes as
    /// purity_violation × goal_relevance) — 0 in peaceful runs by design (the
    /// same situational class as the Iter-65 jealousy channel), firing when
    /// the agent actually appraises a moral violation. Weighs 0.2 (negative)
    /// in `total_attraction()`. Deterministic, no RNG, clamped.
    /// §10.4 (Iteration 79): The agent's kinship taboo against courting
    /// within the local pool — the max transitive genetic relatedness to any
    /// adult partner, derived from the live kinship graph. 0 for founders
    /// (no kin ties), rising as families form: parent/sibling ties read 0.5,
    /// grandparent 0.25, first cousin 0.125 (the plan's §10.6 BFS model, which
    /// the direct-edge 0.25 hard eligibility gate does not catch — the soft
    /// channel is the complement that makes kin courtships less attractive
    /// without hard-blocking them). Weighs 0.3 (negative) in
    /// `total_attraction()`. Deterministic, no RNG, clamped.
    pub fn update_kinship_penalty(&mut self, max_relatedness: Fixed) {
        self.kinship_penalty = max_relatedness.clamp_01();
    }

    pub fn update_moral_disgust(&mut self, disgust: Fixed) {
        self.moral_disgust = disgust.clamp_01();
    }

    /// §8.1.18/§10.4 (Iteration 165): mirror the agent's strongest internalized
    /// taboo (the §8.1.18 `CulturalCognition` layer) into the courtship-cost
    /// channel. Deterministic, no RNG, clamped. Identity at zero: an agent
    /// with no taboos (or a pre-Iter-165 snapshot restore) keeps the channel
    /// exactly 0, so `total_attraction()` is unchanged for taboo-free agents.
    pub fn update_taboo_penalty(&mut self, max_taboo_strength: Fixed) {
        self.taboo_penalty = max_taboo_strength.clamp_01();
    }

    /// §8.1.4 (Iteration 123): mirror the live `emotions.envy` into the
    /// courtship-cost channel. Identity at zero: envy is never produced in
    /// calibrated windows, so the channel stays exactly 0 (zero-blast).
    pub fn update_envy_cost(&mut self, envy: Fixed) {
        self.envy_cost = envy.clamp_01();
    }
}

/// §10.4: Familiarity-growth quality for an interaction kind.
///
/// Deep, prosocial interactions (help, comfort, teaching) build familiarity
/// most; mundane exchanges (talk, gossip, trade) build some; hostile acts
/// (threaten, insult) build almost none — a threat makes you more aware of
/// someone, but that awareness is not the warm familiarity that feeds
/// attraction. The qualities sit one order of magnitude below the 0.3–0.6
/// raw scale so that, through `update_familiarity`'s 0.01-per-interaction
/// scaling, familiarity accumulates as a *slow, discriminating* signal
/// (probe: 0.3–0.6 qualities saturated every agent at the 0.8 cap by tick
/// 1000 in both seeds, erasing all differentiation — the same dead-signal
/// problem this wiring exists to fix).
pub fn familiarity_quality_for(kind: InteractionKind) -> Fixed {
    match kind {
        InteractionKind::Help => Fixed::from_f64(0.06),
        InteractionKind::Comfort => Fixed::from_f64(0.05),
        InteractionKind::Teach => Fixed::from_f64(0.05),
        InteractionKind::Trade => Fixed::from_f64(0.04),
        InteractionKind::Talk => Fixed::from_f64(0.03),
        InteractionKind::Gossip => Fixed::from_f64(0.03),
        InteractionKind::Threaten => Fixed::from_f64(0.005),
        InteractionKind::Insult => Fixed::from_f64(0.005),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_attraction_computed() {
        let a = AttractionModel {
            physical_attraction: Fixed::from_f64(0.8),
            personality_attraction: Fixed::from_f64(0.7),
            ..AttractionModel::default()
        };
        assert!(a.total_attraction() > Fixed::from_f64(0.3));
    }

    #[test]
    fn kinship_penalty_reduces_attraction() {
        let mut a = AttractionModel {
            physical_attraction: Fixed::from_f64(0.8),
            ..AttractionModel::default()
        };
        let without = a.total_attraction();
        a.kinship_penalty = Fixed::from_f64(0.9);
        assert!(a.total_attraction() < without);
    }

    #[test]
    fn update_kinship_penalty_mirrors_relatedness() {
        let mut a = AttractionModel::default();
        a.update_kinship_penalty(Fixed::from_f64(0.125)); // first cousin
        assert_eq!(a.kinship_penalty, Fixed::from_f64(0.125));
        let before = a.total_attraction();
        a.update_kinship_penalty(Fixed::from_f64(0.5)); // parent/sibling
        assert!(a.total_attraction() < before);
    }

    #[test]
    fn update_kinship_penalty_clamps_to_unit() {
        let mut a = AttractionModel::default();
        a.update_kinship_penalty(Fixed::from_f64(1.5));
        assert_eq!(a.kinship_penalty, Fixed::ONE);
        a.update_kinship_penalty(Fixed::from_f64(-0.5));
        assert_eq!(a.kinship_penalty, Fixed::ZERO);
    }

    #[test]
    fn envy_cost_reduces_attraction() {
        // §8.1.4 (Iteration 123): a fully envious profile (envy_cost 1.0)
        // at the 0.2 weight removes exactly 0.2 from total_attraction — the
        // coveting emotion depresses courtship interest like moral_disgust.
        let mut a = AttractionModel {
            physical_attraction: Fixed::from_f64(0.8),
            ..AttractionModel::default()
        };
        let without = a.total_attraction();
        a.envy_cost = Fixed::ONE;
        assert!(a.total_attraction() < without);
        assert_eq!(
            without - a.total_attraction(),
            Fixed::from_f64(0.2),
            "envy 1.0 × weight 0.2 must remove exactly 0.2"
        );
    }

    #[test]
    fn update_envy_cost_mirrors_envy_and_clamps() {
        let mut a = AttractionModel::default();
        a.update_envy_cost(Fixed::from_f64(0.5));
        assert_eq!(a.envy_cost, Fixed::from_f64(0.5));
        let before = a.total_attraction();
        a.update_envy_cost(Fixed::ONE);
        assert!(a.total_attraction() < before);
        // Clamp bounds: out-of-range inputs collapse to the unit interval.
        a.update_envy_cost(Fixed::from_f64(1.5));
        assert_eq!(a.envy_cost, Fixed::ONE);
        a.update_envy_cost(Fixed::from_f64(-0.5));
        assert_eq!(a.envy_cost, Fixed::ZERO);
        // Identity at zero: the default model's channel is inert.
        let mut b = AttractionModel::default();
        b.update_envy_cost(Fixed::ZERO);
        assert_eq!(b.envy_cost, Fixed::ZERO);
    }

    #[test]
    fn cousin_relatedness_depresses_attraction_below_gate_math() {
        // A first-cousin tie (0.125) at the 0.3 weight removes 0.0375 from
        // total_attraction — enough to drop a borderline profile under the
        // 0.4 D4 gate (0.42 base → 0.3825), the soft-taboo complement to the
        // direct-edge hard gate which does not catch cousins.
        let mut a = AttractionModel {
            status_attraction: Fixed::from_f64(0.4),
            familiarity: Fixed::from_f64(0.8),
            ..AttractionModel::default()
        };
        let without = a.total_attraction();
        assert!(without > Fixed::from_f64(0.4)); // D4 reachable pre-kinship
        a.update_kinship_penalty(Fixed::from_f64(0.125));
        assert!(a.total_attraction() < Fixed::from_f64(0.4)); // taboo applies
    }

    #[test]
    fn status_attraction_raises_total_attraction() {
        let mut a = AttractionModel::default();
        let without = a.total_attraction();
        a.update_status_attraction(Fixed::from_f64(0.4));
        // Status carries weight 0.1 in total_attraction().
        assert!(a.total_attraction() > without + Fixed::from_f64(0.03));
        assert!(a.total_attraction() < without + Fixed::from_f64(0.05));
    }

    #[test]
    fn status_attraction_clamps_to_unit() {
        let mut a = AttractionModel::default();
        a.update_status_attraction(Fixed::from_f64(1.5));
        assert_eq!(a.status_attraction, Fixed::ONE);
        a.update_status_attraction(Fixed::from_f64(-0.5));
        assert_eq!(a.status_attraction, Fixed::ZERO);
    }

    #[test]
    fn social_cost_reduces_total_attraction() {
        let mut a = AttractionModel::default();
        let without = a.total_attraction();
        a.update_social_cost(Fixed::from_f64(0.5));
        // Social cost carries weight 0.1 (negative) in total_attraction().
        assert!(a.total_attraction() < without - Fixed::from_f64(0.04));
        assert!(a.total_attraction() > without - Fixed::from_f64(0.06));
        a.update_social_cost(Fixed::from_f64(-0.5));
        assert_eq!(a.total_attraction(), without);
    }

    #[test]
    fn moral_disgust_reduces_total_attraction() {
        let mut a = AttractionModel::default();
        let without = a.total_attraction();
        a.update_moral_disgust(Fixed::from_f64(0.5));
        // Moral disgust carries weight 0.2 (negative) in total_attraction().
        assert!(a.total_attraction() < without - Fixed::from_f64(0.09));
        assert!(a.total_attraction() > without - Fixed::from_f64(0.11));
        a.update_moral_disgust(Fixed::from_f64(0.0));
        assert_eq!(a.total_attraction(), without);
    }

    #[test]
    fn high_status_plus_familiarity_crosses_d4_gate() {
        // The §8.1.16 D4 gate (total_attraction > 0.4) is reachable with
        // status + familiarity alone — no reciprocity needed. Models the live
        // condition: familiarity saturated at the 0.8 cap (probe: sustained
        // interaction in riverford reaches the cap) plus an effective_status
        // of 0.4 (probe: role-holding agents reach 0.4). Base total is 0.30
        // (0.5 physical/personality/moral/attachment/social-approval at their
        // weights), +0.08 familiarity +0.04 status = 0.42 > 0.4. This pins
        // the Iteration-77 claim that wiring status_attraction closes the
        // last D4 factor.
        let mut a = AttractionModel::default();
        for _ in 0..1400 {
            a.update_familiarity(familiarity_quality_for(InteractionKind::Help));
        }
        assert_eq!(a.familiarity, Fixed::from_f64(0.8)); // saturated at cap
        a.update_status_attraction(Fixed::from_f64(0.4));
        assert!(a.total_attraction() > Fixed::from_f64(0.4));
    }

    #[test]
    fn familiarity_accumulates_from_repeated_interaction() {
        let mut a = AttractionModel::default();
        assert_eq!(a.familiarity, Fixed::ZERO);
        // 100 Help-grade interactions (quality 0.06 → +0.0006 each) — the
        // slow, discriminating accumulation the plan calls "familiarity grows
        // with repeated interaction".
        for _ in 0..100 {
            a.update_familiarity(familiarity_quality_for(InteractionKind::Help));
        }
        assert!(a.familiarity > Fixed::from_f64(0.05));
    }

    #[test]
    fn familiarity_caps_at_eight_tenths() {
        let mut a = AttractionModel::default();
        // 20,000 Help-grade interactions would exceed the 0.8 cap without
        // the `.min()` guard; the cap must hold.
        for _ in 0..20_000 {
            a.update_familiarity(familiarity_quality_for(InteractionKind::Help));
        }
        assert_eq!(a.familiarity, Fixed::from_f64(0.8));
    }

    #[test]
    fn hostile_interactions_build_little_familiarity() {
        let mut a = AttractionModel::default();
        for _ in 0..10 {
            a.update_familiarity(familiarity_quality_for(InteractionKind::Insult));
        }
        let hostile = a.familiarity;
        let mut b = AttractionModel::default();
        for _ in 0..10 {
            b.update_familiarity(familiarity_quality_for(InteractionKind::Comfort));
        }
        assert!(hostile < b.familiarity);
    }

    #[test]
    fn familiarity_feeds_total_attraction() {
        let mut a = AttractionModel::default();
        let base = a.total_attraction();
        a.familiarity = Fixed::from_f64(0.8);
        assert!(a.total_attraction() > base);
    }

    /// §8.1.18/§10.4 (Iteration 165): `taboo_penalty` — the RWR row-#11
    /// consumer. A full taboo restraint (1.0) at the 0.1 weight removes
    /// exactly 0.1 from `total_attraction()` (mirroring the social_cost
    /// tier); the channel mirrors the seeded strength and clamps.
    #[test]
    fn taboo_penalty_reduces_total_attraction() {
        let mut a = AttractionModel::default();
        let without = a.total_attraction();
        a.update_taboo_penalty(Fixed::ONE);
        assert!(
            a.total_attraction() < without,
            "taboo restraint must depress courtship interest"
        );
        assert_eq!(
            without - a.total_attraction(),
            Fixed::from_f64(0.1),
            "taboo_penalty 1.0 × weight 0.1 must remove exactly 0.1"
        );
    }

    /// §8.1.18/§10.4 (Iteration 165): `update_taboo_penalty` mirrors the
    /// max taboo strength, clamps out-of-range inputs, and is identity at
    /// zero (a taboo-free agent's channel stays exactly 0).
    #[test]
    fn update_taboo_penalty_mirrors_and_clamps() {
        let mut a = AttractionModel::default();
        a.update_taboo_penalty(Fixed::from_f64(0.525));
        assert_eq!(a.taboo_penalty, Fixed::from_f64(0.525));
        // Clamp bounds collapse to the unit interval.
        a.update_taboo_penalty(Fixed::from_f64(1.5));
        assert_eq!(a.taboo_penalty, Fixed::ONE);
        a.update_taboo_penalty(Fixed::from_f64(-0.5));
        assert_eq!(a.taboo_penalty, Fixed::ZERO);
        // Identity at zero: the default model's channel is inert — a zero
        // write leaves total_attraction exactly equal to a fresh default.
        let b = AttractionModel::default();
        let base = b.total_attraction();
        let mut c = AttractionModel::default();
        c.update_taboo_penalty(Fixed::ZERO);
        assert_eq!(c.taboo_penalty, Fixed::ZERO);
        assert_eq!(c.total_attraction(), base);
    }

    /// §8.1.18/§10.4 (Iteration 165): a mid-traditional profile (taboo
    /// strength 0.525 → 0.0525 removal at the 0.1 weight) EXACTLY mirrors the
    /// seeded strength's restraint — the channel is mathematically the
    /// documented weight × strength, so a borderline profile feels the
    /// restraint while a strong one (high familiarity + status + reciprocity)
    /// still clears the D4 gate (probe-pinned: max total_attraction crosses
    /// 0.4 in 6/10 seeds at 5000 ticks with the shipped sizing).
    #[test]
    fn taboo_restraint_removes_exact_weighted_strength() {
        let mut a = AttractionModel::default();
        // A mid profile: default base 0.30 + familiarity at the 0.8 cap.
        for _ in 0..1400 {
            a.update_familiarity(familiarity_quality_for(InteractionKind::Help));
        }
        a.update_status_attraction(Fixed::from_f64(0.4));
        let before = a.total_attraction();
        assert!(before > Fixed::from_f64(0.4)); // D4 reachable pre-restraint
        a.update_taboo_penalty(Fixed::from_f64(0.525));
        // The restraint is exactly weight × strength: 0.1 × 0.525 = 0.0525.
        assert_eq!(before - a.total_attraction(), Fixed::from_f64(0.0525));
        // A strong profile (reciprocity + attachment) keeps the gate open.
        a.reciprocity = Fixed::from_f64(0.6);
        a.attachment_resonance = Fixed::from_f64(0.8);
        assert!(
            a.total_attraction() > Fixed::from_f64(0.4),
            "a strong profile must clear the D4 gate even under the restraint"
        );
    }
}
