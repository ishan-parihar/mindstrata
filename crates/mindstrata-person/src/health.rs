//! Health — §32 of the architecture spec.
//!
//! Implements disease transmission, injury, and malnutrition effects.
//! §32: "Health, immunity, injury, malnutrition, disease transmission,
//! sanitation, epidemics."
//! §Phase 9: "Disease spreads in dense settlements."

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Hard cap on concurrent diseases per agent (Iteration 185): correct
/// operation holds at most one disease per kind × 5 kinds = 5; the cap is a
/// pathological-state guard against any unbounded infection path re-opening
/// (see `system_health`).
pub const MAX_CONCURRENT_DISEASES: usize = 8;

/// Types of health conditions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DiseaseKind {
    /// Common cold — mild, self-limiting.
    Cold,
    /// Fever — moderate, reduces energy.
    Fever,
    /// Wound infection — from untreated injury.
    WoundInfection,
    /// Malnutrition — from chronic hunger.
    Malnutrition,
    /// Epidemic disease — highly transmissible.
    Epidemic,
}

impl DiseaseKind {
    /// Severity multiplier (0 = mild, 1 = severe).
    pub fn severity(self) -> Fixed {
        match self {
            DiseaseKind::Cold => Fixed::from_f64(0.1),
            DiseaseKind::Fever => Fixed::from_f64(0.3),
            DiseaseKind::WoundInfection => Fixed::from_f64(0.5),
            DiseaseKind::Malnutrition => Fixed::from_f64(0.4),
            DiseaseKind::Epidemic => Fixed::from_f64(0.7),
        }
    }

    /// Base transmission probability per tick of close contact.
    pub fn transmission_rate(self) -> Fixed {
        match self {
            DiseaseKind::Cold => Fixed::from_f64(0.05),
            DiseaseKind::Fever => Fixed::from_f64(0.03),
            DiseaseKind::WoundInfection => Fixed::ZERO, // not contagious
            DiseaseKind::Malnutrition => Fixed::ZERO,   // not contagious
            DiseaseKind::Epidemic => Fixed::from_f64(0.15),
        }
    }

    /// Duration in ticks before self-resolution (if immune system is healthy).
    pub fn base_duration(self) -> u64 {
        match self {
            DiseaseKind::Cold => 200,
            DiseaseKind::Fever => 400,
            DiseaseKind::WoundInfection => 600,
            DiseaseKind::Malnutrition => 1000,
            DiseaseKind::Epidemic => 800,
        }
    }
}

/// Active disease on an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveDisease {
    /// Kind. (doc added at module migration)
    pub kind: DiseaseKind,
    /// Ticks infected. (doc added at module migration)
    pub ticks_infected: u64,
    /// Severity modifier. (doc added at module migration)
    pub severity_modifier: Fixed, // 0..1, modifies base severity
}

impl ActiveDisease {
    /// Fn. (doc added at module migration)
    pub fn new(kind: DiseaseKind) -> Self {
        Self {
            kind,
            ticks_infected: 0,
            severity_modifier: Fixed::from_f64(0.5), // start at moderate
        }
    }

    /// Effective severity considering modifier and duration.
    pub fn effective_severity(&self) -> Fixed {
        let base = self.kind.severity();
        // Severity peaks at 1/3 of duration, then declines
        let duration = self.kind.base_duration() as f64;
        let progress = self.ticks_infected as f64 / duration;
        let duration_factor = if progress < 0.33 {
            // Start with a small initial severity so newly infected agents feel it
            (Fixed::from_f64(progress * 3.0) + Fixed::from_f64(0.05)).min(Fixed::ONE)
        } else {
            Fixed::ONE - Fixed::from_f64((progress - 0.33) * 1.5).clamp_01()
        };

        (base * self.severity_modifier * duration_factor).clamp_01()
    }

    /// Should this disease resolve this tick?
    pub fn should_resolve(&self, health: Fixed) -> bool {
        // Healthy agents resolve diseases faster
        let health_speedup = health * Fixed::from_f64(0.3);
        let effective_duration = (Fixed::from_int(self.kind.base_duration() as i64)
            * (Fixed::ONE - health_speedup))
            .max(Fixed::from_int(100));
        (self.ticks_infected as i64) >= effective_duration.to_raw() / 10000
    }
}

/// Health configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HealthConfig {
    /// Base disease transmission multiplier.
    pub transmission_multiplier: Fixed,
    /// Minimum health for natural immunity.
    pub immunity_threshold: Fixed,
    /// Health decay per tick when malnourished (hunger > 0.7).
    pub malnutrition_decay: Fixed,
    /// Injury chance per threat/conflict interaction.
    pub injury_chance: Fixed,
    /// Health recovery rate per tick when resting and fed.
    pub recovery_rate: Fixed,
}

