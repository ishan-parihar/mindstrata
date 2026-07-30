//! Interoceptive system — translates body state into felt experience.
//!
//! Interoception connects biology to affect. It answers the question:
//! "What does my body feel like right now?"
//!
//! ```text
//! felt_state = body_signals
//!            × attention_to_body
//!            × emotional_sensitivity
//!            × cultural_interpretation
//! ```
//!
//! Emergent effects:
//! - Anxious agents overinterpret bodily signals
//! - Stoic agents ignore injury
//! - Hunger becomes anger, fatigue becomes despair
//! - Arousal becomes attraction or shame

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Interoceptive accuracy — how well the agent reads their own body.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InteroceptiveState {
    /// Hunger awareness (0 = oblivious, 1 = hypervigilant).
    pub hunger_awareness: Fixed,
    /// Thirst awareness.
    pub thirst_awareness: Fixed,
    /// Fatigue awareness.
    pub fatigue_awareness: Fixed,
    /// Pain awareness.
    pub pain_awareness: Fixed,
    /// Emotional bodily tone awareness.
    pub emotional_awareness: Fixed,
    /// Overall interoceptive sensitivity.
    pub sensitivity: Fixed,
    /// Bias toward interpreting signals as negative (anxiety amplification).
    pub negative_bias: Fixed,
    /// Cultural interpretation modifier (some cultures emphasize body signals).
    pub cultural_modifier: Fixed,
}

impl Default for InteroceptiveState {
    fn default() -> Self {
        Self {
            hunger_awareness: Fixed::from_f64(0.5),
            thirst_awareness: Fixed::from_f64(0.5),
            fatigue_awareness: Fixed::from_f64(0.4),
            pain_awareness: Fixed::from_f64(0.5),
            emotional_awareness: Fixed::from_f64(0.4),
            sensitivity: Fixed::from_f64(0.5),
            negative_bias: Fixed::from_f64(0.3),
            cultural_modifier: Fixed::ONE,
        }
    }
}

impl InteroceptiveState {
    /// Compute felt hunger from actual hunger and interoceptive processing.
    pub fn felt_hunger(&self, actual_hunger: Fixed) -> Fixed {
        let awareness = self.hunger_awareness * self.cultural_modifier;
        let amplified = actual_hunger * awareness;
        // Negative bias amplifies distress signals
        let biased = amplified + self.negative_bias * Fixed::from_f64(0.1);
        biased.clamp_01()
    }

    /// Compute felt thirst from actual thirst.
    pub fn felt_thirst(&self, actual_thirst: Fixed) -> Fixed {
        let awareness = self.thirst_awareness * self.cultural_modifier;
        (actual_thirst * awareness + self.negative_bias * Fixed::from_f64(0.05)).clamp_01()
    }

    /// Compute felt fatigue from actual fatigue.
    pub fn felt_fatigue(&self, actual_fatigue: Fixed) -> Fixed {
        let awareness = self.fatigue_awareness * self.cultural_modifier;
        (actual_fatigue * awareness).clamp_01()
    }

    /// Compute felt pain from actual pain state.
    pub fn felt_pain(&self, actual_pain: Fixed) -> Fixed {
        let awareness = self.pain_awareness * self.sensitivity;
        let amplified = actual_pain * awareness;
        // Negative bias amplifies pain perception
        let biased = amplified + self.negative_bias * amplified * Fixed::from_f64(0.3);
        biased.clamp_01()
    }

    /// Compute emotional bodily tone — how much the agent "feels" their emotions in their body.
    pub fn emotional_body_tone(&self, valence: Fixed, arousal: Fixed) -> Fixed {
        let tone = self.emotional_awareness * (valence.abs() + arousal) * Fixed::from_f64(0.5);
        tone.clamp_01()
    }

    /// Initialize interoceptive state from personality traits and trauma history.
    /// Should be called once at agent creation, not per tick, since personality
    /// traits are static and trauma_load changes slowly.
    pub fn initialize_from_personality(
        &mut self,
        neuroticism: Fixed,
        openness: Fixed,
        trauma_load: Fixed,
    ) {
        // Neuroticism increases negative bias
        self.negative_bias = (neuroticism * Fixed::from_f64(0.4)
            + trauma_load * Fixed::from_f64(0.3)
            + Fixed::from_f64(0.1))
            .clamp_01();
        // Openness increases sensitivity
        self.sensitivity = (openness * Fixed::from_f64(0.3)
            + Fixed::from_f64(0.3))
            .clamp_01();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn felt_hunger_amplified_by_awareness() {
        let state = InteroceptiveState {
            hunger_awareness: Fixed::from_f64(0.8),
            ..InteroceptiveState::default()
        };
        let felt = state.felt_hunger(Fixed::from_f64(0.5));
        assert!(felt > Fixed::from_f64(0.3));
    }

    #[test]
    fn negative_bias_amplifies_pain() {
        let state_low_bias = InteroceptiveState {
            negative_bias: Fixed::from_f64(0.1),
            ..InteroceptiveState::default()
        };
        let state_high_bias = InteroceptiveState {
            negative_bias: Fixed::from_f64(0.8),
            ..InteroceptiveState::default()
        };
        let felt_low = state_low_bias.felt_pain(Fixed::from_f64(0.5));
        let felt_high = state_high_bias.felt_pain(Fixed::from_f64(0.5));
        assert!(felt_high > felt_low, "high negative bias should produce more felt pain");
    }

    #[test]
    fn neuroticism_increases_negative_bias() {
        let mut state = InteroceptiveState::default();
        state.initialize_from_personality(
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.5),
            Fixed::ZERO,
        );
        assert!(state.negative_bias > Fixed::from_f64(0.3));
    }
}
