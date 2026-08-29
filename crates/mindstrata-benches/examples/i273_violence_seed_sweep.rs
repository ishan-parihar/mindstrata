//! Seed sweep for violence liveness (DC-1 CO-2026-001).
//!
//! Runs the 12-seed family at 4000 ticks, counts violent ConflictOccurred
//! events, prints base violence count per seed. Used to re-anchor
//! `violence_taboo_aversion_suppresses_escalation_differentially` after
//! the 4-quadrant pathology fan-out.

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];

fn main() {
    for &seed in &SEEDS {
        let config = SimConfig {
            seed,
            max_ticks: 4000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
            ..SimConfig::default()
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(4000);
        let count = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::ConflictOccurred {
                        kind: ConflictKind::Violence,
                        ..
                    }
                )
            })
            .count();
        println!("seed={seed} base_violence={count}");
    }
}
