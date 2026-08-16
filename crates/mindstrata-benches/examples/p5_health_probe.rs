//! Iteration 184 — derived-health component probe.
//! body.health = derived_health() = base × skeletal − stress×0.2 − pain×0.1
//! − sickness×0.15 − shock×0.1. Measures each component at 20K to explain
//! the health dip in calm worlds post violence-fix.

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn run(seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    for &seed in &[42u64, 7, 99, 13, 46, 55] {
        let sim = run(seed, 20_000);
        let n = sim.agents.len() as f64;
        let mut stress = 0.0;
        let mut pain = 0.0;
        let mut sickness = 0.0;
        let mut shock = 0.0;
        let mut derived = 0.0;
        let mut fear = 0.0;
        for a in &sim.agents {
            stress += a.embodied.endocrine.stress.level.to_f64();
            pain += a.embodied.nervous.pain.effective_pain().to_f64();
            sickness += a.embodied.immune.sickness_level().to_f64();
            shock += a.embodied.cardiovascular.shock_risk.to_f64();
            derived += a.embodied.derived_health().to_f64();
            fear += a.emotions.fear.to_f64();
        }
        println!(
            "seed {seed:>2} @20K: derived={:.2} stress={:.2} pain={:.2} sickness={:.2} shock={:.2} fear={:.2}",
            derived / n, stress / n, pain / n, sickness / n, shock / n, fear / n
        );
    }
}
