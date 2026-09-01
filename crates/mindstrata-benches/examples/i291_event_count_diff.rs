use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;
fn main() {
    let cfg = SimConfig {
        seed: 42,
        max_ticks: 1000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(cfg);
    sim.populate();
    sim.run(1000);
    println!("riverford: cumulative={}", sim.event_count());
    let cfg2 = SimConfig {
        seed: 42,
        max_ticks: 4320,
        world_width: 16,
        world_height: 16,
        num_agents: 36,
        snapshot_interval: None,
    };
    let mut sim2 = Simulation::new(cfg2);
    sim2.populate();
    sim2.run(4320);
    println!("collapse:   cumulative={}", sim2.event_count());
}
