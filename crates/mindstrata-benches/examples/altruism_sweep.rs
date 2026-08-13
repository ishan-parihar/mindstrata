//! Iteration 180 calibration probe — AP2 §8.1.6 altruism wiring.
//!
//! The last dead core trait (`altruism`, zero decision consumers) is now
//! wired into the Help decision via `help_propensity` (appraisal.rs):
//! `help_propensity += altruism × social_altruism_help_multiplier`, ONE-SIDED
//! identity-at-zero. This probe documents the evidence behind the calibrated
//! rate 0.23 and the re-pinned reproduction cascade:
//!
//!   1. The tenderness differential (the consumer's own liveness contract —
//!      `tenderness_channel_boosts_helping_when_multiplier_active`) holds
//!      ONLY at rates 0.23–0.24 among probed values (0.2 breaks the §8.1.18
//!      violence-taboo family; 0.3 inverts the tenderness differential).
//!   2. At 0.23 the standing +0.115 help boost re-paces the shared
//!      interaction stream NON-monotonically (the Iter-164 pattern): the
//!      seed-46 first birth moves 51,000 → 120,160 and the liveness leg
//!      re-pins at 160K with the clean 2-chain [120160, 150090].
//!   3. The reproduction-conception multiplier stays live at 160K
//!      (mult=1 → 2 births, mult=2 → 8 births).
//!
//! Run with: `cargo run -p mindstrata-benches --example altruism_sweep --release`

use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

/// The calibrated altruism→help multiplier (sweep-verified: keeps the
/// tenderness differential + both §8.1.18 violence-taboo directionals live).
const RATE: f64 = 0.23;

fn run(seed: u64, ticks: u64, mult: Option<f64>) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.params.social_altruism_help_multiplier = Fixed::from_f64(RATE);
    if let Some(m) = mult {
        sim.params.reproduction_conception_multiplier = Fixed::from_f64(m);
    }
    sim.populate();
    sim.run(ticks);
    sim
}

fn birth_ticks(sim: &Simulation) -> Vec<u64> {
    sim.recent_events(10_000_000)
        .iter()
        .filter_map(|e| match e {
            SimEvent::ChildBorn { tick, .. } => Some(tick.as_u64()),
            _ => None,
        })
        .collect()
}

fn main() {
    // The re-pinned birth pipeline at the calibrated rate: the standing
    // Help boost delays the first birth past 100K; the clean 2-chain lands
    // by 160K (all births through the pregnancy path — children_born 2).
    for ticks in [100_000u64, 130_000, 160_000] {
        let sim = run(46, ticks, None);
        let born: u32 = sim
            .agents
            .iter()
            .map(|a| a.embodied.reproductive.children_born)
            .sum();
        println!(
            "seed46/{ticks}: births={:?} live={} children_born={born}",
            birth_ticks(&sim),
            sim.agents.len()
        );
    }

    // The reproduction-conception multiplier stays live at 160K.
    let base = birth_ticks(&run(46, 160_000, Some(1.0)));
    let boosted = birth_ticks(&run(46, 160_000, Some(2.0)));
    println!(
        "seed46/160K reproduction multiplier: mult1={} births, mult2={} births",
        base.len(),
        boosted.len()
    );
}
