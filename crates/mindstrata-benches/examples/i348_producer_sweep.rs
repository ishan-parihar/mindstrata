//! i348 — producer sweep under the live Background tier.
//!
//! The re-contracted entry gate moved 6 integration liveness anchors. Before
//! re-anchoring anything: are the producers (faction formation, moral panic)
//! dead or re-paced (§4.1/§4.3)? And what share of agent-ticks sits in
//! Background in crisis worlds (where emotion bonus should *raise*
//! importance, making Background rarer than calm)?
//!
//! Run: cargo run --release -p mindstrata-benches --example i348_producer_sweep

use mindstrata_sim::agent_tier::AgentTier;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn main() {
    println!("i348 — producer sweep, pestilence (30K ticks per seed)\n");
    for seed in [1u64, 5, 7, 42, 55, 12345] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 30_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        let mut bg_ticks = 0u64;
        let mut total_ticks = 0u64;
        let mut factions_seen = 0u64;
        let mut max_active = 0usize;
        for t in 0..30_000u64 {
            sim.tick();
            total_ticks += sim.agents.len() as u64;
            for a in &sim.agents {
                if a.agent_tier.tier == AgentTier::Background {
                    bg_ticks += 1;
                }
            }
            let active = sim
                .faction_v2_registry
                .factions
                .iter()
                .filter(|f| f.active)
                .count();
            factions_seen += active as u64;
            max_active = max_active.max(active);
            let _ = t;
        }
        println!(
            "  pestilence seed={seed}: live-faction ticks {factions_seen} (max concurrent {max_active}) · Background share {:.2}%",
            100.0 * bg_ticks as f64 / total_ticks.max(1) as f64
        );
    }

    println!("\n  collapse (seed 42, 4320 ticks):");
    let mut sc = Scenario::collapse();
    sc.seed = 42;
    sc.ticks = 4_320;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    let mut bg = 0u64;
    let mut tot = 0u64;
    for _ in 0..4_320 {
        sim.tick();
        tot += sim.agents.len() as u64;
        for a in &sim.agents {
            if a.agent_tier.tier == AgentTier::Background {
                bg += 1;
            }
        }
    }
    println!(
        "    Background share {:.2}% (agent-ticks {bg}/{tot})",
        100.0 * bg as f64 / tot.max(1) as f64
    );
}
