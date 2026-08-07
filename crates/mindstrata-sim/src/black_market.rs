//! Black market mechanics — §4.4 of the refactor plan.
//!
//! When grain is scarce and norm enforcement is weak, agents can steal
//! without punishment, and stolen goods are sold at higher prices.
//! Black markets emerge when:
//!   - Resource scarcity exceeds a threshold
//!   - Council enforcement capacity is low
//!   - Agents have high risk tolerance and low conformity
//!
//! §13.3: "Black markets emerge when enforcement is weak."

use crate::person::Personality;
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Minimum scarcity level to trigger black market activity.
pub const BLACK_MARKET_SCARCITY_THRESHOLD: Fixed = Fixed::from_raw(4000); // 0.4

/// Maximum enforcement capacity for black market to be active.
/// When enforcement is high, black market is suppressed.
pub const BLACK_MARKET_ENFORCEMENT_THRESHOLD: Fixed = Fixed::from_raw(5000); // 0.5

/// Price multiplier for black market goods (higher than legal market).
pub const BLACK_MARKET_PRICE_MULTIPLIER: Fixed = Fixed::from_raw(15000); // 1.5

/// Risk tolerance threshold for agents to participate in black market.
pub const BLACK_MARKET_RISK_THRESHOLD: Fixed = Fixed::from_raw(6000); // 0.6

/// Conformity threshold — agents with high conformity avoid black market.
pub const BLACK_MARKET_CONFORMITY_THRESHOLD: Fixed = Fixed::from_raw(7000); // 0.7

/// Black market state tracking.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BlackMarketState {
    /// Whether the black market is currently active.
    pub active: bool,
    /// Number of black market transactions this tick.
    pub transactions_this_tick: u32,
    /// Total black market volume in recent ticks.
    pub recent_volume: Vec<Fixed>,
    /// Number of thefts detected this tick.
    pub thefts_detected: u32,
    /// Number of thefts undetected this tick.
    pub thefts_undetected: u32,
}

impl BlackMarketState {
    /// Update black market state based on current conditions.
    pub fn update(
        &mut self,
        scarcity: Fixed,
        enforcement_capacity: Fixed,
    ) {
        // Black market is active when scarcity is high and enforcement is low
        self.active = scarcity >= BLACK_MARKET_SCARCITY_THRESHOLD
            && enforcement_capacity <= BLACK_MARKET_ENFORCEMENT_THRESHOLD;

        // Track volume
        self.recent_volume.push(Fixed::from_int(self.transactions_this_tick as i64));
        if self.recent_volume.len() > 100 {
            self.recent_volume.remove(0);
        }

        // Reset per-tick counters
        self.transactions_this_tick = 0;
        self.thefts_detected = 0;
        self.thefts_undetected = 0;
    }

    /// Check if an agent can participate in the black market.
    pub fn can_participate(&self, personality: &Personality) -> bool {
        if !self.active {
            return false;
        }
        // High risk tolerance + low conformity → participates
        personality.risk_tolerance >= BLACK_MARKET_RISK_THRESHOLD
            && personality.conformity < BLACK_MARKET_CONFORMITY_THRESHOLD
    }

    /// Compute black market price for a resource.
    pub fn black_market_price(&self, legal_price: Fixed) -> Fixed {
        (legal_price * BLACK_MARKET_PRICE_MULTIPLIER).max(Fixed::from_f64(1.0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::person::Personality;

    fn make_personality(risk: f64, conformity: f64) -> Personality {
        Personality {
            openness: Fixed::from_f64(0.5),
            conscientiousness: Fixed::from_f64(0.5),
            extraversion: Fixed::from_f64(0.5),
            agreeableness: Fixed::from_f64(0.5),
            neuroticism: Fixed::from_f64(0.5),
            risk_tolerance: Fixed::from_f64(risk),
            conformity: Fixed::from_f64(conformity),
            ambition: Fixed::from_f64(0.5),
            altruism: Fixed::from_f64(0.5),
            traditionalism: Fixed::from_f64(0.5),
            dominance: Fixed::from_f64(0.5),
            impulsivity: Fixed::from_f64(0.5),
            temperament: crate::person::Temperament::default(),
        }
    }

    #[test]
    fn black_market_active_with_high_scarcity_and_low_enforcement() {
        let mut bm = BlackMarketState::default();
        bm.update(Fixed::from_f64(0.6), Fixed::from_f64(0.3));
        assert!(bm.active, "Black market should be active with high scarcity and low enforcement");
    }

    #[test]
    fn black_market_inactive_with_low_scarcity() {
        let mut bm = BlackMarketState::default();
        bm.update(Fixed::from_f64(0.2), Fixed::from_f64(0.3));
        assert!(!bm.active, "Black market should be inactive with low scarcity");
    }

    #[test]
    fn black_market_inactive_with_high_enforcement() {
        let mut bm = BlackMarketState::default();
        bm.update(Fixed::from_f64(0.6), Fixed::from_f64(0.8));
        assert!(!bm.active, "Black market should be inactive with high enforcement");
    }

    #[test]
    fn high_risk_low_conformity_can_participate() {
        let bm = BlackMarketState { active: true, ..Default::default() };
        let agent = make_personality(0.8, 0.3);
        assert!(bm.can_participate(&agent), "High risk + low conformity should participate");
    }

    #[test]
    fn low_risk_cannot_participate() {
        let bm = BlackMarketState { active: true, ..Default::default() };
        let agent = make_personality(0.2, 0.3);
        assert!(!bm.can_participate(&agent), "Low risk tolerance should not participate");
    }

    #[test]
    fn high_conformity_cannot_participate() {
        let bm = BlackMarketState { active: true, ..Default::default() };
        let agent = make_personality(0.8, 0.9);
        assert!(!bm.can_participate(&agent), "High conformity should not participate");
    }

    #[test]
    fn black_market_price_is_higher_than_legal() {
        let bm = BlackMarketState::default();
        let legal_price = Fixed::from_f64(5.0);
        let bm_price = bm.black_market_price(legal_price);
        assert!(bm_price > legal_price, "Black market price should be higher than legal price");
    }
}
