//! Polarity bias 4K/12-seed family baseline — measure the social
//! action rate with the 0.10 ActiveTension-count bias active.
//! i282 captured the 2K baseline (mean 0.2361, CV 0.536); this probe
//! extends to 4K and 12 seeds for the i268-style family sweep.

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
    // Count ActiveTension polarity claims per agent at end.
    let mut claim_sum = 0_u64;
    let mut tension_sum = 0_u64;
    let mut agent_count = 0_u64;
    for a in &sim.agents {
        agent_count += 1;
        for c in &a.polarity_claims {
            claim_sum += 1;
            if matches!(
                c.polarity,
                mindstrata_development::polarity::PolarityState::ActiveTension
            ) {
                tension_sum += 1;
            }
        }
    }
    let avg_claims = if agent_count > 0 {
        claim_sum as f64 / agent_count as f64
    } else {
        0.0
    };
    let avg_tension = if agent_count > 0 {
        tension_sum as f64 / agent_count as f64
    } else {
        0.0
    };
    // Action distribution: social = InteractionOccurred / total events.
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
    (avg_tension, social_rate)
}

fn main() {
    let mut tension_sum = 0.0_f64;
    let mut social_sum = 0.0_f64;
    for &s in SEEDS {
        let (t, r) = run_sim(s, 4_000);
        tension_sum += t;
        social_sum += r;
        println!("ic4_polarity_seed={s} avg_active_tension={t:.4} social_rate={r:.4}");
    }
    let tension_mean = tension_sum / SEEDS.len() as f64;
    let social_mean = social_sum / SEEDS.len() as f64;
    println!("ic4_polarity_active_tension_mean_4k={tension_mean:.4}");
    println!("ic4_polarity_social_rate_mean_4k={social_mean:.4}");
    // The 0.10 coefficient × avg_active_tension × social_value is the
    // bias utility. At tension_mean=0.05 and social_value=0.5, the
    // bias is 0.10 × 0.05 × 0.5 = 0.0025 — sub-noise.
    println!(
        "ic4_polarity_bias_utility_est={:.6}",
        0.10 * tension_mean * 0.5
    );
    println!("verdict=POLARITY_4K_BASELINE_DONE");
}
