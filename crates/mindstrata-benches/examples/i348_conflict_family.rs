//! i348 — conflict-escalation liveness seed-family sweep.
//!
//! The `live_consumer_conflict_escalation_chance` pin has been re-anchored
//! seven times (42→1→7→99→13→42→55), each on a single seed whose delta the
//! next standing shift collapsed. This probe measures the delta across a
//! 12-seed family under the i348 Background tier so the pin can re-contract
//! onto the family (§4.1: no lucky-seed re-pins) instead of an eighth flip.
//!
//! Run: cargo run --release -p mindstrata-benches --example i348_conflict_family

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::parameters::SimParameters;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn run(seed: u64, ticks: u64, escalate: bool) -> u64 {
    let mut params = SimParameters::default();
    if escalate {
        params.conflict_escalation_chance = Fixed::from_f64(0.9);
    }
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 32,
        world_height: 32,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.params = params;
    sim.populate();
    for _ in 0..ticks {
        sim.tick();
    }
    sim.metrics_snapshot().event_count as u64
}

fn main() {
    println!("i348 — conflict_escalation_chance delta across seeds (3000 ticks)\n");
    for seed in [1u64, 2, 5, 7, 13, 42, 46, 55, 99, 123, 777, 4242] {
        let base = run(seed, 3000, false);
        let treated = run(seed, 3000, true);
        println!(
            "  seed={seed:>5}: baseline {base} → treated {treated} · delta {:+}",
            treated as i64 - base as i64
        );
    }
}
