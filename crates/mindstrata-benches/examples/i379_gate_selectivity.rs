//! i379 — a GATE SELECTIVITY census: which hardcoded thresholds actually discriminate?
//!
//! The i373 census enumerated `const` sites; the i376/i378 refreshes showed the
//! behavioural gates are numbers *inside conditionals*, and that a threshold is
//! only a defect when it stops discriminating. This probe measures that directly:
//! for each gated quantity, the share of samples above its shipped threshold plus
//! the distribution behind it.
//!
//! Three verdicts fall out of the same table:
//!   * **dead-high** (≥99% above) — the gate is always true; the behaviour it
//!     gates is unconditional, and the threshold is decoration.
//!   * **dead-low** (≤1% above) — the gate never opens; the behaviour is dead.
//!     This is the i346/i347/i351 "producer went dark" class.
//!   * **live** — the threshold sits inside the distribution.
//!
//! It also tests the audit's structural claim: trait gates (`traditionalism >
//! 0.6`) should be LESS fragile than endogenous gates, because founder traits are
//! drawn uniform and the sim does not move them, whereas endogenous quantities
//! (fear, trust, belief charge, legitimacy) are driven by the sim itself and can
//! saturate.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i379_gate_selectivity`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

struct Gate {
    name: &'static str,
    threshold: f64,
    /// true when the gate opens ABOVE the threshold, false when below.
    above: bool,
}

const GATES: &[Gate] = &[
    // Class C candidates — bounded, individually normalised scales.
    Gate {
        name: "needs.hunger > 0.85 (routine eat)",
        threshold: 0.85,
        above: true,
    },
    Gate {
        name: "needs.thirst > 0.90 (routine drink)",
        threshold: 0.90,
        above: true,
    },
    Gate {
        name: "needs.fatigue > 0.90 (routine sleep)",
        threshold: 0.90,
        above: true,
    },
    Gate {
        name: "needs.hunger > 0.70 (health effect)",
        threshold: 0.70,
        above: true,
    },
    Gate {
        name: "needs.social > 0.30 (socialise)",
        threshold: 0.30,
        above: true,
    },
    Gate {
        name: "need.social > 0.40 (extravert seek)",
        threshold: 0.40,
        above: true,
    },
    // Founder-uniform trait gates — the structural control.
    Gate {
        name: "personality.traditionalism > 0.60",
        threshold: 0.60,
        above: true,
    },
    Gate {
        name: "personality.openness > 0.60",
        threshold: 0.60,
        above: true,
    },
    Gate {
        name: "personality.extraversion > 0.60",
        threshold: 0.60,
        above: true,
    },
    Gate {
        name: "personality.ambition > 0.60",
        threshold: 0.60,
        above: true,
    },
    Gate {
        name: "personality.conscientiousness > 0.50",
        threshold: 0.50,
        above: true,
    },
    Gate {
        name: "personality.traditionalism > 0.70",
        threshold: 0.70,
        above: true,
    },
    // Endogenous emotions — driven by the sim, so saturation is possible.
    Gate {
        name: "emotions.fear > 0.50",
        threshold: 0.50,
        above: true,
    },
    Gate {
        name: "emotions.anger > 0.50",
        threshold: 0.50,
        above: true,
    },
    Gate {
        name: "emotions.joy > 0.50",
        threshold: 0.50,
        above: true,
    },
    // Endogenous cognitive/relational state.
    Gate {
        name: "cognitive.heuristic_bias > 0.50",
        threshold: 0.50,
        above: true,
    },
    Gate {
        name: "v1 trust > 0.60 (memory encoding)",
        threshold: 0.60,
        above: true,
    },
    Gate {
        name: "v1 trust < 0.50 (marriage strain)",
        threshold: 0.50,
        above: false,
    },
    // Endogenous institutional composites.
    Gate {
        name: "council legitimacy > 0.55",
        threshold: 0.55,
        above: true,
    },
    Gate {
        name: "council legitimacy < 0.50 (faction arm)",
        threshold: 0.50,
        above: false,
    },
];

/// Which world to build for a leg: a plain populated run, or a named crisis.
#[derive(Clone, Copy)]
enum World {
    Calm,
    Collapse,
    Pestilence,
}

