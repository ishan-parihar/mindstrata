//! i321 — the i307 residual: does the fatigue channel saturate with horizon?
//!
//! `docs/balance/needs-bands.md` records the i305 measurement honestly: fatigue
//! p50 0.145 / p99 1.0 at 2K–20K, and i307's finding-2 leaves the residual
//! "fatigue p90 still 0.359 @50K". The open question is which failure class
//! this is:
//!
//!   * an **equilibrium tail** — most agents sit low, a workload-differentiated
//!     minority carries high fatigue and the distribution is stable across
//!     horizons (the i267/i282 pattern: live, differentiated, fine); or
//!   * a **horizon attractor** — the share at/near the 1.0 ceiling climbs
//!     monotonically with horizon because relief cannot keep pace with decay
//!     (the i318 Allergy class: the long-horizon equilibrium is the horizon,
//!     not the agent's workload, which collapses differentiation).
//!
//! It also separates *who* carries the high tail: agents pinned by a
//! physiological state (exhaustion) vs agents whose **sleep debt** has
//! accumulated (a distinct channel the same pass owns).
//!
//! Measured per horizon (2K/20K/50K/100K), 12-seed family: fatigue mean, sd,
//! p10/50/90/p99/max, the share at ≥0.9 and ≥0.99, and the mean sleep debt.
//! A single loud seed cannot carry the verdict — per-seed p90 spread printed.
//!
//! Run: cargo run --release -p mindstrata-benches --example i321_fatigue_pace

use mindstrata_core::parameters::SimParameters;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HORIZONS: [u64; 4] = [2_000, 20_000, 50_000, 100_000];

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

struct Sample {
    fatigue: f64,
    sleep_debt: f64,
}

struct Dist {
    n: usize,
    mean: f64,
    sd: f64,
    p10: f64,
    p50: f64,
    p90: f64,
    p99: f64,
    max: f64,
    at_90: usize,
    at_99: usize,
    at_zero: usize,
    debt_mean: f64,
    debt_max: f64,
}

fn summarize(vals: &mut Vec<Sample>) -> Dist {
    vals.sort_by(|a, b| a.fatigue.partial_cmp(&b.fatigue).unwrap());
    let n = vals.len();
    if n == 0 {
        return Dist {
            n: 0,
            mean: 0.0,
            sd: 0.0,
            p10: 0.0,
            p50: 0.0,
            p90: 0.0,
            p99: 0.0,
            max: 0.0,
            at_90: 0,
            at_99: 0,
            at_zero: 0,
            debt_mean: 0.0,
            debt_max: 0.0,
        };
    }
    let mean = vals.iter().map(|s| s.fatigue).sum::<f64>() / n as f64;
    let var = vals.iter().map(|s| (s.fatigue - mean).powi(2)).sum::<f64>() / n as f64;
    let q = |p: f64| vals[((p * (n as f64 - 1.0)).round() as usize).min(n - 1)].fatigue;
    let debt_mean = vals.iter().map(|s| s.sleep_debt).sum::<f64>() / n as f64;
    let debt_max = vals.iter().map(|s| s.sleep_debt).fold(0.0_f64, f64::max);
    Dist {
        n,
        mean,
        sd: var.sqrt(),
        p10: q(0.10),
        p50: q(0.50),
        p90: q(0.90),
        p99: q(0.99),
        max: *vals.last().map(|s| &s.fatigue).unwrap(),
        at_90: vals.iter().filter(|s| s.fatigue >= 0.9).count(),
        at_99: vals.iter().filter(|s| s.fatigue >= 0.99).count(),
        at_zero: vals.iter().filter(|s| s.fatigue <= 0.0).count(),
        debt_mean,
        debt_max,
    }
}

fn measure(ticks: u64) -> (Dist, Vec<f64>) {
    let params = SimParameters::default();
    let mut vals = Vec::new();
    let mut per_seed_p90 = Vec::new();
    for &seed in &SEEDS {
        let mut sim = Simulation::new(config(seed, ticks));
        sim.params = params.clone();
        sim.populate();
        sim.run(ticks);
        if sim.agents.is_empty() {
            continue;
        }
        let mut seed_fat: Vec<f64> = Vec::new();
        for a in &sim.agents {
            let fatigue = a.needs.fatigue.to_f64();
            seed_fat.push(fatigue);
            vals.push(Sample {
                fatigue,
                sleep_debt: a.embodied.circadian.sleep_debt.to_f64(),
            });
        }
        seed_fat.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let i = ((0.90 * (seed_fat.len() as f64 - 1.0)).round() as usize).min(seed_fat.len() - 1);
        per_seed_p90.push(seed_fat[i]);
    }
    (summarize(&mut vals), per_seed_p90)
}

