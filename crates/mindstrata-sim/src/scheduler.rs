//! §6 Multi-Timescale Scheduler — formalized phase system.
//!
//! Replaces ad-hoc `is_multiple_of` checks throughout the tick loop with
//! a clean, documented phase system. Each tick is classified into one or
//! more phases based on the architecture plan §6.2 System Frequencies.
//!
//! ```text
//! Tick Granularity:
//!   1 tick = 10 simulated minutes
//!   6 ticks = 1 hour
//! 144 ticks = 1 day
//! 1008 ticks = 1 week
//! 4320 ticks = 30-day month
//! 51840 ticks = 360-day year
//!
//! Phase Hierarchy (each phase includes all smaller phases):
//!   FastPhase      → every tick
//!   HourlyPhase    → every 6 ticks
//!   DecaPhase      → every 10 ticks
//!   DuodecaPhase   → every 12 ticks
//!   DailyPhase     → every 144 ticks
//!   CentumPhase    → every 100 ticks (approx. 100 min)
//!   WeeklyPhase    → every 1008 ticks
//!   QuincentPhase  → every 500 ticks
//!   SeasonalPhase  → every 4320 ticks
//!   YearlyPhase    → every 51840 ticks
//! ```

use serde::{Deserialize, Serialize};

/// §6: Multi-timescale scheduler phases.
///
/// Each phase has a tick interval. A tick satisfies a phase if it is a
/// multiple of that phase's interval.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TickPhase {
    /// Every tick (fast systems: perception, action, autonomic, emotion).
    Fast,
    /// Every 6 ticks (~1 hour: hormonal fast modulation).
    Hourly,
    /// Every 10 ticks (body state, demography, conflict update).
    Deca,
    /// Every 12 ticks (~2 hours: ritual execution).
    Duodeca,
    /// Every 100 ticks (~100 min: institutional decisions, policy).
    Centum,
    /// Every 144 ticks (1 day: memory consolidation, decay, culture).
    Daily,
    /// Every 500 ticks (feud decay, hierarchy checks).
    Quincent,
    /// Every 1008 ticks (~1 week: status recalibration, faction dynamics).
    Weekly,
    /// Every 4320 ticks (1 month: developmental, demographic shifts).
    Seasonal,
    /// Every 51840 ticks (1 year: climate, culture drift, technology).
    Yearly,
}

impl TickPhase {
    /// Tick interval for this phase.
    pub const fn interval(&self) -> u64 {
        match self {
            Self::Fast => 1,
            Self::Hourly => 6,
            Self::Deca => 10,
            Self::Duodeca => 12,
            Self::Centum => 100,
            Self::Daily => 144,
            Self::Quincent => 500,
            Self::Weekly => 1008,
            Self::Seasonal => 4320,
            Self::Yearly => 51840,
        }
    }

    /// Human-readable name for debugging/logging.
    pub const fn name(&self) -> &'static str {
        match self {
            Self::Fast => "fast",
            Self::Hourly => "hourly",
            Self::Deca => "deca",
            Self::Duodeca => "duodeca",
            Self::Centum => "centum",
            Self::Daily => "daily",
            Self::Quincent => "quincent",
            Self::Weekly => "weekly",
            Self::Seasonal => "seasonal",
            Self::Yearly => "yearly",
        }
    }
}

/// §6: Computed phase state for a given tick.
///
/// Pre-computed once per tick and passed to subsystem methods, replacing
/// scattered `is_multiple_of` calls with a single lookup.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
#[must_use]
pub struct TickPhases {
    pub tick: u64,
    pub is_hourly: bool,
    pub is_deca: bool,
    pub is_duodeca: bool,
    pub is_centum: bool,
    pub is_daily: bool,
    pub is_quincent: bool,
    pub is_weekly: bool,
    pub is_seasonal: bool,
    pub is_yearly: bool,
}

