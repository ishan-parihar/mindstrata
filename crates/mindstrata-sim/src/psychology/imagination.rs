//! Imagination and prospection system — agents simulate possible futures.
//!
//! Mental scenarios: "If I complain, the council may punish me."
//!                     "If I marry her, my clan gains land."
//!                     "If the harvest fails, my child dies."
//!
//! Prospection is biased by emotion:
//! - fear amplifies dread,
//! - ambition amplifies hope,
//! - trauma amplifies catastrophe,
//! - depression reduces hope.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A mental scenario — an imagined future outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalScenario {
    /// Brief description of the imagined scenario.
    pub description: String,
    /// Perceived probability of this outcome (0–1).
    pub probability: Fixed,
    /// Expected emotional valence if this occurs (-1 to 1).
    pub expected_valence: Fixed,
    /// Expected need relief if this occurs (0–1).
    pub expected_need_relief: Fixed,
    /// Expected social consequence (positive = approval, negative = disapproval).
    pub social_consequence: Fixed,
    /// Expected risk (0–1).
    pub risk: Fixed,
    /// Tick when this scenario was generated.
    pub generated_tick: u64,
    /// How vividly the agent imagines this (affects emotional impact).
    pub vividness: Fixed,
}

/// Imagination and prospection state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProspectionState {
    /// Current mental scenarios being considered.
    pub scenarios: Vec<MentalScenario>,
    /// Overall hope level (0–1).
    pub hope: Fixed,
    /// Overall dread level (0–1).
    pub dread: Fixed,
    /// Optimism bias — tendency to overestimate positive outcomes.
    pub optimism_bias: Fixed,
    /// Catastrophic bias — tendency to overestimate negative outcomes.
    pub catastrophic_bias: Fixed,
    /// Confidence in ability to plan and execute.
    pub planning_confidence: Fixed,
    /// Maximum number of scenarios held in mind.
    pub capacity: usize,
}

impl Default for ProspectionState {
    fn default() -> Self {
        Self {
            scenarios: Vec::new(),
            hope: Fixed::from_f64(0.5),
            dread: Fixed::from_f64(0.2),
            optimism_bias: Fixed::from_f64(0.3),
            catastrophic_bias: Fixed::from_f64(0.2),
            planning_confidence: Fixed::from_f64(0.5),
            capacity: 5,
        }
    }
}

/// §8.1.16 (Iteration 71): Live-state inputs the sim derives each daily pass
/// for deterministic mental-scenario generation. Pure `Copy` struct — the
/// sim builds it from existing agent/world state, and nothing behavioral
/// consumes the scenarios yet (observational state).
#[derive(Debug, Clone, Copy)]
pub struct ScenarioInputs {
    /// Hunger deficit (0–1).
    pub hunger: Fixed,
    /// Thirst deficit (0–1).
    pub thirst: Fixed,
    /// Safety deficit (0–1).
    pub safety: Fixed,
    /// Current fear emotion (0–1).
    pub fear: Fixed,
    /// Current anger emotion (0–1).
    pub anger: Fixed,
    /// Ambition personality trait (0–1).
    pub ambition: Fixed,
    /// Village food scarcity (0–1).
    pub food_scarcity: Fixed,
    /// Village water scarcity (0–1).
    pub water_scarcity: Fixed,
    /// Whether the agent already has a partner — courtship scenarios only
    /// generate for unattached agents.
    pub has_partner: bool,
    /// The agent's total attraction toward potential partners (0–1).
    pub total_attraction: Fixed,
}

