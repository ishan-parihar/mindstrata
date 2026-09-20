//! i341 — the per-edge share of the tick, after i340 removed the contact governor.
//!
//! i338 refuted the contact-driven sparse store (contact was ~45% of the matrix
//! because 8 fixed houses made every N a set of co-residency buckets). i340 fixed
//! the *governor* — with population-scaled housing only ~10% of rows are ever
//! contacted — but not the *storage*: `populate` still seeds a stranger row for
//! every ordered pair (pinned at i336), so every per-edge pass still walks
//! N(N−1) rows per tick regardless of whether they ever carry state.
//!
//! This probe asks the question that decides the next scale iteration: how much
//! of the tick is per-relationship-row work, and how dense is the store really?
//! It groups the opt-in pass profiler's marks into "per-edge" and "the rest",
//! and reports the contacted-row census alongside.
//!
//! Run: cargo run --release -p mindstrata-benches --example i341_edge_share -- [N...]

use mindstrata_sim::sim::{SimConfig, Simulation};

/// Marks whose cost is (primarily) one traversal of the relationship store per
/// tick. Grouped from the i335/i336/i337 per-pass anatomy.
const PER_EDGE: &[&str] = &[
    "cognitive",
    "rel_traces",
    "trust_sync+reset",
    "·derived+belief",
    "social_cluster",
    "kinship_daily",
    "appraisal",
];

fn run(n: u32, warmup: u64, window: u64) -> (f64, f64, usize, usize) {
    Simulation::pass_profile_reset();
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: warmup + window,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(warmup);

    let rels = sim.relationships();
    let total = rels.len();
    let contacted = rels.iter().filter(|r| r.interaction_count > 0).count();

    Simulation::pass_profile_reset();
    let t0 = std::time::Instant::now();
    sim.run(window);
    let us_per_tick = t0.elapsed().as_secs_f64() * 1e6 / window as f64;

    let edge_ns: u64 = Simulation::pass_profile_totals()
        .iter()
        .filter(|(name, _, _)| PER_EDGE.contains(name))
        .map(|(_, ns, _)| *ns)
        .sum();
    let edge_us = edge_ns as f64 / window as f64 / 1000.0;
    (us_per_tick, edge_us, total, contacted)
}

fn main() {
    let ns: Vec<u32> = {
        let args: Vec<u32> = std::env::args()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
        if args.is_empty() {
            vec![48, 96, 192]
        } else {
            args
        }
    };
    println!("i341 — per-relationship-row share of the tick (seed 42, 32x32, housing default)\n");
    println!(
        "{:>5} {:>12} {:>13} {:>9} {:>12} {:>10} {:>14}",
        "N", "µs/tick", "per-edge µs", "share", "store rows", "contacted", "contacted share"
    );
    for n in ns {
        let (total_us, edge_us, rows, contacted) = run(n, 250, 150);
        println!(
            "{:>5} {:>12.1} {:>13.1} {:>8.1}% {:>12} {:>10} {:>13.1}%",
            n,
            total_us,
            edge_us,
            edge_us / total_us * 100.0,
            rows,
            contacted,
            contacted as f64 / rows.max(1) as f64 * 100.0
        );
    }
    println!(
        "\nreading: a large per-edge share alongside a small contacted share ⇒ the store is dense in\n\
         ROWS but sparse in STATE, so the next lever is iteration over rows that carry state (the\n\
         i337 positional-index debt, which birth/death position reuse disqualified) — not the\n\
         contact-driven store i338 refuted."
    );
}
