//! i378 — is the moral-panic trigger KNIFE-EDGE or STARVED?
//!
//! i376 de-saturated the legacy trust store, and the panic seed family had to be
//! re-anchored `{7,11,46}` → `{7,1,23}` (firing density 3/5 → 3/10 on the swept
//! set). Before treating that as debt to repair or as a legitimate consequence to
//! record, measure the trigger's *headroom*: the panic fires when
//! `avg_charge ≥ 0.55 AND panic_ratio ≥ 0.3` over the agents holding a belief
//! about proposition 0 ("the_market_is_fair") or 1 ("the_council_protects_us").
//!
//! Two very different diagnoses look identical from a seed-firing count:
//!   * KNIFE-EDGE — close runs reach ~0.5 but rarely clear 0.55. The mechanism
//!     has no margin and the family will keep moving on every pacing shift. That
//!     is the §4.5 "knife-edge flags are debt" class (the epidemic R0≈1 case):
//!     record it, do not flip-flop pins.
//!   * STARVED — close runs peak far below the bar. The producer is not
//!     marginal, it is under-driven, and reviving it is the fix.
//!
//! This samples the trigger's inputs every tick over a crisis run and reports
//! the best (highest avg_charge) tick, whether both legs cleared, and the gap to
//! the bar. Deterministic; no RNG; pure observation.
//!
//! **i381 note:** the bar this probe measures against is now the relative/anomaly
//! law's *absolute floor* (`MORAL_PANIC_CHARGE_FLOOR`), because i381 replaced the
//! old absolute 0.55 threshold — which is precisely what this probe indicted. The
//! 0.55-based table lives on as the historical measurement in
//! `docs/architecture/AP4-studio/evidence/i378_panic_trigger_knife_edge.md`; this
//! probe now re-measures headroom against whatever bar ships, so it stays a
//! reusable instrument rather than a snapshot of one law.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i378_panic_threshold_headroom`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

/// The trigger's two legs, recomputed exactly as `gossip::detect_moral_panic`.
fn legs(sim: &Simulation, prop: u64) -> Option<(f64, f64)> {
    let charges: Vec<f64> = sim
        .agents
        .iter()
        .filter_map(|a| {
            a.beliefs
                .iter()
                .find(|b| b.proposition_id == prop)
                .map(|b| b.emotional_charge.to_f64())
        })
        .collect();
    if charges.is_empty() {
        return None;
    }
    let avg = charges.iter().sum::<f64>() / charges.len() as f64;
    let ratio = charges.iter().filter(|c| **c > 0.4).count() as f64 / charges.len() as f64;
    Some((avg, ratio))
}

fn main() {
    let bar_charge = mindstrata_social::gossip::MORAL_PANIC_CHARGE_FLOOR.to_f64();
    println!("i378 — panic-trigger headroom, pestilence @20K (bar: avg_charge ≥ {bar_charge:.2}, panic_ratio ≥ 0.30)");
    println!(
        "\n{:>5} {:>4} {:>10} {:>11} {:>11} {:>9} {:>8}",
        "seed", "prop", "max_avg", "max_ratio", "best_score", "cleared", "margin"
    );
    let mut fired = 0usize;
    let mut n = 0usize;
    for seed in [7u64, 1, 23, 11, 46, 5, 42, 13, 99, 3] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 20_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        // Best tick per proposition, chosen by the *minimum* of the two legs'
        // normalized headroom — the tick that came closest to firing.
        // Per proposition: the highest charge ever reached, the highest ratio
        // ever reached, and the best score = min(avg/bar, ratio/0.3) — the tick
        // that came closest to firing on BOTH legs.
        let mut max_avg = [0.0f64; 2];
        let mut max_ratio = [0.0f64; 2];
        let mut best_score = [0.0f64; 2];
        let mut seen = [false; 2];
        for _ in 0..20_000 {
            sim.tick();
            for prop in 0..=1u64 {
                if let Some((avg, ratio)) = legs(&sim, prop) {
                    let p = prop as usize;
                    seen[p] = true;
                    max_avg[p] = max_avg[p].max(avg);
                    max_ratio[p] = max_ratio[p].max(ratio);
                    best_score[p] = best_score[p].max((avg / bar_charge).min(ratio / 0.3));
                }
            }
        }
        // The two lenses the integration tests use: the registry size and the
        // `ConflictOccurred { MoralPanic }` event count.
        let registry = sim.moral_panic_registry.panics.len();
        let events = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    mindstrata_core::event::SimEvent::ConflictOccurred {
                        kind: mindstrata_core::conflict::ConflictKind::MoralPanic,
                        ..
                    }
                )
            })
            .count();
        println!("      seed {seed}: registry {registry}, panic events {events}");
        for prop in 0..=1u64 {
            let p = prop as usize;
            if !seen[p] {
                continue;
            }
            let cleared = best_score[p] >= 1.0;
            if cleared {
                fired += 1;
            }
            n += 1;
            println!(
                "{seed:>5} {prop:>4} {:>10.4} {:>11.4} {:>11.4} {:>8} {:>7.3}",
                max_avg[p],
                max_ratio[p],
                best_score[p],
                if cleared { "YES" } else { "no" },
                best_score[p] - 1.0
            );
        }
        println!();
    }
    println!("\nlegs that cleared the bar at their best tick: {fired}/{n}");
    println!(
        "reading: a score near 1.0 = KNIFE-EDGE (§4.5 debt, record it); a score well\n\
         below 1.0 = STARVED (revive the producer); most rows near 1.0 = the family\n\
         will keep moving on every pacing shift."
    );
}
