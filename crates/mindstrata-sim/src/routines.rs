//! Daily routines and schedules — §10.3 of the architecture spec.
//!
//! Agents should have daily routines that create behavioral stability.
//! Emergence happens when routines are disrupted by hunger, fear,
//! opportunity, conflict, institutional demands, or relationship events.
//!
//! The clock provides `intra_day()` (0..95 ticks per day) and `hour()` (0..23).
//! Routines are defined as time windows mapping to preferred actions.

use crate::actions::ActionKind;
use mindstrata_core::clock::Tick;
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A routine entry: a time window and the preferred action during that window.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutineEntry {
    /// Start hour (inclusive, 0..23).
    pub start_hour: u64,
    /// End hour (exclusive, 0..24).
    pub end_hour: u64,
    /// Preferred action during this window.
    pub preferred_action: ActionKind,
    /// How strongly this routine influences action selection (0..1).
    /// Higher = more likely to follow the routine.
    pub strength: Fixed,
}

/// A daily routine for an agent.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct DailyRoutine {
    /// The routine entries, ordered by start_hour.
    entries: Vec<RoutineEntry>,
    /// Whether the agent is currently following a routine.
    pub following_routine: bool,
    /// Tick when the agent last switched to a new routine entry.
    pub last_routine_switch_tick: u64,
}

impl DailyRoutine {
    /// Create the default village routine.
    ///
    /// §10.3: "06:00 wake, 07:00 eat, 08:00 work, 12:00 socialize/eat,
    /// 13:00 work, 18:00 return home, 20:00 socialize/worship/rest, 22:00 sleep"
    pub fn village_routine() -> Self {
        Self {
            entries: vec![
                // 06:00-07:00: Wake and eat
                RoutineEntry {
                    start_hour: 6,
                    end_hour: 7,
                    preferred_action: ActionKind::Eat,
                    strength: Fixed::from_f64(0.6),
                },
                // 07:00-08:00: Drink (Iteration 190 — the documented
                // "Eat/drink" slot was implemented as Eat, leaving the
                // village with ZERO hydration slots: probe-pinned thirst
                // ~0.55 in every scenario — villagers chronically ~55%
                // dehydrated even next to water, their hunger ~0.004
                // (routinized meals) while thirst drifts to the
                // utility-driven sawtooth. A daily water break is the
                // routine's own documented intent, now real.)
                RoutineEntry {
                    start_hour: 7,
                    end_hour: 8,
                    preferred_action: ActionKind::Drink,
                    strength: Fixed::from_f64(0.5),
                },
                // 08:00-12:00: Work
                RoutineEntry {
                    start_hour: 8,
                    end_hour: 12,
                    preferred_action: ActionKind::Work,
                    strength: Fixed::from_f64(0.7),
                },
                // 12:00-13:00: Socialize/lunch
                RoutineEntry {
                    start_hour: 12,
                    end_hour: 13,
                    preferred_action: ActionKind::Socialize,
                    strength: Fixed::from_f64(0.5),
                },
                // 13:00-18:00: Work
                RoutineEntry {
                    start_hour: 13,
                    end_hour: 18,
                    preferred_action: ActionKind::Work,
                    strength: Fixed::from_f64(0.7),
                },
                // 18:00-20:00: Eat/rest
                RoutineEntry {
                    start_hour: 18,
                    end_hour: 20,
                    preferred_action: ActionKind::Eat,
                    strength: Fixed::from_f64(0.5),
                },
                // 20:00-22:00: Socialize/worship
                RoutineEntry {
                    start_hour: 20,
                    end_hour: 22,
                    preferred_action: ActionKind::Socialize,
                    strength: Fixed::from_f64(0.4),
                },
                // 22:00-06:00: Rest (sleep)
                RoutineEntry {
                    start_hour: 22,
                    end_hour: 6,
                    preferred_action: ActionKind::Rest,
                    strength: Fixed::from_f64(0.8),
                },
            ],
            following_routine: false,
            last_routine_switch_tick: 0,
        }
    }

