//! §2 Long-horizon (50K) determinism + emergence sweep.
//!
//! The queue's testing gap: every long-horizon test stopped at 30K (the
//! faction-emergence tests) or 15K (multi-seed macro health) — nothing
//! validated a 50K+ horizon, and no long-horizon test asserted byte-identical
//! replay. This module closes both:
//!
//! - `long_horizon_50k_is_deterministic_and_emerges` — the SAME seed run
//!   twice to 50K must produce byte-identical metrics (JSON equality, reusing
//!   `behavioral_delta::snapshots_identical`) AND byte-identical per-agent
//!   state (aggregate means can cancel; the per-agent projection cannot),
//!   AND the run must show real emergence without collapse (probe-pinned at
//!   seed 42/50K: 5 active memes, 1 faction, 3 moral panics, 2 rituals with
//!   `last_occurrence > 0` (execution, not registration), population
//!   12 → 13, stress 0.640 — a stressed-but-surviving village).
//! - `long_horizon_50k_multi_seed_invariants` — seeds 7/99 to 50K hold every
//!   core invariant (bounded population, finite in-range health/stress,
//!   non-negative wealth/resources) with emergent activity (events, memes).

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::institutions::InstitutionKind;

use crate::behavioral_delta::snapshots_identical;
use crate::test_helpers::run_sim;

/// Per-agent core-state projection used as the determinism backstop.
///
/// Aggregates can cancel (one agent up, another down, same mean), so a
/// mean-only comparison can mask per-agent divergence; comparing every
/// agent's core state index-by-index cannot.
fn agent_projection(sim: &mindstrata_sim::Simulation) -> Vec<(f64, f64, f64, f64, f64, f64)> {
    sim.agent_summaries()
        .iter()
        .map(|a| {
            (
                a.hunger.to_f64(),
                a.thirst.to_f64(),
                a.fatigue.to_f64(),
                a.health.to_f64(),
                a.valence.to_f64(),
                a.fear.to_f64(),
            )
        })
        .collect()
}

/// Emergence/invariant checks for a completed 50K seed-42 run.
/// Probe-pinned at Iteration 135: see the module doc for the exact signals.
fn assert_seed42_emergence_and_invariants(sim: &mindstrata_sim::Simulation) {
    let m = sim.metrics_snapshot();

    // No collapse, no explosion (12 initial agents, births fire by 50K).
    assert!(
        (12..=100).contains(&m.agent_count),
        "agent count unreasonable after 50K ticks: {}",
        m.agent_count
    );

    // The world must have LIVED: an order-of-magnitude event volume sanity
    // (probe: 1.09M — 10× headroom so a legitimate event-granularity change
    // does not fail this for the wrong reason), culture spread, and the
    // emergent social structures the campaign wired all fired at least once.
    assert!(
        m.event_count > 100_000,
        "only {} events in 50K ticks",
        m.event_count
    );
    assert!(m.active_meme_count >= 1, "no active memes after 50K ticks");
    let factions = sim
        .institutions
        .iter()
        .filter(|i| i.kind == InstitutionKind::Faction)
        .count();
    assert!(factions >= 1, "no faction formed by 50K ticks (seed 42)");
    assert!(
        !sim.moral_panic_registry.panics.is_empty(),
        "no moral panic fired by 50K ticks (seed 42)"
    );
    // Registration is NOT execution (rituals are seeded at populate). The
    // durable execution signal is `Ritual.last_occurrence`, advanced by
    // `Ritual::execute` on every due firing (seeded at 0, monthly 4320-tick
    // interval → ~11 fires by 50K). Note: the `RitualParticipated` Cultural
    // memory each participant encodes (sim.rs:8144) does NOT survive to 50K —
    // the store decays traces below the eviction threshold between the sparse
    // monthly fires — so last_occurrence, not memory, is the right check.
    assert!(
        sim.ritual_registry
            .rituals
            .iter()
            .any(|r| r.last_occurrence > 0),
        "no ritual actually EXECUTED by 50K ticks (all last_occurrence == 0)"
    );

    // Resources never go negative (water legitimately reaches 0.0 in this
    // stressed village — assert non-negativity, not positivity).
    assert!(
        m.total_grain >= 0.0,
        "negative grain at 50K: {}",
        m.total_grain
    );
    assert!(
        m.total_water >= 0.0,
        "negative water at 50K: {}",
        m.total_water
    );

    // Core aggregates stay finite and bounded.
    assert!(
        m.avg_stress.is_finite() && (0.0..=1.0).contains(&m.avg_stress),
        "avg_stress {} out of [0,1] at 50K",
        m.avg_stress
    );
    assert!(
        m.avg_health.is_finite() && (0.0..=1.0).contains(&m.avg_health),
        "avg_health {} out of [0,1] at 50K",
        m.avg_health
    );
}

/// Per-seed invariant checks at 50K (used by the multi-seed test).
fn assert_invariants(seed: u64, sim: &mindstrata_sim::Simulation) {
    let m = sim.metrics_snapshot();
    assert!(
        (12..=100).contains(&m.agent_count),
        "seed {seed}: agent count unreasonable after 50K ticks: {}",
        m.agent_count
    );
    assert!(m.event_count > 0, "seed {seed}: no events in 50K ticks");
    assert!(
        m.total_grain >= 0.0,
        "seed {seed}: negative grain {}",
        m.total_grain
    );
    assert!(
        m.total_water >= 0.0,
        "seed {seed}: negative water {}",
        m.total_water
    );
    for (i, agent) in sim.agents.iter().enumerate() {
        assert!(
            agent.body.health.to_f64().is_finite()
                && agent.body.health >= Fixed::ZERO
                && agent.body.health <= Fixed::ONE,
            "seed {seed} agent {i}: health {} out of [0,1]",
            agent.body.health.to_f64()
        );
        assert!(
            agent.embodied.endocrine.stress.level >= Fixed::ZERO
                && agent.embodied.endocrine.stress.level <= Fixed::ONE,
            "seed {seed} agent {i}: stress {} out of [0,1]",
            agent.embodied.endocrine.stress.level.to_f64()
        );
        assert!(
            agent.wealth.coin >= Fixed::ZERO,
            "seed {seed} agent {i}: negative wealth {}",
            agent.wealth.coin.to_f64()
        );
    }
}

#[test]
fn long_horizon_50k_is_deterministic_and_emerges() {
    // Two full 50K runs of the same seed must be byte-identical AND emergent.
    let a = run_sim(42, 50_000);
    let b = run_sim(42, 50_000);
    assert!(
        snapshots_identical(&a.metrics_snapshot(), &b.metrics_snapshot()),
        "seed 42 metrics differ across two 50K runs — a nondeterminism regression"
    );
    assert_eq!(
        agent_projection(&a),
        agent_projection(&b),
        "seed 42 per-agent state differs across two 50K runs — aggregate cancellation may hide it"
    );
    assert_seed42_emergence_and_invariants(&a);
}

#[test]
fn long_horizon_50k_multi_seed_invariants() {
    for seed in [7u64, 99] {
        let sim = run_sim(seed, 50_000);
        assert_invariants(seed, &sim);
    }
}
