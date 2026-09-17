//! Iter-280 violations sweep — does the §12.3 compliance surface ever go
//! nonzero across seeds/horizons? If NormViolated never fires naturally at
//! N=12, the morale→compliance channel has no observable surface and WP-J
//! must couple to a live read-side parameter instead (Work-utility axis).

use mindstrata_sim::sim::{SimConfig, Simulation};

fn violations(sim: &Simulation) -> usize {
    sim.recent_events(sim.event_count() as usize)
        .iter()
        .filter(|e| matches!(e, mindstrata_core::event::SimEvent::NormViolated { .. }))
        .count()
}

fn main() {
    for horizon in [5_000_i64, 20_000] {
        for seed in 42..48u64 {
            let config = SimConfig {
                seed,
                max_ticks: horizon as u64,
                world_width: 16,
                world_height: 16,
                num_agents: 12,
                snapshot_interval: None,
            };
            let mut sim = Simulation::new(config);
            sim.populate();
            sim.run(horizon as u64);
            println!(
                "t={horizon:>6} seed={seed} violations={} events={}",
                violations(&sim),
                sim.event_count()
            );
        }
    }
}
