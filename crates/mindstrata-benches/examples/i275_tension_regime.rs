//! i275 part 2 — tension-count regime measurement for the action-bias
//! re-derivation. The 0.10 coefficient was ratified (i282/DC-2.4) against a
//! 0–3 ActiveTension-per-agent regime; the severity-grounded projection
//! changes the regime. This probe measures per-agent tension counts at the
//! decision horizons so the coefficient can preserve the RATIFIED CONTRACT
//! ("bias = 5–20% of the social driver", ≈0–0.03 utility), not the constant.

use mindstrata_development::polarity::PolarityState;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn tension_stats(ticks: u64) -> (f64, f64, f64, usize) {
    let config = SimConfig {
        seed: 42,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    let counts: Vec<usize> = sim
        .agents
        .iter()
        .map(|a| {
            a.polarity_claims
                .iter()
                .filter(|c| c.polarity == PolarityState::ActiveTension)
                .count()
        })
        .collect();
    let n = counts.len().max(1) as f64;
    let mean = counts.iter().sum::<usize>() as f64 / n;
    let max = *counts.iter().max().unwrap_or(&0) as f64;
    let integrated: usize = sim
        .agents
        .iter()
        .map(|a| {
            a.polarity_claims
                .iter()
                .filter(|c| c.polarity == PolarityState::Integrated)
                .count()
        })
        .sum();
    (mean, max, counts.iter().sum::<usize>() as f64, integrated)
}

fn main() {
    for ticks in [1000_u64, 2000, 5000, 20_000] {
        let (mean, max, total, integrated) = tension_stats(ticks);
        println!(
            "ticks={ticks:>6}  mean_tension/agent={mean:7.2}  max={max:5.0}  total={total:6.0}  integrated={integrated}"
        );
    }
}
// NOTE: extended count including Integrated claims (i275 follow-up).