impl Default for HealthConfig {
    fn default() -> Self {
        Self {
            transmission_multiplier: Fixed::from_f64(1.0),
            immunity_threshold: Fixed::from_f64(0.7),
            malnutrition_decay: Fixed::from_f64(0.002),
            injury_chance: Fixed::from_f64(0.1),
            recovery_rate: Fixed::from_f64(0.001),
        }
    }
}

/// Compute health effects for one tick on an agent.
pub fn system_health(
    health: &mut Fixed,
    energy: &mut Fixed,
    diseases: &mut Vec<ActiveDisease>,
    hunger: Fixed,
    fatigue: Fixed,
    config: &HealthConfig,
) {
    // Malnutrition effect: chronic hunger degrades health
    if hunger > Fixed::from_f64(0.7) {
        *health = (*health - config.malnutrition_decay).max(Fixed::ZERO);
    }

    // Disease effects
    let mut total_severity = Fixed::ZERO;
    for disease in diseases.iter_mut() {
        disease.ticks_infected += 1;
        total_severity += disease.effective_severity();
    }

    // Diseases reduce energy and health
    *energy = (*energy - total_severity * Fixed::from_f64(0.005)).max(Fixed::ZERO);
    *health = (*health - total_severity * Fixed::from_f64(0.001)).max(Fixed::ZERO);

    // Natural recovery when healthy and fed — scaled by current health
    // Sick agents recover more slowly (realistic: disease suppresses recovery)
    if hunger < Fixed::from_f64(0.3) && fatigue < Fixed::from_f64(0.5) {
        let health_factor = *health; // lower health = slower recovery
        let recovery = config.recovery_rate * health_factor;
        *health = (*health + recovery).min(Fixed::ONE);
        *energy = (*energy + recovery * Fixed::from_f64(2.0)).min(Fixed::ONE);
    }

    // Resolve diseases that have run their course
    diseases.retain(|d| {
        let resolved = d.should_resolve(*health);
        if resolved {
            tracing::debug!(disease = ?d.kind, "Disease resolved");
        }
        !resolved
    });

    // Iteration 185 (P5 100K audit): hard cap on concurrent diseases per
    // agent. Correct operation (one per kind × 5 kinds) never exceeds 5,
    // so this is a pathological-state guard: if any regression re-opens an
    // unbounded infection path (the 17b same-kind dedup is the primary
    // fix), an agent can never accumulate an O(t)-sized vec that drags
    // every per-tick disease pass (system_health, contagion,
    // work_impairment) into quadratic time. Keep the NEWEST entries (they
    // are the active infections; the oldest have run their course).
    if diseases.len() > MAX_CONCURRENT_DISEASES {
        let keep = diseases.len() - MAX_CONCURRENT_DISEASES;
        diseases.drain(..keep);
    }
}

/// Productivity multiplier for an agent carrying `diseases`.
///
/// §7.5/§4.2: Sick agents work at reduced output — each disease's effective
/// severity costs half its value in productivity, floored at 20% so a
/// severely ill agent can still drag themselves to the field. Healthy agents
/// (no diseases) return 1.0 unchanged, so calibrated baseline runs carry no
/// drift; the penalty only bites under outbreaks (pestilence, famine
/// malnutrition, wound infections from violence).
#[must_use]
pub fn work_impairment(diseases: &[ActiveDisease]) -> Fixed {
    let mut total_severity = Fixed::ZERO;
    for disease in diseases {
        total_severity += disease.effective_severity();
    }
    (Fixed::ONE - total_severity * Fixed::from_f64(0.5)).max(Fixed::from_f64(0.2))
}

