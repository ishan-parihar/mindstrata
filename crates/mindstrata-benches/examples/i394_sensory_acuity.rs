//! i392 row 4 — `sensory_acuity` against the salience bias: **MEASURED AND
//! REJECTED** (the row-2 outcome, for a different mechanism).
//!
//! i391's census flagged `sensory_acuity` as dead, and i391's follow-up table
//! named "the i334 perception radius" as its consumer. That consumer is
//! **wrong for the gene**: the radius is an integer (tiles), so an acuity
//! multiplier over it is a 3-bin staircase — the §4.15 gate-2 cliff. The
//! class-consistent candidate was the attention system's hardcoded
//! `salience_bias = 0.5` (the gene draw's mean, `U(0.2, 0.8)` → 0.5). The
//! wiring was built as a field-setter at construction, measured with this
//! probe, and **reverted**:
//!
//! * **A — the anchor was exact**: pinned-0.5 reproduces BOTH stored goldens
//!   byte for byte (riverford_minor MATCH/MATCH; collapse MATCH/MATCH once the
//!   probe uses `from_scenario` — a raw SimConfig diverges because the
//!   scenario pre-sets drought_until). On the reverted tree the natural world
//!   is the control (B reads no divergence in 2 000 ticks).
//! * **C — the response is a threshold lottery, not a gradient**: traces
//!   79 → 337 → 674 → 1 126 → 1 132 across genes 0.2 → 0.8. The encode
//!   threshold (salience ≥ 0.2) sits INSIDE the gene's draw range, and fresh
//!   own-help interactions compute salience ≈ 0.216 — marginally above it — so
//!   the multiplier decides *who encodes at all*, amplifying mean-trace count
//!   13× across the gene range. §4.15 gate 2 fails by measurement (a band
//!   edge, not a coefficient, consumes the trait).
//! * **D — the revolution liveness family collapsed to 1/3** (0/0/2 vs the
//!   pinned ≥2 of {5, 42, 12345}; pestilence @70K, meme mutation isolated —
//!   the i388 config). Mechanism: suppressed encoding shifts the first
//!   deca-tick `rehearse_random` draw on the SHARED Behavior stream (§5 RNG
//!   discipline), re-timing every downstream draw. A dead producer — §2.3
//!   forbids re-pinning that away.
//!
//! Per the row-2 precedent the wiring was reverted (not re-anchored); this
//! probe is the rejection record, and the trait stays deliberately inert
//! until a consumer exists whose band edges are NOT governed by it.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i394_sensory_acuity`
//!

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// Count per-agent memory-store occupancy and flashbulb traces at the final tick.
fn memory_stats(sim: &Simulation) -> (usize, usize, usize, f64) {
    let agents = sim.agents.len();
    let mut traces = 0usize;
    let mut flashbulbs = 0usize;
    let mut owners = 0usize;
    for a in &sim.agents {
        let n = a.memory.episodes.len();
        traces += n;
        if n > 0 {
            owners += 1;
        }
        flashbulbs += a
            .memory
            .episodes
            .iter()
            .filter(|m| m.kind == mindstrata_sim::memory::MemoryKind::Flashbulb)
            .count();
    }
    let mean = if agents > 0 {
        traces as f64 / agents as f64
    } else {
        0.0
    };
    (traces, flashbulbs, owners, mean)
}

/// Build a village with every agent's acuity gene pinned to `acuity` (or the
/// natural draw when `None`).
fn village(n: u32, seed: u64, ticks: u64, acuity: Option<f64>) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    if let Some(g) = acuity {
        for a in &mut sim.agents {
            a.embodied.genome.physical_potential.sensory_acuity = Fixed::from_f64(g);
            a.attention.salience_bias = Fixed::from_f64(g);
        }
    }
    sim
}

/// The metric hash the goldens store, computed the way `golden_replay.rs` does
/// (hash over `to_bits()` of the same 11 metric fields, in the same order).
fn golden_hashes(sim: &Simulation) -> (u64, u64) {
    use std::hash::{Hash, Hasher};
    let hash_metrics = |metrics: &[f64]| {
        let mut h = std::collections::hash_map::DefaultHasher::new();
        for m in metrics {
            m.to_bits().hash(&mut h);
        }
        h.finish()
    };
    let ms = sim.metrics_snapshot();
    let metric_hash = hash_metrics(&[
        ms.avg_hunger,
        ms.avg_thirst,
        ms.avg_fatigue,
        ms.avg_valence,
        ms.avg_joy,
        ms.avg_fear,
        ms.total_grain,
        ms.total_water,
        ms.event_count as f64,
        ms.journal_len as f64,
        ms.agent_count as f64,
    ]);
    let summaries = sim.agent_summaries();
    let agent_hash = hash_metrics(
        &summaries
            .iter()
            .flat_map(|s| {
                [
                    s.hunger.to_f64(),
                    s.thirst.to_f64(),
                    s.fatigue.to_f64(),
                    s.valence.to_f64(),
                ]
            })
            .collect::<Vec<_>>(),
    );
    (metric_hash, agent_hash)
}

