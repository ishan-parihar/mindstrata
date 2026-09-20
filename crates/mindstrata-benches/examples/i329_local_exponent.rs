//! i329 — local cost exponent at the TOP of the envelope.
//!
//! i294 fitted α_total = 2.115 as a single log-log slope across N = 12/48/96/192
//! and read the residual as the dense relationship matrix's Ω(N²) floor. But the
//! top segment of its own table looks steeper: 4 768 → 38 950 µs/tick from
//! N=96 → 192 is a ratio of 8.17 for a 2× population, i.e. a **local** exponent
//! of ≈3.03, not ≈2. The i320 daily-scan probe recorded the same shape
//! (4 570 → 38 758).
//!
//! A single fit across four rows hides the segment that matters, because the
//! low-N rows are floored by fixed per-tick overhead. If the local exponent at
//! N ≥ 96 really is ≈3, there is a surviving O(N³) term — an accident of the
//! i294 class, not the structural Ω(N²) floor — and it is exactly the term that
//! decides the envelope.
//!
//! This probe measures fast-tick cost at N = 48/96/144/192 and reports the
//! **local** log-log exponents between consecutive points.
//!
//! Run: cargo run --release -p mindstrata-benches --example i329_local_exponent
//! (≈2–3 min wall; keep the tick window small — N=192 is ~40 ms/tick.)

use std::time::Instant;

use mindstrata_sim::sim::{SimConfig, Simulation};

const SEED: u64 = 42;
const TICKS: u64 = 400;

fn cost_us(n: u32) -> f64 {
    let mut sim = Simulation::new(SimConfig {
        seed: SEED,
        max_ticks: TICKS,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(50); // warm caches / let the tier split settle
    let t0 = Instant::now();
    sim.run(TICKS);
    t0.elapsed().as_secs_f64() * 1e6 / TICKS as f64
}

fn main() {
    let sizes = [48u32, 96, 144, 192];
    println!("i329 — local cost exponent, fast ticks, seed {SEED}, 32×32 world");
    println!("  N     µs/tick    µs/agent   local α   (vs previous N)");

    let mut prev: Option<(u32, f64)> = None;
    let mut alphas = Vec::new();
    for &n in &sizes {
        let us = cost_us(n);
        let alpha = match prev {
            Some((pn, pus)) => {
                let a = (us / pus).ln() / (n as f64 / pn as f64).ln();
                alphas.push(a);
                format!("{a:>7.3}")
            }
            None => "      —".to_string(),
        };
        println!("  {n:<5} {us:>9.1}  {:>9.2}   {alpha}", us / n as f64);
        prev = Some((n, us));
    }

    // Per-agent cost is the diagnostic: constant ⇒ linear, rising ⇒ superlinear.
    println!("\n  reading: a per-agent cost that RISES with N means superlinear work.");
    let top_alpha = *alphas.last().unwrap_or(&0.0);
    let mid_alpha = alphas.get(alphas.len() / 2).copied().unwrap_or(0.0);
    println!("  local α at N≥96: {top_alpha:.3}  |  mid α: {mid_alpha:.3}");
    println!(
        "verdict={}",
        if top_alpha > 2.5 {
            "SURVIVING_SUPERQUADRATIC_TERM"
        } else if top_alpha > 2.15 {
            "QUADRATIC_FLOOR_CONFIRMED"
        } else {
            "SUBQUADRATIC"
        }
    );
}
