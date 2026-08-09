//! Status system — multi-dimensional power and social standing.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Multi-dimensional status state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDimensions {
    pub dominance: Fixed,
    pub prestige: Fixed,
    pub authority: Fixed,
    pub legitimacy: Fixed,
    pub wealth_rank: Fixed,
    pub moral_reputation: Fixed,
    pub network_centrality: Fixed,
    /// §11.1 (AP2): Power through institutional role (0 = none, 1 = top role).
    /// `#[serde(default)]` keeps old saves without this field restorable.
    #[serde(default)]
    pub institutional_rank: Fixed,
    pub honor: Fixed,
    pub shame: Fixed,
}

impl Default for StatusDimensions {
    fn default() -> Self {
        Self {
            dominance: Fixed::from_f64(0.3),
            prestige: Fixed::from_f64(0.3),
            authority: Fixed::ZERO,
            legitimacy: Fixed::from_f64(0.5),
            wealth_rank: Fixed::from_f64(0.3),
            moral_reputation: Fixed::from_f64(0.5),
            network_centrality: Fixed::from_f64(0.3),
            institutional_rank: Fixed::ZERO,
            honor: Fixed::from_f64(0.5),
            shame: Fixed::ZERO,
        }
    }
}

impl StatusDimensions {
    pub fn effective_status(&self) -> Fixed {
        // §11.1 (Iteration 106): institutional_rank is a first-class status
        // component (0 = none, 1 = top role). Weighted at 0.1 — equal to
        // wealth_rank, below the core three (dominance/prestige/authority) —
        // so a role-holder's composite genuinely reflects their institutional
        // power. The term is exactly zero for roleless agents (rank is zero),
        // so their composite is unchanged; the positive weights sum to 1.05
        // with the new term and clamp at 1.0 for saturated status. Note:
        // institutional_rank overlaps with `authority` (synced from legacy
        // role_status, also set for role-holders) — the two are distinct
        // sources (registry-derived authority vs legacy role status), and
        // §11.1 lists both as separate components, so the overlap is
        // plan-faithful; the 0.1 weight keeps the double-count modest.
        (self.dominance * Fixed::from_f64(0.15)
            + self.prestige * Fixed::from_f64(0.2)
            + self.authority * Fixed::from_f64(0.2)
            + self.legitimacy * Fixed::from_f64(0.15)
            + self.wealth_rank * Fixed::from_f64(0.1)
            + self.moral_reputation * Fixed::from_f64(0.1)
            + self.honor * Fixed::from_f64(0.05)
            + self.institutional_rank * Fixed::from_f64(0.1)
            - self.shame * Fixed::from_f64(0.1))
            .clamp_01()
    }

    pub fn record_victory(&mut self, magnitude: Fixed) {
        self.dominance = (self.dominance + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.prestige = (self.prestige + magnitude * Fixed::from_f64(0.01)).clamp_01();
        self.honor = (self.honor + magnitude * Fixed::from_f64(0.01)).clamp_01();
    }

    pub fn record_defeat(&mut self, magnitude: Fixed) {
        self.dominance = (self.dominance - magnitude * Fixed::from_f64(0.03)).max(Fixed::ZERO);
        self.shame = (self.shame + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.honor = (self.honor - magnitude * Fixed::from_f64(0.01)).max(Fixed::ZERO);
    }

    pub fn decay(&mut self) {
        self.shame = (self.shame - Fixed::from_f64(0.001)).max(Fixed::ZERO);
        self.honor = (self.honor - Fixed::from_f64(0.0005)).max(Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_status_computed() {
        let s = StatusDimensions::default();
        let eff = s.effective_status();
        assert!(eff >= Fixed::ZERO && eff <= Fixed::ONE);
    }

    #[test]
    fn victory_increases_dominance() {
        let mut s = StatusDimensions::default();
        let initial = s.dominance;
        s.record_victory(Fixed::from_f64(0.5));
        assert!(s.dominance > initial);
    }

    #[test]
    fn default_has_zero_institutional_rank() {
        // §11.1: a plain agent with no institutional role starts at rank zero.
        let s = StatusDimensions::default();
        assert_eq!(s.institutional_rank, Fixed::ZERO);
    }

    #[test]
    fn institutional_rank_serde_roundtrip() {
        let s = StatusDimensions {
            institutional_rank: Fixed::from_f64(0.9),
            ..Default::default()
        };
        let json = serde_json::to_string(&s).unwrap();
        let restored: StatusDimensions = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.institutional_rank, s.institutional_rank);
        assert_eq!(restored.effective_status(), s.effective_status());
    }

    #[test]
    fn old_save_without_institutional_rank_restores_zero() {
        // A save from before the §11.1 field existed must deserialize to zero.
        // Fixed serializes as raw scaled i64 (SCALE = 10_000).
        let old_json = r#"{
            "dominance": 3000,
            "prestige": 3000,
            "authority": 0,
            "legitimacy": 5000,
            "wealth_rank": 3000,
            "moral_reputation": 5000,
            "network_centrality": 3000,
            "honor": 5000,
            "shame": 0
        }"#;
        let restored: StatusDimensions = serde_json::from_str(old_json).unwrap();
        assert_eq!(restored.institutional_rank, Fixed::ZERO);
    }

    #[test]
    fn effective_status_weights_institutional_rank() {
        // §11.1 (Iteration 106): the legacy 8-dimension composite is the
        // reference — a roleless agent (rank 0) must be byte-identical to it,
        // and a role-holder must exceed it by exactly rank × 0.1 (0.6 → +0.06
        // here, before any clamp; the arithmetic is exact at SCALE = 10_000).
        let legacy = |s: &StatusDimensions| {
            (s.dominance * Fixed::from_f64(0.15)
                + s.prestige * Fixed::from_f64(0.2)
                + s.authority * Fixed::from_f64(0.2)
                + s.legitimacy * Fixed::from_f64(0.15)
                + s.wealth_rank * Fixed::from_f64(0.1)
                + s.moral_reputation * Fixed::from_f64(0.1)
                + s.honor * Fixed::from_f64(0.05)
                - s.shame * Fixed::from_f64(0.1))
                .clamp_01()
        };
        let base = StatusDimensions::default();
        assert_eq!(base.effective_status(), legacy(&base), "rankless agent must match legacy");
        let holder = StatusDimensions {
            institutional_rank: Fixed::from_f64(0.6),
            ..StatusDimensions::default()
        };
        let expected = (legacy(&holder) + Fixed::from_f64(0.06)).clamp_01();
        assert_eq!(holder.effective_status(), expected, "role-holder must gain rank × 0.1");
        // Saturated status clamps at 1.0 even with the new term.
        let maxed = StatusDimensions {
            dominance: Fixed::ONE,
            prestige: Fixed::ONE,
            authority: Fixed::ONE,
            legitimacy: Fixed::ONE,
            wealth_rank: Fixed::ONE,
            moral_reputation: Fixed::ONE,
            network_centrality: Fixed::ONE,
            honor: Fixed::ONE,
            institutional_rank: Fixed::ONE,
            shame: Fixed::ZERO,
        };
        assert_eq!(maxed.effective_status(), Fixed::ONE);
    }

    #[test]
    fn all_ten_plan_components_present() {
        // §11.1 plan lists ten components; the struct must expose each.
        let s = StatusDimensions::default();
        let _ = [
            s.dominance,
            s.prestige,
            s.authority,
            s.legitimacy,
            s.wealth_rank,
            s.moral_reputation,
            s.network_centrality,
            s.institutional_rank,
            s.honor,
            s.shame,
        ];
    }
}
