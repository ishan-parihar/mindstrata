//! Theory of Mind system — modeling other agents' minds.
//!
//! Agents should infer:
//! - What others know
//! - What others want
//! - What others feel
//! - Whether others are hostile
//! - Whether others are lying
//! - Whether others share identity
//!
//! Errors are important:
//! - Misattribution
//! - Paranoia
//! - Projection
//! - Idealization
//! - Dehumanization
//! - False consensus

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Hypothesis about another agent's goal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GoalHypothesis {
    /// What goal we think they're pursuing.
    pub goal_description: String,
    /// How confident we are in this hypothesis.
    pub confidence: Fixed,
}

/// Hypothesis about another agent's belief.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefHypothesis {
    /// What belief we think they hold.
    pub proposition_id: u64,
    /// How confident we are.
    pub confidence: Fixed,
    /// Whether we think their belief is accurate.
    pub perceived_accuracy: bool,
}

/// Model of another agent's mind.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OtherMindModel {
    /// Which agent this model represents.
    pub target: AgentId,
    /// Perceived traits (may differ from actual).
    pub perceived_traits: PerceivedTraits,
    /// Hypotheses about their goals.
    pub goal_hypotheses: Vec<GoalHypothesis>,
    /// Hypotheses about their beliefs.
    pub belief_hypotheses: Vec<BeliefHypothesis>,
    /// Perceived emotional state.
    pub perceived_emotion: PerceivedEmotion,
    /// Perceived intent toward us.
    pub perceived_intent: IntentPerception,
    /// Trust in this model's accuracy.
    pub model_trust: Fixed,
    /// Projection bias — how much we assume they think like us.
    pub projection_bias: Fixed,
    /// Empathy — how much we feel what they feel.
    pub empathy: Fixed,
    /// Dehumanization level — reduces perceived complexity of their mind.
    pub dehumanization: Fixed,
}

/// Simplified perception of another's traits.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerceivedTraits {
    pub trustworthiness: Fixed,
    pub competence: Fixed,
    pub warmth: Fixed,
    pub threat_level: Fixed,
}

/// Perception of another's emotional state.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PerceivedEmotion {
    pub valence: Fixed,
    pub arousal: Fixed,
    pub is_hostile: bool,
}

/// What we think another intends toward us.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum IntentPerception {
    Friendly,
    Neutral,
    Threatening,
    Deceptive,
    #[default]
    Unknown,
}

impl OtherMindModel {
    /// Create a new model for an agent.
    pub fn new(target: AgentId) -> Self {
        Self {
            target,
            perceived_traits: PerceivedTraits::default(),
            goal_hypotheses: Vec::new(),
            belief_hypotheses: Vec::new(),
            perceived_emotion: PerceivedEmotion::default(),
            perceived_intent: IntentPerception::Unknown,
            model_trust: Fixed::from_f64(0.3),
            projection_bias: Fixed::from_f64(0.3),
            empathy: Fixed::from_f64(0.4),
            dehumanization: Fixed::ZERO,
        }
    }

    /// Update model based on observed behavior.
    pub fn update_from_observation(
        &mut self,
        observed_helpful: Fixed,
        observed_harmful: Fixed,
        observed_honesty: Fixed,
        empathy_modifier: Fixed,
    ) {
        // Update trustworthiness perception
        let trust_delta = observed_honesty * Fixed::from_f64(0.1)
            - observed_harmful * Fixed::from_f64(0.15);
        self.perceived_traits.trustworthiness =
            (self.perceived_traits.trustworthiness + trust_delta).clamp_01();

        // Update warmth perception
        let warmth_delta = observed_helpful * Fixed::from_f64(0.1)
            - observed_harmful * Fixed::from_f64(0.1);
        self.perceived_traits.warmth =
            (self.perceived_traits.warmth + warmth_delta).clamp_01();

        // Update threat level
        self.perceived_traits.threat_level =
            (self.perceived_traits.threat_level + observed_harmful * Fixed::from_f64(0.2)
                - observed_helpful * Fixed::from_f64(0.1))
                .clamp_01();

        // Update empathy through repeated interaction
        self.empathy = (self.empathy + empathy_modifier * Fixed::from_f64(0.01)).clamp_01();

        // Projection bias decays with more information
        self.projection_bias = (self.projection_bias - Fixed::from_f64(0.005)).max(Fixed::ZERO);
    }

