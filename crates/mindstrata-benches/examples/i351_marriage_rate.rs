//! i351 — marriage_formation_rate liveness sweep post-Wander-driver: does
//! the 0.0002 low-rate run still marry within 5000 ticks, and does the
//! high-vs-low differential survive, across seeds?
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn make(seed: u64, rate: f64) -> (usize, Option<u64>) {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: 5_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.params.marriage_formation_rate = Fixed::from_f64(rate);
    sim.populate();
    let mut first: Option<u64> = None;
    for t in 0..5_000 {
        let before = sim.marriage_registry.marriages.len();
        sim.tick();
        if first.is_none() && sim.marriage_registry.marriages.len() > before {
            first = Some(t);
        }
    }
    (sim.marriage_registry.marriages.len(), first)
}

fn main() {
    // Rate sweep on seed 42 (the test's seed): find the lowest rate that
    // still marries within 5000 ticks post-driver.
    for rate in [0.0002, 0.0004, 0.0006, 0.0008, 0.001, 0.002, 0.005] {
        let (c, f) = make(42, rate);
        println!("seed 42 rate {rate}: {c} marriages first@{f:?}");
    }
    println!();
    for seed in [42u64, 7, 43, 44, 46, 11] {
        let (lc, lf) = make(seed, 0.0002);
        let (hc, hf) = make(seed, 0.1);
        println!("seed {seed:>3}: low {lc:>2} marriages first@{lf:?} · high {hc:>2} first@{hf:?}");
    }
}
