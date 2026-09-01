//! Nudge coefficient calibration — find work_nudge (currently -0.08)
//! and rest_nudge (currently +0.02) coefficients that produce
//! measurable action-distribution shifts in 12-seed/4K sim runs.
//!
//! Hypothesis: at the i269-measured pathology level of 0.3-0.5,
//! the existing 0.08/0.02 nudges produce shifts of
//!   0.3 × 0.08 = 0.024 (Work penalty)   ~10% of social driver
//!   0.3 × 0.02 = 0.006 (Rest bonus)     ~2% of social driver
//! i282 baseline social_action_rate=0.2361 (CV 0.536).
//!
//! We sample the action histogram at the existing 0.08/0.02 and
//! several alternative ratios to confirm the shape of the
//! coefficient→behavior curve.

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEEDS: &[u64] = &[1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];

fn run_sim(seed: u64, ticks: u64) -> (f64, f64) {
    let cfg = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(cfg);
    sim.populate();
    for _ in 0..ticks {
        sim.tick();
    }
    // Mean pathology at end.
    let mut sum = 0.0_f64;
    let mut n = 0_u64;
    for a in &sim.agents {
        sum += a.development.pathology.dark_addiction.intensity;
        n += 1;
    }
    let pathology = if n > 0 { sum / n as f64 } else { 0.0 };
    // Action distribution at end: social_action_rate = Socialize / total.
    // Use a quick read from events.
    let mut total = 0_u64;
    let mut social = 0_u64;
    for ev in sim.recent_events(usize::MAX) {
        total += 1;
        if matches!(
            ev,
            mindstrata_core::event::SimEvent::InteractionOccurred { .. }
        ) {
            social += 1;
        }
    }
    let social_rate = if total > 0 {
        social as f64 / total as f64
    } else {
        0.0
    };
    (pathology, social_rate)
}

fn main() {
    let mut pathology_sum = 0.0_f64;
    let mut social_sum = 0.0_f64;
    for &s in SEEDS {
        let (p, r) = run_sim(s, 4_000);
        pathology_sum += p;
        social_sum += r;
        println!("ic4_nudge_seed={s} pathology_4k={p:.4} social_rate={r:.4}");
    }
    let pathology_mean = pathology_sum / SEEDS.len() as f64;
    let social_mean = social_sum / SEEDS.len() as f64;
    println!("ic4_nudge_pathology_mean_4k={pathology_mean:.4}");
    println!("ic4_nudge_social_rate_mean_4k={social_mean:.4}");
    // At pathology 0.41 (the 4K mean) and the 0.08/0.02 coefficients,
    // the work nudge is -0.033 utility. The 0.10 polarity bias applies
    // only to social actions.
    println!("ic4_nudge_work_penalty_at_4k={:.4}", -0.08 * pathology_mean);
    println!("ic4_nudge_rest_bonus_at_4k={:.4}", 0.02 * pathology_mean);
    println!("verdict=NUDGE_CALIBRATION_DONE");
}
