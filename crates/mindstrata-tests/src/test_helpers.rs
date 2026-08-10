//! Shared test utilities for Mindstrata integration and snapshot tests.

use mindstrata_sim::parameters::SimParameters;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::{Simulation, sim::SimConfig};

/// Run a simulation with given seed and tick count, returning it for inspection.
///
/// Uses a 16×16 world with 12 agents — the standard test configuration.
pub fn run_sim(seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

/// Run a simulation with mutated tuning parameters, returning it for inspection.
///
/// The mutation is applied to `sim.params` BEFORE `populate`/`run`, so every
/// consumer that reads a parameter during the run sees the override. Same seed
/// and parameter values produce the identical RNG stream and outcome — the
/// contract the behavioral-delta harness (`behavioral_delta` module) relies on.
///
/// Uses a 16×16 world with 12 agents — the standard test configuration.
pub fn run_sim_with_params(
    seed: u64,
    ticks: u64,
    modify: impl FnOnce(&mut SimParameters),
) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    modify(&mut sim.params);
    sim.populate();
    sim.run(ticks);
    sim
}

/// Run a simulation from a scenario with mutated parameters, returning it.
///
/// Clones the scenario (overriding ticks to the requested horizon), constructs
/// a `Simulation` via `from_scenario`, applies the parameter mutation BEFORE
/// `populate`/`run` (same-seed determinism: same clone → identical RNG
/// stream), and runs with the scenario's shocks firing at their `at_tick`
/// times. Use this to probe consumer liveness in non-calm contexts (drought,
/// famine) where calm-window-inert consumers actually fire.
pub fn run_scenario_with_params(
    scenario: &Scenario,
    ticks: u64,
    modify: impl FnOnce(&mut SimParameters),
) -> Simulation {
    let mut sc = scenario.clone();
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    modify(&mut sim.params);
    sim.populate();
    sim.run(ticks);
    sim
}
