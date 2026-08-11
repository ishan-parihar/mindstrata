//! Musculoskeletal system — strength, endurance, fatigue, adaptation, and physical agency.
//!
//! Labor increases conditioning, overwork increases fatigue/injury, rest restores,
//! starvation causes atrophy, age reduces peak strength, training improves efficiency.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Muscular state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MuscularState {
    /// Current strength (0 = feeble, 1 = peak human).
    pub strength: Fixed,
    /// Endurance capacity (0 = fragile, 1 = inexhaustible).
    pub endurance: Fixed,
    /// Current fatigue (0 = fresh, 1 = exhausted).
    pub fatigue: Fixed,
    /// Muscle soreness from recent exertion (0 = none, 1 = very sore).
    pub soreness: Fixed,
    /// Atrophy from disuse or starvation (0 = none, 1 = severe).
    pub atrophy: Fixed,
    /// Conditioning level — improves with consistent moderate labor.
    pub conditioning: Fixed,
    /// Current injury level (0 = uninjured, 1 = severely injured).
    pub injury: Fixed,
}

impl Default for MuscularState {
    fn default() -> Self {
        Self {
            strength: Fixed::from_f64(0.6),
            endurance: Fixed::from_f64(0.5),
            fatigue: Fixed::ZERO,
            soreness: Fixed::ZERO,
            atrophy: Fixed::ZERO,
            conditioning: Fixed::from_f64(0.4),
            injury: Fixed::ZERO,
        }
    }
}

impl MuscularState {
    /// Update musculoskeletal state each tick.
    pub fn tick_update(
        &mut self,
        activity_level: Fixed,
        nutrition_quality: Fixed,
        rest_quality: Fixed,
        age_modifier: Fixed,
    ) {
        // Fatigue accumulates with activity
        let fatigue_gain = activity_level * Fixed::from_f64(0.015);
        self.fatigue = (self.fatigue + fatigue_gain).clamp_01();

        // Rest reduces fatigue
        let fatigue_recovery = rest_quality * Fixed::from_f64(0.02);
        self.fatigue = (self.fatigue - fatigue_recovery).max(Fixed::ZERO);

        // Soreness from high activity
        if activity_level > Fixed::from_f64(0.7) {
            let soreness_gain = (activity_level - Fixed::from_f64(0.7)) * Fixed::from_f64(0.01);
            self.soreness = (self.soreness + soreness_gain).clamp_01();
        }
        // Soreness decays with rest
        self.soreness = (self.soreness - rest_quality * Fixed::from_f64(0.008)).max(Fixed::ZERO);

        // Conditioning improves with moderate consistent activity
        if activity_level > Fixed::from_f64(0.3) && activity_level < Fixed::from_f64(0.7) {
            let gain = activity_level * nutrition_quality * Fixed::from_f64(0.0003);
            self.conditioning = (self.conditioning + gain).clamp_01();
        }

        // Strength derived from conditioning, minus fatigue, minus atrophy
        self.strength = (self.conditioning * Fixed::from_f64(0.7) + Fixed::from_f64(0.3)
            - self.fatigue * Fixed::from_f64(0.2)
            - self.atrophy * Fixed::from_f64(0.3)
            - self.injury * Fixed::from_f64(0.2))
        .clamp_01();

        // Endurance from conditioning
        self.endurance =
            (self.conditioning * Fixed::from_f64(0.6) + Fixed::from_f64(0.3)).clamp_01();

        // Atrophy from disuse or poor nutrition
        if activity_level < Fixed::from_f64(0.1) || nutrition_quality < Fixed::from_f64(0.2) {
            self.atrophy = (self.atrophy + Fixed::from_f64(0.0002)).clamp_01();
        } else if nutrition_quality > Fixed::from_f64(0.5) {
            // Recovery from atrophy with nutrition and activity
            self.atrophy = (self.atrophy - Fixed::from_f64(0.0001)).max(Fixed::ZERO);
        }

        // Age reduces peak strength
        self.strength = (self.strength - age_modifier * Fixed::from_f64(0.0001)).max(Fixed::ZERO);
    }

    /// Effective strength for tasks (accounting for fatigue and injury).
    pub fn effective_strength(&self) -> Fixed {
        self.strength
    }

    /// Whether the agent is too exhausted for physical labor.
    pub fn exhausted(&self) -> bool {
        self.fatigue > Fixed::from_f64(0.85)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn conditioning_improves_with_moderate_activity() {
        let mut m = MuscularState::default();
        let initial = m.conditioning;
        for _ in 0..200 {
            m.tick_update(
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.8),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.3),
            );
        }
        assert!(m.conditioning > initial);
    }

    #[test]
    fn rest_recovers_fatigue() {
        let mut m = MuscularState {
            fatigue: Fixed::from_f64(0.8),
            ..Default::default()
        };
        m.tick_update(Fixed::ZERO, Fixed::ONE, Fixed::ONE, Fixed::from_f64(0.3));
        assert!(m.fatigue < Fixed::from_f64(0.8));
    }

    #[test]
    fn starvation_causes_atrophy() {
        let mut m = MuscularState::default();
        for _ in 0..100 {
            m.tick_update(Fixed::ZERO, Fixed::ZERO, Fixed::ONE, Fixed::from_f64(0.3));
        }
        assert!(m.atrophy > Fixed::ZERO);
    }
}
