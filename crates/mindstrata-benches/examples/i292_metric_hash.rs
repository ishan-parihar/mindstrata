use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    // riverford: 12 agents, 1000 ticks
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
    let ms = sim.metrics_snapshot();
    let metrics = vec![
        ms.avg_hunger,
        ms.avg_thirst,
        ms.avg_fatigue,
        ms.avg_valence,
        ms.avg_joy,
        ms.avg_fear,
        ms.total_grain,
        ms.total_water,
        ms.event_count as f64,
        ms.journal_len as f64,
        ms.agent_count as f64,
    ];
    let mut h = std::collections::hash_map::DefaultHasher::new();
    use std::hash::Hash;
    use std::hash::Hasher;
    for m in &metrics {
        m.to_bits().hash(&mut h);
    }
    println!("riverford metric_hash: {:016x}", h.finish());

    // collapse: 12 agents, 4320 ticks via from_scenario (matches test)
    let sc = Scenario::collapse();
    let mut sim2 = Simulation::from_scenario(sc);
    sim2.populate();
    sim2.run(4320);
    let ms2 = sim2.metrics_snapshot();
    let metrics2 = vec![
        ms2.avg_hunger,
        ms2.avg_thirst,
        ms2.avg_fatigue,
        ms2.avg_valence,
        ms2.avg_joy,
        ms2.avg_fear,
        ms2.total_grain,
        ms2.total_water,
        ms2.event_count as f64,
        ms2.journal_len as f64,
        ms2.agent_count as f64,
    ];
    let mut h2 = std::collections::hash_map::DefaultHasher::new();
    for m in &metrics2 {
        m.to_bits().hash(&mut h2);
    }
    println!("collapse   metric_hash: {:016x}", h2.finish());
}
