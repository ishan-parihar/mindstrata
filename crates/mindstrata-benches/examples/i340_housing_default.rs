//! i340 — the population-scaled housing default, measured.
//!
//! i339 sized the counterfactual (`ceil(N/4)` houses localize contact: touched-R
//! exponent 1.929 → 0.898) without changing behaviour. i340 ships it as the
//! default: `houses_for_population(N) = max(8, ceil(N/4))`, so N ≤ 32 keeps the
//! historical 8-house village **byte-identically** (which is why the goldens and
//! the calibrated N=12 windows needed no re-anchor) and larger populations spread.
//!
//! This probe measures the shipped default at the charter world size (fixed
//! 32×32) at every N, three legs:
//!
//!   1. **Housing + occupancy** — houses generated, cells occupied, max co-located.
//!   2. **Contact graph** — total R, touched R, mean/max partners, near-pair share,
//!      and the touched-R local exponent (the quantity i338 pinned at 1.872–1.929).
//!   3. **Predictions** — the memory-footprint of a store that only kept contacted
//!      rows, i.e. what the (still-demoted) sparse store would hold *now*.
//!
//! Run: cargo run --release -p mindstrata-benches --example i340_housing_default -- [N...]

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::BTreeMap;

struct Row {
    n: u32,
    houses: usize,
    cells: usize,
    max_co: u32,
    total: usize,
    touched: usize,
    mean_deg: f64,
    max_deg: u32,
    near_share: f64,
}

fn measure(n: u32, ticks: u64) -> Row {
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

    let houses = sim
        .world
        .sites
        .iter()
        .filter(|s| matches!(s.kind, mindstrata_sim::world::SiteKind::House))
        .count();

    let rels = sim.relationships();
    let total = rels.len();
    let touched = rels.iter().filter(|r| r.interaction_count > 0).count();
    let mut out_deg: BTreeMap<u64, u32> = BTreeMap::new();
    for r in rels.iter() {
        if r.interaction_count > 0 {
            *out_deg.entry(r.from.as_u64()).or_insert(0) += 1;
        }
    }
    let mean_deg = out_deg.values().map(|d| *d as f64).sum::<f64>() / n as f64;
    let max_deg = out_deg.values().copied().max().unwrap_or(0);

    let mut by_cell: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    let pos: Vec<(i32, i32)> = sim
        .agents
        .iter()
        .map(|a| (a.position.x, a.position.y))
        .collect();
    for p in pos.iter() {
        *by_cell.entry(*p).or_insert(0) += 1;
    }
    let mut near = 0u32;
    let mut tot = 0u32;
    for i in 0..pos.len() {
        for j in 0..pos.len() {
            if i == j {
                continue;
            }
            tot += 1;
            if (pos[i].0 - pos[j].0).abs() + (pos[i].1 - pos[j].1).abs() <= 5 {
                near += 1;
            }
        }
    }
    Row {
        n,
        houses,
        cells: by_cell.len(),
        max_co: by_cell.values().copied().max().unwrap_or(0),
        total,
        touched,
        mean_deg,
        max_deg,
        near_share: near as f64 / tot.max(1) as f64 * 100.0,
    }
}

fn main() {
    let ticks = 5_000u64;
    let ns: Vec<u32> = {
        let args: Vec<u32> = std::env::args()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
        if args.is_empty() {
            vec![12, 32, 48, 96, 144]
        } else {
            args
        }
    };
    println!("i340 — population-scaled housing, shipped default (seed 42, 32x32, {ticks} ticks)\n");
    println!(
        "{:>5} {:>7} {:>8} {:>9} {:>8} {:>9} {:>11} {:>12} {:>7} {:>11}",
        "N",
        "houses",
        "cells",
        "max co",
        "total R",
        "touched",
        "partners/ag",
        "max partners",
        "near%",
        "kept rows"
    );
    let mut rows = Vec::new();
    for n in ns {
        let r = measure(n, ticks);
        println!(
            "{:>5} {:>7} {:>8} {:>9} {:>8} {:>9} {:>11.1} {:>12} {:>6.1}% {:>11}",
            r.n,
            r.houses,
            r.cells,
            r.max_co,
            r.total,
            format!(
                "{} ({:.0}%)",
                r.touched,
                r.touched as f64 / r.total.max(1) as f64 * 100.0
            ),
            r.mean_deg,
            r.max_deg,
            r.near_share,
            r.touched
        );
        rows.push(r);
    }
    // Local exponent in N across adjacent measured points (both > the 8-house
    // floor, so the comparison is meaningful).
    println!("\nlocal exponent of touched R (adjacent N):");
    for w in rows.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        if a.n <= 32 || b.n <= 32 {
            continue;
        }
        let alpha = (b.touched as f64 / a.touched as f64).ln() / (b.n as f64 / a.n as f64).ln();
        println!("  {:>4} → {:<4} α = {alpha:.3}", a.n, b.n);
    }
    println!(
        "\nbaseline for comparison (i338, 8 fixed houses, 20K ticks): touched-R α 1.872,\n\
         mean partners/agent 6.2 / 25.7 / 47.0 at N=12 / 48 / 96, near share 43–46% flat."
    );
}
