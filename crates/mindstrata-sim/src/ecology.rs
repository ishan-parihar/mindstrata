//! Ecology — §28 of the architecture spec.
//!
//! Implements seasons, resource depletion, and fertility dynamics.
//! §28: "Start with simple fertility, resources, and seasons."
//! §Phase 9: "Harvest varies by season. Overfarming reduces fertility.
//! Ecological decline produces famine and conflict."

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Current season of the year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Season {
    Spring,
    Summer,
    Autumn,
    Winter,
}

impl Season {
    /// Get the next season.
    #[must_use]
    pub fn next(self) -> Self {
        match self {
            Season::Spring => Season::Summer,
            Season::Summer => Season::Autumn,
            Season::Autumn => Season::Winter,
            Season::Winter => Season::Spring,
        }
    }

    /// Human-readable name.
    pub fn name(self) -> &'static str {
        match self {
            Season::Spring => "Spring",
            Season::Summer => "Summer",
            Season::Autumn => "Autumn",
            Season::Winter => "Winter",
        }
    }

    /// Temperature modifier for this season (0.0 = freezing, 1.0 = hot).
    pub fn temperature_modifier(self) -> Fixed {
        match self {
            Season::Spring => Fixed::from_f64(0.6),
            Season::Summer => Fixed::from_f64(0.9),
            Season::Autumn => Fixed::from_f64(0.5),
            Season::Winter => Fixed::from_f64(0.2),
        }
    }

    /// Growth modifier for crops (affects fertility and production).
    /// Winter: no growth. Spring: moderate. Summer: peak. Autumn: declining.
    pub fn growth_modifier(self) -> Fixed {
        match self {
            Season::Spring => Fixed::from_f64(0.6),
            Season::Summer => Fixed::from_f64(1.0),
            Season::Autumn => Fixed::from_f64(0.4),
            Season::Winter => Fixed::from_f64(0.0),
        }
    }

    /// Food spoilage modifier (hot = more spoilage).
    pub fn spoilage_modifier(self) -> Fixed {
        match self {
            Season::Spring => Fixed::from_f64(0.8),
            Season::Summer => Fixed::from_f64(1.2),
            Season::Autumn => Fixed::from_f64(0.6),
            Season::Winter => Fixed::from_f64(0.3),
        }
    }
}

/// Ecology configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EcologyConfig {
    /// Ticks per season (default: 8760 = ~1 year if tick = 15min).
    pub ticks_per_season: u64,
    /// Base fertility decay rate per tick from overfarming.
    pub overfarming_decay_rate: Fixed,
    /// Natural fertility recovery rate per tick.
    pub fertility_recovery_rate: Fixed,
    /// Minimum fertility before resource is considered depleted.
    pub min_fertility: Fixed,
}

impl Default for EcologyConfig {
    fn default() -> Self {
        Self {
            ticks_per_season: 8760,
            overfarming_decay_rate: Fixed::from_f64(0.001),
            fertility_recovery_rate: Fixed::from_f64(0.0002),
            min_fertility: Fixed::from_f64(0.1),
        }
    }
}

/// Season tracker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeasonTracker {
    pub current: Season,
    pub tick_in_season: u64,
    pub ticks_per_season: u64,
    pub year: u64,
}

impl SeasonTracker {
    pub fn new(ticks_per_season: u64) -> Self {
        Self {
            current: Season::Spring,
            tick_in_season: 0,
            ticks_per_season,
            year: 0,
        }
    }

    /// Advance the season tracker by one tick. Returns true if a new season started.
    pub fn advance(&mut self) -> bool {
        self.tick_in_season += 1;
        if self.tick_in_season >= self.ticks_per_season {
            self.tick_in_season = 0;
            let old = self.current;
            self.current = self.current.next();
            if self.current == Season::Spring {
                self.year += 1;
            }
            old != self.current
        } else {
            false
        }
    }

    /// Get current day within the season (0-based).
    pub fn day_of_season(&self) -> u64 {
        self.tick_in_season / 96 // 96 ticks = 1 day
    }
}

