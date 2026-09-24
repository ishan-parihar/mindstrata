//! i335 — is the seed-46 accelerated birth pipeline re-paced or dead under the
//! §2.4 perception gate?
//!
//! The gate changes the memory/attention state the seed-46 accelerated world
//! runs on, and that world's conception->gestation->delivery pipeline is a
//! known knife-edge. This probe distinguishes the two dispositions honestly:
//! "re-paced" (a delivery still lands, just at a different tick/horizon) vs
//! "dead" (no pregnancy-path delivery at any horizon).
//!
//! Run: cargo run --release -p mindstrata-benches --example i335_seed46_pipeline

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn build() -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed: 46,
        max_ticks: 12_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.demography_config.ticks_per_year = 100;
    sim
}

fn main() {
    println!("i335 — seed-46 accelerated pipeline under the perception gate\n");
    println!(
        "{:>7} {:>10} {:>12} {:>12} {:>14}",
        "tick", "pregnant", "births_seen", "last_birth", "men_children"
    );
    let mut sim = build();
    let mut total_births = 0usize;
    for target in [2000u64, 4000, 6000, 8000, 12_000] {
        sim.run(target - sim.current_tick().as_u64());
        let pregnant = sim
            .agents
            .iter()
            .filter(|a| a.embodied.reproductive.pregnancy.is_some())
            .count();
        let births: Vec<u64> = sim
            .recent_events(usize::MAX)
            .iter()
            .filter_map(|e| match e {
                SimEvent::ChildBorn { tick, .. } => Some(tick.as_u64()),
                _ => None,
            })
            .collect();
        total_births = births.len();
        let marriage_children: usize = sim
            .marriage_registry
            .marriages
            .iter()
            .map(|m| m.children.len())
            .sum();
        println!(
            "{:>7} {:>10} {:>12} {:>12} {:>14}",
            target,
            pregnant,
            births.len(),
            births.iter().copied().max().unwrap_or(0),
            marriage_children
        );
    }
    let births: Vec<u64> = sim
        .recent_events(usize::MAX)
        .iter()
        .filter_map(|e| match e {
            SimEvent::ChildBorn { tick, .. } => Some(tick.as_u64()),
            _ => None,
        })
        .collect();
    println!("\nchild_events in the (bounded) window: {births:?}  (n={total_births})");
    let late = births.iter().any(|t| *t >= 700);
    println!(
        "verdict: {}",
        if late {
            "RE_PACED — a pregnancy-path delivery still lands past the conception window"
        } else {
            "DEAD — no pregnancy-path delivery in this world"
        }
    );
}
