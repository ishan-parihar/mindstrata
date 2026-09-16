//! i275 debug — trace esteem/autonomy per-tick to locate the dead increment.
//! Run: cargo run --release -p mindstrata-benches --example i275_need_trace

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 300,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    println!(
        "meaning_decay_rate_raw={} hunger_decay_rate_raw={}",
        sim.params.meaning_decay_rate.to_raw(),
        sim.params.hunger_decay_rate.to_raw()
    );
    for t in 0..300u64 {
        sim.tick();
        if t % 50 == 0 || t < 5 {
            let a = &sim.agents[0];
            println!(
                "tick={t} esteem_raw={} autonomy_raw={} hunger_raw={} meaning_raw={}",
                a.needs.esteem.to_raw(),
                a.needs.autonomy.to_raw(),
                a.needs.hunger.to_raw(),
                a.needs.meaning.to_raw()
            );
        }
    }
}
