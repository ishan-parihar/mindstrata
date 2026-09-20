//! i337 — are the cognitive pass's two per-agent O(R) walks removable?
//!
//! The i330 sub-profile (`i330_pass_profile`) attributes 912 µs of the pass's
//! 1 358 µs at N=192 to two walks over `relationship_v2s`:
//!   · `·cog quality fold`  (464 µs) — `avg_quality` = mean `quality()` over
//!     the agent's own rows, written to `status_v2.network_centrality`.
//!   · `·cog decay walk`    (447 µs) — `is_active_this_tick` → `decay(1)`.
//!
//! i335's recorded plan assumed the second was mostly *dormant-row* scanning
//! ("only needs rows touched this tick; dormant rows already decay daily").
//! That assumption is testable: `pass_social` sets `dirty` on a touched row and
//! the flag is only cleared at the 144-tick daily boundary, so a touched row
//! stays "active" — and keeps decaying every tick — until the next boundary.
//! If most rows are dirty at any moment, the filter is a near no-op and the
//! walk is INHERENT O(R), not an accident.
//!
//! Run: cargo run --release -p mindstrata-benches --example i337_cognitive_walks

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    println!("i337 — dirty-row share in the relationship store (seed 42, 32x32)\n");
    println!(
        "{:>5} {:>12} {:>12} {:>10} {:>12}",
        "N", "total rows", "dirty rows", "share", "per agent"
    );
    for n in [48u32, 96, 192] {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 400 + 40,
            world_width: 32,
            world_height: 32,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(400);
        let mut total = 0u64;
        let mut dirty = 0u64;
        let mut samples = 0u64;
        for _ in 0..40 {
            sim.run(1);
            for a in &sim.agents {
                for r in &a.relationship_v2s {
                    total += 1;
                    if r.dirty {
                        dirty += 1;
                    }
                }
            }
            samples += 1;
        }
        let rows = total as f64 / samples as f64;
        let dirty_rows = dirty as f64 / samples as f64;
        println!(
            "{:>5} {:>12.0} {:>12.0} {:>9.1}% {:>12.1}",
            n,
            rows,
            dirty_rows,
            dirty_rows / rows * 100.0,
            rows / n as f64
        );
    }

    println!("\nreading: `dirty` is set by pass_social on every touched row and only");
    println!("cleared at the 144-tick daily boundary, so a touched row stays active");
    println!("(and decays every tick) for the rest of the day. A HIGH share means the");
    println!("per-tick dirty filter visits nearly every row anyway — the walk is");
    println!("inherent O(R) on a complete graph, not a dormant-row scan.");
    println!("A LOW share would mean an explicit touched-row index would pay.");
}
