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
//! escalates to violence less readily. The noospheric field's
//! `legitimacy_perceived` gained a consumer in Iteration 111: the
//! deterrence factor (`legitimacy_deterrence_factor`) scales the theft
//! amount down when the agent perceives the institution as legitimately
//! ruling (one-sided — identity at/below the 0.5 construction anchor, so
//! legitimacy must be genuinely earned above the baseline; provably
//! zero-blast because legitimacy decays to ~0.04 in every calibrated
//! window). The noospheric field's `collective_fear` gained a consumer in
//! Iteration 112: the panic amplification factor
//! (`collective_fear_panic_amplifier`) scales the legitimacy damage a moral
//! panic inflicts — a genuinely terrified population amplifies the crisis
//! (one-sided — identity at/below the 0.95 anchor, which sits above the
//! calibrated peak of 0.903, so provably zero-blast: the single seed-42
//! panic at ~4,320 ticks sits at collective fear 0.90 < 0.95). The social
//! field's `peer_status` gained a consumer in Iteration 113: the envy
//! channel (`envy_delta`/`envy_apply`) folds status frustration into the
//! daily anger fold — an agent who perceives a GENUINELY dominant
//! neighbor (peer_status above the 0.5 anchor, which sits above the
//! calibrated ceiling of 0.4687 — probe-pinned max at t=200, seed 42,
//! across seeds 1/7/42/99, drought/pestilence scenarios, and the 20K
//! horizon) grows envious,
//! feeding anger — one-sided, so provably zero-blast. The social field's
//! `social_obligation` gained a consumer in Iteration 114: the restraint
//! factor (`obligation_restraint_factor`) scales the failed-threat
//! escalation chance down — an agent bound by deep reciprocal obligations
//! escalates less readily ("I owe this web; attacking dishonors my debts")
//! — a SECOND §19.5.H pacifier alongside Iter-110's trust (trust = "I
//! believe they won't harm me", obligation = "I have duties I must
//! honor"). ONE-SIDED at the semantic-midpoint anchor 0.5: identity in
//! every golden/snapshot horizon (seed-42 max obligation 0.456@5000 —
//! proven by the byte-identical gates), engaging only in deep-debt worlds
//! (mean obligation reaches 0.69@20K) and late crisis scenarios
//! (0.52–0.61@3000) — a designed crisis activation, so the regression
//! blast is zero. With this, EVERY §10.1 relational-field layer has a
//! decisional consumer.

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

/// The legitimacy anchor (§10.1.3, Iteration 111): the construction-time
/// value of `legitimacy_field.overall` (sim.rs:877). The theft-deterrence
/// factor is identity at or below this anchor — an institution must GENUINELY
/// earn legitimacy above the baseline before it deters theft. This is what
/// makes the consumer zero-blast: legitimacy decays to ~0.04 in every
/// calibrated window (probe-pinned), so the factor is exactly 1.0 throughout
/// the golden/snapshot horizons (golden byte-identical, no regeneration).
pub const LEGITIMACY_DETERRENCE_ANCHOR: f64 = 0.5;

/// Per-unit-above-anchor theft-deterrence rate (§10.1.3, Iteration 111):
/// each unit of perceived legitimacy above the anchor reduces the theft
/// amount by this fraction. At legitimacy 0.8 (a genuinely respected
/// institution) the take drops 15%; at full legitimacy 1.0 it drops 25%.
/// Modest so legitimate institutions deter without abolishing theft.
pub const LEGITIMACY_DETERRENCE_RATE: f64 = 0.5;

/// Ceiling on the legitimacy deterrence (§10.1.3, Iteration 111): the
/// institution can deter but never erase theft — the factor stays in
/// [0.5, 1.0] for the shipped constants (cap binds only if a future rate
/// change would push the reduction past 50%).
pub const LEGITIMACY_DETERRENCE_CAP: f64 = 0.5;

