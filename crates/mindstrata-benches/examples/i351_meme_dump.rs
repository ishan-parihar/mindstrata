//! i351 — why did the meme-count pins move? Dump the registry contents for
//! both meme_transmission configs at 3000 ticks.
use mindstrata_sim::sim::{SimConfig, Simulation};

fn run(seed: u64, ticks: u64, mult: f64) -> mindstrata_sim::Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.params.meme_transmission_multiplier = mindstrata_core::fixed::Fixed::from_f64(mult);
    sim.run(ticks);
    sim
}

fn main() {
    for (tag, mult) in [("baseline 1.2", 1.2), ("high 3.0", 3.0)] {
        let sim = run(42, 3000, mult);
        println!("{tag}: {} memes", sim.meme_registry.memes.len());
        for m in &sim.meme_registry.memes {
            println!(
                "  id {} hosts {} active {} [{}]",
                m.id, m.host_count, m.active, m.description
            );
        }
    }
}
