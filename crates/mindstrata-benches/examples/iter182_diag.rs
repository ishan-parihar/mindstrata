//! Iteration 182 diagnostic: multi-seed 160K birth sweep to re-confirm seed
//! 46 as the strongest liveness seed after the S2-2-3 + S2-2-1 biology fixes
//! (fitness/conditioning band widening + sympathetic/tone de-saturation)
//! re-paced the seed-46 trajectory from 6 births to 3 (see the Iter-182
//! recalibration note in integration_tests.rs).
//!
//! The Iter-172/180 pins were backed by 6-seed sweeps ("seeds 1/7/42/50/99 at
//! 0–1 births, seed 46 strongest"). This sweep repeats that evidence standard:
//! count pregnancy-path births per seed at 160K and confirm seed 46 leads.
//!
//! Run: `cargo run -p mindstrata-benches --example iter182_diag --release`

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn base(seed: u64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    println!("=====ITER-182-SEED-SWEEP-160K=====");
    for seed in [1u64, 7, 42, 46, 50, 99] {
        let sim = base(seed, 160_000);
        let births: Vec<u64> = sim
            .recent_events(10_000_000)
            .iter()
            .filter_map(|e| match e {
                SimEvent::ChildBorn { tick, .. } => Some(tick.as_u64()),
                _ => None,
            })
            .collect();
        let children_born: u32 = sim
            .agents
            .iter()
            .map(|a| a.embodied.reproductive.children_born)
            .sum();
        let parentage = sim.agents.iter().filter(|a| a.parent_a.is_some()).count();
        let open_preg = sim
            .agents
            .iter()
            .filter(|a| a.embodied.reproductive.pregnancy.is_some())
            .count();
        println!(
            "seed {seed:>3}: births={} {births:?} children_born={children_born} parentage={parentage} open_preg={open_preg} pop={}",
            births.len(),
            sim.agents.len(),
        );
    }
}
