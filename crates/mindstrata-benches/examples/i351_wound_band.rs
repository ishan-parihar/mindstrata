//! i351 — post-driver sweep of the two i313/i314-dependent pins.
//!
//! The Wander driver re-times conflict exposure (agents roam, meet at
//! different sites/times), so the max-wound single-seed maximums that the
//! i313/i314 pins froze are no longer stable. This probe measures the pain /
//! injury / conflict distributions across seeds and horizons so the pins can
//! re-contract onto the channel's live band (§4.2/§4.4) instead of a
//! knife-edge maximum (§4.5 debt).
//!
//! Run: cargo run --release -p mindstrata-benches --example i351_wound_band -- [horizon]
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let horizon: u64 = std::env::args()
        .nth(1)
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000);
    let seeds: Vec<u64> = vec![42, 7, 43, 44, 46, 47, 123, 11];
    println!(
        "i351 wound band @ {horizon} ticks (post-driver) — per-seed max pain / max injury / min blood / conflicts"
    );
    for &seed in &seeds {
        let config = SimConfig {
            seed,
            max_ticks: horizon,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        let mut max_injury = Fixed::ZERO;
        let mut max_pain = Fixed::ZERO;
        let mut min_blood = Fixed::ONE;
        for _ in 0..horizon {
            sim.tick();
            for a in &sim.agents {
                max_injury = max_injury.max(a.embodied.injury);
                max_pain = max_pain.max(a.embodied.nervous.pain.effective_pain());
                min_blood = min_blood.min(a.embodied.cardiovascular.blood_volume);
            }
        }
        let conflicts = sim
            .recent_events(usize::MAX)
            .iter()
            .filter(|ev| matches!(ev, SimEvent::ConflictOccurred { .. }))
            .count();
        println!(
            "seed {seed:>3}: max_pain {:.4} · max_injury {:.4} · min_blood {:.4} · conflicts {conflicts}",
            max_pain.to_f64(),
            max_injury.to_f64(),
            min_blood.to_f64(),
        );
    }

    // Veto-firing rate at candidate thresholds (agent-ticks above band).
    // i314 ratified 0.9 (0.04–0.31% of agent-ticks) over 0.7 (0.25–1.05%,
    // drifting 11 pins). Post-driver the severe ceiling dropped to ~0.82, so
    // the live band sits between: measure 0.72/0.75/0.78/0.80/0.82/0.85.
    println!("\nveto firing rates (share of agent-ticks with pain ≥ t):");
    let thresholds = [0.70, 0.72, 0.75, 0.78, 0.80, 0.82, 0.85, 0.90];
    let mut totals = vec![0u64; thresholds.len()];
    let mut agent_ticks = 0u64;
    for &seed in &seeds {
        let config = SimConfig {
            seed,
            max_ticks: horizon,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        for _ in 0..horizon {
            sim.tick();
            agent_ticks += sim.agents.len() as u64;
            for a in &sim.agents {
                let p = a.embodied.nervous.pain.effective_pain().to_f64();
                for (i, &t) in thresholds.iter().enumerate() {
                    if p >= t {
                        totals[i] += 1;
                    }
                }
            }
        }
    }
    for (i, &t) in thresholds.iter().enumerate() {
        println!(
            "  t={t:.2}: {:>7} agent-ticks ({:.3}%)",
            totals[i],
            totals[i] as f64 / agent_ticks as f64 * 100.0
        );
    }
}
