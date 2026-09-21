//! i349 — seed-44 kinship anatomy: did the locality fix cause the birth, or
//! was it pre-existing at HEAD `a592510`? (§4.4 classification instrument.)

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    for seed in [43u64, 44, 47, 51] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        for _ in 0..2000 {
            sim.tick();
        }
        let births = sim
            .recent_events(sim.event_count() as usize)
            .iter()
            .filter(|e| matches!(e, mindstrata_core::event::SimEvent::ChildBorn { .. }))
            .count();
        let max_pen = sim
            .agents
            .iter()
            .map(|a| a.attraction.kinship_penalty.to_f64())
            .fold(0.0f64, f64::max);
        println!(
            "seed {seed}: births {births} · kinship edges {} · max kinship_penalty {max_pen:.3}",
            sim.kinship_graph.active_count()
        );
    }
}
