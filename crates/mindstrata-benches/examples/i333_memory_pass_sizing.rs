//! i333 — size the `tick_memory_encoding` O(N·E) pass before touching it.
//!
//! i331 left one recorded term: for every agent, the memory pass iterates
//! *every* event of the tick (`self.events[pre_tick_events..]`), computing
//! `compute_salience` and — with it — a habituation update. That is N·E
//! `compute_salience` calls per tick. This probe measures, at the envelope:
//!
//!   1. E(t), the events-per-tick rate, and how it grows with N;
//!   2. the N·E pair count, and the share of pairs whose event actually
//!      *names* the agent (i.e. the perception-radius-relevant subset);
//!   3. the event-kind mix, so a gate can be judged against the mix.
//!
//! Evidence first: nothing is changed here.
//!
//! Run: cargo run --release -p mindstrata-benches --example i333_memory_pass_sizing

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// Does this event name `a` in a direct perceptual role? Exactly the
/// `from`/`to`/`agent`/`participants` roles `event_relevance` scores at
/// 0.8–0.9 (vs the 0.3 base for everyone else).
fn names(event: &SimEvent, a: u64) -> bool {
    let hit = |id: &mindstrata_core::id::AgentId| id.as_u64() == a;
    match event {
        SimEvent::InteractionOccurred { from, to, .. }
        | SimEvent::RelationshipChanged { from, to, .. } => hit(from) || hit(to),
        SimEvent::AgentAte { agent, .. }
        | SimEvent::AgentDrank { agent, .. }
        | SimEvent::AgentRested { agent, .. }
        | SimEvent::AgentSpawned { agent, .. }
        | SimEvent::AgentDied { agent, .. }
        | SimEvent::GriefStruck { mourner: agent, .. } => hit(agent),
        SimEvent::RitualPerformed { participants, .. }
        | SimEvent::MourningObserved { participants, .. } => participants.iter().any(hit),
        _ => false,
    }
}

fn kind_of(event: &SimEvent) -> &'static str {
    match event {
        SimEvent::InteractionOccurred { .. } => "Interaction",
        SimEvent::RelationshipChanged { .. } => "RelationshipChanged",
        SimEvent::AgentAte { .. } => "Ate",
        SimEvent::AgentDrank { .. } => "Drank",
        SimEvent::AgentRested { .. } => "Rested",
        SimEvent::AgentSpawned { .. } => "Spawned",
        SimEvent::AgentDied { .. } => "Died",
        SimEvent::GriefStruck { .. } => "Grief",
        SimEvent::RitualPerformed { .. } | SimEvent::MourningObserved { .. } => "Ritual",
        _ => "Other",
    }
}

fn run(n: u32, warmup: u64, window: u64) -> Row {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: warmup + window,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(warmup);

    let mut pairs = 0u64;
    let mut relevant = 0u64;
    let mut events = 0u64;
    let mut ticks = 0u64;
    let mut kinds: std::collections::BTreeMap<&'static str, u64> =
        std::collections::BTreeMap::new();

    for _ in 0..window {
        let before = sim.event_count();
        sim.run(1);
        let after = sim.event_count();
        let delta = (after - before) as usize;
        let recent = sim.recent_events(delta);
        events += delta as u64;
        ticks += 1;
        pairs += (delta as u64) * (sim.agent_count() as u64);
        for ev in recent {
            *kinds.entry(kind_of(ev)).or_default() += 1;
            for a in 0..sim.agent_count() as u64 {
                if names(ev, a) {
                    relevant += 1;
                }
            }
        }
    }

    Row {
        n: sim.agent_count(),
        events_per_tick: events as f64 / ticks as f64,
        pairs_per_tick: pairs as f64 / ticks as f64,
        relevant_share: relevant as f64 / pairs.max(1) as f64,
        kinds,
    }
}

struct Row {
    n: usize,
    events_per_tick: f64,
    pairs_per_tick: f64,
    relevant_share: f64,
    kinds: std::collections::BTreeMap<&'static str, u64>,
}

fn main() {
    const WARMUP: u64 = 400;
    const WINDOW: u64 = 40;
    println!("i333 — tick_memory_encoding pass sizing (seed 42, 32x32, warmup {WARMUP}, window {WINDOW} ticks)\n");
    println!(
        "{:>5} {:>14} {:>18} {:>14} {:>16}",
        "N", "events/tick", "pairs/tick", "relevant", "pairs/agent"
    );
    let mut rows = Vec::new();
    for n in [12u32, 48, 96, 192] {
        let r = run(n, WARMUP, WINDOW);
        println!(
            "{:>5} {:>14.1} {:>18.0} {:>13.1}% {:>16.1}",
            r.n,
            r.events_per_tick,
            r.pairs_per_tick,
            r.relevant_share * 100.0,
            r.pairs_per_tick / r.n as f64
        );
        rows.push(r);
    }

    // Local exponent of events/tick and of pairs/tick.
    println!("\nlocal exponents (log-log between consecutive N):");
    for w in rows.windows(2) {
        let (a, b) = (&w[0], &w[1]);
        let e = (b.events_per_tick / a.events_per_tick).ln() / (b.n as f64 / a.n as f64).ln();
        let p = (b.pairs_per_tick / a.pairs_per_tick).ln() / (b.n as f64 / a.n as f64).ln();
        println!(
            "  {:>4} -> {:<4}  events alpha {:.3}   pairs alpha {:.3}",
            a.n, b.n, e, p
        );
    }

    println!("\nevent-kind mix at N={}:", rows.last().unwrap().n);
    let total: u64 = rows.last().unwrap().kinds.values().sum();
    for (k, v) in &rows.last().unwrap().kinds {
        println!(
            "  {:<20} {:>10}  {:>5.1}%",
            k,
            v,
            *v as f64 / total as f64 * 100.0
        );
    }

    // Verdict bands: is the pass N^2 (floor, nothing to gate) or N^3 (a gate wins)?
    let top = rows.last().unwrap();
    let alpha_pairs = (top.pairs_per_tick / rows[rows.len() - 2].pairs_per_tick).ln()
        / (top.n as f64 / rows[rows.len() - 2].n as f64).ln();
    println!("\n--- verdict ---");
    println!(
        "  top-of-envelope pairs alpha      {:.3}  ({})",
        alpha_pairs,
        if alpha_pairs > 2.4 {
            "SUPERQUADRATIC — a perception gate removes a real term"
        } else {
            "AT FLOOR — the pass is N^2; gating saves a constant, not an exponent"
        }
    );
    println!(
        "  relevant share (gate ceiling)    {:.1}%  -> a perfect gate removes {:.1}% of the pass",
        top.relevant_share * 100.0,
        (1.0 - top.relevant_share) * 100.0
    );
}
