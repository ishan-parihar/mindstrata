//! Iteration 241 — moral-panic trigger calibration probe.
//!
//! Prints, at 5K-tick samples: average belief emotional charge for
//! propositions 0/1 (the §7.2 panic triggers), the share of agents above the
//! 0.4 high-charge mark, and the moral-panic registry state. Diagnoses why
//! panics stopped firing in anchored windows (audit E7: belief erosion).
//!
//! Run: `cargo run -p mindstrata-benches --example i241_panic_probe --release`
//! Knobs: `I241_SCEN` (calm|pestilence|famine|collapse), `I241_SEED`,
//! `I241_HORIZON`.

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

fn main() {
    let horizon: u64 = std::env::var("I241_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000);
    let seed: u64 = std::env::var("I241_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    let scen = std::env::var("I241_SCEN").unwrap_or_else(|_| "calm".into());

    let mut sc = match scen.as_str() {
        "calm" => Scenario::calm(),
        "pestilence" => Scenario::pestilence(),
        "famine" => Scenario::famine(),
        "collapse" => Scenario::collapse(),
        _ => unreachable!(),
    };
    sc.seed = seed;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();

    println!("[{scen} seed {seed} @{horizon}] trigger: avg_charge>=0.55 && ratio(>0.4)>=0.30");
    let mut t = 0u64;
    while t < horizon {
        sim.run(1000);
        t += 1000;
        if t % 5000 != 0 && t != horizon {
            continue;
        }
        let belief_refs: Vec<&Vec<mindstrata_sim::person::Belief>> =
            sim.agents.iter().map(|a| &a.beliefs).collect();
        let mut line = String::new();
        for prop in 0..=1u64 {
            let charges: Vec<f64> = belief_refs
                .iter()
                .filter_map(|bs| {
                    bs.iter()
                        .find(|b| b.proposition_id == prop)
                        .map(|b| b.emotional_charge.to_f64())
                })
                .collect();
            let n = charges.len().max(1) as f64;
            let avg = charges.iter().sum::<f64>() / n;
            let ratio = charges.iter().filter(|c| **c > 0.4).count() as f64 / n;
            line.push_str(&format!(" p{prop}: avg={avg:.3} hi%={ratio:.2}"));
        }
        let panics = sim.moral_panic_registry.panics.len();
        let active = sim
            .moral_panic_registry
            .panics
            .iter()
            .filter(|p| p.active)
            .count();
        println!("  t={t:>6}:{line} panics={panics} active={active}");
    }
}
