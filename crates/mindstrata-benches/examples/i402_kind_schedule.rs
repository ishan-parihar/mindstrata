//! i402 — the interaction-kind schedule reads the dyadic store: sized before touching.
//!
//! **Verdict (2026-09-25, `evidence/i402_kind_schedule.md`)**: the premise is refuted —
//! the signal is saturated on BOTH stores (contacted-pair affection p50 = 1.0000 on v1,
//! 0.9986–0.9997 on v2), branch flips 0.0–3.5%, and the 0.70 gate is the §4.10 defect.
//! The read move + gate redesign fold into i403 (writer deletion), re-derived against the
//! post-deletion surface. This probe re-runs there as the pre-change leg.
//!
//! i401 §3 localised the attachment channel's v1 producer to one hop upstream of
//! attachment, in `system_social_interactions`:
//!
//! ```text
//! let (trust, affection) = pair.map_or((default_trust, default_affection), |p|
//!     (relationships[p].trust, relationships[p].affection));   // ← v1, the write i401 deletes
//! let kind = choose_interaction(trust, affection, …, rng, params);
//! ```
//!
//! `choose_interaction` branches on those two numbers (`trust < 0.2`, `affection > 0.7`),
//! and the branch selects the interaction kind — which is what feeds §8.1.14 (Comfort =
//! the `receive_comfort` drain; Gossip = the `on_reunion` recovery). i401's hypothesis is
//! that the v1 write was a **ratchet**: every positive interaction pushed v1 affection up,
//! the daily i376 sync only pulled it back toward v2, so the value this schedule read sat
//! *systematically above* its v2 counterpart — parking pairs in the high-affection branch.
//!
//! This probe measures that claim instead of assuming it, at the exact read pair of every
//! interaction the schedule actually decided:
//!
//! * **A — the ratchet, signed.** Per `InteractionOccurred` event, both stores' trust and
//!   affection for the same pair. Signed means are the hypothesis test (`mean(v1 − v2) > 0`
//!   for affection = the ratchet); the unsigned means frame the perturbation.
//! * **B — the branch boundaries.** How many of those pairs sit on the wrong side of the
//!   two gates (`affection > social_high_affection_threshold`, `trust <
//!   social_low_trust_threshold`) when read from v1 versus v2 — i.e. how many *branch
//!   selections* the migration moves. This is the sensitive quantity: a boundary flip
//!   rewrites the kind, not merely the magnitude.
//! * **C — the kind mix** the schedule produced in the window, and **the threshold the
//!   migration would need**: the v2-affection quantile matching the current v1-era
//!   high-affection share, so the re-derivation is a measurement rather than a widening.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i402_kind_schedule`

use mindstrata_core::event::{InteractionKind, SimEvent};
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
    });
    sim.populate();
    sim
}

fn mean(xs: &[f64]) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.iter().sum::<f64>() / xs.len() as f64
}

fn quantile(xs: &mut [f64], q: f64) -> f64 {
    if xs.is_empty() {
        return 0.0;
    }
    xs.sort_by(f64::total_cmp);
    let i = ((xs.len() - 1) as f64 * q).round() as usize;
    xs[i]
}

fn kind_name(k: InteractionKind) -> &'static str {
    match k {
        InteractionKind::Talk => "Talk",
        InteractionKind::Help => "Help",
        InteractionKind::Threaten => "Threaten",
        InteractionKind::Trade => "Trade",
        InteractionKind::Gossip => "Gossip",
        InteractionKind::Comfort => "Comfort",
        InteractionKind::Insult => "Insult",
        InteractionKind::Teach => "Teach",
    }
}

