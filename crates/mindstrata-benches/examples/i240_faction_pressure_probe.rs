//! Iteration 240 — faction crisis-pressure accumulator calibration probe.
//!
//! Prints, at 5K-tick samples: accumulated crisis pressure, unorganized-pool
//! mean grievance, council legitimacy, and v1/v2 faction counts. Verifies the
//! Iteration-240 hazard-accumulator contract:
//!   - pestilence (sustained grievance excess) arms within ~15K on ALL seeds
//!     (the old single-tick cliff stopped firing entirely on seeds like 5);
//!   - calm worlds stay unarmed in short horizons and produce ~1 formation
//!     per 50K via rare sustained distress streaks (long-horizon liveness);
//!   - pressure bleeds to ~0 during peace stretches.
//!
//! Run: `cargo run -p mindstrata-benches --example i240_faction_pressure_probe --release`
//! Knobs: `I240_SCEN` (pestilence|calm|famine), `I240_SEED`, `I240_HORIZON`.

use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

fn main() {
    let horizon: u64 = std::env::var("I240_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30_000);
    let seed: u64 = std::env::var("I240_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(5);
    let scen = std::env::var("I240_SCEN").unwrap_or_else(|_| "pestilence".into());

    let mut sc = match scen.as_str() {
        "pestilence" => Scenario::pestilence(),
        "calm" => Scenario::calm(),
        "famine" => Scenario::famine(),
        _ => unreachable!(),
    };
    sc.seed = seed;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();

    println!("[{scen} seed {seed} @{horizon}] pressure build=0.002/unit bleed=0.0005 arm@0.5");
    let mut t = 0u64;
    while t < horizon {
        sim.run(1000);
        t += 1000;
        let fine = std::env::var("I240_FINE").is_ok();
        if (!fine && t % 5000 != 0 && t != horizon) || (fine && t > 15_000 && t % 5000 != 0) {
            continue;
        }
        let v1 = sim
            .institutions
            .iter()
            .filter(|i| i.kind == InstitutionKind::Faction)
            .count();
        let v2_active = sim
            .faction_v2_registry
            .factions
            .iter()
            .filter(|f| f.active)
            .count();
        let legit = sim
            .institutions
            .iter()
            .filter(|i| i.kind == InstitutionKind::Council)
            .map(|i| i.legitimacy.to_f64())
            .sum::<f64>();
        println!(
            "  t={t:>6}: pressure={:.3} pool_griev={:.3} legit={:.3} v1={v1} v2_active={v2_active} v2_hist={hist}",
            sim.faction_crisis_pressure(),
            sim.faction_pool_mean_grievance(),
            legit,
            hist = sim.faction_v2_registry.factions.len(),
        );
    }
}
