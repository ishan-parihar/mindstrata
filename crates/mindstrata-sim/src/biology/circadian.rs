//! Circadian system — sleep/wake cycle, sleep pressure, rhythm stability.
//!
//! Effects: morning/evening preferences, sleep deprivation, irritability,
//! ritual timing, work schedules.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Chronotype preference.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Chronotype {
    /// Peaks in morning, declines in evening.
    Morning,
    /// Neutral.
    Neutral,
    /// Peaks in evening, sluggish in morning.
    Evening,
}

/// Circadian state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CircadianState {
    /// Current phase in the day cycle (0.0 = midnight, 0.5 = noon, 1.0 = next midnight).
    pub phase: Fixed,
    /// Sleep pressure (0 = fully rested, 1 = desperately sleepy).
    pub sleep_pressure: Fixed,
    /// Rhythm stability — how consistent the sleep/wake cycle is.
    pub rhythm_stability: Fixed,
    /// Chronotype.
    pub chronotype: Chronotype,
    /// Sleep quality last night (0 = terrible, 1 = perfect).
    pub sleep_quality: Fixed,
    /// Hours of sleep debt accumulated.
    pub sleep_debt: Fixed,
    /// Alertness level (0 = half-asleep, 1 = fully alert).
    pub alertness: Fixed,
}

impl Default for CircadianState {
    fn default() -> Self {
        Self {
            phase: Fixed::from_f64(0.25), // start at 6am
            sleep_pressure: Fixed::ZERO,
            rhythm_stability: Fixed::from_f64(0.5),
            chronotype: Chronotype::Neutral,
            sleep_quality: Fixed::from_f64(0.7),
            sleep_debt: Fixed::ZERO,
            alertness: Fixed::from_f64(0.7),
        }
    }
}

impl CircadianState {
    /// Advance the circadian clock by one tick.
    /// `ticks_per_day` defines how many ticks constitute a full day cycle.
    pub fn tick_update(&mut self, ticks_per_day: u64, is_sleeping: bool) {
        // Advance phase (one tick = 1/ticks_per_day of a full cycle)
        let phase_increment = Fixed::from_f64(1.0) / Fixed::from_f64(ticks_per_day as f64);
        self.phase += phase_increment;
        // Wrap at 1.0 (modular arithmetic)
        if self.phase >= Fixed::ONE {
            self.phase -= Fixed::ONE;
        }

        // Sleep pressure builds during waking hours
        if is_sleeping {
            // Sleeping reduces sleep pressure
            let recovery = self.sleep_quality * Fixed::from_f64(0.008);
            self.sleep_pressure = (self.sleep_pressure - recovery).max(Fixed::ZERO);
            // Sleep debt is repaid
            self.sleep_debt = (self.sleep_debt - Fixed::from_f64(0.001)).max(Fixed::ZERO);
        } else {
            // Waking builds sleep pressure
            let build_rate = Fixed::from_f64(0.001);
            self.sleep_pressure = (self.sleep_pressure + build_rate).clamp_01();
            // Sleep debt accumulates if pressure is high
            if self.sleep_pressure > Fixed::from_f64(0.7) {
                self.sleep_debt = (self.sleep_debt + Fixed::from_f64(0.0005)).clamp_01();
            }
        }

        // Alertness follows circadian rhythm + sleep pressure
        let circadian_alertness = self.circadian_alertness();
        self.alertness = (circadian_alertness - self.sleep_pressure * Fixed::from_f64(0.4)
            - self.sleep_debt * Fixed::from_f64(0.3))
            .clamp_01();

        // Rhythm stability improves with consistent sleep timing
        if is_sleeping && self.sleep_pressure < Fixed::from_f64(0.3) {
            self.rhythm_stability = (self.rhythm_stability + Fixed::from_f64(0.0002)).clamp_01();
        }
        // Sleep deprivation degrades rhythm
        if self.sleep_debt > Fixed::from_f64(0.5) {
            self.rhythm_stability = (self.rhythm_stability - Fixed::from_f64(0.0001)).max(Fixed::ZERO);
        }
    }

    /// Circadian alertness based on time of day.
    /// Peaks around phase 0.3–0.4 (roughly 7–10am) for morning types,
    /// 0.5–0.6 (noon–2pm) for neutral, 0.65–0.75 (4–6pm) for evening types.
    fn circadian_alertness(&self) -> Fixed {
        let peak_phase = match self.chronotype {
            Chronotype::Morning => Fixed::from_f64(0.33),
            Chronotype::Neutral => Fixed::from_f64(0.5),
            Chronotype::Evening => Fixed::from_f64(0.67),
        };
        // Triangular approximation of Gaussian curve around peak.
        let diff = (self.phase - peak_phase).abs();
        let half_width = Fixed::from_f64(0.3);
        // Linear falloff: 1.0 at peak, 0.2 at half_width distance
        if diff < half_width {
            // Linear falloff using Fixed arithmetic (deterministic)
            let ratio = diff / half_width;
            (Fixed::ONE - ratio * Fixed::from_f64(0.8)).clamp_01()
        } else {
            Fixed::from_f64(0.2)
        }
    }

    /// Whether it's currently "night" (low circadian alertness).
    pub fn is_night(&self) -> bool {
        self.phase < Fixed::from_f64(0.2) || self.phase > Fixed::from_f64(0.8)
    }

    /// Whether the agent should be sleeping based on circadian phase and sleep pressure.
    pub fn should_sleep(&self) -> bool {
        self.is_night() && self.sleep_pressure > Fixed::from_f64(0.4)
    }

    /// Whether the agent is sleep-deprived.
    pub fn sleep_deprived(&self) -> bool {
        self.sleep_debt > Fixed::from_f64(0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sleep_pressure_builds_while_awake() {
        let mut c = CircadianState::default();
        let initial = c.sleep_pressure;
        c.tick_update(144, false); // one tick awake
        assert!(c.sleep_pressure > initial);
    }

    #[test]
    fn sleep_reduces_pressure() {
        let mut c = CircadianState::default();
        c.sleep_pressure = Fixed::from_f64(0.8);
        c.tick_update(144, true);
        assert!(c.sleep_pressure < Fixed::from_f64(0.8));
    }

    #[test]
    fn phase_advances() {
        let mut c = CircadianState::default();
        let initial_phase = c.phase;
        c.tick_update(144, false);
        assert!(c.phase > initial_phase);
    }
}
