//! Measure per-pair interaction cadence and V2 trust trajectory to
//! calibrate the decay vs interaction-gain balance.
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 5000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(5000);

    // Interaction counts per directed pair from the event log.
    let mut pair_counts: std::collections::HashMap<(u64, u64), u32> =
        std::collections::HashMap::new();
    for ev in sim.recent_events(10_000_000) {
        if let mindstrata_core::event::SimEvent::InteractionOccurred { from, to, .. } = ev {
            *pair_counts.entry((from.as_u64(), to.as_u64())).or_insert(0) += 1;
        }
    }
    let n_pairs = pair_counts.len().max(1);
    let total: u32 = pair_counts.values().sum();
    let avg_per_pair = total as f64 / n_pairs as f64;
    // Days = 5000/144 ≈ 34.7
    let days = 5000.0 / 144.0;
    let per_day = avg_per_pair / days;

    // V2 trust stats
    let mut trust_sum = 0.0f64;
    let mut below_01 = 0usize;
    let mut below_005 = 0usize;
    let mut n = 0usize;
    let mut max_trust = 0.0f64;
    for a in &sim.agents {
        for r in &a.relationship_v2s {
            let t = r.trust.to_f64();
            trust_sum += t;
            if t < 0.1 {
                below_01 += 1;
            }
            if t < 0.05 {
                below_005 += 1;
            }
            max_trust = max_trust.max(t);
            n += 1;
        }
    }
    println!(
        "interactions: total={total} pairs={n_pairs} avg_per_pair={avg_per_pair:.2} per_day={per_day:.2} | \
         V2 trust: mean={:.3} max={max_trust:.3} below_0.1={below_01} below_0.05={below_005}",
        trust_sum / n as f64
    );
}
