//! i319 — wound reachability after the i314 exertion veto.
//!
//! i318's §2.5 re-audit found `cardiovascular.shock_risk` had gone to exactly
//! 0.0000 in every context, and `min blood-volume` stopped reaching its 0.300
//! floor, even though i313 had just made that channel live (`shock 0.845`,
//! `blood-vol 0.300` in the calm leg). Isolating `pass_action.rs` back to i313
//! restores those numbers, so **the i314 pain veto** is the cause: an injured
//! agent is vetoed from exertion (and therefore from further conflict), so it
//! no longer stacks the second/third gunshot the `injury > 0.3` blood-loss
//! threshold needs.
//!
//! This probe sizes the *reachable* wound range under the veto so the blood-loss
//! threshold can be reconciled with it on measured evidence rather than guessed:
//!
//!   * the single-wound severity distribution (net per-tick injury gain + the
//!     known 0.0005/tick heal),
//!   * the `ConflictKind` mix that produces those wounds,
//!   * the share of agents that ever cross candidate thresholds, and
//!   * whether the cardiovascular chain follows (blood-volume floor, shock).
//!
//! Run: cargo run --release -p mindstrata-benches --example i319_wound_reachability

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
/// `EmbodiedState::heal_injury` rate; subtracted from the net per-tick delta to
/// recover the raw wound the violence path recorded.
const HEAL_PER_TICK: f64 = 0.0005;
/// Candidate thresholds to evaluate for the cardiovascular blood-loss rule.
const CANDIDATES: [f64; 4] = [0.10, 0.15, 0.20, 0.30];

fn main() {
    println!("i319 wound reachability under the i314 exertion veto (12-seed family)");

    for ticks in [20_000u64, 50_000] {
        let mut max_injury = 0.0_f64;
        let mut max_wound = 0.0_f64;
        let mut wounds = 0u64;
        let mut injuries = 0u64;
        let mut combats = 0u64;
        let mut max_shock = 0.0_f64;
        let mut min_blood = 1.0_f64;
        let mut crossed = [0u64; CANDIDATES.len()];
        let mut agent_ticks = 0u64;

        for &seed in &SEEDS {
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: ticks,
                num_agents: 12,
                snapshot_interval: None,
                ..SimConfig::default()
            });
            sim.populate();
            let mut prev: Vec<f64> = sim
                .agents
                .iter()
                .map(|a| a.embodied.injury.to_f64())
                .collect();
            for _ in 0..ticks {
                sim.tick();
                for (i, a) in sim.agents.iter().enumerate() {
                    let inj = a.embodied.injury.to_f64();
                    let delta = inj - prev.get(i).copied().unwrap_or(0.0);
                    if delta > 0.0 {
                        let wound = delta + HEAL_PER_TICK;
                        max_wound = max_wound.max(wound);
                        wounds += 1;
                    }
                    prev.resize(sim.agents.len(), 0.0);
                    prev[i] = inj;
                    max_injury = max_injury.max(inj);
                    max_shock = max_shock.max(a.embodied.cardiovascular.shock_risk.to_f64());
                    min_blood = min_blood.min(a.embodied.cardiovascular.blood_volume.to_f64());
                    agent_ticks += 1;
                    for (ci, t) in CANDIDATES.iter().enumerate() {
                        if inj > *t {
                            crossed[ci] += 1;
                        }
                    }
                }
                prev.resize(sim.agents.len(), 0.0);
            }
            // i327 note: `recent_events` is a BOUNDED window now, so these
            // counts are window-limited at long horizons (the state-based
            // figures below — max wound, shock, threshold crossings — are
            // the verdict-bearing ones and are unaffected).
            for e in sim.recent_events(10_000_000) {
                if let SimEvent::ConflictOccurred { kind, .. } = e {
                    match kind {
                        ConflictKind::Violence => injuries += 1,
                        ConflictKind::Combat => combats += 1,
                        _ => {}
                    }
                }
            }
        }

        println!("\n=== {ticks} ticks ===");
        println!("  conflict mix (bounded window): Violence {injuries} | Combat {combats}");
        println!("  wounds recorded {wounds} | max single wound {max_wound:.4} | max injury {max_injury:.4}");
        println!("  cardiovascular: max shock {max_shock:.5} | min blood-vol {min_blood:.4}");
        for (ci, t) in CANDIDATES.iter().enumerate() {
            println!(
                "  threshold {t:.2}: agent-ticks above = {} ({:.4}% of {agent_ticks})",
                crossed[ci],
                100.0 * crossed[ci] as f64 / agent_ticks as f64
            );
        }
    }
}
