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
//! field's `kin_count` has a decisional consumer too (Iteration 108): the
//! kin-support buffer (`kin_stress_factor`) scales the per-tick stress
//! input to cognition down — family buffers stress, zero-at-zero (no kin
//! → factor 1.0 → byte-identical). The noospheric field's
//! `belief_confidence` gained a consumer in Iteration 109: the dogmatism
//! factor (`belief_rigidity_factor`) scales belief-update resistance — an
//! agent holding its beliefs with high mean confidence updates them more
//! slowly (self-assurance → closed-mindedness). The social field's
//! `social_trust` gained a consumer in Iteration 110: the pacification
//! factor (`trust_pacify_factor`) scales the failed-threat escalation
//! chance down — an agent embedded in a trusting relationship graph
//! escalates to violence less readily. The remaining social
//! (obligation/peer-status) and noospheric (legitimacy, collective fear)
//! layers remain observational.

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

/// Per-kin stress-buffer rate (§10.1.2, Iteration 108): each kin-stage
/// relationship reduces the stress input to cognition by this fraction.
/// Modest so a family buffers without zeroing the signal; zero kin →
/// identity (byte-identical calibrated runs, since kinship forms only via
/// births, earliest at ~4,170 ticks).
pub const KIN_STRESS_RATE: f64 = 0.15;

/// Ceiling on the total kin stress reduction (§10.1.2, Iteration 108):
/// 3+ kin saturate at 45% buffering — the buffer cannot erase the signal.
pub const KIN_STRESS_CAP: f64 = 0.45;

/// Dogmatism rigidity per unit of mean belief confidence (§10.1.3,
/// Iteration 109): the update-time resistance multiplier is
/// `1 + belief_confidence × this`. At confidence 0 (no beliefs) the
/// factor is exactly 1.0 — byte-identical. At the default world's
/// construction-time mean (0.55) it reaches 1.275 — a 27% resistance
/// uplift for fully-confident belief holders, bounded and deterministic.
pub const BELIEF_CONFIDENCE_RIGIDITY: f64 = 0.5;

/// Per-unit-trust violence-pacification rate (§10.1.2, Iteration 110):
/// each unit of mean social trust reduces the failed-threat escalation
/// chance by this fraction. At the calibrated world's mean trust (0.5)
/// the escalation chance drops ~15%; at full trust (1.0) it drops 30%.
/// Zero trust (empty relationship graph, tick 0) → identity 1.0.
pub const SOCIAL_TRUST_PACIFY_RATE: f64 = 0.3;

