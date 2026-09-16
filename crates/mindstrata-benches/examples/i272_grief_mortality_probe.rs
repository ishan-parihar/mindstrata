//! Iteration-272 probe — mortality-horizon observation of Grief and
//! Transgression catalysts (the two kinds the i270 census could not
//! observe at the 2000-tick horizon).
//!
//! Questions, measured at the live horizon across seeds:
//!
//! 1. When do the first Grief (death) and Transgression (NormViolated)
//!    catalysts fire? (i270 recorded both as event-rate-limited at 2000.)
//! 2. What does the Q1 (Grief→identity) field trajectory look like after
//!    real deaths land — does the spec-midpoint magnitude (1.0) produce
//!    differentiated grief work, or saturate/ratchet?
//! 3. Does `system_development` (running live in `sim.run`) actually move
//!    the affected agents' altitudes/identity at death time?
//!
//! Evidence only — no production changes in this probe.

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    println!("=== i272: mortality-horizon Grief/Transgression observation ===\n");

    for &seed in &[42u64, 7, 99, 13, 46] {
        let config = SimConfig {
            seed,
            max_ticks: 20_000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(20_000);

        let mut first_grief: Option<u64> = None;
        let mut grief_count = 0usize;
        let mut first_trans: Option<u64> = None;
        let mut trans_count = 0usize;
        for e in sim.recent_events(10_000_000) {
            match e {
                SimEvent::AgentDied { tick, .. } => {
                    grief_count += 1;
                    let t = tick.as_u64();
                    if first_grief.is_none_or(|f| t < f) {
                        first_grief = Some(t);
                    }
                }
                SimEvent::NormViolated { tick, .. } => {
                    trans_count += 1;
                    let t = tick.as_u64();
                    if first_trans.is_none_or(|f| t < f) {
                        first_trans = Some(t);
                    }
                }
                _ => {}
            }
        }

        // Per-agent development field state after the horizon.
        let mut alt_sum = 0.0f64;
        let mut alt_moved = 0usize;
        let mut max_identity_line = 0.0f64;
        let line_count = mindstrata_development::line::all_lines().count();
        for a in sim.agents.iter() {
            let alts = &a.development.altitudes;
            if alts.len() == line_count {
                let sum: f64 = alts.iter().sum();
                alt_sum += sum;
                if sum.abs() > f64::EPSILON {
                    alt_moved += 1;
                }
                // Line 0 is the Grief-indexed identity line in the pinned
                // stable mapping (Grief => 0).
                if !alts.is_empty() {
                    max_identity_line = max_identity_line.max(alts[0]);
                }
            }
        }
        let n = sim.agents.len().max(1) as f64;
        println!(
            "seed {seed:>4}: deaths={grief_count:>2} (first @ {:>6?})  norm_viol={trans_count:>3} (first @ {:>6?})  agents_with_altitude_movement={alt_moved:>2}/{}  mean_alt_sum={:.4}  max_line0={:.4}",
            first_grief,
            first_trans,
            sim.agents.len(),
            alt_sum / n,
            max_identity_line,
        );
    }
    println!("\nprobe complete.");
}
