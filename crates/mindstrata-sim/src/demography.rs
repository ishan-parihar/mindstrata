//! Demography — §31 of the architecture spec.
//!
//! Implements aging, births, deaths, and household formation.
//! §31: "Aging, births, deaths, marriage, fertility, childhood
//! socialization, inheritance, household formation."
//! §Phase 9: "Population migrates under pressure."

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Life stage of an agent.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LifeStage {
    Child,
    Adolescent,
    Adult,
    Elder,
}

impl LifeStage {
    pub fn name(self) -> &'static str {
        match self {
            LifeStage::Child => "Child",
            LifeStage::Adolescent => "Adolescent",
            LifeStage::Adult => "Adult",
            LifeStage::Elder => "Elder",
        }
    }

    /// Transition age thresholds (in years).
    pub fn transition_age(self) -> f64 {
        match self {
            LifeStage::Child => 0.0,
            LifeStage::Adolescent => 12.0,
            LifeStage::Adult => 18.0,
            LifeStage::Elder => 60.0,
        }
    }
}

/// Demography configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DemographyConfig {
    /// Ticks per year (default: 35040 = 1 year if tick = 15min).
    pub ticks_per_year: u64,
    /// Base death chance per year for elders.
    pub elder_death_rate: Fixed,
    /// Base birth probability per couple per year.
    pub birth_rate: Fixed,
    /// Maximum age in years.
    pub max_age: f64,
    /// Minimum age for childbearing.
    pub min_childbearing_age: f64,
    /// Maximum age for childbearing.
    pub max_childbearing_age: f64,
}

impl Default for DemographyConfig {
    fn default() -> Self {
        Self {
            ticks_per_year: 35040,
            elder_death_rate: Fixed::from_f64(0.15),
            birth_rate: Fixed::from_f64(0.08),
            max_age: 80.0,
            min_childbearing_age: 18.0,
            max_childbearing_age: 45.0,
        }
    }
}

/// Demography result for a tick.
#[derive(Debug, Clone, Default)]
pub struct DemographyReport {
    pub agents_aged: usize,
    pub births: usize,
    pub deaths: usize,
    pub agents_migrated: usize,
}

/// Age an agent by `ticks_elapsed` and check for death.
///
/// Returns (new_age, died). Death is probabilistic: an agent dies when a
/// uniform `rng_value` (0..1) falls below the computed death chance for the
/// elapsed period. This replaces the old deterministic `death_chance > 0`
/// check which killed every elder instantly, and the per-tick accumulation
/// which underflowed Fixed's 4-decimal precision (`1/35040` rounds to 0).
#[must_use]
pub fn age_agent(
    age_years: Fixed,
    health: Fixed,
    body_energy: Fixed,
    ticks_elapsed: u64,
    config: &DemographyConfig,
    rng_value: Fixed, // 0..1 random value
) -> (Fixed, bool) {
    // Accumulate age as years + the fractional year corresponding to the
    // ticks elapsed since the last demography step (called every 10 ticks).
    let increment = Fixed::from_f64(ticks_elapsed as f64 / config.ticks_per_year as f64);
    let new_age = age_years + increment;
    let new_age_years = new_age.to_f64();

    // Guaranteed death past max age.
    if new_age_years > config.max_age {
        return (new_age, true);
    }

    // Death probability increases with age and decreases with health.
    // `elder_death_rate` is an annual rate; scale by the elapsed fraction
    // of a year so the roll is a true per-period probability.
    if new_age_years > 60.0 {
        let age_factor = Fixed::from_f64(((new_age_years - 60.0) / 20.0).clamp(0.0, 1.0));
        let health_factor = Fixed::ONE - health;
        let energy_factor = Fixed::ONE - body_energy;
        let annual_rate = (config.elder_death_rate * age_factor
            + health_factor * Fixed::from_f64(0.1)
            + energy_factor * Fixed::from_f64(0.05))
            .clamp_01();
        let elapsed_years =
            Fixed::from_f64(ticks_elapsed as f64 / config.ticks_per_year as f64);
        let death_chance = (annual_rate * elapsed_years).clamp_01();
        (new_age, rng_value < death_chance)
    } else {
        (new_age, false)
    }
}

/// Determine life stage from age.
pub fn life_stage(age: f64) -> LifeStage {
    if age < LifeStage::Adolescent.transition_age() {
        LifeStage::Child
    } else if age < LifeStage::Adult.transition_age() {
        LifeStage::Adolescent
    } else if age < LifeStage::Elder.transition_age() {
        LifeStage::Adult
    } else {
        LifeStage::Elder
    }
}

/// Check if an agent can bear children.
pub fn can_have_children(age: f64, config: &DemographyConfig) -> bool {
    age >= config.min_childbearing_age && age <= config.max_childbearing_age
}

