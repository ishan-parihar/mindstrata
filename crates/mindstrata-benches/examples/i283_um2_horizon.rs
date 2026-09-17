//! Iter-283 probe — UM-2 unify review at the DESIGNED cultural horizon.
//!
//! i278 measured the 20K gate: generated jaccard 0.369, 12/15 pairs
//! disjoint, but the bucket census was 21/21 Safety — Identity (stage 2
//! ~44K, i279) and Relational (~80K, i274) have not crossed their first
//! genesis threshold at 20K. This probe asks the actual UM-2 question:
//! do the slow buckets START generating at their designed horizons, and
//! does disjointness improve?
//!
//! Horizon: 100K — the i283 corrected model (per-catalyst pressure is
//! 1/n_agents, per-event press (1/12)×0.05 = 0.00417) puts Relational
//! stage-2 at ~54K and Identity at ~74K; 100K is past both.

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::HashSet;

fn run(seed: u64, horizon: u64) -> (HashSet<String>, HashSet<String>) {
    let config = SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(horizon);
    let mut seeded = HashSet::new();
    let mut generated = HashSet::new();
    for m in &sim.meme_registry.memes {
        if m.description.contains("[genesis:") {
            generated.insert(m.description.clone());
        } else {
            seeded.insert(m.description.clone());
        }
    }
    (seeded, generated)
}

fn jaccard(a: &HashSet<String>, b: &HashSet<String>) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.intersection(b).count();
    inter as f64 / (a.len() + b.len() - inter) as f64
}

fn main() {
    let horizon = 100_000u64;
    let seeds = [42_u64, 43, 44];
    let mut all_generated = Vec::new();
    for s in seeds {
        let (seeded, generated) = run(s, horizon);
        let buckets: HashSet<String> = generated
            .iter()
            .filter_map(|d| {
                d.split("[genesis:")
                    .nth(1)
                    .map(|rest| rest.split(':').next().unwrap_or("").to_string())
            })
            .collect();
        println!(
            "seed {s}: seeded={} generated={} buckets={:?}",
            seeded.len(),
            generated.len(),
            buckets
        );
        all_generated.push(generated);
    }
    // Bucket census across all seeds.
    let mut census: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for g in &all_generated {
        for d in g {
            if let Some(rest) = d.split("[genesis:").nth(1) {
                *census
                    .entry(rest.split(':').next().unwrap_or("").to_string())
                    .or_default() += 1;
            }
        }
    }
    println!("BUCKET CENSUS @50K: {census:?}");
    // Cross-seed jaccard.
    let mut jac = Vec::new();
    for i in 0..seeds.len() {
        for j in (i + 1)..seeds.len() {
            let jv = jaccard(&all_generated[i], &all_generated[j]);
            jac.push(jv);
            println!("  {}-{}: generated={:.3}", seeds[i], seeds[j], jv);
        }
    }
    let mean = jac.iter().sum::<f64>() / jac.len() as f64;
    println!("GENERATED MEAN JACCARD @50K: {mean:.3} (20K was 0.369)");
}
