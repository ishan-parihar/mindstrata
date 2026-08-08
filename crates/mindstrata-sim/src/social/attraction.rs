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
            + self.social_cost * Fixed::from_f64(0.1);
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
        let mut a = AttractionModel { physical_attraction: Fixed::from_f64(0.8), ..AttractionModel::default() };
        let without = a.total_attraction();
        a.kinship_penalty = Fixed::from_f64(0.9);
        assert!(a.total_attraction() < without);
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
}

