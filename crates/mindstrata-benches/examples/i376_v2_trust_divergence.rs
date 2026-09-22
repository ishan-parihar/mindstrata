//! i376 — is the legacy mean-reversion writer a v1↔v2 divergence SOURCE?
//!
//! Two stores hold the same quantity with different laws:
//!   v1 `Relationship.trust`   — interaction gains, plus (pre-i376) mean reversion
//!                               toward 0.5 at `relationship_dormant_decay`/day;
//!   v2 `RelationshipV2.trust` — interaction gains via `record_positive`, plus
//!                               decay toward ZERO at `decay_rate`/tick.
//!
//! Leg A (divergence census): measure the level each store settles at, and the
//! per-pair gap |v1 − v2|, over 5K→50K. Growing = a live divergence source.
//!
//! Leg B (sweep): with the i376 convergence law in place, sweep the coupling
//! `relationship_dormant_decay` ∈ {0.001 (the old rate), 0.05, 0.25, 0.5, 1.0}
//! and report the v1 level, the residual divergence, and the *selectivity* of
//! the v1 trust signal — the share of pairs landing in the discriminating band
//! (0.30, 0.70) that gates like `memory_ops`'s `r.trust > 0.6` need.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i376_v2_trust_divergence`

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::HashMap;

fn pct(v: &mut Vec<f64>, p: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let idx = ((v.len() as f64 - 1.0) * p).round() as usize;
    v[idx]
}

fn mean(v: &[f64]) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v.iter().sum::<f64>() / v.len() as f64
}

struct Stats {
    v1_mean: f64,
    v1_p95: f64,
    v1_pin95: f64,
    v1_pin99: f64,
    v1_discriminating: f64,
    v2_mean: f64,
    v2_p95: f64,
    v2_pin95: f64,
    div_mean: f64,
    div_p50: f64,
    div_p95: f64,
    div_gt05: f64,
    pairs: usize,
}

fn measure(sim: &Simulation) -> Stats {
    let mut v1: HashMap<(u64, u64), f64> = HashMap::new();
    for r in sim.relationships() {
        v1.insert((r.from.as_u64(), r.to.as_u64()), r.trust.to_f64());
    }
    let mut v2_trusts: Vec<f64> = Vec::new();
    let mut v1_trusts: Vec<f64> = Vec::new();
    let mut divs: Vec<f64> = Vec::new();
    for a in sim.agents.iter() {
        for rv2 in a.relationship_v2s.iter() {
            let t2 = rv2.trust.to_f64();
            v2_trusts.push(t2);
            if let Some(t1) = v1.get(&(rv2.from.as_u64(), rv2.to.as_u64())) {
                v1_trusts.push(*t1);
                divs.push((t1 - t2).abs());
            }
        }
    }
    let pairs = divs.len();
    let mut d = divs.clone();
    let mut v1s = v1_trusts.clone();
    let mut v2s = v2_trusts.clone();
    let pin = |v: &[f64], t: f64| -> f64 {
        if v.is_empty() {
            return f64::NAN;
        }
        100.0 * v.iter().filter(|x| **x >= t).count() as f64 / v.len() as f64
    };
    // Selectivity: the share of v1 pairs a threshold gate can still separate —
    // i.e. rows that are neither saturated high nor collapsed low.
    let discriminating = |v: &[f64]| -> f64 {
        if v.is_empty() {
            return f64::NAN;
        }
        100.0 * v.iter().filter(|x| **x > 0.30 && **x < 0.70).count() as f64 / v.len() as f64
    };
    Stats {
        v1_mean: mean(&v1_trusts),
        v1_p95: pct(&mut v1s, 0.95),
        v1_pin95: pin(&v1_trusts, 0.95),
        v1_pin99: pin(&v1_trusts, 0.99),
        v1_discriminating: discriminating(&v1_trusts),
        v2_mean: mean(&v2_trusts),
        v2_p95: pct(&mut v2s, 0.95),
        v2_pin95: pin(&v2_trusts, 0.95),
        div_mean: mean(&divs),
        div_p50: pct(&mut d, 0.50),
        div_p95: pct(&mut d, 0.95),
        div_gt05: if divs.is_empty() {
            f64::NAN
        } else {
            100.0 * divs.iter().filter(|x| **x > 0.05).count() as f64 / divs.len() as f64
        },
        pairs,
    }
}

fn build(seed: u64, coupling: f64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 46,
        world_height: 46,
        num_agents: 48,
        snapshot_interval: None,
    });
    sim.params.relationship_dormant_decay = Fixed::from_f64(coupling);
    sim.populate();
    sim
}

fn leg_a() {
    println!("── Leg A: v1↔v2 trust divergence at the OLD law (coupling 0.001) ──");
    println!(
        "{:>5} {:>7} {:>9} {:>9} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>9} {:>8}",
        "seed",
        "tick",
        "v1.mean",
        "v2.mean",
        "v1.p95",
        "v2.p95",
        "v1%≥.95",
        "v2%≥.95",
        "d.mean",
        "d.p95",
        "d>0.05%",
        "sel%"
    );
    for seed in [42u64, 7, 23] {
        let mut sim = build(seed, 0.001, 50_000);
        let mut t = 0u64;
        for step in [5_000u64, 5_000, 10_000, 30_000] {
            sim.run(step);
            t += step;
            let s = measure(&sim);
            println!(
                "{seed:>5} {t:>7} {:>9.4} {:>9.4} {:>8.4} {:>8.4} {:>8.1} {:>8.1} {:>8.4} {:>8.4} {:>9.1} {:>8.1}",
                s.v1_mean,
                s.v2_mean,
                s.v1_p95,
                s.v2_p95,
                s.v1_pin95,
                s.v2_pin95,
                s.div_mean,
                s.div_p95,
                s.div_gt05,
                s.v1_discriminating
            );
        }
        println!();
    }
}

fn leg_b() {
    println!("── Leg B: coupling sweep with the i376 convergence law (N=48, 50K) ──");
    println!("      sel% = share of pairs in (0.30, 0.70) — the band a trust gate can separate");
    println!(
        "{:>8} {:>7} {:>9} {:>9} {:>8} {:>8} {:>8} {:>9} {:>9} {:>7}",
        "coupling",
        "seed",
        "v1.mean",
        "v2.mean",
        "v1%≥.95",
        "v2%≥.95",
        "sel%",
        "d.mean",
        "d>0.05%",
        "pairs"
    );
    for coupling in [0.001f64, 0.05, 0.25, 0.5, 1.0] {
        for seed in [42u64, 7, 23] {
            let mut sim = build(seed, coupling, 50_000);
            sim.run(50_000);
            let s = measure(&sim);
            println!(
                "{coupling:>8} {seed:>7} {:>9.4} {:>9.4} {:>8.1} {:>8.1} {:>8.1} {:>9.4} {:>9.1} {:>7}",
                s.v1_mean,
                s.v2_mean,
                s.v1_pin95,
                s.v2_pin95,
                s.v1_discriminating,
                s.div_mean,
                s.div_gt05,
                s.pairs
            );
        }
    }
}

fn main() {
    println!("i376 — v1↔v2 trust: divergence census + convergence sweep (46×46, N=48)\n");
    leg_a();
    leg_b();
    let me = AgentId::new(0);
    let rv2 = mindstrata_social::social::relationship_v2::RelationshipV2::new(me, AgentId::new(1));
    println!(
        "(v2 `RelationshipV2::decay` walks trust toward ZERO at decay_rate = {}/tick; \
         `Fixed` resolution is 1e-4)",
        rv2.decay_rate.to_f64()
    );
}
