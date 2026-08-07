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
}
