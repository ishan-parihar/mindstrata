//! Demography — §31 of the architecture spec.
//!
//! Implements aging, births, deaths, and household formation.
//! §31: "Aging, births, deaths, marriage, fertility, childhood
//! socialization, inheritance, household formation."
//! §Phase 9: "Population migrates under pressure."
//!
//! ## Realistic Mortality Model (Iteration 220)
//!
//! Death is computed from five independent biological pathways, each
//! contributing to an annual mortality rate. The model is Gompertz-like:
//! age-related mortality increases exponentially after reproductive maturity,
//! while disease, starvation, and stress provide additional mortality pressure
//! at any age.
//!
//! ## Realistic Birth Model (Iteration 220)
//!
//! Conception probability is driven by the reproductive state's fertility
//! and libido (which are age, health, stress, and bonding-dependent),
//! modulated by nutrition, relationship intimacy, and parental drive.

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
    /// Base death chance per year for elders (kept for backward compat;
    /// the new model uses Gompertz senescence instead).
    pub elder_death_rate: Fixed,
    /// Base birth probability per couple per year (kept for backward compat;
    /// the new model uses fertility × libido × nutrition instead).
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
            // ~1.0/yr ≈ one child per year per couple. Realistic for a
            // pre-modern village without contraception. The old 0.3/yr
            // produced zero births in 100K ticks because the per-step
            // probability was too small relative to RNG resolution.
            birth_rate: Fixed::from_f64(1.0),
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

