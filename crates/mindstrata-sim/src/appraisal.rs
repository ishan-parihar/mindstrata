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
        }
    }
}

/// Derive emotional response from an appraisal.
///
/// This implements the cognitive appraisal → emotion mapping from
/// the architecture spec's Section 9.2.
pub fn appraise(appraisal: &Appraisal, _tick: Tick) -> EmotionDelta {
    let mut delta = EmotionDelta::default();

    if appraisal.goal_relevance > Fixed::from_f64(0.3) {
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
                    delta.sadness = intensity * Fixed::from_f64(0.7);
                    delta.fear = intensity
                        * (Fixed::ONE - appraisal.coping_potential)
                        * Fixed::from_f64(0.5);
                }
            }
        }
    }

    // Low coping potential intensifies fear
    if appraisal.coping_potential < Fixed::from_f64(0.3) {
        delta.fear = (delta.fear + Fixed::from_f64(0.1)).clamp_01();
    }

    // Unexpected events increase arousal (fear)
    if appraisal.expectedness < Fixed::from_f64(0.3) {
        delta.fear = (delta.fear + Fixed::from_f64(0.05)).clamp_01();
    }

    delta
}

#[cfg(test)]
mod tests {
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
        };
        let delta = appraise(&appraisal, Tick::new(0));
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
        };
        let delta = appraise(&appraisal, Tick::new(0));
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
        };
        let delta = appraise(&appraisal, Tick::new(0));
        assert!(
            delta.fear > Fixed::ZERO,
            "Low coping should increase fear"
        );
    }
}
