//! §10.1: Three Relational Fields.
//!
//! When agents enter one another's vicinity, three layers activate:
//!
//! - **§10.1.1 Sensory field** — what the agent can perceive: proximity,
//!   expression (perceived emotional stress of neighbors), environmental cues.
//! - **§10.1.2 Social field** — the agent's relationship graph summary:
//!   trust, obligation, kinship, peer status.
//! - **§10.1.3 Noospheric field** — the shared symbolic space: beliefs,
//!   legitimacy, collective emotions.
//!
//! The per-agent snapshot is refreshed each tick (see
//! `Simulation::refresh_relational_fields`). It is deterministic
//! (index-order iteration, no RNG). Since Iteration 107 the sensory
//! field's `perceived_stress` has a decisional consumer: the daily fear
//! contagion fold (`contagion_delta`) at the emotion-decay site — an
//! agent's fear rises with the ambient stress it perceives. The social
//! (trust/obligation/kin/peer-status) and noospheric (beliefs,
//! legitimacy, collective fear) layers remain observational.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Perception radius (manhattan tiles) for the sensory field.
pub const PERCEPTION_RADIUS: i32 = 5;

/// Daily fear-contagion rate (§10.1.1, Iteration 107): the fraction of the
/// perceived ambient stress added to the agent's fear each day. Kept low so
/// the annual ambient uplift (~rate × stress × 365) stays bounded against
/// the §22.1 per-tick decay; the per-tick appraisal recompute, not this
/// fold's clamp, sets the observed equilibrium.
pub const FEAR_CONTAGION_RATE: f64 = 0.05;

/// §10.1: The three relational fields an agent perceives around itself.
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct RelationalFields {
    // ── §10.1.1 Sensory field ─────────────────────────────────────
    /// Agents within `PERCEPTION_RADIUS` (proximity).
    pub nearby_agents: u32,
    /// Normalized closeness of the nearest agent (1 at distance 1 → 0 at the
    /// perception radius).
    pub nearest_closeness: Fixed,
    /// Mean perceived emotional stress of neighbors — "expression" (0–1):
    /// average of (fear + anger) / 2 over nearby agents.
    pub perceived_stress: Fixed,
    // ── §10.1.2 Social field ──────────────────────────────────────
    /// Mean trust over the agent's relationship graph (0–1).
    pub social_trust: Fixed,
    /// Mean obligation over the agent's relationship graph (0–1).
    pub social_obligation: Fixed,
    /// Number of kin-stage relationships (kinship, §10.3 kin branch).
    pub kin_count: u32,
    /// Highest effective status among nearby agents (0–1).
    pub peer_status: Fixed,
    // ── §10.1.3 Noospheric field ──────────────────────────────────
    /// Mean confidence of the agent's held beliefs (0–1).
    pub belief_confidence: Fixed,
    /// Highest emotional charge of the agent's held beliefs (0–1).
    pub hottest_charge: Fixed,
    /// The agent's perceived institutional legitimacy (0–1).
    pub legitimacy_perceived: Fixed,
    /// World mean fear — collective emotion (0–1).
    pub collective_fear: Fixed,
}

impl RelationalFields {
    /// §10.1.1 (Iteration 107): the fear-contagion contribution of the
    /// sensory field's perceived ambient stress ("expression"). Identity at
    /// zero stress (no stressed neighbors → no contagion), monotone in both
    /// arguments, deterministic (no RNG).
    pub fn contagion_delta(perceived_stress: Fixed, rate: Fixed) -> Fixed {
        (perceived_stress * rate).clamp_01()
    }

    /// §10.1.1 (Iteration 107): the full daily contagion apply-step —
    /// `fear + contagion_delta`, clamped to [0, 1]. Shared by the sim's
    /// daily fold and the unit tests so the clamp/saturation contract is
    /// exercised on the real path, never a reimplementation.
    pub fn contagion_apply(fear: Fixed, perceived_stress: Fixed, rate: Fixed) -> Fixed {
        (fear + Self::contagion_delta(perceived_stress, rate)).clamp_01()
    }

    /// Mean of a slice of `Fixed`; 0 when empty.
    pub fn mean(values: &[Fixed]) -> Fixed {
        if values.is_empty() {
            return Fixed::ZERO;
        }
        values.iter().fold(Fixed::ZERO, |acc, &v| acc + v)
            / Fixed::from_int(values.len() as i64)
    }