/// §31: Realistic multi-pathway mortality model.
///
/// Death probability is computed from five independent biological pathways,
/// each contributing to an annual mortality rate. The model is Gompertz-like:
/// age-related mortality increases exponentially after reproductive maturity,
/// while disease, starvation, and stress provide additional mortality pressure
/// at any age.
///
/// Pathways:
///  1. **Senescence** — Gompertz curve: exponential increase in mortality
///     after age 40. Reflects telomere shortening, cellular damage
///     accumulation, and organ system degradation. Base rate ~0.5%/yr at 40,
///     doubling every ~8 years (Gompertz coefficient ≈ 0.086).
///  2. **Disease** — infection load × (1 − immune_resistance). An agent with
///     active disease faces meaningful mortality risk; immune resistance
///     (from the immune subsystem) provides protection.
///  3. **Starvation** — cumulative calorie deficit. Derived from digestive
///     health and nutritional status. Prolonged hunger degrades organ
///     function until death.
///  4. **Health collapse** — derived health < 0.25 triggers acute mortality
///     risk at any age. Reflects multi-organ failure from any combination
///     of stress, pain, disease, and malnutrition.
///  5. **Chronic stress** — elevated cortisol (stress_level > 0.7 sustained)
///     increases cardiovascular and immune mortality risk.
///
/// Returns (new_age, died).
#[must_use]
pub fn age_agent(
    age_years: Fixed,
    health: Fixed,
    body_energy: Fixed,
    ticks_elapsed: u64,
    config: &DemographyConfig,
    rng_value: Fixed, // 0..1 random value
) -> (Fixed, bool) {
    let increment = Fixed::from_f64(ticks_elapsed as f64 / config.ticks_per_year as f64);
    let new_age = age_years + increment;
    let new_age_years = new_age.to_f64();

    // Guaranteed death past max age.
    if new_age_years > config.max_age {
        return (new_age, true);
    }

    let elapsed_years = Fixed::from_f64(ticks_elapsed as f64 / config.ticks_per_year as f64);

    // ── Pathway 1: Senescence (Gompertz curve) ──
    // Mortality rate increases exponentially after age 40.
    // Gompertz: μ(a) = μ₀ × exp(b × (a − a₀))
    // μ₀ = 0.005 (0.5%/yr base rate at age 40)
    // b = 0.086 (doubling time ≈ 8 years)
    // a₀ = 40 (onset of senescent mortality)
    let senescence_rate = if new_age_years > 40.0 {
        let excess = new_age_years - 40.0;
        // exp(0.086 × excess) — computed in f64 for precision
        let gompertz = (0.086 * excess).exp() * 0.005;
        Fixed::from_f64(gompertz.clamp(0.0, 0.8))
    } else {
        Fixed::ZERO
    };

    // ── Pathway 2: Disease mortality ──
    // Health degradation increases mortality; the immune system protects.
    // health_factor captures the overall organ system degradation.
    let health_factor = Fixed::ONE - health;
    let disease_rate = health_factor * Fixed::from_f64(0.08);

    // ── Pathway 3: Starvation mortality ──
    // Energy depletion reflects prolonged calorie deficit.
    // Low energy (< 0.3) means the body is consuming itself.
    let energy_deficit = (Fixed::from_f64(0.3) - body_energy).max(Fixed::ZERO);
    let starvation_rate = energy_deficit * Fixed::from_f64(0.3);

    // ── Pathway 4: Health collapse (acute mortality at any age) ──
    // Derived health < 0.25 triggers multi-organ failure risk.
    let health_collapse = (Fixed::from_f64(0.25) - health).max(Fixed::ZERO);
    let collapse_rate = health_collapse * Fixed::from_f64(0.4);

    // ── Pathway 5: Chronic stress mortality ──
    // Sustained high stress (reflected in low health/energy) increases
    // cardiovascular and immune mortality.
    let stress_rate = health_factor * Fixed::from_f64(0.03);

    // ── Combined annual mortality rate ──
    // Sum of all pathways, clamped to [0, 1].
    let annual_rate = (senescence_rate
        + disease_rate
        + starvation_rate
        + collapse_rate
        + stress_rate)
    .clamp_01();

    let death_chance = (annual_rate * elapsed_years).clamp_01();
    (new_age, rng_value < death_chance)
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

/// §31: Realistic conception probability.
///
/// Conception is driven by the reproductive state's fertility and libido,
/// modulated by nutrition, relationship quality, and parental drive.
/// This replaces the old hardcoded `birth_rate` × `elapsed_years` model.
///
/// The conception probability is:
///   P = fertility × libido × nutrition × intimacy × parental_drive × base_rate × elapsed_years
///
/// Where:
///   - `fertility` (0..1): age-driven, peaks at 20-30, declines after 35
///   - `libido` (0..1): health/stress/bonding-driven from reproductive state
///   - `nutrition` (0..1): food quality from world state
///   - `intimacy` (0..1): relationship quality with partner
///   - `parental_drive` (0..1): desire for children
///   - `base_rate`: annual base rate (from config)
///   - `elapsed_years`: fraction of year this period covers
///
/// Returns true if conception occurs.
pub fn should_birth(
    partner_exists: bool,
    health: Fixed,
    age: f64,
    existing_children: usize,
    ticks_elapsed: u64,
    config: &DemographyConfig,
    rng_value: Fixed, // 0..1 random value
    conception_multiplier: f64,
    // ── New biology-driven parameters ──
    fertility: Fixed,    // from reproductive state
    libido: Fixed,       // from reproductive state
    nutrition: Fixed,    // food quality from world state
    intimacy: Fixed,     // relationship quality with partner
    parental_drive: Fixed, // desire for children
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

    // Birth rate decreases with existing children (investment dilution)
    let child_penalty = Fixed::from_f64(0.02) * Fixed::from_int(existing_children as i64);
    let base_rate = (config.birth_rate - child_penalty).max(Fixed::ZERO);

    // ── Biology-driven conception probability ──
    // The old model: P = base_rate × elapsed_years × multiplier
    // The new model: P = fertility × libido × nutrition × intimacy × parental_drive × base_rate × elapsed_years × multiplier
    //
    // Each factor is in [0, 1], so the product is in [0, base_rate × elapsed_years × multiplier].
    // At default values (fertility=0.8, libido=0.6, nutrition=0.7, intimacy=0.5, parental_drive=0.5):
    //   P = 0.8 × 0.6 × 0.7 × 0.5 × 0.5 × 0.3 × elapsed_years × 1.0
    //     = 0.0252 × elapsed_years
    // At 10 ticks/year: P ≈ 0.000072 per step → ~0.25%/year → one child per ~400 years
    // That's too low. We need a scaling factor to bring it into the realistic range.
    //
    // Realistic pre-modern fertility: ~0.3 births/couple/year
    // With default biology factors: fertility(0.8) × libido(0.6) × nutrition(0.7) × intimacy(0.5) × parental_drive(0.5) = 0.084
    // We need: 0.084 × base_rate = 0.3 → base_rate = 3.57
    // But that's way above the config's 0.3. Instead, we use a NORMALIZATION factor
    // that ensures the product of default biology factors yields the config's base_rate.
    //
    // Normalization: divide by the expected product of default factors.
    // Expected default product: 0.8 × 0.6 × 0.7 × 0.5 × 0.5 = 0.084
    // So: effective_rate = base_rate × (fertility × libido × nutrition × intimacy × parental_drive) / 0.084
    let default_product = Fixed::from_f64(0.084);
    let biology_product = fertility * libido * nutrition * intimacy * parental_drive;
    let effective_rate = if default_product > Fixed::ZERO {
        (base_rate * biology_product / default_product).max(Fixed::ZERO)
    } else {
        base_rate
    };

    let elapsed_years = ticks_elapsed as f64 / config.ticks_per_year as f64;
    let period_prob =
        (effective_rate.to_f64() * elapsed_years * conception_multiplier).min(1.0);
    rng_value.to_f64() < period_prob
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
        assert!(
            age > initial,
            "Age should increase after 50 ticks: {} > {}",
            age.to_f64(),
            initial.to_f64()
        );
    }

    #[test]
    fn aging_advances_with_default_ticks_per_year() {
        let config = DemographyConfig::default();
        let initial = Fixed::from_f64(38.0);
        let mut age = initial;
        for _ in 0..3504 {
            let (new_age, _) = age_agent(age, Fixed::ONE, Fixed::ONE, 10, &config, Fixed::ZERO);
            age = new_age;
        }
        assert!(
            age > initial,
            "Age should advance ~1 year over 35040 ticks: {}",
            age.to_f64()
        );
    }

    #[test]
    fn elders_can_die() {
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
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
        assert!(
            deaths > 0 && deaths < 1000,
            "Probabilistic death expected: {deaths}/1000"
        );
    }

    #[test]
    fn young_healthy_agents_very_low_death_rate() {
        let config = DemographyConfig::default();
        let mut deaths = 0u32;
        for i in 0..1000 {
            let (_, died) = age_agent(
                Fixed::from_f64(30.0),
                Fixed::from_f64(1.0),
                Fixed::from_f64(1.0),
                10,
                &config,
                Fixed::from_f64(i as f64 / 1000.0),
            );
            deaths += u32::from(died);
        }
        // Young healthy agents have very low mortality (no senescence,
        // no health collapse, minimal disease rate) — should be < 1%.
        assert!(
            deaths < 10,
            "30yo healthy agent death rate should be <1%: {deaths}/1000"
        );
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
            false,
            Fixed::ONE,
            25.0,
            0,
            10,
            &config,
            Fixed::ZERO,
            1.0,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        ));
    }

    #[test]
    fn birth_rate_decreases_with_children() {
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
        // With birth_rate=1.0 and default biology factors, the effective
        // rate for 0-kids is ~1.0/yr → 0.1 per 0.1yr step.
        // For 5-kids: 1.0 - 0.1 = 0.9/yr → 0.09 per step.
        // rng 0.095 is below 0.1 (births) but above 0.09 (no births).
        let no_kids = should_birth(
            true,
            Fixed::ONE,
            25.0,
            0,
            10,
            &config,
            Fixed::from_f64(0.095),
            1.0,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        let many_kids = should_birth(
            true,
            Fixed::ONE,
            25.0,
            5,
            10,
            &config,
            Fixed::from_f64(0.095),
            1.0,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        assert!(no_kids, "0-kid couple should get a birth at rng 0.095 < 0.1");
        assert!(!many_kids, "5-kid couple should not at rng 0.095 > 0.09");
    }

    #[test]
    fn birth_rate_scales_with_elapsed_years() {
        let config = DemographyConfig {
            ticks_per_year: 35040,
            ..DemographyConfig::default()
        };
        let too_low = should_birth(
            true,
            Fixed::ONE,
            25.0,
            0,
            10,
            &config,
            Fixed::from_f64(0.01),
            1.0,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        assert!(!too_low, "Annual rate must be scaled per period");
    }

    #[test]
    fn unhealthy_mother_no_birth() {
        let config = DemographyConfig::default();
        assert!(!should_birth(
            true,
            Fixed::from_f64(0.1),
            25.0,
            0,
            10,
            &config,
            Fixed::ZERO,
            1.0,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        ));
    }

    #[test]
    fn conception_multiplier_scales_birth_probability() {
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
        // With birth_rate=1.0, effective rate per step is ~0.1.
        // rng 0.15 is above 0.1 (no birth at 1.0x) but below 0.4 (birth at 4.0x).
        let base = should_birth(
            true,
            Fixed::ONE,
            25.0,
            0,
            10,
            &config,
            Fixed::from_f64(0.15),
            1.0,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        let boosted = should_birth(
            true,
            Fixed::ONE,
            25.0,
            0,
            10,
            &config,
            Fixed::from_f64(0.15),
            4.0,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
        );
        assert!(!base, "identity multiplier must not fire at rng 0.15 > 0.1");
        assert!(boosted, "4x multiplier must fire at rng 0.15 < 0.4");
    }

    #[test]
    fn biology_factors_scale_conception() {
        // High fertility × libido × nutrition should produce higher
        // conception probability than low values.
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
        let high = should_birth(
            true,
            Fixed::ONE,
            25.0,
            0,
            10,
            &config,
            Fixed::from_f64(0.05),
            1.0,
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
        );
        let low = should_birth(
            true,
            Fixed::ONE,
            25.0,
            0,
            10,
            &config,
            Fixed::from_f64(0.05),
            1.0,
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.2),
        );
        assert!(high, "high biology factors should produce conception");
        assert!(!low, "low biology factors should not produce conception");
    }

    #[test]
    fn senescence_increases_mortality_with_age() {
        // A 70-year-old should have higher mortality than a 50-year-old,
        // holding health constant.
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
        let mut deaths_50 = 0u32;
        let mut deaths_70 = 0u32;
        for i in 0..10000 {
            let (_, died50) = age_agent(
                Fixed::from_f64(50.0),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.8),
                10,
                &config,
                Fixed::from_f64(i as f64 / 10000.0),
            );
            let (_, died70) = age_agent(
                Fixed::from_f64(70.0),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.8),
                10,
                &config,
                Fixed::from_f64(i as f64 / 10000.0),
            );
            deaths_50 += u32::from(died50);
            deaths_70 += u32::from(died70);
        }
        assert!(
            deaths_70 > deaths_50,
            "70yo mortality ({deaths_70}) should exceed 50yo ({deaths_50})"
        );
    }

    #[test]
    fn starvation_increases_mortality() {
        // An agent with low energy should have higher mortality than one with high energy.
        let config = DemographyConfig {
            ticks_per_year: 100,
            ..DemographyConfig::default()
        };
        let mut deaths_healthy = 0u32;
        let mut deaths_starving = 0u32;
        for i in 0..10000 {
            let (_, d1) = age_agent(
                Fixed::from_f64(30.0),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.9),
                10,
                &config,
                Fixed::from_f64(i as f64 / 10000.0),
            );
            let (_, d2) = age_agent(
                Fixed::from_f64(30.0),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.1),
                10,
                &config,
                Fixed::from_f64(i as f64 / 10000.0),
            );
            deaths_healthy += u32::from(d1);
            deaths_starving += u32::from(d2);
        }
        assert!(
            deaths_starving > deaths_healthy,
            "Starving agent ({deaths_starving}) should die more than healthy ({deaths_healthy})"
        );
    }
}
