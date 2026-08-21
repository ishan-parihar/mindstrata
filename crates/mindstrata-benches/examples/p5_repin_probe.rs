//! Sweep seeds for a complete moral-panic register→escalate→drain lifecycle:
//! first panic start_tick, active@11000, drained@20000.
use mindstrata_sim::sim::{SimConfig, Simulation};

fn run(seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    println!("== moral-panic lifecycle sweep ==");
    for seed in [42u64, 44, 46, 50, 55, 99, 13, 7, 3, 5, 1, 2] {
        let mid = run(seed, 11000);
        let late = run(seed, 20000);
        let mid_panic = mid.moral_panic_registry.panics.first();
        let late_panic = late.moral_panic_registry.panics.first();
        let m = mid_panic.map(|p| {
            format!(
                "start={} active={} int={:.3}",
                p.start_tick,
                p.active,
                p.intensity.to_f64()
            )
        });
        let l = late_panic.map(|p| format!("active={} int={:.3}", p.active, p.intensity.to_f64()));
        println!(
            "seed {seed}: mid@11K={m:?} late@20K={l:?} count={}",
            mid.moral_panic_registry.panics.len()
        );
    }
}
