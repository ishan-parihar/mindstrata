//! i374 — probe: the legitimacy-coupled dividend share.
//!
//! i373's audit queued the first Class-B organic promotion: the council's
//! surplus dividend share is the hardcoded `COUNCIL_SURPLUS_DIVIDEND_SHARE =
//! 0.25`, but the sim already measures the state the constant is guessing at
//! — the council's legitimacy. The organic law:
//!
//!     s = s0 + k * (1 - legitimacy)
//!
//! A nervous/resented council buys goodwill with patronage (spends more of
//! its surplus); a secure one hoards (spends less). The hoard then carries
//! feedback: hoard → inequality grievance → legitimacy ↓ → payout ↑ → hoard ↓.
//!
//! This probe A/Bs three share laws on the i365 six-seed family at 20K:
//!   A. constant 0.25 (the i363 landed behaviour — the control)
//!   B. s = 0.15 + 0.4·(1 − legit)   (k = 0.4)
//!   C. s = 0.15 + 0.8·(1 − legit)   (k = 0.8, stress case)
//!
//! Contract to hold (from i365): mean Gini below the pre-fix plateau
//! (~0.647–0.652), no destitution, and — new for the organic law — the
//! treasury equilibrium should *tighten* (lower mean treasury) because
//! low-legitimacy regimes pay out more.
//!
//! The probe MEASURES ONLY; the share stays 0.25 until the sweep passes.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i374_legit_dividend`
//! Knobs: `I374_HORIZON`.

use mindstrata_sim::institutions::InstitutionKind;
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

/// The share law under test. `k = 0.0` degenerates to the constant law at s0.
fn share_law(k: f64, s0: f64, legitimacy: f64) -> f64 {
    (s0 + k * (1.0 - legitimacy)).clamp(0.05, 0.95)
}

fn run(seed: u64, ticks: u64, k: f64, s0: f64) -> (f64, f64, f64, f64, f64) {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 46,
        world_height: 46,
        num_agents: 48,
        snapshot_interval: None,
    });
    sim.populate();
    sim.params.council_dividend_share_s0 = mindstrata_core::fixed::Fixed::from_f64(s0);
    sim.params.council_dividend_share_k = mindstrata_core::fixed::Fixed::from_f64(k);

    let mut mean_legit = 0.0f64;
    let mut samples = 0.0f64;
    for _t in 0..ticks {
        sim.tick();
        if _t % 100 == 0 {
            if let Some(c) = sim
                .institutions
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
            {
                mean_legit += c.legitimacy.to_f64();
                samples += 1.0;
            }
        }
    }
    mean_legit /= samples.max(1.0);

    let treasury = sim
        .institutions
        .iter()
        .find(|i| i.kind == InstitutionKind::Council)
        .map(|c| c.treasury.to_f64())
        .unwrap_or(0.0);
    let mut coins: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
    let g = gini(&mut coins);
    let min_coin = coins.iter().cloned().fold(f64::INFINITY, f64::min);
    let legitimacy = sim
        .institutions
        .iter()
        .find(|i| i.kind == InstitutionKind::Council)
        .map(|c| c.legitimacy.to_f64())
        .unwrap_or(0.0);
    (g, treasury, min_coin, legitimacy, mean_legit)
}

fn main() {
    let horizon: u64 = std::env::var("I374_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000);
    let seeds = [7u64, 23, 42, 55, 99, 123];
    let laws: [(&str, f64, f64); 3] = [
        ("A: const 0.25", 0.0, 0.25),
        ("B: s=0.15+0.4(1-L)", 0.4, 0.15),
        ("C: s=0.15+0.8(1-L)", 0.8, 0.15),
    ];

    println!("== i374: legitimacy-coupled dividend share (N=48, 46x46, horizon {horizon}) ==");
    for (name, k, s0) in laws {
        let mut g_sum = 0.0;
        let mut t_sum = 0.0;
        let mut min_all = f64::INFINITY;
        println!("-- law {name} --");
        for seed in seeds {
            let (g, treasury, min_coin, legit, mean_legit) = run(seed, horizon, k, s0);
            g_sum += g;
            t_sum += treasury;
            min_all = min_all.min(min_coin);
            println!(
                "  seed {seed:>3}: gini={g:.4} treasury={treasury:>9.1} min_coin={min_coin:>8.2} legit_end={legit:.3} legit_mean={mean_legit:.3}"
            );
        }
        println!(
            "  == {} == mean_gini={:.4} mean_treasury={:.1} min_coin={:.2}\n",
            name,
            g_sum / seeds.len() as f64,
            t_sum / seeds.len() as f64,
            min_all
        );
    }
}
