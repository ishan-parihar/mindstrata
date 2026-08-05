//! Relational Power (§11.2) — asymmetric power inside each relationship.
//!
//! AP2 §11.2: relationships are never free of power. Power balance is computed
//! from dependence, resource control, emotional leverage, social leverage,
//! coercive capacity, and the target's alternatives. The computed balance fills
//! `RelationshipV2.power_balance`, which was declared but never updated — the
//! classic declared-but-dead-field pattern this iteration closes.
//!
//! Zero-drift design: `power_balance` is write-only today (nothing downstream
//! reads it; it is not in `agent_summaries()`), so populating it cannot perturb
//! any calibrated run, snapshot, or golden baseline. The computation is a pure
//! function of existing relationship + status state — no RNG, fully
//! deterministic.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// §11.2: The power structure of one directed relationship (A → B).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RelationalPower {
    /// How much A depends on B (0 = independent, 1 = fully dependent).
    pub dependence_a_on_b: Fixed,
    /// How much B depends on A (0 = independent, 1 = fully dependent).
    pub dependence_b_on_a: Fixed,
    /// A's control over resources B needs.
    pub resource_control: Fixed,
    /// A's emotional leverage over B (who cares less has more).
    pub emotional_leverage: Fixed,
    /// A's social leverage over B (reputation, network position).
    pub social_leverage: Fixed,
    /// A's capacity to coerce B (fear, dominance).
    pub coercive_capacity: Fixed,
    /// Moral obligation B owes A (debt, gratitude, guilt).
    pub moral_obligation: Fixed,
    /// B's alternatives outside A (0 = none, 1 = many).
    pub alternative_options: Fixed,
}

impl Default for RelationalPower {
    fn default() -> Self {
        Self {
            dependence_a_on_b: Fixed::ZERO,
            dependence_b_on_a: Fixed::ZERO,
            resource_control: Fixed::ZERO,
            emotional_leverage: Fixed::ZERO,
            social_leverage: Fixed::ZERO,
            coercive_capacity: Fixed::ZERO,
            moral_obligation: Fixed::ZERO,
            alternative_options: Fixed::from_f64(0.5),
        }
    }
}

impl RelationalPower {
    /// Compute the power structure of the directed relationship A → B.
    ///
    /// `dependence_b_on_a` (how much B needs A) is derived from the strength of
    /// B's bonds toward A: the more commitment/attachment B holds, the more B
    /// depends. `status_advantage` is A's effective status minus B's, in
    /// [-1, 1], feeding resource control and coercive capacity.
    pub fn compute(
        dependence_a_on_b: Fixed,
        commitment_b_to_a: Fixed,
        attachment_b_to_a: Fixed,
        status_advantage: Fixed,
        fear_b_of_a: Fixed,
        obligation_b_to_a: Fixed,
        moral_debt_b_to_a: Fixed,
    ) -> Self {
        let status_advantage = status_advantage.clamp(Fixed::from_f64(-1.0), Fixed::ONE);
        let dependence_b_on_a = (commitment_b_to_a * Fixed::from_f64(0.6)
            + attachment_b_to_a * Fixed::from_f64(0.4))
            .clamp_01();
        Self {
            dependence_a_on_b,
            dependence_b_on_a,
            // Resource control scales with status advantage (positive half only —
            // A dominates resources B needs when A is higher status).
            resource_control: status_advantage.max(Fixed::ZERO),
            // Emotional leverage: the less A depends on B, the more leverage A holds.
            emotional_leverage: (Fixed::ONE - dependence_a_on_b) * Fixed::from_f64(0.7),
            social_leverage: status_advantage.max(Fixed::ZERO) * Fixed::from_f64(0.5)
                + dependence_b_on_a * Fixed::from_f64(0.3),
            coercive_capacity: fear_b_of_a * Fixed::from_f64(0.5)
                + status_advantage.max(Fixed::ZERO) * Fixed::from_f64(0.3),
            moral_obligation: (obligation_b_to_a * Fixed::from_f64(0.5)
                + moral_debt_b_to_a * Fixed::from_f64(0.5))
                .clamp_01(),
            // Alternatives = inverse of B's dependence on A.
            alternative_options: (Fixed::ONE - dependence_b_on_a).clamp_01(),
        }
    }

    /// §11.2 formula:
    /// ```text
    /// power_balance = (resource_control + social_leverage
    ///                + coercive_capacity + emotional_leverage) − alternatives
    /// ```
    /// Positive means A dominates B; negative means B dominates A. Clamped to
    /// [-1, 1] so it is directly comparable across relationships.
    #[must_use]
    pub fn power_balance(&self) -> Fixed {
        let a_power = self.resource_control + self.social_leverage + self.coercive_capacity
            + self.emotional_leverage;
        (a_power - self.alternative_options).clamp(Fixed::from_f64(-1.0), Fixed::ONE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn equal_status_produces_balanced_power() {
        let p = RelationalPower::compute(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
        );
        // Symmetric inputs → exactly balanced: A_power = 0 (resource) + 0.15
        // (social) + 0 (coercive) + 0.35 (emotional) = 0.5; alternatives = 0.5;
        // balance = 0.0 exactly. Equal dependence ⇒ no power differential.
        assert_eq!(p.power_balance(), Fixed::ZERO);
    }

    #[test]
    fn high_status_dominates_low_status() {
        let p = RelationalPower::compute(
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.5),
        );
        assert!(p.power_balance() > Fixed::from_f64(0.5));
        assert!(p.dependence_b_on_a > p.dependence_a_on_b);
        assert!(p.resource_control > Fixed::ZERO);
        assert!(p.coercive_capacity > Fixed::ZERO);
    }

    #[test]
    fn low_status_is_dominated() {
        let p = RelationalPower::compute(
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.2),
            Fixed::from_f64(-0.8),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
        );
        assert!(p.power_balance() < Fixed::ZERO);
        assert_eq!(p.resource_control, Fixed::ZERO);
        assert!(p.alternative_options > Fixed::from_f64(0.7));
    }

    #[test]
    fn dependence_reduces_leverage() {
        // A deeply dependent on B → low emotional leverage, negative balance.
        let p = RelationalPower::compute(
            Fixed::from_f64(0.95),
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.1),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
        );
        assert!(p.emotional_leverage < Fixed::from_f64(0.1));
        assert!(p.power_balance() < Fixed::ZERO);
    }

    #[test]
    fn moral_obligation_accumulates() {
        let p = RelationalPower::compute(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::ONE,
        );
        assert_eq!(p.moral_obligation, Fixed::ONE);
    }
}