fn main() {
    println!("i379 — gate selectivity census (per-agent samples over the sampled window)");
    println!("      'open%' = share of samples that OPEN the gate");
    println!("      calm vs crisis is the point: a gate dark in calm may be a legitimate\n      loneliness/famine trigger, so classify against BOTH.\n");
    for (label, w, h, n, ticks, world) in [
        (
            "calm village 16x16 N=12",
            16u32,
            16u32,
            12u32,
            50_000u64,
            World::Calm,
        ),
        ("calm town 46x46 N=48", 46, 46, 48, 20_000, World::Calm),
        (
            "CRISIS collapse village 16x16 N=12",
            16,
            16,
            12,
            4_320,
            World::Collapse,
        ),
        (
            "CRISIS pestilence town 46x46 N=48",
            46,
            46,
            48,
            20_000,
            World::Pestilence,
        ),
    ] {
        println!("══ {label} ══");
        println!(
            "{:>44} {:>8} {:>8} {:>7} {:>7} {:>7} {:>7} {:>9}",
            "gate", "thresh", "open%", "p05", "p50", "p95", "max", "verdict"
        );
        let mut sim = match world {
            World::Calm => Simulation::new(SimConfig {
                seed: 42,
                max_ticks: ticks,
                world_width: w,
                world_height: h,
                num_agents: n,
                snapshot_interval: None,
            }),
            World::Collapse | World::Pestilence => {
                let mut sc = match world {
                    World::Collapse => Scenario::collapse(),
                    _ => Scenario::pestilence(),
                };
                sc.seed = 42;
                sc.ticks = ticks;
                Simulation::from_scenario(sc)
            }
        };
        sim.populate();
        // Sample every 100 ticks to keep the run cheap while covering the horizon.
        let mut samples: Vec<Vec<f64>> = vec![Vec::new(); GATES.len()];
        let step = 50u64;
        let mut done = 0u64;
        while done < ticks {
            sim.run(step);
            done += step;
            let rel_trust: Vec<f64> = sim
                .relationships()
                .iter()
                .map(|r| r.trust.to_f64())
                .collect();
            for i in 0..sim.agents.len() {
                let a = &sim.agents[i];
                let belief_charge = a
                    .beliefs
                    .iter()
                    .map(|b| b.emotional_charge.to_f64())
                    .fold(f64::NAN, f64::max);
                let vals: Vec<f64> = vec![
                    a.needs.hunger.to_f64(),
                    a.needs.thirst.to_f64(),
                    a.needs.fatigue.to_f64(),
                    a.needs.hunger.to_f64(),
                    a.needs.social.to_f64(),
                    a.needs.social.to_f64(),
                    a.personality.traditionalism.to_f64(),
                    a.personality.openness.to_f64(),
                    a.personality.extraversion.to_f64(),
                    a.personality.ambition.to_f64(),
                    a.personality.conscientiousness.to_f64(),
                    a.personality.traditionalism.to_f64(),
                    a.emotions.fear.to_f64(),
                    a.emotions.anger.to_f64(),
                    a.emotions.joy.to_f64(),
                    a.cognitive.heuristic_bias.to_f64(),
                    // v1 trust and legitimacy use a separate per-sample pool below.
                    f64::NAN,
                    f64::NAN,
                    sim.institutions
                        .iter()
                        .find(|inst| {
                            inst.kind == mindstrata_sim::institutions::InstitutionKind::Council
                        })
                        .map_or(f64::NAN, |inst| inst.legitimacy.to_f64()),
                    sim.institutions
                        .iter()
                        .find(|inst| {
                            inst.kind == mindstrata_sim::institutions::InstitutionKind::Council
                        })
                        .map_or(f64::NAN, |inst| inst.legitimacy.to_f64()),
                ];
                // Belief charge is sampled separately (it is the panic input) — report it
                // through the trust rows' pool to keep the table one row per gate.
                let _ = belief_charge;
                for (gi, v) in vals.into_iter().enumerate() {
                    if v.is_finite() {
                        samples[gi].push(v);
                    }
                }
            }
            // Trust rows draw from the whole relationship pool.
            samples[16].extend(rel_trust.iter().copied());
            samples[17].extend(rel_trust.iter().copied());
        }
        for (gi, gate) in GATES.iter().enumerate() {
            let mut v = std::mem::take(&mut samples[gi]);
            if v.is_empty() {
                println!("{:>44} — no samples", gate.name);
                continue;
            }
            v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let q = |p: f64| v[(((v.len() - 1) as f64) * p).round() as usize];
            let open = v
                .iter()
                .filter(|x| {
                    if gate.above {
                        **x > gate.threshold
                    } else {
                        **x < gate.threshold
                    }
                })
                .count() as f64
                / v.len() as f64
                * 100.0;
            let verdict = if open >= 99.0 {
                "DEAD-HIGH"
            } else if open <= 1.0 {
                "DEAD-LOW"
            } else {
                "live"
            };
            println!(
                "{:>44} {:>8.2} {:>8.1} {:>7.3} {:>7.3} {:>7.3} {:>7.3} {:>9}",
                gate.name,
                gate.threshold,
                open,
                q(0.05),
                q(0.50),
                q(0.95),
                v[v.len() - 1],
                verdict
            );
        }
        println!();
    }
}
