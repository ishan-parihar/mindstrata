//! Verify golden-window prediction errors stay under the 0.3 fold gate.
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};
fn main() {
    for (name, ticks, seed) in [
        ("golden riverford@1000", 1000u64, 42u64),
        ("golden riverford@2000", 2000u64, 42u64),
        ("calm@500", 500u64, 42u64),
        ("calm@2000", 2000u64, 42u64),
        ("calm@10000", 10_000u64, 42u64),
    ] {
        let mut sim = if name.contains("riverford") {
            let mut sc = Scenario::riverford();
            sc.seed = seed;
            Simulation::from_scenario(sc)
        } else {
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: ticks,
                world_width: 16,
                world_height: 16,
                num_agents: 12,
                snapshot_interval: None,
            });
            sim.populate();
            sim
        };
        if name.contains("calm") {
            sim.populate();
        }
        sim.run(ticks);
        let mut max_err = 0.0f64;
        let mut over = 0usize;
        for a in &sim.agents {
            let e = a.neural_like.expectation.last_prediction_error.to_f64();
            max_err = max_err.max(e);
            if e > 0.3 {
                over += 1;
            }
        }
        println!("{name}: max_pred_err={max_err:.4} agents_over_0.3={over}");
    }
}
