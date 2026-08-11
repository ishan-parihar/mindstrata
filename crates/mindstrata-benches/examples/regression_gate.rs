//! §18.5 Release-mode throughput regression gate (Iteration 160).
//!
//! The debug-profile integration gate (`tick_throughput_regression_gate`)
//! catches order-of-magnitude blowups with a very generous budget, but it
//! cannot detect optimizable-path regressions that only manifest in release
//! builds (the shipped configuration). This gate runs the FULL-CAPACITY
//! world (48 agents — the §19.5.F MAX_POPULATION designed settlement size)
//! in release mode and enforces a hard ticks/sec floor.
//!
//! Run with: `cargo run -p mindstrata-benches --example regression_gate --release`
//!
//! Exits non-zero (failing CI) when the measured throughput drops below the
//! floor — calibrated so a slow shared CI runner still passes (~1.7x
//! headroom over the probe-pinned 1,040 ticks/sec), while an accidental
//! O(n²) loop or per-tick allocation explosion fails hard.
//!
//! The gate prints per-config throughput and a final PASS/FAIL verdict.

use mindstrata_sim::{sim::SimConfig, Simulation};
use std::time::Instant;

/// Hard floor: ticks/sec at the 48-agent full-capacity world.
/// Probe-pinned on a mid-range desktop (release profile): 1,040 ticks/sec
/// at 48 agents @ 2,000 ticks. The 600 floor = ~1.7x headroom for shared
/// CI runners while still tripping on a 2x slowdown.
const MIN_TICKS_PER_SEC: f64 = 600.0;

fn measure(seed: u64, num_agents: u32, ticks: u64, world: u32) -> (f64, f64) {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: world,
        world_height: world,
        num_agents,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let start = Instant::now();
    sim.run(ticks);
    let elapsed = start.elapsed().as_secs_f64();
    (elapsed, ticks as f64 / elapsed)
}

fn main() {
    let mut all_pass = true;

    // Full-capacity world: the §19.5.F designed settlement size.
    for (seed, ticks) in [(42u64, 2000u64), (7u64, 2000u64)] {
        let (elapsed, rate) = measure(seed, 48, ticks, 32);
        let pass = rate >= MIN_TICKS_PER_SEC;
        all_pass &= pass;
        println!(
            "{}: seed={seed} agents=48 ticks={ticks} elapsed={elapsed:.2}s rate={rate:.1} ticks/sec (floor {MIN_TICKS_PER_SEC})",
            if pass { "PASS" } else { "FAIL" }
        );
    }

    // 24-agent mid-size sanity config (the standard test population).
    let (elapsed, rate) = measure(42, 24, 2000, 16);
    let pass = rate >= MIN_TICKS_PER_SEC;
    all_pass &= pass;
    println!(
        "{}: seed=42 agents=24 ticks=2000 elapsed={elapsed:.2}s rate={rate:.1} ticks/sec (floor {MIN_TICKS_PER_SEC})",
        if pass { "PASS" } else { "FAIL" }
    );

    if all_pass {
        println!("REGRESSION GATE: PASS");
    } else {
        println!("REGRESSION GATE: FAIL — throughput below the release-mode floor");
        std::process::exit(1);
    }
}
