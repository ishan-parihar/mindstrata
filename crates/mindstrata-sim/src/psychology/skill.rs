//! Skill and habit system — agents learn skills through practice and form habits.
//!
//! Skills: farming, cooking, healing, trading, fighting, speaking, leadership,
//! ritual, crafting, parenting, deception, diplomacy.
//!
//! Habits form through repetition and stress. Under stress, agents fall back on habits.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Skill identifier.
pub type SkillId = u32;

/// Skill level with metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillLevel {
    /// Current proficiency (0 = novice, 1 = master).
    pub proficiency: Fixed,
    /// Total practice ticks invested.
    pub practice_ticks: u32,
    /// Last tick when practiced.
    pub last_practiced: u64,
    /// Whether this skill is currently being actively used.
    pub active: bool,
}

impl SkillLevel {
    pub fn new() -> Self {
        Self {
            proficiency: Fixed::ZERO,
            practice_ticks: 0,
            last_practiced: 0,
            active: false,
        }
    }
}

impl Default for SkillLevel {
    fn default() -> Self {
        Self::new()
    }
}

/// A habit — an automatic behavioral pattern.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Habit {
    /// Description of the habitual behavior.
    pub description: String,
    /// Strength of the habit (0 = weak, 1 = automatic).
    pub strength: Fixed,
    /// Number of times this habit has been performed.
    pub repetition_count: u32,
    /// Last tick when performed.
    pub last_performed: u64,
    /// Associated need or trigger.
    pub trigger: String,
}

/// Complete skill and habit state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillState {
    /// All skills with their levels.
    pub skills: HashMap<SkillId, SkillLevel>,
    /// Formed habits.
    pub habits: Vec<Habit>,
    /// Overall automaticity — how much behavior is habitual vs deliberate.
    pub automaticity: Fixed,
    /// Neuroplasticity — how easily new skills are learned.
    pub neuroplasticity: Fixed,
}

impl Default for SkillState {
    fn default() -> Self {
        Self {
            skills: HashMap::new(),
            habits: Vec::new(),
            automaticity: Fixed::from_f64(0.3),
            neuroplasticity: Fixed::from_f64(0.6),
        }
    }
}

impl SkillState {
    /// Practice a skill, improving proficiency.
    pub fn practice(&mut self, skill_id: SkillId, tick: u64, neuroplasticity: Fixed) {
        let skill = self.skills.entry(skill_id).or_default();
        // Proficiency gain depends on current level (harder to improve at high levels)
        let difficulty_modifier = Fixed::ONE - skill.proficiency * Fixed::from_f64(0.5);
        let gain = difficulty_modifier * neuroplasticity * Fixed::from_f64(0.001);
        skill.proficiency = (skill.proficiency + gain).clamp_01();
        skill.practice_ticks += 1;
        skill.last_practiced = tick;
    }

    /// Get proficiency for a skill (0 if not learned).
    pub fn proficiency(&self, skill_id: SkillId) -> Fixed {
        self.skills
            .get(&skill_id)
            .map_or(Fixed::ZERO, |s| s.proficiency)
    }

    /// Form a new habit through repetition.
    pub fn form_habit(
        &mut self,
        description: String,
        trigger: String,
        strength: Fixed,
        tick: u64,
    ) {
        // Check if habit already exists
        if self.habits.iter().any(|h| h.description == description) {
            // Strengthen existing habit
            if let Some(habit) = self.habits.iter_mut().find(|h| h.description == description) {
                habit.strength = (habit.strength + Fixed::from_f64(0.05)).clamp_01();
                habit.repetition_count += 1;
                habit.last_performed = tick;
            }
            return;
        }
        self.habits.push(Habit {
            description,
            strength,
            repetition_count: 1,
            last_performed: tick,
            trigger,
        });
    }

    /// Execute a habit (returns true if a habit was performed).
    pub fn execute_habit(&mut self, trigger: &str, tick: u64) -> Option<String> {
        // Find strongest habit matching trigger
        let best = self.habits
            .iter()
            .enumerate()
            .filter(|(_, h)| h.trigger == trigger)
            .max_by(|(_, a), (_, b)| a.strength.to_raw().cmp(&b.strength.to_raw()))
            .map(|(i, _)| i);

        if let Some(idx) = best {
            self.habits[idx].strength = (self.habits[idx].strength + Fixed::from_f64(0.01)).clamp_01();
            self.habits[idx].repetition_count += 1;
            self.habits[idx].last_performed = tick;
            Some(self.habits[idx].description.clone())
        } else {
            None
        }
    }

    /// Update automaticity based on habit strength and stress.
    pub fn update_automaticity(&mut self, stress: Fixed, fatigue: Fixed) {
        // Under stress, agents fall back on habits more
        let stress_boost = stress * Fixed::from_f64(0.2) + fatigue * Fixed::from_f64(0.1);
        let avg_habit_strength = if self.habits.is_empty() {
            Fixed::ZERO
        } else {
            let total: Fixed = self.habits.iter().map(|h| h.strength).fold(Fixed::ZERO, |a, b| a + b);
            total / Fixed::from_f64(self.habits.len() as f64)
        };
        self.automaticity = (avg_habit_strength * Fixed::from_f64(0.5) + stress_boost).clamp_01();
    }

    /// Decay habit strength over time (habits weaken without practice).
    pub fn decay_habits(&mut self, current_tick: u64) {
        for habit in &mut self.habits {
            let ticks_since = current_tick.saturating_sub(habit.last_performed);
            if ticks_since > 100 {
                // Use Fixed arithmetic for determinism: decay = 0.001 * (ticks_since / 100)
                let ticks_fixed = Fixed::from_int(ticks_since as i64);
                let decay = Fixed::from_f64(0.001) * ticks_fixed / Fixed::from_f64(100.0);
                habit.strength = (habit.strength - decay).max(Fixed::ZERO);
            }
        }
        // Remove habits that have decayed to zero
        self.habits.retain(|h| h.strength > Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn skill_improves_with_practice() {
        let mut s = SkillState::default();
        s.practice(0, 0, Fixed::from_f64(0.8));
        assert!(s.proficiency(0) > Fixed::ZERO);
    }

    #[test]
    fn habit_forms_through_repetition() {
        let mut s = SkillState::default();
        s.form_habit("morning_walk".into(), "morning".into(), Fixed::from_f64(0.3), 0);
        s.form_habit("morning_walk".into(), "morning".into(), Fixed::from_f64(0.3), 1);
        assert_eq!(s.habits.len(), 1);
        assert_eq!(s.habits[0].repetition_count, 2);
    }

    #[test]
    fn habit_execution_returns_description() {
        let mut s = SkillState::default();
        s.form_habit("eat_grain".into(), "hunger".into(), Fixed::from_f64(0.5), 0);
        let result = s.execute_habit("hunger", 1);
        assert!(result.is_some());
    }

    #[test]
    fn stress_increases_automaticity() {
        let mut s = SkillState::default();
        s.form_habit("test".into(), "trigger".into(), Fixed::from_f64(0.5), 0);
        s.update_automaticity(Fixed::from_f64(0.8), Fixed::ZERO);
        assert!(s.automaticity > Fixed::ZERO);
    }
}
