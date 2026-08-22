//! Biological body state and need deficits.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Biological / body state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BodyState {
    /// Health. (doc added at S2 extraction)
    pub health: Fixed,
    /// Energy. (doc added at S2 extraction)
    pub energy: Fixed,
    /// Hunger. (doc added at S2 extraction)
    pub hunger: Fixed,
    /// Thirst. (doc added at S2 extraction)
    pub thirst: Fixed,
    /// Fatigue. (doc added at S2 extraction)
    pub fatigue: Fixed,
    /// Sickness. (doc added at S2 extraction)
    pub sickness: Fixed,
    /// Injury. (doc added at S2 extraction)
    pub injury: Fixed,
    /// Fertility. (doc added at S2 extraction)
    pub fertility: Option<Fixed>,
}

impl Default for BodyState {
    fn default() -> Self {
        Self {
            health: Fixed::ONE,
            energy: Fixed::ONE,
            hunger: Fixed::ZERO,
            thirst: Fixed::ZERO,
            fatigue: Fixed::ZERO,
            sickness: Fixed::ZERO,
            injury: Fixed::ZERO,
            fertility: None,
        }
    }
}

/// Need deficit values (0 = fully satisfied, 1 = critical).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NeedState {
    /// Hunger. (doc added at S2 extraction)
    pub hunger: Fixed,
    /// Thirst. (doc added at S2 extraction)
    pub thirst: Fixed,
    /// Fatigue. (doc added at S2 extraction)
    pub fatigue: Fixed,
    /// Safety. (doc added at S2 extraction)
    pub safety: Fixed,
    /// Social. (doc added at S2 extraction)
    pub social: Fixed,
    /// Esteem. (doc added at S2 extraction)
    pub esteem: Fixed,
    /// Autonomy. (doc added at S2 extraction)
    pub autonomy: Fixed,
    /// Meaning. (doc added at S2 extraction)
    pub meaning: Fixed,
}

impl Default for NeedState {
    fn default() -> Self {
        Self {
            hunger: Fixed::from_f64(0.2),
            thirst: Fixed::from_f64(0.1),
            fatigue: Fixed::from_f64(0.1),
            safety: Fixed::from_f64(0.1),
            social: Fixed::from_f64(0.3),
            esteem: Fixed::from_f64(0.2),
            autonomy: Fixed::from_f64(0.1),
            meaning: Fixed::from_f64(0.1),
        }
    }
}
