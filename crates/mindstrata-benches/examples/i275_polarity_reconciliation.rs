//! Iter-275 probe — WP-H2: polarity reconciliation reachability (PLAN_DC2 §6.3).
//!
//! Static analysis says the reconciliation pass (live in
//! `system_polarity_claim_emit` since DC-2.1) is *structurally* dead in vivo:
//! `project_catalyst` maps each CatalystKind to ONE fixed (domain, referent,
//! claim, line) quartet, so an agent's claims are mono-subtle per (referent,
//! line) unless TWO catalyst kinds collide there. The only in-vivo collision is
//! Threat+Grief on (Event, cognitive) — and Grief is mortality-blocked at N=12
//! (i272). The action-level ActiveTension bias (actions/mod.rs:845) therefore
//! never fires either.
//!
//! This probe measures the live claim inventory and the tension/reconciliation
//! counters at N=12 / 20K to confirm the dead-producer verdict, then measures
//! what a collision-rich projection WOULD produce (forced GriefStruck window
//! injection) as the upper-bound evidence for the i275 fix.

use mindstrata_development::polarity::PolarityState;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn make(n: u32, seed: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: n,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim
}

fn claim_stats(sim: &Simulation) -> (usize, usize, usize, usize) {
    let (mut undiscovered, mut tension, mut integrated, mut total) = (0, 0, 0, 0);
    for a in &sim.agents {
        for c in &a.polarity_claims {
            total += 1;
            match c.polarity {
                PolarityState::Undiscovered => undiscovered += 1,
                PolarityState::ActiveTension => tension += 1,
                PolarityState::Integrated => integrated += 1,
            }
        }
    }
    (total, undiscovered, tension, integrated)
}

fn main() {
    // Part 1 — natural diet at 20K: confirm the dead-producer verdict.
    let mut sim = make(12, 42);
    sim.run(20_000);
    let (total, u, t, i) = claim_stats(&sim);
    println!("NATURAL N=12 20K  claims: total={total} undiscovered={u} active_tension={t} integrated={i}");
    if t == 0 && i == 0 {
        println!("VERDICT: reconciliation structurally dead in vivo (zero tension, zero integrated)");
    } else {
        println!("VERDICT: reconciliation LIVE in natural runs — static analysis wrong, no fix needed");
    }

    // Part 2 — upper bound: inject GriefStruck windows (Threat's collision
    // partner) and measure what tension+reconciliation would look like if the
    // catalyst projection DID collide at N=12 horizons. Injection via the
    // development pass directly over synthetic events (same shape the deaths
    // pass produces), so no RNG stream is consumed and the natural run stays
    // byte-identical.
    let mut sim2 = make(12, 42);
    // Run 1000 ticks to a natural state, then inject grief windows daily.
    for day in 0..20 {
        sim2.run(200);
        let tick = sim2.current_tick();
        // One GriefStruck per agent per injection round — the collision source.
        let evs: Vec<mindstrata_core::event::SimEvent> = (0..12usize)
            .map(|a| mindstrata_core::event::SimEvent::GriefStruck {
                mourner: mindstrata_core::id::AgentId::new(a as u64),
                deceased: mindstrata_core::id::AgentId::new(((a + 1) % 12) as u64),
                tick,
            })
            .collect();
        mindstrata_sim::systems::development::system_polarity_claim_emit(
            &mut sim2.agents,
            &evs,
        );
        let _ = day;
    }
    let (total2, u2, t2, i2) = claim_stats(&sim2);
    println!(
        "FORCED-GRIEF      claims: total={total2} undiscovered={u2} active_tension={t2} integrated={i2}"
    );
    if t2 > 0 || i2 > 0 {
        println!("UPPER-BOUND: collision-rich diet DOES drive tension→reconciliation (mechanism viable)");
    } else {
        println!("UPPER-BOUND: even collisions fail — inspect advance_to_active_tension gating");
    }
}
