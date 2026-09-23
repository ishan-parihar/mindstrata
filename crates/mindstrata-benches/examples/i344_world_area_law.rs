//! i344 — sizing A9: the world's area must scale with the population.
//!
//! i340 closed the *housing* half of the Ω(N²) story (`ceil(N/4)` houses, N ≤ 32
//! byte-identical) and measured the remaining limit: at the charter's **fixed
//! 32×32** world the contact graph re-saturates past N≈144 (touched-R α 2.654,
//! the best-connected agent has met everyone) because 36 houses pack onto the
//! same ring. i340 recommended defining the envelope at **constant density** and
//! explicitly left it to a charter decision.
//!
//! This probe supplies the evidence for that decision. It runs the same
//! population at two world policies and measures, for each:
//!
//!   1. **Spatial density** — cells per agent, near-pair share, max co-located.
//!   2. **Contact volume** — Σ `interaction_count` (cumulative pair contacts),
//!      per-agent volume, contacted-row share, mean partners.
//!   3. **Tick cost** — µs/tick and its local log-log exponent, plus the
//!      interaction-driven pass marks (`social_pass`, `appraisal`).
//!   4. **Liveness** — population, avg health, avg stress (does a bigger world
//!      starve or isolate the village?).
//!
//! The density law is anchored on the simulator's own two calibrated points —
//! N=12 in 16×16 and N=48 in 32×32 — which both give **21.33 cells/agent**, so
//! the law reproduces both exactly and holds that density above.
//!
//! Run: cargo run --release -p mindstrata-benches --example i344_world_area_law -- [N...]

use std::time::Instant;

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::BTreeMap;

const SEED: u64 = 42;
const WARMUP: u64 = 50;
const TICKS: u64 = 300;

// i392: the law this probe measured is now **the** world-area policy and lives
// in the world generator (`world_side_for_population`, anchored on the two
// calibrated points). Do not re-implement it here.
use mindstrata_sim::world_gen::{
    world_side_for_population as side_for_population, CELLS_PER_AGENT,
};

struct Row {
    n: u32,
    world: u32,
    us_per_tick: f64,
    rows: usize,
    contacted: usize,
    mean_deg: f64,
    max_deg: u32,
    near_share: f64,
    volume: u64,
    max_co: u32,
    social_us: f64,
    appraisal_us: f64,
}

