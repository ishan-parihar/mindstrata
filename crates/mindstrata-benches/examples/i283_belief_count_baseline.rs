//! i283 belief-count baseline — measures the natural belief distribution
//! per agent across 12 seeds × 2000 ticks. This probe determines whether
//! adding 19 per-line beliefs (one per registered line) for the polarity
//! → belief data path is a 2× or 10× increase in belief count.

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEEDS: [u64; 6] = [1, 7, 21, 42, 99, 12345];
const TICKS: u64 = 2000;

fn main() {
    let mut counts: Vec<f64> = Vec::new();
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
        let mut total = 0usize;
        let mut max = 0usize;
        for agent in &sim.agents {
            total += agent.beliefs.len();
            if agent.beliefs.len() > max {
                max = agent.beliefs.len();
            }
        }
        let mean = total as f64 / sim.agents.len() as f64;
        counts.push(mean);
        println!("seed={seed} mean_beliefs={mean:.2} max_beliefs={max}");
    }
    let sum: f64 = counts.iter().sum();
    let mean = sum / counts.len() as f64;
    let var: f64 = counts.iter().map(|x| (x - mean).powi(2)).sum::<f64>() / counts.len() as f64;
    let stddev = var.sqrt();
    println!(
        "belief_count_per_agent mean={mean:.2} stddev={stddev:.2} cv={:.3}",
        stddev / mean
    );
    println!("verdict=BELIEF_COUNT_BASELINE");
}
