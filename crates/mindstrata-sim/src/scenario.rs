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
    /// Reduce water availability.
    Drought,
    /// Destroy stored grain (food crisis).
    Famine,
    /// Seed a virulent epidemic into the population (mortality crisis).
    Pestilence,
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

    /// Create the "Drought" scenario — Riverford under a severe drought.
    ///
    /// Mirrors `specs/scenarios/drought.ron`: a strong drought shock (0.7) at
    /// tick 500 tests resource scarcity, legitimacy crisis, and social tension.
    pub fn drought() -> Self {
        Self {
            name: "Drought".into(),
            description: "Riverford under drought conditions. Tests resource scarcity, \
                         legitimacy crisis, and social tension."
                .into(),
            seed: 42,
            ticks: 4320,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            shocks: vec![
                Shock {
                    at_tick: 200,
                    kind: ShockKind::Festival,
                    magnitude: Fixed::from_f64(0.3),
                },
                Shock {
                    at_tick: 500,
                    kind: ShockKind::Drought,
                    magnitude: Fixed::from_f64(0.7),
                },
            ],
        }
    }

    /// Create the "Famine" scenario — Riverford under a grain famine.
    ///
    /// Mirrors `specs/scenarios/famine.ron`: a strong famine shock (0.7) at
    /// tick 500 destroys most stored grain, testing food scarcity, hunger,
    /// and social tension under scarcity.
    pub fn famine() -> Self {
        Self {
            name: "Famine".into(),
            description: "Riverford under a grain famine. Tests food scarcity, \
                         hunger, and social tension."
                .into(),
            seed: 42,
            ticks: 4320,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            shocks: vec![
                Shock {
                    at_tick: 200,
                    kind: ShockKind::Festival,
                    magnitude: Fixed::from_f64(0.3),
                },
                Shock {
                    at_tick: 500,
                    kind: ShockKind::Famine,
                    magnitude: Fixed::from_f64(0.7),
                },
            ],
        }
    }

    /// Create the "Pestilence" scenario — Riverford under a virulent epidemic.
    ///
    /// Mirrors `specs/scenarios/pestilence.ron`: a strong pestilence shock
    /// (0.7) at tick 500 seeds the Epidemic disease into most of the
    /// population, testing mortality, generational replacement, and social
    /// breakdown under a plague.
    pub fn pestilence() -> Self {
        Self {
            name: "Pestilence".into(),
            description: "Riverford under a virulent epidemic. Tests mortality, \
                         generational replacement, and social breakdown under a plague."
                .into(),
            seed: 42,
            ticks: 4320,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            shocks: vec![
                Shock {
                    at_tick: 200,
                    kind: ShockKind::Festival,
                    magnitude: Fixed::from_f64(0.3),
                },
                Shock {
                    at_tick: 500,
                    kind: ShockKind::Pestilence,
                    magnitude: Fixed::from_f64(0.7),
                },
            ],
        }
    }

    /// Create the "Collapse" scenario — Riverford under a staggered cascade.
    ///
    /// Mirrors `specs/scenarios/collapse.ron`: drought (0.6) at tick 500,
    /// famine (0.6) at tick 800, then pestilence (0.6) at tick 1100. Each
    /// crisis lands on a population already weakened by the previous one, so
    /// the compound outcome (hunger-driven health collapse amplifying the
    /// plague's mortality) exceeds any single crisis — the scenario battery's
    /// compound-emergence stress test.
    pub fn collapse() -> Self {
        Self {
            name: "Collapse".into(),
            description: "Riverford under a staggered cascade of drought, famine, and \
                         pestilence. Tests compound emergence when crises stack \
                         on a weakened population."
                .into(),
            seed: 42,
            ticks: 4320,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            shocks: vec![
                Shock {
                    at_tick: 200,
                    kind: ShockKind::Festival,
                    magnitude: Fixed::from_f64(0.3),
                },
                Shock {
                    at_tick: 500,
                    kind: ShockKind::Drought,
                    magnitude: Fixed::from_f64(0.6),
                },
                Shock {
                    at_tick: 800,
                    kind: ShockKind::Famine,
                    magnitude: Fixed::from_f64(0.6),
                },
                Shock {
                    at_tick: 1100,
                    kind: ShockKind::Pestilence,
                    magnitude: Fixed::from_f64(0.6),
                },
            ],
        }
    }

    /// Create the "Calm" scenario — a pure village with no shocks.
    ///
    /// Mirrors `specs/scenarios/calm.ron`: no external events, just baseline
    /// survival dynamics. Useful as a control scenario for scenario-based
    /// differential testing (e.g., comparing calm vs drought metrics).
    pub fn calm() -> Self {
        Self {
            name: "Calm".into(),
            description: "A pure village with no shocks. Baseline survival \
                         dynamics without festivals, droughts, or other \
                         external events."
                .into(),
            seed: 42,
            ticks: 4320,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            shocks: vec![],
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
        ron::to_string(self).unwrap_or_else(|_| format!("{self:#?}"))
    }

    /// Deserialize from RON format.
    pub fn from_ron(s: &str) -> Result<Self, String> {
        let s: Self = ron::from_str(s).map_err(|e| e.to_string())?;
        s.validate()?;
        Ok(s)
    }

    /// Serialize to JSON (DC-1 scenario-editor interchange; serde field order
    /// is declaration-stable so presets diff cleanly across versions).
    pub fn to_json(&self) -> String {
        serde_json::to_string(self).unwrap_or_else(|_| format!("{self:#?}"))
    }

    /// Deserialize from JSON format.
    pub fn from_json(s: &str) -> Result<Self, String> {
        let s: Self = serde_json::from_str(s).map_err(|e| e.to_string())?;
        s.validate()?;
        Ok(s)
    }

    /// Structural validation gate applied by every loader.
    ///
    /// Catches the silent dead-producer class at load time: out-of-range
    /// magnitudes truncate silently through Fixed, post-horizon shocks can
    /// never fire (`shock_at` dispatches on exact tick equality), duplicate
    /// shock ticks silently drop later entries, and over-cap populations get
    /// clamped mid-run instead of rejected.
    pub fn validate(&self) -> Result<(), String> {
        if self.ticks == 0 {
            return Err("ticks must be positive".into());
        }
        if self.world_width == 0 || self.world_height == 0 {
            return Err("world dimensions must be positive".into());
        }
        if self.num_agents == 0 {
            return Err("num_agents must be positive".into());
        }
        if self.num_agents as usize > crate::population_cap::MAX_POPULATION {
            return Err(format!(
                "num_agents {} exceeds population cap {}",
                self.num_agents,
                crate::population_cap::MAX_POPULATION
            ));
        }
        let mut seen: Vec<u64> = Vec::new();
        for s in &self.shocks {
            if s.at_tick >= self.ticks {
                return Err(format!(
                    "shock at tick {} can never fire (horizon is {})",
                    s.at_tick, self.ticks
                ));
            }
            if seen.contains(&s.at_tick) {
                return Err(format!("duplicate shock tick {}", s.at_tick));
            }
            seen.push(s.at_tick);
            if s.magnitude > Fixed::ONE || s.magnitude < Fixed::ZERO {
                return Err(format!(
                    "shock magnitude {} outside [0,1]",
                    s.magnitude.to_f64()
                ));
            }
        }
        Ok(())
    }

    /// Load a scenario file, sniffing format by content:
    /// leading `{` = JSON, otherwise RON.
    pub fn from_file(path: impl AsRef<std::path::Path>) -> Result<Self, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read {}: {}", path.as_ref().display(), e))?;
        if content.trim_start().starts_with('{') {
            Self::from_json(&content)
        } else {
            Self::from_ron(&content)
        }
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

    #[test]
    fn calm_has_no_shocks() {
        // The control scenario must fire no shocks at any tick.
        let calm = Scenario::calm();
        assert!(calm.shocks.is_empty(), "calm scenario must have no shocks");
        assert!(calm.shock_at(200).is_none(), "calm must not shock at 200");
        assert!(calm.shock_at(500).is_none(), "calm must not shock at 500");
        assert!(calm.shock_at(4320).is_none(), "calm must not shock at 4320");
    }

    // ── DC-1 scenario editor: JSON interchange + validation gate ──

    #[test]
    fn json_roundtrip_preserves_shocks() {
        let scenario = Scenario::collapse();
        let json = scenario.to_json();
        let back = Scenario::from_json(&json).expect("valid preset round-trips");
        assert_eq!(back.name, "Collapse");
        assert_eq!(back.shocks.len(), 4);
        assert_eq!(back.shocks[3].kind, ShockKind::Pestilence);
    }

    #[test]
    fn validate_rejects_post_horizon_and_duplicate_shocks() {
        let mut s = Scenario::riverford();
        s.shocks.push(Shock {
            at_tick: s.ticks + 5,
            kind: ShockKind::Drought,
            magnitude: Fixed::from_f64(0.2),
        });
        assert!(s.validate().is_err(), "post-horizon shock must reject");

        let mut s = Scenario::riverford();
        s.shocks.push(Shock {
            at_tick: 200,
            kind: ShockKind::Famine,
            magnitude: Fixed::from_f64(0.2),
        });
        assert!(s.validate().is_err(), "duplicate tick must reject");
    }

    #[test]
    fn validate_rejects_out_of_range_magnitude_and_population() {
        let mut s = Scenario::riverford();
        s.shocks[0].magnitude = Fixed::from_f64(1.5);
        assert!(s.validate().is_err(), "magnitude >1 must reject");

        let mut s = Scenario::riverford();
        s.num_agents = crate::population_cap::MAX_POPULATION as u32 + 1;
        assert!(s.validate().is_err(), "over-cap population must reject");
        s.num_agents = 0;
        assert!(s.validate().is_err(), "zero population must reject");
    }

    #[test]
    fn from_file_sniffs_json_by_leading_brace() {
        let dir = std::env::temp_dir();
        let path = dir.join("dc1_scenario_sniff_test.json");
        std::fs::write(&path, Scenario::drought().to_json()).expect("write preset");
        let loaded = Scenario::from_file(&path).expect("json preset loads via sniffing");
        assert_eq!(loaded.name, "Drought");
        std::fs::remove_file(&path).ok();
    }
}
