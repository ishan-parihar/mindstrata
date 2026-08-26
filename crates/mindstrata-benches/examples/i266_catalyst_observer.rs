//! i266 — catalyst observer harness probe (DC-1 task 2.8).
//!
//! Drives a seeded village through its daily loop while accumulating draft
//! CatalystEvents into an inert caller-owned buffer, then prints IC-4
//! key=value verdict rows. Read-side only: proves observers neither crash
//! under sustained windows nor perturb the simulation (a silent twin run
//! provides the perturbation control).
//!
//! Run with: `cargo run -p mindstrata-benches --example i266_catalyst_observer --release`

use mindstrata_sim::sim::catalyst_observers::{CatalystEvent, Drive};
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const SEED: u64 = 4242;
const HORIZON: u64 = 2000;

fn make_sim() -> Simulation {
    let config = SimConfig {
        seed: SEED,
        max_ticks: HORIZON,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim
}

fn main() {
    // Observed run — buffer accumulates draft catalysts each tick.
    let mut observed = make_sim();
    let mut buffer: Vec<CatalystEvent> = Vec::new();
    for _ in 0..HORIZON {
        let mark = observed.catalyst_watermark();
        observed.tick();
        buffer.extend(observed.observe_catalysts_since(mark));
    }

    // Silent twin — identical seed, never observed.
    let mut silent = make_sim();
    for _ in 0..HORIZON {
        silent.tick();
    }
    let untouched = observed.metrics_snapshot() == silent.metrics_snapshot();

    let mut grief = 0usize;
    let mut bond = 0usize;
    let mut threat = 0usize;
    let mut transgression = 0usize;
    for ev in &buffer {
        match ev.drive {
            Drive::Grief => grief += 1,
            Drive::Bond => bond += 1,
            Drive::Threat => threat += 1,
            Drive::Transgression => transgression += 1,
        }
    }

    println!("ic4_tick={}", observed.current_tick().as_u64());
    println!("ic4_events_total={}", buffer.len());
    println!("ic4_drive_grief={grief}");
    println!("ic4_drive_bond={bond}");
    println!("ic4_drive_threat={threat}");
    println!("ic4_drive_transgression={transgression}");
    println!("ic4_golden_untouched={untouched}");

    let pass = untouched && !buffer.is_empty();
    if pass {
        println!("verdict=OBSERVER_PASS");
    } else {
        println!(
            "verdict=OBSERVER_FAIL golden_untouched={untouched} nonempty={}",
            !buffer.is_empty()
        );
        std::process::exit(1);
    }
}
