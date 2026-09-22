//! i358 — does wealth concentrate without bound, or is the Gini stable?
//!
//! The plan lists wealth dynamics as an open item: "Gini concentration has weak
//! counter-forces; add/inspect inheritance, charity, redistribution norms". But
//! the engine already carries **inheritance** (a death splits the estate among
//! heirs — `births_deaths.rs`), **taxation** (institution tax_rate collection +
//! legitimacy/fatigue side-effects — `institutions_impl.rs`), and a market
//! inequality read. So the first question is not "add a counter-force" but
//! "is concentration even unbounded?" — a Gini drifting to 1.0 with everyone
//! else at zero is the §4.3 saturation hazard; a Gini that oscillates mid-band
//! is a working economy.
//!
//! Legs:
//!   A — Gini + wealth percentiles over a long horizon (25K, N=48).
//!   B — the wealth tail: SHARE held by the top decile, and the bottom-half
//!       share, so "concentration" is a distribution not a scalar.
//!   C — liveness of the counter-forces: do estates actually change hands
//!       (inheritance), and does tax actually collect?
//!
//! Run: `cargo run --release -p mindstrata-benches --example i358_wealth_dynamics`

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
    // Gini = (2 * Σ i·x_i) / (n * Σ x_i) − (n+1)/n  (1-indexed i)
    let weighted: f64 = xs
        .iter()
        .enumerate()
        .map(|(i, x)| (i as f64 + 1.0) * x)
        .sum();
    (2.0 * weighted) / (n * sum) - (n + 1.0) / n
}

fn main() {
    println!("i358 — wealth dynamics (46x46, N=48, seed 42)\n");
    for (seed, n, horizon) in [(42u64, 48u32, 50_000u64), (7, 48, 50_000)] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: horizon,
            world_width: 46,
            world_height: 46,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();

        println!("== seed {seed} (N={n}) ==");
        println!(
            "{:>7} {:>7} {:>8} {:>8} {:>8} {:>8} {:>8}",
            "tick", "agents", "gini", "p50", "p90", "max", "top10%"
        );
        for _ in 1..=10u64 {
            sim.run(horizon / 10);
            let agents = sim.agents.len();
            let mut w: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
            let total: f64 = w.iter().sum();
            w.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let pct = |q: f64| -> f64 {
                if w.is_empty() {
                    0.0
                } else {
                    w[((q * (w.len() - 1) as f64) as usize).min(w.len() - 1)]
                }
            };
            let top10: f64 = w[w.len() - w.len().div_ceil(10)..].iter().sum();
            let g = sim.metrics_snapshot().gini;
            println!(
                "{:>7} {:>7} {:>8.4} {:>8.2} {:>8.2} {:>8.2} {:>7.1}%",
                sim.current_tick().as_u64(),
                agents,
                g,
                pct(0.5),
                pct(0.9),
                w.last().copied().unwrap_or(0.0),
                if total > 0.0 {
                    100.0 * top10 / total
                } else {
                    0.0
                }
            );
        }
        // Leg B: the bottom-half share at the end.
        let mut w: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
        let total: f64 = w.iter().sum();
        let g = gini(&mut w);
        let half = w.len() / 2;
        let bottom: f64 = w[..half].iter().sum();
        let zero = w.iter().filter(|x| **x <= 1e-6).count();
        println!(
            "  final: recomputed gini {g:.4}  bottom-half share {:.1}%  zero-wealth agents {zero}/{}",
            if total > 0.0 {
                100.0 * bottom / total
            } else {
                0.0
            },
            w.len()
        );
    }
}
