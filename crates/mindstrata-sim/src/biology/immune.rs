//! Immune system — disease resistance, inflammation, infection, and recovery.
//!
//! Interacts with nutrition, stress, sleep, age, injury, hygiene, and crowding.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Immune state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImmuneState {
    /// Baseline resistance (0 = immunocompromised, 1 = robust).
    pub resistance: Fixed,
    /// Current inflammation level (0 = none, 1 = systemic).
    pub inflammation: Fixed,
    /// Active infection load (0 = healthy, 1 = severely infected).
    pub infection_load: Fixed,
    /// Recovery capacity — how fast the body fights infection.
    pub recovery_capacity: Fixed,
    /// Autoimmune risk — chronic inflammation from overactive immunity.
    pub autoimmune_risk: Fixed,
}

impl Default for ImmuneState {
    fn default() -> Self {
        Self {
            resistance: Fixed::from_f64(0.6),
            inflammation: Fixed::ZERO,
            infection_load: Fixed::ZERO,
            recovery_capacity: Fixed::from_f64(0.5),
            autoimmune_risk: Fixed::ZERO,
        }
    }
}

impl ImmuneState {
    /// Update immune state each tick.
    pub fn tick_update(
        &mut self,
        nutrition_quality: Fixed,
        stress_level: Fixed,
        sleep_quality: Fixed,
        injury_severity: Fixed,
        crowding: Fixed,
        hygiene: Fixed,
        age_modifier: Fixed,
    ) {
        // Nutrition supports immune function
        let nutrition_boost = nutrition_quality * Fixed::from_f64(0.002);
        self.resistance = (self.resistance + nutrition_boost).clamp_01();

        // Stress suppresses immune function
        let stress_suppression = stress_level * Fixed::from_f64(0.001);
        self.resistance = (self.resistance - stress_suppression).max(Fixed::ZERO);

        // Sleep is critical for immune recovery
        let sleep_boost = sleep_quality * Fixed::from_f64(0.001);
        self.resistance = (self.resistance + sleep_boost).clamp_01();

        // Age reduces immune function
        self.resistance = (self.resistance - age_modifier * Fixed::from_f64(0.0001)).max(Fixed::ZERO);

        // Infection triggers immune response
        if self.infection_load > Fixed::ZERO {
            // Inflammation rises to fight infection
            self.inflammation = (self.infection_load * Fixed::from_f64(0.6)).clamp_01();
            // Immune system fights infection
            let fight = self.resistance * self.recovery_capacity * Fixed::from_f64(0.005);
            self.infection_load = (self.infection_load - fight).max(Fixed::ZERO);
            // Chronic inflammation damages resistance
            if self.inflammation > Fixed::from_f64(0.5) {
                self.autoimmune_risk = (self.autoimmune_risk + Fixed::from_f64(0.0001)).clamp_01();
            }
        } else {
            // No infection → inflammation decays
            self.inflammation = (self.inflammation - Fixed::from_f64(0.01)).max(Fixed::ZERO);
        }

        // Crowding increases infection risk
        if crowding > Fixed::from_f64(0.5) && hygiene < Fixed::from_f64(0.3) {
            let exposure = (crowding - Fixed::from_f64(0.5)) * (Fixed::ONE - hygiene) * Fixed::from_f64(0.001);
            self.infection_load = (self.infection_load + exposure).clamp_01();
        }

        // Injury increases infection vulnerability
        if injury_severity > Fixed::from_f64(0.5) {
            let wound_infection = (injury_severity - Fixed::from_f64(0.5)) * Fixed::from_f64(0.002);
            self.infection_load = (self.infection_load + wound_infection).clamp_01();
        }

        // Recovery capacity derived from resistance and nutrition
        self.recovery_capacity = (self.resistance * Fixed::from_f64(0.6)
            + nutrition_quality * Fixed::from_f64(0.4))
            .clamp_01();
    }

    /// Whether the agent is currently sick.
    pub fn is_sick(&self) -> bool {
        self.infection_load > Fixed::from_f64(0.2) || self.inflammation > Fixed::from_f64(0.4)
    }

    /// Sickness severity for derived body state (0–1).
    pub fn sickness_level(&self) -> Fixed {
        (self.infection_load * Fixed::from_f64(0.6) + self.inflammation * Fixed::from_f64(0.4))
            .clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nutrition_boosts_resistance() {
        let mut immune = ImmuneState::default();
        let initial = immune.resistance;
        for _ in 0..50 {
            immune.tick_update(
                Fixed::from_f64(0.9),
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.3),
            );
        }
        assert!(immune.resistance > initial);
    }

    #[test]
    fn infection_triggers_inflammation() {
        let mut immune = ImmuneState::default();
        immune.infection_load = Fixed::from_f64(0.5);
        immune.tick_update(
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::from_f64(0.3),
        );
        assert!(immune.inflammation > Fixed::ZERO);
    }

    #[test]
    fn stress_suppresses_immunity() {
        let mut immune = ImmuneState::default();
        immune.resistance = Fixed::from_f64(0.7);
        for _ in 0..100 {
            immune.tick_update(
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.9),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ONE,
                Fixed::from_f64(0.3),
            );
        }
        assert!(immune.resistance < Fixed::from_f64(0.7));
    }
}
