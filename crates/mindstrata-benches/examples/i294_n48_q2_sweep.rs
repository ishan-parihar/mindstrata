//! i294 — N=48 20K Q2/Q4 check (does larger N fire Transgression/Grief at scale?).
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    for seed in [42u64, 1, 7] {
        for n in [12usize, 48] {
            let cfg = SimConfig {
                seed,
                max_ticks: 20000,
                world_width: 24,
                world_height: 24,
                num_agents: n as u32,
                snapshot_interval: None,
            };
            let mut sim = Simulation::new(cfg);
            sim.populate();
            sim.run(20000);
            let inv = if sim.agents.is_empty() {
                0.0
            } else {
                1.0 / sim.agents.len() as f64
            };
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
                "seed={} N={} q1={:.4} q2={:.4} q3={:.4} q4={:.4} events={} agents={}",
                seed,
                n,
                q1,
                q2,
                q3,
                q4,
                sim.event_count(),
                sim.agents.len()
            );
        }
    }
    println!("i294 done");
}
