//! i289 — A2: Grief/Threat catalyst magnitude ratification sweep
//! (PLAN_DC3 §3.1 A2; the i270 debt "Grief observation requires a
//! mortality-horizon probe" — i285 proved pestilence generates deaths at
//! N=12, so the observer harness can finally see them).
//!
//! Method: run the mortality scenarios (pestilence, collapse) across seeds
//! and horizons; census the live event stream through the SAME mapping the
//! development pass consumes (`collect_catalysts`), measuring:
//!   - Grief count + observed magnitude (spec pin: 1.0, "maximal-loss
//!     exemplar")
//!   - Threat count + magnitude distribution (spec: injury/fear-coupled
//!     0.311–0.661 observed at i270; re-measure at mortality horizons)
//!   - Grief subject validity (mourners must be distinct live agents)
//!   - Downstream: Q4 (golden allergy) mean at horizons — the Grief-routed
//!     quadrant the magnitudes feed
//!
//! Verdict rules (§4.2): if the observed Grief magnitude is constant at the
//! pin (1.0, single-event semantics), ratify; if it spreads, re-contract to
//! the measured distribution. Threat: confirm the i270 band still holds at
//! mortality horizons (larger conflict volumes, feud cascades).

use mindstrata_core::event::SimEvent;
use mindstrata_development::catalyst::CatalystKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

/// In-probe replication of `collect_catalysts`' Grief/Threat branches (the
/// function is `pub(crate)`; the i270 precedent replicates the mapping in
/// probes to keep the production hot-path private). Kept adjacent to the
/// production match arms for drift review.
fn census_grief_threat(events: &[SimEvent]) -> (Vec<f64>, Vec<f64>) {
    let mut grief = Vec::new();
    let mut threat = Vec::new();
    for ev in events {
        match *ev {
            SimEvent::GriefStruck { .. } => grief.push(1.0),
            SimEvent::FeudFormed { .. } => {
                threat.push(0.4);
                threat.push(0.4);
            }
            SimEvent::ConflictOccurred {
                injury,
                fear_induced,
                ..
            } => {
                let base = 0.3 + injury.to_f64().clamp(0.0, 0.5);
                threat.push(base.min(1.0));
                threat.push((base + fear_induced.to_f64().clamp(0.0, 0.2)).min(1.0));
            }
            _ => {}
        }
    }
    (grief, threat)
}

fn run(name: &str, seed: u64, ticks: u64) {
    let mut sc = match name {
        "pestilence" => Scenario::pestilence(),
        _ => Scenario::collapse(),
    };
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(ticks);

    let events = sim.recent_events(sim.event_count()).to_vec();
    let deaths = events
        .iter()
        .filter(|e| matches!(e, SimEvent::AgentDied { .. }))
        .count();
    let griefs = events
        .iter()
        .filter(|e| matches!(e, SimEvent::GriefStruck { .. }))
        .count();
    let (mut grief_mag, mut threat_mag) = census_grief_threat(&events);
    grief_mag.push(0.0);
    grief_mag.pop();
    let _ = &mut threat_mag;
    let _ = CatalystKind::Grief; // vocabulary marker (IC-1 freeze)
    let q4_mean: f64 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.golden_allergy.intensity)
        .sum::<f64>()
        / sim.agents.len().max(1) as f64;
    let q1_mean: f64 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.dark_addiction.intensity)
        .sum::<f64>()
        / sim.agents.len().max(1) as f64;
    let gm = (
        grief_mag.iter().copied().sum::<f64>() / grief_mag.len().max(1) as f64,
        grief_mag.iter().copied().fold(f64::INFINITY, f64::min),
        grief_mag.iter().copied().fold(0.0_f64, f64::max),
    );
    let tm = (
        threat_mag.iter().copied().sum::<f64>() / threat_mag.len().max(1) as f64,
        threat_mag.iter().copied().fold(f64::INFINITY, f64::min),
        threat_mag.iter().copied().fold(0.0_f64, f64::max),
    );
    println!(
        "{name:>10} seed={seed} t={ticks:>5}: deaths={deaths} griefs={griefs} \
         | Grief mag mean/min/max {:.3}/{:.3}/{:.3} | Threat n={} mean/min/max {:.3}/{:.3}/{:.3} \
         | Q1={q1_mean:.3} Q4={q4_mean:.3}",
        gm.0,
        gm.1,
        gm.2,
        threat_mag.len(),
        tm.0,
        tm.1,
        tm.2,
    );
}

fn main() {
    println!("== i289 Grief/Threat magnitude observer sweep (N=12) ==");
    for ticks in [2_000_u64, 4_320, 20_000] {
        for seed in [42_u64, 1, 7] {
            run("pestilence", seed, ticks);
        }
    }
    {
        let seed = 42_u64;
        run("collapse", seed, 4_320);
    }
}
