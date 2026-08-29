//! Recompute golden baseline hashes (DC-1 CO-2026-001).
//!
//! Mirrors `crates/mindstrata-tests/src/golden_replay.rs` hashing exactly
//! and prints the new JSON line for hand-replacement under `golden/`.

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

fn hash_metrics(metrics: &[f64]) -> u64 {
    let mut hasher = DefaultHasher::new();
    for m in metrics {
        m.to_bits().hash(&mut hasher);
    }
    hasher.finish()
}

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 1000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(1000);
    let ms = sim.metrics_snapshot();
    let metrics = vec![
        ms.avg_hunger,
        ms.avg_thirst,
        ms.avg_fatigue,
        ms.avg_valence,
        ms.avg_joy,
        ms.avg_fear,
        ms.total_grain,
        ms.total_water,
        ms.event_count as f64,
        ms.journal_len as f64,
        ms.agent_count as f64,
    ];
    let metric_hash = hash_metrics(&metrics);
    let summaries = sim.agent_summaries();
    let agent_data: Vec<(f64, f64, f64, f64)> = summaries
        .iter()
        .map(|s| {
            (
                s.hunger.to_f64(),
                s.thirst.to_f64(),
                s.fatigue.to_f64(),
                s.valence.to_f64(),
            )
        })
        .collect();
    let agent_hash = hash_metrics(
        &agent_data
            .iter()
            .flat_map(|(a, b, c, d)| [*a, *b, *c, *d])
            .collect::<Vec<_>>(),
    );
    let event_count_hash = {
        let mut h = DefaultHasher::new();
        ms.event_count.hash(&mut h);
        h.finish()
    };
    println!("{{");
    println!("  \"scenario\": \"riverford_minor\",");
    println!("  \"seed\": 42,");
    println!("  \"ticks\": 1000,");
    println!("  \"metric_hash\": {metric_hash},");
    println!("  \"event_hash\": {event_count_hash},");
    println!("  \"agent_hash\": {agent_hash},");
    println!("  \"agent_count\": {},", ms.agent_count);
    println!("  \"total_grain\": {:.4},", ms.total_grain);
    println!("  \"total_water\": {:.4}", ms.total_water);
    println!("}}");

    // collapse / seed 42 / 4320 ticks (scenario-driven)
    use mindstrata_sim::scenario::Scenario;
    let sc = Scenario::collapse();
    let mut sim2 = Simulation::from_scenario(sc.clone());
    sim2.populate();
    sim2.run(sc.ticks);
    let ms2 = sim2.metrics_snapshot();
    let metrics2 = vec![
        ms2.avg_hunger,
        ms2.avg_thirst,
        ms2.avg_fatigue,
        ms2.avg_valence,
        ms2.avg_joy,
        ms2.avg_fear,
        ms2.total_grain,
        ms2.total_water,
        ms2.event_count as f64,
        ms2.journal_len as f64,
        ms2.agent_count as f64,
    ];
    let metric_hash2 = hash_metrics(&metrics2);
    let summaries2 = sim2.agent_summaries();
    let agent_data2: Vec<(f64, f64, f64, f64)> = summaries2
        .iter()
        .map(|s| {
            (
                s.hunger.to_f64(),
                s.thirst.to_f64(),
                s.fatigue.to_f64(),
                s.valence.to_f64(),
            )
        })
        .collect();
    let agent_hash2 = hash_metrics(
        &agent_data2
            .iter()
            .flat_map(|(a, b, c, d)| [*a, *b, *c, *d])
            .collect::<Vec<_>>(),
    );
    let event_count_hash2 = {
        let mut h = DefaultHasher::new();
        ms2.event_count.hash(&mut h);
        h.finish()
    };
    println!("{{");
    println!("  \"scenario\": \"collapse\",");
    println!("  \"seed\": 42,");
    println!("  \"ticks\": {},", sc.ticks);
    println!("  \"metric_hash\": {metric_hash2},");
    println!("  \"event_hash\": {event_count_hash2},");
    println!("  \"agent_hash\": {agent_hash2},");
    println!("  \"agent_count\": {},", ms2.agent_count);
    println!("  \"total_grain\": {:.4},", ms2.total_grain);
    println!("  \"total_water\": {:.4}", ms2.total_water);
    println!("}}");
}
