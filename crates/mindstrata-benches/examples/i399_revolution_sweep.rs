//! i399 — is the revolution producer STARVED by the marriage store migration,
//! or merely re-timed?
//!
//! The i399 migration (marriage formation gate + bond boost onto the dyadic
//! store) broke the `revolution_is_regime_change_not_repeat_loop` family: the
//! discovered members {5, 42, 12345} measured revolutions 3/1/7 before, and
//! **0/0/7** after — only 1 of 3 seeds still fires, below the ≥2-of-3 liveness
//! bar. Per AGENTS.md §2.3 a dark producer is a bug, not a re-anchor, so the
//! first question is whether the producer is dark or the chaos re-timed.
//!
//! This re-runs the exact sweep the i381/i388 re-anchors used (pestilence @70K,
//! meme mutation isolated to zero, 10 seeds) so the before/after densities are
//! directly comparable. The producer is *healthy* if the total across seeds is
//! comparable and the firing seeds merely move; it is *starved* if the total
//! collapses.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i399_revolution_sweep`

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

/// The i381/i388 sweep seeds — kept identical so the densities compare.
const SEEDS: [u64; 10] = [5, 42, 12345, 7, 23, 11, 1, 99, 3, 13];

fn main() {
    println!("i399 — revolution 10-seed sweep (pestilence @70K, mutation off)\n");
    println!("  seed   revolutions");
    let mut total = 0usize;
    let mut firing = Vec::new();
    for seed in SEEDS {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 70_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.params.meme_mutation_rate_base = Fixed::ZERO;
        sim.populate();
        sim.run(70_000);
        let revs = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::ConflictOccurred {
                        kind: ConflictKind::Revolution,
                        ..
                    }
                )
            })
            .count();
        total += revs;
        if revs > 0 {
            firing.push((seed, revs));
        }
        println!("  {seed:>6}   {revs}");
    }
    println!(
        "\n  total {total} across {} seeds; firing {firing:?} ({} of {})",
        SEEDS.len(),
        firing.len(),
        SEEDS.len()
    );
}
