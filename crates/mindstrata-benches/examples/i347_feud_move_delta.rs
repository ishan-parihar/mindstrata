//! i347 — what the revived §19.5.G producer actually moves, and whether
//! dominance still feeds escalation once it does.
//!
//! i347 re-contracted the §19.5.G feud-approach branch (gate `0.4 → 0.02`, and
//! above the daily routine instead of below it). `i346_decision_census` confirms
//! the *liveness* half in vivo — `Move` is now 0.55% of decisions, all from the
//! `feud` source (was 0.00%).
//!
//! This probe answers the two questions the sweep needs:
//!
//! 1. **Anatomy** — how much does the sim actually move now? Position changes,
//!    distinct cells, and the contact-graph consequence (near share, touched
//!    rows, mean partners), against i346's pre-fix baseline.
//! 2. **The statistic that moved** — `relational_dominance_feeds_violence_
//!    escalation` asserts an aggregate (dominant − subordinate) violence margin
//!    over 3 seeds; the fix flipped it from `+17` (i200 sweep: 35 vs 18) to
//!    `−2` (27 vs 29). A margin that small on 3 seeds is a *power* problem, not
//!    a mechanism statement (§4.1), so this leg re-measures the same statistic
//!    over a 12-seed family before anything is re-anchored.
//!
//! Run: cargo run --release -p mindstrata-benches --example i347_feud_move_delta

use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

const ANATOMY_TICKS: u64 = 5_000;

fn violence_of(sim: &Simulation) -> usize {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::ConflictOccurred {
                    kind: mindstrata_sim::conflict::ConflictKind::Violence,
                    ..
                }
            )
        })
        .count()
}

/// The test's crafted world: every directed relationship carries the same
/// `dependence`, so the daily `power_balance` recompute reproduces the asymmetry.
fn crafted_world(dependence: f64, seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    for a in &mut sim.agents {
        for r in &mut a.relationship_v2s {
            r.dependence = Fixed::from_f64(dependence);
        }
    }
    sim.run(ticks);
    sim
}

fn anatomy() {
    println!("=== i347 anatomy (seed 42, 32x32, {ANATOMY_TICKS} ticks) ===\n");
    println!(
        "{:>5} {:>10} {:>12} {:>12} {:>11} {:>11} {:>11}",
        "N", "cells/ag", "near pairs", "touched", "partners", "conflicts", "escalations"
    );
    for n in [12u32, 48] {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: ANATOMY_TICKS,
            world_width: 32,
            world_height: 32,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();

        let mut cells = vec![std::collections::HashSet::new(); sim.agents.len()];
        for _ in 0..ANATOMY_TICKS {
            sim.run(1);
            for (i, a) in sim.agents.iter().enumerate() {
                cells[i].insert((a.position.x, a.position.y));
            }
        }
        let mean_cells = cells
            .iter()
            .map(std::collections::HashSet::len)
            .sum::<usize>() as f64
            / cells.len().max(1) as f64;

        let mut near = 0usize;
        let mut pairs = 0usize;
        let mut partners = vec![0usize; sim.agents.len()];
        for (i, a) in sim.agents.iter().enumerate() {
            for (j, b) in sim.agents.iter().enumerate() {
                if i == j {
                    continue;
                }
                pairs += 1;
                let d = (a.position.x - b.position.x).abs() + (a.position.y - b.position.y).abs();
                if d <= 5 {
                    near += 1;
                    partners[i] += 1;
                }
            }
        }
        let touched = sim
            .relationships()
            .iter()
            .filter(|r| r.interaction_count > 0)
            .count();
        let mean_partners = partners.iter().sum::<usize>() as f64 / partners.len().max(1) as f64;
        println!(
            "{:>5} {:>10.1} {:>11.1}% {:>12} {:>11.1} {:>11} {:>11}",
            n,
            mean_cells,
            100.0 * near as f64 / pairs.max(1) as f64,
            touched,
            mean_partners,
            sim.recent_events(10_000_000)
                .iter()
                .filter(|e| matches!(e, SimEvent::ConflictOccurred { .. }))
                .count(),
            violence_of(&sim),
        );
    }
    println!("\n  i346 pre-fix baseline (same probe shape): N=12 0 position changes / 0 cells,\n  N=48 1.0 cell mean with the §10.4 courtship walk as the only mover.");
}

fn escalation_family() {
    println!("\n=== relational dominance → violence, 12-seed family (5000 ticks, 16x16) ===");
    println!(
        "{:>6} {:>11} {:>11} {:>9}",
        "seed", "dominant", "subordinate", "margin"
    );
    let (mut dom_total, mut sub_total) = (0i64, 0i64);
    let (mut dom_wins, mut seeds) = (0i64, 0i64);
    for seed in [1u64, 2, 3, 5, 7, 11, 17, 21, 33, 42, 44, 99] {
        let d = violence_of(&crafted_world(0.0, seed, 5_000));
        let s = violence_of(&crafted_world(1.0, seed, 5_000));
        dom_total += d as i64;
        sub_total += s as i64;
        seeds += 1;
        if d > s {
            dom_wins += 1;
        }
        println!("{seed:>6} {d:>11} {s:>11} {:>9}", d as i64 - s as i64);
    }
    println!(
        "\n  aggregate {dom_total} vs {sub_total} (margin {}) · dominant strictly higher on {dom_wins}/{seeds} seeds",
        dom_total - sub_total
    );
}

fn main() {
    anatomy();
    escalation_family();
}
