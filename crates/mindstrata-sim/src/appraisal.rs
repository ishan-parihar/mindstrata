//! Appraisal system — emotions arise from events, not direct assignment.
//!
//! This is the heart of the cognitive pipeline.  When an event occurs,
//! agents appraise it along multiple dimensions and derive emotional
//! responses.  This produces psychologically grounded emergence.

use mindstrata_core::clock::Tick;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Who or what caused the event.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Agency {
    /// The agent themselves caused it.
    Self_,
    /// Another agent caused it.
    Other(AgentId),
    /// Circumstances / environment caused it.
    Circumstance,
    /// Unknown cause.
    Unknown,
}

/// An appraisal of an event along cognitive dimensions.
///
/// These dimensions follow Lazarus's cognitive appraisal theory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Appraisal {
    /// How relevant is this event to the agent's goals? (0..1)
    pub goal_relevance: Fixed,
    /// Is the event congruent with the agent's goals? (negative = threat)
    pub goal_congruence: Fixed,
    /// How much control does the agent have over the outcome? (0..1)
    pub coping_potential: Fixed,
    /// Was this event expected? (0 = unexpected, 1 = expected)
    pub expectedness: Fixed,
    /// Is the event fair or unfair? (negative = unfair)
    pub fairness: Fixed,
    /// Who caused this event?
    pub agency: Agency,
    /// How visible is this event to others? (0 = private, 1 = public)
    pub social_visibility: Fixed,
    /// How relevant is this to the agent's identity? (0..1)
    pub identity_relevance: Fixed,
    /// §8.1.4: Did the event violate a sacred value? (0..1)
    pub sacredness_violation: Fixed,
    /// §8.1.4: Did the event threaten an attachment bond? (0..1)
    pub attachment_threat: Fixed,
    /// §8.1.4: Did the event threaten social standing? (0..1)
    pub status_threat: Fixed,
    /// §8.1.4: Did the event violate purity/cleanliness norms? (0..1)
    pub purity_violation: Fixed,
    /// §8.1.4: How much control does the agent have over the outcome? (0..1)
    pub controllability: Fixed,
    /// §8.1.4: How will the event shape the future? (−1 = dire, +1 = bright)
    pub future_implication: Fixed,
    /// §8.1.4: How much narrative meaning does the event carry? (0..1)
    pub narrative_meaning: Fixed,
}

impl Default for Appraisal {
    fn default() -> Self {
        Self {
            goal_relevance: Fixed::ZERO,
            goal_congruence: Fixed::ZERO,
            coping_potential: Fixed::HALF,
            expectedness: Fixed::HALF,
            fairness: Fixed::HALF,
            agency: Agency::Unknown,
            social_visibility: Fixed::ZERO,
            identity_relevance: Fixed::ZERO,
            sacredness_violation: Fixed::ZERO,
            attachment_threat: Fixed::ZERO,
            status_threat: Fixed::ZERO,
            purity_violation: Fixed::ZERO,
            controllability: Fixed::ZERO,
            future_implication: Fixed::ZERO,
            narrative_meaning: Fixed::ZERO,
        }
    }
}

/// Emotional changes resulting from an appraisal.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EmotionDelta {
    pub fear: Fixed,
    pub anger: Fixed,
    pub joy: Fixed,
    pub sadness: Fixed,
    pub trust: Fixed,
    pub shame: Fixed,
    pub pride: Fixed,
    pub guilt: Fixed,
    /// §8.1.4 expanded emotion families (22 total).
    pub disgust: Fixed,
    pub contempt: Fixed,
    pub awe: Fixed,
    pub gratitude: Fixed,
    pub jealousy: Fixed,
    pub envy: Fixed,
    pub loneliness: Fixed,
    pub tenderness: Fixed,
    pub humiliation: Fixed,
    pub relief: Fixed,
    pub hope: Fixed,
    pub despair: Fixed,
    pub nostalgia: Fixed,
    pub moral_outrage: Fixed,
}

