//! i398 — the norms_impl v1→v2 migration, sized before touching.
//!
//! `norms_impl.rs` holds four v1 sites, and the ledger's reader-first sweep
//! starts here because the file contains BOTH halves of the store contract:
//!
//! * **Readers** — one: the belief-evidence channel (L~1029). Every
//!   `InteractionOccurred` event reads `v1.trust` for the speaker→listener pair
//!   and turns `trust − 0.5` into the evidence strength that
//!   `update_beliefs_with_polarity` consumes. This is the only place the
//!   interaction stream reaches beliefs directly.
//! * **Writers** — three: violence damage (victim→attacker AND attacker→victim,
//!   `trust −= 0.3`, `affection −= 0.2`) and norm-punishment damage
//!   (`trust −= 0.15 × punishment`), each also recording a provenance trace.
//!
//! The migration moves all four onto the dyadic store in one commit (the i384
//! rule: a site's read and write move together, or the site simply changes
//! which store it corrupts). This probe measures what that moves:
//!
//! * **A — the reader's input.** For every `InteractionOccurred` event in the
//!   window, both stores' trust for the same pair: the divergence the reader
//!   will inherit, and what it does to `evidence_strength = trust − 0.5`
//!   (sign flips are the sharp edge — they reverse a belief update).
//! * **B — the writer stakes.** How much damage the three sites land in the
//!   window (event counts × magnitudes), and the resulting level of each store.
//!   After the migration the dyadic store carries this damage and v1 does not,
//!   so the *other* v1 readers (household, births_deaths, marriage, legal —
//!   later sweep items) see a slightly higher v1: recorded, not hidden.
//! * **C — the consumer stakes.** Belief proposition 2's charge trajectory
//!   under both evidence sources, projected by re-running the window's belief
//!   updates is out of scope here; instead the probe reports the distribution
//!   of `|Δevidence_strength|` per event — the input the belief update is
//!   linear in (`evidence × source_trust × factors`), so its mean is the
//!   per-event perturbation the sweep should expect.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i398_norms_store`

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn v2_pos(a: usize, b: usize) -> usize {
    if b > a {
        b - 1
    } else {
        b
    }
}

fn village(n: u32, seed: u64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: n,
        snapshot_interval: None,
        ..SimConfig::default()
    });
    sim.populate();
    sim
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn main() {
    println!("i398 — the norms_impl v1→v2 migration, sized\n");

    for (n, horizon) in [(12u32, 5_000u64), (48, 5_000), (48, 20_000)] {
        let t0 = horizon - 2_000;
        let mut sim = village(n, 42, horizon);
        sim.run(t0);
        let events: Vec<SimEvent> = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|ev| matches!(ev, SimEvent::InteractionOccurred { .. }))
            .cloned()
            .collect();

        println!("══ N={n}, seed 42, window {t0}..{horizon} ══");

        // ── A — the reader's input: both stores at the same pairs ────────
        let mut div = Vec::new();
        let mut ev1 = Vec::new();
        let mut ev2 = Vec::new();
        let mut flips = 0usize;
        let mut dv = Vec::new();
        for ev in &events {
            if let SimEvent::InteractionOccurred { from, to, .. } = ev {
                let (fi, ti) = (from.as_u64() as usize, to.as_u64() as usize);
                if fi >= sim.agents.len() || ti >= sim.agents.len() {
                    continue;
                }
                // v1 trust for the pair — the same lookup `rel_pos` performs
                // (first-occurrence scan; the matrix has one row per pair).
                let t1 = sim
                    .relationships
                    .iter()
                    .find(|r| r.from.as_u64() as usize == fi && r.to.as_u64() as usize == ti)
                    .map_or(0.5, |r| r.trust.to_f64());
                let t2 = sim.agents[fi].relationship_v2s[v2_pos(fi, ti)]
                    .trust
                    .to_f64();
                let (e1, e2) = (t1 - 0.5, t2 - 0.5);
                if e1.signum() != e2.signum() {
                    flips += 1;
                }
                div.push((t1 - t2).abs());
                ev1.push(e1.abs());
                ev2.push(e2.abs());
                dv.push((e2 - e1).abs());
            }
        }
        let n_ev = div.len();
        println!(
            "  A  events {n_ev}   |v1−v2| trust mean {:.4} max {:.4}   evidence |E| v1 {:.4} → v2 {:.4}   |ΔE| mean {:.4} max {:.4}   sign flips {flips}",
            mean(&div),
            div.iter().cloned().fold(0.0, f64::max),
            mean(&ev1),
            mean(&ev2),
            mean(&dv),
            dv.iter().cloned().fold(0.0, f64::max),
        );

        // ── B — the writer stakes ─────────────────────────────────────────
        let conf: Vec<SimEvent> = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|ev| matches!(ev, SimEvent::ConflictOccurred { .. }))
            .cloned()
            .collect();
        let violence = conf
            .iter()
            .filter(|ev| {
                matches!(
                    ev,
                    SimEvent::ConflictOccurred {
                        kind: mindstrata_core::conflict::ConflictKind::Violence,
                        ..
                    }
                )
            })
            .count();
        let t1_mean = mean(
            &sim.relationships
                .iter()
                .map(|r| r.trust.to_f64())
                .collect::<Vec<_>>(),
        );
        let t2_mean = mean(
            &sim.agents
                .iter()
                .flat_map(|a| a.relationship_v2s.iter().map(|r| r.trust.to_f64()))
                .collect::<Vec<_>>(),
        );
        println!(
            "  B  window conflicts {}/violence {violence} → 4 damage writes each (0.3/0.2 ×2, 0.15×p ×2)   store levels: v1 {t1_mean:.4}  v2 {t2_mean:.4}",
            conf.len()
        );
        println!();
    }

    println!(
        "reading: the reader inherits |ΔE| ≈ the pair divergence; its consumer is linear in \
         evidence, so the mean |ΔE| is the per-event belief perturbation the sweep should \
         expect. The writers move damage off v1, so the OTHER v1 readers see a slightly \
         higher v1 until their own sweep items land — recorded, not hidden."
    );
}
