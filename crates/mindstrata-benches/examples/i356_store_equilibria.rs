//! i356 — the dual-store question re-framed as a **liveness** question.
//!
//! i353 measured v1-vs-v2 *divergence* and called the stores split. But
//! divergence between two quantities is not itself a bug — the bug class the
//! doctrine owns (§4.3) is a **dead or saturated producer**. So before moving
//! any consumer, ask the sharper question: what equilibrium does each store
//! settle at?
//!
//! The two stores decay by *different laws*: v1 gets the daily mean-reversion
//! `trust -= (trust − 0.5)·decay` (`systems/cognitive.rs`), pulling it toward a
//! 0.5 baseline; v2 gets `RelationshipV2::decay`, pulling trust toward **0**.
//! If v2 trust collapses toward zero population-wide, then the richer
//! psychological model that appraisal/cognitive/status-centrality actually
//! reason over is a dead quantity, while the observable v1 graph is alive —
//! exactly the split that matters.
//!
//! Legs:
//!   A — per-store field equilibria (mean / p10 / p50 / p90) over all pairs.
//!   B — dead-row share: fraction of v2 rows whose trust has collapsed (<0.01),
//!       and the v1 counterpart, at behavioural horizons.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i356_store_equilibria`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

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

fn pct(v: &[f64], q: f64) -> f64 {
    if v.is_empty() {
        return f64::NAN;
    }
    v[(q * (v.len() - 1) as f64) as usize]
}

fn main() {
    println!("== i356: per-store equilibria — is v2 dead? ==");
    println!(
        "{:<5} {:<7} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8} {:>8}",
        "N", "ticks", "v1_trµ", "v1_tr50", "v2_trµ", "v2_tr50", "v1_afµ", "v2_afµ", "v2_dead"
    );
    for n in [12u32, 48] {
        for ticks in [500u64, 2_000, 10_000, 20_000] {
            let sim = build(n, 42, ticks);
            let mut v1t: Vec<f64> = Vec::new();
            let mut v2t: Vec<f64> = Vec::new();
            let mut v1a: Vec<f64> = Vec::new();
            let mut v2a: Vec<f64> = Vec::new();
            let mut v2_dead = 0usize;
            let mut v2_total = 0usize;
            for r in sim.relationships() {
                let from = r.from.as_u64() as usize;
                let to = r.to.as_u64() as usize;
                if to >= n as usize {
                    continue;
                }
                let pos = if to > from { to - 1 } else { to };
                let Some(v2) = sim.agents[from].relationship_v2s.get(pos) else {
                    continue;
                };
                let vt = v2.trust.to_f64();
                v2_total += 1;
                if vt < 0.01 {
                    v2_dead += 1;
                }
                v1t.push(r.trust.to_f64());
                v2t.push(vt);
                v1a.push(r.affection.to_f64());
                v2a.push(v2.affection.to_f64());
            }
            v1t.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            v2t.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
            let mean = |v: &[f64]| {
                if v.is_empty() {
                    0.0
                } else {
                    v.iter().sum::<f64>() / v.len() as f64
                }
            };
            let m1t = mean(&v1t);
            let m2t = mean(&v2t);
            let m1a = mean(&v1a);
            let m2a = mean(&v2a);
            println!(
                "{:<5} {:<7} {:>8.4} {:>8.4} {:>8.4} {:>8.4} {:>8.4} {:>8.4} {:>7.2}%",
                n,
                ticks,
                m1t,
                pct(&v1t, 0.5),
                m2t,
                pct(&v2t, 0.5),
                m1a,
                m2a,
                if v2_total > 0 {
                    100.0 * v2_dead as f64 / v2_total as f64
                } else {
                    0.0
                }
            );
        }
    }
    let _ = Fixed::ONE;
}