impl EmotionDelta {
    /// Combine two deltas additively.
    #[must_use]
    pub fn combine(self, other: &EmotionDelta) -> Self {
        Self {
            fear: (self.fear + other.fear).clamp_01(),
            anger: (self.anger + other.anger).clamp_01(),
            joy: (self.joy + other.joy).clamp_01(),
            sadness: (self.sadness + other.sadness).clamp_01(),
            trust: (self.trust + other.trust).clamp_01(),
            shame: (self.shame + other.shame).clamp_01(),
            pride: (self.pride + other.pride).clamp_01(),
            guilt: (self.guilt + other.guilt).clamp_01(),
            disgust: (self.disgust + other.disgust).clamp_01(),
            contempt: (self.contempt + other.contempt).clamp_01(),
            awe: (self.awe + other.awe).clamp_01(),
            gratitude: (self.gratitude + other.gratitude).clamp_01(),
            jealousy: (self.jealousy + other.jealousy).clamp_01(),
            envy: (self.envy + other.envy).clamp_01(),
            loneliness: (self.loneliness + other.loneliness).clamp_01(),
            tenderness: (self.tenderness + other.tenderness).clamp_01(),
            humiliation: (self.humiliation + other.humiliation).clamp_01(),
            relief: (self.relief + other.relief).clamp_01(),
            hope: (self.hope + other.hope).clamp_01(),
            despair: (self.despair + other.despair).clamp_01(),
            nostalgia: (self.nostalgia + other.nostalgia).clamp_01(),
            moral_outrage: (self.moral_outrage + other.moral_outrage).clamp_01(),
        }
    }
}

/// Derive emotional response from an appraisal.
///
/// This implements the cognitive appraisal → emotion mapping from
/// the architecture spec's Section 9.2.
/// §8.1.4 (Iteration 116): per-tick proportional decay for the expanded
/// emotion families. The base 8 emotions decay subtractively (0.002/tick,
/// §22.1), but the 14 expanded families had NO decay — their per-tick
/// appraisal deltas (~0.08–0.12/tick, probe-pinned: awe climbs 0.08 → 1.0
/// in ~20 ticks) accumulated to saturation (awe/relief/hope/gratitude/
/// nostalgia/tenderness/loneliness pinned at 1.0 in every calibrated run,
/// probe-pinned). The appraisal runs EVERY tick (the per-tick emotion
/// path in sim.rs, not the daily phase), so a daily decay cannot hold
/// production — a per-tick proportional decay at 0.12 cancels production
/// to producer-driven
/// steady states (probe-pinned post-wiring, seeds 1/7/42/99 @ 5000): awe
/// 0.51–0.61, relief 0.65–0.79, hope 0.82–0.88, gratitude/nostalgia
/// 0.88 (their production is higher, ~0.12/tick). Applied to the 12
/// NON-consumed families only — `loneliness` (Iter-98 social-seeking) and
/// `tenderness` (Iter-99 helping) are consumed and were calibrated against
/// their saturated state; decaying those two is a separate, larger
/// calibration.
pub const SECONDARY_EMOTION_DECAY_RATE: f64 = 0.12;

/// §8.1.4 (Iteration 116): per-unit humiliation escalation rate — a
/// humiliated agent escalates a failed threat to violence more readily
/// (status-defeat → aggression, the amplifier counterpoint to the
/// Iter-110 trust / Iter-114 obligation pacifiers on the same §19.5.H
/// decision). At full humiliation 1.0 the escalation chance rises exactly
/// 30% (factor 1.30). ONE-SIDED: identity at zero — the calibration shows
/// humiliation is NEVER produced in any calibrated window (its appraisal
/// inputs — status threat + identity relevance — don't fire in calm
/// worlds; probe: mean/max 0.0000 across seeds and scenarios), so the
/// factor is exactly 1.0 throughout the golden/snapshot horizons
/// (provably zero-blast).
pub const HUMILIATION_ESCALATION_RATE: f64 = 0.3;

/// §8.1.4 (Iteration 116): proportional daily decay for a secondary
/// emotion — `value × (1 − rate)` clamped. Pure and deterministic (no
/// RNG); used by the daily emotion block to keep the expanded families at
/// meaningful producer-driven levels instead of saturating at 1.0.
pub fn secondary_emotion_decay(value: Fixed, rate: f64) -> Fixed {
    (value * (Fixed::ONE - Fixed::from_f64(rate))).clamp_01()
}

