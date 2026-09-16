//! i268 — forced-Bond Q3 golden_addiction calibration (PLAN_DC2 Iter-268).
//!
//! i293 measured Q3 at 0.0329–0.0414 mean at 5K/12 natural event rates —
//! live but far from the ratified plateau band 0.70–0.90. This probe
//! measures the Q3 trajectory under a forced Bond-catalyst regime
//! (synthetic MarriageFormed/ChildBorn events fed directly to the pure
//! `system_development` pass) to answer:
//!   1. Does Q3 reach the 0.70–0.90 ceiling band when Bond pressure is
//!      sustained? (calibration evidence for growth/decay)
//!   2. What is the natural-regime equilibrium vs forced-regime
//!      equilibrium (event-rate sensitivity)?
//!   3. Q3 altitude coupling: does the Bond line altitude advance
//!      concurrently (the "premature reach" discriminant)?

use mindstrata_core::event::SimEvent;
use mindstrata_core::{AgentId, Tick};
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::systems::development::system_development;
use mindstrata_sim::Simulation;

/// Per-agent Q3/altitude probe over a run.
struct Q3Reading {
    /// Time-averaged mean intensity across agents.
    mean_intensity: f64,
    /// Final-tick mean intensity.
    final_mean: f64,
    /// Max per-tick mean observed.
    peak_mean: f64,
    /// Final mean altitude on the Bond line (index 1).
    final_altitude_bond: f64,
    /// Agents whose Q3 exceeded 0.5 at any sample point.
    agents_above_half: usize,
}

fn run_with_bond_forcing(seed: u64, force_every: Option<u64>, horizon: u64) -> Q3Reading {
    let cfg = SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(cfg);
    sim.populate();

    let mut sum = 0.0f64;
    let mut peak = 0.0f64;
    let mut above_half = std::collections::HashSet::new();
    let mut samples = 0u64;

    for t in 0..horizon {
        sim.tick();

        if let Some(every) = force_every {
            if t % every == 0 {
                // Deterministic forced-Bond window: marriages + childbirths
                // rotate over the first 4 agents (2 couples). Fed straight to
                // the pure daily pass — production tick() is untouched.
                let events: Vec<SimEvent> = (0..2)
                    .map(|c| SimEvent::MarriageFormed {
                        spouse_a: AgentId::new(c * 2),
                        spouse_b: AgentId::new(c * 2 + 1),
                        tick: Tick::new(t),
                    })
                    .chain((0..2).map(|c| SimEvent::ChildBorn {
                        child: AgentId::new(8 + c),
                        parent_a: AgentId::new(c * 2),
                        parent_b: AgentId::new(c * 2 + 1),
                        tick: Tick::new(t),
                    }))
                    .collect();
                system_development(&mut sim.agents, &events);
            }
        }

        let n = sim.agents.len() as f64;
        if n > 0.0 {
            let tick_mean: f64 = sim
                .agents
                .iter()
                .map(|a| a.development.pathology.golden_addiction.intensity)
                .sum::<f64>()
                / n;
            sum += tick_mean;
            peak = peak.max(tick_mean);
            samples += 1;
            for (i, a) in sim.agents.iter().enumerate() {
                if a.development.pathology.golden_addiction.intensity > 0.5 {
                    above_half.insert(i);
                }
            }
        }
    }

    let n = sim.agents.len() as f64;
    let final_mean: f64 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.golden_addiction.intensity)
        .sum::<f64>()
        / n.max(1.0);
    let final_altitude_bond: f64 = if sim.agents.is_empty() {
        0.0
    } else {
        sim.agents
            .iter()
            .map(|a| a.development.altitudes.get(1).copied().unwrap_or(0.0))
            .sum::<f64>()
            / n
    };

    Q3Reading {
        mean_intensity: if samples > 0 {
            sum / samples as f64
        } else {
            0.0
        },
        final_mean,
        peak_mean: peak,
        final_altitude_bond,
        agents_above_half: above_half.len(),
    }
}

fn main() {
    println!("=== i268 forced-Bond Q3 calibration (N=12) ===");
    println!(
        "{:>18} {:>10} {:>10} {:>10} {:>10} {:>8}",
        "regime", "mean", "final", "peak", "bond_alt", ">0.5"
    );

    for seed in [42u64, 4242, 2026] {
        let r = run_with_bond_forcing(seed, None, 5000);
        println!(
            "{:>18} {:>10.4} {:>10.4} {:>10.4} {:>10.4} {:>8}",
            format!("natural s{seed}"),
            r.mean_intensity,
            r.final_mean,
            r.peak_mean,
            r.final_altitude_bond,
            r.agents_above_half
        );
        // Forced regimes: Bond catalysts every 50 and every 200 ticks.
        for every in [200u64, 50] {
            let r = run_with_bond_forcing(seed, Some(every), 5000);
            println!(
                "{:>18} {:>10.4} {:>10.4} {:>10.4} {:>10.4} {:>8}",
                format!("forced/{every}s{seed}"),
                r.mean_intensity,
                r.final_mean,
                r.peak_mean,
                r.final_altitude_bond,
                r.agents_above_half
            );
        }
    }

    println!("verdict=I268_FORCED_BOND_Q3_DONE");
}
