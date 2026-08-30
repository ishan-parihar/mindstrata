//! Action-distribution baseline probe (DC-2 prep).
//!
//! Measures the per-action frequency across 12 seeds × 2000 ticks.
//! This is the baseline for the Era III lite polarity-bias
//! implementation: any polarity-bias coefficient must shift the
//! social-action rate by less than the per-seed natural variance
//! (Iter-263 H5 lesson: don't reshape without coordinated sweep).
//!
//! Run: cargo run --release -p mindstrata-benches --example i282_action_distribution_baseline

use mindstrata_sim::actions::ActionKind;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;
use std::collections::HashMap;

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 2000;

fn main() {
    let mut all_action_counts: HashMap<String, usize> = HashMap::new();
    let mut per_seed_counts: Vec<HashMap<String, usize>> = Vec::new();

    for &seed in &SEEDS {
        let config = SimConfig {
            seed,
            max_ticks: TICKS,
            num_agents: 12,
            snapshot_interval: None,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(TICKS);

        let mut counts: HashMap<String, usize> = HashMap::new();
        for agent in &sim.agents {
            let label = format!("{:?}", agent.current_action);
            *counts.entry(label).or_insert(0) += 1;
        }
        for (k, v) in &counts {
            *all_action_counts.entry(k.clone()).or_insert(0) += v;
        }
        per_seed_counts.push(counts);
    }

    let total: usize = all_action_counts.values().sum();
    println!("baseline_action_total={total}");
    let mut sorted: Vec<(&String, &usize)> = all_action_counts.iter().collect();
    sorted.sort_by(|a, b| b.1.cmp(a.1));
    for (action, count) in sorted {
        let pct = 100.0 * *count as f64 / total as f64;
        println!("action={action} count={count} pct={pct:.2}");
    }

    // Per-seed social-action rate (Talk, Console, Mourn, Greet, etc.)
    // to measure natural variance. A polarity bias must shift this
    // rate by less than the stddev across seeds.
    let social_labels: Vec<&str> = vec![
        "Talk",
        "Console",
        "Mourn",
        "Greet",
        "Trade",
        "Worship",
        "Socialize",
        "Romance",
        "Play",
        "Care",
        "TellStory",
        "Ritual",
    ];
    let social_rates: Vec<f64> = per_seed_counts
        .iter()
        .map(|counts| {
            let social: usize = social_labels
                .iter()
                .map(|l| counts.get(*l).copied().unwrap_or(0))
                .sum();
            social as f64 / 12.0
        })
        .collect();
    let mean: f64 = social_rates.iter().sum::<f64>() / social_rates.len() as f64;
    let var: f64 =
        social_rates.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / social_rates.len() as f64;
    let stddev = var.sqrt();
    println!(
        "social_action_rate mean={mean:.4} stddev={stddev:.4} cv={:.3}",
        stddev / mean.max(1e-9)
    );
    // Per-seed social rate (so any future bias probe can compare directly).
    for (i, rate) in social_rates.iter().enumerate() {
        println!("seed={seed} social_rate={rate:.4}", seed = SEEDS[i]);
    }
    // Suppress unused import warning for ActionKind.
    let _ = std::mem::discriminant(&ActionKind::Idle);
}
