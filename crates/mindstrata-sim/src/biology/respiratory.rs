//! Respiratory system — exertion limits, environmental health, and disease vulnerability.
//!
//! Represents lung capacity, oxygenation, and vulnerability to respiratory disease.
//! Interacts with exertion, cold, smoke, damp housing, and epidemics.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Respiratory state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RespiratoryState {
    /// Oxygenation level (0 = hypoxic, 1 = fully oxygenated).
    pub oxygenation: Fixed,
    /// Breathlessness (0 = none, 1 = severely limited).
    pub breathlessness: Fixed,
    /// Lung health baseline (0 = damaged, 1 = healthy).
    pub lung_health: Fixed,
    /// Environmental irritation (smoke, cold, damp).
    pub irritation: Fixed,
    /// Current disease load on respiratory system.
    pub disease_load: Fixed,
    /// Endurance modifier from respiratory capacity.
    pub endurance_modifier: Fixed,
}

impl Default for RespiratoryState {
    fn default() -> Self {
        Self {
            oxygenation: Fixed::from_f64(0.9),
            breathlessness: Fixed::ZERO,
            lung_health: Fixed::from_f64(0.8),
            irritation: Fixed::ZERO,
            disease_load: Fixed::ZERO,
            endurance_modifier: Fixed::from_f64(0.8),
        }
    }
}

impl RespiratoryState {
    /// Update respiratory state each tick.
    pub fn tick_update(
        &mut self,
        exertion: Fixed,
        cold_exposure: Fixed,
        smoke_exposure: Fixed,
        damp_housing: Fixed,
        age_modifier: Fixed,
    ) {
        // Exertion increases breathlessness temporarily
        let exertion_effect = exertion * Fixed::from_f64(0.3);
        self.breathlessness = (self.breathlessness * Fixed::from_f64(0.9)
            + exertion_effect * Fixed::from_f64(0.1))
        .clamp_01();

        // Oxygenation inversely related to breathlessness and disease
        self.oxygenation = (Fixed::from_f64(1.0)
            - self.breathlessness * Fixed::from_f64(0.4)
            - self.disease_load * Fixed::from_f64(0.3)
            - self.irritation * Fixed::from_f64(0.1))
        .clamp_01();

        // Environmental irritation accumulates
        self.irritation = (smoke_exposure * Fixed::from_f64(0.15)
            + cold_exposure * Fixed::from_f64(0.05)
            + damp_housing * Fixed::from_f64(0.08))
        .clamp_01();

        // Lung health degrades slowly with chronic irritation and age
        if self.irritation > Fixed::from_f64(0.3) {
            self.lung_health = (self.lung_health - Fixed::from_f64(0.0001)).max(Fixed::ZERO);
        }
        self.lung_health =
            (self.lung_health - age_modifier * Fixed::from_f64(0.00005)).max(Fixed::ZERO);

        // Disease load decays with good health
        if self.lung_health > Fixed::from_f64(0.5) {
            self.disease_load = (self.disease_load - Fixed::from_f64(0.001)).max(Fixed::ZERO);
        }

        // Endurance modifier derived from lung health and oxygenation
        self.endurance_modifier = (self.lung_health * Fixed::from_f64(0.5)
            + self.oxygenation * Fixed::from_f64(0.5))
        .clamp_01();
    }

    /// Effective respiratory capacity for physical tasks.
    pub fn effective_capacity(&self) -> Fixed {
        self.endurance_modifier
    }

    /// Whether the agent has a respiratory illness.
    pub fn is_ill(&self) -> bool {
        self.disease_load > Fixed::from_f64(0.3)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn exertion_increases_breathlessness() {
        let mut r = RespiratoryState::default();
        r.tick_update(
            Fixed::from_f64(0.9),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.3),
        );
        assert!(r.breathlessness > Fixed::ZERO);
    }

    #[test]
    fn smoke_sets_irritation() {
        let mut r = RespiratoryState::default();
        // smoke_exposure * 0.15 = 0.12
        r.tick_update(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::from_f64(0.3),
        );
        assert!(r.irritation > Fixed::from_f64(0.1));
    }

    #[test]
    fn high_smoke_degrades_lung_health() {
        let mut r = RespiratoryState::default();
        let initial = r.lung_health;
        // irritation = 3.0 * 0.15 = 0.45 > 0.3 threshold
        for _ in 0..100 {
            r.tick_update(
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(3.0),
                Fixed::ZERO,
                Fixed::from_f64(0.3),
            );
        }
        assert!(r.lung_health < initial);
    }
}
