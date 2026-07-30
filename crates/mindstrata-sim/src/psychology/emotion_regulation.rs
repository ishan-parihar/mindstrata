//! Emotion regulation system — strategies agents use to manage their emotional states.
//!
//! Agents don't just feel emotions; they regulate them. The strategy chosen depends
//! on personality, stress level, social support, culture, and skill.
//!
//! Regulation success depends on: personality, stress, social support, culture, skill, exhaustion.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Available emotion regulation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RegulationStrategy {
    /// Reframe the situation to change its emotional meaning.
    Reappraisal,
    /// Hide emotional expression without changing the feeling.
    Suppression,
    /// Repeat and dwell on negative thoughts.
    Rumination,
    /// Avoid the source of negative emotion.
    Avoidance,
    /// Seek comfort from trusted others.
    SeekingSupport,
    /// Turn to ritual or prayer for comfort.
    Prayer,
    /// Use ritual participation for emotional soothing.
    Ritual,
    /// Discharge emotion through aggression.
    Aggression,
    /// Use humor to defuse tension.
    Humor,
    /// Psychologically detach from the situation.
    Dissociation,
    /// Throw yourself into productive work.
    Work,
    /// Care for others to restore sense of purpose.
    Caregiving,
}

impl Default for RegulationStrategy {
    fn default() -> Self {
        Self::Reappraisal
    }
}

/// Agent's emotion regulation profile and current state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmotionRegulationState {
    /// Preferred strategy (most practiced).
    pub preferred: RegulationStrategy,
    /// Skill level in reappraisal (0–1).
    pub reappraisal_skill: Fixed,
    /// Skill level in suppression (0–1).
    pub suppression_skill: Fixed,
    /// Tendency toward rumination (high = ruminator).
    pub rumination_tendency: Fixed,
    /// Tendency toward avoidance (high = avoidant).
    pub avoidance_tendency: Fixed,
    /// Receptivity to social support (high = seeks comfort).
    pub support_receptivity: Fixed,
    /// Overall regulation capacity (0 = overwhelmed, 1 = excellent regulator).
    pub capacity: Fixed,
    /// Current regulation effort (high = actively trying to manage emotions).
    pub current_effort: Fixed,
}

impl Default for EmotionRegulationState {
    fn default() -> Self {
        Self {
            preferred: RegulationStrategy::Reappraisal,
            reappraisal_skill: Fixed::from_f64(0.4),
            suppression_skill: Fixed::from_f64(0.3),
            rumination_tendency: Fixed::from_f64(0.3),
            avoidance_tendency: Fixed::from_f64(0.2),
            support_receptivity: Fixed::from_f64(0.5),
            capacity: Fixed::from_f64(0.5),
            current_effort: Fixed::ZERO,
        }
    }
}

impl EmotionRegulationState {
    /// Select the best regulation strategy given current conditions.
    pub fn select_strategy(
        &self,
        stress: Fixed,
        social_support: Fixed,
        extraversion: Fixed,
    ) -> RegulationStrategy {
        // High stress + high support → SeekingSupport
        if stress > Fixed::from_f64(0.6) && social_support > Fixed::from_f64(0.5) {
            return RegulationStrategy::SeekingSupport;
        }
        // High stress + low support + high rumination → Rumination (maladaptive)
        if stress > Fixed::from_f64(0.6) && self.rumination_tendency > Fixed::from_f64(0.6) {
            return RegulationStrategy::Rumination;
        }
        // Moderate stress + good reappraisal skill → Reappraisal (adaptive)
        if stress > Fixed::from_f64(0.3) && self.reappraisal_skill > Fixed::from_f64(0.4) {
            return RegulationStrategy::Reappraisal;
        }
        // High extraversion → Humor or Social
        if extraversion > Fixed::from_f64(0.7) {
            return RegulationStrategy::Humor;
        }
        // Default to preferred strategy
        self.preferred
    }

