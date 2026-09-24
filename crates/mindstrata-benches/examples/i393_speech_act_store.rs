//! i393 — the dual-store writer: where the two gain schedules disagree, and
//! whether the v1 one can be imported onto the dyadic store.
//!
//! The v1→v2 migration is down to its last structural item. i384 moved the one
//! self-contained v1 read+write pair (the §13.3 trade price) and left the
//! **general writer** standing: `process_interaction` applies a per-kind gain to
//! the legacy `Relationship` row (Talk 0.01/0.005, Help 0.05/0.03, …) *and* the
//! pass applies a `magnitude` (0.2–0.6 by kind) to the dyadic row through
//! `record_positive`, whose effective trust gain is
//! `magnitude × volatility × 0.02`. Two schedules, one act.
//!
//! * **A — the sync signature.** Reading each store's per-act gain out of
//!   `Δtrust / Δacts` does not work since i376: the daily pass pulls v1.trust
//!   onto the dyadic value at coupling 1.0, so a v1 row's realised movement is
//!   dominated by the sync rather than by its own gain (the first draft's
//!   negative v1 medians were exactly that, and at N=12 every sampled pair sat
//!   at the clamp). The measurable question is *when* the stores disagree:
//!   on a daily boundary (`tick % 144 == 0`), or rebuilt within the day? And
//!   **affection** is measured alongside trust because the i376 sync touches
//!   `rel.trust` ONLY — if affection diverges without bound, the migration
//!   cannot simply delete the v1 gain and leave the row to the sync.
//! * **B — the divergence it has already produced** across all directed pairs,
//!   the share beyond 0.05, and the saturation share (≥ 0.95) in each store.
//! * **C — the decisive leg: regime projection.** For every directed pair, take
//!   the acts actually performed in the window, by kind, and project the pair's
//!   dyadic trust under each candidate end-state, gain-only (an upper bound:
//!   decay only lowers it):
//!
//!   1. **delete-only** — the v1 gain is removed, the dyadic schedule stays;
//!   2. **import-v1** — the v1 per-kind deltas become the dyadic schedule;
//!   3. **import-effect** — the credibility-scaled `resolve_effect` deltas
//!      become the dyadic schedule.
//!
//!   The saturation share is the discriminator. i376 established that the v1
//!   schedule saturates its own store (mean 0.887–0.920, 79–85% of pairs ≥0.95
//!   at 50K) because its 0.001/day reversion loses to the gains by ~14×.
//!   Importing that schedule onto a store that decays 0.0002/tick cannot
//!   un-saturate it, so if regimes 2/3 saturate, the migration's correct shape
//!   is regime 1.
//! * **D — the blast radius** is a source census, not a simulation reading:
//!   recorded in `i393_speech_act_store.md` §4.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i393_speech_act_store`

use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

const WINDOW: u64 = 1_000;
const DAY: u64 = 144;

const KINDS: [InteractionKind; 8] = [
    InteractionKind::Talk,
    InteractionKind::Help,
    InteractionKind::Threaten,
    InteractionKind::Trade,
    InteractionKind::Gossip,
    InteractionKind::Comfort,
    InteractionKind::Insult,
    InteractionKind::Teach,
];

/// The v1 schedule, as `process_interaction` holds it today (trust, affection).
fn v1_delta(kind: InteractionKind) -> (f64, f64) {
    match kind {
        InteractionKind::Talk => (0.01, 0.005),
        InteractionKind::Help => (0.05, 0.03),
        InteractionKind::Threaten => (-0.1, -0.05),
        InteractionKind::Trade => (0.02, 0.0),
        InteractionKind::Gossip => (0.005, 0.01),
        InteractionKind::Comfort => (0.03, 0.05),
        InteractionKind::Insult => (-0.08, -0.1),
        InteractionKind::Teach => (0.04, 0.02),
    }
}

/// The dyadic schedule, as `pass_social.rs` holds it today.
fn v2_magnitude(kind: InteractionKind) -> f64 {
    match kind {
        InteractionKind::Help | InteractionKind::Comfort => 0.6,
        InteractionKind::Threaten | InteractionKind::Insult => 0.5,
        InteractionKind::Gossip | InteractionKind::Talk => 0.3,
        _ => 0.2,
    }
}

/// `resolve_effect`'s credibility-scaled trust delta at the calibrated neutral
/// credibility the pass supplies in calm worlds.
fn effect_delta(kind: InteractionKind) -> f64 {
    let (base, _) = v1_delta(kind);
    base * (0.5 + 0.5 * 0.5)
}

