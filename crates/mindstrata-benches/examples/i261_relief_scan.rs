//! Iteration 261 — council poor-relief liveness scan: earliest tick at
//! which a `poor_relief` provenance trace appears per seed, and total
//! payouts over the window. Feeds the suite pin.
fn first_relief(seed: u64, horizon: u64) -> Option<(u64, usize)> {
    let config = mindstrata_sim::sim::SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = mindstrata_sim::Simulation::new(config);
    sim.populate();
    sim.run(horizon);
    let traces = sim.provenance().institutional_for("council");
    let reliefs: Vec<_> = traces
        .iter()
        .filter(|t| t.decision_kind == "poor_relief")
        .collect();
    let first = reliefs.iter().map(|t| t.tick).min();
    first.map(|f| (f, reliefs.len()))
}
fn main() {
    for seed in [42u64, 7, 13, 5] {
        match first_relief(seed, 20_000) {
            Some((tick, n)) => {
                println!("seed {seed}: first relief @ tick {tick}, total payouts {n}")
            }
            None => println!("seed {seed}: NO relief fired in 20K ticks"),
        }
    }
}
