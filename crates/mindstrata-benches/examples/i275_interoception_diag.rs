//! i275 diagnostic: interoception test divergence under the live tension
//! channel. Measures mean arousal (embodied vs detached) + tension counts +
//! conflict counts at the test's horizon to evidence the re-contract.

use mindstrata_sim::sim::{SimConfig, Simulation};

fn run(sensitivity: f64) -> (f64, usize, usize) {
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    for agent in &mut sim.agents {
        agent.interoception.sensitivity = mindstrata_core::fixed::Fixed::from_f64(sensitivity);
    }
    for _ in 0..4000 {
        sim.tick();
    }
    let n = sim.agents.len();
    let total: f64 = sim.agents.iter().map(|a| a.affect.arousal.to_f64()).sum();
    let tension: usize = sim
        .agents
        .iter()
        .map(|a| {
            a.polarity_claims
                .iter()
                .filter(|c| {
                    c.polarity == mindstrata_development::polarity::PolarityState::ActiveTension
                })
                .count()
        })
        .sum();
    let conflicts = sim
        .recent_events(usize::MAX)
        .iter()
        .filter(|ev| {
            matches!(
                ev,
                mindstrata_core::event::SimEvent::ConflictOccurred { .. }
            )
        })
        .count();
    (total / n as f64, tension, conflicts)
}

fn main() {
    let (a_emb, t_emb, c_emb) = run(0.9);
    let (a_det, t_det, c_det) = run(0.1);
    println!("EMBODIED  arousal={a_emb:.4} tension_claims={t_emb} conflicts={c_emb}");
    println!("DETACHED  arousal={a_det:.4} tension_claims={t_det} conflicts={c_det}");
    println!("PIN (embodied > detached): {}", a_emb > a_det);
}
