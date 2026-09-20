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
    budget_exhausted: f64,
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
    collect(sim, format!("calm s{seed}"))
}

fn pestilence(seed: u64, ticks: u64) -> Row {
    let mut sc = mindstrata_sim::scenario::Scenario::pestilence();
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(ticks);
    collect(sim, format!("pest s{seed}"))
}

fn collect(sim: Simulation, label: String) -> Row {
    let n = sim.agents.len().max(1) as f64;
    let mut max_charge = 0.0f64;
    let mut charge_sum = 0.0f64;
    let mut charge_n = 0.0f64;
    let mut fear = 0.0f64;
    let mut anger = 0.0f64;
    let mut distress = 0.0f64;
    let mut traces = 0.0f64;
    let mut budget = 0.0f64;
    let mut exhausted = 0.0f64;
    for a in &sim.agents {
        for b in a.beliefs.iter().filter(|b| b.proposition_id <= 1) {
            let c = b.emotional_charge.to_f64();
            max_charge = max_charge.max(c);
            charge_sum += c;
            charge_n += 1.0;
        }
        let f = a.emotions.fear.to_f64();
        let g = a.emotions.anger.to_f64();
        let v = a.affect.valence.to_f64();
        fear += f;
        anger += g;
        distress += (0.6 * f + 0.4 * (-v).max(0.0) + 0.2 * g).clamp(0.0, 1.0);
        traces += a.memory.count() as f64;
        let b = a.agent_tier.budget_tracker.remaining_memory_operations() as f64;
        budget += b;
        if b <= 0.0 {
            exhausted += 1.0;
        }
    }
    let max_intensity = sim
        .moral_panic_registry
        .panics
        .iter()
        .map(|p| p.intensity.to_f64())
        .fold(0.0f64, f64::max);
    Row {
        label,
        seed: 0,
        panic_count: sim.moral_panic_registry.panics.len(),
        max_intensity,
        max_charge,
        mean_charge: charge_sum / charge_n.max(1.0),
        mean_fear: fear / n,
        mean_anger: anger / n,
        mean_distress: distress / n,
        traces_per_agent: traces / n,
        budget_mean: budget / n,
        budget_exhausted: exhausted / n,
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
