//! H6 mechanism audit probe (DC-1) — measures which needs actually dominate
//! across the seed family, and which of the 19 needs have live (non-saturated)
//! pressure at the end of 2000 ticks.
//!
//! Run: cargo run --release -p mindstrata-benches --example i279_dominant_need_audit

use mindstrata_sim::psychology::motivation::MotiveCategory;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 2000;

fn need_pressure(agent: &mindstrata_sim::sim::AgentBundle, n: MotiveCategory) -> f64 {
    agent.motivation.pressure_full(n).to_f64()
}

fn main() {
    let mut rows = Vec::new();
    for &seed in &SEEDS {
        let config = SimConfig {
            seed,
            max_ticks: TICKS,
            num_agents: 12,
            snapshot_interval: None,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(TICKS);

        // Count dominant_need frequency across the run.
        let mut counts: std::collections::HashMap<&'static str, usize> =
            std::collections::HashMap::new();
        for a in &sim.agents {
            // We can't read tick-by-tick, so we read the final state and
            // extrapolate which needs are saturated (the dead-channel signal).
            let labels: &[(&str, f64)] = &[
                ("hunger", need_pressure(a, MotiveCategory::Hunger)),
                ("thirst", need_pressure(a, MotiveCategory::Thirst)),
                ("sleep", need_pressure(a, MotiveCategory::Sleep)),
                ("warmth", need_pressure(a, MotiveCategory::Warmth)),
                ("health", need_pressure(a, MotiveCategory::Health)),
                ("safety", need_pressure(a, MotiveCategory::Safety)),
                ("attachment", need_pressure(a, MotiveCategory::Attachment)),
                ("belonging", need_pressure(a, MotiveCategory::Belonging)),
                ("esteem", need_pressure(a, MotiveCategory::Esteem)),
                ("autonomy", need_pressure(a, MotiveCategory::Autonomy)),
                ("competence", need_pressure(a, MotiveCategory::Competence)),
                ("meaning", need_pressure(a, MotiveCategory::Meaning)),
                ("certainty", need_pressure(a, MotiveCategory::Certainty)),
                ("novelty", need_pressure(a, MotiveCategory::Novelty)),
                ("play", need_pressure(a, MotiveCategory::Play)),
                ("care", need_pressure(a, MotiveCategory::Care)),
                ("romance", need_pressure(a, MotiveCategory::Romance)),
                ("justice", need_pressure(a, MotiveCategory::Justice)),
                ("recognition", need_pressure(a, MotiveCategory::Recognition)),
            ];
            for (label, pressure) in labels {
                if *pressure > 0.95 {
                    *counts.entry(*label).or_insert(0) += 1;
                }
            }
        }
        // Record: how many agents (out of 12) saturate each need.
        let mut sorted: Vec<(&&str, &usize)> = counts.iter().collect();
        sorted.sort_by(|x, y| y.1.cmp(x.1));
        for (need, n_sat) in sorted.iter().take(3) {
            println!(
                "seed={seed} need={need} n_saturated_agents={n_sat}/12 threshold=0.95 class=saturated"
            );
        }
        rows.push((seed, counts));
    }
    // Aggregate: which needs saturate most often across all 12 seeds.
    let mut total: std::collections::HashMap<&'static str, usize> =
        std::collections::HashMap::new();
    for (_seed, counts) in &rows {
        for (n, c) in counts {
            *total.entry(*n).or_insert(0) += c;
        }
    }
    let mut sorted: Vec<(&&str, &usize)> = total.iter().collect();
    sorted.sort_by(|x, y| y.1.cmp(x.1));
    println!("\n--- aggregate saturation across {} seeds × 12 agents = {} agent-ticks ---", SEEDS.len(), SEEDS.len() * 12);
    for (need, count) in sorted {
        let rate = *count as f64 / (SEEDS.len() as f64 * 12.0);
        println!("need={need} sat_count={count} rate={rate:.3}");
    }
}
