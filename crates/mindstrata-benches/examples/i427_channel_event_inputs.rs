//! i427 — per-channel gate inputs on the channels' OWN event streams
//! (§4.12: measure at the predicate's own scope, not a contacted-graph proxy).
//!
//! i426 established the deficit is formation-time — but its gates were
//! evaluated on the contacted GRAPH (any pair with v2 interaction_count > 0).
//! The distributional bars the re-attempt needs must be sized on the pairs
//! each channel actually sees: the event stream. This probe drains every
//! InteractionOccurred event per tick (journal_len/watchdog delta over the
//! bounded ring, loss counted) and records, at that tick:
//!
//! * v1 trust (the dense spawn matrix, `U(0.3,0.7)` + dynamics), and
//! * v2 trust (the honest dyadic store),
//!
//! for the interacting ordered pair — binned by interaction kind × surface ×
//! epoch. All three i426 channel gates reduce to trust-column comparisons
//! (measured facts from the source, not assumptions):
//!
//! * **Diffusion** acceptance `trust×0.5 + openness×0.5 ≥ 0.5` — and since
//!   production `cultural.openness ≡ 0.5` (i426), this is exactly `trust > 0.5`
//!   at runtime. It fires on Talk/Help/Trade events (site §19.5.I).
//! * **ToM** Friendliness ⟺ kind ∈ {Help, Comfort} AND trust > 0.5
//!   (the call site passes per-event `observed_helpful` = 0.6/0.3/0.1).
//! * **Gossip** has NO trust gate (acceptance = rumor salience > threshold);
//!   trust enters only as a ±0.08 fidelity discount (`0.9 + 0.08·trust`,
//!   gossip.rs:130), so a surface shift of Δt ≤ 0.10 moves delivered belief
//!   strength by ≤ 0.008 — reported but not an acceptance bar.
//!
//! Verdict feeds the i403 §6 re-derivation: the early-epoch v1-vs-v2 event
//! distributions show exactly what a distributional bar must admit (and what
//! it must not), sized by the i381 gap method.
//!
//! Run: cargo run --release -p mindstrata-benches --example i427_channel_event_inputs
use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;
use std::collections::HashMap;

const TICKS: u64 = 20_000;
const EPOCH_BOUNDARY: u64 = 5_000;

/// Trust histogram bins over [0, 1).
const BINS: usize = 100;

/// (channel class, surface, epoch) — one histogram cell.
struct Cell {
    /// Trust histogram per affinity bucket.
    hist: [u64; BINS],
    /// Event count.
    n: u64,
    /// Sum of trusts (for the mean).
    sum: f64,
}

impl Cell {
    fn add(&mut self, t: f64) {
        let b = ((t.clamp(0.0, 0.999999) * BINS as f64) as usize).min(BINS - 1);
        self.hist[b] += 1;
        self.n += 1;
        self.sum += t;
    }

    fn median(&self) -> f64 {
        let half = self.n / 2;
        let mut acc = 0u64;
        for (i, &c) in self.hist.iter().enumerate() {
            acc += c;
            if acc > half {
                return (i as f64 + 0.5) / BINS as f64;
            }
        }
        0.0
    }

    fn frac_gt(&self, bar: f64) -> f64 {
        let from = (bar * BINS as f64).ceil() as usize;
        let above: u64 = self.hist[from.min(BINS)..].iter().sum();
        above as f64 / self.n.max(1) as f64
    }
}

/// Channel-relevant kind class.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum KClass {
    /// Talk / Trade (and Help, which also feeds WarmContact) — the
    /// diffusion channel's trigger kinds (social_cluster.rs:880-882).
    Diffusion,
    /// Gossip — belief relay (fidelity-discount only, no trust bar).
    Gossip,
    /// Help / Comfort — ToM Friendly-eligible kinds
    /// (per-event `observed_helpful` = 0.6).
    WarmContact,
    /// Everything else (Insult, Threaten, Teach).
    Other,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            hist: [0; BINS],
            n: 0,
            sum: 0.0,
        }
    }
}