/// Apply injury from a conflict interaction.
///
/// §32 (Iteration 187): `rng_value` is now an EXPLICIT parameter — the
/// previous hardcoded `0.3` made the wound-infection roll a deterministic
/// no-op against the default `injury_chance` 0.1 (never infected) and left
/// the function unusable for callers with a real RNG. The caller draws from
/// its seeded stream; the function stays pure and unit-testable. Also
/// dedup-guarded: an agent never holds two WoundInfections, and the
/// pathological [`MAX_CONCURRENT_DISEASES`] cap is respected.
pub fn apply_injury(
    health: &mut Fixed,
    energy: &mut Fixed,
    diseases: &mut Vec<ActiveDisease>,
    severity: Fixed,
    config: &HealthConfig,
    rng_value: Fixed,
) {
    let damage = severity * Fixed::from_f64(0.1);
    *health = (*health - damage).max(Fixed::ZERO);
    *energy = (*energy - damage * Fixed::from_f64(0.5)).max(Fixed::ZERO);

    // Chance of wound infection from injury — the caller's seeded RNG draw.
    if rng_value < config.injury_chance
        && diseases.len() < MAX_CONCURRENT_DISEASES
        && !diseases
            .iter()
            .any(|d| d.kind == DiseaseKind::WoundInfection)
    {
        diseases.push(ActiveDisease::new(DiseaseKind::WoundInfection));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn disease_severity_varies() {
        assert!(DiseaseKind::Cold.severity() < DiseaseKind::Epidemic.severity());
    }

    #[test]
    fn active_disease_severity_peaks() {
        let mut d = ActiveDisease::new(DiseaseKind::Fever);
        // At start, severity is low
        let initial = d.effective_severity();
        // After some ticks, severity increases
        d.ticks_infected = 100;
        let mid = d.effective_severity();
        assert!(mid >= initial);
    }

    #[test]
    fn non_contagious_diseases_have_zero_transmission() {
        assert_eq!(DiseaseKind::WoundInfection.transmission_rate(), Fixed::ZERO);
        assert_eq!(DiseaseKind::Malnutrition.transmission_rate(), Fixed::ZERO);
    }

    #[test]
    fn health_decreases_with_disease() {
        let config = HealthConfig::default();
        let mut health = Fixed::from_f64(0.8);
        let mut energy = Fixed::from_f64(0.9);
        let mut diseases = vec![ActiveDisease::new(DiseaseKind::Fever)];

        // Pass high hunger/fatigue to prevent natural recovery from overriding disease
        for _ in 0..200 {
            system_health(
                &mut health,
                &mut energy,
                &mut diseases,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.8),
                &config,
            );
        }

        assert!(
            health < Fixed::from_f64(0.8),
            "Health should decrease with fever: {}",
            health.to_f64()
        );
        assert!(
            energy < Fixed::from_f64(0.9),
            "Energy should decrease with fever: {}",
            energy.to_f64()
        );
    }

    #[test]
    fn malnutrition_degrades_health() {
        let config = HealthConfig::default();
        let mut health = Fixed::from_f64(0.9);
        let mut energy = Fixed::from_f64(0.8);
        let mut diseases = Vec::new();

        // High hunger triggers malnutrition
        system_health(
            &mut health,
            &mut energy,
            &mut diseases,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            &config,
        );

        assert!(health < Fixed::from_f64(0.9));
    }

    #[test]
    fn recovery_when_healthy_and_fed() {
        let config = HealthConfig::default();
        let mut health = Fixed::from_f64(0.5);
        let mut energy = Fixed::from_f64(0.4);
        let mut diseases = Vec::new();

        system_health(
            &mut health,
            &mut energy,
            &mut diseases,
            Fixed::ZERO,
            Fixed::ZERO,
            &config,
        );

        assert!(health > Fixed::from_f64(0.5));
        assert!(energy > Fixed::from_f64(0.4));
    }

    #[test]
    fn disease_resolves_after_duration() {
        let config = HealthConfig::default();
        let mut health = Fixed::ONE;
        let mut energy = Fixed::ONE;
        let mut diseases = vec![ActiveDisease::new(DiseaseKind::Cold)];

        // Fast-forward past cold duration (200 ticks)
        for _ in 0..300 {
            system_health(
                &mut health,
                &mut energy,
                &mut diseases,
                Fixed::ZERO,
                Fixed::ZERO,
                &config,
            );
        }

        assert!(diseases.is_empty());
    }

    #[test]
    fn work_impairment_is_one_for_healthy() {
        // A healthy agent (no diseases) works at full productivity — this is
        // what keeps calibrated baseline runs free of drift.
        let factor = work_impairment(&[]);
        assert!(
            factor == Fixed::ONE,
            "healthy agent must work at full output (got {})",
            factor.to_f64()
        );
    }

    #[test]
    fn work_impairment_reduces_output_for_sick() {
        // A virulent Epidemic (severity_modifier 1.0 at peak severity 0.7)
        // costs half its severity in productivity: 1 − 0.7×0.5 = 0.65.
        let mut epidemic = ActiveDisease::new(DiseaseKind::Epidemic);
        epidemic.severity_modifier = Fixed::ONE;
        epidemic.ticks_infected = 200; // inside the severity ramp
        let factor = work_impairment(&[epidemic]);
        assert!(
            factor < Fixed::ONE,
            "a sick agent must work less (got {})",
            factor.to_f64()
        );
        assert!(factor >= Fixed::from_f64(0.2), "must respect the floor");
    }

    #[test]
    fn work_impairment_respects_floor_for_stacked_diseases() {
        // Three peak-severity epidemics (total severity 2.1) drive the
        // multiplier to the 0.2 floor rather than below zero.
        let sick = vec![
            ActiveDisease {
                kind: DiseaseKind::Epidemic,
                ticks_infected: 264, // 1/3 of the 800-tick duration = severity peak
                severity_modifier: Fixed::ONE,
            };
            3
        ];
        let factor = work_impairment(&sick);
        assert!(
            factor == Fixed::from_f64(0.2),
            "stacked diseases must floor at 0.2 (got {})",
            factor.to_f64()
        );
    }
}
