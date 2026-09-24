//! i338 — sizing the contact-driven sparse relationship store BEFORE touching it.
//!
//! i335 established that the store is a complete directed graph (edges =
//! N(N−1)) seeded at populate, and that this completeness — not space — is what
//! makes the tick quadratic. i326 found 46–53% of those rows end a run with
//! `interaction_count == 0`, i.e. never touched.
//!
//! The obvious repair is to keep only contact-driven edges. Before building it,
//! two things must be measured, because each can kill the idea outright:
//!
//!   1. **Touched-R vs total-R by N and horizon.** If contact converges back to
//!      N² (everyone eventually meets everyone), the repair buys only a constant
//!      AND the tick becomes time-dependent. If touched-R grows ~linearly in N,
//!      the repair is real and changes the asymptotics.
//!   2. **Per-agent distinct partners** — that is exactly the length of the
//!      per-tick own-list walks i336/i337 attacked, so it sizes the win in the
//!      passes we already know are hot.
//!
//! Run: cargo run --release -p mindstrata-benches --example i338_sparse_store_sizing

use mindstrata_sim::sim::{SimConfig, Simulation};

/// Baseline density from the i295 charter: N=48 in a 32×32 world.
const BASELINE_DENSITY: f64 = 48.0 / (32.0 * 32.0);

/// Long-horizon contact census at an arbitrary world size. Returns
/// (total R, touched R, mean out-degree, max out-degree).
fn census_at(n: u32, world: u32, ticks: u64) -> (usize, usize, f64, u32) {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: ticks,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(ticks);
    let rels = sim.relationships();
    let total = rels.len();
    let touched = rels.iter().filter(|r| r.interaction_count > 0).count();
    let mut out_deg: std::collections::BTreeMap<u64, u32> = std::collections::BTreeMap::new();
    for r in rels {
        if r.interaction_count > 0 {
            *out_deg.entry(r.from.as_u64()).or_insert(0) += 1;
        }
    }
    let mean_deg = out_deg.values().map(|d| *d as f64).sum::<f64>() / n as f64;
    let max_deg = out_deg.values().copied().max().unwrap_or(0);
    (total, touched, mean_deg, max_deg)
}

/// Constant-density leg: world grows with N, so the number of agents within a
/// fixed perception radius stays constant. This separates the two candidate
/// mechanisms behind the quadratic contact graph:
///   * the fixed 32×32 world making everyone co-located as N rises, or
///   * village-wide mobility closing the contact graph transitively.
fn density_leg() {
    println!("\n=== fixed-DENSITY leg (world scales with N, 20K ticks) ===\n");
    println!(
        "{:>5} {:>6} {:>10} {:>11} {:>7} {:>13} {:>13}",
        "N", "world", "total R", "touched R", "share", "partners/ag", "partners max"
    );
    let mut rows: Vec<(u32, f64, f64)> = Vec::new();
    for n in [24u32, 48, 96] {
        let w = ((n as f64 / BASELINE_DENSITY).sqrt()).round() as u32;
        let (total, touched, mean_deg, max_deg) = census_at(n, w, 20_000);
        println!(
            "{:>5} {:>6} {:>10} {:>11} {:>6.1}% {:>13.1} {:>13}",
            n,
            w,
            total,
            touched,
            touched as f64 / total.max(1) as f64 * 100.0,
            mean_deg,
            max_deg
        );
        rows.push((n, total as f64, touched as f64));
    }
    let alpha = |i: usize| -> f64 {
        let (a, b) = (&rows[0], &rows[2]);
        let (av, bv) = if i == 1 { (a.1, b.1) } else { (a.2, b.2) };
        (bv / av).ln() / (b.0 as f64 / a.0 as f64).ln()
    };
    println!("\n  total R   α (24 → 96) = {:.3}", alpha(1));
    println!("  touched R α (24 → 96) = {:.3}", alpha(2));
    println!(
        "\n  reading: touched α ≈ 1.0 at constant density ⇒ the quadratic contact\n  \
         graph is a FIXED-WORLD artifact and the scale fix is to grow the world\n  \
         with N (config), not to build a sparse store. touched α ≈ 2.0 ⇒ mobility\n  \
         closes the graph transitively regardless of density, and no store shape\n  \
         can fix it."
    );
}

