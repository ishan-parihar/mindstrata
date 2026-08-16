//! Distinguish gate-closure from weak-knob for appraisal_fear_coping_multiplier
//! in the pestilence window: run 1x vs 10x and compare the FULL snapshot.
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn main() {
    let run = |mult: f64| {
        let mut sc = Scenario::pestilence();
        sc.ticks = 5000;
        let mut sim = Simulation::from_scenario(sc);
        sim.params.appraisal_fear_coping_multiplier = Fixed::from_f64(mult);
        sim.populate();
        sim.run(5000);
        let ms = sim.metrics_snapshot();
        let fear: Vec<f64> = sim.agents.iter().map(|a| a.emotions.fear.to_f64()).collect();
        let mean = fear.iter().sum::<f64>() / fear.len() as f64;
        // Full-snapshot byte-identity via Debug (MetricsSnapshot has no JSON
        // dep in benches; Debug is total and order-stable for fixed-size f64s).
        (ms.avg_fear, mean, format!("{ms:?}"))
    };
    for mult in [1.0, 2.0, 4.0, 6.0, 8.0, 10.0, 12.0] {
        let (a, m, _) = run(mult);
        println!("mult={mult:>4}: avg_fear={a:.6} mean={m:.6}");
    }
}
