//! i353 — size the dual-store migration before moving any consumer.
//!
//! The engine carries two relationship stores: the legacy dense v1
//! `relationships` matrix and the per-agent v2 `relationship_v2s`. They are
//! written independently and diverge (i352: 63% of pairs >0.01 at N=48). The
//! plan is to migrate each consumer read+write together, with a sweep — but
//! first: **how much would the first candidate actually move?**
//!
//! Candidate: the marriage pass. It is the one consumer that *both* reads and
//! writes trust/affection for the same pair, so its read and its bond-boost
//! write can move to v2 atomically.
//!
//! Leg A — inventory + divergence per store at behavioural horizons.
//! Leg B — for every marriage-eligible pair, compute the formation chance the
//!         gate sees under v1 vs v2 (same formula, different store) and report
//!         how far the two distributions sit apart. That is the sweep sizer.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i353_dual_store`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_social::social::AttractionModel;

fn build(n: u32, seed: u64, ticks: u64) -> Simulation {
    let side: u32 = if n <= 48 {
        if n <= 12 {
            16
        } else {
            32
        }
    } else {
        46
    };
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: side,
        world_height: side,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(ticks);
    sim
}

/// Leg A — per-store divergence over all pairs, at two horizons.
fn divergence_leg() {
    println!("== Leg A: v1 vs v2 divergence (all pairs) ==");
    println!(
        "{:<5} {:<5} {:>6} {:>9} {:>9} {:>9} {:>9} {:>8}",
        "N", "ticks", "pairs", "dTr50", "dTr90", "dAf50", "dAf90", ">0.01"
    );
    for n in [12u32, 48] {
        for ticks in [2_000u64, 20_000] {
            let sim = build(n, 42, ticks);
            let mut dtr: Vec<f64> = Vec::new();
            let mut daf: Vec<f64> = Vec::new();
            let mut material = 0usize;
            for r in sim.relationships.iter() {
                let from = r.from.as_u64() as usize;
                let to = r.to.as_u64() as usize;
                if from >= n as usize || to >= n as usize || from == to {
                    continue;
                }
                let pos = if to > from { to - 1 } else { to };
                let Some(v2) = sim.agents[from].relationship_v2s.get(pos) else {
                    continue;
                };
                let dt = (r.trust - v2.trust).to_f64().abs();
                let da = (r.affection - v2.affection).to_f64().abs();
                if dt > 0.01 || da > 0.01 {
                    material += 1;
                }
                dtr.push(dt);
                daf.push(da);
            }
            dtr.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            daf.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let p = |v: &[f64], q: f64| -> f64 {
                if v.is_empty() {
                    0.0
                } else {
                    v[(q * (v.len() - 1) as f64) as usize]
                }
            };
            println!(
                "{:<5} {:<5} {:>6} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>8}",
                n,
                ticks,
                dtr.len(),
                p(&dtr, 0.5),
                p(&dtr, 0.9),
                p(&daf, 0.5),
                p(&daf, 0.9),
                material
            );
        }
    }
}

/// Leg B — the marriage gate's formation chance under v1 vs v2.
fn marriage_gate_leg() {
    println!("\n== Leg B: marriage-gate chance, v1 read vs v2 read ==");
    println!(
        "{:<5} {:<5} {:>8} {:>10} {:>10} {:>9} {:>9}",
        "N", "ticks", "eligible", "mean v1", "mean v2", "ratio", "cross>10%"
    );
    let rate = 0.0004_f64; // the i351-calibrated default
                           // Early horizons: the formation gate only matters while adults are still
                           // unpartnered (by ~5K everyone is paired, so `eligible` collapses).
    for n in [12u32, 48] {
        for ticks in [0u64, 50, 150, 300, 500] {
            let sim = build(n, 42, ticks);
            let n_agents = n as usize;
            let adults = sim
                .agents
                .iter()
                .filter(|a| a.age >= Fixed::from_f64(18.0))
                .count();
            let unpartnered = sim
                .agents
                .iter()
                .filter(|a| a.age >= Fixed::from_f64(18.0) && a.partner.is_none())
                .count();
            eprintln!("  [diag] N={n} t={ticks}: adults={adults} unpartnered-adults={unpartnered}");
            let mut eligible = 0usize;
            let mut cross = 0usize;
            let mut sum_v1 = 0.0f64;
            let mut sum_v2 = 0.0f64;
            for i in 0..n_agents {
                let ai = &sim.agents[i];
                if ai.partner.is_some() || ai.age < Fixed::from_f64(18.0) {
                    continue;
                }
                for j in (i + 1)..n_agents {
                    let aj = &sim.agents[j];
                    if aj.partner.is_some() || aj.age < Fixed::from_f64(18.0) {
                        continue;
                    }
                    if (ai.age - aj.age).abs() > Fixed::from_f64(15.0) {
                        continue;
                    }
                    let pos = if j > i { j - 1 } else { j };
                    let Some(v2) = ai.relationship_v2s.get(pos) else {
                        continue;
                    };
                    // v1 read via the public accessor (rel_pos is pub(crate));
                    // the scan is probe-only and matches find-first semantics.
                    let (from_id, to_id) = (
                        mindstrata_core::id::AgentId::new(i as u64),
                        mindstrata_core::id::AgentId::new(j as u64),
                    );
                    let v1 = sim
                        .relationships()
                        .iter()
                        .find(|r| r.from == from_id && r.to == to_id)
                        .map_or((Fixed::ZERO, Fixed::ZERO), |r| (r.trust, r.affection));
                    let personality_compat = Fixed::ONE
                        - (ai.personality.agreeableness - aj.personality.agreeableness).abs();
                    let distance =
                        Fixed::from_int(ai.position.manhattan_distance(&aj.position) as i64);
                    let proximity =
                        (Fixed::ONE - distance / Fixed::from_f64(20.0)).max(Fixed::ZERO);
                    let health = (ai.body.health + aj.body.health) * Fixed::from_f64(0.5);
                    let model = |affection: Fixed| AttractionModel {
                        personality_attraction: personality_compat,
                        familiarity: affection,
                        physical_attraction: proximity,
                        reciprocity: affection,
                        ..AttractionModel::default()
                    };
                    let c1 =
                        (model(v1.1).total_attraction() * health * v1.0 * Fixed::from_f64(rate))
                            .to_f64();
                    let c2 = (model(v2.affection).total_attraction()
                        * health
                        * v2.trust
                        * Fixed::from_f64(rate))
                    .to_f64();
                    eligible += 1;
                    sum_v1 += c1;
                    sum_v2 += c2;
                    let denom = c1.abs().max(1e-12);
                    if (c2 - c1).abs() / denom > 0.10 {
                        cross += 1;
                    }
                }
            }
            let m1 = if eligible > 0 {
                sum_v1 / eligible as f64
            } else {
                0.0
            };
            let m2 = if eligible > 0 {
                sum_v2 / eligible as f64
            } else {
                0.0
            };
            println!(
                "{:<5} {:<5} {:>8} {:>10.6} {:>10.6} {:>9.3} {:>9}",
                n,
                ticks,
                eligible,
                m1,
                m2,
                if m1 > 0.0 { m2 / m1 } else { 0.0 },
                cross
            );
        }
    }
}

fn main() {
    divergence_leg();
    marriage_gate_leg();
}