/// Apply ecology systems for one tick.
pub fn system_ecology(
    season: &SeasonTracker,
    site_fertilities: &mut [Fixed],
    site_work_ticks: &[u32], // how many work ticks each site received this cycle
    config: &EcologyConfig,
) {
    let growth = season.current.growth_modifier();
    let n = site_fertilities.len();

    for i in 0..n {
        // Overfarming decay: more work = more decay
        let work_pressure = if site_work_ticks[i] > 0 {
            Fixed::from_int(site_work_ticks[i] as i64) * config.overfarming_decay_rate
        } else {
            Fixed::ZERO
        };

        // Natural recovery (slower in winter)
        let recovery = config.fertility_recovery_rate * growth;

        site_fertilities[i] = (site_fertilities[i] - work_pressure + recovery)
            .max(config.min_fertility)
            .min(Fixed::ONE);
    }
}

/// Check if a site is depleted (fertility below threshold).
pub fn is_depleted(fertility: Fixed, config: &EcologyConfig) -> bool {
    fertility <= config.min_fertility + Fixed::from_f64(0.05)
}

/// Compute migration pressure for an agent based on resource scarcity and season.
pub fn migration_pressure(
    local_food_scarcity: Fixed, // 0 = abundant, 1 = no food
    season: Season,
    has_shelter: bool,
) -> Fixed {
    let base = local_food_scarcity * Fixed::from_f64(0.6);

    // Winter without shelter = much higher pressure
    let winter_factor = if season == Season::Winter && !has_shelter {
        Fixed::from_f64(0.3)
    } else {
        Fixed::ZERO
    };

    // Spring = lowest pressure (new growth)
    let season_factor = match season {
        Season::Spring => Fixed::from_f64(-0.1),
        Season::Summer => Fixed::ZERO,
        Season::Autumn => Fixed::from_f64(0.1),
        Season::Winter => Fixed::from_f64(0.2),
    };

    (base + winter_factor + season_factor).clamp_01()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn season_cycle_completes() {
        let mut tracker = SeasonTracker::new(4); // 4 ticks per season
        assert_eq!(tracker.current, Season::Spring);
        assert!(!tracker.advance()); // still spring
        assert!(!tracker.advance());
        assert!(!tracker.advance());
        assert!(tracker.advance()); // now summer
        assert_eq!(tracker.current, Season::Summer);
    }

    #[test]
    fn year_increments_after_winter() {
        let mut tracker = SeasonTracker::new(1);
        assert_eq!(tracker.year, 0);
        tracker.advance(); // summer
        tracker.advance(); // autumn
        tracker.advance(); // winter
        assert_eq!(tracker.year, 0);
        tracker.advance(); // spring again
        assert_eq!(tracker.year, 1);
    }

    #[test]
    fn growth_modifier_winter_is_zero() {
        assert!(Season::Winter.growth_modifier() == Fixed::ZERO);
    }

    #[test]
    fn growth_modifier_summer_is_max() {
        assert!(Season::Summer.growth_modifier() == Fixed::from_f64(1.0));
    }

    #[test]
    fn ecology_system_applies_fertility_decay() {
        let config = EcologyConfig::default();
        let mut fertilities = vec![Fixed::from_f64(0.8); 3];
        let work_ticks = vec![10, 0, 5];

        system_ecology(&SeasonTracker::new(100), &mut fertilities, &work_ticks, &config);

        // Site 0 had work, should have lower fertility
        assert!(fertilities[0] < Fixed::from_f64(0.8));
        // Site 1 had no work, should have recovered slightly
        assert!(fertilities[1] >= Fixed::from_f64(0.8));
        // Site 2 had some work
        assert!(fertilities[2] < Fixed::from_f64(0.8));
    }

    #[test]
    fn depleted_site_detected() {
        let config = EcologyConfig::default();
        assert!(is_depleted(Fixed::from_f64(0.1), &config));
        assert!(!is_depleted(Fixed::from_f64(0.5), &config));
    }

    #[test]
    fn migration_pressure_increases_with_scarcity() {
        let low = migration_pressure(Fixed::from_f64(0.2), Season::Summer, true);
        let high = migration_pressure(Fixed::from_f64(0.9), Season::Summer, true);
        assert!(high > low);
    }

    #[test]
    fn migration_pressure_winter_no_shelter() {
        let with = migration_pressure(Fixed::from_f64(0.5), Season::Winter, true);
        let without = migration_pressure(Fixed::from_f64(0.5), Season::Winter, false);
        assert!(without > with);
    }
}
