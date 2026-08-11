//! Simulation clock and tick counting.

use serde::{Deserialize, Serialize};
use std::fmt;

/// A discrete simulation tick.
///
/// 1 tick = 15 minutes of in-game time.
/// 1 day  = 96 ticks.
/// 1 year = 35 040 ticks.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize)]
pub struct Tick(u64);

impl Tick {
    pub const ZERO: Self = Self(0);

    /// Ticks per in-game day (96 × 15 min = 24 h).
    pub const TICKS_PER_DAY: u64 = 96;
    /// Ticks per in-game year.
    pub const TICKS_PER_YEAR: u64 = 35_040;

    pub const fn new(v: u64) -> Self {
        Self(v)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }

    /// Advance by one tick.
    pub fn advance(&mut self) {
        self.0 += 1;
    }

    /// Day number (0-based) within the simulation.
    pub const fn day(self) -> u64 {
        self.0 / Self::TICKS_PER_DAY
    }

    /// Year number (0-based).
    pub const fn year(self) -> u64 {
        self.0 / Self::TICKS_PER_YEAR
    }

    /// Tick within the current day (0 .. 95).
    pub const fn intra_day(self) -> u64 {
        self.0 % Self::TICKS_PER_DAY
    }

    /// Rough in-game hour (0 .. 23), assuming 1 tick = 15 min.
    pub const fn hour(self) -> u64 {
        self.intra_day() / 4
    }
}

impl fmt::Debug for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Tick({}, day={}, h={})", self.0, self.day(), self.hour())
    }
}

impl fmt::Display for Tick {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "t{}", self.0)
    }
}

/// Simulation clock holding the current tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Clock {
    current: Tick,
}

impl Default for Clock {
    fn default() -> Self {
        Self {
            current: Tick::ZERO,
        }
    }
}

impl Clock {
    pub fn new() -> Self {
        Self::default()
    }

    /// Current tick.
    pub fn tick(&self) -> Tick {
        self.current
    }

    /// Advance the clock by one tick and return the new tick.
    pub fn advance(&mut self) -> Tick {
        self.current.advance();
        self.current
    }

    /// Set the clock to a specific tick.
    pub fn set(&mut self, tick: Tick) {
        self.current = tick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tick_day_and_hour() {
        let t = Tick::new(100); // 100 / 96 = day 1, intra 4 → hour 1
        assert_eq!(t.day(), 1);
        assert_eq!(t.intra_day(), 4);
        assert_eq!(t.hour(), 1);
    }

    #[test]
    fn clock_advance() {
        let mut clock = Clock::new();
        assert_eq!(clock.tick(), Tick::ZERO);
        clock.advance();
        assert_eq!(clock.tick(), Tick::new(1));
    }
}
