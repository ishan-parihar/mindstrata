//! i352 — size the marriage-formation pass before touching it.
//!
//! Two questions, both must be answered before any edit (§2.2 probe-first):
//!
//! 1. **Cost.** `tick_marriage_formation` runs EVERY tick (no phase guard,
//!    unlike birth mechanics) and does two `self.relationships.iter().find(..)`
//!    linear scans per candidate pair inside its `for i { for j { .. } }`
//!    loop. Rows R = N(N−1), so that is O(N² pairs × R) = **O(N³) per tick**.
//!    Is it actually material at the charter envelope?
//! 2. **Semantics.** The pass reads the legacy v1 `relationships` matrix, not
//!    the honest per-agent v2 store (i351 ledger item 4). How far apart are
//!    the two stores in vivo — i.e. how big is the behavioural risk if a
//!    future iteration migrates the read?
//!
//! Run: `cargo run --release -p mindstrata-benches --example i352_marriage_scan`

use mindstrata_sim::sim::{SimConfig, Simulation};

fn build(n: usize, seed: u64, ticks: u64) -> Simulation {
    let side: u32 = if n <= 48 { 32 } else { 46 }; // §density law (i344): ~21 cells/agent
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: side,
        world_height: side,
        num_agents: n as u32,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

/// Leg A — per-pass profile at the two envelope sizes.
fn cost_leg() {
    for n in [48usize, 96] {
        Simulation::pass_profile_reset();
        let mut sim = build(n, 42, 2000);
        sim.run(2000);
        let totals = Simulation::pass_profile_totals();
        let grand: u64 = totals.iter().map(|(_, t, _)| *t).sum();
        println!("\n== Leg A: pass profile, N={n}, 2000 ticks ==");
        println!("{:<28} {:>12} {:>8}", "pass", "total µs", "share");
        for (name, total, _samples) in totals.iter().take(8) {
            println!(
                "{:<28} {:>12.1} {:>7.1}%",
                name,
                *total as f64 / 1000.0,
                100.0 * *total as f64 / grand.max(1) as f64
            );
        }
        // Marriage is keyed "marriage"; the preceding mark is "pre_marriage".
        if let Some((_, t, s)) = totals.iter().find(|(n, _, _)| *n == "marriage") {
            println!(
                "  → marriage pass: {:.1} µs total over {} ticks = {:.2} µs/tick",
                *t as f64 / 1000.0,
                s,
                *t as f64 / *s.max(&1) as f64 / 1000.0
            );
        }
    }
}

/// Leg B — v1 matrix vs v2 store, per pair.
fn divergence_leg() {
    println!("\n== Leg B: v1 `relationships` vs v2 `relationship_v2s` ==");
    println!(
        "{:<6} {:<5} {:>6} {:>9} {:>9} {:>9} {:>9} {:>8}",
        "seed", "N", "pairs", "dTrust50", "dTrust90", "dAff50", "dAff90", ">0.01"
    );
    for seed in [42u64, 7, 43] {
        for n in [12usize, 48] {
            let mut sim = build(n, seed, 5000);
            sim.run(5000);
            let mut dtr: Vec<f64> = Vec::new();
            let mut daf: Vec<f64> = Vec::new();
            let mut material = 0usize;
            let mut pairs = 0usize;
            for r in sim.relationships.iter() {
                let from = r.from.as_u64() as usize;
                let to = r.to.as_u64() as usize;
                if from >= n || to >= n || from == to {
                    continue;
                }
                let pos = if to > from { to - 1 } else { to };
                let Some(v2) = sim.agents[from].relationship_v2s.get(pos) else {
                    continue;
                };
                pairs += 1;
                let dt = (r.trust - v2.trust).to_f64().abs();
                let da = (r.affection - v2.affection).to_f64().abs();
                if dt > 0.01 || da > 0.01 {
                    material += 1;
                }
                dtr.push(dt);
                daf.push(da);
            }
            dtr.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            daf.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p = |v: &[f64], q: f64| -> f64 {
                if v.is_empty() {
                    0.0
                } else {
                    v[(q * (v.len() - 1) as f64) as usize]
                }
            };
            println!(
                "{:<6} {:<5} {:>6} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>8}",
                seed,
                n,
                pairs,
                p(&dtr, 0.5),
                p(&dtr, 0.9),
                p(&daf, 0.5),
                p(&daf, 0.9),
                material
            );
        }
    }
}

fn main() {
    // Opt-in per-pass accumulation (i335). Set before the first tick.
    std::env::set_var("MINDSTRATA_PROFILE_TICK", "1");
    cost_leg();
    divergence_leg();
}