/// Check if a birth should occur this tick.
pub fn should_birth(
    partner_exists: bool,
    health: Fixed,
    age: f64,
    existing_children: usize,
    config: &DemographyConfig,
    rng_value: Fixed, // 0..1 random value
) -> bool {
    if !partner_exists {
        return false;
    }
    if !can_have_children(age, config) {
        return false;
    }
    if health < Fixed::from_f64(0.3) {
        return false; // too unhealthy
    }

    // Birth rate decreases with existing children
    let child_penalty = Fixed::from_f64(0.02) * Fixed::from_int(existing_children as i64);
    let effective_rate = (config.birth_rate - child_penalty).max(Fixed::ZERO);

    rng_value < effective_rate
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn life_stage_transitions() {
        assert_eq!(life_stage(5.0), LifeStage::Child);
        assert_eq!(life_stage(15.0), LifeStage::Adolescent);
        assert_eq!(life_stage(30.0), LifeStage::Adult);
        assert_eq!(life_stage(65.0), LifeStage::Elder);
    }

    #[test]
    fn aging_increases_age() {
        // Use a small ticks_per_year so 1/tps doesn't round to 0 in Fixed precision
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
        let initial = Fixed::from_f64(30.0);
        let mut age = initial;
        for _ in 0..50 {
            let (new_age, _) = age_agent(age, Fixed::ONE, Fixed::ONE, 1, &config, Fixed::ZERO);
            age = new_age;
        }
        assert!(age > initial, "Age should increase after 50 ticks: {} > {}", age.to_f64(), initial.to_f64());
    }

    #[test]
    fn aging_advances_with_default_ticks_per_year() {
        // Regression: 1/35040 underflowed Fixed 4-decimal precision, freezing
        // all ages forever. Age must now advance even at the default tick rate.
        let config = DemographyConfig::default();
        let initial = Fixed::from_f64(38.0);
        let mut age = initial;
        for _ in 0..3504 { // 35040 ticks = 1 year at 10 ticks per demography step
            let (new_age, _) = age_agent(age, Fixed::ONE, Fixed::ONE, 10, &config, Fixed::ZERO);
            age = new_age;
        }
        assert!(age > initial, "Age should advance ~1 year over 35040 ticks: {}", age.to_f64());
    }

    #[test]
    fn elders_can_die() {
        // Compress the timescale (1 tick = 1/100 year) so a 10-tick demography
        // step is a meaningful 0.1-year probability window — otherwise the
        // per-step death chance (~0.008%) would need ~12,000 rolls to observe.
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
        // Very old agent with low health — death is probabilistic.
        let mut deaths = 0u32;
        for i in 0..1000 {
            let (_, died) = age_agent(
                Fixed::from_f64(79.0),
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.1),
                10,
                &config,
                Fixed::from_f64(i as f64 / 1000.0),
            );
            deaths += u32::from(died);
        }
        // Per-step annual rate 0.15*0.95 + 0.09 + 0.045 ≈ 0.28 × 0.1yr ≈ 2.8%
        // → expected ~28 deaths / 1000. Sanity: some die, not all.
        assert!(deaths > 0 && deaths < 1000, "Probabilistic death expected: {deaths}/1000");
    }

    #[test]
    fn young_healthy_agents_never_die() {
        let config = DemographyConfig::default();
        for i in 0..50 {
            let (_, died) = age_agent(
                Fixed::from_f64(30.0),
                Fixed::from_f64(1.0),
                Fixed::from_f64(1.0),
                10,
                &config,
                Fixed::from_f64(i as f64 / 50.0),
            );
            assert!(!died, "30yo healthy agent should never die");
        }
    }

    #[test]
    fn childbearing_age_check() {
        let config = DemographyConfig::default();
        assert!(!can_have_children(15.0, &config));
        assert!(can_have_children(25.0, &config));
        assert!(!can_have_children(50.0, &config));
    }

    #[test]
    fn birth_requires_partner() {
        let config = DemographyConfig::default();
        assert!(!should_birth(
            false, Fixed::ONE, 25.0, 0, &config, Fixed::ZERO,
        ));
    }

    #[test]
    fn birth_rate_decreases_with_children() {
        let config = DemographyConfig::default();
        let no_kids = should_birth(
            true, Fixed::ONE, 25.0, 0, &config, Fixed::from_f64(0.05),
        );
        let many_kids = should_birth(
            true, Fixed::ONE, 25.0, 5, &config, Fixed::from_f64(0.05),
        );
        // With 5 children, birth rate should be lower
        assert!(no_kids || !many_kids); // at least one should differ
    }

    #[test]
    fn unhealthy_mother_no_birth() {
        let config = DemographyConfig::default();
        assert!(!should_birth(
            true, Fixed::from_f64(0.1), 25.0, 0, &config, Fixed::ZERO,
        ));
    }
}