/// How much of the map does one agent actually cover in a run? If agents visit
/// ~every cell, then for any pair the walk is recurrent — they meet — so contact
/// is transitive by construction and no store shape can be sub-quadratic. That
/// would make the lever bounded mobility (activity spaces), not storage.
fn mobility_leg() {
    println!("\n=== mobility leg (distinct cells visited per agent, 20K ticks) ===\n");
    println!(
        "{:>5} {:>6} {:>9} {:>13} {:>13} {:>11}",
        "N", "world", "cells", "visited mean", "visited min", "coverage"
    );
    for n in [24u32, 48, 96] {
        let w = ((n as f64 / BASELINE_DENSITY).sqrt()).round() as u32;
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 20_000,
            world_width: w,
            world_height: w,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        // Sampling every tick is the honest measure; the walk is slow enough
        // that a coarser sample would under-report coverage.
        let mut seen: Vec<std::collections::HashSet<(i32, i32)>> =
            vec![std::collections::HashSet::new(); sim.agents.len()];
        for _ in 0..20_000 {
            sim.run(1);
            for (i, a) in sim.agents.iter().enumerate() {
                if i < seen.len() {
                    seen[i].insert((a.position.x, a.position.y));
                }
            }
        }
        let counts: Vec<usize> = seen.iter().map(std::collections::HashSet::len).collect();
        let cells = (w * w) as f64;
        let mean = counts.iter().sum::<usize>() as f64 / counts.len().max(1) as f64;
        let min = counts.iter().copied().min().unwrap_or(0);
        println!(
            "{:>5} {:>6} {:>9} {:>13.0} {:>13} {:>10.1}%",
            n,
            w,
            w * w,
            mean,
            min,
            mean / cells * 100.0
        );
    }
    println!(
        "\n  coverage ≈ 100% ⇒ agents roam the whole map, contact is recurrent for every\n  \
         pair, and the quadratic store is intrinsic to UNBOUNDED MOBILITY — the\n  \
         lever is bounded activity spaces (behavioural), not a store refactor."
    );
}

fn main() {
    println!("i338 — contact-driven sparse-store sizing (seed 42, 32x32)\n");
    println!(
        "{:>5} {:>7} {:>10} {:>11} {:>8} {:>12} {:>13} {:>10}",
        "N", "ticks", "total R", "touched R", "share", "partners/ag", "partners max", "α(touch)"
    );

    let mut cells: Vec<(u32, u64, f64, f64)> = Vec::new();
    for n in [12u32, 48, 96] {
        for ticks in [2_000u64, 20_000] {
            let mut sim = Simulation::new(SimConfig {
                seed: 42,
                max_ticks: ticks,
                world_width: 32,
                world_height: 32,
                num_agents: n,
                snapshot_interval: None,
            });
            sim.populate();
            sim.run(ticks);

            let rels = sim.relationships();
            let total = rels.len();
            let touched = rels.iter().filter(|r| r.interaction_count > 0).count();

            // Distinct partners per agent — a contact-driven store's own-list
            // length. Counted from the touched edges (out-degree of the
            // contact graph). AgentId values are not guaranteed dense 0..N−1
            // (deaths/births move the high-water mark), so key by id.
            let mut out_deg: std::collections::BTreeMap<u64, u32> =
                std::collections::BTreeMap::new();
            for r in rels {
                if r.interaction_count > 0 {
                    *out_deg.entry(r.from.as_u64()).or_insert(0) += 1;
                }
            }
            let mean_deg = out_deg.values().map(|d| *d as f64).sum::<f64>() / n as f64;
            let max_deg = out_deg.values().copied().max().unwrap_or(0);

            println!(
                "{:>5} {:>7} {:>10} {:>11} {:>7.1}% {:>12.1} {:>13} {:>10}",
                n,
                ticks,
                total,
                touched,
                touched as f64 / total.max(1) as f64 * 100.0,
                mean_deg,
                max_deg,
                "-"
            );
            cells.push((n, ticks, total as f64, touched as f64));
        }
    }

    let alpha = |n0: u32, n1: u32, col: usize| -> Option<f64> {
        let a = cells.iter().find(|c| c.0 == n0 && c.1 == 20_000)?;
        let b = cells.iter().find(|c| c.0 == n1 && c.1 == 20_000)?;
        let (av, bv) = match col {
            2 => (a.2, b.2),
            _ => (a.3, b.3),
        };
        Some((bv / av).ln() / ((n1 as f64 / n0 as f64).ln()))
    };

    println!("\nlocal exponent 48 → 96 @20K:");
    if let (Some(t), Some(s)) = (alpha(48, 96, 2), alpha(48, 96, 3)) {
        println!("  total R   α = {t:.3}   (the dense store: N(N−1))");
        println!("  touched R α = {s:.3}");
    }
    println!(
        "\nreading: touched α ≈ 2.0 ⇒ contact does NOT localize (everyone eventually meets\n\
         everyone): the sparse store buys only a constant, and tick cost becomes\n\
         time-dependent. touched α ≈ 1.0 ⇒ contact localizes, the repair changes the\n\
         asymptotics, and it is worth building."
    );

    density_leg();
    mobility_leg();
}