impl TickPhases {
    /// Compute phase flags for a given tick.
    pub fn compute(tick: u64) -> Self {
        Self {
            tick,
            is_hourly: tick > 0 && tick.is_multiple_of(6),
            is_deca: tick > 0 && tick.is_multiple_of(10),
            is_duodeca: tick > 0 && tick.is_multiple_of(12),
            is_centum: tick > 0 && tick.is_multiple_of(100),
            is_daily: tick > 0 && tick.is_multiple_of(144),
            is_quincent: tick > 0 && tick.is_multiple_of(500),
            is_weekly: tick > 0 && tick.is_multiple_of(1008),
            is_seasonal: tick > 0 && tick.is_multiple_of(4320),
            is_yearly: tick > 0 && tick.is_multiple_of(51840),
        }
    }

    /// Check if a given phase is active this tick.
    pub fn is_active(&self, phase: TickPhase) -> bool {
        match phase {
            TickPhase::Fast => true,
            TickPhase::Hourly => self.is_hourly,
            TickPhase::Deca => self.is_deca,
            TickPhase::Duodeca => self.is_duodeca,
            TickPhase::Centum => self.is_centum,
            TickPhase::Daily => self.is_daily,
            TickPhase::Quincent => self.is_quincent,
            TickPhase::Weekly => self.is_weekly,
            TickPhase::Seasonal => self.is_seasonal,
            TickPhase::Yearly => self.is_yearly,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hourly_every_6_ticks() {
        assert!(TickPhases::compute(6).is_hourly);
        assert!(TickPhases::compute(12).is_hourly);
        assert!(!TickPhases::compute(5).is_hourly);
        assert!(!TickPhases::compute(7).is_hourly);
    }

    #[test]
    fn daily_every_144_ticks() {
        assert!(TickPhases::compute(144).is_daily);
        assert!(TickPhases::compute(288).is_daily);
        assert!(!TickPhases::compute(143).is_daily);
        assert!(!TickPhases::compute(145).is_daily);
    }

    #[test]
    fn tick_zero_has_no_non_fast_phases() {
        let p = TickPhases::compute(0);
        assert!(!p.is_hourly);
        assert!(!p.is_deca);
        assert!(!p.is_daily);
        assert!(!p.is_weekly);
    }

    #[test]
    fn weekly_every_1008_ticks() {
        assert!(TickPhases::compute(1008).is_weekly);
        assert!(TickPhases::compute(2016).is_weekly);
        assert!(!TickPhases::compute(1007).is_weekly);
    }

    #[test]
    fn phase_specific_ticks() {
        // Daily (144) implies hourly (144%6==0) but NOT deca (144%10==4)
        let p = TickPhases::compute(144);
        assert!(p.is_daily);
        assert!(p.is_hourly);
        assert!(!p.is_deca);
        // Weekly (1008) implies hourly (1008%6==0) but NOT centum (1008%100==8)
        let w = TickPhases::compute(1008);
        assert!(w.is_weekly);
        assert!(w.is_hourly);
        assert!(!w.is_centum);
    }

    #[test]
    fn is_active_lookup() {
        let p = TickPhases::compute(144);
        assert!(p.is_active(TickPhase::Daily));
        assert!(!p.is_active(TickPhase::Weekly));
        assert!(p.is_active(TickPhase::Fast));
    }

    #[test]
    fn phase_intervals_match_architecture() {
        assert_eq!(TickPhase::Fast.interval(), 1);
        assert_eq!(TickPhase::Hourly.interval(), 6);
        assert_eq!(TickPhase::Deca.interval(), 10);
        assert_eq!(TickPhase::Duodeca.interval(), 12);
        assert_eq!(TickPhase::Daily.interval(), 144);
        assert_eq!(TickPhase::Weekly.interval(), 1008);
        assert_eq!(TickPhase::Seasonal.interval(), 4320);
        assert_eq!(TickPhase::Yearly.interval(), 51840);
    }
}
