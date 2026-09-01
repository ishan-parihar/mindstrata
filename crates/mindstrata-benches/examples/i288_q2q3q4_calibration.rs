//! Q2/Q3/Q4 quadrant trajectory probe — measure the dark_allergy,
//! golden_addiction, golden_allergy trajectories to verify that
//! `OperatorParams::pending()` symmetrically fits all four quadrants
//! at the i269/i287-observed event mix.

use mindstrata_development::dynamics::{Metabolism, OperatorParams, QuadrantState};
use mindstrata_sim::sim::{AgentBundle, SimConfig};
use mindstrata_sim::Simulation;

fn make_sim() -> Simulation {
    let cfg = SimConfig {
        seed: 42,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(cfg);
    sim.populate();
    sim
}

fn stat_for(sim: &Simulation, accessor: fn(&AgentBundle) -> f64) -> (f64, f64) {
    let vals: Vec<f64> = sim.agents.iter().map(accessor).collect();
    if vals.is_empty() {
        return (0.0, 0.0);
    }
    let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    (mean, max)
}

fn main() {
    let mut sim = make_sim();

    let (init_da, _) = stat_for(&sim, |a| a.development.pathology.dark_addiction.intensity);
    let (init_dl, _) = stat_for(&sim, |a| a.development.pathology.dark_allergy.intensity);
    let (init_ga, _) = stat_for(&sim, |a| a.development.pathology.golden_addiction.intensity);
    let (init_gl, _) = stat_for(&sim, |a| a.development.pathology.golden_allergy.intensity);
    println!("ic4_q_all_init da={init_da:.6} dl={init_dl:.6} ga={init_ga:.6} gl={init_gl:.6}");

    for _ in 0..5_000 {
        sim.tick();
    }
    let (m5_da, x5_da) = stat_for(&sim, |a| a.development.pathology.dark_addiction.intensity);
    let (m5_dl, _) = stat_for(&sim, |a| a.development.pathology.dark_allergy.intensity);
    let (m5_ga, _) = stat_for(&sim, |a| a.development.pathology.golden_addiction.intensity);
    let (m5_gl, _) = stat_for(&sim, |a| a.development.pathology.golden_allergy.intensity);
    println!(
        "ic4_q_5k da_mean={m5_da:.4} dl_mean={m5_dl:.4} ga_mean={m5_ga:.4} gl_mean={m5_gl:.4}"
    );
    println!("ic4_q_5k da_max={x5_da:.4}");

    for _ in 0..15_000 {
        sim.tick();
    }
    let (m20_da, x20_da) = stat_for(&sim, |a| a.development.pathology.dark_addiction.intensity);
    let (m20_dl, _) = stat_for(&sim, |a| a.development.pathology.dark_allergy.intensity);
    let (m20_ga, _) = stat_for(&sim, |a| a.development.pathology.golden_addiction.intensity);
    let (m20_gl, _) = stat_for(&sim, |a| a.development.pathology.golden_allergy.intensity);
    println!(
        "ic4_q_20k da_mean={m20_da:.4} dl_mean={m20_dl:.4} ga_mean={m20_ga:.4} gl_mean={m20_gl:.4}"
    );
    println!("ic4_q_20k da_max={x20_da:.4}");

    // Steady-state predictions at the 4 catalyst magnitudes via pending()
    // (g=0.05, d=0.02, c=1.0) — same shape for all 4 quadrants.
    let pending = OperatorParams::pending();
    for (name, mag) in [
        ("grief=1.0", 1.0_f64),
        ("bond=0.7", 0.7),
        ("threat=0.4", 0.4),
        ("transgression=0.5", 0.5),
        ("low_pressure=0.2", 0.2),
        ("trace=0.05", 0.05),
    ] {
        let mut q_addiction = QuadrantState::neutral();
        let mut q_allergy = QuadrantState::neutral();
        for _ in 0..20_000 {
            q_addiction = q_addiction.step(Metabolism::Addiction, mag, &pending);
            q_allergy = q_allergy.step(Metabolism::Allergy, mag, &pending);
        }
        println!(
            "ic4_q_predicted {name} addiction={:.4} allergy={:.4}",
            q_addiction.intensity, q_allergy.intensity
        );
    }
    println!("verdict=Q2Q3Q4_CALIBRATION_DONE");
}
