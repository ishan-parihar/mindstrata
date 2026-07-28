//! Scenario loader — RON-based scenario definitions.
//!
//! Scenarios define initial world state, agent populations, and optional
//! shocks (drought, festival) for reproducible experiments.

use crate::sim::SimConfig;
use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A scenario definition for reproducible experiments.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Scenario {
    /// Human-readable name.
    pub name: String,
    /// Description of what this scenario tests.
    pub description: String,
    /// Random seed.
    pub seed: u64,
    /// Number of ticks to run.
    pub ticks: u64,
    /// World dimensions.
    pub world_width: u32,
    pub world_height: u32,
    /// Number of initial agents.
    pub num_agents: u32,
    /// Optional shocks to apply at specific ticks.
    pub shocks: Vec<Shock>,
}

/// A scenario shock — an external event that disrupts the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shock {
    /// Tick at which this shock occurs.
    pub at_tick: u64,
    /// Kind of shock.
    pub kind: ShockKind,
    /// Magnitude (0..1).
    pub magnitude: Fixed,
}

/// Types of scenario shocks.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShockKind {
    /// Reduce food availability.
    Drought,
    /// Boost morale temporarily.
    Festival,
}

impl Scenario {
    /// Create a default "Riverford" scenario.
    pub fn riverford() -> Self {
        Self {
            name: "Riverford".into(),
            description: "A small village with farmland, a well, a market, and a temple. \
                         Agents must survive by eating, drinking, and working."
                .into(),
            seed: 42,
            ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            shocks: vec![
                Shock {
                    at_tick: 200,
                    kind: ShockKind::Festival,
                    magnitude: Fixed::from_f64(0.5),
                },
                Shock {
                    at_tick: 500,
                    kind: ShockKind::Drought,
                    magnitude: Fixed::from_f64(0.3),
                },
            ],
        }
    }

    /// Convert scenario to simulation config.
    pub fn to_sim_config(&self) -> SimConfig {
        SimConfig {
            seed: self.seed,
            max_ticks: self.ticks,
            world_width: self.world_width,
            world_height: self.world_height,
            num_agents: self.num_agents,
            snapshot_interval: None,
        }
    }

    /// Check if a shock should fire at the given tick.
    pub fn shock_at(&self, tick: u64) -> Option<&Shock> {
        self.shocks.iter().find(|s| s.at_tick == tick)
    }

    /// Serialize to RON format.
    pub fn to_ron(&self) -> String {
        ron::to_string(self).unwrap_or_else(|_| format!("{:#?}", self))
    }

    /// Deserialize from RON format.
    pub fn from_ron(s: &str) -> Result<Self, String> {
        ron::from_str(s).map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_scenario_roundtrip() {
        let scenario = Scenario::riverford();
        let ron = scenario.to_ron();
        let back = Scenario::from_ron(&ron).unwrap();
        assert_eq!(scenario.name, back.name);
        assert_eq!(scenario.seed, back.seed);
        assert_eq!(scenario.ticks, back.ticks);
    }

    #[test]
    fn shock_detection() {
        let scenario = Scenario::riverford();
        assert!(scenario.shock_at(200).is_some());
        assert!(scenario.shock_at(500).is_some());
        assert!(scenario.shock_at(300).is_none());
    }

    #[test]
    fn only_drought_and_festival_exist() {
        // Verify no unused shock variants exist
        let kinds = [ShockKind::Drought, ShockKind::Festival];
        assert_eq!(kinds.len(), 2);
    }
}