/// §8.1.4 (Iteration 116): the humiliation escalation factor — a
/// humiliated agent escalates a failed threat to violence more readily
/// (`1 + humiliation × rate`, ∈ [1.0, 1.3] for the shipped constant).
/// The caller multiplies the escalation chance in `should_escalate` by the
/// returned factor. NO clamp: the factor provably exceeds 1.0 (the
/// Iter-112 lesson — `clamp_01` would silently erase the amplification),
/// and it can never bind at the cap with the shipped rate. One-sided:
/// identity at zero, monotone above. Deterministic, no RNG.
pub fn humiliation_escalation_factor(humiliation: Fixed, rate: f64) -> Fixed {
    Fixed::ONE + humiliation * Fixed::from_f64(rate)
}

/// §8.1.4 (Iteration 122): the contempt escalation rate — a contemptuous
/// aggressor escalates a failed threat to violence more readily ("the
/// target is beneath me — no restraint needed"). At full contempt 1.0 the
/// escalation chance rises exactly 30% (factor 1.30, mirroring
/// humiliation's shipped constant — both are dignity emotions, both
/// amplify the same §19.5.H decision). ONE-SIDED: identity at zero — the
/// calibration probe shows contempt is NEVER produced in any calibrated
/// window (its appraisal input — `(1 − status_threat) × unfair` — only
/// fires under an unfair-other appraisal from a secure position, which
/// calm worlds never generate; probe mean/max 0.0000 across seeds and
/// scenarios), so the factor is exactly 1.0 throughout the
/// golden/snapshot horizons (provably zero-blast).
pub const CONTEMPT_ESCALATION_RATE: f64 = 0.3;

/// §8.1.4 (Iteration 122): the contempt escalation factor —
/// `1 + contempt × rate`, ∈ [1.0, 1.3] for the shipped constant. The
/// caller multiplies the escalation chance in `should_escalate` by the
/// returned factor. NO clamp (the Iter-112 lesson — `clamp_01` would
/// silently erase the amplification). One-sided: identity at zero,
/// monotone above. Deterministic, no RNG.
pub fn contempt_escalation_factor(contempt: Fixed, rate: f64) -> Fixed {
    Fixed::ONE + contempt * Fixed::from_f64(rate)
}

/// §8.1.4 (Iteration 122): the despair pacification rate and floor — a
/// despairing aggressor escalates a failed threat to violence LESS
/// readily ("nothing I do matters — violence cannot change this"). At
/// full despair 1.0 the escalation chance falls to exactly the floor
/// 0.5 (halved, never erased — hopelessness demobilizes but cannot
/// fully pacify an otherwise violent agent; same never-zero design as
/// the Iter-110 trust pacifier's cap). ONE-SIDED: identity at zero —
/// the calibration probe shows despair is NEVER produced in any
/// calibrated window (its appraisal input —
/// `future_negative × (1 − coping_potential)` — only fires when a
/// negative future implication combines with low coping, which calm
/// worlds never generate; probe mean/max 0.0000 across seeds and
/// scenarios), so the factor is exactly 1.0 throughout the
/// golden/snapshot horizons (provably zero-blast).
pub const DESPAIR_PACIFY_RATE: f64 = 0.5;
pub const DESPAIR_PACIFY_FLOOR: f64 = 0.5;

/// §8.1.4 (Iteration 122): the despair pacification factor —
/// `1 − despair × rate` floored at `floor`, ∈ [floor, 1.0] for the
/// shipped constants. The caller multiplies the escalation chance in
/// `should_escalate` by the returned factor. The floor guarantees the
/// pacification never fully erases the escalation chance. One-sided:
/// identity at zero, monotone below. Deterministic, no RNG.
pub fn despair_pacify_factor(despair: Fixed, rate: f64, floor: f64) -> Fixed {
    (Fixed::ONE - despair * Fixed::from_f64(rate)).max(Fixed::from_f64(floor))
}

