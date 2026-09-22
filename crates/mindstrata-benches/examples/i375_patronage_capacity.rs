//! i375 — probe: status-scaled patronage capacity.
//!
//! i373's audit queued Class-B promotion #4: `PATRONAGE_MAX_CLIENTS_PER_PATRON
//! = 3` is a hard constant, but the sim already measures the state it guesses
//! at — the patron's status. The organic law:
//!
//!     cap = 3 + floor(effective_status × 4)
//!
//! The rich attract more clients because more people want to be their client —
//! that IS the patronage mechanism; the constant was a ceiling imposed on it.
//! Low-status patrons (0.1) keep the old cap 3; a paramount chief (0.9) gets 6.
//!
//! Measured per seed at 20K (N=48, 46x46):
//! 1. patronage relation counts (the old law must not collapse),
//! 2. the client-count distribution across patrons (does capacity actually
//!    differentiate by status — the mechanism claim),
//! 3. Gini (patronage transfers wealth to clients — the distribution must
//!    hold its band).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i375_patronage_capacity`
//! Knobs: `I375_HORIZON`.

use mindstrata_sim::sim::{SimConfig, Simulation};

fn gini(xs: &mut [f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let n = xs.len() as f64;
    let sum: f64 = xs.iter().sum();
    if sum <= 0.0 {
        return 0.0;
    }
    let weighted: f64 = xs
        .iter()
        .enumerate()
        .map(|(i, x)| (i as f64 + 1.0) * x)
        .sum();
    (2.0 * weighted) / (n * sum) - (n + 1.0) / n
}

fn main() {
    let horizon: u64 = std::env::var("I375_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000);
    println!("== i375: status-scaled patronage capacity (N=48, 46x46, horizon {horizon}) ==");
    let mut g_sum = 0.0;
    let mut rel_sum = 0usize;
    for seed in [7u64, 23, 42, 55, 99, 123] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: horizon,
            world_width: 46,
            world_height: 46,
            num_agents: 48,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(horizon);

        let mut patron_counts: Vec<(f64, usize)> = Vec::new();
        for rel in &sim.patronage_registry.relations {
            if !rel.active {
                continue;
            }
            let status = sim.agents[rel.patron].status_v2.effective_status().to_f64();
            match patron_counts
                .iter_mut()
                .find(|(s, _)| (*s - status).abs() < 1e-6)
            {
                Some((_, c)) => *c += 1,
                None => patron_counts.push((status, 1)),
            }
        }
        patron_counts.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));
        let active: usize = sim
            .patronage_registry
            .relations
            .iter()
            .filter(|r| r.active)
            .count();
        let mut coins: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
        let g = gini(&mut coins);
        g_sum += g;
        rel_sum += active;
        let top: Vec<String> = patron_counts
            .iter()
            .take(4)
            .map(|(s, c)| format!("({s:.2}→{c})"))
            .collect();
        println!(
            "seed {seed:>3}: relations={active:>3} gini={g:.4} top_patrons=[{}]",
            top.join(", ")
        );
    }
    println!(
        "== mean: gini={:.4} relations={:.1} ==",
        g_sum / 6.0,
        rel_sum as f64 / 6.0
    );
}
