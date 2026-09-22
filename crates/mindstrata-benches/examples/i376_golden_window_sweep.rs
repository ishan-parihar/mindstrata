//! i376 leg D — re-anchor the biology golden-window seed.
//!
//! The `conception_pregnancy_birth_pipeline` golden leg pins a calibrated
//! 16×16/N=12 window (2000 ticks) with no conception, pregnancy, birth or
//! marriage child. It has been re-anchored five times as pacing shifted
//! (iterations 98/185/190/269/i363) and always by this sweep: find the clean
//! seeds, move the leg to one of them, leave the liveness leg untouched.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i376_golden_window_sweep`

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    println!("i376 leg D — 16×16, N=12, @2000: pregnancy / births / marriages by seed");
    println!("      (the golden leg needs 0 / 0 / 0; i363 shipped seed 44)\n");
    println!(
        "{:>6} {:>11} {:>8} {:>12} {:>8}",
        "seed", "pregnancies", "parents", "mar_children", "clean"
    );
    let mut clean: Vec<u64> = Vec::new();
    for seed in 1u64..=70 {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(2000);
        let preg = sim
            .agents
            .iter()
            .filter(|a| a.embodied.reproductive.pregnancy.is_some())
            .count();
        let parents = sim.agents.iter().filter(|a| a.parent_a.is_some()).count();
        // The golden leg pins marriage CHILDREN (an empty registry), not
        // marriage events — every seed forms marriages inside the window.
        let marriages: usize = sim
            .marriage_registry
            .marriages
            .iter()
            .map(|m| m.children.len())
            .sum();
        let is_clean = preg == 0 && parents == 0 && marriages == 0;
        if is_clean {
            clean.push(seed);
        }
        if seed == 44 || seed == 42 || !is_clean {
            println!(
                "{seed:>6} {preg:>11} {parents:>8} {marriages:>12} {:>8}",
                if is_clean { "yes" } else { "no" }
            );
        }
    }
    println!("\nclean seeds: {clean:?}");
    println!("clean count: {} / 70", clean.len());
}