pub fn appraise(appraisal: &Appraisal, _tick: Tick, params: &crate::parameters::SimParameters) -> EmotionDelta {
    let mut delta = EmotionDelta::default();

    if appraisal.goal_relevance > params.appraisal_goal_relevance_threshold {
        if appraisal.goal_congruence > Fixed::ZERO {
            // Goal-congruent: positive emotions
            delta.joy = appraisal.goal_relevance * appraisal.goal_congruence;

            match appraisal.agency {
                Agency::Self_ => {
                    delta.pride = appraisal.identity_relevance * Fixed::from_f64(0.5);
                }
                Agency::Other(_) => {
                    // Someone helped us → trust increase
                    delta.trust = Fixed::from_f64(0.1);
                }
                _ => {}
            }
        } else {
            // Goal-incongruent: negative emotions
            let intensity = appraisal.goal_relevance * appraisal.goal_congruence.abs();

            match appraisal.agency {
                Agency::Self_ => {
                    delta.guilt = intensity * appraisal.fairness.abs();
                    delta.shame = intensity * appraisal.identity_relevance;
                }
                Agency::Other(_) => {
                    delta.anger = intensity;
                    if appraisal.fairness < Fixed::ZERO {
                        // Unfair treatment intensifies anger
                        delta.anger = (delta.anger
                            + appraisal.fairness.abs() * Fixed::from_f64(0.3))
                            .clamp_01();
                    }
                }
                Agency::Circumstance | Agency::Unknown => {
                    delta.sadness = intensity * params.appraisal_sadness_multiplier;
                    delta.fear = intensity
                        * (Fixed::ONE - appraisal.coping_potential)
                        * params.appraisal_fear_coping_multiplier;
                }
            }
        }
    }

    // Low coping potential intensifies fear
    if appraisal.coping_potential < params.appraisal_low_coping_threshold {
        delta.fear = (delta.fear + Fixed::from_f64(0.1)).clamp_01();
    }

    // Unexpected events increase arousal (fear)
    if appraisal.expectedness < Fixed::from_f64(0.3) {
        delta.fear = (delta.fear + Fixed::from_f64(0.05)).clamp_01();
    }

    // ── §8.1.4: Expanded emotion families — derived from the deepened
    // appraisal dimensions. All pure and deterministic; the 8 core deltas
    // above are untouched, so calibrated trajectories stay byte-identical.
    let unfair = (-appraisal.fairness).max(Fixed::ZERO);
    let positive = appraisal.goal_congruence.max(Fixed::ZERO);
    let incongruent = (-appraisal.goal_congruence).max(Fixed::ZERO);
    let future_positive = appraisal.future_implication.max(Fixed::ZERO);
    let future_negative = (-appraisal.future_implication).max(Fixed::ZERO);

    // moral outrage — a sacred order violated.
    delta.moral_outrage = (appraisal.sacredness_violation * appraisal.goal_relevance).clamp_01();
    // disgust — purity violated.
    delta.disgust = (appraisal.purity_violation * appraisal.goal_relevance).clamp_01();
    // contempt — looking down on the unfair from a secure position.
    delta.contempt = ((Fixed::ONE - appraisal.status_threat) * unfair).clamp_01();
    // awe — overwhelming, unexpected significance.
    delta.awe = (appraisal.future_implication.abs() * appraisal.identity_relevance
        * (Fixed::ONE - appraisal.expectedness)).clamp_01();
    // gratitude — unexpected positive help.
    delta.gratitude = (positive * (Fixed::ONE - appraisal.expectedness)).clamp_01();
    // jealousy — threat to a valued attachment/status bond.
    delta.jealousy = (appraisal.status_threat * appraisal.attachment_threat).clamp_01();
    // envy — wanting what the higher-status other has.
    delta.envy = (appraisal.status_threat * incongruent).clamp_01();
    // loneliness — attachment threat with no visible social presence.
    delta.loneliness = (appraisal.attachment_threat
        * (Fixed::ONE - appraisal.social_visibility)).clamp_01();
    // tenderness — warm congruence toward close others.
    delta.tenderness = (positive * (Fixed::ONE - appraisal.status_threat)).clamp_01();
    // humiliation — public status loss.
    delta.humiliation = (appraisal.status_threat * appraisal.identity_relevance).clamp_01();
    // relief — an uncontrollable outcome turning out positive.
    delta.relief = (positive * (Fixed::ONE - appraisal.controllability)).clamp_01();
    // hope / despair — signed future implication split by coping.
    delta.hope = (future_positive * appraisal.coping_potential).clamp_01();
    delta.despair = (future_negative * (Fixed::ONE - appraisal.coping_potential)).clamp_01();
    // nostalgia — meaningful past woven into the narrative.
    delta.nostalgia = (appraisal.narrative_meaning * positive).clamp_01();

    delta
}

