//! Iteration 184 — population-collapse diagnosis.
//!
//! The emergent probe found most seeds collapse to 0–4/12 alive by 20K
//! ticks (seed 42: 385 deaths in ~0.57 years — mass death in CALM
//! conditions). This probe samples the population/food trajectory and
//! breaks down deaths by cause + agent age to find the collapse driver.

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn tick_of(t: mindstrata_core::clock::Tick) -> u64 {
    t.as_u64()
}

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
    for &seed in &[42u64, 7, 99, 13, 46, 55] {
        let sim = run(seed, 20_000);

        // Death causes
        let mut causes: std::collections::BTreeMap<String, u32> = std::collections::BTreeMap::new();
        let mut death_ticks: Vec<u64> = Vec::new();
        for e in sim.recent_events(10_000_000) {
            if let SimEvent::AgentDied { cause, tick, .. } = e {
                *causes.entry(format!("{cause:?}")).or_insert(0) += 1;
                death_ticks.push(tick_of(*tick));
            }
        }
        let first_death = death_ticks.iter().min().copied();
        let (mut old, mut mid, mut young) = (0usize, 0usize, 0usize);
        let mut max_age = 0.0f64;
        let alive = sim.agents.iter().filter(|a| a.body.health > mindstrata_core::fixed::Fixed::ZERO).count();
        for a in &sim.agents {
            let age = a.age.to_f64();
            max_age = max_age.max(age);
            if age >= 60.0 {
                old += 1;
            } else if age >= 30.0 {
                mid += 1;
            } else {
                young += 1;
            }
        }
        let grain = sim.total_grain().to_f64();
        let pop = sim.agents.len();
        println!(
            "seed {seed:>2} @20K: alive={alive}/{pop} first_death={first_death:?} causes={causes:?} | \
             ages: old(60+)={old} mid(30-59)={mid} young(<30)={young} max_age={max_age:.1} grain={grain:.0}"
        );
    }

    // Population + food trajectory for the worst seed (42)
    println!("\n=== seed 42 trajectory (sampled every 2K) ===");
    for ticks in [2000u64, 4000, 6000, 8000, 10000, 12000, 14000, 16000, 18000, 20000] {
        let sim = run(42, ticks);
        let alive = sim.agents.iter().filter(|a| a.body.health > mindstrata_core::fixed::Fixed::ZERO).count();
        let deaths = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| matches!(e, SimEvent::AgentDied { .. }))
            .count();
        let births = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| matches!(e, SimEvent::ChildBorn { .. }))
            .count();
        let grain = sim.total_grain().to_f64();
        let hunger_mean: f64 = sim
            .agents
            .iter()
            .map(|a| a.needs.hunger.to_f64())
            .sum::<f64>()
            / sim.agents.len() as f64;
        let health_mean: f64 = sim
            .agents
            .iter()
            .map(|a| a.body.health.to_f64())
            .sum::<f64>()
            / sim.agents.len() as f64;
        println!(
            "@{ticks:>5}: alive={alive:>2}/{} deaths_cum={deaths:>3} births_cum={births:>2} grain={grain:>7.0} hunger_mean={hunger_mean:.2} health_mean={health_mean:.2}",
            sim.agents.len()
        );
    }
}
