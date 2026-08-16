//! Probe the household-headship (ElderJunior) orphan-reset end state.
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 300,
        world_width: 16,
        world_height: 16,
        num_agents: 6,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(144);
    // Joint household 0+1, head 0 → daily pass labels ElderJunior.
    let mut hh = mindstrata_sim::social::household::Household::new(0, sim.agents[0].home_site, 144);
    hh.head = Some(0);
    hh.add_member(1);
    sim.households.push(hh);
    sim.run(144);
    let stage = |sim: &Simulation| -> String {
        sim.agents[0]
            .relationship_v2s
            .iter()
            .find(|r| r.to == mindstrata_core::id::AgentId::new(1))
            .map(|r| format!("{:?}", r.stage))
            .unwrap_or_default()
    };
    let trust = sim.agents[0]
        .relationship_v2s
        .iter()
        .find(|r| r.to == mindstrata_core::id::AgentId::new(1))
        .map(|r| r.trust.to_f64());
    println!("after-household: stage={:?} trust={:?}", stage(&sim), trust.map(|t| format!("{t:.3}")));
    // Remove the producer: dissolve the household (drop member 1).
    for h in &mut sim.households {
        h.members.retain(|m| *m != 1);
    }
    sim.run(144);
    println!("after-dissolve: stage={:?}", stage(&sim));
}