/// Dyadic position of target `b` inside `a`'s `relationship_v2s` (the sim's own
/// `relationship_v2_pos`: `a` skips itself, so `b` sits at `b` or `b − 1`).
fn v2_pos(a: usize, b: usize) -> usize {
    if b > a {
        b - 1
    } else {
        b
    }
}

#[derive(Clone)]
struct Row {
    v1_trust: f64,
    v1_acts: u32,
    v1_aff: f64,
    v2_trust: f64,
    v2_acts: u32,
    v2_aff: f64,
    vol: f64,
}

fn snapshot(sim: &Simulation) -> Vec<Row> {
    let n = sim.agents.len();
    let mut rows = vec![
        Row {
            v1_trust: 0.0,
            v1_acts: 0,
            v1_aff: 0.0,
            v2_trust: 0.0,
            v2_acts: 0,
            v2_aff: 0.0,
            vol: 0.5,
        };
        n * n
    ];
    for from in 0..n {
        for to in 0..n {
            let idx = from * n + to;
            if let Some(rel) = sim
                .relationships
                .iter()
                .find(|r| r.from.as_u64() as usize == from && r.to.as_u64() as usize == to)
            {
                rows[idx].v1_trust = rel.trust.to_f64();
                rows[idx].v1_acts = rel.interaction_count;
                rows[idx].v1_aff = rel.affection.to_f64();
            }
            if from != to {
                if let Some(v2) = sim.agents[from].relationship_v2s.get(v2_pos(from, to)) {
                    if v2.to.as_u64() as usize == to {
                        rows[idx].v2_trust = v2.trust.to_f64();
                        rows[idx].v2_acts = v2.interaction_count;
                        rows[idx].v2_aff = v2.affection.to_f64();
                        rows[idx].vol = v2.volatility.to_f64();
                    }
                }
            }
        }
    }
    rows
}

/// (trust mean, trust max, affection mean, affection max) of `|v1 − v2|` over
/// every v1 row that has a validated dyadic counterpart.
fn divergence(sim: &Simulation) -> (f64, f64, f64, f64) {
    let n = sim.agents.len();
    let mut t = Vec::new();
    let mut f = Vec::new();
    for rel in &sim.relationships {
        let fi = rel.from.as_u64() as usize;
        let ti = rel.to.as_u64() as usize;
        if fi >= n || ti >= n || fi == ti {
            continue;
        }
        if let Some(v2) = sim.agents[fi].relationship_v2s.get(v2_pos(fi, ti)) {
            if v2.to.as_u64() as usize == ti {
                t.push((rel.trust - v2.trust).abs().to_f64());
                f.push((rel.affection - v2.affection).abs().to_f64());
            }
        }
    }
    (
        mean(&t),
        t.iter().copied().fold(0.0, f64::max),
        mean(&f),
        f.iter().copied().fold(0.0, f64::max),
    )
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
        0.0
    } else {
        xs.iter().sum::<f64>() / xs.len() as f64
    }
}