#[cfg(test)]
mod tests {
    use crate::parameters::SimParameters;
    use super::*;
    use mindstrata_core::rng::RngStreams;

    #[test]
    fn goal_congruent_produces_joy() {
        let _rng = RngStreams::new(42);
        let appraisal = Appraisal {
            goal_relevance: Fixed::from_f64(0.8),
            goal_congruence: Fixed::from_f64(0.7),
            coping_potential: Fixed::from_f64(0.6),
            expectedness: Fixed::from_f64(0.5),
            fairness: Fixed::from_f64(0.5),
            agency: Agency::Circumstance,
            social_visibility: Fixed::ZERO,
            identity_relevance: Fixed::ZERO,
            sacredness_violation: Fixed::ZERO,
            attachment_threat: Fixed::ZERO,
            status_threat: Fixed::ZERO,
            purity_violation: Fixed::ZERO,
            controllability: Fixed::ZERO,
            future_implication: Fixed::ZERO,
            narrative_meaning: Fixed::ZERO,
        };
        let delta = appraise(&appraisal, Tick::new(0), &SimParameters::default());
        assert!(delta.joy > Fixed::ZERO, "Should produce joy");
        assert_eq!(delta.anger, Fixed::ZERO, "Should not produce anger");
    }

    #[test]
    fn unfair_other_action_produces_anger() {
        let _rng = RngStreams::new(42);
        let appraisal = Appraisal {
            goal_relevance: Fixed::from_f64(0.8),
            goal_congruence: Fixed::from_f64(-0.5),
            coping_potential: Fixed::from_f64(0.3),
            expectedness: Fixed::from_f64(0.5),
            fairness: Fixed::from_f64(-0.7),
            agency: Agency::Other(AgentId::new(1)),
            social_visibility: Fixed::ZERO,
            identity_relevance: Fixed::ZERO,
            sacredness_violation: Fixed::ZERO,
            attachment_threat: Fixed::ZERO,
            status_threat: Fixed::ZERO,
            purity_violation: Fixed::ZERO,
            controllability: Fixed::ZERO,
            future_implication: Fixed::ZERO,
            narrative_meaning: Fixed::ZERO,
        };
        let delta = appraise(&appraisal, Tick::new(0), &SimParameters::default());
        assert!(
            delta.anger > Fixed::from_f64(0.3),
            "Should produce significant anger"
        );
    }

    #[test]
    fn low_coping_increases_fear() {
        let _rng = RngStreams::new(42);
        let appraisal = Appraisal {
            goal_relevance: Fixed::from_f64(0.6),
            goal_congruence: Fixed::from_f64(-0.3),
            coping_potential: Fixed::from_f64(0.1),
            expectedness: Fixed::from_f64(0.5),
            fairness: Fixed::ZERO,
            agency: Agency::Circumstance,
            social_visibility: Fixed::ZERO,
            identity_relevance: Fixed::ZERO,
            sacredness_violation: Fixed::ZERO,
            attachment_threat: Fixed::ZERO,
            status_threat: Fixed::ZERO,
            purity_violation: Fixed::ZERO,
            controllability: Fixed::ZERO,
            future_implication: Fixed::ZERO,
            narrative_meaning: Fixed::ZERO,
        };
        let delta = appraise(&appraisal, Tick::new(0), &SimParameters::default());
        assert!(
            delta.fear > Fixed::ZERO,
            "Low coping should increase fear"
        );
    }