    /// Get the preferred action for the current time of day.
    /// Returns (preferred_action, routine_strength).
    pub fn preferred_action(&self, tick: Tick) -> (ActionKind, Fixed) {
        let hour = tick.hour();

        for entry in &self.entries {
            if entry.start_hour < entry.end_hour {
                // Normal window (e.g., 8..12)
                if hour >= entry.start_hour && hour < entry.end_hour {
                    return (entry.preferred_action, entry.strength);
                }
            } else {
                // Wrapping window (e.g., 22..6 = 22..24 + 0..6)
                if hour >= entry.start_hour || hour < entry.end_hour {
                    return (entry.preferred_action, entry.strength);
                }
            }
        }

        // Default: idle
        (ActionKind::Idle, Fixed::ZERO)
    }

    /// Should the agent follow the routine given their current state?
    /// Routines are overridden by critical needs, emotional shocks, etc.
    pub fn should_follow(
        &self,
        hunger: Fixed,
        thirst: Fixed,
        fatigue: Fixed,
        stress: Fixed,
    ) -> bool {
        // Critical needs override routines
        if hunger > Fixed::from_f64(0.85) || thirst > Fixed::from_f64(0.85) {
            return false;
        }
        // High stress overrides routines (agent acts on emotion)
        if stress > Fixed::from_f64(0.7) {
            return false;
        }
        // Extreme fatigue overrides routines
        if fatigue > Fixed::from_f64(0.9) {
            return false;
        }
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn village_routine_has_entries() {
        let routine = DailyRoutine::village_routine();
        assert!(!routine.entries.is_empty());
        assert_eq!(routine.entries.len(), 8);
    }

    #[test]
    fn morning_prefers_eating() {
        let routine = DailyRoutine::village_routine();
        // Tick 24 = hour 6 (24/4 = 6) — the 06:00-07:00 breakfast slot.
        let tick = Tick::new(24);
        let (action, strength) = routine.preferred_action(tick);
        assert_eq!(action, ActionKind::Eat);
        assert!(strength > Fixed::ZERO);
    }

    #[test]
    fn mid_morning_prefers_drinking() {
        // Iteration 190: the 07:00-08:00 slot is the routine's hydration
        // break (the documented "Eat/drink" intent) — a village that eats
        // three times a day must also drink.
        let routine = DailyRoutine::village_routine();
        // Tick 28 = hour 7 (28/4 = 7) — the 07:00-08:00 drink slot.
        let tick = Tick::new(28);
        let (action, strength) = routine.preferred_action(tick);
        assert_eq!(action, ActionKind::Drink);
        assert!(strength > Fixed::ZERO);
    }

    #[test]
    fn work_hours_prefers_work() {
        let routine = DailyRoutine::village_routine();
        // Tick 36 = hour 9 (36/4 = 9)
        let tick = Tick::new(36);
        let (action, _strength) = routine.preferred_action(tick);
        assert_eq!(action, ActionKind::Work);
    }

    #[test]
    fn night_prefers_rest() {
        let routine = DailyRoutine::village_routine();
        // Tick 0 = hour 0
        let tick = Tick::new(0);
        let (action, _strength) = routine.preferred_action(tick);
        assert_eq!(action, ActionKind::Rest);
    }

    #[test]
    fn critical_needs_override_routine() {
        let routine = DailyRoutine::village_routine();
        assert!(!routine.should_follow(
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.3),
        ));
    }

    #[test]
    fn high_stress_overrides_routine() {
        let routine = DailyRoutine::village_routine();
        assert!(!routine.should_follow(
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.8),
        ));
    }

    #[test]
    fn normal_state_follows_routine() {
        let routine = DailyRoutine::village_routine();
        assert!(routine.should_follow(
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.3),
        ));
    }
}
