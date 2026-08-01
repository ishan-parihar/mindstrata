//! Status system — multi-dimensional power and social standing.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Multi-dimensional status state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusDimensions {
    pub dominance: Fixed,
    pub prestige: Fixed,
    pub authority: Fixed,
    pub legitimacy: Fixed,
    pub wealth_rank: Fixed,
    pub moral_reputation: Fixed,
    pub network_centrality: Fixed,
    pub honor: Fixed,
    pub shame: Fixed,
}

impl Default for StatusDimensions {
    fn default() -> Self {
        Self {
            dominance: Fixed::from_f64(0.3),
            prestige: Fixed::from_f64(0.3),
            authority: Fixed::ZERO,
            legitimacy: Fixed::from_f64(0.5),
            wealth_rank: Fixed::from_f64(0.3),
            moral_reputation: Fixed::from_f64(0.5),
            network_centrality: Fixed::from_f64(0.3),
            honor: Fixed::from_f64(0.5),
            shame: Fixed::ZERO,
        }
    }
}

impl StatusDimensions {
    pub fn effective_status(&self) -> Fixed {
        (self.dominance * Fixed::from_f64(0.15)
            + self.prestige * Fixed::from_f64(0.2)
            + self.authority * Fixed::from_f64(0.2)
            + self.legitimacy * Fixed::from_f64(0.15)
            + self.wealth_rank * Fixed::from_f64(0.1)
            + self.moral_reputation * Fixed::from_f64(0.1)
            + self.honor * Fixed::from_f64(0.05)
            - self.shame * Fixed::from_f64(0.1))
            .clamp_01()
    }

    pub fn record_victory(&mut self, magnitude: Fixed) {
        self.dominance = (self.dominance + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.prestige = (self.prestige + magnitude * Fixed::from_f64(0.01)).clamp_01();
        self.honor = (self.honor + magnitude * Fixed::from_f64(0.01)).clamp_01();
    }

    pub fn record_defeat(&mut self, magnitude: Fixed) {
        self.dominance = (self.dominance - magnitude * Fixed::from_f64(0.03)).max(Fixed::ZERO);
        self.shame = (self.shame + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.honor = (self.honor - magnitude * Fixed::from_f64(0.01)).max(Fixed::ZERO);
    }

    pub fn decay(&mut self) {
        self.shame = (self.shame - Fixed::from_f64(0.001)).max(Fixed::ZERO);
        self.honor = (self.honor - Fixed::from_f64(0.0005)).max(Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn effective_status_computed() {
        let s = StatusDimensions::default();
        let eff = s.effective_status();
        assert!(eff >= Fixed::ZERO && eff <= Fixed::ONE);
    }

    #[test]
    fn victory_increases_dominance() {
        let mut s = StatusDimensions::default();
        let initial = s.dominance;
        s.record_victory(Fixed::from_f64(0.5));
        assert!(s.dominance > initial);
    }
}
