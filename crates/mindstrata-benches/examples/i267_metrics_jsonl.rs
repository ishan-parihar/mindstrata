//! Metrics JSONL emission standard (DC-1 task 2.14).
//!
//! Emits one compact `MetricsSnapshot` object per line keyed by tick, using
//! the same serde representation the zero-blast contract already trusts
//! (behavioral_delta byte-identity). Field order follows struct declaration,
//! so lines are directly diffable across runs.
//!
//! Run: cargo run --release -p mindstrata-benches --example i267_metrics_jsonl -- <out.jsonl> [ticks]

use mindstrata_sim::sim::{MetricsSnapshot, SimConfig};
use mindstrata_sim::Simulation;

fn main() -> Result<(), String> {
    let args: Vec<String> = std::env::args().collect();
    let out_path = args
        .get(1)
        .ok_or("usage: i267_metrics_jsonl <out.jsonl> [ticks=2000]")?
        .clone();
    let ticks: u64 = args
        .get(2)
        .map(|s| s.parse().unwrap_or(2000))
        .unwrap_or(2000);

    let config = SimConfig {
        seed: 42,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    };
    let mut sim = Simulation::new(config);
    sim.populate();

    let mut out = String::new();
    for t in 0..ticks {
        sim.tick();
        // Per-tick cadence via the pure read-side snapshot (not the 10-tick
        // metric_history ring), so streams are complete and comparable.
        if t % 10 == 0 {
            let snap: MetricsSnapshot = sim.metrics_snapshot();
            let line = serde_json::to_string(&snap)
                .map_err(|e| format!("non-finite metrics at tick {t}: {e}"))?;
            out.push_str(&line);
            out.push('\n');
        }
    }
    std::fs::write(&out_path, out).map_err(|e| format!("write {out_path}: {e}"))?;
    println!("wrote {out_path} rows={}", ticks / 10);
    Ok(())
}
