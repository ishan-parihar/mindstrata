//! Attraction system — multi-factor mate selection model.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Attraction model for evaluating potential partners.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AttractionModel {
    pub physical_attraction: Fixed,
    pub personality_attraction: Fixed,
    pub status_attraction: Fixed,
    pub moral_attraction: Fixed,
    pub familiarity: Fixed,
    pub reciprocity: Fixed,
    pub social_approval: Fixed,
    pub kinship_penalty: Fixed,
    pub moral_disgust: Fixed,
    pub social_cost: Fixed,
    pub attachment_resonance: Fixed,
}

impl Default for AttractionModel {
    fn default() -> Self {
        Self {
            physical_attraction: Fixed::from_f64(0.5),
            personality_attraction: Fixed::from_f64(0.5),
            status_attraction: Fixed::ZERO,
            moral_attraction: Fixed::from_f64(0.5),
            familiarity: Fixed::ZERO,
            reciprocity: Fixed::ZERO,
            social_approval: Fixed::from_f64(0.5),
            kinship_penalty: Fixed::ZERO,
            moral_disgust: Fixed::ZERO,
            social_cost: Fixed::ZERO,
            attachment_resonance: Fixed::from_f64(0.5),
        }
    }
}

impl AttractionModel {
    pub fn total_attraction(&self) -> Fixed {
        let positive = self.physical_attraction * Fixed::from_f64(0.15)
            + self.personality_attraction * Fixed::from_f64(0.2)
            + self.status_attraction * Fixed::from_f64(0.1)
            + self.moral_attraction * Fixed::from_f64(0.1)
            + self.familiarity * Fixed::from_f64(0.1)
            + self.reciprocity * Fixed::from_f64(0.15)
            + self.social_approval * Fixed::from_f64(0.05)
            + self.attachment_resonance * Fixed::from_f64(0.1);
        let negative = self.kinship_penalty * Fixed::from_f64(0.3)
            + self.moral_disgust * Fixed::from_f64(0.2)
            + self.social_cost * Fixed::from_f64(0.1);
        (positive - negative).clamp_01()
    }

    pub fn update_familiarity(&mut self, interaction_quality: Fixed) {
        self.familiarity = (self.familiarity + interaction_quality * Fixed::from_f64(0.01))
            .min(Fixed::from_f64(0.8));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_attraction_computed() {
        let a = AttractionModel {
            physical_attraction: Fixed::from_f64(0.8),
            personality_attraction: Fixed::from_f64(0.7),
            ..AttractionModel::default()
        };
        assert!(a.total_attraction() > Fixed::from_f64(0.3));
    }

    #[test]
    fn kinship_penalty_reduces_attraction() {
        let mut a = AttractionModel { physical_attraction: Fixed::from_f64(0.8), ..AttractionModel::default() };
        let without = a.total_attraction();
        a.kinship_penalty = Fixed::from_f64(0.9);
        assert!(a.total_attraction() < without);
    }
}
