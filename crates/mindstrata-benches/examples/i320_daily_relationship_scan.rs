//! i320 — daily relationship mean-reversion scan: an O(N·R)=O(N³) accident.
//!
//! `systems/cognitive.rs` runs the legacy relationship mean-reversion inside the
//! *per-agent* loop (`for i in 0..agents.len()`), and each agent re-scans the
//! ENTIRE `relationships` matrix filtering `rel.from == i`. That is O(N·R) =
//! O(N³) on every daily boundary tick (every 144 ticks) — the same accident
//! class i294 fixed in the social-support block (which was O(N³) *every* tick).
//!
//! i294's phase leg measured daily ticks "+132% over fast ticks but 0.8%
//! amortized" at N=192 and recorded it as second-order. That amortization is
//! fixed (1/144), but the *absolute* boundary cost is cubic — it is the term
//! that will dominate first when the operating envelope grows past N≈96.
//!
//! This probe measures the boundary excess at N=96 and N=192 so the fix's
//! effect is observable, not asserted.
//!
//! Run: cargo run -p mindstrata-benches --release --example i320_daily_relationship_scan

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::time::Instant;

fn sample(n: u32, ticks: u64, seed: u64) -> (f64, f64) {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let mut daily: Vec<f64> = Vec::new();
    let mut fast: Vec<f64> = Vec::new();
    for t in 1..=ticks {
        let s = Instant::now();
        sim.tick();
        let us = s.elapsed().as_micros() as f64;
        if t % 144 == 0 {
            daily.push(us);
        } else {
            fast.push(us);
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    // Drop the first boundary sample: tick 144 is still cache-cold relative to
    // steady state (i294 method).
    let d = mean(&daily[1.min(daily.len())..]);
    (d, mean(&fast))
}

fn main() {
    let ticks: u64 = 2_000;
    let seed = 42;
    println!("i320 daily relationship mean-reversion scan — release, {ticks} ticks, seed {seed}, world 32×32");
    println!();
    for n in [96_u32, 192] {
        let (d, f) = sample(n, ticks, seed);
        let excess = (d - f) / f * 100.0;
        let amortized = (d - f) * 13.0 / (f * ticks as f64) * 100.0;
        println!(
            "N={n:>3}  fast {f:>9.1} µs/tick   daily-boundary {d:>9.1} µs/tick   excess {excess:+.1}%   amortized share {amortized:.1}%"
        );
    }
    println!();
    println!("If the per-agent matrix scan is the boundary cost, the excess should shrink");
    println!("after hoisting the mean-reversion to one O(R) pass (bit-identical).");
}
