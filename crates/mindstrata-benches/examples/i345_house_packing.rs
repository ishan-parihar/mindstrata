//! i345 — A11: the housing layout must fit the world it is placed in.
//!
//! i344 measured the ring's degeneration in the charter's fixed 32×32 world:
//! `ring_span = min(w,h)/2 − 2` is a function of world size only, so above ~29
//! houses the angular stride collapses — at 48 houses it is `2π·14/48 ≈ 1.8`
//! tiles, the measured minimum pairwise house gap is **1 tile**, 8.4% of house
//! pairs sit inside the radius-5 perception neighbourhood, and **19 agents end up
//! co-located on one cell** against the declared `SiteKind::House.capacity = 4`.
//!
//! i345 replaces the ring with a Vogel/sunflower **area packing** above the
//! largest house count ever measured/calibrated (24 ⇒ N=96), so every calibrated
//! run keeps the legacy ring byte-for-byte and only unexplored territory changes.
//! This probe measures the new layout against i344's recorded ring numbers:
//! geometry first (spacing, occupancy), then the live consequences (contact
//! state, co-location, tick cost), all at the **fixed 32×32** charter world.
//!
//! Run: cargo run --release -p mindstrata-benches --example i345_house_packing -- [N...]

use std::collections::BTreeMap;
use std::time::Instant;

use mindstrata_sim::sim::{SimConfig, Simulation};

const SEED: u64 = 42;
const TICKS: u64 = 5_000;
const WARMUP: u64 = 50;
const COST_TICKS: u64 = 300;

struct Row {
    n: u32,
    houses: usize,
    cells: usize,
    max_per_cell: u32,
    min_gap: i32,
    near_house_share: f64,
    contacted: usize,
    mean_deg: f64,
    near_share: f64,
    max_co_located: u32,
}

fn house_tiles(sim: &Simulation, world: u32) -> Vec<(i32, i32)> {
    let mut out = Vec::new();
    for site in &sim.world.sites {
        if !matches!(site.kind, mindstrata_sim::world::SiteKind::House) {
            continue;
        }
        for y in 0..world as i32 {
            for x in 0..world as i32 {
                if sim
                    .world
                    .tile(x, y)
                    .is_some_and(|t| t.site == Some(site.id))
                {
                    out.push((x, y));
                }
            }
        }
    }
    out
}

fn measure(n: u32, world: u32) -> Row {
    let mut sim = Simulation::new(SimConfig {
        seed: SEED,
        max_ticks: TICKS,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();

    let tiles = house_tiles(&sim, world);
    let mut per_tile: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    for t in &tiles {
        *per_tile.entry(*t).or_insert(0) += 1;
    }
    let mut min_gap = i32::MAX;
    let mut near_houses = 0u32;
    let mut house_pairs = 0u32;
    for i in 0..tiles.len() {
        for j in (i + 1)..tiles.len() {
            let d = (tiles[i].0 - tiles[j].0).abs() + (tiles[i].1 - tiles[j].1).abs();
            min_gap = min_gap.min(d);
            house_pairs += 1;
            if d <= 5 {
                near_houses += 1;
            }
        }
    }

    sim.run(TICKS);

    let rels = sim.relationships();
    let mut out_deg: BTreeMap<u64, u32> = BTreeMap::new();
    for r in rels {
        if r.interaction_count > 0 {
            *out_deg.entry(r.from.as_u64()).or_insert(0) += 1;
        }
    }
    let contacted = out_deg.values().map(|d| *d as usize).sum::<usize>();
    let mean_deg = out_deg.values().map(|d| f64::from(*d)).sum::<f64>() / f64::from(n);

    let pos: Vec<(i32, i32)> = sim
        .agents
        .iter()
        .map(|a| (a.position.x, a.position.y))
        .collect();
    let mut by_cell: BTreeMap<(i32, i32), u32> = BTreeMap::new();
    for p in &pos {
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
        houses: tiles.len(),
        cells: per_tile.len(),
        max_per_cell: per_tile.values().copied().max().unwrap_or(0),
        min_gap: if min_gap == i32::MAX { 0 } else { min_gap },
        near_house_share: f64::from(near_houses) / f64::from(house_pairs.max(1)) * 100.0,
        contacted,
        mean_deg,
        near_share: f64::from(near) / f64::from(tot.max(1)) * 100.0,
        max_co_located: by_cell.values().copied().max().unwrap_or(0),
    }
}

/// Charter method (i332): min of three independent processes.
fn cost_us_min3(n: u32, world: u32) -> f64 {
    let mut best = f64::INFINITY;
    for rep in 0..3u64 {
        let mut sim = Simulation::new(SimConfig {
            seed: SEED + rep,
            max_ticks: WARMUP + COST_TICKS,
            world_width: world,
            world_height: world,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(WARMUP);
        let t0 = Instant::now();
        sim.run(COST_TICKS);
        best = best.min(t0.elapsed().as_secs_f64() * 1e6 / COST_TICKS as f64);
    }
    best
}

fn main() {
    let ns: Vec<u32> = {
        let args: Vec<u32> = std::env::args()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
        if args.is_empty() {
            vec![96, 144, 192]
        } else {
            args
        }
    };
    println!("i345 — house layout, fixed 32x32 world, seed {SEED} ({TICKS} ticks)\n");
    println!(
        "{:>5} {:>7} {:>7} {:>8} {:>8} {:>11} {:>11} {:>11} {:>9} {:>10}",
        "N",
        "houses",
        "tiles",
        "per tile",
        "min gap",
        "pairs<=5",
        "contacted",
        "partners/ag",
        "near%",
        "max co"
    );
    for n in &ns {
        let r = measure(*n, 32);
        println!(
            "{:>5} {:>7} {:>7} {:>8} {:>8} {:>10.1}% {:>11} {:>11.1} {:>8.1}% {:>10}",
            r.n,
            r.houses,
            r.cells,
            r.max_per_cell,
            r.min_gap,
            r.near_house_share,
            r.contacted,
            r.mean_deg,
            r.near_share,
            r.max_co_located
        );
    }

    println!("\ncost leg (min of 3, 300 ticks after 50):");
    println!("{:>5} {:>12} {:>12}", "N", "µs/tick", "vs i344 ring");
    // i344's recorded min-of-3 ring figures at the same params.
    for (n, ring) in [(96u32, 1_053.0_f64), (144, 2_298.6), (192, 3_734.6)] {
        if !ns.contains(&n) {
            continue;
        }
        let us = cost_us_min3(n, 32);
        println!(
            "{:>5} {:>12.1} {:>11.1}%",
            n,
            us,
            (us - ring) / ring * 100.0
        );
    }
    println!(
        "\nreading: i344's ring at the same probe recorded (N=96/144/192) min house gap 1,\n\
         pairs<=5 5.1/8.0/8.4%, contacted 792/2200/4234, partners 8.2/15.3/22.1, near\n\
         8.7/10.5/11.3%, max co-located 4/7/19. The packing should hold the gap and\n\
         co-location while the ring degenerates; N=96 (24 houses) must stay on the legacy\n\
         ring, so its row is the control."
    );
}
