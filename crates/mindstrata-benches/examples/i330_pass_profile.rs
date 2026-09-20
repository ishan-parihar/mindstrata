//! i330 — per-pass tick profiler. Attributes the superlinear tick cost to a
//! pass by enabling the sim's opt-in `MINDSTRATA_PROFILE_TICK` instrumentation
//! and printing per-pass nanoseconds at the chosen N.
//!
//! Run: cargo run --release -p mindstrata-benches --example i330_pass_profile -- 192
//! (the probe sets the env var itself; tick 400 is used because it is a `deca`
//! boundary at every N, so the samples are comparable.)

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let n: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(192);
    // Safety: set before any thread reads it; the sim parses it once via
    // `OnceLock` on first tick.
    std::env::set_var("MINDSTRATA_PROFILE_TICK", "400");
    eprintln!("i330 pass profile — N={n} (tick 400)");
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 400,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(400);
    eprintln!("done");
}
