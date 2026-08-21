//! Size the P5 fix blast on the golden window (riverford seed 42 @1000):
//! marriages formed, positive interactions recorded, intimacy levels.
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn main() {
    let mut sc = Scenario::riverford();
    sc.seed = 42;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(1000);
    let active_marriages = sim
        .marriage_registry
        .marriages
        .iter()
        .filter(|m| m.active)
        .count();
    let pair_bonds = sim.marriage_registry.pair_bonds.len();
    let mut int_sum = 0.0f64;
    let mut int_nonzero = 0usize;
    let mut edge_count = 0usize;
    let mut max_int = 0.0f64;
    for a in &sim.agents {
        for r in &a.relationship_v2s {
            edge_count += 1;
            let i = r.intimacy.to_f64();
            int_sum += i;
            max_int = max_int.max(i);
            if i > 0.0 {
                int_nonzero += 1;
            }
        }
    }
    println!(
        "golden riverford@1000: marriages={active_marriages} pair_bonds={pair_bonds} | \
         edges={edge_count} intimacy_mean={:.4} max={max_int:.4} nonzero={int_nonzero}",
        int_sum / edge_count.max(1) as f64
    );
}
