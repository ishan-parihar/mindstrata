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

    /// Rainfall baseline (0..1) for the season — the mean the weather layer
    /// reverts toward. Autumn is wettest, Winter driest.
    pub fn rainfall_baseline(self) -> Fixed {
        match self {
            Season::Spring => Fixed::from_f64(0.55),
            Season::Summer => Fixed::from_f64(0.45),
            Season::Autumn => Fixed::from_f64(0.70),
            Season::Winter => Fixed::from_f64(0.35),
        }
    }
}

// ── Weather (§5 AP2, Iteration 147) ───────────────────────────────
// A continuous temperature/rainfall layer with EMERGENT drought/flood
// regimes, replacing the pre-existing state where the only droughts and
// floods were scripted scenario shocks. The layer is deterministic (one
// seeded RNG draw per field per tick from the Ecology stream) and wired to
// three production consumers: farm growth, food spoilage, and well water.
//
// REGIME REACHABILITY (honest framing): the default thresholds — 240 dry
// ticks below rainfall 0.25 / 120 wet ticks above 0.85 — sit far outside
// the rainfall walk's reach in bounded horizons (stationary σ ≈ 0.25
// around the 0.55 seasonal baseline, so sustained extreme spells are
// effectively never). Emergent droughts/floods are therefore RARE-TO-NEVER
// events in normal runs BY DESIGN; the machinery is proven via re-tuned
// deterministic configs (the tests force reversion 1.0 + zero noise) and
// will genuinely fire over very long horizons when the walk wanders.
//
// SEASON-GROWTH DECISION (deliberate, not a missed consumer): the season
// growth modifier (`Season::growth_modifier`, Winter = 0.0) still has zero
// production consumers — the survey proved farm output is season-
// independent — and `growth_factor` deliberately does NOT fold it in.
// Wiring season growth into production would cut Winter farm output to
// zero and cascade into a Winter-starvation cliff; the weather layer
// modulates production mildly (≈1.015 normal) instead, and season growth
// gating stays a queued calibration item.

/// Weather configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherConfig {
    /// Per-tick temperature noise amplitude around the seasonal baseline.
    pub temperature_noise: Fixed,
    /// Per-tick rainfall noise amplitude around the seasonal baseline.
    pub rainfall_noise: Fixed,
    /// Mean-reversion strength of temperature/rainfall toward their seasonal
    /// baselines (0..1; 1.0 snaps to baseline each tick).
    pub mean_reversion: Fixed,
    /// Rainfall below this (with reversion-driven persistence) can declare a
    /// drought once the dry spell outlasts `drought_ticks`.
    pub drought_threshold: Fixed,
    /// Consecutive dry ticks required to declare an emergent drought.
    pub drought_ticks: u64,
    /// Rainfall above this can declare a flood once the wet spell outlasts
    /// `flood_ticks`.
    pub flood_threshold: Fixed,
    /// Consecutive wet ticks required to declare an emergent flood.
    pub flood_ticks: u64,
    /// Fraction of each well's water drained per tick during a drought.
    pub drought_water_drain: Fixed,
    /// Fraction of each well's water recharged per tick during a flood.
    pub flood_water_recharge: Fixed,
}

impl Default for WeatherConfig {
    fn default() -> Self {
        Self {
            temperature_noise: Fixed::from_f64(0.05),
            rainfall_noise: Fixed::from_f64(0.08),
            mean_reversion: Fixed::from_f64(0.05),
            drought_threshold: Fixed::from_f64(0.25),
            drought_ticks: 240,
            flood_threshold: Fixed::from_f64(0.85),
            flood_ticks: 120,
            drought_water_drain: Fixed::from_f64(0.002),
            flood_water_recharge: Fixed::from_f64(0.0015),
        }
    }
}

/// Weather regime. Normal weather is noise around the seasonal baseline;
/// Drought and Flood are emergent regimes declared from sustained dry/wet
/// spells and carrying real economic teeth (see the sim wiring).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatherRegime {
    Normal,
    Drought,
    Flood,
}

/// A regime transition, exposed so the sim can log (and tests can observe)
/// emergent weather events.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum WeatherEvent {
    DroughtStarted,
    FloodStarted,
    DroughtEnded,
    FloodEnded,
}

