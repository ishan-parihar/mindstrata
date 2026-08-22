//! §18.3 Integration Tests - verify cross-system emergent behaviors.
//!
//! These tests check that multiple subsystems interact correctly to produce
//! emergent phenomena that require cooperation across biology, psychology,
//! social, and institutional layers.

use crate::test_helpers::{run_scenario, run_sim};
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::{scenario::Scenario, sim::SimConfig, Simulation};

// ── §19 Phase 5: Parameter-Sensitivity Integration Tests ─────────
// These tests prove the tuning pipeline is functional (not just wired)
// by varying a parameter and asserting a measurably different outcome.

/// Helper: run simulation with custom params, return metrics snapshot.
/// Delegates to the shared `run_sim_with_params` harness (test_helpers.rs).
fn run_with_params(
    seed: u64,
    ticks: u64,
    modify: impl FnOnce(&mut mindstrata_sim::parameters::SimParameters),
) -> mindstrata_sim::sim::MetricsSnapshot {
    crate::test_helpers::run_sim_with_params(seed, ticks, modify).metrics_snapshot()
}

// ── Phase 4 Modding API (Iteration 156) ──────────────────────────────────────

fn mod_pack_fixture() -> mindstrata_sim::mods::ContentPack {
    use mindstrata_core::fixed::Fixed;
    use mindstrata_sim::culture::knowledge::KnowledgeCategory;
    use mindstrata_sim::culture::Knowledge;
    use mindstrata_sim::mods::{ContentPack, ModManifest};
    use mindstrata_sim::norms::Norm;

    ContentPack {
        manifest: ModManifest {
            name: "River Trade Pact".into(),
            description: "Integration fixture: one norm + one knowledge item.".into(),
            version: "1.0.0".into(),
        },
        scenario: None,
        knowledge: vec![Knowledge {
            id: 100,
            name: "River Ferry".into(),
            category: KnowledgeCategory::Craft,
            difficulty: Fixed::from_f64(0.2),
            utility: Fixed::from_f64(0.9),
            holders: 0,
            discovered_tick: 0,
        }],
        norms: vec![Norm {
            id: 100,
            name: "Honor River Trade".into(),
            strength: Fixed::from_f64(0.6),
            internalization: Fixed::from_f64(0.5),
            punishment: Fixed::from_f64(0.4),
            reinforcing_identity: None,
        }],
    }
}

mod biology;
mod culture;
mod economy;
mod governance;
mod infra;
mod legal;
mod psychology;
mod social;