/// Every channel class an event participates in (Help flows to BOTH the
/// diffusion and ToM channels — measured once per channel it feeds).
fn class_of(kind: InteractionKind) -> (KClass, Option<KClass>) {
    match kind {
        InteractionKind::Talk | InteractionKind::Trade => (KClass::Diffusion, None),
        InteractionKind::Help => (KClass::Diffusion, Some(KClass::WarmContact)),
        InteractionKind::Gossip => (KClass::Gossip, None),
        InteractionKind::Comfort => (KClass::WarmContact, None),
        _ => (KClass::Other, None),
    }
}

fn main() {
    println!("i427 channel event-stream inputs (pestilence, {TICKS} ticks; epochs split at {EPOCH_BOUNDARY}):");
    for seed in [5u64, 42, 12345] {
        let mut sc = Scenario::pestilence();
        sc.seed = seed;
        sc.ticks = TICKS;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();

        // cells[(class, surface 0=v1/1=v2, epoch 0=early/1=late)]
        let mut cells: HashMap<(KClass, usize, usize), Cell> = HashMap::new();
        let mut v1_idx: HashMap<(u64, u64), usize> = HashMap::new();
        let mut lost_events = 0u64;

        for t in 0..TICKS {
            let before = sim.event_count();
            sim.tick();
            let after = sim.event_count();
            let ring = sim.recent_events(usize::MAX);
            let delta = after - before;
            if delta > ring.len() {
                lost_events += delta as u64 - ring.len() as u64;
            }
            // v1 index (rows are (from,to)-keyed; dense from populate).
            v1_idx.clear();
            for (p, r) in sim.relationships().iter().enumerate() {
                v1_idx.entry((r.from.as_u64(), r.to.as_u64())).or_insert(p);
            }
            let rels = sim.relationships();
            let epoch = if t < EPOCH_BOUNDARY { 0 } else { 1 };
            let tick_start = ring.len().saturating_sub(delta);
            for ev in &ring[tick_start..] {
                if let SimEvent::InteractionOccurred { from, to, kind, .. } = ev {
                    let (i, j) = (from.as_u64() as usize, to.as_u64() as usize);
                    let v2_t = sim
                        .relationship_v2_between(i, j)
                        .map_or(0.5, |r| r.trust.to_f64());
                    let v1_t = v1_idx
                        .get(&(i as u64, j as u64))
                        .map_or(0.5, |&p| rels[p].trust.to_f64());
                    let (c0, c1) = class_of(*kind);
                    cells.entry((c0, 0, epoch)).or_default().add(v1_t);
                    cells.entry((c0, 1, epoch)).or_default().add(v2_t);
                    if let Some(c1) = c1 {
                        cells.entry((c1, 0, epoch)).or_default().add(v1_t);
                        cells.entry((c1, 1, epoch)).or_default().add(v2_t);
                    }
                }
            }
        }

        println!("seed {seed} (ring loss {lost_events}):");
        println!("  class  epoch |  v1: n mean p50 >50% | v2: n mean p50 >50%");
        for cls in [
            KClass::Diffusion,
            KClass::Gossip,
            KClass::WarmContact,
            KClass::Other,
        ] {
            for epoch in 0..2 {
                let v1 = cells.get(&(cls, 0, epoch));
                let v2 = cells.get(&(cls, 1, epoch));
                let (n1, m1, p1, g1) = v1.map_or((0, 0.0, 0.0, 0.0), |c| {
                    (c.n, c.sum / c.n.max(1) as f64, c.median(), c.frac_gt(0.5))
                });
                let (n2, m2, p2, g2) = v2.map_or((0, 0.0, 0.0, 0.0), |c| {
                    (c.n, c.sum / c.n.max(1) as f64, c.median(), c.frac_gt(0.5))
                });
                println!(
                    "  {cls:?} e{epoch} | {n1:>5} {m1:.4} {p1:.4} {:>5.1}% | {n2:>5} {m2:.4} {p2:.4} {:>5.1}%",
                    g1 * 100.0,
                    g2 * 100.0
                );
            }
        }
    }
}
