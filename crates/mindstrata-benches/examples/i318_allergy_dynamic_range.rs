//! i318 — difficulty-levers row 3 residual: the Allergy quadrants' dynamic
//! range against their ceilings.
//!
//! `docs/balance/difficulty-levers.md` records the residual honestly: "the Q2
//! dark-allergy quadrant still sits at 0.6–0.7 against ceiling 0.80 in the
//! canon band (the i293/i294 always-step absence-growth law), so its dynamic
//! range is compressed." This probe sizes that claim before anything is
//! touched: is Q2 *clustered near its ceiling* (a degenerate attractor that
//! erases agent differentiation) or merely *slower to move* (fine)?
//!
//! The absence law under test (`development.rs` always-step pass):
//!     next = I + g·0.1·headroom·(1−pressure) − d·pressure·I
//! so absence (pressure 0) is pure growth toward the ceiling and pressure is
//! the ONLY decay channel. If transgression pressure is sparse, every agent
//! asymptotes to the same fixed point no matter their diet.
//!
//! Measured per horizon: agent-level Q2/Q4 distribution (mean, sd, p10/50/90,
//! max), the share within 5% of the ceiling and within 5% of the floor, and
//! the per-seed spread so a single loud seed cannot carry the verdict.
//!
//! Run: cargo run --release -p mindstrata-benches --example i318_allergy_dynamic_range

use mindstrata_core::parameters::SimParameters;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HORIZONS: [u64; 3] = [20_000, 50_000, 100_000];

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

#[derive(Default, Clone, Copy)]
struct Dist {
    n: usize,
    mean: f64,
    sd: f64,
    p10: f64,
    p50: f64,
    p90: f64,
    max: f64,
    near_ceiling: usize,
    near_floor: usize,
    alive: usize,
}

fn summarize(vals: &mut [f64], ceiling: f64) -> Dist {
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    let n = vals.len();
    if n == 0 {
        return Dist::default();
    }
    let mean = vals.iter().sum::<f64>() / n as f64;
    let var = vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
    let q = |p: f64| vals[((p * (n as f64 - 1.0)).round() as usize).min(n - 1)];
    let hi = ceiling * 0.95;
    let lo = ceiling * 0.05;
    Dist {
        n,
        mean,
        sd: var.sqrt(),
        p10: q(0.10),
        p50: q(0.50),
        p90: q(0.90),
        max: *vals.last().unwrap(),
        near_ceiling: vals.iter().filter(|&&v| v >= hi).count(),
        near_floor: vals.iter().filter(|&&v| v <= lo).count(),
        alive: n,
    }
}

struct HorizonStats {
    q2: Dist,
    q4: Dist,
    per_seed_q2_mean: Vec<f64>,
}

fn measure(ticks: u64) -> HorizonStats {
    let params = SimParameters::default();
    let ceiling_q2 = 0.80_f64;
    let ceiling_q4 = 0.75_f64;
    let mut q2 = Vec::new();
    let mut q4 = Vec::new();
    let mut per_seed_q2 = Vec::new();
    for &seed in &SEEDS {
        let mut sim = Simulation::new(config(seed, ticks));
        sim.params = params;
        sim.populate();
        sim.run(ticks);
        if sim.agents.is_empty() {
            continue;
        }
        let mut seed_q2 = 0.0;
        for a in &sim.agents {
            let q2v = a.development.pathology.dark_allergy.intensity;
            let q4v = a.development.pathology.golden_allergy.intensity;
            q2.push(q2v);
            q4.push(q4v);
            seed_q2 += q2v;
        }
        per_seed_q2.push(seed_q2 / sim.agents.len() as f64);
    }
    HorizonStats {
        q2: summarize(&mut q2, ceiling_q2),
        q4: summarize(&mut q4, ceiling_q4),
        per_seed_q2_mean: per_seed_q2,
    }
}

fn print_dist(name: &str, d: &Dist, ceiling: f64) {
    println!(
        "  {name:<8} n={:<4} mean={:.4} sd={:.4} p10={:.4} p50={:.4} p90={:.4} max={:.4} \
         | ceil {:.2} | >=95% ceil {}/{} ({:.0}%) | <=5% ceil {}/{}",
        d.n,
        d.mean,
        d.sd,
        d.p10,
        d.p50,
        d.p90,
        d.max,
        ceiling,
        d.near_ceiling,
        d.alive,
        100.0 * d.near_ceiling as f64 / d.alive.max(1) as f64,
        d.near_floor,
        d.alive,
    );
}

/// The old vs new Allergy law stepped directly, so the probe carries its own
/// before/after rather than relying on a remembered baseline. Q2 params
/// (growth 0.045, decay 0.022, ceiling 0.80) and the i318 resting relaxation
/// `0.05 × decay`, at a representative sparse pressure rate.
fn law_leg(growth: f64, decay: f64, ceiling: f64, pressure: f64, ticks: usize) -> (f64, f64) {
    let g_eff = growth * 0.1;
    let mut old = 0.0_f64;
    let mut new = 0.0_f64;
    let relax = decay * 0.05;
    for _ in 0..ticks {
        old += g_eff * (ceiling - old).max(0.0) * (1.0 - pressure) - decay * pressure * old;
        new += g_eff * (ceiling - new).max(0.0) * (1.0 - pressure)
            - decay * pressure * new
            - relax * new;
    }
    (old, new)
}

fn main() {
    println!("i318 — Allergy quadrant dynamic range, canon band, 12-seed family");

    // ── Leg 0: the law itself, old vs new ────────────────────────────────
    println!("\n[leg 0] Allergy law: old (pressure-only decay) vs new (resting relaxation)");
    for (name, p) in [("absent (p=0.00)", 0.0), ("sparse (p=0.05)", 0.05)] {
        for ticks in [1_000usize, 20_000, 100_000] {
            let (old, new) = law_leg(0.045, 0.022, 0.80, p, ticks);
            println!("  {name} @{ticks:>6} ticks: old={old:.4} new={new:.4} (ceiling 0.80)");
        }
    }

    let mut saturating_anywhere = false;
    for ticks in HORIZONS {
        let s = measure(ticks);
        println!("\n=== {ticks} ticks ===");
        print_dist("Q2 dark", &s.q2, 0.80);
        print_dist("Q4 gold", &s.q4, 0.75);
        let spread = s
            .per_seed_q2_mean
            .iter()
            .copied()
            .fold((f64::INFINITY, f64::NEG_INFINITY), |(lo, hi), v| {
                (lo.min(v), hi.max(v))
            });
        println!(
            "  per-seed Q2 means: lo={:.4} hi={:.4} spread={:.4}",
            spread.0,
            spread.1,
            spread.1 - spread.0
        );
        // Saturation test: a quadrant is degenerate if >1/3 of agents sit
        // within 5% of the ceiling AND the p10 is also within 10% of it
        // (i.e. even the least-exposed agent is pinned high).
        for (name, d, c) in [("Q2", &s.q2, 0.80), ("Q4", &s.q4, 0.75)] {
            let hi_share = d.near_ceiling as f64 / d.alive.max(1) as f64;
            let p10_pinned = d.p10 >= c * 0.90;
            println!(
                "  {name} saturation: hi_share={:.0}% p10_pinned={p10_pinned}",
                100.0 * hi_share
            );
            if hi_share > 0.33 && p10_pinned {
                saturating_anywhere = true;
            }
        }
    }
    println!(
        "\nverdict={}",
        if saturating_anywhere {
            "ALLERGY_QUADRANTS_SATURATE"
        } else {
            "ALLERGY_QUADRANTS_DIFFERENTIATE"
        }
    );
}