fn main() {
    println!("i392 row 4 — sensory_acuity → attention.salience_bias\n");

    // ── A — control arm: gene pinned to the retired constant ──────────
    // The stored goldens: riverford_minor (calm 1 000 ticks, 12 agents) and
    // collapse (4 320 ticks). Both are 16×16 worlds — `run_golden_configured`'s
    // exact config.
    println!("══ A — control: pinned-0.5 must reproduce the stored goldens ══");
    {
        let stored: [(&str, u64, u64, u64, u32); 2] = [
            (
                "riverford_minor",
                1000,
                12871778371033085037,
                17325466158580800444,
                12,
            ),
            (
                "collapse",
                4320,
                11026946663692602518,
                12947111318289908785,
                12,
            ),
        ];
        for (name, ticks, sm, sa, n) in stored {
            // riverford_minor is the plain 16×16 config; collapse is a
            // scenario world (`from_scenario` pre-sets drought_until from its
            // drought shock — a raw SimConfig would diverge before the gate
            // is even reached).
            let mut sim = if name == "collapse" {
                let sc = Scenario::collapse();
                let mut sim = Simulation::from_scenario(sc);
                sim.populate();
                sim
            } else {
                let mut sim = Simulation::new(SimConfig {
                    seed: 42,
                    max_ticks: ticks,
                    world_width: 16,
                    world_height: 16,
                    num_agents: n,
                    snapshot_interval: None,
                });
                sim.populate();
                sim
            };
            for a in &mut sim.agents {
                a.embodied.genome.physical_potential.sensory_acuity = Fixed::from_f64(0.5);
                a.attention.salience_bias = Fixed::from_f64(0.5);
            }
            sim.run(ticks);
            let (mh, ah) = golden_hashes(&sim);
            println!(
                "  {name:<16} metric {} · agent {}",
                if mh == sm {
                    "MATCH".to_string()
                } else {
                    format!("DIFFER (got {mh:#x} want {sm:#x})")
                },
                if ah == sa {
                    "MATCH".to_string()
                } else {
                    format!("DIFFER (got {ah:#x} want {sa:#x})")
                },
            );
        }
    }
    println!();

    // ── B — attribution: first divergence from the control ────────────
    println!("══ B — attribution: natural-gene world vs pinned-0.5 control ══");
    {
        let ticks = 2000u64;
        let mut control = village(12, 42, 0, Some(0.5));
        let mut natural = village(12, 42, 0, None);
        let mut found = false;
        for t in 0..ticks {
            control.run(1);
            natural.run(1);
            if !found {
                for (c, n) in control.agents.iter().zip(natural.agents.iter()) {
                    let c_v = c.affect.valence.to_f64().to_bits();
                    let n_v = n.affect.valence.to_f64().to_bits();
                    if c_v != n_v {
                        let gene = n.embodied.genome.physical_potential.sensory_acuity.to_f64();
                        println!(
                            "  first divergence tick {t}: agent {} valence differs \
                             (gene {gene:.4} → ×{:.3}); population mean multiplier {:.4}",
                            c.name,
                            gene,
                            natural
                                .agents
                                .iter()
                                .map(|a| {
                                    a.embodied.genome.physical_potential.sensory_acuity.to_f64()
                                })
                                .sum::<f64>()
                                / natural.agents.len() as f64
                        );
                        found = true;
                        break;
                    }
                }
            }
        }
        if !found {
            println!("  no divergence in 2000 ticks (band closed)");
        }
        let (ct, cf, co, cm) = memory_stats(&control);
        let (nt, nf, no, nm) = memory_stats(&natural);
        println!(
            "  memory @2K  control: {ct} traces / {cf} flashbulbs / {co} owners (mean {cm:.2})\n  memory @2K  natural: {nt} traces / {nf} flashbulbs / {no} owners (mean {nm:.2})"
        );
    }
    println!();

    // ── C — the response curve at pinned genes ─────────────────────────
    println!("══ C — response curve (pinned gene → memory outcome, N=12 s42 @2K) ══");
    println!("gene   traces  flashbulbs  owners  mean/agent");
    for g in [0.2f64, 0.35, 0.5, 0.65, 0.8] {
        let sim = {
            let mut s = village(12, 42, 0, None);
            for a in &mut s.agents {
                a.embodied.genome.physical_potential.sensory_acuity = Fixed::from_f64(g);
                a.attention.salience_bias = Fixed::from_f64(g);
            }
            s.run(2000);
            s
        };
        let (t, f, o, m) = memory_stats(&sim);
        println!("{g:<6} {t:>6}  {f:>10}  {o:>6}  {m:>10.2}");
    }
    println!();

    // ── D — liveness families (the row-2 kill criterion) ──────────────
    // The revolution family's pinned contract (governance.rs, i388):
    // pestilence @70K, meme mutation isolated, seeds {5, 42, 12345},
    // ≥ 2 of 3 must fire. Same config, same measurement (`ConflictOccurred
    // { kind: Revolution }` via recent_events).
    println!("══ D — liveness: revolution family (pestilence @70K) ══");
    let mut firing_seeds = 0u32;
    for seed in [5u64, 42, 12345] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = 70_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.params.meme_mutation_rate_base = Fixed::ZERO;
        sim.populate();
        sim.run(70_000);
        let revs = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    mindstrata_core::event::SimEvent::ConflictOccurred {
                        kind: mindstrata_core::conflict::ConflictKind::Revolution,
                        ..
                    }
                )
            })
            .count();
        if revs > 0 {
            firing_seeds += 1;
        }
        println!("  revolution s{seed:<6} revolutions: {revs}");
    }
    println!("  firing seeds: {firing_seeds}/3 (pinned family floor: ≥ 2)");
    println!();
    println!(
        "reading: A proves the anchor is exact; B attributes the divergence; C shows \
    the response the gene buys; D checks no liveness producer went dark (the row-2 \
    criterion). If D regresses, the row is rejected and reverted, not re-anchored."
    );
}