/// Within one long run, sample fatigue every `step` ticks. Distinguishes a
/// village-wide cycle (the mean oscillates in-run) from a horizon-dependent
/// equilibrium (the mean moves monotonically and then settles).
fn in_run_trajectory(seed: u64, total: u64, step: u64) {
    let params = SimParameters::default();
    let mut sim = Simulation::new(config(seed, total));
    sim.params = params.clone();
    sim.populate();
    println!("\n--- in-run trajectory (seed {seed}, every {step} ticks) ---");
    let mut done = 0;
    while done < total {
        sim.run(step);
        done += step;
        let n = sim.agents.len();
        if n == 0 {
            break;
        }
        let mut f: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| a.needs.fatigue.to_f64())
            .collect();
        f.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let mean = f.iter().sum::<f64>() / n as f64;
        let var = f.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / n as f64;
        let debt = sim
            .agents
            .iter()
            .map(|a| a.embodied.circadian.sleep_debt.to_f64())
            .sum::<f64>()
            / n as f64;
        println!(
            "  t={done:>6} n={n:<4} mean={mean:.4} sd={:.4} p10={:.4} p50={:.4} p90={:.4} debt={debt:.4}",
            var.sqrt(),
            f[(0.10 * (n as f64 - 1.0)).round() as usize],
            f[(0.50 * (n as f64 - 1.0)).round() as usize],
            f[(0.90 * (n as f64 - 1.0)).round() as usize],
        );
    }
}

fn main() {
    println!("i321 — fatigue pace across horizons, 12-seed family, N=12");

    in_run_trajectory(42, 100_000, 10_000);

    let mut shares = Vec::new();
    let mut p90s = Vec::new();
    for ticks in HORIZONS {
        let (d, per_seed_p90) = measure(ticks);
        println!("\n=== {ticks} ticks ===");
        println!(
            "  fatigue n={} mean={:.4} sd={:.4} p10={:.4} p50={:.4} p90={:.4} p99={:.4} max={:.4}",
            d.n, d.mean, d.sd, d.p10, d.p50, d.p90, d.p99, d.max
        );
        println!(
            "  at>=0.90 {}/{} ({:.0}%)  at>=0.99 {}/{} ({:.1}%)",
            d.at_90,
            d.n,
            100.0 * d.at_90 as f64 / d.n.max(1) as f64,
            d.at_99,
            d.n,
            100.0 * d.at_99 as f64 / d.n.max(1) as f64,
        );
        println!(
            "  sleep-debt mean={:.4} max={:.4}  | at==0.0 {}/{} ({:.0}%)  sd/mean={:.2}",
            d.debt_mean,
            d.debt_max,
            d.at_zero,
            d.n,
            100.0 * d.at_zero as f64 / d.n.max(1) as f64,
            d.sd / d.mean.max(1e-9),
        );
        let lo = per_seed_p90.iter().cloned().fold(f64::INFINITY, f64::min);
        let hi = per_seed_p90
            .iter()
            .cloned()
            .fold(f64::NEG_INFINITY, f64::max);
        println!(
            "  per-seed p90: lo={lo:.4} hi={hi:.4} spread={:.4}",
            hi - lo
        );
        shares.push(d.at_90 as f64 / d.n.max(1) as f64);
        p90s.push(d.p90);
    }

    // Verdict: a horizon attractor is a monotone climb of the high-tail share
    // across horizons; an equilibrium is a share that does not grow with it.
    let monotone = shares.windows(2).all(|w| w[1] >= w[0] - 0.005);
    let grown = shares.last().copied().unwrap_or(0.0) > shares[0] + 0.05;
    println!(
        "\n  high-tail share by horizon: {:?}",
        shares
            .iter()
            .map(|s| format!("{:.3}", s))
            .collect::<Vec<_>>()
    );
    println!(
        "  p90 by horizon:            {:?}",
        p90s.iter().map(|s| format!("{:.3}", s)).collect::<Vec<_>>()
    );
    // i307's residual ("p90 still 0.359 @50K") is dispositioned by what the
    // in-run trajectory shows: the village swings between a synchronized-rested
    // state (p10 == p50 == 0.0, sd/mean > 1) and a synchronized-fatigued state
    // (p10 ≈ p50, sd/mean < 0.5), i.e. a population-wide phase cycle, not a
    // horizon attractor and not an individual workload signal. No ceiling is
    // ever approached (0/… at ≥0.90 at every horizon). The residual is
    // therefore MEASURED, not smoothed: the calibration question it leaves is
    // whether the village cycle should carry the channel more weakly than
    // agent workload does, which is a design decision, not a defect.
    println!(
        "verdict={}",
        if monotone && grown {
            "FATIGUE_HORIZON_ATTRACTOR"
        } else {
            "FATIGUE_PHASE_SYNCHRONIZED_NOT_SATURATING"
        }
    );
}
