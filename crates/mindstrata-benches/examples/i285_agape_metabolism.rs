//! Iter-285 probe — WP-H3 agape metabolism: mourning rites as Agape-metabolizer
//! vehicles (the wave-brief's i291 behavioral test).
//!
//! Mechanism under test: a death GENERATES a one-shot Funeral rite binding the
//! grief-target set; the executor fires it at the next duodeca boundary and the
//! development pass reads `MourningObserved` as Agape pressure — the ONLY
//! consumption channel on an Allergy quadrant (`− decay × pressure × intensity`;
//! absence GROWS it). So post-casualty villages WITH the channel should show
//! faster Q4 (Golden-Allergy, the Grief-routed quadrant) decay than controls.
//!
//! Method: same seed/horizon, channel ON vs OFF via `MINDSTRATA_MOURNING_RITES=0`
//! (the production gate). Measure: rites executed, Q4 mean/peak trajectory at
//! 5K/10K/20K, and per-death metabolizer dose coverage. Zero-at-zero control:
//! calm scenario (no deaths) must be byte-identical ON vs OFF.

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn q4_stats(sim: &Simulation) -> (f64, f64, usize) {
    let n = sim.agents.len().max(1) as f64;
    let mut sum = 0.0;
    let mut peak = 0.0f64;
    let mut nonzero = 0usize;
    for a in &sim.agents {
        let v = a.development.pathology.golden_allergy.intensity;
        sum += v;
        peak = peak.max(v);
        if v > 0.0 {
            nonzero += 1;
        }
    }
    (sum / n, peak, nonzero)
}

fn run(sc: Scenario, label: &str) {
    let horizon = 20_000u64;
    let mut sc = sc;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    let mut rites_cum = 0usize;
    let mut deaths_cum = 0usize;
    for h in [5_000u64, 10_000, 20_000] {
        let before = sim
            .recent_events(sim.event_count() as usize)
            .iter()
            .filter(|e| matches!(e, mindstrata_core::event::SimEvent::MourningObserved { .. }))
            .count();
        let deaths_before = sim
            .recent_events(sim.event_count() as usize)
            .iter()
            .filter(|e| matches!(e, mindstrata_core::event::SimEvent::AgentDied { .. }))
            .count();
        sim.run(h);
        // The event journal is bounded — count per-window deltas since the
        // journal may have trimmed, accumulating our own census.
        let now = sim
            .recent_events(sim.event_count() as usize)
            .iter()
            .filter(|e| matches!(e, mindstrata_core::event::SimEvent::MourningObserved { .. }))
            .count();
        let deaths_now = sim
            .recent_events(sim.event_count() as usize)
            .iter()
            .filter(|e| matches!(e, mindstrata_core::event::SimEvent::AgentDied { .. }))
            .count();
        rites_cum += now.saturating_sub(before);
        deaths_cum += deaths_now.saturating_sub(deaths_before);
        let (mean, peak, nz) = q4_stats(&sim);
        println!(
            "{label:>11} @{h:>5}: q4_mean={mean:.4} q4_peak={peak:.4} q4_nonzero={nz:>2}  deaths≥{deaths_cum} rites≥{rites_cum}"
        );
    }
}

fn main() {
    let enabled = std::env::var("MINDSTRATA_MOURNING_RITES").map_or(true, |v| v != "0");
    println!(
        "mourning-rite channel: {}",
        if enabled { "ON" } else { "OFF" }
    );
    run(Scenario::calm(), "calm");
    run(Scenario::pestilence(), "pestilence");
}
