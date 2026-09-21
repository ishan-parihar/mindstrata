//! i351 — presence-band probe: with the Wander driver live, how many agents
//! sit within perception radius of a neighbour at the daily field snapshot?
//!
//! The `sensory_field_fear_contagion_is_live_and_sustains_fear` pin asserted
//! 12/12 agents perceive ambient stress — written when no agent ever moved.
//! Roaming agents are legitimately alone sometimes (`perceived_stress = 0`
//! when `nearby_count == 0`, social_cluster.rs). This probe measures the
//! perceiving-count band across seeds so the reach assertion re-contracts
//! onto the live-locomotion distribution (§4.2).
//!
//! Run: cargo run --release -p mindstrata-benches --example i351_presence
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let seeds = [42u64, 7, 43, 44, 46, 47, 123, 11];
    println!("i351 presence band @ 2000 ticks (agents with perceived_stress > 0):");
    for &seed in &seeds {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(2000);
        let n_pos = sim
            .agents
            .iter()
            .filter(|a| a.relational_fields.perceived_stress > Fixed::ZERO)
            .count();
        let mean_fear: f64 = {
            let n = sim.agents.len();
            let total: Fixed = sim
                .agents
                .iter()
                .fold(Fixed::ZERO, |acc, a| acc + a.emotions.fear);
            (total / Fixed::from_int(n as i64)).to_f64()
        };
        println!("seed {seed:>3}: {n_pos}/12 perceive · mean fear {mean_fear:.4}");
    }
}