    /// Normalized closeness of a manhattan distance: 1 at distance 1,
    /// 0 at `PERCEPTION_RADIUS` and beyond (linear in between).
    pub fn closeness(distance: i32) -> Fixed {
        let d = distance.clamp(1, PERCEPTION_RADIUS);
        Fixed::from_f64(
            (PERCEPTION_RADIUS - d) as f64 / (PERCEPTION_RADIUS - 1) as f64,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mean_is_zero_for_empty_slice() {
        assert_eq!(RelationalFields::mean(&[]), Fixed::ZERO);
    }

    #[test]
    fn mean_averages_values() {
        let vals = [
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.6),
        ];
        assert_eq!(
            RelationalFields::mean(&vals),
            Fixed::from_f64(0.4)
        );
    }

    #[test]
    fn closeness_is_one_at_adjacency() {
        assert_eq!(
            RelationalFields::closeness(1),
            Fixed::from_f64(1.0)
        );
    }

    #[test]
    fn closeness_decays_to_zero_at_radius() {
        assert_eq!(
            RelationalFields::closeness(PERCEPTION_RADIUS),
            Fixed::ZERO
        );
        assert_eq!(RelationalFields::closeness(99), Fixed::ZERO);
    }

    #[test]
    fn contagion_delta_is_identity_at_zero_and_monotone() {
        // §10.1.1 (Iteration 107): zero perceived stress (no stressed
        // neighbors) must contribute nothing; the contribution is monotone
        // in both stress and rate, bounded by the clamp, and deterministic.
        let rate = Fixed::from_f64(FEAR_CONTAGION_RATE);
        assert_eq!(RelationalFields::contagion_delta(Fixed::ZERO, rate), Fixed::ZERO);
        assert_eq!(
            RelationalFields::contagion_delta(Fixed::from_f64(0.2), Fixed::ZERO),
            Fixed::ZERO
        );
        let low = RelationalFields::contagion_delta(Fixed::from_f64(0.2), rate);
        let high = RelationalFields::contagion_delta(Fixed::from_f64(0.8), rate);
        assert!(low < high, "higher ambient stress must contribute more");
        assert_eq!(
            RelationalFields::contagion_delta(Fixed::ONE, rate),
            rate,
            "stress 1.0 × rate must contribute exactly the rate"
        );
        // Clamp: a large rate saturates at 1.0.
        assert_eq!(
            RelationalFields::contagion_delta(Fixed::ONE, Fixed::from_f64(2.0)),
            Fixed::ONE
        );
        // Determinism.
        for _ in 0..3 {
            assert_eq!(
                RelationalFields::contagion_delta(Fixed::from_f64(0.55), rate),
                RelationalFields::contagion_delta(Fixed::from_f64(0.55), rate)
            );
        }
    }

    #[test]
    fn contagion_apply_clamps_and_preserves_identity() {
        // The real apply-step shared with the sim fold: identity at zero
        // stress, unclamped pass-through below 1.0, and saturation when
        // fear + delta would overflow.
        let rate = Fixed::from_f64(FEAR_CONTAGION_RATE);
        assert_eq!(
            RelationalFields::contagion_apply(Fixed::from_f64(0.5), Fixed::ZERO, rate),
            Fixed::from_f64(0.5),
            "zero ambient stress must leave fear unchanged"
        );
        assert_eq!(
            RelationalFields::contagion_apply(Fixed::from_f64(0.5), Fixed::from_f64(0.2), rate),
            Fixed::from_f64(0.5) + Fixed::from_f64(0.2) * rate,
            "fear + delta below 1.0 must pass through unclamped"
        );
        assert_eq!(
            RelationalFields::contagion_apply(Fixed::from_f64(0.99), Fixed::ONE, rate),
            Fixed::ONE,
            "fear + delta above 1.0 must saturate at the clamp"
        );
    }

    #[test]
    fn relational_fields_defaults_are_in_range() {
        let f = RelationalFields::default();
        assert_eq!(f.nearby_agents, 0);
        assert_eq!(f.kin_count, 0);
        assert!(f.social_trust >= Fixed::ZERO && f.social_trust <= Fixed::ONE);
        assert!(f.collective_fear >= Fixed::ZERO && f.collective_fear <= Fixed::ONE);
    }
}