/// Ceiling on the trust pacification (§10.1.2, Iteration 110): the social
/// fabric can dampen escalation but never erase it — the factor stays in
/// [0.4, 1.0] for the shipped constants (cap binds only if a future rate
/// change would push the reduction past 60%).
pub const SOCIAL_TRUST_PACIFY_CAP: f64 = 0.6;

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

    /// §10.1.2 (Iteration 108): the kin-support buffer — kinship reduces
    /// the stress input to cognition (family buffers stress). Identity at
    /// zero kin (no family → no buffer), monotone in `kin_count`, capped at
    /// `cap` total reduction, deterministic (no RNG). The caller multiplies
    /// the agent's per-tick stress input (fear + anger) by the returned
    /// factor before `CognitiveState::update`.
    pub fn kin_stress_factor(kin_count: u32, rate: Fixed, cap: Fixed) -> Fixed {
        let reduction = (Fixed::from_int(kin_count as i64) * rate).min(cap);
        // The clamp is defensive: reduction ∈ [0, cap] with cap < 1.0, so
        // the factor is provably in [0.55, 1.0] for the shipped constants.
        (Fixed::ONE - reduction).clamp_01()
    }

    /// §10.1.3 (Iteration 109): the dogmatism factor — an agent holding
    /// its beliefs with high mean confidence resists counter-evidence more
    /// (self-assurance → closed-mindedness). Identity at zero confidence
    /// (no beliefs → factor exactly 1.0 → byte-identical), monotone in
    /// `belief_confidence`, deterministic (no RNG). The caller multiplies
    /// the per-belief update `resistance` by the returned factor before
    /// `update_belief`'s base update.
    pub fn belief_rigidity_factor(belief_confidence: Fixed, rigidity: Fixed) -> Fixed {
        // confidence ∈ [0, 1] and rigidity is small, so the factor is
        // provably in [1.0, ~1.5]; no clamp needed, but documented.
        Fixed::ONE + belief_confidence * rigidity
    }

    /// §10.1.2 (Iteration 110): the trust pacification factor — an agent
    /// embedded in a trusting relationship graph escalates a failed threat
    /// to violence less readily ("the social fabric restrains me").
    /// Identity at zero mean trust (no relationships → factor exactly 1.0
    /// → byte-identical at tick 0), monotone in `social_trust`, capped so
    /// the pacification can never erase the escalation chance entirely,
    /// deterministic (no RNG). The caller multiplies the escalation chance
    /// in `should_escalate` by the returned factor.
    pub fn trust_pacify_factor(social_trust: Fixed, rate: Fixed, cap: Fixed) -> Fixed {
        let reduction = (social_trust * rate).min(cap);
        // The clamp is defensive: reduction ∈ [0, cap] with cap < 1.0, so
        // the factor is provably in [0.4, 1.0] for the shipped constants.
        (Fixed::ONE - reduction).clamp_01()
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
    fn kin_stress_factor_is_identity_at_zero_and_capped() {
        // §10.1.2 (Iteration 108): zero kin (no family) must leave the
        // stress input untouched; the buffer is monotone in kin count and
        // capped so it can never erase the signal.
        let rate = Fixed::from_f64(KIN_STRESS_RATE);
        let cap = Fixed::from_f64(KIN_STRESS_CAP);
        assert_eq!(
            RelationalFields::kin_stress_factor(0, rate, cap),
            Fixed::ONE,
            "no kin must be a byte-identical identity"
        );
        assert_eq!(
            RelationalFields::kin_stress_factor(1, rate, cap),
            Fixed::ONE - rate,
            "one kin buffers exactly the rate"
        );
        assert_eq!(
            RelationalFields::kin_stress_factor(2, rate, cap),
            Fixed::ONE - rate - rate,
            "two kin buffer twice the rate"
        );
        // 3 × 0.15 = 0.45 = the cap exactly.
        assert_eq!(
            RelationalFields::kin_stress_factor(3, rate, cap),
            Fixed::ONE - cap,
            "three kin reach the cap"
        );
        assert_eq!(
            RelationalFields::kin_stress_factor(10, rate, cap),
            Fixed::ONE - cap,
            "many kin saturate at the cap, never below"
        );
    }

    #[test]
    fn trust_pacify_factor_is_identity_at_zero_and_capped() {
        // §10.1.2 (Iteration 110): zero mean trust (empty relationship
        // graph) must leave the escalation chance untouched; the pacification
        // is monotone in trust and capped so it never erases violence.
        let rate = Fixed::from_f64(SOCIAL_TRUST_PACIFY_RATE);
        let cap = Fixed::from_f64(SOCIAL_TRUST_PACIFY_CAP);
        assert_eq!(
            RelationalFields::trust_pacify_factor(Fixed::ZERO, rate, cap),
            Fixed::ONE,
            "zero trust must be a byte-identical identity"
        );
        assert_eq!(
            RelationalFields::trust_pacify_factor(Fixed::from_f64(0.5), rate, cap),
            Fixed::ONE - Fixed::from_f64(0.15),
            "mean-trust world (0.5) must pacify by exactly 0.15"
        );
        let low = RelationalFields::trust_pacify_factor(Fixed::from_f64(0.2), rate, cap);
        let high = RelationalFields::trust_pacify_factor(Fixed::from_f64(0.8), rate, cap);
        assert!(low > high, "higher trust must pacify more");
        // 1.0 × 0.3 = 0.3 < cap 0.6 → the reduction is exactly rate at full trust.
        assert_eq!(
            RelationalFields::trust_pacify_factor(Fixed::ONE, rate, cap),
            Fixed::ONE - rate,
            "full trust with the shipped rate must pacify by exactly the rate"
        );
        // Cap: a rate that would exceed the cap binds instead.
        assert_eq!(
            RelationalFields::trust_pacify_factor(Fixed::ONE, Fixed::from_f64(2.0), cap),
            Fixed::ONE - cap,
            "an oversized rate must saturate at the cap, never below"
        );
    }

    #[test]
    fn belief_rigidity_factor_is_identity_at_zero_and_monotone() {
        // §10.1.3 (Iteration 109): zero mean belief confidence (no beliefs)
        // must leave update resistance untouched (identity, byte-identical);
        // higher confidence scales resistance up monotonically.
        let rigidity = Fixed::from_f64(BELIEF_CONFIDENCE_RIGIDITY);
        assert_eq!(
            RelationalFields::belief_rigidity_factor(Fixed::ZERO, rigidity),
            Fixed::ONE,
            "no beliefs must be a byte-identical identity"
        );
        assert_eq!(
            RelationalFields::belief_rigidity_factor(Fixed::from_f64(0.5), rigidity),
            Fixed::from_f64(1.25),
            "confidence 0.5 × rigidity 0.5 must give exactly 1.25"
        );
        let low = RelationalFields::belief_rigidity_factor(Fixed::from_f64(0.2), rigidity);
        let high = RelationalFields::belief_rigidity_factor(Fixed::from_f64(0.8), rigidity);
        assert!(low < high, "higher confidence must scale resistance up more");
        assert_eq!(
            RelationalFields::belief_rigidity_factor(Fixed::ONE, rigidity),
            Fixed::from_f64(1.5),
            "full confidence with the shipped rigidity caps at 1.5"
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