/// The collective-fear panic anchor (§10.1.3, Iteration 112): the world
/// mean fear above which a moral panic's legitimacy damage is amplified.
/// Placed at 0.95 — ABOVE the calibrated world's collective-fear peak of
/// 0.903 (probe-pinned) — so the amplifier is identity (factor 1.0)
/// throughout the golden/snapshot horizons. The single moral panic that
/// fires in seed-42 runs (~4,320 ticks) sits at collective fear 0.90, so
/// it is not amplified either: the consumer engages only in true terror.
pub const COLLECTIVE_FEAR_PANIC_ANCHOR: f64 = 0.95;

/// Per-unit-above-anchor panic-amplification rate (§10.1.3, Iteration 112):
/// each unit of collective fear above the anchor multiplies the panic's
/// legitimacy damage by `1 + above × this`. At full terror (fear 1.0) the
/// damage is amplified by exactly 2.5% — modest, bounded, deterministic.
pub const COLLECTIVE_FEAR_PANIC_RATE: f64 = 0.5;

/// Ceiling on the panic amplification (§10.1.3, Iteration 112): collective
/// fear can intensify a panic but never more than double its legitimacy
/// damage — the factor stays in [1.0, 1.5] for the shipped constants (cap
/// binds only if a future rate change would push the amplification past
/// 50%).
pub const COLLECTIVE_FEAR_PANIC_CAP: f64 = 0.5;

/// The peer-status envy anchor (§10.1.2, Iteration 113): the highest
/// effective status among nearby agents above which an agent grows envious
/// of a genuinely dominant presence. Placed at 0.5 — ABOVE the calibrated
/// ceiling of 0.4687 (probe-pinned max at t=200, seed 42; the widest sweep
/// yet: seeds 1/7/42/99, drought/pestilence scenarios, and the 20K horizon
/// all stay ≤ 0.4687) — so the envy term is identity (zero delta)
/// throughout the golden/snapshot horizons (provably zero-blast).
pub const PEER_ENVY_ANCHOR: f64 = 0.5;

/// Per-unit-above-anchor envy rate (§10.1.2, Iteration 113): each unit of
/// perceived peer status above the anchor adds this fraction to the
/// agent's daily anger. At peer status 0.6 (a genuinely dominant presence)
/// anger rises 0.03/day — a visible but bounded status-frustration
/// pressure.
pub const PEER_ENVY_RATE: f64 = 0.3;

/// Ceiling on the daily envy contribution (§10.1.2, Iteration 113): status
/// frustration can add at most this much anger per day — the term stays in
/// [0, 0.15] for the shipped constants (cap binds exactly at full
/// dominance: (1.0 − 0.5) × 0.3 = 0.15).
pub const PEER_ENVY_CAP: f64 = 0.15;

/// The obligation-restraint anchor (§10.1.2, Iteration 114): mean
/// obligation over the agent's relationship graph above which the
/// failed-threat escalation chance is dampened. Placed at the semantic
/// midpoint 0.5 — the wide-sweep probe (seeds 1/7/42/99, drought/
/// pestilence/riverford/famine, 20K horizon) shows every golden/snapshot
/// horizon stays below it (seed-42 max 0.456@5000, proven by the
/// byte-identical gates), so the factor is identity (1.0) throughout the
/// golden/snapshot horizons (zero regression blast). The anchor is deliberately reachable: deep
/// debt webs DO cross it (seed-42 mean obligation reaches 0.69@20K;
/// scenarios 0.52–0.61@3000), so the restraint genuinely engages in
/// long-running worlds — a designed crisis activation, not dead code.
pub const OBLIGATION_RESTRAINT_ANCHOR: f64 = 0.5;

/// Per-unit-above-anchor obligation-restraint rate (§10.1.2, Iteration
/// 114): each unit of mean obligation above the anchor reduces the
/// escalation chance by this fraction. At obligation 0.6 (a genuinely
/// bound relationship web) the chance drops 3%; at full obligation 1.0 it
/// drops 15% — modest, bounded, deterministic.
pub const OBLIGATION_RESTRAINT_RATE: f64 = 0.3;

