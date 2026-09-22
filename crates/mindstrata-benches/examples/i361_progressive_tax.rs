//! i361 — does a progressive tax bend the wealth tail i358 measured?
//!
//! i358 recorded, with this exact harness (46×46, N=48, seed 42/7, 50 000
//! ticks): Gini **0.388 → 0.647** (seed 42) and **0.419 → 0.652** (seed 7),
//! top-decile share 33% → **52–54%**, bottom-half share 15.4% → 10.3% / 12.7% →
//! 8.8%, with **0 zero-wealth agents** — bounded, but a single agent holding
//! ~50% of all coin. It also found the fault: `collect_taxes` was **proportional**
//! (`wealth × rate`), which is scale-invariant and provably inert on the Gini.
//!
//! i361 adds `TAX_PROGRESSIVE_SURCHARGE` on wealth **above the membership
//! median**. This probe re-runs the identical harness so the before/after is a
//! like-for-like comparison, and checks that the economy still functions
//! (grain, zero-coin, nobody destitute).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i361_progressive_tax`

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
    println!("i361 — progressive tax (46x46, N=48, 50K; i358 baseline: gini 0.647/0.652, top10% 52/54%)\n");
    for seed in [42u64, 7] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 50_000,
            world_width: 46,
            world_height: 46,
            num_agents: 48,
            snapshot_interval: None,
        });
        sim.populate();

        println!("== seed {seed} ==");
        println!(
            "{:>7} {:>7} {:>8} {:>8} {:>8} {:>8} {:>9}",
            "tick", "agents", "gini", "p50", "max", "zero$", "top10%"
        );
        for _ in 1..=10u64 {
            sim.run(5_000);
            let agents = sim.agents.len();
            let mut w: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
            let total: f64 = w.iter().sum();
            w.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p50 = w.get(w.len() / 2).copied().unwrap_or(0.0);
            let max = w.last().copied().unwrap_or(0.0);
            let zero = w.iter().filter(|x| **x <= 1e-6).count();
            let top10: f64 = w[w.len() - w.len().div_ceil(10)..].iter().sum();
            println!(
                "{:>7} {:>7} {:>8.4} {:>8.2} {:>8.2} {:>8} {:>8.1}%",
                sim.current_tick().as_u64(),
                agents,
                sim.metrics_snapshot().gini,
                p50,
                max,
                zero,
                if total > 0.0 {
                    100.0 * top10 / total
                } else {
                    0.0
                }
            );
        }
        // Final distribution + provisioning liveness.
        let mut w: Vec<f64> = sim.agents.iter().map(|a| a.wealth.coin.to_f64()).collect();
        let total: f64 = w.iter().sum();
        let g = gini(&mut w);
        let half = w.len() / 2;
        let bottom: f64 = w[..half].iter().sum();
        let ms = sim.metrics_snapshot();
        println!(
            "  final: gini {g:.4}  bottom-half share {:.1}%  health {:.3}  hunger {:.4}  stress {:.3}  grain {:.2}",
            if total > 0.0 { 100.0 * bottom / total } else { 0.0 },
            ms.avg_health,
            ms.avg_hunger,
            ms.avg_stress,
            ms.total_grain
        );
        for inst in &sim.institutions {
            println!(
                "    inst {:?}: members {}  treasury {:.1}",
                inst.kind,
                inst.members.len(),
                inst.treasury.to_f64()
            );
        }
    }
}
