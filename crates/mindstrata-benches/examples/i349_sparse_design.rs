//! i349 — sparse-store design probe: how much do the per-edge folds READ from
//! never-contacted rows?
//!
//! i341 measured the per-edge passes at ~58% of the tick while only ~10% of
//! rows carry interaction state, and pre-registered the contact-driven sparse
//! store — but also recorded that those rows "are semantics": populate seeds a
//! stranger row (random trust 0.3–0.7, affection 0.2–0.6) for every ordered
//! pair, and the per-edge folds (top-3 social support, trust sync, appraisal
//! mean trust) walk all of them. Before any store redesign, this probe sizes
//! the SEMANTIC delta: if the stranger rows vanished, what would the fold
//! outputs actually lose?
//!
//! Legs (seed 42, 32×32, 5K ticks):
//!   1. top-3 social-support composition — how many of the 3 slots per agent
//!      are held by rows with interaction_count == 0 (strangers outranking
//!      friends = a model finding in its own right)?
//!   2. appraisal mean-trust delta — mean trust over ALL rows vs over
//!      contacted rows only, per agent (the fold sparse storage changes).
//!   3. TrustNetwork stranger mass — epistemic trust-network entries fed by
//!      never-contacted rows (the trust_sync prepass copies every row).
//!   4. contacted-row trust distribution vs stranger band — does interaction
//!      lift trust out of the [0.3, 0.7] seed band (i.e. do extremes
//!      differentiate at all)?
//!
//! Run: cargo run --release -p mindstrata-benches --example i349_sparse_design -- [N...]

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::HashSet;