/// Per-tick weather tracker: continuous temperature + rainfall with
/// mean-reverting seasonal baselines and streak-based emergent regimes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeatherTracker {
    /// Current temperature, 0 (freezing) .. 1 (hot).
    pub temperature: Fixed,
    /// Current rainfall, 0 (bone dry) .. 1 (torrential).
    pub rainfall: Fixed,
    /// Current emergent regime.
    pub regime: WeatherRegime,
    /// Configuration (tests may re-tune to force deterministic regimes).
    pub config: WeatherConfig,
    /// Consecutive dry ticks (private — regime bookkeeping).
    dry_streak: u64,
    /// Consecutive wet ticks (private — regime bookkeeping).
    wet_streak: u64,
    /// Lifetime count of drought declarations.
    pub drought_events: u64,
    /// Lifetime count of flood declarations.
    pub flood_events: u64,
}

impl WeatherTracker {
    pub fn new(config: WeatherConfig) -> Self {
        Self {
            temperature: Fixed::from_f64(0.6),
            rainfall: Fixed::from_f64(0.55),
            regime: WeatherRegime::Normal,
            config,
            dry_streak: 0,
            wet_streak: 0,
            drought_events: 0,
            flood_events: 0,
        }
    }

    /// Advance weather by one tick: mean-revert temperature and rainfall
    /// toward their seasonal baselines, add seeded noise, then update the
    /// emergent regime from the sustained dry/wet spell bookkeeping. Returns
    /// the regime transition, if any. Consumes exactly two draws from `rng`
    /// (the Ecology stream in production) — deterministic for a seeded rng.
    pub fn advance(&mut self, season: Season, rng: &mut impl rand::Rng) -> Option<WeatherEvent> {
        // Temperature: revert toward the seasonal baseline, add noise.
        let temp_noise =
            Fixed::from_f64(rng.random_range(-1.0f64..1.0)) * self.config.temperature_noise;
        self.temperature = (self.temperature
            + (season.temperature_modifier() - self.temperature) * self.config.mean_reversion
            + temp_noise)
            .clamp_01();
        // Rainfall: same mean-reversion toward the seasonal baseline.
        let rain_noise =
            Fixed::from_f64(rng.random_range(-1.0f64..1.0)) * self.config.rainfall_noise;
        self.rainfall = (self.rainfall
            + (season.rainfall_baseline() - self.rainfall) * self.config.mean_reversion
            + rain_noise)
            .clamp_01();

        let mut event = None;
        match self.regime {
            WeatherRegime::Normal => {
                if self.rainfall < self.config.drought_threshold {
                    self.dry_streak += 1;
                    self.wet_streak = 0;
                    if self.dry_streak >= self.config.drought_ticks {
                        self.regime = WeatherRegime::Drought;
                        self.drought_events += 1;
                        event = Some(WeatherEvent::DroughtStarted);
                    }
                } else if self.rainfall > self.config.flood_threshold {
                    self.wet_streak += 1;
                    self.dry_streak = 0;
                    if self.wet_streak >= self.config.flood_ticks {
                        self.regime = WeatherRegime::Flood;
                        self.flood_events += 1;
                        event = Some(WeatherEvent::FloodStarted);
                    }
                } else {
                    self.dry_streak = 0;
                    self.wet_streak = 0;
                }
            }
            WeatherRegime::Drought => {
                // Hysteresis: rain must clearly recover before the drought lifts.
                if self.rainfall > self.config.drought_threshold + Fixed::from_f64(0.1) {
                    self.regime = WeatherRegime::Normal;
                    self.dry_streak = 0;
                    event = Some(WeatherEvent::DroughtEnded);
                }
            }
            WeatherRegime::Flood => {
                if self.rainfall < self.config.flood_threshold - Fixed::from_f64(0.1) {
                    self.regime = WeatherRegime::Normal;
                    self.wet_streak = 0;
                    event = Some(WeatherEvent::FloodEnded);
                }
            }
        }
        event
    }

