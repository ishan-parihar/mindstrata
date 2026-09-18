//! i288 — in-vivo verdict probe: the Transgression feed after the
//! NormViolated emission fix.
//!
//! Pre-fix (i280 sweep): zero Transgression catalysts in every regime —
//! the only NormViolated emitter (caught theft) is structurally unreachable
//! at the seeded topology (all stocks `AccessRight::Public`, so
//! `inaccessible_farm_with_grain_amount` can never match).
//! Post-fix expectation: Transgression catalysts ≈ violence count
//! (public-by-nature semantics, no detection roll per Iter-88).
//!
//! Census at the pinned horizons (blast surface) and the 20K working
//! horizon (feed liveness), plus the downstream counters: Q2 real-pressure
//! ticks, justice-line claims, and norm-proposal reachability inputs.

use mindstrata_core::event::SimEvent;
use mindstrata_sim::sim::{SimConfig, Simulation};

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    }
}

fn census(seed: u64, ticks: u64) {
    let mut sim = Simulation::new(config(seed, ticks));
    sim.populate();
    sim.run(ticks);
    let events = sim.recent_events(sim.event_count() as usize);
    let norm_violated = events
        .iter()
        .filter(|e| matches!(e, SimEvent::NormViolated { .. }))
        .count();
    let violence = events
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Violence,
                    ..
                }
            )
        })
        .count();
    let justice_claims: usize = sim
        .agents
        .iter()
        .map(|a| {
            a.polarity_claims
                .iter()
                .filter(|c| c.line.slug() == "justice")
                .count()
        })
        .sum();
    let q2_mean: f64 = sim
        .agents
        .iter()
        .map(|a| a.development.pathology.dark_allergy.intensity)
        .sum::<f64>()
        / sim.agents.len() as f64;
    let proposals = sim
        .norms
        .norms()
        .iter()
        .filter(|n| n.name.starts_with("[proposed:"))
        .count();
    println!(
        "seed={seed} t={ticks:>6}: NormViolated={norm_violated} violence={violence} \
         justice_claims={justice_claims} Q2_mean={q2_mean:.4} proposed_norms={proposals}"
    );
}

fn main() {
    println!("== i288 in-vivo verdict: Transgression feed post-fix (N=12) ==");
    println!("-- pinned horizons (calm, seed 42) --");
    for h in [500_u64, 1_000, 2_000, 4_320, 10_000] {
        census(42, h);
    }
    println!("-- working horizon 20K, multi-seed --");
    for seed in [42_u64, 1, 7] {
        census(seed, 20_000);
    }
}