/// Leg D — is the affection gate a **latch**?
///
/// The schedule picks the interaction kind from a *step* at 0.7, and the kind is a
/// magnitude lever in `pass_social` (Help/Comfort 0.6, Gossip/Talk 0.3, Trade 0.2).
/// So the state "affection > 0.7" is self-reinforcing (above → 0.6 → grows; below →
/// 0.3 → grows slower, if at all against decay). If the below-state is sticky on the
/// store the gate reads *after* a migration, then a read move that tips only a few
/// percent of decisions is not a few percent of behaviour — the tips accumulate.
///
/// Measured on the dyadic store (the migration target), sampled every `step` ticks:
/// the transition matrix across 0.7 and the mean per-sample drift in each state.
fn latch(n: u32, seed: u64, horizon: u64, step: u64) {
    let mut sim = village(n, seed, horizon);
    let mut prev: Option<Vec<bool>> = None;
    let (mut stay_hi, mut down, mut stay_lo, mut up) = (0usize, 0usize, 0usize, 0usize);
    let (mut d_hi, mut d_lo) = (Vec::new(), Vec::new());
    let mut prev_aff: Option<Vec<f64>> = None;

    let mut t = 0;
    while t < horizon {
        // `run(n)` executes n ticks (a `for _ in 0..n` loop) — it does NOT advance
        // to tick n. Stepping by `step` per call keeps the total at `horizon`
        // ticks; calling `run(t + step)` here would re-run the whole prefix every
        // sample (Σ(t+step) = 10.5× `horizon`) and read a deep-future world.
        sim.run(step);
        t += step;
        let mut state = Vec::new();
        let mut aff = Vec::new();
        for i in 0..n as usize {
            for j in 0..n as usize {
                if i == j || j >= sim.agents.len() {
                    continue;
                }
                let a = sim.agents[i].relationship_v2s[v2_pos(i, j)]
                    .affection
                    .to_f64();
                state.push(a > 0.7);
                aff.push(a);
            }
        }
        if let (Some(p), Some(pa)) = (&prev, &prev_aff) {
            for k in 0..state.len().min(p.len()) {
                match (p[k], state[k]) {
                    (true, true) => stay_hi += 1,
                    (true, false) => down += 1,
                    (false, false) => stay_lo += 1,
                    (false, true) => up += 1,
                }
                if p[k] {
                    d_hi.push(aff[k] - pa[k]);
                } else {
                    d_lo.push(aff[k] - pa[k]);
                }
            }
        }
        prev = Some(state);
        prev_aff = Some(aff);
    }

    let hi = (stay_hi + down) as f64;
    let lo = (stay_lo + up) as f64;
    println!(
        "  D  latch (dyadic affection, {step}-tick samples, N={n} seed {seed}):\n     \
         from-above {:.0}: {:.2}% stay above, {:.2}% fall below   drift/sample {:+.5}\n     \
         from-below {:.0}: {:.2}% stay below, {:.2}% rise above   drift/sample {:+.5}",
        hi,
        100.0 * stay_hi as f64 / hi.max(1.0),
        100.0 * down as f64 / hi.max(1.0),
        mean(&d_hi),
        lo,
        100.0 * stay_lo as f64 / lo.max(1.0),
        100.0 * up as f64 / lo.max(1.0),
        mean(&d_lo),
    );
    println!();
}

