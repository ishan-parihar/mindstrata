//! Iteration 184 — violence-cascade instrumentation.
//! Measures the interaction-kind mix, threat→violence pipeline rates, and
//! the personality distribution that drives threats, to calibrate the
//! calm-baseline conflict-lethality fix.

use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

fn run(seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    let sim = run(42, 2000);

    // Interaction mix
    let mut kinds: BTreeMap<String, u32> = BTreeMap::new();
    let mut threats = 0u32;
    let mut insults = 0u32;
    let mut conflicts = 0u32;
    let mut deaths = 0u32;
    for e in sim.recent_events(10_000_000) {
        match e {
            SimEvent::InteractionOccurred { kind, .. } => {
                let k = format!("{kind:?}");
                *kinds.entry(k.clone()).or_insert(0) += 1;
                if matches!(kind, InteractionKind::Threaten) {
                    threats += 1;
                }
                if matches!(kind, InteractionKind::Insult) {
                    insults += 1;
                }
            }
            SimEvent::ConflictOccurred { .. } => conflicts += 1,
            SimEvent::AgentDied { .. } => deaths += 1,
            _ => {}
        }
    }
    println!("seed42 @2000 interaction mix: {kinds:?}");
    println!("  threats={threats} insults={insults} conflicts={conflicts} deaths={deaths}");

    // Personality distribution (drives the negativity branch)
    let mut agree_below = 0;
    let mut dom_risk_high = 0;
    for a in &sim.agents {
        let ag = a.personality.agreeableness.to_f64();
        let dr = a.personality.dominance.to_f64() + a.personality.risk_tolerance.to_f64();
        if ag < 0.35 {
            agree_below += 1;
        }
        if dr > 1.2 {
            dom_risk_high += 1;
        }
    }
    println!("  agreeableness<0.35: {agree_below}/{}  dom+risk>1.2: {dom_risk_high}/{}", sim.agents.len(), sim.agents.len());

    // Fear levels (threat_failed gate: fear < 0.3 → escalate opportunity)
    let fear_lt = sim
        .agents
        .iter()
        .filter(|a| a.emotions.fear.to_f64() < 0.3)
        .count();
    let anger_gt = sim
        .agents
        .iter()
        .filter(|a| a.emotions.anger.to_f64() > 0.5)
        .count();
    println!("  agents fear<0.3: {fear_lt}/{}  anger>0.5: {anger_gt}/{}", sim.agents.len(), sim.agents.len());

    // Health recovery check: what's the mean health trajectory?
    let health_mean: f64 =
        sim.agents.iter().map(|a| a.body.health.to_f64()).sum::<f64>() / sim.agents.len() as f64;
    println!("  mean health @2000: {health_mean:.2}");
}
