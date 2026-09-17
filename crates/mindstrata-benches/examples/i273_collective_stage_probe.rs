//! Iter-273 probe — Era IV collective activation (PLAN_DC2 §4 ledger horizon).
//!
//! Before wiring the first collective-line *content* consumer (substrate §5:
//! "causal domain selection weighted by collective line stages", replacing
//! `seed_initial_memes`), measure what the collective field actually does at
//! calibration horizons:
//!
//! 1. Per-bucket mean stage/fulfillment at 2K (golden horizon) and 20K ticks.
//! 2. Stage differentiation across the 29 lines (is there signal to weight by?)
//! 3. Catalyst-bucket pressure cadence (which buckets ever saturate?)
//!
//! Zero-Blast reference: N=12, seed 42, same harness family as i266/i270.
//! Doctrine §4.3: probe equilibrium values, not just assertions.

use mindstrata_development::collective::{CollectiveBucket, CollectiveField};
use mindstrata_sim::sim::{SimConfig, Simulation};

fn bucket_name(b: CollectiveBucket) -> &'static str {
    match b {
        CollectiveBucket::Relational => "Relational",
        CollectiveBucket::Safety => "Safety",
        CollectiveBucket::Identity => "Identity",
        CollectiveBucket::Meaning => "Meaning",
    }
}

fn all_buckets() -> [CollectiveBucket; 4] {
    [
        CollectiveBucket::Relational,
        CollectiveBucket::Safety,
        CollectiveBucket::Identity,
        CollectiveBucket::Meaning,
    ]
}

fn dump(field: &CollectiveField, label: &str) {
    println!("--- {label} ---");
    let slugs = CollectiveField::line_slugs();
    let mut max_stage = 0.0_f64;
    let mut max_line = "";
    for bucket in all_buckets() {
        let mut stages = Vec::new();
        let mut fills = Vec::new();
        for (idx, line) in field.lines.iter().enumerate() {
            stages.push(line.stage);
            fills.push(line.fulfillment);
            if line.stage > max_stage {
                max_stage = line.stage;
                max_line = slugs.get(idx).map(|s| s.slug()).unwrap_or("?");
            }
        }
        println!(
            "  {:>11}: mean_stage {:.4}  mean_fill {:.4}  (bucket read {:.4})",
            bucket_name(bucket),
            stages.iter().sum::<f64>() / stages.len() as f64,
            fills.iter().sum::<f64>() / fills.len() as f64,
            field.mean_fulfillment_for_bucket(bucket),
        );
    }
    // Per-line spread (differentiation signal for domain weighting).
    let all_stages: Vec<f64> = field.lines.iter().map(|l| l.stage).collect();
    let mean = all_stages.iter().sum::<f64>() / all_stages.len() as f64;
    let var = all_stages.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / all_stages.len() as f64;
    println!(
        "  SPREAD: mean_stage {:.4}  sigma {:.4}  max {:.4} ({max_line})  CV {:.3}",
        mean,
        var.sqrt(),
        max_stage,
        if mean > 0.0 { var.sqrt() / mean } else { 0.0 },
    );
}

fn main() {
    for horizon in [2_000_u64, 20_000] {
        let config = SimConfig {
            seed: 42,
            max_ticks: horizon,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(horizon);
        dump(
            &sim.collective_field,
            &format!("seed42 N=12 ticks={horizon}"),
        );
        // Era IV genesis observability (i273): how much culture did the
        // village generate from its own collective development?
        let genesis: Vec<&str> = sim
            .meme_registry
            .memes
            .iter()
            .filter(|m| m.description.contains("[genesis:"))
            .map(|m| m.description.as_str())
            .collect();
        println!(
            "  GENESIS: {} generated / {} total memes",
            genesis.len(),
            sim.meme_registry.memes.len()
        );
        for g in &genesis {
            println!("    · {g}");
        }
    }
}
