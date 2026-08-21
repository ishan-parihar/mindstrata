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

/// §8.1.16 (Iteration 117): the concern domain that generated a mental
/// scenario — D1 scarcity (urgent deficit / failing harvest), D2 threat
/// (exposure to danger), D3 injustice (grievance seeking redress), D4
/// courtship (unattached with a viable target), D5 ambition (status-
/// seeking, EF-gated), D6 hopeful default. Observational infrastructure:
/// `MentalScenario` was previously opaque, so domain-aware consumers
/// (decision folds, narrative systems) could not distinguish concern
/// domains; the discriminator is the foundation for those. (An earlier
/// Iteration-117 draft wired D3–D5 into `compute_utility` as direct
/// decision folds; the probe premise that only D1/D2 fire in calibrated
/// windows is FALSE — D3/D4/D5 fire in larger/longer worlds — and AP2
/// §8.1.16 prescribes emotion-biased prospection rather than
/// domain→action folds, so that fold was reverted. D1/D2 reach decisions
/// through the Iter-103 dread channel.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ScenarioKind {
    /// D1 — Scarcity: an urgent deficit or a failing harvest.
    D1Scarcity,
    /// D2 — Threat: the agent feels exposed to danger.
    D2Threat,
    /// D3 — Injustice: a grievance seeks redress.
    D3Injustice,
    /// D4 — Courtship: unattached with a viable attraction target.
    D4Courtship,
    /// D5 — Ambition: status-seeking (EF-gated).
    D5Ambition,
    /// D6 — Hopeful default.
    D6Hopeful,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MentalScenario {
    /// The concern domain that generated this scenario (§8.1.16 Iter-117).
    pub kind: ScenarioKind,
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
    /// §8.1.12 (Iteration 80): Whether the agent's executive function
    /// currently permits long-term planning (`CognitiveRuntime::can_plan_long_term`).
    /// Gates the D5 ambition scenario — the plan's "high executive function
    /// enables long-term planning" coupling: an ambitious agent whose EF is
    /// degraded (stressed/sleep-deprived/traumatized) cannot simulate the
    /// status-seeking future and falls through to the D6 hopeful default.
    pub can_plan_long_term: bool,
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
                ScenarioKind::D1Scarcity,
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
                ScenarioKind::D2Threat,
                "If I travel alone, I may come to harm.".into(),
                inputs.fear,
                Fixed::from_f64(-0.5),
                Fixed::ZERO,
                Fixed::from_f64(-0.3),
                Fixed::from_f64(0.7),
                Fixed::from_f64(0.6),
                tick,
            );
            // §8.1.16 (P3-3, August 14, 2026): the D2 gate (`fear > 0.4`)
            // fired for the majority at the calibrated ambient fear
            // (0.65+), structurally starving the positive scenarios —
            // probe-pinned: `prospection.hope` 0.000 for 12/12 agents in
            // every window while dread stayed live (~0.39). Even under
            // chronic ambient fear, a tolerable present keeps a hopeful
            // future in view: generate the D6 hopeful default alongside
            // the threat scenario so hope has a producer. Deterministic,
            // observational (hope has no behavioral consumer; prospection
            // is absent from the golden projection) — calibrated runs stay
            // byte-identical.
            self.generate_scenario(
                ScenarioKind::D6Hopeful,
                "If the rains come, the village thrives.".into(),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.4),
                Fixed::from_f64(0.2),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.4),
                tick,
            );
            return;
        }
        // D3 — Injustice: a grievance seeks redress.
        if inputs.anger > Fixed::from_f64(0.4) {
            self.generate_scenario(
                ScenarioKind::D3Injustice,
                "If I complain, the council may punish me.".into(),
                Fixed::from_f64(0.4),
                Fixed::from_f64(-0.3),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.2),
                Fixed::from_f64(0.5),
                Fixed::from_f64(0.5),
                tick,
            );
            // §8.1.16 (P3-3): same hopeful alongside the grievance — a
            // just world the agent still believes in.
            self.generate_scenario(
                ScenarioKind::D6Hopeful,
                "If the rains come, the village thrives.".into(),
                Fixed::from_f64(0.6),
                Fixed::from_f64(0.4),
                Fixed::from_f64(0.2),
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.1),
                Fixed::from_f64(0.4),
                tick,
            );
            return;
        }
        // D4 — Courtship: unattached with a viable attraction target.
        if !inputs.has_partner && inputs.total_attraction > Fixed::from_f64(0.4) {
            self.generate_scenario(
                ScenarioKind::D4Courtship,
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
        // D5 — Ambition: status-seeking. §8.1.12 gate (Iteration 80):
        // long-term planning requires intact executive function — an
        // ambitious agent that cannot plan long-term right now falls
        // through to the D6 hopeful default instead.
        if inputs.ambition > Fixed::from_f64(0.5) && inputs.can_plan_long_term {
            self.generate_scenario(
                ScenarioKind::D5Ambition,
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
            ScenarioKind::D6Hopeful,
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
        kind: ScenarioKind,
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
            kind,
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
        self.scenarios.iter().max_by(|a, b| {
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
        self.catastrophic_bias = (Fixed::from_f64(0.2)
            + trauma_load * Fixed::from_f64(0.3)
            + fear * Fixed::from_f64(0.2))
        .clamp_01();

        // Ambition amplifies optimism
        self.optimism_bias = (Fixed::from_f64(0.3) + ambition * Fixed::from_f64(0.3)
            - depression * Fixed::from_f64(0.2))
        .clamp_01();

        // Iteration 229: hope has a producer (optimism_bias) and a
        // drain (depression). Without the producer, hope was permanently
        // 0.000 because only decay existed. Optimistic agents in calm
        // worlds slowly rebuild hope; depressed agents lose it faster.
        let hope_gain = self.optimism_bias * Fixed::from_f64(0.002);
        let hope_loss = depression * Fixed::from_f64(0.008);
        self.hope = (self.hope + hope_gain - hope_loss).clamp_01();

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
                ScenarioKind::D6Hopeful,
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
            ScenarioKind::D2Threat,
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
            ScenarioKind::D6Hopeful,
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
                can_plan_long_term: true,
            },
        );
        let s = pro.scenarios.last().unwrap();
        assert!(s.description.contains("harvest fails"));
        assert!(
            s.expected_valence < Fixed::ZERO,
            "scarcity scenarios are dread"
        );
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
                can_plan_long_term: true,
            },
        );
        // P3-3: the D2 branch now also emits the D6 hopeful alongside, so
        // the threat scenario is present (not necessarily last).
        assert!(pro
            .scenarios
            .iter()
            .any(|s| s.description.contains("come to harm")));
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
                can_plan_long_term: true,
            },
        );
        // P3-3: the D3 branch now also emits the D6 hopeful alongside, so
        // the complaint scenario is present (not necessarily last).
        assert!(pro
            .scenarios
            .iter()
            .any(|s| s.description.contains("council may punish")));
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
                can_plan_long_term: true,
            },
        );
        assert!(!attached
            .scenarios
            .last()
            .unwrap()
            .description
            .contains("court"));
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
                can_plan_long_term: true,
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
                can_plan_long_term: true,
            },
        );
        assert!(viable
            .scenarios
            .last()
            .unwrap()
            .description
            .contains("court"));
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
                can_plan_long_term: true,
            },
        );
        assert!(pro
            .scenarios
            .last()
            .unwrap()
            .description
            .contains("gain respect"));
    }

    #[test]
    fn ambition_domain_gated_on_planning_capacity() {
        // Ambitious agent with intact EF: D5 ambition scenario fires.
        let mut intact = ProspectionState::default();
        intact.generate_daily_scenario(
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
                can_plan_long_term: true,
            },
        );
        assert!(intact
            .scenarios
            .last()
            .unwrap()
            .description
            .contains("gain respect"));
        // Ambitious agent with degraded EF: falls through to the D6 hopeful default.
        let mut degraded = ProspectionState::default();
        degraded.generate_daily_scenario(
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
                can_plan_long_term: false,
            },
        );
        let d = degraded.scenarios.last().unwrap();
        assert!(!d.description.contains("gain respect"));
        assert!(d.description.contains("rains come"));
    }

    #[test]
    fn non_ambitious_agent_never_fires_d5_even_with_planning() {
        let mut pro = ProspectionState::default();
        pro.generate_daily_scenario(
            100,
            ScenarioInputs {
                hunger: Fixed::ZERO,
                thirst: Fixed::ZERO,
                safety: Fixed::ZERO,
                fear: Fixed::ZERO,
                anger: Fixed::ZERO,
                ambition: Fixed::from_f64(0.4),
                food_scarcity: Fixed::ZERO,
                water_scarcity: Fixed::ZERO,
                has_partner: false,
                total_attraction: Fixed::ZERO,
                can_plan_long_term: true,
            },
        );
        let d = pro.scenarios.last().unwrap();
        assert!(!d.description.contains("gain respect"));
        assert!(d.description.contains("rains come"));
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
                can_plan_long_term: true,
            },
        );
        let s = pro.scenarios.last().unwrap();
        assert!(s.description.contains("rains come"));
        assert!(
            s.expected_valence > Fixed::ZERO,
            "default scenario is hopeful"
        );
    }

    /// §8.1.16 (P3-3): a fearful agent (D2 gate) previously generated ONLY
    /// the threat scenario — the D6 hopeful default was unreachable at the
    /// calibrated ambient fear, starving `hope` (probe: 0.000 in every
    /// window). The D2 branch must now also generate the hopeful scenario.
    #[test]
    fn fearful_agent_generates_hopeful_alongside_threat() {
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
                can_plan_long_term: true,
            },
        );
        let has_threat = pro
            .scenarios
            .iter()
            .any(|s| s.kind == ScenarioKind::D2Threat);
        let has_hopeful = pro
            .scenarios
            .iter()
            .any(|s| s.kind == ScenarioKind::D6Hopeful);
        assert!(has_threat, "fearful agent generates the threat scenario");
        assert!(has_hopeful, "fearful agent ALSO generates the hopeful default");
        // And the hopeful scenario feeds `hope` after evaluation.
        pro.evaluate_scenarios();
        assert!(pro.hope > Fixed::ZERO, "hope is no longer structurally starved");
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
            can_plan_long_term: true,
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
                can_plan_long_term: true,
            },
        );
        pro.evaluate_scenarios();
        assert!(
            pro.dread > baseline_dread,
            "scenario evaluation must raise dread"
        );
        assert!(
            pro.hope <= baseline_hope,
            "dread scenario must not raise hope"
        );
    }

    /// §8.1.16 (Iteration 117): `generate_daily_scenario` stamps each
    /// scenario with the concern domain that produced it — D1 scarcity
    /// (hunger > 0.6), D2 threat (fear > 0.4), D3 injustice (anger > 0.4),
    /// D4 courtship (unattached + attraction), D5 ambition (EF-gated),
    /// D6 hopeful default. This is the discriminator future domain-aware
    /// consumers (decision folds, narrative systems) key on — if a domain
    /// ever stops setting its kind, its scenarios become indistinguishable
    /// from the default.
    #[test]
    fn daily_scenario_kinds_are_stamped_per_domain() {
        let inputs = |hunger: Fixed,
                      fear: Fixed,
                      anger: Fixed,
                      ambition: Fixed,
                      has_partner: bool,
                      attraction: Fixed,
                      can_plan: bool| ScenarioInputs {
            hunger,
            thirst: Fixed::ZERO,
            safety: Fixed::ZERO,
            fear,
            anger,
            ambition,
            food_scarcity: Fixed::ZERO,
            water_scarcity: Fixed::ZERO,
            has_partner,
            total_attraction: attraction,
            can_plan_long_term: can_plan,
        };
        let mut pro = ProspectionState::default();

        // D1 scarcity (hunger beats a zero elsewheres).
        pro.generate_daily_scenario(
            1,
            inputs(
                Fixed::from_f64(0.8),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                false,
                Fixed::ZERO,
                true,
            ),
        );
        assert_eq!(pro.scenarios.last().unwrap().kind, ScenarioKind::D1Scarcity);

        // D2 threat (fear 0.6, hunger below the scarcity gate).
        pro.generate_daily_scenario(
            2,
            inputs(
                Fixed::from_f64(0.3),
                Fixed::from_f64(0.6),
                Fixed::ZERO,
                Fixed::ZERO,
                false,
                Fixed::ZERO,
                true,
            ),
        );
        assert!(pro.scenarios.iter().any(|s| s.kind == ScenarioKind::D2Threat));

        // D3 injustice (anger 0.6, everything else calm).
        pro.generate_daily_scenario(
            3,
            inputs(
                Fixed::from_f64(0.3),
                Fixed::ZERO,
                Fixed::from_f64(0.6),
                Fixed::ZERO,
                false,
                Fixed::ZERO,
                true,
            ),
        );
        assert!(pro
            .scenarios
            .iter()
            .any(|s| s.kind == ScenarioKind::D3Injustice));

        // D4 courtship (unattached with attraction 0.6, else calm).
        pro.generate_daily_scenario(
            4,
            inputs(
                Fixed::from_f64(0.3),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                false,
                Fixed::from_f64(0.6),
                true,
            ),
        );
        assert_eq!(
            pro.scenarios.last().unwrap().kind,
            ScenarioKind::D4Courtship
        );

        // D5 ambition (ambition 0.7 + intact EF, else calm).
        pro.generate_daily_scenario(
            5,
            inputs(
                Fixed::from_f64(0.3),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::from_f64(0.7),
                false,
                Fixed::ZERO,
                true,
            ),
        );
        assert_eq!(pro.scenarios.last().unwrap().kind, ScenarioKind::D5Ambition);

        // D6 hopeful default (all calm).
        pro.generate_daily_scenario(
            6,
            inputs(
                Fixed::from_f64(0.3),
                Fixed::ZERO,
                Fixed::ZERO,
                Fixed::ZERO,
                false,
                Fixed::ZERO,
                true,
            ),
        );
        assert_eq!(pro.scenarios.last().unwrap().kind, ScenarioKind::D6Hopeful);
    }

    /// §8.1.16 (Iteration 117): the plan's four emotion-bias lines are
    /// genuinely wired in `update` — fear amplifies dread, ambition
    /// amplifies hope, trauma amplifies catastrophe, depression reduces
    /// hope. Each line must move its target monotonically from a calm
    /// baseline. Guards against a future refactor silently deleting one
    /// of the plan's bias terms.
    #[test]
    fn emotion_bias_lines_match_the_plan() {
        let mut pro = ProspectionState::default();
        pro.update(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        let base_cat = pro.catastrophic_bias;
        let base_opt = pro.optimism_bias;

        // Trauma amplifies catastrophe.
        pro.update(Fixed::ZERO, Fixed::ZERO, Fixed::from_f64(0.5), Fixed::ZERO);
        assert!(
            pro.catastrophic_bias > base_cat,
            "trauma must amplify catastrophe"
        );

        // Fear amplifies dread (the catastrophic-bias channel).
        pro.update(Fixed::from_f64(0.5), Fixed::ZERO, Fixed::ZERO, Fixed::ZERO);
        assert!(pro.catastrophic_bias > base_cat, "fear must amplify dread");

        // Ambition amplifies hope (the optimism-bias channel).
        pro.update(Fixed::ZERO, Fixed::from_f64(0.5), Fixed::ZERO, Fixed::ZERO);
        assert!(pro.optimism_bias > base_opt, "ambition must amplify hope");

        // Depression reduces hope and tempers optimism.
        pro.hope = Fixed::from_f64(0.5);
        pro.update(Fixed::ZERO, Fixed::ZERO, Fixed::ZERO, Fixed::from_f64(0.5));
        assert!(
            pro.hope < Fixed::from_f64(0.5),
            "depression must reduce hope"
        );
        assert!(
            pro.optimism_bias < base_opt,
            "depression must temper optimism"
        );
    }
}
