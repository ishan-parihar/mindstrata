//! i288 — transgression-diet probe (PLAN_DC3 §4 i288, debt A1).
//!
//! A1: the Transgression catalyst feed is dead at N=12 — `NormViolated` = 0 in
//! every probed regime, so (a) the i286 norm-proposal gate can never accept
//! Value/Norm syntheses and (b) WP-J channel #1 (§12.3 compliance) has no
//! surface. Before designing a revival, diagnose WHERE the pipeline starves:
//!
//!   hunger/thirst pressure → accessible-source miss → enforce_theft →
//!   deterrence (resistance × hypocrisy × legitimacy) → detection roll →
//!   NormViolated event
//!
//! Observable stages censused per regime:
//!   - `NormViolated` events (the Transgression producer itself)
//!   - `TheftDetected` journal entries (post-detection)
//!   - mean no-theft norm strength + population moral resistance (deterrence)
//!   - owned sites (does the inaccessible-source precondition even exist?)
//!   - hunger equilibrium (is there pressure to steal at all?)
//!
//! Regimes: calm (no shock), drought (spec), famine (spec — the Iter-232
//! crop-failure semantics: production suppression + granary drain), seeds
//! 42/1/7 at 20K. Verdict names the starving stage; the revival design
//! follows the verdict.

use mindstrata_core::event::SimEvent;
use mindstrata_sim::norms::NO_THEFT_NORM_ID;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    }
}

fn run(seed: u64, ticks: u64, regime: &str) {
    // Famine rides the real scenario machinery (production suppression +
    // granary drain, Iter-232 semantics) — not an ad-hoc stock mutation.
    // `famine_20k` opens the suppression window every 2500 ticks (the shipped
    // window is 2000) — sustained Malthusian pressure, since the one-shot
    // famine recovers. `famine_long` holds the window open for the whole run
    // via a shock at tick 100 — the desperate-village regime.
    let mut sim = match regime {
        "famine" => {
            let mut sc = Scenario::famine();
            sc.seed = seed;
            sc.ticks = ticks;
            Simulation::from_scenario(sc)
        }
        "famine_long" => {
            let mut sc = Scenario::famine();
            sc.seed = seed;
            sc.ticks = ticks;
            sc.name = "FamineSustained".into();
            sc.shocks[1].at_tick = 100; // open the suppression window at t=100…
                                        // …and extend it manually for the whole horizon below (window =
                                        // shock_tick + FAMINE_WINDOW_TICKS=2000; sustained = re-shock).
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 2_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 4_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 6_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 8_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 10_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 12_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 14_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 16_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            sc.shocks.push(mindstrata_sim::scenario::Shock {
                at_tick: 18_100,
                kind: mindstrata_sim::scenario::ShockKind::Famine,
                magnitude: mindstrata_core::fixed::Fixed::from_f64(0.7),
            });
            Simulation::from_scenario(sc)
        }
        _ => Simulation::new(config(seed, ticks)),
    };
    sim.populate();
    sim.run(ticks);

    let events = sim.recent_events(sim.event_count());
    let violations = events
        .iter()
        .filter(|e| matches!(e, SimEvent::NormViolated { .. }))
        .count();
    let detected = sim
        .journal()
        .recent(sim.journal_len())
        .iter()
        .filter(|e| {
            matches!(
                e.kind,
                mindstrata_sim::journal::JournalEntryKind::TheftDetected { .. }
            )
        })
        .count();

    let norms = sim.norms.norms();
    let mean_strength = if norms.is_empty() {
        0.0
    } else {
        norms.iter().map(|n| n.strength.to_f64()).sum::<f64>() / norms.len() as f64
    };
    let no_theft_name = norms
        .iter()
        .find(|n| n.id == NO_THEFT_NORM_ID)
        .map(|n| n.name.clone());
    let mean_resistance = match &no_theft_name {
        Some(name) => {
            let acc: f64 = sim
                .agents
                .iter()
                .map(|a| a.moral_cognition.norm_resistance(name).to_f64())
                .sum();
            acc / sim.agents.len() as f64
        }
        None => 0.0,
    };
    let owned = sim.world.sites.iter().filter(|s| s.owner.is_some()).count();
    let total_sites = sim.world.sites.len();
    let hunger = sim.metric_history.last().map_or(0.0, |m| m.avg_hunger);
    println!(
        "regime={regime:<7} seed={seed} viol={violations} detected={detected} \
         strength={mean_strength:.3} resistance={mean_resistance:.3} \
         owned={owned}/{total_sites} hunger={hunger:.3}"
    );
}
fn main() {
    println!("== i288 transgression-diet diagnosis (N=12, 20K) ==");
    for regime in ["calm", "famine", "famine_long"] {
        for seed in [42_u64, 1, 7] {
            run(seed, 20_000, regime);
        }
    }

    // Blast-window census: how much violence fires inside the PINNED horizons
    // (golden 1000/4320, snapshots 500/2000/10000)? Each Violence conflict
    // will emit one NormViolated event after the fix, so these counts bound
    // the re-anchor surface.
    println!("\n-- violence census at pinned horizons (calm, seed 42) --");
    for horizon in [500_u64, 1_000, 2_000, 4_320, 10_000] {
        let mut sim = Simulation::new(config(42, horizon));
        sim.populate();
        sim.run(horizon);
        let violence = sim
            .recent_events(sim.event_count())
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::ConflictOccurred {
                        kind: mindstrata_core::conflict::ConflictKind::Violence,
                        ..
                    }
                )
            })
            .count();
        let conflicts = sim
            .recent_events(sim.event_count())
            .iter()
            .filter(|e| matches!(e, SimEvent::ConflictOccurred { .. }))
            .count();
        println!("  t={horizon:>6}: violence={violence} all_conflicts={conflicts}");
    }
}
