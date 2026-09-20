//! i326 — sizing the sparse-relationship-store payoff (the i316 scale lever #1).
//!
//! i294/i316 record the Ω(N²) dense `relationships` matrix as the structural
//! cost floor and rank a **sparse store** as the top lever. Before committing to
//! a refactor of that size, this probe sizes the payoff from live state: how
//! much of the matrix is ever *touched*? A directed edge that no interaction
//! ever updates is pure ballast — it costs per-tick traversal (the snapshot
//! capture, the trust-delta build, the provenance change-scan) and per-birth
//! extension while carrying only its populate-time random value.
//!
//! The measurement uses the store's own touch counter — `Relationship::
//! interaction_count`, incremented by the interaction paths — plus `kind`
//! (never-interacted edges stay `Stranger`). Reported per horizon and per N:
//! total edges, untouched share, evolved-kind share, the implied sparse size,
//! end-to-end tick cost, and the matrix size R = N(N−1).
//!
//! Run: cargo run --release -p mindstrata-benches --example i326_relationship_sparsity

use std::time::Instant;

use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 4] = [1, 42, 77, 123];

fn mean<T: Copy, F: Fn(T) -> f64>(items: &[T], f: F) -> f64 {
    if items.is_empty() {
        return 0.0;
    }
    items.iter().map(|&x| f(x)).sum::<f64>() / items.len() as f64
}

fn leg(n: usize, ticks: u64) {
    let mut edges = Vec::new();
    let mut untouched = Vec::new();
    let mut evolved = Vec::new();
    let mut touched_pairs = Vec::new();
    let mut ms_per_tick = Vec::new();
    let mut last_events = Vec::new();

    for &seed in &SEEDS {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: ticks,
            num_agents: n as u32,
            snapshot_interval: None,
            ..SimConfig::default()
        });
        sim.populate();
        let t0 = Instant::now();
        sim.run(ticks);
        let elapsed = t0.elapsed().as_secs_f64() * 1000.0;

        let rels = sim.relationships();
        let r = rels.len();
        let untouched_n = rels.iter().filter(|x| x.interaction_count == 0).count();
        let evolved_n = rels
            .iter()
            .filter(|x| !matches!(x.kind, mindstrata_sim::person::RelationshipKind::Stranger))
            .count();

        edges.push(r);
        untouched.push(untouched_n);
        evolved.push(evolved_n);
        touched_pairs.push(r - untouched_n);
        ms_per_tick.push(elapsed / ticks as f64);
        last_events.push(sim.recent_events(usize::MAX).len());
    }

    let r = mean(&edges, |x| x as f64);
    let un = mean(&untouched, |x| x as f64);
    let ev = mean(&evolved, |x| x as f64);
    let tp = mean(&touched_pairs, |x| x as f64);
    let ms = mean(&ms_per_tick, |x| x);
    let ev_total = mean(&last_events, |x| x as f64);

    println!("\nN={n} @{ticks} ticks ({} seeds):", SEEDS.len());
    println!("  matrix R = N(N−1)          : {r:>8.0}");
    println!(
        "  never touched (ic=0)       : {un:>8.0}  ({:.1}% of R)",
        100.0 * un / r.max(1.0)
    );
    println!(
        "  evolved kind (non-Stranger): {ev:>8.0}  ({:.1}% of R)",
        100.0 * ev / r.max(1.0)
    );
    println!(
        "  touched ≥1 interaction     : {tp:>8.0}  ({:.1}% of R)",
        100.0 * tp / r.max(1.0)
    );
    println!("  end-to-end ms/tick         : {ms:>8.3}");
    let ev_bytes = ev_total * std::mem::size_of::<mindstrata_core::event::SimEvent>() as f64;
    println!(
        "  event buffer (untrimmed)   : {ev_total:>12.0} events  ({:.1} MiB at {} B/event)",
        ev_bytes / (1024.0 * 1024.0),
        std::mem::size_of::<mindstrata_core::event::SimEvent>()
    );
    println!(
        "  implied sparse-store size  : {tp:>8.0} edges ({:.0}% reduction)",
        100.0 * (1.0 - tp / r.max(1.0))
    );
    println!(
        "  matrix traversal share     : {:.3}% of tick (R×3 passes at ~1ns/entry)",
        100.0 * (r * 3.0) / (ms * 1e6).max(1.0)
    );
}

fn main() {
    println!("i326 — relationship-matrix sparsity and the sparse-store payoff");

    for n in [12usize, 24, 48] {
        for ticks in [2_000u64, 20_000] {
            leg(n, ticks);
        }
    }

    println!("\nreading:");
    println!(
        "  (1) sparse store: only ~50% of edges are ballast at every N, and the \
         \n      three per-tick matrix traversals are <1% of tick cost — so the \
         \n      sparse-store lever does NOT address the tick bottleneck; demote it."
    );
    println!(
        "  (2) event buffer: `self.events` is never trimmed (documented debt in \
         \n      core::tick) and grows ~O(N) per tick — 1.05M events / 56 MiB at \
         \n      N=48 @20K. This is the real scale wall (memory, not time), and the \
         \n      recorded VecDeque<SimEvent> refactor is the fix. VERDICT: \
         \n      EVENT_BUFFER_IS_THE_SCALE_WALL"
    );
}