fn main() {
    println!("i402 — the interaction-kind schedule's v1 input, sized (probe)\n");

    let params = mindstrata_core::parameters::SimParameters::default();
    let hi = params.social_high_affection_threshold.to_f64();
    let lo = params.social_low_trust_threshold.to_f64();

    // Legs are selectable — the full sweep is ~20 min of release CPU across nine
    // 20K-tick villages, and the latch leg needs its own run to read cleanly.
    let legs = std::env::args().nth(1).unwrap_or_else(|| "abcd".into());
    if legs.contains('d') {
        for (n, seed, horizon, step) in
            [(12u32, 42u64, 20_000u64, 1_000u64), (48, 42, 20_000, 1_000)]
        {
            latch(n, seed, horizon, step);
        }
    }
    if !legs.contains('a') {
        return;
    }

    for (n, seed, horizon) in [(12u32, 42u64, 20_000u64), (48, 42, 20_000), (48, 7, 20_000)] {
        let t0 = horizon - 2_000;
        let mut sim = village(n, seed, horizon);
        // One `run(horizon)` executes exactly `horizon` ticks from zero (the
        // window filter below then selects the last 2K) — a fresh sim plus
        // `run(t0)` would also be `t0` ticks, so both shapes work here; only
        // the single-call form keeps the requested window non-empty.
        sim.run(horizon);

        // i327: `recent_events` is a bounded buffer, so the retained window is NOT
        // necessarily the requested one — filter on tick and report the real span,
        // or every rate below is over an unknown denominator.
        let retained: Vec<SimEvent> = sim.recent_events(10_000_000).to_vec();
        let span = retained
            .iter()
            .find_map(|ev| match ev {
                SimEvent::InteractionOccurred { tick, .. } => Some(tick.as_u64()),
                _ => None,
            })
            .unwrap_or(0);
        let events: Vec<SimEvent> = retained
            .iter()
            .filter(|ev| {
                matches!(ev, SimEvent::InteractionOccurred { tick, .. } if tick.as_u64() >= t0)
            })
            .cloned()
            .collect();
        let ticks = horizon.saturating_sub(t0).max(1);

        println!(
            "══ N={n}, seed {seed}, window {t0}..{horizon}  (buffer starts at tick {span}, 
             {ticks} ticks) ══"
        );

        // ── Z — is the pass even live across the retained window? ───────
        // The bounded buffer means an unwindowed read spans ~10K ticks; bucketing the
        // interaction events by 1 000 ticks separates "the schedule reads the wrong
        // store" from "the pass went dark", which are different bugs (§4.3).
        println!("     population {}", sim.agents.len());
        for b in (span / 1_000)..=(horizon.saturating_sub(1) / 1_000) {
            let (lo_b, hi_b) = (b * 1_000, (b + 1) * 1_000);
            let c = retained
                .iter()
                .filter(|ev| {
                    matches!(ev, SimEvent::InteractionOccurred { tick, .. }
                        if tick.as_u64() >= lo_b && tick.as_u64() < hi_b)
                })
                .count();
            let bar = "█".repeat((c / 10).min(60));
            println!("     ticks {lo_b:>6}..{hi_b:<6} {c:>6}  {bar}");
        }
        println!();

        // ── A — the ratchet, signed ─────────────────────────────────────
        let (mut d_trust, mut d_aff) = (Vec::new(), Vec::new());
        let (mut s_trust, mut s_aff) = (Vec::new(), Vec::new());
        // ── B — the two gates, both stores ──────────────────────────────
        let (mut hi_v1, mut hi_v2) = (0usize, 0usize);
        let mut hi_flip = 0usize;
        let (mut lo_v1, mut lo_v2) = (0usize, 0usize);
        let mut lo_flip = 0usize;
        // ── C — the mix, and the migration's candidate threshold ────────
        let mut mix: Vec<(&'static str, usize)> = Vec::new();
        let mut aff1_all = Vec::new();
        let mut aff2_all = Vec::new();

        for ev in &events {
            let SimEvent::InteractionOccurred { from, to, kind, .. } = ev else {
                continue;
            };
            let (fi, ti) = (from.as_u64() as usize, to.as_u64() as usize);
            if fi >= sim.agents.len() || ti >= sim.agents.len() {
                continue;
            }
            // v1 for the pair — the same lookup `rel_pos` performs (first-occurrence
            // scan; the matrix holds one row per pair).
            let rel = sim
                .relationships
                .iter()
                .find(|r| r.from.as_u64() as usize == fi && r.to.as_u64() as usize == ti);
            let (t1, a1) = rel.map_or(
                (
                    params.social_default_trust.to_f64(),
                    params.social_default_affection.to_f64(),
                ),
                |r| (r.trust.to_f64(), r.affection.to_f64()),
            );
            let d = &sim.agents[fi].relationship_v2s[v2_pos(fi, ti)];
            let (t2, a2) = (d.trust.to_f64(), d.affection.to_f64());

            d_trust.push((t1 - t2).abs());
            d_aff.push((a1 - a2).abs());
            s_trust.push(t1 - t2);
            s_aff.push(a1 - a2);
            aff1_all.push(a1);
            aff2_all.push(a2);

            if a1 > hi {
                hi_v1 += 1;
            }
            if a2 > hi {
                hi_v2 += 1;
            }
            if (a1 > hi) != (a2 > hi) {
                hi_flip += 1;
            }
            if t1 < lo {
                lo_v1 += 1;
            }
            if t2 < lo {
                lo_v2 += 1;
            }
            if (t1 < lo) != (t2 < lo) {
                lo_flip += 1;
            }

            let name = kind_name(*kind);
            match mix.iter_mut().find(|(k, _)| *k == name) {
                Some(slot) => slot.1 += 1,
                None => mix.push((name, 1)),
            }
        }

        let n_ev = d_trust.len();
        if n_ev == 0 {
            println!("  (no interactions in the window)\n");
            continue;
        }
        println!(
            "  A  events {n_ev} over {ticks} ticks = {:.1}/tick, {:.2}/agent/tick\n     \
             trust     |Δ| mean {:.4} max {:.4}   signed mean {:+.4}\n     \
             affection |Δ| mean {:.4} max {:.4}   signed mean {:+.4}   ← ratchet sign",
            n_ev as f64 / ticks as f64,
            n_ev as f64 / ticks as f64 / n as f64,
            mean(&d_trust),
            d_trust.iter().copied().fold(0.0, f64::max),
            mean(&s_trust),
            mean(&d_aff),
            d_aff.iter().copied().fold(0.0, f64::max),
            mean(&s_aff),
        );
        println!(
            "  B  high-affection gate (>{hi:.2}): v1 {hi_v1} ({:.1}%)  v2 {hi_v2} ({:.1}%)  \
             branch flips {hi_flip} ({:.1}%)\n     \
             low-trust gate     (<{lo:.2}): v1 {lo_v1} ({:.1}%)  v2 {lo_v2} ({:.1}%)  \
             branch flips {lo_flip} ({:.1}%)",
            100.0 * hi_v1 as f64 / n_ev as f64,
            100.0 * hi_v2 as f64 / n_ev as f64,
            100.0 * hi_flip as f64 / n_ev as f64,
            100.0 * lo_v1 as f64 / n_ev as f64,
            100.0 * lo_v2 as f64 / n_ev as f64,
            100.0 * lo_flip as f64 / n_ev as f64,
        );

        mix.sort_by_key(|a| std::cmp::Reverse(a.1));
        let mix_str = mix
            .iter()
            .map(|(k, c)| format!("{k} {:.1}%", 100.0 * *c as f64 / n_ev as f64))
            .collect::<Vec<_>>()
            .join("  ");
        println!("  C  mix  {mix_str}");

        // The re-derivation point: the v2-affection value carrying the same share of
        // contacted pairs as the v1-era high-affection branch does today.
        let share = hi_v1 as f64 / n_ev as f64;
        let mut sorted = aff2_all;
        let q_hi = quantile(&mut sorted, 1.0 - share);
        let a2_p50 = quantile(&mut sorted.clone(), 0.5);
        let a2_p90 = quantile(&mut sorted.clone(), 0.9);
        let a1_p50 = quantile(&mut aff1_all, 0.5);
        let a1_p90 = quantile(&mut aff1_all, 0.9);
        println!(
            "     affection on contacted pairs: v1 p50 {a1_p50:.4} p90 {a1_p90:.4}   \
             v2 p50 {a2_p50:.4} p90 {a2_p90:.4}\n     \
             v1-era high-affection share {:.1}% ↔ same share at v2 gate {q_hi:.4}  
             (current gate {hi:.2})\n",
            100.0 * share
        );
    }
}
