//! i428 — measured re-anchor inputs for the four pins the v1-writer deletion
//! moved. Doctrine §4.2: a re-anchor needs measured value + old band +
//! mechanism; this probe mirrors each pin's exact contract config on the
//! post-deletion tree, across its own seed family.
//!
//! Pins covered:
//! * attachment_separation_distress_coupling (seed 42, 5K, 24×24, N=48) —
//!   nonzero-partnered share, mean, max.
//! * noospheric_belief_confidence_sustains_conviction (`run_seeded` legs:
//!   seed 42, 2K, 16×16, N=12; beliefs forced to 0.9 / 0.1 at populate; mean
//!   confidence + mean fear at 2000; the evidence-stream identity leg).
//! * peer_status_envy_feeds_daily_anger (helper `run_sim` config: 2K, 16×16,
//!   N=12; relational peer_status mean/max) on {42, 7, 11}.
//! * sensory_field_fear_contagion (2K, 16×16, N=12): perceiving-agent fold-
//!   occupancy stats vs the 0.001 quantization floor (i425).
//!
//! Run: cargo run --release -p mindstrata-benches --example i428_pin_reanchor_sweep
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn run_world(seed: u64, ticks: u64, n: u32, width: u32) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: width,
        world_height: width,
        num_agents: n,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    println!("i428 re-anchor sweep (post-writer-deletion tree):");

    println!("== attachment distress (contract config: 5K, 24×24, N=48) on {{42,7,11}}:");
    for seed in [42u64, 7, 11] {
        let sim = run_world(seed, 5000, 48, 24);
        let partnered: Vec<_> = sim.agents.iter().filter(|a| a.partner.is_some()).collect();
        let nonzero = partnered
            .iter()
            .filter(|a| a.attachment.separation_distress > Fixed::ZERO)
            .count();
        let mean = partnered
            .iter()
            .map(|a| a.attachment.separation_distress.to_f64())
            .sum::<f64>()
            / partnered.len().max(1) as f64;
        let max = partnered
            .iter()
            .map(|a| a.attachment.separation_distress.to_f64())
            .fold(0.0f64, f64::max);
        println!(
            "  seed {seed}: partnered {} nonzero {} ({:.1}%) mean {mean:.4} max {max:.4}",
            partnered.len(),
            nonzero,
            nonzero as f64 / partnered.len().max(1) as f64 * 100.0
        );
    }

    println!("== conviction differential (seed 42, 2K, 16×16, N=12; beliefs forced at populate):");
    let mut fear_pair = [0.0f64; 2];
    for (slot, conf) in [(0usize, 0.9f64), (1usize, 0.1f64)] {
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for a in &mut sim.agents {
            for b in &mut a.beliefs {
                b.confidence = Fixed::from_f64(conf);
            }
        }
        sim.run(2000);
        let beliefs: usize = sim.agents.iter().map(|a| a.beliefs.len()).sum();
        let mean: f64 = sim
            .agents
            .iter()
            .flat_map(|a| a.beliefs.iter())
            .map(|b| b.confidence.to_f64())
            .sum::<f64>()
            / beliefs.max(1) as f64;
        let mean_fear: f64 = sim
            .agents
            .iter()
            .map(|a| a.emotions.fear.to_f64())
            .sum::<f64>()
            / sim.agents.len() as f64;
        fear_pair[slot] = mean_fear;
        println!(
            "  leg conf={conf}: belief-mean {mean:.4} mean-fear {mean_fear:.6} (beliefs {beliefs})"
        );
    }
    println!(
        "  evidence-stream identity: {}",
        if fear_pair[0] == fear_pair[1] {
            "HOLDS (byte-identical)"
        } else {
            "BROKEN"
        }
    );

    println!("== peer_status (helper config: 2K, 16×16, N=12) on {{42,7,11}}:");
    for seed in [42u64, 7, 11] {
        let sim = run_world(seed, 2000, 12, 16);
        let n = sim.agents.len().max(1) as f64;
        let mean: f64 = sim
            .agents
            .iter()
            .map(|a| a.relational_fields.peer_status.to_f64())
            .sum::<f64>()
            / n;
        let max: f64 = sim
            .agents
            .iter()
            .map(|a| a.relational_fields.peer_status.to_f64())
            .fold(0.0, f64::max);
        println!("  seed {seed}: peer_status mean {mean:.4} max {max:.4}");
    }

    println!("== fear-contagion perceiving occupancy (seed 42, 2K, 16×16, N=12):");
    {
        // Step tick-by-tick; the daily boundary sample mirrors i425's probe:
        // perceived_stress is refreshed at the end of tick t−1 and read by
        // the fold during tick t, so sample-at-tick-start before tick()
        // when t % 144 == 0 (the fold cadence).
        let config = SimConfig {
            seed: 42,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let mut perceiving = 0u64;
        let mut above_floor = 0u64;
        let mut subquant = 0u64;
        let mut stress_min = f64::INFINITY;
        let mut p50s: Vec<f64> = Vec::new();
        for t in 0u64..2000 {
            if t % 144 == 0 && t > 0 {
                for a in &sim.agents {
                    let s = a.relational_fields.perceived_stress.to_f64();
                    if s > 0.0 {
                        perceiving += 1;
                        p50s.push(s);
                        stress_min = stress_min.min(s);
                        if s * 0.05 >= 5e-5 {
                            above_floor += 1;
                        } else {
                            subquant += 1;
                        }
                    }
                }
            }
            sim.tick();
        }
        p50s.sort_unstable_by(f64::total_cmp);
        let p50 = p50s.get(p50s.len() / 2).copied().unwrap_or(0.0);
        println!(
            "  perceiving agent-days {perceiving}: above-quantum-floor {above_floor}, \
             sub-quantum {subquant}; perceived-stress min {stress_min:.5}, p50 {p50:.5}"
        );
    }
}