fn main() {
    let ns: Vec<u32> = std::env::args()
        .skip(1)
        .filter_map(|a| a.parse().ok())
        .collect();
    let ns = if ns.is_empty() { vec![12, 48, 96] } else { ns };

    println!("i349 — sparse-store design: stranger-row semantic mass (seed 42, 5K)\n");
    for n in ns {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 5_000,
            world_width: 32,
            world_height: 32,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        for _ in 0..5_000 {
            sim.tick();
        }

        let rels = sim.relationships();
        let total = rels.len();
        let contacted: HashSet<(u64, u64)> = rels
            .iter()
            .filter(|r| r.interaction_count > 0)
            .map(|r| (r.from.as_u64(), r.to.as_u64()))
            .collect();

        // ── Leg 1: top-3 composition (exact scan over (trust, contacted)) ──
        let mut top3_slots = 0usize;
        let mut top3_stranger = 0usize;
        let mut per_from_exact: std::collections::HashMap<u64, Vec<(f64, bool)>> =
            std::collections::HashMap::new();
        for r in rels.iter() {
            per_from_exact
                .entry(r.from.as_u64())
                .or_default()
                .push((r.trust.to_f64(), r.interaction_count > 0));
        }
        for (_, pairs) in &per_from_exact {
            let mut sorted = pairs.clone();
            sorted.sort_by(|a, b| b.0.total_cmp(&a.0));
            for (t, contacted_flag) in sorted.iter().take(3) {
                top3_slots += 1;
                if !contacted_flag {
                    top3_stranger += 1;
                    let _ = t;
                }
            }
        }

        // ── Leg 2: appraisal mean-trust delta ──
        let mut deltas: Vec<f64> = Vec::new();
        for (_, pairs) in &per_from_exact {
            if pairs.is_empty() {
                continue;
            }
            let all: f64 = pairs.iter().map(|p| p.0).sum::<f64>() / pairs.len() as f64;
            let contacted_rows: Vec<f64> = pairs.iter().filter(|p| p.1).map(|p| p.0).collect();
            let cont = if contacted_rows.is_empty() {
                0.5
            } else {
                contacted_rows.iter().sum::<f64>() / contacted_rows.len() as f64
            };
            deltas.push((all - cont).abs());
        }
        deltas.sort_by(f64::total_cmp);
        let dp = |q: f64| -> f64 {
            deltas
                .get(((deltas.len() - 1) as f64 * q).round() as usize)
                .copied()
                .unwrap_or(f64::NAN)
        };

        // ── Leg 3: TrustNetwork stranger mass ──
        let mut tn_entries = 0usize;
        let mut tn_stranger = 0usize;
        let mut tn_stranger_mass = 0.0f64;
        let mut tn_mass = 0.0f64;
        for (i, a) in sim.agents.iter().enumerate() {
            for (id, trust) in &a.epistemic.trust_network.agent_trust {
                tn_entries += 1;
                tn_mass += trust.to_f64();
                if !contacted.contains(&(i as u64, *id)) {
                    tn_stranger += 1;
                    tn_stranger_mass += trust.to_f64();
                }
            }
        }

        // ── Leg 4: trust distributions ──
        let mut contacted_trusts: Vec<f64> = Vec::new();
        let mut stranger_trusts: Vec<f64> = Vec::new();
        for r in rels.iter() {
            if r.interaction_count > 0 {
                contacted_trusts.push(r.trust.to_f64());
            } else {
                stranger_trusts.push(r.trust.to_f64());
            }
        }
        contacted_trusts.sort_by(f64::total_cmp);
        stranger_trusts.sort_by(f64::total_cmp);
        let ct = |v: &[f64], q: f64| -> f64 {
            v.get(((v.len() - 1) as f64 * q).round() as usize)
                .copied()
                .unwrap_or(f64::NAN)
        };

        println!(
            "N={n}: rows {total} · contacted {} ({:.1}%)",
            contacted.len(),
            100.0 * contacted.len() as f64 / total as f64
        );
        println!(
            "  [1] top-3 social-support slots held by NEVER-CONTACTED rows: {top3_stranger}/{top3_slots} = {:.1}%",
            100.0 * top3_stranger as f64 / top3_slots.max(1) as f64
        );
        println!(
            "  [2] |mean trust all − mean trust contacted|: p50 {:.4} · p90 {:.4} · max {:.4}",
            dp(0.5),
            dp(0.9),
            dp(1.0)
        );
        println!(
            "  [3] TrustNetwork entries fed by strangers: {tn_stranger}/{tn_entries} = {:.1}% · mass {:.1}%",
            100.0 * tn_stranger as f64 / tn_entries.max(1) as f64,
            100.0 * tn_stranger_mass / tn_mass.max(1e-9)
        );
        println!(
            "  [4] trust: contacted p10/p50/p90 = {:.3}/{:.3}/{:.3} · strangers {:.3}/{:.3}/{:.3}",
            ct(&contacted_trusts, 0.1),
            ct(&contacted_trusts, 0.5),
            ct(&contacted_trusts, 0.9),
            ct(&stranger_trusts, 0.1),
            ct(&stranger_trusts, 0.5),
            ct(&stranger_trusts, 0.9)
        );
        // ── Leg 5: the honest "touched" census — the sparse-store predicate ──
        // update_witnesses writes trust WITHOUT bumping interaction_count,
        // so `interaction_count > 0` undercounts state-carrying rows. The
        // honest predicate is `interaction_count > 0 || last_interaction_tick > 0`
        // (witness writes stamp the tick) — or rows whose trust left the
        // populate band entirely.
        let mut touched = 0usize;
        let mut saturated = 0usize; // trust == 1.0, no direct interaction (witness ratchet)
        for r in rels.iter() {
            if r.interaction_count > 0 || r.last_interaction_tick > 0 {
                touched += 1;
            }
            if r.trust >= Fixed::from_f64(0.999) && r.interaction_count == 0 {
                saturated += 1;
            }
        }
        println!(
            "  [5] touched rows (count>0 || witness-stamped): {touched} ({:.1}%) · witness-saturated (==1.0, never interacted): {saturated} ({:.1}%)",
            100.0 * touched as f64 / total as f64,
            100.0 * saturated as f64 / total as f64
        );
        println!();
    }
}
