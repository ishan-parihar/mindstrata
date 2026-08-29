//! Golden baseline regen (DC-1 CO-2026-001).
//!
//! Runs the calm `riverford_minor` config and the `collapse` scenario at
//! their golden-tick horizons, prints the new metric_hash, event_hash,
//! agent_hash, agent_count, total_grain, total_water for both. The new
//! JSONs replace the existing baselines under `golden/`.

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    // riverford_minor / seed 42 / 1000 ticks
    let config = SimConfig {
        seed: 42,
        max_ticks: 1000,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(1000);
    let ms = sim.metrics_snapshot();
    println!(
        "riverford_minor seed=42 ticks=1000 agent_count={} total_grain={:.4} total_water={:.4}",
        ms.agent_count, ms.total_grain, ms.total_water
    );

    // collapse / seed 42 / 4320 ticks
    let sc = Scenario::collapse();
    let mut sim2 = Simulation::from_scenario(sc.clone());
    sim2.populate();
    sim2.run(sc.ticks);
    let ms2 = sim2.metrics_snapshot();
    println!(
        "collapse seed=42 ticks={} agent_count={} total_grain={:.4} total_water={:.4}",
        sc.ticks, ms2.agent_count, ms2.total_grain, ms2.total_water
    );
}
