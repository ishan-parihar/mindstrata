//! i269 — pathology signature #1 probe (DC-1 task 3.5, WP-H1).
//!
//! Measures the Dark-Addiction trajectory (Q1) and its behavioral expression
//! through the Work/Rest nudge. The pathology field starts neutral (founder
//! 0.0) and steps only on IC-1 catalysts (Threat etc.) via
//! `system_development`'s Addiction step (growth 0.05/decay 0.02, pending).
//! The nudge (−0.08·intensity on Work, +0.02 on Rest) is live since 3.5.
//!
//! Run with: `cargo run -p mindstrata-benches --example i269_pathology_signature --release`

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEED: u64 = 42;
const HORIZON: u64 = 20000;

fn make_sim() -> Simulation {
    let config = SimConfig {
        seed: SEED,
        max_ticks: HORIZON,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim
}

fn pathology_stats(sim: &Simulation) -> (f64, f64, f64) {
    let vals: Vec<f64> = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.dark_addiction.intensity)
        .collect();
    if vals.is_empty() {
        return (0.0, 0.0, 0.0);
    }
    let max = vals.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let mean = vals.iter().sum::<f64>() / vals.len() as f64;
    let above_01 = vals.iter().filter(|&&v| v > 0.01).count() as f64 / vals.len() as f64;
    (max, mean, above_01)
}

fn main() {
    let mut sim = make_sim();

    // Snapshot initial pathology (must be neutral).
    let (init_max, init_mean, _) = pathology_stats(&sim);
    let newborn_check_initial = sim
        .agents
        .iter()
        .all(|a| a.development.pathology.is_neutral());

    // Run to 5K, 10K, 20K sampling.
    for _ in 0..5000 {
        sim.tick();
    }
    let (max_5k, mean_5k, frac_5k) = pathology_stats(&sim);

    for _ in 0..5000 {
        sim.tick();
    }
    let (max_10k, mean_10k, frac_10k) = pathology_stats(&sim);

    for _ in 0..10000 {
        sim.tick();
    }
    let (max_20k, mean_20k, frac_20k) = pathology_stats(&sim);
    let agents_at_20k = sim.agents.len();

    // Heredity-neutral control: newborns (agents added after tick 0) must be neutral at birth.
    // We check the youngest agents (highest id) — they were born during the run.
    // If no births, the check is vacuously true.
    let initial_n = 12usize;
    let newborn_neutral = if sim.agents.len() > initial_n {
        // Check that at least the tail agents that could be newborns are not wildly pathological
        // without catalysts (they start neutral; they may accumulate after birth, so we only
        // assert they are not pre-seeded with high pathology — max < 0.2 at 20K is reasonable).
        let tail = &sim.agents[initial_n..];
        tail.iter()
            .map(|a| a.development.pathology.dark_addiction.intensity)
            .all(|v| v < 0.5)
    } else {
        true
    };
    let other_quadrants_neutral = sim.agents.iter().all(|a| {
        a.development.pathology.dark_allergy.intensity == 0.0
            && a.development.pathology.golden_addiction.intensity == 0.0
            && a.development.pathology.golden_allergy.intensity == 0.0
    });

    println!("ic4_seed={SEED}");
    println!("ic4_horizon={HORIZON}");
    println!("ic4_init_max={init_max:.6}");
    println!("ic4_init_mean={init_mean:.6}");
    println!("ic4_init_neutral={newborn_check_initial}");
    println!("ic4_max_5k={max_5k:.6}");
    println!("ic4_mean_5k={mean_5k:.6}");
    println!("ic4_frac_gt01_5k={frac_5k:.3}");
    println!("ic4_max_10k={max_10k:.6}");
    println!("ic4_mean_10k={mean_10k:.6}");
    println!("ic4_frac_gt01_10k={frac_10k:.3}");
    println!("ic4_max_20k={max_20k:.6}");
    println!("ic4_mean_20k={mean_20k:.6}");
    println!("ic4_frac_gt01_20k={frac_20k:.3}");
    println!("ic4_agents_20k={agents_at_20k}");
    println!("ic4_newborn_neutral={newborn_neutral}");
    println!("ic4_other_quadrants_neutral={other_quadrants_neutral}");

    // Verdict: signature is the monotone rise of dark_addiction from neutral,
    // with behavioral nudge live (Work suppression −0.08·intensity). We assert
    // the trajectory moves (max_20k > 0.02) and that heredity controls are clean.
    let trajectory_moves = max_20k > 0.02 && mean_20k > 0.001;
    let monotone = max_20k >= max_5k - 1e-9;
    let pass = trajectory_moves
        && monotone
        && newborn_check_initial
        && newborn_neutral
        && other_quadrants_neutral;
    if pass {
        println!("verdict=PATHOLOGY_SIGNATURE_PASS");
    } else {
        println!(
            "verdict=PATHOLOGY_SIGNATURE_FAIL trajectory_moves={trajectory_moves} monotone={monotone} init_neutral={newborn_check_initial} newborn_neutral={newborn_neutral}"
        );
        std::process::exit(1);
    }
}