fn main() {
    println!("i393 — the dual-store writer (v1 gain schedule vs the dyadic store)\n");
    let bonding = Simulation::new(SimConfig::default())
        .params
        .bonding_rate
        .to_f64();
    println!("bonding_rate = {bonding:.4}, window = {WINDOW} ticks, day = {DAY}\n");

    for (n, horizon) in [(12u32, 20_000u64), (48, 20_000), (48, 50_000)] {
        println!("══ N={n}, seed 42, horizon {horizon} ══");

        // ── A — the sync signature ────────────────────────────────────────
        let bound = (horizon / DAY) * DAY;
        let mut day = village(n, 42, bound);
        day.run(bound);
        let (t_b, tx_b, a_b, ax_b) = divergence(&day);
        day.run(DAY / 2);
        let (t_m, tx_m, a_m, ax_m) = divergence(&day);
        println!(
            "  A  |v1−v2| on the boundary (tick {bound}):  trust {t_b:.4} / max {tx_b:.4}   affection {a_b:.4} / max {ax_b:.4}"
        );
        println!(
            "     |v1−v2| mid-day (+{})            :  trust {t_m:.4} / max {tx_m:.4}   affection {a_m:.4} / max {ax_m:.4}",
            DAY / 2
        );

        // ── B/C — on the window ending at `horizon` ───────────────────────
        let t0 = horizon - WINDOW;
        let mut sim = village(n, 42, horizon);
        sim.run(t0);
        let _ = snapshot(&sim);
        sim.run(WINDOW);
        let b = snapshot(&sim);
        let events: Vec<SimEvent> = sim.recent_events(10_000_000).to_vec();

        // ── B — the divergence already produced ───────────────────────────
        let mut div: Vec<f64> = Vec::new();
        let (mut sat1, mut sat2, mut contacted) = (0usize, 0usize, 0usize);
        for row in &b {
            if row.v1_acts == 0 && row.v2_acts == 0 {
                continue;
            }
            contacted += 1;
            div.push((row.v1_trust - row.v2_trust).abs());
            if row.v1_trust >= 0.95 {
                sat1 += 1;
            }
            if row.v2_trust >= 0.95 {
                sat2 += 1;
            }
        }
        let over = div.iter().filter(|d| **d > 0.05).count();
        println!(
            "  B  contacted pairs {contacted}   |v1−v2| mean {:.4} max {:.4}   >0.05: {over} ({:.0}%)",
            mean(&div),
            div.iter().copied().fold(0.0, f64::max),
            100.0 * over as f64 / div.len().max(1) as f64
        );
        println!(
            "     saturation ≥0.95:  v1 {sat1}/{contacted} ({:.0}%)   v2 {sat2}/{contacted} ({:.0}%)",
            100.0 * sat1 as f64 / contacted.max(1) as f64,
            100.0 * sat2 as f64 / contacted.max(1) as f64
        );

        // ── C — regime projection (gain-only upper bound) ──────────────────
        let mut acts = vec![[0u32; 8]; b.len()];
        for ev in &events {
            if let SimEvent::InteractionOccurred { from, to, kind, .. } = ev {
                let (fi, ti) = (from.as_u64() as usize, to.as_u64() as usize);
                if fi >= n as usize || ti >= n as usize {
                    continue;
                }
                if let Some(k) = KINDS.iter().position(|kk| kk == kind) {
                    acts[fi * n as usize + ti][k] += 1;
                }
            }
        }
        let names = [
            "1 delete-only   (v2 schedule kept) ",
            "2 import-v1     (v1 deltas → v2)   ",
            "3 import-effect (resolve_effect→v2)",
        ];
        let mut proj: [Vec<(f64, f64, f64)>; 3] = [Vec::new(), Vec::new(), Vec::new()];
        for (i, row) in b.iter().enumerate() {
            if row.v1_acts == 0 {
                continue;
            }
            let clamp01 = |x: f64| x.clamp(0.0, 1.0);
            let d = |f: &dyn Fn(InteractionKind) -> f64| -> f64 {
                acts[i]
                    .iter()
                    .enumerate()
                    .map(|(k, c)| f64::from(*c) * f(KINDS[k]))
                    .sum()
            };
            let d_v2 = d(&|k| v2_magnitude(k) * row.vol * 0.02);
            let d_v1 = d(&|k| v1_delta(k).0 * bonding);
            let d_eff = d(&effect_delta);
            proj[0].push((row.v2_trust, clamp01(row.v2_trust + d_v2), d_v2));
            proj[1].push((row.v2_trust, clamp01(row.v2_trust + d_v1), d_v1));
            proj[2].push((row.v2_trust, clamp01(row.v2_trust + d_eff), d_eff));
        }
        println!("  C  regime projection over a {WINDOW}-tick window (gain-only, clamped):");
        for (idx, name) in names.iter().enumerate() {
            let v = &proj[idx];
            let before = mean(&v.iter().map(|x| x.0).collect::<Vec<_>>());
            let after = mean(&v.iter().map(|x| x.1).collect::<Vec<_>>());
            let sat = v.iter().filter(|x| x.1 >= 0.95).count();
            println!(
                "     {name}  mean {before:.4} → {after:.4} (Δ {:+.4})   ≥0.95 {sat}/{} ({:.0}%)",
                mean(&v.iter().map(|x| x.2).collect::<Vec<_>>()),
                v.len(),
                100.0 * sat as f64 / v.len().max(1) as f64
            );
        }
        let mut by_kind = [0usize; 8];
        for ev in &events {
            if let SimEvent::InteractionOccurred { kind, .. } = ev {
                if let Some(k) = KINDS.iter().position(|kk| kk == kind) {
                    by_kind[k] += 1;
                }
            }
        }
        print!("     act mix:");
        for (k, kind) in KINDS.iter().enumerate() {
            print!("  {kind:?} {}", by_kind[k]);
        }
        println!("\n");
    }

    println!(
        "reading: the v1 schedule is ~an order of magnitude larger per act than the dyadic one, \
         so importing it (regimes 2/3) drives the dyadic store toward the saturation i376 \
         diagnosed on v1 — which is the evidence for the migration's shape being regime 1: \
         delete the v1 gain, keep the dyadic schedule, and re-point the speech-act model's \
         `base_delta` documentation at the schedule that is actually applied."
    );
}
