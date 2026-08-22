//! Iteration 242 — engineered-marriage gate diagnosis.
//!
//! Replicates the marriage_forges_spouse test setup (seed 42, 12 agents,
//! pair (0,6) with trust/affection/health/age/position pinned) and prints,
//! every 100 daily cycles: the legacy trust/affection on the pair, body
//! health, the AttractionModel inputs the formation block computes, and
//! whether a feud currently blocks the pair via clan enmity.

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 40000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();

    // Mirror the test's kin edges + pinned state.
    use mindstrata_sim::social::kinship::KinshipLink;
    for p in [2usize, 3] {
        sim.kinship_graph
            .add_link(p, 0, KinshipLink::ParentChild, 0);
        sim.kinship_graph
            .add_link(0, p, KinshipLink::ParentChild, 0);
    }
    sim.kinship_graph.add_link(0, 4, KinshipLink::Sibling, 0);
    sim.kinship_graph
        .add_link(5, 6, KinshipLink::ParentChild, 0);
    sim.kinship_graph
        .add_link(6, 5, KinshipLink::ParentChild, 0);
    sim.agents[0].body.health = Fixed::ONE;
    sim.agents[6].body.health = Fixed::ONE;
    sim.agents[0].personality.agreeableness = Fixed::from_f64(0.5);
    sim.agents[6].personality.agreeableness = Fixed::from_f64(0.5);
    sim.agents[0].position = mindstrata_sim::sim::Position::new(1, 1);
    sim.agents[6].position = mindstrata_sim::sim::Position::new(1, 2);
    sim.agents[0].age = Fixed::from_f64(30.0);
    sim.agents[6].age = Fixed::from_f64(30.0);
    for r in &mut sim.relationships {
        if (r.from == AgentId0() && r.to == AgentId6())
            || (r.from == AgentId6() && r.to == AgentId0())
        {
            r.trust = Fixed::ONE;
            r.affection = Fixed::ONE;
        } else {
            r.trust = Fixed::ZERO;
            r.affection = Fixed::ZERO;
        }
    }

    let mut steps = 0usize;
    while steps < 900 {
        sim.run(144);
        steps += 1;
        if steps % 100 != 0 {
            continue;
        }
        let rel01 = sim
            .relationships
            .iter()
            .find(|r| r.from.as_u64() == 6 && r.to.as_u64() == 0);
        let (trust, affection) = rel01
            .map(|r| (r.trust.to_f64(), r.affection.to_f64()))
            .unwrap_or((-1.0, -1.0));
        println!(
            "step={steps:>3} t={:>6} trust={trust:.3} aff={affection:.3} h0={:.2} h6={:.2} \
             age0={:.0} partner0={:?} feuds0={} feuds6={} homes {:?}/{:?}",
            steps * 144,
            sim.agents[0].body.health.to_f64(),
            sim.agents[6].body.health.to_f64(),
            sim.agents[0].age.to_f64(),
            sim.agents[0].partner,
            sim.agents[0].feuds.len(),
            sim.agents[6].feuds.len(),
            sim.agents[0].home_site,
            sim.agents[6].home_site,
        );
    }
}

fn AgentId0() -> mindstrata_core::id::AgentId {
    mindstrata_core::id::AgentId::new(0)
}
fn AgentId6() -> mindstrata_core::id::AgentId {
    mindstrata_core::id::AgentId::new(6)
}