/// Ceiling on the obligation restraint (§10.1.2, Iteration 114): duties can
/// restrain violence but never erase it — the factor stays in [0.7, 1.0]
/// for the shipped constants (cap binds only if a future rate change would
/// push the reduction past 30%; with the shipped rate the max reduction is
/// (1.0 − 0.5) × 0.3 = 0.15 at full obligation).
pub const OBLIGATION_RESTRAINT_CAP: f64 = 0.3;

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

    /// §10.1.3 (Iteration 111): the legitimacy deterrence factor — an agent
    /// who believes the institution rules rightfully takes less in a theft
    /// ("the grain belongs to a legitimate authority; stealing undermines
    /// it"). ONE-SIDED: identity at or below the construction anchor (0.5),
    /// so a merely-default institution deters nothing — it must GENUINELY
    /// earn legitimacy above the baseline. This is provably zero-blast:
    /// legitimacy decays to ~0.04 in every calibrated window (probe-pinned),
    /// so the factor is exactly 1.0 throughout the golden/snapshot horizons
    /// (golden byte-identical, no regeneration). Monotone above the anchor,
    /// capped so the institution can never erase theft entirely,
    /// deterministic (no RNG). The caller multiplies the theft amount in
    /// `enforce_theft` by the returned factor.
    pub fn legitimacy_deterrence_factor(
        legitimacy_perceived: Fixed,
        anchor: Fixed,
        rate: Fixed,
        cap: Fixed,
    ) -> Fixed {
        let above_anchor = (legitimacy_perceived - anchor).max(Fixed::ZERO);
        let reduction = (above_anchor * rate).min(cap);
        // The clamp is defensive: reduction ∈ [0, cap] with cap < 1.0, so
        // the factor is provably in [0.5, 1.0] for the shipped constants.
        (Fixed::ONE - reduction).clamp_01()
    }

    /// §10.1.3 (Iteration 112): the panic amplification factor — when a
    /// moral panic fires, a genuinely terrified population amplifies the
    /// legitimacy damage it inflicts ("fear → gossip → more fear", §13.5's
    /// self-reinforcing loop made live). ONE-SIDED: identity at or below the
    /// anchor (0.95), so the amplifier engages only in true terror — the
    /// calibrated world's collective fear peaks at 0.903 (probe-pinned), so
    /// the factor is exactly 1.0 throughout the golden/snapshot horizons
    /// (golden byte-identical, no regeneration; the single panic that fires
    /// at ~4,320 ticks in seed-42 runs sits at collective fear 0.90 < 0.95
    /// and is likewise unamplified). Monotone above the anchor, capped so
    /// the amplification can never more than double the damage,
    /// deterministic (no RNG). The caller multiplies
    /// `MoralPanicResult::legitimacy_damage` in
    /// `tick_moral_panic_and_revolution` by the returned factor.
    pub fn collective_fear_panic_amplifier(
        collective_fear: Fixed,
        anchor: Fixed,
        rate: Fixed,
        cap: Fixed,
    ) -> Fixed {
        let above_anchor = (collective_fear - anchor).max(Fixed::ZERO);
        let amplification = (above_anchor * rate).min(cap);
        // NOTE: no `clamp_01` here — an amplifier must exceed 1.0, and
        // `clamp_01` caps at 1.0 (it would silently erase the amplification
        // and turn this into an identity, which a shipping unit test caught).
        // No clamp is needed: amplification ∈ [0, cap] with cap < 1.0, so
        // the factor is provably in [1.0, 1.5] for the shipped constants.
        Fixed::ONE + amplification
    }

    /// §10.1.2 (Iteration 114): the obligation-restraint factor — an agent
    /// bound by deep reciprocal obligations escalates a failed threat to
    /// violence less readily ("I owe this web; attacking dishonors my
    /// debts"). This is the SECOND §19.5.H pacifier alongside Iter-110's
    /// trust (trust = "I believe they won't harm me" — relational
    /// confidence; obligation = "I have duties I must honor" — moral
    /// constraint); §10.1.2 lists both as social-field layers, so both act
    /// on the violence decision. ONE-SIDED: identity at or below the anchor
    /// (0.5) — every golden/snapshot horizon sits below it (seed-42 max
    /// 0.456@5000, proven by the byte-identical gates), so the factor is
    /// exactly 1.0 throughout the golden/snapshot horizons (zero
    /// regression blast); deep-debt worlds cross it by design (mean
    /// obligation reaches 0.69@20K), activating the restraint. Monotone above the anchor, capped so the restraint
    /// can never erase the escalation chance entirely, deterministic (no
    /// RNG). The caller multiplies the escalation chance in
    /// `should_escalate` by the returned factor.
    pub fn obligation_restraint_factor(
        social_obligation: Fixed,
        anchor: Fixed,
        rate: Fixed,
        cap: Fixed,
    ) -> Fixed {
        let above_anchor = (social_obligation - anchor).max(Fixed::ZERO);
        let reduction = (above_anchor * rate).min(cap);
        // The clamp is defensive: reduction ∈ [0, cap] with cap < 1.0, so
        // the factor is provably in [0.7, 1.0] for the shipped constants.
        (Fixed::ONE - reduction).clamp_01()
    }

    /// §10.1.2 (Iteration 113): the peer-status envy contribution — an
    /// agent who perceives a GENUINELY dominant neighbor (peer_status above
    /// the 0.5 anchor) grows envious: status frustration feeds anger ("the
    /// most dominant presence I see outranks me enough to sting"). ONE-SIDED:
    /// identity (zero delta) at or below the anchor — the calibrated world's
    /// peer status peaks at 0.4687 (probe-pinned), so this term is exactly
    /// zero throughout the golden/snapshot horizons (provably zero-blast).
    /// Monotone above the anchor, capped so the daily envy pressure stays
    /// bounded, deterministic (no RNG).
    pub fn envy_delta(
        peer_status: Fixed,
        anchor: Fixed,
        rate: Fixed,
        cap: Fixed,
    ) -> Fixed {
        let above_anchor = (peer_status - anchor).max(Fixed::ZERO);
        // No clamp needed: above_anchor ∈ [0, 1], rate and cap are small, so
        // the delta is provably in [0, cap] ⊆ [0, 0.15] for the shipped
        // constants.
        (above_anchor * rate).min(cap)
    }

    /// §10.1.2 (Iteration 113): the full daily envy apply-step — `anger +
    /// envy_delta`, clamped to [0, 1]. Shared by the sim's daily fold and
    /// the unit tests so the clamp/saturation contract is exercised on the
    /// real path, never a reimplementation (same pattern as
    /// `contagion_apply`).
    pub fn envy_apply(
        anger: Fixed,
        peer_status: Fixed,
        anchor: Fixed,
        rate: Fixed,
        cap: Fixed,
    ) -> Fixed {
        (anger + Self::envy_delta(peer_status, anchor, rate, cap)).clamp_01()
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
    fn legitimacy_deterrence_factor_is_identity_at_anchor_and_capped() {
        // §10.1.3 (Iteration 111): at/below the construction anchor the
        // factor is exactly 1.0 (byte-identical — legitimacy decays to ~0.04
        // in every calibrated window, so the golden horizons never see the
        // consumer); above the anchor it deters monotonically and is capped
        // so the institution can never erase theft entirely.
        let anchor = Fixed::from_f64(LEGITIMACY_DETERRENCE_ANCHOR);
        let rate = Fixed::from_f64(LEGITIMACY_DETERRENCE_RATE);
        let cap = Fixed::from_f64(LEGITIMACY_DETERRENCE_CAP);
        assert_eq!(
            RelationalFields::legitimacy_deterrence_factor(anchor, anchor, rate, cap),
            Fixed::ONE,
            "at the construction anchor the factor must be a byte-identical identity"
        );
        assert_eq!(
            RelationalFields::legitimacy_deterrence_factor(Fixed::ZERO, anchor, rate, cap),
            Fixed::ONE,
            "below the anchor (decayed legitimacy) must also be identity"
        );
        assert_eq!(
            RelationalFields::legitimacy_deterrence_factor(
                Fixed::from_f64(0.8),
                anchor,
                rate,
                cap,
            ),
            Fixed::from_f64(0.85),
            "legitimacy 0.8 must deter by exactly 0.15"
        );
        let low = RelationalFields::legitimacy_deterrence_factor(
            Fixed::from_f64(0.6),
            anchor,
            rate,
            cap,
        );
        let high = RelationalFields::legitimacy_deterrence_factor(
            Fixed::from_f64(0.9),
            anchor,
            rate,
            cap,
        );
        assert!(low > high, "higher legitimacy must deter more");
        assert_eq!(
            RelationalFields::legitimacy_deterrence_factor(Fixed::ONE, anchor, rate, cap),
            Fixed::ONE - rate * Fixed::from_f64(0.5),
            "full legitimacy with the shipped rate must deter by exactly 0.25"
        );
        // Cap: an oversized rate saturates at the cap, never below.
        assert_eq!(
            RelationalFields::legitimacy_deterrence_factor(
                Fixed::ONE,
                anchor,
                Fixed::from_f64(5.0),
                cap,
            ),
            Fixed::ONE - cap,
            "an oversized rate must saturate at the cap"
        );
    }

    #[test]
    fn collective_fear_panic_amplifier_is_identity_below_anchor_and_capped() {
        // §10.1.3 (Iteration 112): at/below the anchor the factor is exactly
        // 1.0 (byte-identical — the calibrated world's collective fear peaks
        // at 0.903 < 0.95, so the golden horizons never see the consumer);
        // above the anchor it amplifies monotonically and is capped so the
        // panic damage can never more than double.
        let anchor = Fixed::from_f64(COLLECTIVE_FEAR_PANIC_ANCHOR);
        let rate = Fixed::from_f64(COLLECTIVE_FEAR_PANIC_RATE);
        let cap = Fixed::from_f64(COLLECTIVE_FEAR_PANIC_CAP);
        assert_eq!(
            RelationalFields::collective_fear_panic_amplifier(anchor, anchor, rate, cap),
            Fixed::ONE,
            "at the anchor the factor must be a byte-identical identity"
        );
        assert_eq!(
            RelationalFields::collective_fear_panic_amplifier(
                Fixed::from_f64(0.9),
                anchor,
                rate,
                cap,
            ),
            Fixed::ONE,
            "below the anchor (calibrated-world fear 0.903) must also be identity"
        );
        assert_eq!(
            RelationalFields::collective_fear_panic_amplifier(Fixed::ONE, anchor, rate, cap),
            Fixed::from_f64(1.025),
            "full terror with the shipped rate must amplify by exactly 0.025"
        );
        let low = RelationalFields::collective_fear_panic_amplifier(
            Fixed::from_f64(0.96),
            anchor,
            rate,
            cap,
        );
        let high = RelationalFields::collective_fear_panic_amplifier(
            Fixed::from_f64(0.99),
            anchor,
            rate,
            cap,
        );
        assert!(low < high, "higher collective fear must amplify more");
        // Cap: an oversized rate saturates at the cap, never above.
        // (Rate 20.0 × above-anchor 0.05 = 1.0 > cap 0.5, so the cap binds;
        // rate 5.0 would give 0.25 < cap and NOT saturate — the cap only
        // engages when above-anchor × rate ≥ cap.)
        assert_eq!(
            RelationalFields::collective_fear_panic_amplifier(
                Fixed::ONE,
                anchor,
                Fixed::from_f64(20.0),
                cap,
            ),
            Fixed::ONE + cap,
            "an oversized rate must saturate at the cap"
        );
    }

    #[test]
    fn obligation_restraint_factor_is_identity_below_anchor_and_capped() {
        // §10.1.2 (Iteration 114): at/below the anchor the factor is exactly
        // 1.0 (byte-identical — every golden/snapshot horizon sits below
        // 0.5: seed-42 max obligation 0.456@5000, proven by the
        // byte-identical gates, so the golden/snapshot horizons never see
        // the consumer); above the anchor it restrains monotonically and is
        // capped so the duty can never erase the escalation chance
        // entirely.
        let anchor = Fixed::from_f64(OBLIGATION_RESTRAINT_ANCHOR);
        let rate = Fixed::from_f64(OBLIGATION_RESTRAINT_RATE);
        let cap = Fixed::from_f64(OBLIGATION_RESTRAINT_CAP);
        assert_eq!(
            RelationalFields::obligation_restraint_factor(anchor, anchor, rate, cap),
            Fixed::ONE,
            "at the anchor the factor must be a byte-identical identity"
        );
        assert_eq!(
            RelationalFields::obligation_restraint_factor(
                Fixed::from_f64(0.456),
                anchor,
                rate,
                cap,
            ),
            Fixed::ONE,
            "below the anchor (seed-42 max 0.456@5000) must also be identity"
        );
        assert_eq!(
            RelationalFields::obligation_restraint_factor(
                Fixed::from_f64(0.6),
                anchor,
                rate,
                cap,
            ),
            Fixed::from_f64(0.97),
            "obligation 0.6 must restrain by exactly 0.03"
        );
        let low = RelationalFields::obligation_restraint_factor(
            Fixed::from_f64(0.55),
            anchor,
            rate,
            cap,
        );
        let high = RelationalFields::obligation_restraint_factor(
            Fixed::from_f64(0.8),
            anchor,
            rate,
            cap,
        );
        assert!(low > high, "higher obligation must restrain more");
        // Full obligation: (1.0 − 0.5) × 0.3 = 0.15 reduction.
        assert_eq!(
            RelationalFields::obligation_restraint_factor(Fixed::ONE, anchor, rate, cap),
            Fixed::ONE - Fixed::from_f64(0.15),
            "full obligation with the shipped rate must restrain by exactly 0.15"
        );
        // Cap: an oversized rate saturates at the cap, never below.
        assert_eq!(
            RelationalFields::obligation_restraint_factor(
                Fixed::ONE,
                anchor,
                Fixed::from_f64(5.0),
                cap,
            ),
            Fixed::ONE - cap,
            "an oversized rate must saturate at the cap"
        );
    }

    #[test]
    fn peer_envy_is_identity_below_anchor_and_capped() {
        // §10.1.2 (Iteration 113): at/below the anchor the envy delta is
        // exactly zero (byte-identical — the calibrated world's peer status
        // peaks at 0.4687 < 0.5, so the golden horizons never see the term);
        // above the anchor it rises monotonically and is capped so the
        // daily status-frustration pressure stays bounded.
        let anchor = Fixed::from_f64(PEER_ENVY_ANCHOR);
        let rate = Fixed::from_f64(PEER_ENVY_RATE);
        let cap = Fixed::from_f64(PEER_ENVY_CAP);
        assert_eq!(
            RelationalFields::envy_delta(anchor, anchor, rate, cap),
            Fixed::ZERO,
            "at the anchor the envy delta must be exactly zero"
        );
        assert_eq!(
            RelationalFields::envy_delta(Fixed::from_f64(0.46), anchor, rate, cap),
            Fixed::ZERO,
            "below the anchor (calibrated-world ceiling 0.46) must also be zero"
        );
        assert_eq!(
            RelationalFields::envy_delta(Fixed::from_f64(0.6), anchor, rate, cap),
            Fixed::from_f64(0.03),
            "a genuinely dominant presence (0.6) must add exactly 0.03/day"
        );
        let low = RelationalFields::envy_delta(Fixed::from_f64(0.55), anchor, rate, cap);
        let high = RelationalFields::envy_delta(Fixed::from_f64(0.8), anchor, rate, cap);
        assert!(low < high, "higher peer status must enrage more");
        // Full dominance: (1.0 − 0.5) × 0.3 = 0.15 = the cap exactly.
        assert_eq!(
            RelationalFields::envy_delta(Fixed::ONE, anchor, rate, cap),
            cap,
            "full dominance with the shipped rate must hit the cap exactly"
        );
        // Cap: an oversized rate saturates at the cap, never above.
        assert_eq!(
            RelationalFields::envy_delta(Fixed::ONE, anchor, Fixed::from_f64(5.0), cap),
            cap,
            "an oversized rate must saturate at the cap"
        );
        // The shared apply-step: anger + delta, clamped.
        assert_eq!(
            RelationalFields::envy_apply(
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.6),
                anchor,
                rate,
                cap,
            ),
            Fixed::from_f64(0.53),
            "envy_apply must add the delta to anger on the real path"
        );
        assert_eq!(
            RelationalFields::envy_apply(
                Fixed::from_f64(0.99),
                Fixed::ONE,
                anchor,
                rate,
                cap,
            ),
            Fixed::ONE,
            "anger + delta above 1.0 must saturate at the clamp"
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
