//! i330 — per-pass tick profiler. Attributes the tick cost to a pass by
//! enabling the sim's opt-in `MINDSTRATA_PROFILE_TICK` instrumentation.
//!
//! i335: the instrumentation now ACCUMULATES (name → total ns, samples) and is
//! read back through `Simulation::pass_profile_totals()`, so this probe prints
//! the **mean ns/tick per pass** over the window rather than one tick's sample.
//! A single-tick sample ran ±15% noisy at N=192 — enough to mis-attribute a
//! term, which is the failure mode i294's single log-log fit paid for.
//!
//! Run: cargo run --release -p mindstrata-benches --example i330_pass_profile -- 192

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let n: u32 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(192);
    let warmup: u64 = std::env::args()
        .nth(2)
        .and_then(|s| s.parse().ok())
        .unwrap_or(400);
    let window: u64 = std::env::args()
        .nth(3)
        .and_then(|s| s.parse().ok())
        .unwrap_or(200);
    // Safety: set before any thread reads it; the sim parses it once via
    // `OnceLock` on first tick.
    std::env::set_var("MINDSTRATA_PROFILE_TICK", "1");
    Simulation::pass_profile_reset();
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: warmup + window,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(warmup);
    Simulation::pass_profile_reset();
    let t0 = std::time::Instant::now();
    sim.run(window);
    let us_per_tick = t0.elapsed().as_secs_f64() * 1e6 / window as f64;

    eprintln!("i330 pass profile — N={n}, mean over {window} ticks after {warmup} warmup");
    eprintln!("  whole tick: {us_per_tick:.1} µs/tick\n");
    let rows = Simulation::pass_profile_totals();
    let total: u64 = rows.iter().map(|(_, ns, _)| *ns).sum();
    for (name, ns, samples) in rows {
        let mean_us = ns as f64 / samples.max(1) as f64 / 1000.0;
        eprintln!(
            "PROFILE {:>22} {:>12.1} µs/tick {:>6.1}%",
            name,
            mean_us,
            mean_us * 1000.0 / (us_per_tick * 1000.0) * 100.0
        );
    }
    eprintln!(
        "\n  summed pass means: {:.1} µs/tick (residual = unmarked work)",
        total as f64 / window as f64 / 1000.0
    );
}
