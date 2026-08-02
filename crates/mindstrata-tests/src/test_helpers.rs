//! Shared test utilities for Mindstrata integration and snapshot tests.

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
