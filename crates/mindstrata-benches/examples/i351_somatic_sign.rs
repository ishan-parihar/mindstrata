//! i351 — classify the `emotional_body_tone_resists_regulation_in_tick`
//! divergence: signed somatic-channel behaviour or chaos?
//!
//! With the Wander driver live, the interoception somatic marker biases the
//! (now reachable) risky action down — high-sensitivity agents genuinely
//! explore less, so encounter schedules differ. If the conflict-count
//! divergence is SIGNED (embodied ≤ detached in most seeds) and bounded in
//! relative terms, it is the channel working as designed and the pin
//! re-contracts onto direction + relative bound (§4.4). If the sign flips
//! across seeds, it is chaos and the design needs a rethink.
//!
//! Run: cargo run --release -p mindstrata-benches --example i351_somatic_sign
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn conflicts(sim: &Simulation) -> usize {
    sim.recent_events(usize::MAX)
        .iter()
        .filter(|ev| matches!(ev, SimEvent::ConflictOccurred { .. }))
        .count()
}

fn mean_arousal(sim: &Simulation) -> f64 {
    let n = sim.agents.len();
    let total: Fixed = sim
        .agents
        .iter()
        .fold(Fixed::ZERO, |acc, a| acc + a.affect.arousal);
    (total / Fixed::from_int(n as i64)).to_f64()
}

fn run(seed: u64, sensitivity: f64) -> (usize, f64) {
    let config = SimConfig {
        seed,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    for agent in &mut sim.agents {
        agent.interoception.sensitivity = Fixed::from_f64(sensitivity);
    }
    // The test ticks both worlds in lockstep for 4000 ticks.
    for _ in 0..4000 {
        sim.tick();
    }
    (conflicts(&sim), mean_arousal(&sim))
}

fn main() {
    let seeds = [42u64, 7, 43, 44, 46, 47, 123, 11];
    println!("i351 somatic-sign classification (4000 ticks, lockstep worlds)");
    let mut embodied_fewer = 0usize;
    for &seed in &seeds {
        let (emb_c, emb_a) = run(seed, 0.9);
        let (det_c, det_a) = run(seed, 0.1);
        let sign = match emb_c.cmp(&det_c) {
            std::cmp::Ordering::Less => {
                embodied_fewer += 1;
                "embodied<detached"
            }
            std::cmp::Ordering::Equal => "equal",
            std::cmp::Ordering::Greater => "embodied>detached",
        };
        println!(
            "seed {seed:>3}: conflicts {emb_c:>4} vs {det_c:>4} ({sign:>18}) · arousal {emb_a:.4} vs {det_a:.4}"
        );
    }
    println!(
        "\nverdict: embodied-fewer in {embodied_fewer}/{} seeds — {}",
        seeds.len(),
        if embodied_fewer as f64 / seeds.len() as f64 >= 0.75 {
            "SIGNED somatic channel (re-contract onto direction + relative bound)"
        } else {
            "NOT signed — treat as pacing noise, widen absolute band"
        }
    );
}
