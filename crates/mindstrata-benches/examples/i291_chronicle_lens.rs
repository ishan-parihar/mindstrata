//! i291 — Era V WP-L chronicle lens in-vivo (PLAN_DC3 §4 i291).
//!
//! Renders the village chronicle after 2K ticks (golden seed 42) and prints
//! the lore annal so the ray tint is visible. D5 contract: the lens tints
//! narrative text only — every mechanical surface is untouched by
//! construction (read-only render pass).

use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 2000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(2000);
    let chronicle = mindstrata_sim::sim::chronicle::render_chronicle(&sim);
    let mut in_lore = false;
    for line in chronicle.lines() {
        if line == "The lore of the village" {
            in_lore = true;
        }
        if in_lore {
            println!("{line}");
        }
    }
}
