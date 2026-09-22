//! i381 — attribute the blast radius of the relative/anomaly panic trigger.
//!
//! The trigger change displaced two pinned integration contracts:
//!   * `biology::attachment_separation_distress_coupling_is_live_after_tuning`
//!     (default village, seed 42, 24×24 N=48 @5K): non-zero partnered distress
//!     39/46 → 34/48.
//!   * `governance::revolution_is_regime_change_not_repeat_loop`
//!     (pestilence @70K, family {5, 11, 42}): revolutions 3/3/3 → 0/0/3.
//!
//! "The pin moved" is not an explanation. Both tests read state that only the
//! §7.2 panic path can reach (panic → legitimacy damage → grievance → coup), so
//! this measures the causal channel directly: does a panic that the *old*
//! absolute bar missed fire under the new law, and where does the charge sit at
//! the moment it does?
//!
//! Part A: the attachment test's exact config, reporting the trigger's own
//!         inputs and both laws' verdicts at the tick they diverge.
//! Part B: the revolution family's seed sweep — the evidence a re-anchor needs
//!         (§4.8: hold the family fixed and *discover* which members fire).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i381_blast_radius`
//! (≈4–6 min wall: 1 × 5K + 10 × 70K pestilence.)

use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

/// The trigger's two raw legs, recomputed exactly as `gossip::detect_moral_panic`.
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

fn panics(sim: &Simulation) -> usize {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| format!("{e:?}").contains("Panic"))
        .count()
}

fn revolutions(sim: &Simulation) -> usize {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Revolution,
                    ..
                }
            )
        })
        .count()
}

/// The old law's verdict on the same sample — what the pre-i381 world decided.
fn old_law_fires(avg: f64, ratio: f64) -> bool {
    avg >= 0.55 && ratio >= 0.30
}

fn main() {
    // ── Part A: the attachment contract's world ─────────────────────────────
    println!("i381 — blast-radius attribution\n");
    println!("=== A. attachment contract world (default village, seed 42, 24x24 N=48) @5K ===");
    let config = mindstrata_sim::sim::SimConfig {
        seed: 42,
        max_ticks: 5000,
        world_width: 24,
        world_height: 24,
        num_agents: 48,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let mut first_gap_tick: Option<(u64, u64, f64, f64)> = None;
    for tick in 1..=5000u64 {
        sim.tick();
        for prop in 0..=1u64 {
            if let Some((avg, ratio)) = legs(&sim, prop) {
                let new_fires = avg >= 0.47 && ratio >= 0.30;
                let old_fires = old_law_fires(avg, ratio);
                if new_fires && !old_fires && first_gap_tick.is_none() {
                    first_gap_tick = Some((tick, prop, avg, ratio));
                }
            }
        }
    }
    println!("  panic events (new law): {}", panics(&sim));
    println!("  baseline at end: {:?}", sim.moral_charge_baseline());
    match first_gap_tick {
        Some((tick, prop, avg, ratio)) => println!(
            "  first tick where the NEW bar fires and the OLD one does not: tick {tick}, \
             prop {prop}, avg {avg:.4}, ratio {ratio:.4}  (old bar needed avg >= 0.55)"
        ),
        None => println!(
            "  no tick at which the two laws disagree — the attachment shift is NOT \
             panic-mediated"
        ),
    }
    let partnered = sim.agents.iter().filter(|a| a.partner.is_some()).count();
    let nonzero = sim
        .agents
        .iter()
        .filter(|a| a.partner.is_some() && a.attachment.separation_distress > Fixed::ZERO)
        .count();
    println!("  partnered {partnered}, non-zero separation distress {nonzero}");

    // ── Part B: the revolution family sweep ────────────────────────────────
    println!("\n=== B. revolution family sweep — pestilence @70K, meme mutation isolated ===");
    println!(
        "{:>7} {:>13} {:>8} {:>10} {:>14} {:>14}",
        "seed", "revolutions", "panics", "council_leg", "baseline[0]", "baseline[1]"
    );
    let mut firing: Vec<u64> = Vec::new();
    for seed in [5u64, 11, 42, 7, 1, 23, 99, 12345, 3, 13] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 70_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.params.meme_mutation_rate_base = Fixed::ZERO;
        sim.populate();
        sim.run(70_000);
        let revs = revolutions(&sim);
        if revs > 0 {
            firing.push(seed);
        }
        let council_leg = sim
            .institutions
            .iter()
            .find(|i| {
                matches!(
                    i.kind,
                    mindstrata_sim::institutions::InstitutionKind::Council
                )
            })
            .map_or(0.0, |i| i.legitimacy.to_f64());
        println!(
            "{seed:>7} {revs:>13} {:>8} {council_leg:>10.4} {:>14.4} {:>14.4}",
            panics(&sim),
            sim.moral_charge_baseline()[0],
            sim.moral_charge_baseline()[1]
        );
    }
    println!("  → firing seeds: {firing:?}");
    println!(
        "\nreading: the family must stay a FAMILY (≥2 firing seeds, §4.8). If only one seed\n\
         fires, the trigger change has starved the revolution producer and the honest\n\
         response is to widen the family onto the discovered members — or, if none fire,\n\
         to revive the producer rather than re-pin."
    );
}