fn measure(n: u32, world: u32) -> Row {
    let mut sim = Simulation::new(SimConfig {
        seed: SEED,
        max_ticks: WARMUP + TICKS,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(WARMUP);
    Simulation::pass_profile_reset();
    let t0 = Instant::now();
    sim.run(TICKS);
    let us_per_tick = t0.elapsed().as_secs_f64() * 1e6 / TICKS as f64;

    let marks = Simulation::pass_profile_totals();
    let mark_us = |name: &str| -> f64 {
        marks
            .iter()
            .filter(|(m, _, _)| *m == name)
            .map(|(_, ns, _)| *ns as f64 / TICKS as f64 / 1000.0)
            .sum()
    };

    let rels = sim.relationships();
    let rows = rels.len();
    let mut out_deg: BTreeMap<u64, u32> = BTreeMap::new();
    let mut volume = 0u64;
    for r in rels.iter() {
        volume += u64::from(r.interaction_count);
        if r.interaction_count > 0 {
            *out_deg.entry(r.from.as_u64()).or_insert(0) += 1;
        }
    }
    let contacted = out_deg.values().map(|d| *d as u64).sum::<u64>() as usize;
    let mean_deg = out_deg.values().map(|d| f64::from(*d)).sum::<f64>() / f64::from(n);
    let max_deg = out_deg.values().copied().max().unwrap_or(0);

    let pos: Vec<(i32, i32)> = sim
        .agents
        .iter()
        .map(|a| (a.position.x, a.position.y))
        .collect();
    let mut by_cell: BTreeMap<(i32, i32), u32> = BTreeMap::new();
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
        world,
        us_per_tick,
        rows,
        contacted,
        mean_deg,
        max_deg,
        near_share: f64::from(near) / f64::from(tot.max(1)) * 100.0,
        volume,
        max_co: by_cell.values().copied().max().unwrap_or(0),
        social_us: mark_us("social_pass"),
        appraisal_us: mark_us("appraisal"),
    }
}

fn print_leg(label: &str, rows: &[Row]) {
    println!("\n=== {label} ===\n");
    println!(
        "{:>5} {:>6} {:>8} {:>9} {:>10} {:>12} {:>9} {:>7} {:>9} {:>11}",
        "N",
        "world",
        "µs/tick",
        "cells/ag",
        "max co",
        "near share",
        "rows",
        "contactd",
        "volume/ag",
        "partners/ag"
    );
    for r in rows {
        println!(
            "{:>5} {:>6} {:>8.1} {:>9.1} {:>10} {:>11.1}% {:>12} {:>9} {:>7.0} {:>11.1}",
            r.n,
            r.world,
            r.us_per_tick,
            f64::from(r.world * r.world) / f64::from(r.n),
            r.max_co,
            r.near_share,
            r.rows,
            r.contacted,
            r.volume as f64 / f64::from(r.n),
            r.mean_deg
        );
    }
    println!("\nlocal exponent (adjacent N):");
    for w in rows.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let ratio = f64::from(b.n) / f64::from(a.n);
        let alpha = |x: f64, y: f64| (y / x).ln() / ratio.ln();
        println!(
            "  {:>4} → {:<4}  tick α = {:>6.3}   volume α = {:>6.3}   contacted α = {:>6.3}",
            a.n,
            b.n,
            alpha(a.us_per_tick, b.us_per_tick),
            alpha(
                a.volume as f64 / f64::from(a.n),
                b.volume as f64 / f64::from(b.n)
            ),
            alpha(
                a.contacted as f64 / f64::from(a.n),
                b.contacted as f64 / f64::from(b.n)
            ),
        );
    }
}