    /// §8.1.4: The deepened appraisal dimensions drive the expanded emotion
    /// families — sacredness violation → moral outrage, purity violation →
    /// disgust, status threat + incongruence → envy, status threat + identity
    /// → humiliation, attachment threat → loneliness, and the signed
    /// future-implication dimension splits hope vs despair. Zero new
    /// dimensions must leave all 14 new deltas at zero (the 8 core emotions
    /// stay byte-identical).
    #[test]
    fn deepened_appraisal_dimensions_drive_expanded_emotions() {
        let params = SimParameters::default();
        let base = Appraisal {
            goal_relevance: Fixed::from_f64(0.8),
            goal_congruence: Fixed::from_f64(-0.5),
            coping_potential: Fixed::from_f64(0.5),
            expectedness: Fixed::from_f64(0.5),
            fairness: Fixed::from_f64(-0.7),
            agency: Agency::Other(AgentId::new(1)),
            social_visibility: Fixed::ZERO,
            identity_relevance: Fixed::from_f64(0.4),
            sacredness_violation: Fixed::from_f64(0.6),
            attachment_threat: Fixed::from_f64(0.5),
            status_threat: Fixed::from_f64(0.6),
            purity_violation: Fixed::from_f64(0.7),
            controllability: Fixed::ZERO,
            future_implication: Fixed::from_f64(0.5),
            narrative_meaning: Fixed::from_f64(0.8),
        };
        let delta = appraise(&base, Tick::new(0), &params);
        assert!(delta.moral_outrage > Fixed::ZERO, "sacredness violation → moral outrage");
        assert!(delta.disgust > Fixed::ZERO, "purity violation → disgust");
        assert!(delta.envy > Fixed::ZERO, "status threat + incongruence → envy");
        assert!(delta.humiliation > Fixed::ZERO, "status threat + identity → humiliation");
        assert!(delta.loneliness > Fixed::ZERO, "attachment threat → loneliness");
        assert!(delta.hope > Fixed::ZERO, "positive future implication → hope");
        assert_eq!(delta.despair, Fixed::ZERO, "positive future implication → no despair");

        // A fully-neutral appraisal (all dimensions at their Defaults, the
        // 8 core included) → all 14 new deltas at zero: the expanded families
        // only fire from the deepened dimensions or affective existing ones,
        // never leaking into the 8 core outputs.
        let d = appraise(&Appraisal::default(), Tick::new(0), &params);
        assert_eq!(d.disgust, Fixed::ZERO);
        assert_eq!(d.contempt, Fixed::ZERO);
        assert_eq!(d.awe, Fixed::ZERO);
        assert_eq!(d.gratitude, Fixed::ZERO);
        assert_eq!(d.jealousy, Fixed::ZERO);
        assert_eq!(d.envy, Fixed::ZERO);
        assert_eq!(d.loneliness, Fixed::ZERO);
        assert_eq!(d.tenderness, Fixed::ZERO);
        assert_eq!(d.humiliation, Fixed::ZERO);
        assert_eq!(d.relief, Fixed::ZERO);
        assert_eq!(d.hope, Fixed::ZERO);
        assert_eq!(d.despair, Fixed::ZERO);
        assert_eq!(d.nostalgia, Fixed::ZERO);
        assert_eq!(d.moral_outrage, Fixed::ZERO);
    }

    #[test]
    fn secondary_emotion_decay_is_proportional_and_exact() {
        // §8.1.4 (Iteration 116): `value × (1 − rate)` — a saturated 1.0
        // emotion decays to exactly 0.4 at a 0.6 rate; zero stays zero; the
        // decay is monotone in the value. Explicit rates (not the shipped
        // const) so the pure math is pinned independent of calibration.
        assert_eq!(
            secondary_emotion_decay(Fixed::ONE, 0.6),
            Fixed::from_f64(0.4),
            "1.0 × (1 − 0.6) must be exactly 0.4"
        );
        assert_eq!(
            secondary_emotion_decay(Fixed::from_f64(0.5), 0.6),
            Fixed::from_f64(0.2),
            "0.5 × (1 − 0.6) must be exactly 0.2"
        );
        assert_eq!(
            secondary_emotion_decay(Fixed::ZERO, 0.12),
            Fixed::ZERO
        );
        // The shipped per-tick rate must sit in the meaningful band (1–15%).
        assert!(
            (0.01..0.15).contains(&SECONDARY_EMOTION_DECAY_RATE),
            "per-tick rate must be 1–15%, got {SECONDARY_EMOTION_DECAY_RATE}"
        );
        let low = secondary_emotion_decay(Fixed::from_f64(0.3), 0.12);
        let high = secondary_emotion_decay(Fixed::from_f64(0.9), 0.12);
        assert!(low < high, "decay must be monotone in the value");
    }

