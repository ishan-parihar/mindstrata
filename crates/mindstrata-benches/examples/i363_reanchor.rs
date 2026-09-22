//! i363 — re-anchor probe for the council-surplus-dividend sweep.
//!
//! The dividend is behavioural and distributional, so it re-paces three
//! long-standing integration pins that must be re-anchored with a named
//! mechanism (§4.2), not widened blindly. This probe measures the replacement
//! values on the pins' own harnesses (16×16, N=12):
//!
//!   A — the conception/birth **golden-window** seed: find a seed in 40..70 with
//!       zero pregnancies at tick 2000 (the pin runs a clean seed by construction).
//!   B — the seed-42/5000 mean `fear`/`joy` (the emotion-context pin).
//!   C — the seed-42/2000 high/low belief confidence (the conviction pin).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i363_reanchor`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn run_sim(seed: u64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(ticks);
    sim
}

fn beliefs_mean(seed: u64, ticks: u64, conf: f64) -> (f64, f64) {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    for a in &mut sim.agents {
        for b in &mut a.beliefs {
            b.confidence = Fixed::from_f64(conf);
        }
    }
    sim.run(ticks);
    let mean: f64 = sim
        .agents
        .iter()
        .flat_map(|a| a.beliefs.iter())
        .map(|b| b.confidence.to_f64())
        .sum::<f64>()
        / sim.agents.iter().map(|a| a.beliefs.len()).sum::<usize>() as f64;
    let fear: f64 = sim
        .agents
        .iter()
        .map(|a| a.emotions.fear.to_f64())
        .sum::<f64>()
        / sim.agents.len() as f64;
    (mean, fear)
}

fn main() {
    println!("i363 — re-anchor probe (16x16, N=12)\n");

    // Leg A: golden-window seed sweep.
    print!("A — pregnancies @2000 for seeds 40..70: ");
    let mut clean = Vec::new();
    for seed in 40..=70u64 {
        let sim = run_sim(seed, 2000);
        let preg = sim
            .agents
            .iter()
            .filter(|a| a.embodied.reproductive.pregnancy.is_some())
            .count();
        if preg == 0 {
            clean.push(seed);
        }
    }
    println!("clean seeds = {clean:?}");

    // Leg B: emotion context seed 42/5000.
    let sim = run_sim(42, 5000);
    let n = sim.agents.len() as f64;
    let fear = sim
        .agents
        .iter()
        .map(|a| a.emotions.fear.to_f64())
        .sum::<f64>()
        / n;
    let joy = sim
        .agents
        .iter()
        .map(|a| a.emotions.joy.to_f64())
        .sum::<f64>()
        / n;
    println!("B — seed 42/5000: fear {fear:.4}  joy {joy:.4}");

    // Leg C: conviction seed 42/2000.
    let (high, _) = beliefs_mean(42, 2000, 0.9);
    let (low, _) = beliefs_mean(42, 2000, 0.1);
    println!(
        "C — seed 42/2000: high {high:.4}  low {low:.4}  delta {:.4}",
        high - low
    );
}
