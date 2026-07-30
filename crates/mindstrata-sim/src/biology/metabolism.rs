//! Metabolic system — energy processing, hunger, satiety, thermoregulation.
//!
//! Converts food intake into energy, manages fat reserves, and produces
//! thermoregulatory signals. Interacts with the endocrine metabolic axis.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Core metabolic state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetabolicState {
    /// Current energy reserves (0 = depleted, 1 = fully charged).
    pub energy_reserves: Fixed,
    /// Fat storage level (0 = lean, 1 = obese). Affects energy buffer.
    pub fat_reserves: Fixed,
    /// Baseline metabolic rate — how fast energy is consumed at rest.
    pub basal_rate: Fixed,
    /// Current metabolic demand (increases with activity, cold, illness).
    pub demand: Fixed,
    /// Satiety signal (0 = starving, 1 = completely full).
    pub satiety: Fixed,
    /// Body temperature analog (0.45 = hypothermic, 0.5 = normal, 0.55 = febrile).
    pub body_temperature: Fixed,
    /// Cold stress (0 = comfortable, 1 = dangerously cold).
    pub cold_stress: Fixed,
    /// Heat stress (0 = comfortable, 1 = dangerously hot).
    pub heat_stress: Fixed,
    /// Hydration level (0 = severely dehydrated, 1 = fully hydrated).
    pub hydration: Fixed,
}

impl Default for MetabolicState {
    fn default() -> Self {
        Self {
            energy_reserves: Fixed::from_f64(0.7),
            fat_reserves: Fixed::from_f64(0.3),
            basal_rate: Fixed::from_f64(0.01),
            demand: Fixed::from_f64(0.01),
            satiety: Fixed::from_f64(0.5),
            body_temperature: Fixed::from_f64(0.5),
            cold_stress: Fixed::ZERO,
            heat_stress: Fixed::ZERO,
            hydration: Fixed::from_f64(0.8),
        }
    }
}

impl MetabolicState {
    /// Consume energy each tick. Returns energy actually expended.
    pub fn tick_update(&mut self, activity_level: Fixed, ambient_temperature: Fixed) {
        // Total demand = basal + activity-driven
        let activity_demand = activity_level * Fixed::from_f64(0.02);
        self.demand = self.basal_rate + activity_demand;

        // Energy reserves deplete based on demand
        let fat_contribution = if self.energy_reserves < Fixed::from_f64(0.3) {
            // When energy is low, fat reserves contribute (slowly)
            self.fat_reserves * Fixed::from_f64(0.001)
        } else {
            Fixed::ZERO
        };
        self.energy_reserves = (self.energy_reserves - self.demand + fat_contribution).clamp_01();

        // Fat reserves slowly convert to energy when starving
        if self.energy_reserves < Fixed::from_f64(0.2) && self.fat_reserves > Fixed::ZERO {
            let conversion = (Fixed::from_f64(0.2) - self.energy_reserves) * Fixed::from_f64(0.005);
            let converted = conversion.min(self.fat_reserves);
            self.energy_reserves = (self.energy_reserves + converted).clamp_01();
            self.fat_reserves = (self.fat_reserves - converted).max(Fixed::ZERO);
        }

        // Satiety decays over time
        self.satiety = (self.satiety - Fixed::from_f64(0.003)).clamp_01();

        // Thermoregulation
        let temp_delta = ambient_temperature - self.body_temperature;
        self.cold_stress = if temp_delta < -Fixed::from_f64(0.05) {
            (-temp_delta - Fixed::from_f64(0.05)).clamp_01()
        } else {
            Fixed::ZERO
        };
        self.heat_stress = if temp_delta > Fixed::from_f64(0.05) {
            (temp_delta - Fixed::from_f64(0.05)).clamp_01()
        } else {
            Fixed::ZERO
        };
        // Body temperature drifts toward ambient (slowed by metabolic regulation)
        let metabolic_warmth = self.energy_reserves * Fixed::from_f64(0.01);
        self.body_temperature = (self.body_temperature + temp_delta * Fixed::from_f64(0.001)
            - metabolic_warmth * self.cold_stress * Fixed::from_f64(0.001))
            .clamp_01();

        // Hydration depletes slowly
        self.hydration = (self.hydration - Fixed::from_f64(0.002)).clamp_01();
    }

    /// Process food intake.
    pub fn consume_food(&mut self, quality: Fixed) {
        // Better quality food gives more satiety and energy
        let energy_gain = quality * Fixed::from_f64(0.15);
        let satiety_gain = quality * Fixed::from_f64(0.2);
        let fat_gain = (Fixed::ONE - quality) * Fixed::from_f64(0.03); // poor food → more fat
        self.energy_reserves = (self.energy_reserves + energy_gain).clamp_01();
        self.satiety = (self.satiety + satiety_gain).clamp_01();
        self.fat_reserves = (self.fat_reserves + fat_gain).clamp_01();
    }

    /// Process water intake.
    pub fn consume_water(&mut self, amount: Fixed) {
        self.hydration = (self.hydration + amount).clamp_01();
    }

    /// Energy available for physical activity (0–1).
    pub fn available_energy(&self) -> Fixed {
        (self.energy_reserves * Fixed::from_f64(0.7) + self.fat_reserves * Fixed::from_f64(0.3))
            .clamp_01()
    }

    /// Whether the agent is in metabolic crisis (starving or dehydrated).
    pub fn in_crisis(&self) -> bool {
        self.energy_reserves < Fixed::from_f64(0.15) || self.hydration < Fixed::from_f64(0.15)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn energy_depletes_with_activity() {
        let mut m = MetabolicState::default();
        let initial = m.energy_reserves;
        m.tick_update(Fixed::from_f64(0.8), Fixed::from_f64(0.5));
        assert!(m.energy_reserves < initial);
    }

    #[test]
    fn food_increases_energy_and_satiety() {
        let mut m = MetabolicState::default();
        let initial_energy = m.energy_reserves;
        let initial_satiety = m.satiety;
        m.consume_food(Fixed::from_f64(0.8));
        assert!(m.energy_reserves > initial_energy);
        assert!(m.satiety > initial_satiety);
    }

    #[test]
    fn dehydration_detected() {
        let mut m = MetabolicState::default();
        m.hydration = Fixed::from_f64(0.1);
        assert!(m.in_crisis());
    }

    #[test]
    fn cold_stress_from_low_temperature() {
        let mut m = MetabolicState::default();
        m.tick_update(Fixed::ZERO, Fixed::from_f64(0.1)); // very cold
        assert!(m.cold_stress > Fixed::ZERO);
    }
}
