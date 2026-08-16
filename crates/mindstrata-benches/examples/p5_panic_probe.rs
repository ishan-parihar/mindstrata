//! Seed-99 moral-panic lifecycle: first-panic start/end, activity at the
//! mid sample, drain by the horizon.
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let config = SimConfig {
        seed: 99,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(20_000);
    for (i, p) in sim.moral_panic_registry.panics.iter().enumerate().take(4) {
        println!(
            "panic {i}: start={} active={} intensity={:.3}",
            p.start_tick,
            p.active,
            p.intensity.to_f64()
        );
    }
    // State at the 11,000 sample.
    let mid = Simulation::new(SimConfig {
        seed: 99,
        max_ticks: 11_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    let mut mid = mid;
    mid.populate();
    mid.run(11_000);
    println!("@11000: panics={}", mid.moral_panic_registry.panics.len());
    for (i, p) in mid.moral_panic_registry.panics.iter().enumerate().take(3) {
        println!(
            "  panic {i}: start={} active={} intensity={:.3}",
            p.start_tick,
            p.active,
            p.intensity.to_f64()
        );
    }
}
