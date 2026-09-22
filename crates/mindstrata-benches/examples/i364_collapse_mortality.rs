//! i364 — did i363's council surplus dividend soften the collapse scenario?
//!
//! i363 landed a wealth-inverse council surplus dividend and, among its
//! consequences, both goldens moved **agent_count 12 → 13** — including the
//! `collapse` golden, whose entire purpose is *compound emergence when crises
//! stack on a weakened population* (drought → famine → pestilence over 4320
//! ticks). If redistribution neutralized crisis mortality, the scenario loses
//! its bite. This probe measures the mortality trajectory across the cascade so
//! the softening can be quantified rather than assumed.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i364_collapse_mortality`

use mindstrata_core::event::SimEvent;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

fn main() {
    let sc = Scenario::collapse();
    let total = sc.ticks;
    let mut sim = Simulation::from_scenario(Scenario::collapse());
    sim.populate();

    println!(
        "i364 — collapse-scenario mortality (seed {}, {total} ticks)\n",
        sc.seed
    );
    println!(
        "{:>6} {:>7} {:>7} {:>7}",
        "tick", "agents", "deaths", "births"
    );
    let mut deaths = 0u64;
    let mut births = 0u64;
    let step = 240u64;
    let mut done = 0u64;
    let mut min_agents = sim.agents.len();
    while done < total {
        let seg = step.min(total - done);
        sim.run(seg);
        done += seg;
        // Count this segment's terminal events (bounded buffer ⇒ incremental).
        let lo = done - seg;
        for ev in sim.recent_events(usize::MAX) {
            let t = match ev {
                SimEvent::AgentDied { tick, .. } => tick.as_u64(),
                SimEvent::ChildBorn { tick, .. } => tick.as_u64(),
                _ => continue,
            };
            if t > lo && t <= done {
                match ev {
                    SimEvent::AgentDied { .. } => deaths += 1,
                    SimEvent::ChildBorn { .. } => births += 1,
                    _ => {}
                }
            }
        }
        min_agents = min_agents.min(sim.agents.len());
        println!(
            "{:>6} {:>7} {:>7} {:>7}",
            done,
            sim.agents.len(),
            deaths,
            births
        );
    }
    let ms = sim.metrics_snapshot();
    println!(
        "\nfinal: agents {} (min {min_agents})  deaths {deaths}  births {births}  health {:.3}  hunger {:.4}  gini {:.3}",
        ms.agent_count, ms.avg_health, ms.avg_hunger, ms.gini
    );
    println!(
        "verdict: {}",
        if deaths == 0 {
            "NO_MORTALITY (scenario softened)"
        } else {
            "MORTALITY_PRESENT (scenario retains bite)"
        }
    );
}