    /// Crop-growth multiplier for farm production. Designed to be ≈ identity
    /// (1.015 at the 0.55 rainfall baseline) in normal weather so calibrated
    /// windows drift only mildly, while an emergent drought nearly halves
    /// output (0.61) — genuine famine pressure that scripts previously
    /// faked. A flood modestly boosts growth (1.07).
    pub fn growth_factor(&self) -> Fixed {
        let rain_term = (Fixed::from_f64(0.85) + self.rainfall * Fixed::from_f64(0.3))
            .clamp(Fixed::from_f64(0.5), Fixed::from_f64(1.2));
        let regime_mult = match self.regime {
            WeatherRegime::Normal => Fixed::ONE,
            WeatherRegime::Drought => Fixed::from_f64(0.6),
            WeatherRegime::Flood => Fixed::from_f64(1.05),
        };
        (rain_term * regime_mult).clamp(Fixed::from_f64(0.4), Fixed::from_f64(1.25))
    }

    /// Spoilage multiplier for perishable stock. Hot weather rots food
    /// faster (1.03 at the 0.6 temperature baseline), droughts keep it
    /// better (×0.8), floods rot it faster (×1.2).
    pub fn temperature_factor(&self) -> Fixed {
        let base = (Fixed::from_f64(0.85) + self.temperature * Fixed::from_f64(0.3))
            .clamp(Fixed::from_f64(0.8), Fixed::from_f64(1.2));
        let regime_mult = match self.regime {
            WeatherRegime::Normal => Fixed::ONE,
            WeatherRegime::Drought => Fixed::from_f64(0.8),
            WeatherRegime::Flood => Fixed::from_f64(1.2),
        };
        (base * regime_mult).clamp(Fixed::from_f64(0.7), Fixed::from_f64(1.3))
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

    debug_assert_eq!(
        site_fertilities.len(),
        site_work_ticks.len(),
        "ecology: site_fertilities and site_work_ticks length mismatch"
    );
    for i in 0..n {
        // Overfarming decay: more work = more decay
        // Guard against mismatched site_work_ticks length (e.g. empty world)
        let work_pressure = if i < site_work_ticks.len() && site_work_ticks[i] > 0 {
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
        assert_eq!(Season::Winter.growth_modifier(), Fixed::ZERO);
    }

    #[test]
    fn growth_modifier_summer_is_max() {
        assert_eq!(Season::Summer.growth_modifier(), Fixed::from_f64(1.0));
    }

    #[test]
    fn ecology_system_applies_fertility_decay() {
        let config = EcologyConfig::default();
        let mut fertilities = vec![Fixed::from_f64(0.8); 3];
        let work_ticks = vec![10, 0, 5];

        system_ecology(
            &SeasonTracker::new(100),
            &mut fertilities,
            &work_ticks,
            &config,
        );

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

    // ── Weather (§5, Iteration 147) ──────────────────────────────
    // Note: the tests only hand the rng to `advance` (whose `impl rand::Rng`
    // bound brings the trait methods into scope) — `SeedableRng` is the only
    // import needed here.
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;

    #[test]
    fn weather_rainfall_baseline_is_seasonal() {
        assert!(Season::Autumn.rainfall_baseline() > Season::Winter.rainfall_baseline());
        assert!(Season::Autumn.rainfall_baseline() > Season::Summer.rainfall_baseline());
    }

    #[test]
    fn weather_advance_stays_bounded_and_is_seed_deterministic() {
        let config = WeatherConfig::default();
        let run = || {
            let mut tracker = WeatherTracker::new(config.clone());
            let mut rng = ChaCha8Rng::seed_from_u64(7);
            for _ in 0..1000 {
                tracker.advance(Season::Spring, &mut rng);
                assert!(
                    (Fixed::ZERO..=Fixed::ONE).contains(&tracker.temperature),
                    "temperature out of [0,1]: {}",
                    tracker.temperature.to_f64()
                );
                assert!(
                    (Fixed::ZERO..=Fixed::ONE).contains(&tracker.rainfall),
                    "rainfall out of [0,1]: {}",
                    tracker.rainfall.to_f64()
                );
            }
            (
                tracker.temperature.to_raw(),
                tracker.rainfall.to_raw(),
                tracker.regime,
            )
        };
        assert_eq!(run(), run(), "weather must be seed-deterministic");
    }

    #[test]
    fn weather_drought_emerges_after_sustained_dry_spell() {
        // reversion 1.0 + zero noise pins rainfall to the 0.55 Spring
        // baseline; a 0.8 threshold makes every tick dry → drought on tick 10.
        let config = WeatherConfig {
            mean_reversion: Fixed::ONE,
            rainfall_noise: Fixed::ZERO,
            drought_threshold: Fixed::from_f64(0.8),
            drought_ticks: 10,
            ..WeatherConfig::default()
        };
        let mut tracker = WeatherTracker::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        // The event fires on the drought_ticks-th advance (dry_streak reaches
        // the threshold then) — the first drought_ticks - 1 must be silent.
        for tick in 0..9 {
            let ev = tracker.advance(Season::Spring, &mut rng);
            assert!(
                ev.is_none(),
                "no event before the dry spell completes (tick {tick})"
            );
        }
        let ev = tracker.advance(Season::Spring, &mut rng);
        assert_eq!(ev, Some(WeatherEvent::DroughtStarted));
        assert_eq!(tracker.regime, WeatherRegime::Drought);
        assert_eq!(tracker.drought_events, 1);
        // Hysteresis: rainfall pinned at 0.55 < 0.9 keeps the drought active.
        let ev = tracker.advance(Season::Spring, &mut rng);
        assert_eq!(ev, None);
        assert_eq!(tracker.regime, WeatherRegime::Drought);
    }

    #[test]
    fn weather_flood_emerges_after_sustained_wet_spell() {
        // Baseline 0.55 above a 0.5 threshold → every tick wet → flood on tick 8.
        let config = WeatherConfig {
            mean_reversion: Fixed::ONE,
            rainfall_noise: Fixed::ZERO,
            flood_threshold: Fixed::from_f64(0.5),
            flood_ticks: 8,
            ..WeatherConfig::default()
        };
        let mut tracker = WeatherTracker::new(config);
        let mut rng = ChaCha8Rng::seed_from_u64(1);
        // Same cadence as the drought test: silent for flood_ticks - 1, the
        // flood_ticks-th advance declares the regime.
        for _ in 0..7 {
            tracker.advance(Season::Spring, &mut rng);
        }
        let ev = tracker.advance(Season::Spring, &mut rng);
        assert_eq!(ev, Some(WeatherEvent::FloodStarted));
        assert_eq!(tracker.regime, WeatherRegime::Flood);
        assert_eq!(tracker.flood_events, 1);
    }

    #[test]
    fn weather_growth_factor_identity_in_normal_and_suppressed_in_drought() {
        let mut normal = WeatherTracker::new(WeatherConfig::default());
        normal.rainfall = Fixed::from_f64(0.55);
        normal.regime = WeatherRegime::Normal;
        let g_normal = normal.growth_factor();
        // ≈ 1.015 at the rainfall baseline — near-identity, mild calibrated drift.
        assert!(
            g_normal > Fixed::ONE && g_normal < Fixed::from_f64(1.05),
            "normal growth factor should hover at identity, got {}",
            g_normal.to_f64()
        );

        let mut drought = normal.clone();
        drought.rainfall = Fixed::from_f64(0.1);
        drought.regime = WeatherRegime::Drought;
        let g_drought = drought.growth_factor();
        assert!(
            g_drought < Fixed::from_f64(0.7),
            "drought must nearly halve farm output, got {}",
            g_drought.to_f64()
        );
        assert!(g_drought < g_normal);

        // Last use of `normal` — move instead of clone (clippy).
        let mut flood = normal;
        flood.rainfall = Fixed::from_f64(0.9);
        flood.regime = WeatherRegime::Flood;
        let g_flood = flood.growth_factor();
        assert!(g_flood > g_normal, "flood should modestly boost growth");
    }

    #[test]
    fn weather_temperature_factor_is_bounded_and_regime_gated() {
        let mut normal = WeatherTracker::new(WeatherConfig::default());
        normal.temperature = Fixed::from_f64(0.6);
        normal.regime = WeatherRegime::Normal;
        let t_normal = normal.temperature_factor();
        assert!(
            t_normal > Fixed::ONE && t_normal < Fixed::from_f64(1.1),
            "normal temperature factor should hover at identity, got {}",
            t_normal.to_f64()
        );

        let mut flood = normal.clone();
        flood.regime = WeatherRegime::Flood;
        assert!(
            flood.temperature_factor() > t_normal,
            "flood must rot food faster"
        );

        // Last use of `normal` — move instead of clone (clippy).
        let mut drought = normal;
        drought.regime = WeatherRegime::Drought;
        assert!(
            drought.temperature_factor() < t_normal,
            "drought must keep food better"
        );
    }
}
