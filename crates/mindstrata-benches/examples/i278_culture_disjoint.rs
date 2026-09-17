//! Iter-278 probe — UM-2 GATE: seed-disjoint cultures (PLAN_DC2 §6.3).
//!
//! AP3 04-waves Era III exit gate: "seed-disjoint cultures in 20K-tick probe".
//! Method: run the 12-seed family at 20K, collect each village's LIVE meme
//! roster (seeded vocabulary + generated), and compute cross-seed jaccard on
//! the meme description sets. The gate compares GENERATED items specifically:
//! the seeded founding vocabulary is shared by construction (same founder
//! bands — audit H5), so the disjointness signal must come from genesis.
//!
//! Protocol (i284 recipe): two seeds, same founder bands, different event
//! histories → disjoint meme rosters by 20K. We measure:
//!   1. per-seed generated-roster size and content-class mix,
//!   2. cross-seed jaccard of generated sets (gate: low overlap),
//!   3. the seeded-pool jaccard as the CONTROL (should be ~1.0 — same seeds
//!      of culture — proving divergence comes from generation, not vocabulary).

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::HashSet;

fn run(seed: u64) -> (HashSet<String>, HashSet<String>) {
    let config = SimConfig {
        seed,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(20_000);
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
    let union = a.len() + b.len() - inter;
    inter as f64 / union as f64
}

fn main() {
    let seeds = [42_u64, 43, 44, 45, 46, 47];
    let mut all_seeded = Vec::new();
    let mut all_generated = Vec::new();
    for s in seeds {
        let (seeded, generated) = run(s);
        println!(
            "seed {s}: seeded={} generated={}",
            seeded.len(),
            generated.len()
        );
        for g in &generated {
            println!("    · {g}");
        }
        all_seeded.push(seeded);
        all_generated.push(generated);
    }

    // Pairwise jaccard across all seed pairs.
    let n = seeds.len();
    let mut gen_j = Vec::new();
    let mut seed_j = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            gen_j.push(jaccard(&all_generated[i], &all_generated[j]));
            seed_j.push(jaccard(&all_seeded[i], &all_seeded[j]));
        }
    }
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    println!(
        "CROSS-SEED JACCARD: generated mean={:.3} ({} pairs)  seeded-control mean={:.3}",
        mean(&gen_j),
        gen_j.len(),
        mean(&seed_j)
    );
    // Gate: generated cultures diverge (low overlap) while the seeded control
    // stays high (shared founding vocabulary). Print per-pair for the record.
    let mut k = 0;
    for i in 0..n {
        for j in (i + 1)..n {
            println!(
                "  {}-{}: generated={:.3} seeded={:.3}",
                seeds[i], seeds[j], gen_j[k], seed_j[k]
            );
            k += 1;
        }
    }
}
