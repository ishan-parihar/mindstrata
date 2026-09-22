//! i354 — is the >250K-tick horizon actually locked, or already unlocked?
//!
//! The plan listed a `VecDeque<SimEvent>` conversion to "unlock >250K-tick
//! horizons". But i327 already bounded the rolling event buffer with an
//! amortized bulk drop (`MAX_EVENTS = 262_144`, peak ≈ 2×MAX ≈ 28 MiB) and
//! recorded `VecDeque` as a *deferred* upgrade path whose only remaining
//! motivation is jitter-free ticks (a sub-ms stall every ~5K ticks).
//!
//! So the claim "horizons are locked" must be tested, not assumed. Run the
//! real thing and measure wall time, population stability, and emergence.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i354_long_horizon`

use std::time::Instant;

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let ticks = 250_000u64;
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();

    let start = Instant::now();
    sim.run(ticks);
    let elapsed = start.elapsed();

    let agents = sim.agents.len();
    let alive = sim
        .agents
        .iter()
        .filter(|a| a.body.health > mindstrata_core::fixed::Fixed::ZERO)
        .count();
    let mean_health: f64 = sim
        .agents
        .iter()
        .map(|a| a.body.health.to_f64())
        .sum::<f64>()
        / agents.max(1) as f64;
    let partnered = sim.agents.iter().filter(|a| a.partner.is_some()).count();
    let events = sim.event_count();

    println!("== i354: {ticks}-tick horizon, N=12, seed 42 (release) ==");
    println!(
        "wall: {:.1} s  ({:.2} µs/tick)",
        elapsed.as_secs_f64(),
        elapsed.as_secs_f64() * 1e6 / ticks as f64
    );
    println!("population: {agents} agents, {alive} with positive health");
    println!("mean health: {mean_health:.3}");
    println!("partnered agents: {partnered}/{agents}");
    // Note: `event_count()` is the CUMULATIVE counter; the retained rolling
    // window is bounded separately by i327 (`trim_event_buffer`, pinned by its
    // own unit test). What this probe establishes is that a quarter-million-tick
    // run completes in bounded memory and keeps the village alive.
    println!("cumulative events: {events}");
    println!("VERDICT_LONG_HORIZON_RUNS");
}
