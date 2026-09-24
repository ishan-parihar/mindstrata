//! i289 — verdict probe: catalyst magnitudes vs the observed diet.
//!
//! i288 revived the Transgression feed (violence → NormViolated). This probe
//! asks the ratification question for the two remaining spec-midpoint pins
//! (A2): Grief 1.0 (maximal-loss exemplar) and Transgression 0.5 (spec
//! midpoint). It does NOT change production code — it measures what the
//! downstream pathology channels (Q1 dark-addiction via Threat, Q2 dark-
//! allergy via Transgression, Q4 golden-allergy via Grief) look like at
//! 20K under the live diet in the mortality scenario, so the magnitude pins
//! can be ratified or honestly re-scoped against observed channel states.
//!
//! Method: pestilence scenario (the only regime with both griefs AND
//! violations at feasible horizons), seeds 42/1/7, 20K. Censused: catalyst
//! counts, per-channel intensities, altitude-line states, and the
//! moderation surface (mourning-rite Agape decay vs Grief arrival).

use mindstrata_core::event::SimEvent;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn run(seed: u64, ticks: u64) {
    let mut sc = Scenario::pestilence();
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(ticks);
    let events = sim.recent_events(sim.event_count());
    let griefs = events
        .iter()
        .filter(|e| matches!(e, SimEvent::GriefStruck { .. }))
        .count();
    let violations = events
        .iter()
        .filter(|e| matches!(e, SimEvent::NormViolated { .. }))
        .count();
    let n = sim.agents.len().max(1) as f64;
    let q1 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.dark_addiction.intensity)
        .sum::<f64>()
        / n;
    let q2 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.dark_allergy.intensity)
        .sum::<f64>()
        / n;
    let q3 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.golden_addiction.intensity)
        .sum::<f64>()
        / n;
    let q4 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.golden_allergy.intensity)
        .sum::<f64>()
        / n;
    // Altitude lines: 0=grief, 3=transgression (development.rs mapping).
    let alt_grief = sim
        .agents
        .iter()
        .map(|a| a.development.altitudes.first().copied().unwrap_or(0.0))
        .sum::<f64>()
        / n;
    let alt_trans = sim
        .agents
        .iter()
        .map(|a| a.development.altitudes.get(3).copied().unwrap_or(0.0))
        .sum::<f64>()
        / n;
    println!(
        "pest seed={seed} t={ticks}: griefs={griefs} violations={violations} | \
         Q1={q1:.3} Q2={q2:.3} Q3={q3:.3} Q4={q4:.3} | alt_grief={alt_grief:.3} alt_trans={alt_trans:.3}"
    );
}

fn main() {
    println!("== i289 magnitude-ratification verdict (pestilence, N=12, 20K) ==");
    for seed in [42_u64, 1, 7] {
        run(seed, 20_000);
    }
}
