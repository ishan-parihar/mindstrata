//! Iteration 186 — emergent-quality diagnostic probe for the two remaining
//! flags from the 100K stability check:
//!   1. calm-world life-theme convergence to all-Mission,
//!   2. inverted revolution churn (42–129 coups in calm vs 1–2 in famine).
//!
//! Run: `cargo run -p mindstrata-benches --example p5_emergent_diag --release`
//!
//! For each (scenario, seed): prints per-sample-horizon snapshots of
//!   - council legitimacy / faction count / avg grievance / population /
//!     revolution ticks so far / life-theme histogram, plus a final
//!     per-agent theme + script-balance dump for the theme flag.

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

fn run_scenario(name: &str, seed: u64, ticks: u64) -> Simulation {
    let mut sc = match name {
        "calm" => Scenario::calm(),
        "famine" => Scenario::famine(),
        "pestilence" => Scenario::pestilence(),
        _ => unreachable!(),
    };
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim
}

fn council_legitimacy(sim: &Simulation) -> f64 {
    let mut max = Fixed::ZERO;
    for i in &sim.institutions {
        if i.kind == InstitutionKind::Council {
            max = max.max(i.legitimacy);
        }
    }
    max.to_f64()
}

fn faction_count(sim: &Simulation) -> usize {
    sim.institutions
        .iter()
        .filter(|i| i.kind == InstitutionKind::Faction)
        .count()
}

fn avg_grievance(sim: &Simulation) -> f64 {
    let n = sim.agents.len();
    if n == 0 {
        return 0.0;
    }
    let sum: Fixed = sim
        .agents
        .iter()
        .map(|a| {
            mindstrata_sim::factions::compute_grievance(
                a.derived.resentment,
                a.emotions.fear,
                a.emotions.anger,
                a.needs.autonomy,
                a.needs.meaning,
                a.moral_values.fairness,
                sim.market.inequality,
            )
        })
        .fold(Fixed::ZERO, |a, b| a + b);
    sum.to_f64() / n as f64
}

fn theme_hist(sim: &Simulation) -> BTreeMap<String, u32> {
    let mut h = BTreeMap::new();
    for a in &sim.agents {
        *h.entry(format!("{:?}", a.narrative.life_theme)).or_insert(0) += 1;
    }
    h
}

fn revolution_count(sim: &Simulation) -> u32 {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                mindstrata_core::event::SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Revolution,
                    ..
                }
            )
        })
        .count() as u32
}

