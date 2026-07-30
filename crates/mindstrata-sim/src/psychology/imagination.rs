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

impl ProspectionState {
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
}
