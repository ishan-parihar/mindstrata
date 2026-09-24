//! Iter-286 probe — WP-H3 norm-proposal channel in vivo.
//!
//! The unit pins prove the mechanism (band gate, quorum, prescription
//! filter, dedup). This probe asks the reachability question at the system
//! level: with the Safety bucket FORCED past the band-III gate (6.0), do
//! natural runs produce surviving Integrated Value/Norm syntheses with
//! majority tension-cluster consensus — i.e. does a proposed norm ever
//! enter the registry through natural claim dynamics? Census at 5K/20K/50K.
//!
//! Control: natural stages (no forcing) must propose zero at 20K — the
//! band gate holds below 4.0 in vivo too.

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn proposed_count(sim: &Simulation) -> usize {
    sim.norms
        .norms()
        .iter()
        .filter(|n| n.name.starts_with("[proposed:"))
        .count()
}

fn run(label: &str, force_stage: bool) {
    let mut sc = Scenario::drought();
    sc.ticks = 50_000;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    if force_stage {
        for line in &mut sim.collective_field.lines {
            line.stage = 6.0;
        }
    }
    let mut done = 0u64;
    for h in [5_000u64, 20_000, 50_000] {
        sim.run(h - done);
        done = h;
        println!(
            "{label} @{h:>5}: proposed_norms={} (registry={})",
            proposed_count(&sim),
            sim.norms.norms().len()
        );
    }
}

fn main() {
    run("natural", false);
    run("forced", true);
}
