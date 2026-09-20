//! i332 — did the i330/i331 scan removals expand the DC-3 Phase-1 envelope?
//!
//! i295 re-baselined the charter budgets after i294 and recorded **27% headroom**
//! at N=96 (4577.7 µs/tick against the 6500 budget). i330 and i331 then removed
//! every accidental per-agent / per-event / per-trade relationship-matrix scan
//! (each O(N³)/tick), cutting tick cost ~30% at N=96 and ~49% at N=192.
//!
//! The question this probe answers: does the freed budget now cover a **larger
//! population**? It measures the i295 charter method (world 32×32, 2000 ticks,
//! seed 42, timer spans new+populate+run, min-of-3) at N = 12/48/96/144/192 and
//! reports each row against the two charter budgets.
//!
//! Verdict bands (the N=96 charter budget is the fixed yardstick):
//!   ENVELOPE_EXPANDED_2X   N=192 fits the 6500 µs/tick N=96 charter budget
//!   ENVELOPE_EXPANDED_1_5X N=144 fits
//!   ENVELOPE_UNCHANGED     only N=96 still fits
//!
//! Warn-only, like i295: exit code is always 0; enforcement stays with the
//! i270/i271 hard floors.
//!
//! Run: cargo run -p mindstrata-benches --release --example i332_envelope_expansion

use mindstrata_sim::sim::{SimConfig, Simulation};
use std::time::Instant;

const TICKS: u64 = 2_000;
const SEED: u64 = 42;
const WORLD: u32 = 32;
const REPS: u32 = 3;

/// Charter DC3-P0 §2 budgets (µs/tick).
const N12_BUDGET: f64 = 150.0;
const N96_BUDGET: f64 = 6_500.0;

fn measure_us_per_tick(n: u32) -> f64 {
    // Min-of-REPS: timing noise is strictly additive, so min is the robust
    // estimator (i295 method).
    let mut best = f64::INFINITY;
    for _ in 0..REPS {
        let config = SimConfig {
            seed: SEED,
            max_ticks: TICKS,
            world_width: WORLD,
            world_height: WORLD,
            num_agents: n,
            snapshot_interval: None,
        };
        let t = Instant::now();
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(TICKS);
        best = best.min(t.elapsed().as_micros() as f64 / TICKS as f64);
    }
    best
}

fn main() {
    println!(
        "i332 envelope expansion — i295 charter method, {TICKS} ticks, seed {SEED}, min-of-{REPS}"
    );
    println!("  N     µs/tick   vs N12(150)   vs N96(6500)");
    for &n in &[12u32, 48, 96, 144, 192] {
        let us = measure_us_per_tick(n);
        println!(
            "  {n:<5} {us:>8.1}   {:>+10.0}   {:>+10.0}",
            us - N12_BUDGET,
            us - N96_BUDGET
        );
    }

    let n96 = measure_us_per_tick(96);
    let n144 = measure_us_per_tick(144);
    let n192 = measure_us_per_tick(192);
    let n96_headroom = (N96_BUDGET - n96) / N96_BUDGET * 100.0;
    println!(
        "\n  N=96  {n96:.1} µs/tick  → {n96_headroom:.0}% headroom vs the {N96_BUDGET:.0} budget"
    );
    println!(
        "  N=144 {n144:.1} µs/tick  → {} vs the N=96 budget",
        if n144 <= N96_BUDGET {
            "fits"
        } else {
            "breaches"
        }
    );
    println!(
        "  N=192 {n192:.1} µs/tick  → {} vs the N=96 budget",
        if n192 <= N96_BUDGET {
            "fits"
        } else {
            "breaches"
        }
    );
    println!(
        "verdict={}",
        if n192 <= N96_BUDGET {
            "ENVELOPE_EXPANDED_2X"
        } else if n144 <= N96_BUDGET {
            "ENVELOPE_EXPANDED_1_5X"
        } else {
            "ENVELOPE_UNCHANGED"
        }
    );
}
