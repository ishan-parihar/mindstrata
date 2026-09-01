//! i293 — sweep seeds for Q2/Q3/Q4 activity (5K ticks, find firing seed).
//! Prints per-seed Q1-Q4 mean intensities to identify seed that fires
//! dark_allergy / golden quadrants.  Q1 should fire at most seeds
//! (Threat catalysts); Q2 needs Transgression+norm violation,
//! Q3/Q4 need Bond/Grief golden catalysts.

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    let seeds = [
        1u64, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345, 999, 2026, 4242, 7777, 100, 200, 300,
        400,
    ];
    println!("seed,q1_dark_addiction,q2_dark_allergy,q3_golden_addiction,q4_golden_allergy,events,agent_count");
    for seed in seeds {
        let cfg = SimConfig {
            seed,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(cfg);
        sim.populate();
        sim.run(5000);
        let n = sim.agents.len() as f64;
        let inv = if n > 0.0 { 1.0 / n } else { 0.0 };
        let q1: f64 = sim
            .agents
            .iter()
            .map(|a| a.development.pathology.dark_addiction.intensity)
            .sum::<f64>()
            * inv;
        let q2: f64 = sim
            .agents
            .iter()
            .map(|a| a.development.pathology.dark_allergy.intensity)
            .sum::<f64>()
            * inv;
        let q3: f64 = sim
            .agents
            .iter()
            .map(|a| a.development.pathology.golden_addiction.intensity)
            .sum::<f64>()
            * inv;
        let q4: f64 = sim
            .agents
            .iter()
            .map(|a| a.development.pathology.golden_allergy.intensity)
            .sum::<f64>()
            * inv;
        println!(
            "{},{:.4},{:.4},{:.4},{:.4},{},{}",
            seed,
            q1,
            q2,
            q3,
            q4,
            sim.event_count(),
            sim.agents.len()
        );
    }
    // Verdict: find max Q2/Q3/Q4 across seeds
    println!("i293 done — look for max Q2/Q3/Q4 > 0.01 to identify firing seed");
}
