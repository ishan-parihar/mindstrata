//! Emotion regulation system — strategies agents use to manage their emotional states.
//!
//! Agents don't just feel emotions; they regulate them. The strategy chosen depends
//! on personality, stress level, social support, culture, and skill.
//!
//! Regulation success depends on: personality, stress, social support, culture, skill, exhaustion.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Available emotion regulation strategies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RegulationStrategy {
    /// Reframe the situation to change its emotional meaning.
    #[default]
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
    /// Self-medicate with intoxicants (when the world offers them).
    SubstanceUse,
    /// Throw yourself into productive work.
    Work,
    /// Care for others to restore sense of purpose.
    Caregiving,
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
    /// §8.1.8 (P3-4, August 14, 2026): personality-driven strategy
    /// preference at birth. Previously `preferred` was NEVER assigned
    /// after construction, so every agent was permanently Reappraisal
    /// (probe-pinned 12/12 in every window) — the plan's "strategy
    /// selection from personality" was unwired. Seeds the preference
    /// from the big-five profile; the dynamic `select_strategy` state
    /// branches still override when genuine overwhelm hits.
    /// Deterministic, no RNG.
    pub fn initialize_from_personality(&mut self, p: &crate::person::Personality) {
        self.preferred = if p.extraversion > Fixed::from_f64(0.6) {
            RegulationStrategy::Humor
        } else if p.agreeableness > Fixed::from_f64(0.6) {
            RegulationStrategy::SeekingSupport
        } else if p.neuroticism > Fixed::from_f64(0.55) {
            RegulationStrategy::Suppression
        } else {
            RegulationStrategy::Reappraisal
        };
    }

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
        // §8.1.8 (P3-4): extraverts reach for humor at moderate stress —
        // this branch was UNREACHABLE behind the reappraisal branch below
        // (every agent with stress > 0.3 and reappraisal_skill > 0.4 — i.e.
        // everyone after one applied strategy — locked into Reappraisal).
        if extraversion > Fixed::from_f64(0.65) {
            return RegulationStrategy::Humor;
        }
        // Moderate stress + good reappraisal skill → Reappraisal (adaptive)
        if stress > Fixed::from_f64(0.3) && self.reappraisal_skill > Fixed::from_f64(0.4) {
            return RegulationStrategy::Reappraisal;
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
        // §8.1.8 (P3-4): effort scales with the emotion load being
        // regulated — the fixed 0.3 was write-only constant state
        // (identical for every agent, every tick). The load is the
        // stronger of |valence| and arousal: calm agents exert light
        // effort, agitated agents exert heavy effort. Observational (no
        // consumer reads current_effort), deterministic, no RNG.
        let load = current_valence.abs().max(current_arousal);
        self.current_effort = (Fixed::from_f64(0.2) + load * Fixed::from_f64(0.4)).clamp_01();
        match strategy {
            RegulationStrategy::Reappraisal => {
                // Reappraisal reduces negative valence and arousal
                let skill = self.reappraisal_skill;
                let valence_boost = current_valence.abs() * skill * Fixed::from_f64(0.1);
                let arousal_reduce = current_arousal * skill * Fixed::from_f64(0.15);
                self.reappraisal_skill =
                    (self.reappraisal_skill + Fixed::from_f64(0.001)).clamp_01();
                (valence_boost, -arousal_reduce)
            }
            RegulationStrategy::Suppression => {
                // Suppression reduces expressed arousal but not valence
                let skill = self.suppression_skill;
                let arousal_reduce = current_arousal * skill * Fixed::from_f64(0.2);
                self.suppression_skill =
                    (self.suppression_skill + Fixed::from_f64(0.001)).clamp_01();
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
                (
                    -Fixed::from_f64(0.02),
                    -current_arousal * Fixed::from_f64(0.15),
                )
            }
            RegulationStrategy::Humor => {
                // Humor improves valence and reduces arousal
                (Fixed::from_f64(0.05), -Fixed::from_f64(0.04))
            }
            RegulationStrategy::Dissociation => {
                // Dissociation reduces all emotional experience
                (
                    -Fixed::from_f64(0.1),
                    -current_arousal * Fixed::from_f64(0.3),
                )
            }
            RegulationStrategy::SubstanceUse => {
                // Intoxicants blunt both poles but carry dependency risk
                (
                    -Fixed::from_f64(0.04),
                    -current_arousal * Fixed::from_f64(0.25),
                )
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
        self.capacity = (Fixed::from_f64(0.5) + social_support * Fixed::from_f64(0.2)
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

    #[test]
    fn extraversion_branch_reachable_before_reappraisal() {
        // P3-4 regression: the extravert branch sat behind the reappraisal
        // branch (everyone with stress > 0.3 && skill > 0.4 locked into
        // Reappraisal, probe-pinned 12/12). A high-extraversion agent with
        // moderate stress and a grown reappraisal skill must still choose
        // Humor.
        let reg = EmotionRegulationState {
            reappraisal_skill: Fixed::from_f64(0.9),
            ..EmotionRegulationState::default()
        };
        let strategy = reg.select_strategy(
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.8),
        );
        assert_eq!(strategy, RegulationStrategy::Humor);
    }

    #[test]
    fn preferred_seeded_from_personality() {
        // P3-4 regression: `preferred` was never assigned, so every agent
        // was permanently Reappraisal. Personality seeding must diversify.
        fn personality(
            extraversion: f64,
            agreeableness: f64,
            neuroticism: f64,
        ) -> crate::person::Personality {
            crate::person::Personality {
                openness: Fixed::from_f64(0.5),
                conscientiousness: Fixed::from_f64(0.5),
                extraversion: Fixed::from_f64(extraversion),
                agreeableness: Fixed::from_f64(agreeableness),
                neuroticism: Fixed::from_f64(neuroticism),
                risk_tolerance: Fixed::from_f64(0.5),
                conformity: Fixed::from_f64(0.5),
                ambition: Fixed::from_f64(0.5),
                altruism: Fixed::from_f64(0.5),
                traditionalism: Fixed::from_f64(0.5),
                dominance: Fixed::from_f64(0.5),
                impulsivity: Fixed::from_f64(0.5),
                temperament: crate::person::Temperament::default(),
                constitution: None,
            }
        }
        let mut extravert = EmotionRegulationState::default();
        extravert.initialize_from_personality(&personality(0.8, 0.5, 0.5));
        assert_eq!(extravert.preferred, RegulationStrategy::Humor);

        let mut agreeable = EmotionRegulationState::default();
        agreeable.initialize_from_personality(&personality(0.5, 0.8, 0.5));
        assert_eq!(agreeable.preferred, RegulationStrategy::SeekingSupport);

        let mut neurotic = EmotionRegulationState::default();
        neurotic.initialize_from_personality(&personality(0.5, 0.5, 0.7));
        assert_eq!(neurotic.preferred, RegulationStrategy::Suppression);
    }

    #[test]
    fn effort_scales_with_emotion_load() {
        // P3-4 regression: current_effort was a fixed 0.3 write-only
        // constant; it must now scale with the regulated load.
        let mut reg = EmotionRegulationState::default();
        let (_, _) = reg.apply_strategy(
            RegulationStrategy::Reappraisal,
            Fixed::from_f64(-0.1),
            Fixed::from_f64(0.1),
        );
        let calm_effort = reg.current_effort;
        let (_, _) = reg.apply_strategy(
            RegulationStrategy::Reappraisal,
            Fixed::from_f64(-0.9),
            Fixed::from_f64(0.9),
        );
        assert!(reg.current_effort > calm_effort);
        assert!(reg.current_effort > Fixed::ZERO && reg.current_effort <= Fixed::ONE);
    }
}
