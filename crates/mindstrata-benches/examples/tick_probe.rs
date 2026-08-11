//! Wall-clock timing probe — measures debug-mode tick cost to calibrate the
//! benchmark regression gate budget (used by the CI smoke gate and the
//! `tick_throughput_regression_gate` integration test).
//!
//! Run with: `cargo run -p mindstrata-benches --example tick_probe --release`

use mindstrata_sim::{sim::SimConfig, Simulation};
use std::time::Instant;

fn main() {
    for (agents, ticks) in [(12u32, 2000u64), (24u32, 2000u64), (24u32, 5000u64)] {
        let config = SimConfig {
            seed: 42,
            max_ticks: ticks,
            world_width: 16,
            world_height: 16,
            num_agents: agents,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let start = Instant::now();
        sim.run(ticks);
        let elapsed = start.elapsed();
        println!(
            "agents={agents} ticks={ticks} elapsed={elapsed:?} ({:.2} ticks/sec)",
            ticks as f64 / elapsed.as_secs_f64()
        );
    }
}