/// Housing geometry: with a fixed world the ring's radius is bounded by the map
/// (`ring_span = min(w,h)/2 − 2`), so above some house count the angular stride
/// collapses and houses pile up. Reports houses, distinct house cells, the
/// minimum pairwise Manhattan spacing, and the share of house pairs inside the
/// radius-5 perception neighbourhood.
fn housing_geometry(n: u32, world: u32) -> (usize, usize, i32, f64, u32) {
    let mut sim = Simulation::new(SimConfig {
        seed: SEED,
        max_ticks: 1,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    let mut cells: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    let mut positions: Vec<(i32, i32)> = Vec::new();
    for site in sim.world.sites.iter() {
        if matches!(site.kind, mindstrata_sim::world::SiteKind::House) {
            // Recover the placement from the site's tile: scan once per site.
            for y in 0..world as i32 {
                for x in 0..world as i32 {
                    if sim
                        .world
                        .tile(x, y)
                        .is_some_and(|t| t.site == Some(site.id))
                    {
                        positions.push((x, y));
                        *cells.entry((x, y)).or_insert(0) += 1;
                    }
                }
            }
        }
    }
    let mut min_gap = i32::MAX;
    let mut near = 0u32;
    let mut pairs = 0u32;
    for i in 0..positions.len() {
        for j in (i + 1)..positions.len() {
            let d =
                (positions[i].0 - positions[j].0).abs() + (positions[i].1 - positions[j].1).abs();
            min_gap = min_gap.min(d);
            pairs += 1;
            if d <= 5 {
                near += 1;
            }
        }
    }
    (
        positions.len(),
        cells.len(),
        if min_gap == i32::MAX { 0 } else { min_gap },
        f64::from(near) / f64::from(pairs.max(1)) * 100.0,
        cells.values().copied().max().unwrap_or(0),
    )
}

/// Charter method (i332): min of three independent processes' windows. i337
/// measured whole-tick A/B at ±5–10% *between processes*, and the single-window
/// readings of this probe disagreed in sign at N=192 (fixed 3 795 vs 4 776 in two
/// runs), so a single window cannot decide a world policy.
fn cost_us_min3(n: u32, world: u32) -> f64 {
    let mut best = f64::INFINITY;
    for rep in 0..3u64 {
        let mut sim = Simulation::new(SimConfig {
            seed: SEED + rep,
            max_ticks: WARMUP + TICKS,
            world_width: world,
            world_height: world,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(WARMUP);
        let t0 = Instant::now();
        sim.run(TICKS);
        best = best.min(t0.elapsed().as_secs_f64() * 1e6 / TICKS as f64);
    }
    best
}

/// Liveness: a bigger world must not starve or isolate the village.
fn liveness(n: u32, world: u32, ticks: u64) -> (u64, f64, f64, f64, f64) {
    let mut sim = Simulation::new(SimConfig {
        seed: SEED,
        max_ticks: ticks,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(ticks);
    let m = sim.metrics_snapshot();
    (
        m.agent_count,
        m.avg_health,
        m.avg_stress,
        m.total_grain,
        m.total_water,
    )
}

fn main() {
    let ns: Vec<u32> = {
        let args: Vec<u32> = std::env::args()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
        if args.is_empty() {
            vec![48, 96, 144, 192]
        } else {
            args
        }
    };

    println!(
        "i344 — constant-density world law (seed {SEED}, {TICKS} measured ticks after {WARMUP})\n\
         law: side(N) = max(16, ceil(sqrt({CELLS_PER_AGENT:.3} * N)))  \
         → N=12→16, N=48→32 (both calibrated points reproduced)"
    );
    for n in &ns {
        println!(
            "  N={:<4} derived side = {} ({:.1} cells/agent)",
            n,
            side_for_population(*n),
            f64::from(side_for_population(*n).pow(2)) / f64::from(*n)
        );
    }

    let fixed: Vec<Row> = ns.iter().map(|n| measure(*n, 32)).collect();
    let scaled: Vec<Row> = ns
        .iter()
        .map(|n| measure(*n, side_for_population(*n)))
        .collect();

    print_leg("charter policy: fixed 32x32 for every N", &fixed);
    print_leg("constant-density policy: world grows with N", &scaled);

    println!("\n=== cost leg (min of 3 independent processes, same seed family) ===\n");
    println!(
        "{:>5} {:>14} {:>14} {:>10}",
        "N", "fixed 32", "density", "Δ"
    );
    for n in ns.iter().filter(|n| **n >= 96) {
        let fixed_us = cost_us_min3(*n, 32);
        let density_us = cost_us_min3(*n, side_for_population(*n));
        println!(
            "{:>5} {:>14.1} {:>14.1} {:>9.1}%",
            n,
            fixed_us,
            density_us,
            (density_us - fixed_us) / fixed_us * 100.0
        );
    }

    println!("\n=== housing geometry (placement only, no run) ===\n");
    println!(
        "{:>5} {:>6} {:>8} {:>10} {:>10} {:>12} {:>10}",
        "N", "world", "houses", "cells", "min gap", "pairs <=5", "per cell"
    );
    for n in &ns {
        for world in [32u32, side_for_population(*n)] {
            let (houses, cells, min_gap, near, per_cell) = housing_geometry(*n, world);
            println!(
                "{:>5} {:>6} {:>8} {:>10} {:>10} {:>11.1}% {:>10}",
                n, world, houses, cells, min_gap, near, per_cell
            );
        }
    }

    println!("\n=== liveness leg (N=192, 2 000 ticks) ===\n");
    println!(
        "{:>10} {:>7} {:>12} {:>12} {:>12} {:>12}",
        "policy", "world", "population", "avg health", "avg stress", "grain"
    );
    for (label, world) in [("fixed", 32u32), ("density", side_for_population(192))] {
        let (pop, health, stress, grain, water) = liveness(192, world, 2_000);
        println!("{label:>10} {world:>7} {pop:>12} {health:>12.4} {stress:>12.4} {grain:>12.1}");
        println!("           (water {water:.1})");
    }

    println!(
        "\nreading: world policy is judged on the CONTACT VOLUME exponent — if volume/agent is flat\n\
         under the density policy while it climbs under the fixed world, the remaining superlinear\n\
         term is an area artifact and A9 is the fix. Liveness must stay flat: a bigger world that\n\
         starves the village is not a scaling win."
    );
}
