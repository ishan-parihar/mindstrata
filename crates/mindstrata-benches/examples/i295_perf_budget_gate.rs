//! i295 — Perf-envelope regression bench, warn-only (PLAN_DC3 §4 i295;
//! charter DC3-P0 §5 items 1+2, §2 budgets).
//!
//! i270/i271 hard-fail their floors (tps, render IC-8) — appropriate for
//! budget-table enforcement. This probe adds the *golden-budget* envelopes
//! the charter binds DC-3 work to, as **warnings in gate output** rather than
//! gate failures: release-timing noise (±8% measured, i284) on a shared host
//! must not block commits, but drift must be VISIBLE every gate run.
//!
//! Budgets (charter DC3-P0 §2, re-baselined by i294):
//!   N=12  ≤  150 µs/tick  (golden/snapshot suite wall-time budget; 34% headroom)
//!   N=96  ≤ 6500 µs/tick  (DC-3 Phase-1 target; 27% headroom post-i294)
//!
//! Method matches the charter baseline (i274/i294): world 32×32, 2000 ticks,
//! seed 42, timer spans new+populate+run, min-of-N (timing noise is strictly
//! additive; min is the robust estimator).
//!
//! Trigger review (charter §3/§4 standing items, re-checked at i294 evidence):
//!   - VecDeque event-buffer conversion: trigger >250K-tick horizons — NOT hit
//!     (suite horizon 10K; i293 festival probe max 20K). Unchanged.
//!   - Per-agent recent-claims index: trigger >250K ticks — NOT hit. Unchanged.
//!   - Sparse relationship store: NEW lever recorded at i294 (α_rels=2.03
//!     structural floor); deferred — N≤96 is the Phase-1 envelope, N=96 has
//!     27% budget headroom. No action this iteration.
//!
//! Exit code is always 0 unless the probe itself cannot run — enforcement
//! stays with the hard-fail probes; this one makes drift visible.
//!
//! Run: cargo run -p mindstrata-benches --release --example i295_perf_budget_gate

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::time::Instant;

/// (population, world side, µs/tick budget, repeats, label)
const BUDGETS: &[(u32, u32, f64, u32, &str)] = &[
    (12, 32, 150.0, 3, "golden budget (charter DC3-P0 §2.1)"),
    (
        96,
        32,
        6500.0,
        1,
        "DC-3 Phase-1 target (charter DC3-P0 §2.2)",
    ),
];

fn measure_us_per_tick(n: u32, world: u32, ticks: u64, seed: u64) -> f64 {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: world,
        world_height: world,
        num_agents: n,
        snapshot_interval: None,
    };
    let t = Instant::now();
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    t.elapsed().as_micros() as f64 / ticks as f64
}

fn main() {
    let ticks: u64 = 2_000;
    let seed: u64 = 42;
    println!(
        "i295 perf budgets — warn-only, release, {ticks} ticks, seed {seed} (charter method: populate included)"
    );

    let mut any_breach = false;
    for &(n, world, budget, reps, label) in BUDGETS {
        let mut best = f64::INFINITY;
        for _ in 0..reps {
            let us = measure_us_per_tick(n, world, ticks, seed);
            if us < best {
                best = us;
            }
        }
        let status = if best <= budget { "OK" } else { "BREACH" };
        if best > budget {
            any_breach = true;
        }
        println!(
            "perf_budget n={n:>3} us_per_tick={best:>9.1} budget={budget:>7.1} status={status}  ({label})"
        );
        if status == "BREACH" {
            eprintln!(
                "perf_budget_warning: N={n} measured {best:.1} µs/tick exceeds the {budget:.0} µs/tick {label} — investigate before stacking more per-tick work (warn-only; hard floors remain in i270/i271)"
            );
        }
    }

    if any_breach {
        println!(
            "note: budget drift is a warning, not a gate failure — but a breach here means the golden suite wall-time or the Phase-1 envelope is eroding; schedule a hot-path iteration (charter §3) if it persists across seeds/hosts."
        );
    }
}