fn main() {
    let filter = std::env::var("P5_FLAG").unwrap_or_default(); // "theme" | "rev" | ""
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(100_000);

    for (scenario, seed) in [("calm", 42u64), ("calm", 7), ("famine", 42), ("famine", 13)] {
        if filter == "theme" && scenario != "calm" {
            continue;
        }
        println!("=== {scenario} seed {seed} ===");
        let mut sim = run_scenario(scenario, seed, horizon);
        // sample every 5000 ticks up to the horizon
        let mut ticks_run = 0u64;
        let sample = 5000u64;
        let mut last_revs = 0u32;
        while ticks_run < horizon {
            sim.run(sample.min(horizon - ticks_run));
            ticks_run += sample.min(horizon - ticks_run);
            let revs = revolution_count(&sim);
            if ticks_run.is_multiple_of(10000) || ticks_run == horizon {
                println!(
                    "  t={ticks_run:>6} pop={:<3} legit={:.3} factions={} grievance={:.3} revs={} (+{}) themes={:?}",
                    sim.agents.len(),
                    council_legitimacy(&sim),
                    faction_count(&sim),
                    avg_grievance(&sim),
                    revs,
                    revs - last_revs,
                    theme_hist(&sim),
                );
                last_revs = revs;
            }
        }
        if filter == "rev" {
            // Revolution tick timeline — the honest cadence (every event's
            // tick from the ring buffer, oldest→newest, with gaps).
            let rev_ticks: Vec<u64> = sim
                .recent_events(10_000_000)
                .iter()
                .filter_map(|e| match e {
                    mindstrata_core::event::SimEvent::ConflictOccurred {
                        kind: mindstrata_core::conflict::ConflictKind::Revolution,
                        tick,
                        ..
                    } => Some(tick.as_u64()),
                    _ => None,
                })
                .collect();
            println!("  revolution ticks: {rev_ticks:?}");
            let gaps: Vec<u64> = rev_ticks.windows(2).map(|w| w[1] - w[0]).collect();
            println!("  gaps: {gaps:?}");
        }

        if filter == "rev" {
            // Decompose grievance + legitimacy drivers at the final horizon.
            let n = sim.agents.len().max(1) as f64;
            let (mut res, mut fear, mut anger, mut auton, mut mean, mut fair) =
                (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
            for a in &sim.agents {
                res += a.derived.resentment.to_f64();
                fear += a.emotions.fear.to_f64();
                anger += a.emotions.anger.to_f64();
                auton += a.needs.autonomy.to_f64();
                mean += a.needs.meaning.to_f64();
                fair += a.moral_values.fairness.to_f64();
            }
            println!(
                "  grievance-decomp @{horizon}: resentment={:.3} fear={:.3} anger={:.3} autonomy_need={:.3} meaning_need={:.3} fairness={:.3} gini={:.3}",
                res / n, fear / n, anger / n, auton / n, mean / n, fair / n, sim.market.inequality.to_f64()
            );
            for i in &sim.institutions {
                println!(
                    "    inst {:?}: legit={:.3} corruption={:.3} cohesion={:.3} morale={:.3} treasury={:.0} members={}",
                    i.kind, i.legitimacy.to_f64(), i.corruption.to_f64(), i.cohesion.to_f64(),
                    i.collective.morale.to_f64(), i.treasury.to_f64(), i.members.len()
                );
            }
            println!(
                "    rituals={} marriages={} households={} wealth_gini_agents={:.3}",
                sim.ritual_registry.rituals.len(),
                sim.marriage_registry.marriages.len(),
                sim.households.len(),
                0.0
            );
        }
        if filter == "coin" {
            let mut coins: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
            coins.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let under1 = coins.iter().filter(|c| **c < 1.0).count();
            let under3 = coins.iter().filter(|c| **c < 3.0).count();
            println!(
                "  coins @{horizon}: sorted={coins:?} under1={under1}/{} under3={under3}/{} market_treasury={:.0} council_treasury={:.0} avg={:.2} median={:.2}",
                coins.len(),
                coins.len(),
                sim.institutions.iter().find(|i| i.kind == InstitutionKind::Market).map_or(0.0, |m| m.treasury.to_f64()),
                sim.institutions.iter().find(|i| i.kind == InstitutionKind::Council).map_or(0.0, |c| c.treasury.to_f64()),
                coins.iter().sum::<f64>() / coins.len() as f64,
                coins[coins.len() / 2]
            );
            let (mut hunger, mut thirst) = (0.0f64, 0.0f64);
            for a in &sim.agents {
                hunger += a.needs.hunger.to_f64();
                thirst += a.needs.thirst.to_f64();
            }
            println!(
                "  needs @{horizon}: hunger={:.3} thirst={:.3}",
                hunger / sim.agents.len() as f64, thirst / sim.agents.len() as f64
            );
        }
        if filter == "theme" {
            // Per-agent theme + script balances at the final horizon.
            println!("  per-agent scripts @{horizon}:");
            for (i, a) in sim.agents.iter().enumerate() {
                let n = &a.narrative;
                println!(
                    "    a{i}: theme={:?} pos={:.2}/{:.2}/{:.2} neg={:.2}/{:.2}/{:.2} neuro={:.2} consc={:.2} extra={:.2} amb={:.2}",
                    n.life_theme,
                    n.redemption_script.to_f64(),
                    n.heroism_script.to_f64(),
                    n.chosenness_script.to_f64(),
                    n.contamination_script.to_f64(),
                    n.victimhood_script.to_f64(),
                    n.shame_script.to_f64(),
                    a.personality.neuroticism.to_f64(),
                    a.personality.conscientiousness.to_f64(),
                    a.personality.extraversion.to_f64(),
                    a.personality.ambition.to_f64(),
                );
            }
        }
    }
}