    /// Apply a regulation strategy and return the emotional dampening effect.
    /// Returns (valence_change, arousal_change).
    pub fn apply_strategy(
        &mut self,
        strategy: RegulationStrategy,
        current_valence: Fixed,
        current_arousal: Fixed,
    ) -> (Fixed, Fixed) {
        self.current_effort = Fixed::from_f64(0.3);
        match strategy {
            RegulationStrategy::Reappraisal => {
                // Reappraisal reduces negative valence and arousal
                let skill = self.reappraisal_skill;
                let valence_boost = current_valence.abs() * skill * Fixed::from_f64(0.1);
                let arousal_reduce = current_arousal * skill * Fixed::from_f64(0.15);
                self.reappraisal_skill = (self.reappraisal_skill + Fixed::from_f64(0.001)).clamp_01();
                (valence_boost, -arousal_reduce)
            }
            RegulationStrategy::Suppression => {
                // Suppression reduces expressed arousal but not valence
                let skill = self.suppression_skill;
                let arousal_reduce = current_arousal * skill * Fixed::from_f64(0.2);
                self.suppression_skill = (self.suppression_skill + Fixed::from_f64(0.001)).clamp_01();
                (Fixed::ZERO, -arousal_reduce)
            }
            RegulationStrategy::Rumination => {
                // Rumination increases negative valence and arousal (maladaptive)
                let ruminate = self.rumination_tendency;
                let valence_worsen = Fixed::from_f64(0.05) * ruminate;
                (valence_worsen, Fixed::from_f64(0.03) * ruminate)
            }
            RegulationStrategy::Avoidance => {
                // Avoidance temporarily reduces arousal but doesn't resolve the issue
                let arousal_reduce = current_arousal * Fixed::from_f64(0.1);
                (Fixed::ZERO, -arousal_reduce)
            }
            RegulationStrategy::SeekingSupport => {
                // Social support improves valence and reduces arousal (if available)
                let boost = self.support_receptivity * Fixed::from_f64(0.08);
                (boost, -Fixed::from_f64(0.05))
            }
            RegulationStrategy::Prayer | RegulationStrategy::Ritual => {
                // Ritual reduces arousal through predictability and meaning
                (-Fixed::from_f64(0.03), -Fixed::from_f64(0.06))
            }
            RegulationStrategy::Aggression => {
                // Aggression discharges arousal but may worsen valence
                (-Fixed::from_f64(0.02), -current_arousal * Fixed::from_f64(0.15))
            }
            RegulationStrategy::Humor => {
                // Humor improves valence and reduces arousal
                (Fixed::from_f64(0.05), -Fixed::from_f64(0.04))
            }
            RegulationStrategy::Dissociation => {
                // Dissociation reduces all emotional experience
                (-Fixed::from_f64(0.1), -current_arousal * Fixed::from_f64(0.3))
            }
            RegulationStrategy::Work => {
                // Work provides distraction and sense of purpose
                (Fixed::from_f64(0.02), -Fixed::from_f64(0.03))
            }
            RegulationStrategy::Caregiving => {
                // Caregiving restores meaning and positive affect
                (Fixed::from_f64(0.04), -Fixed::from_f64(0.02))
            }
        }
    }

    /// Update regulation capacity based on stress, fatigue, and social support.
    pub fn update_capacity(&mut self, stress: Fixed, fatigue: Fixed, social_support: Fixed) {
        // Capacity is reduced by stress and fatigue, boosted by social support
        self.capacity = (Fixed::from_f64(0.5)
            + social_support * Fixed::from_f64(0.2)
            - stress * Fixed::from_f64(0.3)
            - fatigue * Fixed::from_f64(0.2))
            .clamp_01();
        // Effort decays each tick
        self.current_effort = (self.current_effort - Fixed::from_f64(0.05)).max(Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reappraisal_reduces_arousal() {
        let mut reg = EmotionRegulationState {
            reappraisal_skill: Fixed::from_f64(0.8),
            ..EmotionRegulationState::default()
        };
        let (_, arousal_change) = reg.apply_strategy(
            RegulationStrategy::Reappraisal,
            Fixed::from_f64(-0.5),
            Fixed::from_f64(0.7),
        );
        assert!(arousal_change < Fixed::ZERO);
    }

    #[test]
    fn rumination_worsens_valence() {
        let mut reg = EmotionRegulationState {
            rumination_tendency: Fixed::from_f64(0.8),
            ..EmotionRegulationState::default()
        };
        let (valence_change, _) = reg.apply_strategy(
            RegulationStrategy::Rumination,
            Fixed::from_f64(-0.3),
            Fixed::from_f64(0.5),
        );
        assert!(valence_change > Fixed::ZERO); // worsens negative valence
    }

    #[test]
    fn high_stress_chooses_support() {
        let reg = EmotionRegulationState::default();
        let strategy = reg.select_strategy(
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
        );
        assert_eq!(strategy, RegulationStrategy::SeekingSupport);
    }
}