impl ProspectionState {
    /// Generate the agent's daily mental scenario — the plan's core
    /// "agents simulate possible futures" mechanic. Picks the dominant
    /// concern domain by deterministic priority (urgent deficit > threat >
    /// injustice > courtship > ambition > hopeful default) and derives the
    /// scenario's parameters (probability, valence, need relief, social
    /// consequence, risk, vividness) from live state. Fully deterministic —
    /// no RNG. Bounded by `capacity` (oldest evicted).
    pub fn generate_daily_scenario(&mut self, tick: u64, inputs: ScenarioInputs) {
        let scarcity = inputs.food_scarcity.max(inputs.water_scarcity);
        // D1 — Scarcity: an urgent deficit or a failing harvest.
        if inputs.hunger > Fixed::from_f64(0.6)
            || inputs.thirst > Fixed::from_f64(0.6)
            || scarcity > Fixed::from_f64(0.5)
        {
            let probability = (Fixed::from_f64(0.5) + scarcity * Fixed::from_f64(0.4)).clamp_01();
            self.generate_scenario(
                "If the harvest fails, my household goes hungry.".into(),
                probability,
                Fixed::from_f64(-0.6),
                Fixed::ZERO,
                Fixed::from_f64(-0.2),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.7),
                tick,
            );
            return;
        }
        // D2 — Threat: the agent feels exposed to danger.
        if inputs.safety > Fixed::from_f64(0.5) || inputs.fear > Fixed::from_f64(0.4) {
            self.generate_scenario(
                "If I travel alone, I may come to harm.".into(),
                inputs.fear,
                Fixed::from_f64(-0.5),
                Fixed::ZERO,
                Fixed::from_f64(-0.3),
                Fixed::from_f64(0.7),
                Fixed::from_f64(0.6),
                tick,
            );
            return;
        }
        // D3 — Injustice: a grievance seeks redress.
        if inputs.anger > Fixed::from_f64(0.4) {
            self.generate_scenario(
                "If I complain, the council may punish me.".into(),
                Fixed::from_f64(0.4),
                Fixed::from_f64(-0.3),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.2),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                tick,
            );
            return;
        }
        // D4 — Courtship: unattached with a viable attraction target.
        if !inputs.has_partner && inputs.total_attraction > Fixed::from_f64(0.4) {
            self.generate_scenario(
                "If I court them, my kin gain standing.".into(),
                inputs.total_attraction,
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.2),
                Fixed::from_f64(0.6),
                tick,
            );
            return;
        }
        // D5 — Ambition: status-seeking.
        if inputs.ambition > Fixed::from_f64(0.5) {
            self.generate_scenario(
                "If I distinguish myself, I may gain respect.".into(),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.4),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.5),
                tick,
            );
            return;
        }
        // D6 — Hopeful default.
        self.generate_scenario(
            "If the rains come, the village thrives.".into(),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.4),
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.4),
            tick,
        );
    }

    /// Generate a new mental scenario based on current state and action options.
    pub fn generate_scenario(
        &mut self,
        description: String,
        base_probability: Fixed,
        expected_valence: Fixed,
        expected_need_relief: Fixed,
        social_consequence: Fixed,
        risk: Fixed,
        vividness: Fixed,
        tick: u64,
    ) {
        // Apply biases
        let biased_valence = if expected_valence > Fixed::ZERO {
            expected_valence + self.optimism_bias * Fixed::from_f64(0.1)
        } else {
            expected_valence - self.catastrophic_bias * Fixed::from_f64(0.1)
        };

        let scenario = MentalScenario {
            description,
            probability: base_probability,
            expected_valence: biased_valence.clamp(-Fixed::ONE, Fixed::ONE),
            expected_need_relief,
            social_consequence,
            risk,
            generated_tick: tick,
            vividness,
        };

        // Evict oldest if at capacity
        if self.scenarios.len() >= self.capacity {
            self.scenarios.remove(0);
        }
        self.scenarios.push(scenario);
    }

    /// Evaluate all scenarios and update hope/dread levels.
    pub fn evaluate_scenarios(&mut self) {
        if self.scenarios.is_empty() {
            return;
        }

        let mut total_hope = Fixed::ZERO;
        let mut total_dread = Fixed::ZERO;
        let mut count = Fixed::ZERO;

        for scenario in &self.scenarios {
            let weight = scenario.probability * scenario.vividness;
            if scenario.expected_valence > Fixed::ZERO {
                total_hope += weight * scenario.expected_valence;
            } else {
                total_dread += weight * scenario.expected_valence.abs();
            }
            count += Fixed::ONE;
        }

        if count > Fixed::ZERO {
            self.hope = (total_hope / count).clamp_01();
            self.dread = (total_dread / count).clamp_01();
        }

        // Keep all scenarios for now — future optimization: age out old ones
    }

    /// Select the best scenario to pursue based on expected value.
    pub fn best_scenario(&self) -> Option<&MentalScenario> {
        self.scenarios
            .iter()
            .max_by(|a, b| {
                let val_a = a.expected_need_relief + a.expected_valence * Fixed::from_f64(0.5)
                    - a.risk * Fixed::from_f64(0.3);
                let val_b = b.expected_need_relief + b.expected_valence * Fixed::from_f64(0.5)
                    - b.risk * Fixed::from_f64(0.3);
                val_a.to_raw().cmp(&val_b.to_raw())
            })
    }

    /// Update prospection state based on current emotions and trauma.
    pub fn update(&mut self, fear: Fixed, ambition: Fixed, trauma_load: Fixed, depression: Fixed) {
        // Fear amplifies catastrophic bias
        self.catastrophic_bias = (Fixed::from_f64(0.2) + trauma_load * Fixed::from_f64(0.3)
            + fear * Fixed::from_f64(0.2))
            .clamp_01();

        // Ambition amplifies optimism
        self.optimism_bias = (Fixed::from_f64(0.3) + ambition * Fixed::from_f64(0.3)
            - depression * Fixed::from_f64(0.2))
            .clamp_01();

        // Depression reduces hope
        self.hope = (self.hope - depression * Fixed::from_f64(0.01)).max(Fixed::ZERO);

        // Planning confidence affected by cognitive state
        self.planning_confidence = (Fixed::from_f64(0.5) - fear * Fixed::from_f64(0.2)
            + ambition * Fixed::from_f64(0.2))
            .clamp_01();
    }

    /// Clear scenarios (e.g., after major life event).
    pub fn clear(&mut self) {
        self.scenarios.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scenario_generation_respects_capacity() {
        let mut pro = ProspectionState {
            capacity: 3,
            ..ProspectionState::default()
        };
        for i in 0..5 {
            pro.generate_scenario(
                format!("scenario {i}"),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.4),
                Fixed::ZERO,
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.6),
                i,
            );
        }
        assert!(pro.scenarios.len() <= 3);
    }

    #[test]
    fn best_scenario_maximizes_value() {
        let mut pro = ProspectionState::default();
        pro.generate_scenario(
            "bad".into(),
            Fixed::from_f64(0.8),
            Fixed::from_f64(-0.5),
            Fixed::from_f64(0.1),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        );
        pro.generate_scenario(
            "good".into(),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.5),
            0,
        );
        let best = pro.best_scenario().unwrap();
        assert_eq!(best.description, "good");
    }

    #[test]
    fn fear_increases_catastrophic_bias() {
        let mut pro = ProspectionState::default();
        let initial = pro.catastrophic_bias;
        pro.update(Fixed::from_f64(0.9), Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        assert!(pro.catastrophic_bias > initial);
    }

    #[test]
    fn scarcity_domain_generates_dread_scenario() {
        let mut pro = ProspectionState::default();
        pro.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::from_f64(0.8),
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::from_f64(0.9),
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::ZERO,
            },
        );
        let s = pro.scenarios.last().unwrap();
        assert!(s.description.contains("harvest fails"));
        assert!(s.expected_valence < Fixed::ZERO, "scarcity scenarios are dread");
    }

    #[test]
    fn threat_domain_fires_on_fear() {
        let mut pro = ProspectionState::default();
        pro.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::from_f64(0.7),
                anger: Fixed::ZERO,
                ambition: Fixed::ZERO,
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::ZERO,
            },
        );
        assert!(pro.scenarios.last().unwrap().description.contains("come to harm"));
    }

    #[test]
    fn injustice_domain_generates_complaint() {
        let mut pro = ProspectionState::default();
        pro.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::from_f64(0.6),
                ambition: Fixed::ZERO,
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::ZERO,
            },
        );
        assert!(pro.scenarios.last().unwrap().description.contains("council may punish"));
    }

    #[test]
    fn courtship_domain_requires_unattached_and_attraction() {
        // Attached agent: no courtship scenario even with high attraction.
        let mut attached = ProspectionState::default();
        attached.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::ZERO,
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: true,
                total_attraction: Fixed::from_f64(0.8),
            },
        );
        assert!(!attached.scenarios.last().unwrap().description.contains("court"));
        // Unattached with low attraction: no courtship scenario either.
        let mut low = ProspectionState::default();
        low.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::ZERO,
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::from_f64(0.2),
            },
        );
        assert!(!low.scenarios.last().unwrap().description.contains("court"));
        // Unattached with high attraction: courtship fires.
        let mut viable = ProspectionState::default();
        viable.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::ZERO,
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::from_f64(0.8),
            },
        );
        assert!(viable.scenarios.last().unwrap().description.contains("court"));
    }

    #[test]
    fn ambition_domain_generates_respect_scenario() {
        let mut pro = ProspectionState::default();
        pro.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::from_f64(0.7),
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::ZERO,
            },
        );
        assert!(pro.scenarios.last().unwrap().description.contains("gain respect"));
    }

    #[test]
    fn default_domain_is_hopeful() {
        let mut pro = ProspectionState::default();
        pro.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::ZERO,
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::ZERO,
            },
        );
        let s = pro.scenarios.last().unwrap();
        assert!(s.description.contains("rains come"));
        assert!(s.expected_valence > Fixed::ZERO, "default scenario is hopeful");
    }

    #[test]
    fn daily_generation_respects_capacity() {
        let mut pro = ProspectionState {
            capacity: 3,
            ..ProspectionState::default()
        };
        let inputs = ScenarioInputs {
            hunger: Fixed::ZERO,
            thirst: Fixed::ZERO,
            safety: Fixed::ZERO,
            fear: Fixed::ZERO,
            anger: Fixed::ZERO,
            ambition: Fixed::ZERO,
            food_scarcity: Fixed::ZERO,
            water_scarcity: Fixed::ZERO,
            has_partner: false,
            total_attraction: Fixed::ZERO,
        };
        for i in 0..10u64 {
            pro.generate_daily_scenario(i, inputs);
        }
        assert!(pro.scenarios.len() <= 3);
    }

    #[test]
    fn evaluation_regrounds_hope_and_dread() {
        let mut pro = ProspectionState::default();
        let baseline_hope = pro.hope;
        let baseline_dread = pro.dread;
        // A vivid, probable dread scenario drives dread up and hope down.
        pro.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::from_f64(0.9),
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::ZERO,
                food_scarcity: Fixed::from_f64(0.8),
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::ZERO,
            },
        );
        pro.evaluate_scenarios();
        assert!(pro.dread > baseline_dread, "scenario evaluation must raise dread");
        assert!(pro.hope <= baseline_hope, "dread scenario must not raise hope");
    }
}
