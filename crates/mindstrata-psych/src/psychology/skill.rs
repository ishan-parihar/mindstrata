//! Skill and habit system — agents learn skills through practice and form habits.
//!
//! Skills: farming, cooking, healing, trading, fighting, speaking, leadership,
//! ritual, crafting, parenting, deception, diplomacy.
//!
//! Habits form through repetition and stress. Under stress, agents fall back on habits.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};
// §8.1.19 (P3-1): a `BTreeMap` (not the plan's HashMap) keeps key-sorted
// serialization — the snapshot's `deterministic_hash` hashes postcard bytes,
// and a non-empty HashMap iterates in per-instance random order, breaking
// byte-determinism across a save/load round trip (probe-caught: the skills
// map sat EMPTY before P3-1 wired practice, hiding the issue). Same
// convention as the noosphere's narrative_dominance (echo_chamber.rs:150).
use std::collections::BTreeMap;

/// Skill identifier.
pub type SkillId = u32;

/// §8.1.19 (P3-1, August 14, 2026): canonical skill ids for the action
/// wiring. Previously the psychology SkillState had ZERO production call
/// sites for `practice`/`form_habit`/`execute_habit`, so the skills and
/// habits maps stayed permanently empty (probe: skill_count/habit_count
/// 0.000 for 12/12 agents in every window). These ids anchor the
/// ActionKind → skill mapping in the sim's practice pass.
pub const SKILL_FARMING: SkillId = 0;
/// Const. (doc added at S3 extraction)
pub const SKILL_COOKING: SkillId = 1;
/// Const. (doc added at S3 extraction)
pub const SKILL_HEALING: SkillId = 2;
/// Const. (doc added at S3 extraction)
pub const SKILL_TRADING: SkillId = 3;
/// Const. (doc added at S3 extraction)
pub const SKILL_FIGHTING: SkillId = 4;
/// Const. (doc added at S3 extraction)
pub const SKILL_SPEAKING: SkillId = 5;
/// Const. (doc added at S3 extraction)
pub const SKILL_LEADERSHIP: SkillId = 6;
/// Const. (doc added at S3 extraction)
pub const SKILL_RITUAL: SkillId = 7;
/// Const. (doc added at S3 extraction)
pub const SKILL_CRAFTING: SkillId = 8;
/// Const. (doc added at S3 extraction)
pub const SKILL_PARENTING: SkillId = 9;
/// Const. (doc added at S3 extraction)
pub const SKILL_DECEPTION: SkillId = 10;
/// Const. (doc added at S3 extraction)
pub const SKILL_DIPLOMACY: SkillId = 11;

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
    /// Fn. (doc added at S3 extraction)
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
    pub skills: BTreeMap<SkillId, SkillLevel>,
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
            skills: BTreeMap::new(),
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
    pub fn form_habit(&mut self, description: String, trigger: String, strength: Fixed, tick: u64) {
        // Check if habit already exists
        if self.habits.iter().any(|h| h.description == description) {
            // Strengthen existing habit
            if let Some(habit) = self
                .habits
                .iter_mut()
                .find(|h| h.description == description)
            {
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

    /// §8.1.19 (P3-1 completion, re-audit August 14, 2026): refresh a
    /// matching habit when the agent practices the same action. The
    /// original P3-1 wiring only refreshed `last_performed` via
    /// `execute_habit` — but that gate requires `automaticity > 0.5`,
    /// which the calibrated formula never reaches (probe: max 0.413), so
    /// a formed habit's `last_performed` stayed frozen and the centum
    /// `decay_habits` pass ground every habit to zero by ~10K ticks
    /// (probe: habit_count 1.17 @2K → 0.000 @10K in calm/famine).
    /// Practicing the action IS exercising the habit: each practice tick
    /// refreshes the habit's recency and adds a small reinforcement, so
    /// habits persist exactly as long as the agent keeps doing the
    /// action ("use it or lose it") and only decay when the action is
    /// abandoned. Deterministic, no RNG; habits feed no golden-hashed
    /// metric, so calibrated runs stay byte-identical.
    pub fn refresh_habit(&mut self, trigger: &str, tick: u64) {
        for habit in self.habits.iter_mut().filter(|h| h.trigger == trigger) {
            habit.last_performed = tick;
            habit.repetition_count += 1;
            habit.strength = (habit.strength + Fixed::from_f64(0.005)).clamp_01();
        }
    }

    /// Execute a habit (returns true if a habit was performed).
    pub fn execute_habit(&mut self, trigger: &str, tick: u64) -> Option<String> {
        // Find strongest habit matching trigger
        let best = self
            .habits
            .iter()
            .enumerate()
            .filter(|(_, h)| h.trigger == trigger)
            .max_by(|(_, a), (_, b)| a.strength.to_raw().cmp(&b.strength.to_raw()))
            .map(|(i, _)| i);

        if let Some(idx) = best {
            self.habits[idx].strength =
                (self.habits[idx].strength + Fixed::from_f64(0.01)).clamp_01();
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
            let total: Fixed = self
                .habits
                .iter()
                .map(|h| h.strength)
                .fold(Fixed::ZERO, |a, b| a + b);
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
        s.form_habit(
            "morning_walk".into(),
            "morning".into(),
            Fixed::from_f64(0.3),
            0,
        );
        s.form_habit(
            "morning_walk".into(),
            "morning".into(),
            Fixed::from_f64(0.3),
            1,
        );
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

    /// §8.1.19 (P3-1 completion): practice refreshes habit recency, so a
    /// habit the agent keeps performing survives the decay pass while an
    /// abandoned habit decays out. The pre-fix lifecycle (form → frozen
    /// last_performed → centum decay → death) is the re-audit residual.
    #[test]
    fn practice_refreshes_matching_habit_recency() {
        let mut s = SkillState::default();
        s.form_habit("Work".into(), "hunger".into(), Fixed::from_f64(0.3), 0);
        // Practice the matching action many ticks later.
        s.refresh_habit("hunger", 5000);
        assert_eq!(
            s.habits[0].last_performed, 5000,
            "practice must refresh the matching habit's recency"
        );
        // The decay pass then leaves it untouched (recency within range).
        s.decay_habits(5100);
        assert_eq!(
            s.habits.len(),
            1,
            "an actively-practiced habit must persist"
        );
        // An abandoned habit (no refresh) decays out.
        let mut s2 = SkillState::default();
        s2.form_habit("Work".into(), "hunger".into(), Fixed::from_f64(0.3), 0);
        s2.decay_habits(100_000);
        assert_eq!(s2.habits.len(), 0, "an abandoned habit must decay to zero");
    }

    #[test]
    fn stress_increases_automaticity() {
        let mut s = SkillState::default();
        s.form_habit("test".into(), "trigger".into(), Fixed::from_f64(0.5), 0);
        s.update_automaticity(Fixed::from_f64(0.8), Fixed::ZERO);
        assert!(s.automaticity > Fixed::ZERO);
    }

    /// §17 (Iteration 197): higher neuroplasticity (fed by the
    /// cognitive-development modulation at the practice call site) must
    /// grow proficiency faster — the life-stage learning channel.
    #[test]
    fn higher_neuroplasticity_learns_faster() {
        let mut low = SkillState::default();
        let mut high = SkillState::default();
        for _ in 0..100 {
            low.practice(0, 0, Fixed::from_f64(0.4));
            high.practice(0, 0, Fixed::from_f64(0.9));
        }
        assert!(
            high.proficiency(0) > low.proficiency(0),
            "higher neuroplasticity must learn faster ({} vs {})",
            high.proficiency(0),
            low.proficiency(0)
        );
    }
}
