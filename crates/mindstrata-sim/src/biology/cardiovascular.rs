//! Cardiovascular system — stamina, shock, blood loss, fitness, and acute mortality risk.
//!
//! Fitness improves with healthy labor, overexertion under starvation causes collapse,
//! combat injury causes blood loss, chronic stress raises long-term risk, aging reduces capacity.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Cardiovascular state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CardiovascularState {
    /// Cardiac capacity (0 = failing, 1 = peak). Declines with age and illness.
    pub cardiac_capacity: Fixed,
    /// Blood volume (0.5 = hemorrhaging, 1.0 = normal).
    pub blood_volume: Fixed,
    /// Fitness level (0 = unfit, 1 = elite). Improves with healthy labor.
    pub fitness: Fixed,
    /// Blood pressure analog (0 = dangerously low, 0.5 = normal, 1 = hypertensive).
    pub blood_pressure: Fixed,
    /// Recovery rate — how fast the cardiovascular system recovers from stress.
    pub recovery_rate: Fixed,
    /// Shock risk (0 = none, 1 = imminent collapse).
    pub shock_risk: Fixed,
}

impl Default for CardiovascularState {
    fn default() -> Self {
        Self {
            cardiac_capacity: Fixed::from_f64(0.7),
            blood_volume: Fixed::ONE,
            fitness: Fixed::from_f64(0.5),
            blood_pressure: Fixed::from_f64(0.5),
            recovery_rate: Fixed::from_f64(0.5),
            shock_risk: Fixed::ZERO,
        }
    }
}

impl CardiovascularState {
    /// Update cardiovascular state each tick.
    pub fn tick_update(
        &mut self,
        activity_level: Fixed,
        injury_severity: Fixed,
        stress_level: Fixed,
        nutrition_quality: Fixed,
        age_modifier: Fixed,
    ) {
        // Blood loss from injury
        if injury_severity > Fixed::from_f64(0.3) {
            let blood_loss = injury_severity * Fixed::from_f64(0.005);
            self.blood_volume = (self.blood_volume - blood_loss).max(Fixed::from_f64(0.3));
        }
        // Blood volume recovers slowly with good nutrition
        if injury_severity < Fixed::from_f64(0.2) {
            let recovery = nutrition_quality * self.recovery_rate * Fixed::from_f64(0.002);
            self.blood_volume = (self.blood_volume + recovery).min(Fixed::ONE);
        }

        // Fitness improves with moderate activity + good nutrition
        if activity_level > Fixed::from_f64(0.3) && activity_level < Fixed::from_f64(0.8) {
            let fitness_gain = activity_level * nutrition_quality * Fixed::from_f64(0.0005);
            self.fitness = (self.fitness + fitness_gain).clamp_01();
        }
        // Overexertion reduces fitness
        if activity_level > Fixed::from_f64(0.85) {
            self.fitness = (self.fitness - Fixed::from_f64(0.0002)).max(Fixed::ZERO);
        }

        // Cardiac capacity declines with age and chronic stress
        self.cardiac_capacity = (Fixed::from_f64(0.8)
            - age_modifier * Fixed::from_f64(0.001)
            - stress_level * Fixed::from_f64(0.0003))
            .clamp_01();

        // Blood pressure responds to stress
        self.blood_pressure = (Fixed::from_f64(0.5) + stress_level * Fixed::from_f64(0.2)
            - self.fitness * Fixed::from_f64(0.1))
            .clamp_01();

        // Shock risk from severe blood loss + stress
        self.shock_risk = if self.blood_volume < Fixed::from_f64(0.6) {
            ((Fixed::from_f64(0.6) - self.blood_volume) * Fixed::from_f64(2.0)
                + stress_level * Fixed::from_f64(0.3))
                .clamp_01()
        } else {
            Fixed::ZERO
        };

        // Recovery rate influenced by fitness
        self.recovery_rate = (self.fitness * Fixed::from_f64(0.6) + Fixed::from_f64(0.2)).clamp_01();
    }

    /// Effective stamina for physical tasks (0–1).
    pub fn effective_stamina(&self) -> Fixed {
        (self.fitness * Fixed::from_f64(0.4)
            + self.blood_volume * Fixed::from_f64(0.3)
            + self.cardiac_capacity * Fixed::from_f64(0.3))
            .clamp_01()
    }

    /// Whether the agent is at risk of collapse.
    pub fn collapse_risk(&self) -> bool {
        self.shock_risk > Fixed::from_f64(0.7) || self.blood_volume < Fixed::from_f64(0.4)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fitness_improves_with_moderate_activity() {
        let mut cv = CardiovascularState::default();
        let initial = cv.fitness;
        for _ in 0..100 {
            cv.tick_update(
                Fixed::from_f64(0.5),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.3),
            );
        }
        assert!(cv.fitness > initial);
    }

    #[test]
    fn injury_causes_blood_loss() {
        let mut cv = CardiovascularState::default();
        cv.tick_update(
            Fixed::ZERO,
            Fixed::from_f64(0.8),
            Fixed::ZERO,
            Fixed::ONE,
            Fixed::from_f64(0.3),
        );
        assert!(cv.blood_volume < Fixed::ONE);
    }

    #[test]
    fn shock_risk_from_severe_blood_loss() {
        let mut cv = CardiovascularState {
            blood_volume: Fixed::from_f64(0.4),
            ..Default::default()
        };
        cv.tick_update(
            Fixed::ZERO,
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.7),
            Fixed::ONE,
            Fixed::from_f64(0.3),
        );
        assert!(cv.shock_risk > Fixed::ZERO);
    }
}
