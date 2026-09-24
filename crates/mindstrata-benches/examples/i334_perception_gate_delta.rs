//! i334 — behavioural delta of the §2.4 perception gate on memory/attention.
//!
//! The gate removes the pre-i334 assumption that every agent perceives every
//! event in the village. Before landing it, this probe measures what that
//! assumption was *carrying*: the crisis world's grievance fuel (belief
//! charge -> moral panic), the emotion means that drive it, the memory
//! volume, and the cognitive-budget drain (the memory pass consumes one
//! budget op per *encoded* percept, so a village-wide agent used to spend
//! budget on other people's events).
//!
//! Run baseline and gated by stashing:
//!   cargo run --release -p mindstrata-benches --example i334_perception_gate_delta
//!
//! Evidence only — no assertions, no behaviour here.

use mindstrata_sim::sim::{SimConfig, Simulation};

struct Row {
    label: String,
    seed: u64,
    panic_count: usize,
    max_intensity: f64,
    max_charge: f64,
    mean_charge: f64,
    mean_fear: f64,
    mean_anger: f64,
    mean_distress: f64,
    traces_per_agent: f64,
    budget_mean: f64,
}

fn calm(seed: u64, ticks: u64) -> Row {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 32,
        world_height: 32,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(ticks);
    collect(&sim, format!("calm s{seed}"))
}

fn pestilence(seed: u64, ticks: u64) -> Row {
    let mut sc = mindstrata_sim::scenario::Scenario::pestilence();
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(ticks);
    collect(&sim, format!("pest s{seed}"))
}

fn collect(sim: &Simulation, label: String) -> Row {
    let agent_count = sim.agents.len().max(1) as f64;
    let mut max_charge = 0.0f64;
    let mut charge_sum = 0.0f64;
    let mut charge_n = 0.0f64;
    let mut fear = 0.0f64;
    let mut anger = 0.0f64;
    let mut distress = 0.0f64;
    let mut traces = 0.0f64;
    let mut budget = 0.0f64;
    for agent in &sim.agents {
        for belief in agent
            .beliefs
            .iter()
            .filter(|belief| belief.proposition_id <= 1)
        {
            let charge = belief.emotional_charge.to_f64();
            max_charge = max_charge.max(charge);
            charge_sum += charge;
            charge_n += 1.0;
        }
        let fear_val = agent.emotions.fear.to_f64();
        let anger_val = agent.emotions.anger.to_f64();
        let valence = agent.affect.valence.to_f64();
        fear += fear_val;
        anger += anger_val;
        distress += (0.6 * fear_val + 0.4 * (-valence).max(0.0) + 0.2 * anger_val).clamp(0.0, 1.0);
        traces += agent.memory.count() as f64;
        let remaining_budget = agent
            .agent_tier
            .budget_tracker
            .remaining_memory_operations() as f64;
        budget += remaining_budget;
    }
    let max_intensity = sim
        .moral_panic_registry
        .panics
        .iter()
        .map(|panic_ev| panic_ev.intensity.to_f64())
        .fold(0.0f64, f64::max);
    Row {
        label,
        seed: 0,
        panic_count: sim.moral_panic_registry.panics.len(),
        max_intensity,
        max_charge,
        mean_charge: charge_sum / charge_n.max(1.0),
        mean_fear: fear / agent_count,
        mean_anger: anger / agent_count,
        mean_distress: distress / agent_count,
        traces_per_agent: traces / agent_count,
        budget_mean: budget / agent_count,
    }
}

fn main() {
    let ticks = 20_000u64;
    let mut rows = vec![calm(42, ticks), pestilence(5, ticks), pestilence(7, ticks)];
    // Fall back to a shorter crisis window if a seed is slow.
    println!(
        "i334 — perception-gate behavioural delta (20K ticks; calm N=12, pestilence scenario)\n"
    );
    println!(
        "{:<12} {:>6} {:>9} {:>9} {:>10} {:>9} {:>9} {:>10} {:>8} {:>9}",
        "world",
        "panics",
        "max_int",
        "max_chg",
        "mean_chg",
        "fear",
        "anger",
        "distress",
        "traces",
        "budget"
    );
    let r = &mut rows[0];
    r.seed = 42;
    for r in &rows {
        println!(
            "{:<12} {:>6} {:>9.4} {:>9.4} {:>10.4} {:>9.4} {:>9.4} {:>10.4} {:>8.1} {:>9.4}",
            r.label,
            r.panic_count,
            r.max_intensity,
            r.max_charge,
            r.mean_charge,
            r.mean_fear,
            r.mean_anger,
            r.mean_distress,
            r.traces_per_agent,
            r.budget_mean
        );
    }
    println!("\nbudget = mean remaining fraction of the per-agent memory-op budget");
    println!("traces = mean live memory traces per agent (store capacity 200)");
}