    #[test]
    fn humiliation_escalation_factor_is_identity_at_zero_and_amplifies() {
        // §8.1.4 (Iteration 116): identity at zero (never produced in
        // calibrated windows → zero-blast), exact 1.15 at 0.5, exact 1.30
        // at full humiliation. NO clamp — the factor provably exceeds 1.0
        // (the Iter-112 lesson: `clamp_01` would erase the amplification).
        assert_eq!(
            humiliation_escalation_factor(Fixed::ZERO, HUMILIATION_ESCALATION_RATE),
            Fixed::ONE,
            "zero humiliation must be a byte-identical identity"
        );
        assert_eq!(
            humiliation_escalation_factor(Fixed::from_f64(0.5), HUMILIATION_ESCALATION_RATE),
            Fixed::from_f64(1.15),
            "0.5 × 0.3 must add exactly 0.15"
        );
        assert_eq!(
            humiliation_escalation_factor(Fixed::ONE, HUMILIATION_ESCALATION_RATE),
            Fixed::from_f64(1.3),
            "full humiliation × 0.3 must be exactly 1.30"
        );
    }

    #[test]
    fn contempt_escalation_factor_is_identity_at_zero_and_amplifies() {
        // §8.1.4 (Iteration 122): identity at zero (never produced in
        // calibrated windows → zero-blast), exact 1.15 at 0.5, exact 1.30
        // at full contempt. NO clamp — the factor provably exceeds 1.0
        // (the Iter-112 lesson: `clamp_01` would erase the amplification).
        assert_eq!(
            contempt_escalation_factor(Fixed::ZERO, CONTEMPT_ESCALATION_RATE),
            Fixed::ONE,
            "zero contempt must be a byte-identical identity"
        );
        assert_eq!(
            contempt_escalation_factor(Fixed::from_f64(0.5), CONTEMPT_ESCALATION_RATE),
            Fixed::from_f64(1.15),
            "0.5 × 0.3 must add exactly 0.15"
        );
        assert_eq!(
            contempt_escalation_factor(Fixed::ONE, CONTEMPT_ESCALATION_RATE),
            Fixed::from_f64(1.3),
            "full contempt × 0.3 must be exactly 1.30"
        );
    }

    #[test]
    fn despair_pacify_factor_is_identity_at_zero_and_floored() {
        // §8.1.4 (Iteration 122): identity at zero (never produced in
        // calibrated windows → zero-blast), exact 0.75 at 0.5, exact floor
        // 0.5 at full despair. The floor is load-bearing: the pacification
        // must never fully erase the escalation chance (same never-zero
        // design as the Iter-110 trust pacifier's cap).
        assert_eq!(
            despair_pacify_factor(
                Fixed::ZERO,
                DESPAIR_PACIFY_RATE,
                DESPAIR_PACIFY_FLOOR
            ),
            Fixed::ONE,
            "zero despair must be a byte-identical identity"
        );
        assert_eq!(
            despair_pacify_factor(
                Fixed::from_f64(0.5),
                DESPAIR_PACIFY_RATE,
                DESPAIR_PACIFY_FLOOR
            ),
            Fixed::from_f64(0.75),
            "1 − 0.5 × 0.5 must be exactly 0.75"
        );
        assert_eq!(
            despair_pacify_factor(
                Fixed::ONE,
                DESPAIR_PACIFY_RATE,
                DESPAIR_PACIFY_FLOOR
            ),
            Fixed::from_f64(0.5),
            "full despair must hit the exact floor 0.5"
        );
        // The floor must actually bind below the unfloored value.
        assert_eq!(
            despair_pacify_factor(Fixed::ONE, 0.8, DESPAIR_PACIFY_FLOOR),
            Fixed::from_f64(0.5),
            "floor 0.5 must bind at high rates"
        );
    }
}
