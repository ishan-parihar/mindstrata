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
    /// f64 integrator shadow (Iteration 239). The convergence rate (0.001)
    /// and the metabolic-warmth term (~1e-5/tick) are below the 1e-4 Fixed
    /// quantum, so accumulating through the quantized field erased them
    /// every tick. This shadow carries full precision between ticks;
    /// `body_temperature` mirrors it rounded for all consumers. `None`
    /// until the first tick (or a legacy snapshot restore), then seeded
    /// lazily from `body_temperature`.
    #[serde(default)]
    body_temperature_f64: Option<f64>,
}

impl Default for ThermalState {
    fn default() -> Self {
        Self {
            body_temperature: Fixed::from_f64(0.5),
            cold_stress: Fixed::ZERO,
            heat_stress: Fixed::ZERO,
            thermal_set_point: Fixed::from_f64(0.5),
            body_temperature_f64: None,
        }
    }
}

impl ThermalState {
    /// Construct with a genome-derived set-point; all else default.
    /// Used by `EmbodiedState::random` and tests so the private
    /// integrator shadow never leaks into struct-update sites.
    pub fn with_set_point(set_point: Fixed) -> Self {
        Self {
            thermal_set_point: set_point,
            ..Self::default()
        }
    }

    /// Drive body temperature toward ambient and derive cold/heat stress.
    ///
    /// `energy_reserves` moderates warmth (the "metabolic regulation" term
    /// from the pre-Iter-45 block): a well-fed agent resists cold slightly
    /// better. Call order matters for determinism — this must run after
    /// `MetabolicState::tick_update` so `energy_reserves` is the post-update
    /// value, exactly as in the original inlined block.
    pub fn tick_update(&mut self, ambient_temperature: Fixed, energy_reserves: Fixed) {
        // Iteration 239 fix (two stacked defects):
        //
        // Defect 1 (Iteration 222): the restoring term `(ambient − body)`
        // was replaced by `(ambient − set_point)`, which never vanishes
        // when body == ambient — body temperature performed a biased
        // random walk and ratcheted to the 1.0 clamp instead of tracking
        // the seasonal ambient (probe: body 0.769 vs ambient 0.535 after
        // 4320 Spring ticks).
        //
        // Defect 2 (quantization): the 0.001 convergence rate multiplied
        // by a sub-0.05 delta rounds to ZERO in 4-decimal Fixed, so even
        // the original formula stalled ~0.05 short of ambient — and the
        // genome set-point offset (±0.025) was a complete no-op in
        // dynamics.
        //
        // The fix restores first-order convergence on ONE shared delta
        // computed in f64 (the `should_birth` / nervous-recovery
        // precedent for sub-resolution rates), keeping the genome-derived
        // set-point meaningful: a high-set-point agent (0.525) feels any
        // given ambient ~0.025 colder, a low-set-point agent warmer — so
        // equilibrium body temperature varies ±0.025 across genomes and
        // stress stays responsive within a single tick (the winter
        // cold-stress consumers keep their immediate signal). The
        // metabolic-warmth term is likewise promoted from quantized-dead
        // to genuinely live (a well-fed agent measurably resists cold
        // drift). Deterministic: pure f64 arithmetic on Fixed inputs,
        // single rounded store per tick, no RNG.
        let body_f = self
            .body_temperature_f64
            .get_or_insert(self.body_temperature.to_f64());
        let felt_ambient = ambient_temperature.to_f64() + (0.5 - self.thermal_set_point.to_f64());
        let temp_delta = felt_ambient - *body_f;
        self.cold_stress = if temp_delta < -0.05 {
            Fixed::from_f64(-temp_delta - 0.05).clamp_01()
        } else {
            Fixed::ZERO
        };
        self.heat_stress = if temp_delta > 0.05 {
            Fixed::from_f64(temp_delta - 0.05).clamp_01()
        } else {
            Fixed::ZERO
        };
        let metabolic_warmth = energy_reserves.to_f64() * 0.01;
        // Insulation semantics (Iteration 239 sign fix): the pre-Iter-45
        // inline block SUBTRACTED this term while its own comment said
        // "slowed by metabolic regulation" — a well-fed agent cooled
        // FASTER. Invisible for its entire life because the term sat
        // below Fixed quantum; the f64 core exposes it, so the sign now
        // matches the documented intent: energy reserves insulate.
        let next =
            *body_f + temp_delta * 0.001 + metabolic_warmth * self.cold_stress.to_f64() * 0.001;
        self.body_temperature_f64 = Some(next);
        self.body_temperature = Fixed::from_f64(next).clamp_01();
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
    fn body_converges_to_ambient_not_clamp() {
        // Iteration 239 regression pin for the Iter-222 ratchet: the old
        // formula drifted toward `(ambient − set_point)` with no restoring
        // force, so sustained Spring ambient pushed body temperature to the
        // 1.0 clamp (probe: 0.769 @4320 ticks) instead of tracking ambient.
        let mut t = ThermalState::default();
        for _ in 0..4320 {
            t.tick_update(Fixed::from_f64(0.6), Fixed::from_f64(0.7));
        }
        assert!(
            (t.body_temperature.to_f64() - 0.6).abs() <= 0.05,
            "body must track sustained Spring ambient, got {}",
            t.body_temperature.to_f64()
        );
        // A long winter must converge DOWN toward the cold ambient — never
        // upward past it.
        let mut w = ThermalState::default();
        for _ in 0..20_000 {
            w.tick_update(Fixed::from_f64(0.2), Fixed::from_f64(0.7));
        }
        assert!(
            w.body_temperature.to_f64() < 0.35,
            "body must converge toward winter ambient, got {}",
            w.body_temperature.to_f64()
        );
        // Genome set-point shifts the equilibrium by ±set-point-offset only:
        // a high-set-point agent rests ~0.025 BELOW the same ambient (feels
        // colder), a low-set-point agent above — never runaway divergence.
        let mut hot = ThermalState::with_set_point(Fixed::from_f64(0.525));
        let mut cold_sp = ThermalState::with_set_point(Fixed::from_f64(0.475));
        for _ in 0..10_000 {
            hot.tick_update(Fixed::from_f64(0.55), Fixed::from_f64(0.7));
            cold_sp.tick_update(Fixed::from_f64(0.55), Fixed::from_f64(0.7));
        }
        assert!((hot.body_temperature.to_f64() - 0.525).abs() <= 0.01);
        assert!((cold_sp.body_temperature.to_f64() - 0.575).abs() <= 0.01);
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
    fn metabolic_warmth_resists_cold_drift() {
        // Iteration 239 re-pin: under the old Fixed-only math the metabolic
        // term (energy × 0.01 × cold_stress × 0.001, ~1e-5/tick) rounded to
        // zero — a starving and a well-fed agent cooled byte-identically
        // (the documented "inert dead weight"). The f64 core promotes the
        // term to its pre-Iter-45 INTENT: a well-fed agent measurably
        // resists cold drift; the starving agent sits colder at equilibrium.
        let mut starving = ThermalState::default();
        let mut fed = ThermalState::default();
        for _ in 0..100 {
            starving.tick_update(Fixed::from_f64(0.1), Fixed::from_f64(0.05));
            fed.tick_update(Fixed::from_f64(0.1), Fixed::from_f64(0.9));
        }
        assert!(
            fed.body_temperature > starving.body_temperature,
            "fed agent must stay warmer than starving agent under sustained cold (fed {} vs starving {})",
            fed.body_temperature.to_f64(),
            starving.body_temperature.to_f64()
        );
    }
}
