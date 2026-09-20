//! i339 — sizing the i338 lever: does scaling the house count with N localize the
//! contact graph?
//!
//! i338 proved the quadratic relationship store is a **housing artifact**: eight
//! fixed house sites for every N (and every world size), round-robin assignment,
//! frozen positions ⇒ every agent lives at one of 8 points, so `ceil(N/8)` agents
//! share each cell and contact saturates as co-residency.
//!
//! The counterfactual is cheap to measure because `set_house_count` is opt-in and
//! default-identical (`DEFAULT_HOUSE_COUNT = 8` keeps the generated world
//! byte-identical). This probe asks the two questions that decide whether the
//! lever is worth a behavioural iteration:
//!
//!   1. **Contact degree**: mean partners/agent and the touched-R exponent under
//!      `houses = ceil(N/4)` (one house per ~4 villagers, matching the declared
//!      `SiteKind::House.capacity`) vs the shipped 8.
//!   2. **Occupancy**: how many cells the population occupies, and the near-pair
//!      share — the quantity that drives the store's row count.
//!
//! Caveat (recorded in the evidence doc): a larger village is a *different*
//! village — the world RNG stream shifts downstream of the house loop — so this
//! is a spread experiment, not a re-housing of one fixed run.
//!
//! Run: cargo run --release -p mindstrata-benches --example i339_housing_spread

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::BTreeMap;

struct Leg {
    total: usize,
    touched: usize,
    mean_deg: f64,
    max_deg: u32,
    cells: usize,
    max_co: u32,
    near_share: f64,
}

fn measure(n: u32, world: u32, houses: Option<u32>, ticks: u64) -> Leg {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: ticks,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    });
    if let Some(h) = houses {
        sim.set_house_count(h);
    }
    sim.populate();
    sim.run(ticks);

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
    Leg {
        total,
        touched,
        mean_deg,
        max_deg,
        cells: by_cell.len(),
        max_co: by_cell.values().copied().max().unwrap_or(0),
        near_share: near as f64 / tot.max(1) as f64 * 100.0,
    }
}

fn main() {
    // 5K is enough: i338 showed the contact graph's SHAPE is settled by 2K
    // (touched R 46% at 2K vs 48% at 20K), and the exponent is what matters here.
    let ticks = 5_000u64;
    const BASE_DENSITY: f64 = 48.0 / (32.0 * 32.0);
    println!(
        "i339 — housing spread counterfactual (seed 42, {ticks} ticks, world scaled with N)\n"
    );
    println!(
        "{:>5} {:>7} {:>7} {:>10} {:>11} {:>13} {:>9} {:>11} {:>9} {:>11}",
        "N",
        "world",
        "houses",
        "total R",
        "touched R",
        "partners/ag",
        "max",
        "cells occ.",
        "max co",
        "near share"
    );

    let mut shipped: Vec<(u32, f64)> = Vec::new();
    let mut spread: Vec<(u32, f64)> = Vec::new();
    for n in [24u32, 48, 96] {
        let w = ((n as f64 / BASE_DENSITY).sqrt()).round() as u32;
        for (label, houses) in [("8 (shipped)", None), ("ceil(N/4)", Some((n / 4).max(1)))] {
            let l = measure(n, w, houses, ticks);
            let _ = label;
            println!(
                "{:>5} {:>7} {:>7} {:>10} {:>11} {:>13.1} {:>9} {:>11} {:>9} {:>10.1}%",
                n,
                w,
                if houses.is_none() {
                    "8".to_string()
                } else {
                    format!("{}", (n / 4).max(1))
                },
                l.total,
                format!(
                    "{} ({:.0}%)",
                    l.touched,
                    l.touched as f64 / l.total.max(1) as f64 * 100.0
                ),
                l.mean_deg,
                l.max_deg,
                l.cells,
                l.max_co,
                l.near_share
            );
            if houses.is_none() {
                shipped.push((n, l.touched as f64));
            } else {
                spread.push((n, l.touched as f64));
            }
        }
    }

    // Touched-R exponent across the leg, in N.
    let exp = |rows: &Vec<(u32, f64)>| -> f64 {
        if rows.len() < 2 {
            return f64::NAN;
        }
        let (n0, a) = rows[0];
        let (n1, b) = *rows.last().unwrap();
        (b / a).ln() / (n1 as f64 / n0 as f64).ln()
    };
    println!(
        "\ntouched-R exponent (24 → 96):  shipped {:.3} | ceil(N/4) houses {:.3}",
        exp(&shipped),
        exp(&spread)
    );
    println!(
        "\nreading: an exponent near 1.0 for the spread leg ⇒ co-residency stops growing with N and\n\
         the relationship store can be sparse (the i338 lever is real); an exponent near 2.0 ⇒\n\
         spread does not localize because the perception radius (5) still covers the whole village\n\
         at these world sizes, and the lever is smaller than it looks."
    );
}
