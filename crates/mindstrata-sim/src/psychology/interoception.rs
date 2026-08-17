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

    /// §8.1: The *felt* need deficit — how much of the raw body-need deficit the
    /// agent registers after interoceptive filtering (awareness scales the
    /// signal, `negative_bias` amplifies distress).
    ///
    /// Baseline-corrected against [`Self::default`]: the default configuration is
    /// an exact no-op, so only personality-driven deviation (e.g. `negative_bias`
    /// from neuroticism) changes behavior. This keeps the simulation calibration
    /// transparent while making the §8.1 filters genuinely behavioral — an
    /// anxious agent feels the same hunger as more dire, feeding higher
    /// depression risk downstream.
    pub fn felt_need_deficit(
        &self,
        hunger: Fixed,
        thirst: Fixed,
        fatigue: Fixed,
        safety: Fixed,
    ) -> Fixed {
        let raw = (hunger + thirst + fatigue + safety) * Fixed::from_f64(0.25);
        let felt = (self.felt_hunger(hunger)
            + self.felt_thirst(thirst)
            + self.felt_fatigue(fatigue)
            + safety)
            * Fixed::from_f64(0.25);
        let baseline = Self::default();
        let baseline_felt = (baseline.felt_hunger(hunger)
            + baseline.felt_thirst(thirst)
            + baseline.felt_fatigue(fatigue)
            + safety)
            * Fixed::from_f64(0.25);
        // Amplify the deviation-from-default distortion so it is behaviorally
        // visible while remaining exactly zero at the baseline configuration.
        (raw + (felt - baseline_felt) * Fixed::from_f64(2.0)).clamp_01()
    }

    /// §8.1: Emotion-regulation efficacy scale — how much the body's
    /// embodiment of the current emotion resists cognitive regulation.
    ///
    /// High interoceptive sensitivity means emotions are felt more intensely
    /// in the body, so cognitive strategies bite less; low sensitivity leaves
    /// emotions "cognitive", so regulation works better. Baseline-corrected
    /// against the default sensitivity (0.5): returns exactly 1.0 (a no-op) at
    /// the baseline, so only personality-driven deviation changes behavior.
    pub fn regulation_scale(&self, valence: Fixed, arousal: Fixed) -> Fixed {
        let tone = self.emotional_body_tone(valence, arousal);
        let deviation = self.sensitivity - Fixed::from_f64(0.5);
        // ±~16% at the extremes of the initialized sensitivity range [0.3, 0.6]
        // combined with intense emotion; the clamp is defensive only.
        (Fixed::ONE - deviation * tone * Fixed::from_f64(2.0))
            .clamp(Fixed::from_f64(0.5), Fixed::from_f64(1.5))
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
        self.sensitivity = (openness * Fixed::from_f64(0.3) + Fixed::from_f64(0.3)).clamp_01();
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
    fn felt_need_deficit_is_identity_at_default_configuration() {
        let state = InteroceptiveState::default();
        let (h, t, f, s) = (
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        );
        let raw = (h + t + f + s) * Fixed::from_f64(0.25);
        let felt = state.felt_need_deficit(h, t, f, s);
        assert_eq!(felt, raw, "default interoception must be an exact no-op");
    }

    #[test]
    fn felt_need_deficit_amplifies_with_negative_bias() {
        let (h, t, f, s) = (
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        );
        let raw = (h + t + f + s) * Fixed::from_f64(0.25);
        let low = InteroceptiveState {
            negative_bias: Fixed::from_f64(0.0),
            ..InteroceptiveState::default()
        };
        let high = InteroceptiveState {
            negative_bias: Fixed::from_f64(0.9),
            ..InteroceptiveState::default()
        };
        assert!(
            low.felt_need_deficit(h, t, f, s) < raw,
            "low bias numbs the felt deficit"
        );
        assert!(
            high.felt_need_deficit(h, t, f, s) > raw,
            "high bias amplifies the felt deficit"
        );
    }

    #[test]
    fn regulation_scale_is_identity_at_default_sensitivity() {
        let state = InteroceptiveState::default();
        let scale = state.regulation_scale(Fixed::from_f64(0.4), Fixed::from_f64(0.6));
        assert_eq!(
            scale,
            Fixed::ONE,
            "default sensitivity must not change regulation"
        );
    }

    #[test]
    fn regulation_scale_penalizes_high_sensitivity() {
        let high = InteroceptiveState {
            sensitivity: Fixed::from_f64(0.9),
            ..InteroceptiveState::default()
        };
        let low = InteroceptiveState {
            sensitivity: Fixed::from_f64(0.1),
            ..InteroceptiveState::default()
        };
        let (v, a) = (Fixed::from_f64(0.5), Fixed::from_f64(0.6));
        assert!(
            high.regulation_scale(v, a) < Fixed::ONE,
            "embodied emotions resist regulation"
        );
        assert!(
            low.regulation_scale(v, a) > Fixed::ONE,
            "detached emotions regulate more easily"
        );
        // No emotion → no modulation, regardless of sensitivity.
        assert_eq!(high.regulation_scale(Fixed::ZERO, Fixed::ZERO), Fixed::ONE);
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
        assert!(
            felt_high > felt_low,
            "high negative bias should produce more felt pain"
        );
    }

    #[test]
    fn neuroticism_increases_negative_bias() {
        let mut state = InteroceptiveState::default();
        state.initialize_from_personality(Fixed::from_f64(0.8), Fixed::from_f64(0.5), Fixed::ZERO);
        assert!(state.negative_bias > Fixed::from_f64(0.3));
    }

    #[test]
    fn trauma_load_amplifies_negative_bias() {
        let mut low = InteroceptiveState::default();
        low.initialize_from_personality(Fixed::from_f64(0.5), Fixed::from_f64(0.5), Fixed::ZERO);
        let mut high = InteroceptiveState::default();
        high.initialize_from_personality(
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        );
        let delta = high.negative_bias - low.negative_bias;
        assert!(delta > Fixed::from_f64(0.08));
        assert!(delta < Fixed::from_f64(0.10));
    }
}
