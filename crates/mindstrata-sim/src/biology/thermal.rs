//! Thermoregulation (Architecture-plan-2 §7.3.3).
//!
//! The plan's integrated body architecture (§7.4) mandates a distinct
//! `ThermalState` inside `EmbodiedState` alongside the other subsystems:
//!
//! ```text
//! ThermalState {
//!     body_temperature: Fixed,  // 0.45 hypothermic, 0.5 normal, 0.55 febrile
//!     cold_stress: Fixed,       // 0 comfortable, 1 dangerously cold
//!     heat_stress: Fixed,       // 0 comfortable, 1 dangerously hot
//! }
//! ```
//!
//! Before Iteration 45 these three fields lived inside `MetabolicState`; the
//! extraction is a pure relocation — the update math, call order, and field
//! values are identical (thermal runs immediately after metabolic with the
//! post-update energy reserves), so calibrated trajectories are byte-identical.
//!
//! Effects (per the plan): winter hardship, housing quality, clothing needs,
//! migration pressure. These consumers are future iterations; the dynamics
//! here are the homeostasis itself.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Body thermoregulation state (§7.3.3).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThermalState {
    /// Body temperature analog (0.45 = hypothermic, 0.5 = normal, 0.55 = febrile).
    pub body_temperature: Fixed,
    /// Cold stress (0 = comfortable, 1 = dangerously cold).
    pub cold_stress: Fixed,
    /// Heat stress (0 = comfortable, 1 = dangerously hot).
    pub heat_stress: Fixed,
    /// Individual thermal set-point (0.475–0.525). Genome-derived
    /// variation means each agent drifts toward a slightly different
    /// equilibrium — the same ambient temperature produces different
    /// cold/heat stress across agents, breaking the 0.554 probe
    /// saturation. Exactly 0.5 for agents created without RNG (default).
    #[serde(default)]
    pub thermal_set_point: Fixed,
}

impl Default for ThermalState {
    fn default() -> Self {
        Self {
            body_temperature: Fixed::from_f64(0.5),
            cold_stress: Fixed::ZERO,
            heat_stress: Fixed::ZERO,
            thermal_set_point: Fixed::from_f64(0.5),
        }
    }
}

impl ThermalState {
    /// Drive body temperature toward ambient and derive cold/heat stress.
    ///
    /// `energy_reserves` moderates warmth (the "metabolic regulation" term
    /// from the pre-Iter-45 block): a well-fed agent resists cold slightly
    /// better. Call order matters for determinism — this must run after
    /// `MetabolicState::tick_update` so `energy_reserves` is the post-update
    /// value, exactly as in the original inlined block.
    pub fn tick_update(&mut self, ambient_temperature: Fixed, energy_reserves: Fixed) {
        let temp_delta = ambient_temperature - self.body_temperature;
        self.cold_stress = if temp_delta < -Fixed::from_f64(0.05) {
            (-temp_delta - Fixed::from_f64(0.05)).clamp_01()
        } else {
            Fixed::ZERO
        };
        self.heat_stress = if temp_delta > Fixed::from_f64(0.05) {
            (temp_delta - Fixed::from_f64(0.05)).clamp_01()
        } else {
            Fixed::ZERO
        };
        // Iteration 222: body temperature drifts toward ambient RELATIVE
        // to the agent's individual thermal set-point (genome-derived ±5%
        // from 0.5). This means the same ambient temperature produces
        // different body temperatures across agents — one agent may feel
        // slightly cold while another feels comfortable. The set-point
        // anchors the resting temperature; the ambient delta drives
        // deviation from that anchor. Metabolic warmth from energy
        // reserves resists cold drift. Deterministic, no RNG.
        let set_point_delta = ambient_temperature - self.thermal_set_point;
        let metabolic_warmth = energy_reserves * Fixed::from_f64(0.01);
        self.body_temperature = (self.body_temperature
            + set_point_delta * Fixed::from_f64(0.001)
            - metabolic_warmth * self.cold_stress * Fixed::from_f64(0.001))
        .clamp_01();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_thermoneutral() {
        let t = ThermalState::default();
        assert_eq!(t.body_temperature, Fixed::from_f64(0.5));
        assert_eq!(t.cold_stress, Fixed::ZERO);
        assert_eq!(t.heat_stress, Fixed::ZERO);
    }

    #[test]
    fn cold_stress_from_low_ambient() {
        let mut t = ThermalState::default();
        t.tick_update(Fixed::from_f64(0.1), Fixed::from_f64(0.7)); // very cold
        assert!(t.cold_stress > Fixed::ZERO);
        assert_eq!(t.heat_stress, Fixed::ZERO);
        // Body temperature drifts down toward the cold ambient.
        assert!(t.body_temperature < Fixed::from_f64(0.5));
    }

    #[test]
    fn heat_stress_from_high_ambient() {
        let mut t = ThermalState::default();
        t.tick_update(Fixed::from_f64(0.9), Fixed::from_f64(0.7)); // very hot
        assert!(t.heat_stress > Fixed::ZERO);
        assert_eq!(t.cold_stress, Fixed::ZERO);
        assert!(t.body_temperature > Fixed::from_f64(0.5));
    }

    #[test]
    fn neutral_ambient_keeps_thermoneutral() {
        let mut t = ThermalState::default();
        t.tick_update(Fixed::from_f64(0.5), Fixed::from_f64(0.7));
        assert_eq!(t.body_temperature, Fixed::from_f64(0.5));
        assert_eq!(t.cold_stress, Fixed::ZERO);
        assert_eq!(t.heat_stress, Fixed::ZERO);
    }

    #[test]
    fn small_delta_stays_in_comfort_band() {
        // |delta| <= 0.05 is within the comfort band — no stress recorded.
        let mut t = ThermalState::default();
        t.tick_update(Fixed::from_f64(0.52), Fixed::from_f64(0.7));
        assert_eq!(t.cold_stress, Fixed::ZERO);
        assert_eq!(t.heat_stress, Fixed::ZERO);
    }

    #[test]
    fn energy_term_is_below_fixed_precision() {
        // Probe-verified: the pre-Iter-45 metabolic block's "metabolic warmth"
        // term (energy_reserves x 0.01 x cold_stress x 0.001, max ~1e-5/tick)
        // is below Fixed precision — even 1000 cold ticks leave a starving and
        // a well-fed agent byte-identical. The extraction preserves this
        // exactly (byte-identical mandate): the term is inert dead weight,
        // carried over verbatim rather than silently changing behavior.
        let mut starving = ThermalState::default();
        let mut fed = ThermalState::default();
        for _ in 0..100 {
            starving.tick_update(Fixed::from_f64(0.1), Fixed::from_f64(0.05));
            fed.tick_update(Fixed::from_f64(0.1), Fixed::from_f64(0.9));
        }
        assert_eq!(
            starving.body_temperature, fed.body_temperature,
            "energy term must remain inert at Fixed precision"
        );
    }
}