    /// Infer another's intent based on observed behavior and relationship.
    pub fn infer_intent(
        &mut self,
        trust: Fixed,
        recent_positive: Fixed,
        recent_negative: Fixed,
    ) {
        if recent_negative > Fixed::from_f64(0.3) {
            self.perceived_intent = IntentPerception::Threatening;
        } else if recent_positive > Fixed::from_f64(0.3) && trust > Fixed::from_f64(0.5) {
            self.perceived_intent = IntentPerception::Friendly;
        } else if trust < Fixed::from_f64(0.2) {
            self.perceived_intent = IntentPerception::Deceptive;
        } else {
            self.perceived_intent = IntentPerception::Neutral;
        }
    }

    /// Check if this agent is perceived as an in-group member.
    pub fn is_in_group(
        &self,
        shared_identity: Fixed,
        trust: Fixed,
    ) -> bool {
        shared_identity > Fixed::from_f64(0.5) || trust > Fixed::from_f64(0.7)
    }

    /// Dehumanize — reduce perceived mental complexity.
    pub fn dehumanize(&mut self, severity: Fixed) {
        self.dehumanization = (self.dehumanization + severity).clamp_01();
        // Dehumanization reduces empathy
        self.empathy = (self.empathy - severity * Fixed::from_f64(0.3)).max(Fixed::ZERO);
    }
}

/// Collection of mind models for all known agents.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MindModels {
    pub models: Vec<OtherMindModel>,
}

impl MindModels {
    /// Get or create a model for an agent.
    pub fn get_or_create(&mut self, target: AgentId) -> &mut OtherMindModel {
        if !self.models.iter().any(|m| m.target == target) {
            self.models.push(OtherMindModel::new(target));
        }
        self.models.iter_mut().find(|m| m.target == target).expect("just pushed model for target")
    }

    /// Get a model for an agent, if it exists.
    pub fn get(&self, target: AgentId) -> Option<&OtherMindModel> {
        self.models.iter().find(|m| m.target == target)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_model_starts_neutral() {
        let model = OtherMindModel::new(AgentId::new(1));
        assert_eq!(model.perceived_intent, IntentPerception::Unknown);
        assert!(model.dehumanization == Fixed::ZERO);
    }

    #[test]
    fn update_builds_trust_from_helpful_behavior() {
        let mut model = OtherMindModel::new(AgentId::new(1));
        let initial = model.perceived_traits.trustworthiness;
        model.update_from_observation(
            Fixed::from_f64(0.8), // helpful
            Fixed::ZERO,          // not harmful
            Fixed::from_f64(0.9), // honest
            Fixed::from_f64(0.5),
        );
        assert!(model.perceived_traits.trustworthiness > initial);
    }

    #[test]
    fn dehumanization_reduces_empathy() {
        let mut model = OtherMindModel::new(AgentId::new(1));
        model.empathy = Fixed::from_f64(0.8);
        model.dehumanize(Fixed::from_f64(0.5));
        assert!(model.empathy < Fixed::from_f64(0.8));
    }

    #[test]
    fn mind_models_collection_works() {
        let mut models = MindModels::default();
        let _m = models.get_or_create(AgentId::new(1));
        let _m = models.get_or_create(AgentId::new(2));
        assert_eq!(models.models.len(), 2);
        // Getting same agent doesn't duplicate
        let _m = models.get_or_create(AgentId::new(1));
        assert_eq!(models.models.len(), 2);
    }
}
