//! Probe: tick-rate and memory snapshot for PLATFORM budget table v1 (task 3.16).
//!
//! Measures release-mode ticks-per-second at N=12 (UM-1 baseline) and N=48
//! (cap projection) over 2K and 10K horizons, plus peak RSS proxy via
//! allocator-agnostic footprint (agent/snapshot byte size). Prints stable
//! `key=value` rows for the budget table's measured basis.
//!
//! Run: cargo run --release -p mindstrata-benches --example i270_perf_snapshot

use std::time::Instant;

use mindstrata_sim::{sim::SimConfig, Simulation};

fn measure(num_agents: u32, ticks: u64, seed: u64) -> (f64, f64) {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        num_agents,
        world_width: 16,
        world_height: 16,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let start = Instant::now();
    sim.run(ticks);
    let elapsed = start.elapsed().as_secs_f64();
    let tps = ticks as f64 / elapsed.max(1e-9);
    (elapsed, tps)
}

fn main() {
    // Three runs per configuration to show noise envelope (criterion does
    // the rigorous version; this is the budget-table's "measured basis" seed).
    for n in [12u32, 48] {
        for horizon in [2000u64, 10_000] {
            let mut tps_vals = Vec::new();
            for seed in [42u64, 123, 999] {
                let (elapsed, tps) = measure(n, horizon, seed);
                tps_vals.push(tps);
                println!(
                    "n={} horizon={} seed={} elapsed_s={:.3} tps={:.0}",
                    n, horizon, seed, elapsed, tps
                );
            }
            let mean = tps_vals.iter().sum::<f64>() / tps_vals.len() as f64;
            let min = tps_vals.iter().cloned().fold(f64::INFINITY, f64::min);
            let max = tps_vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
            println!(
                "summary n={} horizon={} mean_tps={:.0} min_tps={:.0} max_tps={:.0}",
                n, horizon, mean, min, max
            );
        }
    }
    // Memory proxy: snapshot size at each population (serde_json length).
    for n in [12u32, 24, 48] {
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            num_agents: n,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(500);
        let snap = sim.save_snapshot();
        let json = serde_json::to_string(&snap).unwrap_or_default();
        println!("snapshot_bytes n={} bytes={}", n, json.len());
    }
}
