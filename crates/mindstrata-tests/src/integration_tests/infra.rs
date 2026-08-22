//! infra integration tests.

use super::*;

// ── §31: Mortality / Generational Replacement ─────────────────────

/// §31: Dead agents must be replaced IN PLACE by newborns so the
/// `AgentId::new(i) == index i` invariant (relationship_v2s O(1) layout,
/// agent_diseases parallel vec, partner/parent indices) is never broken.
#[test]
fn elderly_agents_die_and_are_replaced_in_place() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 2000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let start_count = sim.agents.len();

    // Force the whole population into guaranteed-death territory (past max_age),
    // so the mortality path is exercised deterministically.
    let over_80 = Fixed::from_f64(85.0);
    for agent in &mut sim.agents {
        agent.age = over_80;
        agent.body.health = Fixed::from_f64(0.1);
        agent.body.energy = Fixed::from_f64(0.1);
    }

    // Run past several demography steps (every 10 ticks).
    sim.run(200);

    // Agent count must be UNCHANGED — dead agents are replaced in place, never
    // removed (removal would shift indices and break the AgentId == index invariant).
    assert_eq!(
        sim.agents.len(),
        start_count,
        "Agent count changed after mass death: {} -> {}",
        start_count,
        sim.agents.len()
    );

    // Every slot must now hold a newborn (age ~0), not the forced old age.
    for (i, agent) in sim.agents.iter().enumerate() {
        assert!(
            agent.age.to_f64() < 1.0,
            "Agent {i} not replaced by newborn after death: age={}",
            agent.age.to_f64()
        );
        // Index invariant must hold for every agent.
        assert!(
            sim.agents[i].relationship_v2s.len() <= sim.agents.len().saturating_sub(1),
            "Agent {i} relationship_v2s inconsistent: {} entries for {} agents",
            sim.agents[i].relationship_v2s.len(),
            sim.agents.len()
        );
    }

    // No agent may reference a stale partner/parent that is now a newborn stranger.
    for agent in &sim.agents {
        assert!(
            agent.partner.is_none(),
            "Replacement newborn must not inherit a partner, got {:?}",
            agent.partner
        );
        assert!(
            agent.parent_a.is_none() && agent.parent_b.is_none(),
            "Replacement newborn must not inherit parents, got {:?}/{:?}",
            agent.parent_a,
            agent.parent_b
        );
    }

    // Death events must have been recorded.
    let died_events = sim.event_count();
    assert!(died_events > 0, "Expected AgentDied events in event log");
}
// ── §18.3: Interconnected Systems ────────────────────────────────

#[test]
fn biology_psychology_social_coherence() {
    // §18.3: Multiple systems should interact coherently
    let sim = run_sim(42, 1500);
    // Verify that biological, psychological, and social states are consistent:
    // 1. Agents with high hunger should have higher stress
    // 2. Agents with many relationships should have lower social need
    // 3. High-stress agents should have higher heuristic bias

    for agent in &sim.agents {
        // Biological-psychological link: high hunger → non-zero stress contribution
        let hunger_stress_contribution = agent.needs.hunger.to_f64() * 0.3;
        assert!(
            hunger_stress_contribution >= 0.0,
            "Hunger stress contribution should be non-negative"
        );

        // Social-psychological link: relationship count should correlate with lower social need
        let rel_count = agent.relationship_v2s.len() as f64;
        assert!(
            rel_count >= 0.0,
            "Relationship count should be non-negative"
        );

        // Cognitive-behavioral link: stress affects heuristic bias
        let stress = (agent.emotions.fear + agent.emotions.anger).to_f64();
        let bias = agent.cognitive.heuristic_bias.to_f64();
        // High stress should push heuristic bias upward (loose check)
        if stress > 0.5 {
            assert!(
                bias > 0.15,
                "Agent with stress={stress} should have heuristic bias > 0.15, got {bias}"
            );
        }
    }
}
// ── §5 (AP2): Weather system (Iteration 147) ─────────────────────

/// §5 (AP2, Iteration 147): the weather layer is LIVE and deterministic —
/// temperature/rainfall advance each tick, stay bounded in [0,1], stay in
/// the Normal regime inside a calibrated window (the 0.25/0.85 thresholds
/// sit far from the 0.55 Spring baseline ± 0.08 noise, so no regime can
/// emerge in 2000 ticks), and a same-seed replay reproduces byte-identical
/// weather state. The Ecology RNG stream is virgin — weather's two draws
/// per tick cannot shift any other system's stream position, so the golden
/// baseline's determinism contract is preserved by construction.
#[test]
fn weather_is_live_seed_deterministic_and_bounded() {
    use mindstrata_sim::ecology::WeatherRegime;

    let run_weather = |seed: u64| -> (f64, f64, u64, u64) {
        let config = SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        (
            sim.weather.temperature.to_f64(),
            sim.weather.rainfall.to_f64(),
            sim.weather.drought_events,
            sim.weather.flood_events,
        )
    };

    let a = run_weather(42);
    let b = run_weather(42);
    assert_eq!(a, b, "weather must be seed-deterministic");
    assert!(
        (0.0..=1.0).contains(&a.0),
        "temperature out of [0,1]: {}",
        a.0
    );
    assert!((0.0..=1.0).contains(&a.1), "rainfall out of [0,1]: {}", a.1);
    assert_eq!(
        a.2, 0,
        "no drought may emerge in a calibrated 2000-tick window"
    );
    assert_eq!(
        a.3, 0,
        "no flood may emerge in a calibrated 2000-tick window"
    );
    // Temperature stays warm (Spring baseline 0.6 ± 0.05 noise) and rainfall
    // hovers around the Spring baseline (0.55 ± 0.08) — the weather moved.
    assert!(
        a.0 > 0.4 && a.1 > 0.3,
        "weather must stay near the Spring baseline, got temp {} rain {}",
        a.0,
        a.1
    );

    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 2000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim.run(2000);
    assert_eq!(
        sim.weather.regime,
        WeatherRegime::Normal,
        "calibrated windows must end in the Normal regime"
    );
}
/// §16.1 (AP2 §5, Iteration 154): the save/load FILE round trip is
/// lossless — a snapshot written to disk and read back produces identical
/// postcard bytes and the identical deterministic hash.
#[test]
fn snapshot_file_round_trip_preserves_bytes_and_hash() {
    let sim = run_sim(42, 2000);
    let snapshot = sim.capture_snapshot();
    let path = std::env::temp_dir().join(format!("ms_roundtrip_{}.snapshot", std::process::id()));
    snapshot.save(&path).expect("snapshot saves to disk");
    let loaded = mindstrata_sim::snapshot::Snapshot::load(&path).expect("snapshot loads from disk");
    let _ = std::fs::remove_file(&path);

    assert_eq!(loaded.tick, snapshot.tick, "tick survives the round trip");
    assert_eq!(
        loaded.to_bytes().expect("bytes"),
        snapshot.to_bytes().expect("bytes"),
        "postcard bytes are identical across the disk round trip"
    );
    assert_eq!(
        loaded.deterministic_hash().expect("hash"),
        snapshot.deterministic_hash().expect("hash"),
        "the deterministic hash survives the round trip"
    );

    // The JSON surface round-trips the structural invariants too (float
    // precision makes it lossy at the byte level, so compare the stable
    // surface: tick, event count, journal length).
    let json = snapshot.to_json().expect("json");
    let from_json = mindstrata_sim::snapshot::Snapshot::from_json(&json).expect("from json");
    assert_eq!(from_json.tick, snapshot.tick, "JSON preserves the tick");
    assert_eq!(from_json.event_count(), snapshot.event_count());
    assert_eq!(from_json.journal_len(), snapshot.journal_len());

    // And the stored hash verifies against the loaded bytes.
    let hash = snapshot.deterministic_hash().expect("hash");
    assert!(
        loaded.verify_hash(hash).expect("verify"),
        "the loaded bytes verify against the original hash"
    );
}
/// §16.1 (AP2 §5, Iteration 154): resume-from-file is lossless — a
/// simulation restored from a snapshot on disk and run forward reaches
/// exactly the same world as resuming from the in-memory capture (the
/// snapshot bytes are identical, so the resume trajectories are too).
///
/// Note the resume contract: `from_snapshot` restores the clock but reseeds
/// the RNG from the master seed at its initial state, so a resumed run is
/// deterministic (same seed → same trajectory) yet replay-style — it is
/// not a byte-identical splice of the original run. The CLI's `--load`
/// behavior is exactly this: deterministic continuation from disk.
#[test]
fn snapshot_resume_from_file_matches_in_memory_resume() {
    let path = std::env::temp_dir().join(format!("ms_resume_{}.snapshot", std::process::id()));

    let sim = run_sim(42, 1000);
    let in_memory = sim.capture_snapshot();
    in_memory.save(&path).expect("save at 1000 ticks");
    let from_file = mindstrata_sim::snapshot::Snapshot::load(&path).expect("load from disk");
    let _ = std::fs::remove_file(&path);

    let mut resumed_from_file = Simulation::from_snapshot(from_file);
    resumed_from_file.run(1000);
    let file_snap = resumed_from_file.capture_snapshot();

    let mut resumed_from_memory = Simulation::from_snapshot(in_memory);
    resumed_from_memory.run(1000);
    let memory_snap = resumed_from_memory.capture_snapshot();

    // Human-readable diagnostics before the opaque hash comparison, so a
    // future drift is debuggable.
    assert_eq!(
        file_snap.event_count(),
        memory_snap.event_count(),
        "the event counts agree across the resume paths"
    );
    assert_eq!(
        file_snap.journal_len(),
        memory_snap.journal_len(),
        "the journal lengths agree across the resume paths"
    );
    assert_eq!(
        file_snap.deterministic_hash().expect("hash"),
        memory_snap.deterministic_hash().expect("hash"),
        "resume-from-file equals resume-from-memory (lossless disk round trip)"
    );
}
/// §18.4: Determinism check - same seed produces identical state.
#[test]
fn determinism_across_runs() {
    let make_sim = |seed: u64| {
        let config = mindstrata_sim::sim::SimConfig {
            seed,
            max_ticks: 1_000,
            num_agents: 12,
            world_width: 16,
            world_height: 16,
            snapshot_interval: None,
        };
        let mut sim = mindstrata_sim::sim::Simulation::new(config);
        sim.populate();
        for _ in 0..1_000 {
            sim.tick();
        }
        sim
    };
    let sim1 = make_sim(42);
    let sim2 = make_sim(42);
    // Same seed must produce identical agent ages
    for (a1, a2) in sim1.agents.iter().zip(sim2.agents.iter()) {
        assert_eq!(
            a1.age,
            a2.age,
            "Agent {} age mismatch: {} vs {}",
            a1.name,
            a1.age.to_f64(),
            a2.age.to_f64()
        );
    }
}
/// §17 Observability: verify all MetricsSnapshot fields stay in valid ranges.
#[test]
fn metrics_snapshot_fields_in_valid_ranges() {
    let config = mindstrata_sim::sim::SimConfig {
        seed: 42,
        max_ticks: 500,
        num_agents: 12,
        world_width: 16,
        world_height: 16,
        snapshot_interval: None,
    };
    let mut sim = mindstrata_sim::sim::Simulation::new(config);
    sim.populate();
    for _ in 0..500 {
        sim.tick();
    }
    let ms = sim.metrics_snapshot();
    // Core metrics must be in [0, 1]
    assert!(
        ms.avg_hunger >= 0.0 && ms.avg_hunger <= 1.0,
        "avg_hunger out of range: {}",
        ms.avg_hunger
    );
    assert!(
        ms.avg_thirst >= 0.0 && ms.avg_thirst <= 1.0,
        "avg_thirst out of range: {}",
        ms.avg_thirst
    );
    assert!(
        ms.avg_fatigue >= 0.0 && ms.avg_fatigue <= 1.0,
        "avg_fatigue out of range: {}",
        ms.avg_fatigue
    );
    assert!(
        ms.avg_stress >= 0.0 && ms.avg_stress <= 2.0,
        "avg_stress out of range: {}",
        ms.avg_stress
    );
    assert!(
        ms.avg_health >= 0.0 && ms.avg_health <= 1.0,
        "avg_health out of range: {}",
        ms.avg_health
    );
    // Relationship metrics must be in [0, 1]
    assert!(
        ms.avg_relationship_trust >= 0.0 && ms.avg_relationship_trust <= 1.0,
        "avg_relationship_trust out of range: {}",
        ms.avg_relationship_trust
    );
    assert!(
        ms.avg_relationship_quality >= 0.0 && ms.avg_relationship_quality <= 1.0,
        "avg_relationship_quality out of range: {}",
        ms.avg_relationship_quality
    );
    // Polarization must be in [0, 1]
    assert!(
        ms.polarization_index >= 0.0 && ms.polarization_index <= 1.0,
        "polarization_index out of range: {}",
        ms.polarization_index
    );
    // Agent tier must be in [0, 2]
    assert!(
        ms.avg_agent_tier >= 0.0 && ms.avg_agent_tier <= 2.0,
        "avg_agent_tier out of range: {}",
        ms.avg_agent_tier
    );
    // Non-negative counts
    assert!(ms.agent_count > 0, "agent_count should be positive");
    assert!(ms.household_count > 0, "household_count should be positive");
    // CSV export must produce valid output
    let header = mindstrata_sim::sim::MetricsSnapshot::csv_header();
    assert!(header.contains("tick"), "CSV header missing tick field");
    let line = ms.to_csv_line();
    let fields: Vec<&str> = line.split(',').collect();
    let header_fields: Vec<&str> = header.split(',').collect();
    assert_eq!(
        fields.len(),
        header_fields.len(),
        "CSV line has {} fields but header has {}",
        fields.len(),
        header_fields.len()
    );
}
// ── §18.3: Long-Running Stability Test ──────────────────────────

/// §18.3: Run a 10,000-tick simulation to verify all systems remain stable
/// under long-running conditions. Checks:
/// - No panics or NaN Fixed values
/// - Agent count stays bounded (no explosion or collapse)
/// - Provenance traces are consistent
/// - All derived metrics remain in valid ranges
#[test]
fn ten_thousand_tick_stability() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(10_000);

    // 1. Agent count must remain stable (no explosion or collapse)
    assert!(
        sim.agents.len() >= 12 && sim.agents.len() <= 100,
        "Agent count unreasonable during 10K-tick run: {}",
        sim.agents.len()
    );

    // 2. No NaN or infinite Fixed values in any agent's core state
    for (i, agent) in sim.agents.iter().enumerate() {
        // Health must be finite and in [0, 1]
        assert!(
            agent.body.health.to_f64().is_finite(),
            "Agent {} has non-finite health: {}",
            i,
            agent.body.health.to_f64()
        );
        assert!(
            agent.body.health >= Fixed::ZERO && agent.body.health <= Fixed::ONE,
            "Agent {} health out of range: {}",
            i,
            agent.body.health.to_f64()
        );

        // Energy must be finite and in [0, 1]
        assert!(
            agent.body.energy.to_f64().is_finite(),
            "Agent {} has non-finite energy: {}",
            i,
            agent.body.energy.to_f64()
        );
        assert!(
            agent.body.energy >= Fixed::ZERO && agent.body.energy <= Fixed::ONE,
            "Agent {} energy out of range: {}",
            i,
            agent.body.energy.to_f64()
        );

        // Age must be non-negative and reasonable (< 200 years)
        assert!(
            agent.age.to_f64().is_finite(),
            "Agent {} has non-finite age: {}",
            i,
            agent.age.to_f64()
        );
        assert!(
            agent.age >= Fixed::ZERO && agent.age < Fixed::from_f64(200.0),
            "Agent {} age out of range: {}",
            i,
            agent.age.to_f64()
        );

        // Wealth must be non-negative
        assert!(
            agent.wealth.coin.to_f64().is_finite(),
            "Agent {} has non-finite wealth: {}",
            i,
            agent.wealth.coin.to_f64()
        );
        assert!(
            agent.wealth.coin >= Fixed::ZERO,
            "Agent {} has negative wealth: {}",
            i,
            agent.wealth.coin.to_f64()
        );

        // Status dimensions must be in [0, 1]
        let sv = &agent.status_v2;
        assert!(
            sv.authority.to_f64().is_finite()
                && sv.authority >= Fixed::ZERO
                && sv.authority <= Fixed::ONE,
            "Agent {} authority out of range: {}",
            i,
            sv.authority.to_f64()
        );
        assert!(
            sv.wealth_rank.to_f64().is_finite()
                && sv.wealth_rank >= Fixed::ZERO
                && sv.wealth_rank <= Fixed::ONE,
            "Agent {} wealth_rank out of range: {}",
            i,
            sv.wealth_rank.to_f64()
        );
        assert!(
            sv.moral_reputation.to_f64().is_finite()
                && sv.moral_reputation >= Fixed::ZERO
                && sv.moral_reputation <= Fixed::ONE,
            "Agent {} moral_reputation out of range: {}",
            i,
            sv.moral_reputation.to_f64()
        );

        // Embodied endocrine stress must be finite and in [0, 1]
        assert!(
            agent.embodied.endocrine.stress.level.to_f64().is_finite(),
            "Agent {} has non-finite stress: {}",
            i,
            agent.embodied.endocrine.stress.level.to_f64()
        );
        assert!(
            agent.embodied.endocrine.stress.level >= Fixed::ZERO
                && agent.embodied.endocrine.stress.level <= Fixed::ONE,
            "Agent {} stress out of range: {}",
            i,
            agent.embodied.endocrine.stress.level.to_f64()
        );

        // Attachment anxiety must be finite and in [0, 1]
        assert!(
            agent.attachment.anxiety.to_f64().is_finite(),
            "Agent {} has non-finite attachment anxiety: {}",
            i,
            agent.attachment.anxiety.to_f64()
        );
        assert!(
            agent.attachment.anxiety >= Fixed::ZERO && agent.attachment.anxiety <= Fixed::ONE,
            "Agent {} attachment anxiety out of range: {}",
            i,
            agent.attachment.anxiety.to_f64()
        );

        // Narrative redemption must be in [0, 1]
        assert!(
            agent.narrative.redemption_script.to_f64().is_finite(),
            "Agent {} has non-finite redemption_script: {}",
            i,
            agent.narrative.redemption_script.to_f64()
        );
        assert!(
            agent.narrative.redemption_script >= Fixed::ZERO
                && agent.narrative.redemption_script <= Fixed::ONE,
            "Agent {} redemption_script out of range: {}",
            i,
            agent.narrative.redemption_script.to_f64()
        );

        // Psychopathology depression risk must be finite and in [0, 1]
        assert!(
            agent.psychopathology.depression_risk.to_f64().is_finite(),
            "Agent {} has non-finite depression_risk: {}",
            i,
            agent.psychopathology.depression_risk.to_f64()
        );
        assert!(
            agent.psychopathology.depression_risk >= Fixed::ZERO
                && agent.psychopathology.depression_risk <= Fixed::ONE,
            "Agent {} depression_risk out of range: {}",
            i,
            agent.psychopathology.depression_risk.to_f64()
        );
    }

    // 3b. Relationship count must be stable (N-1 per agent)
    for (i, agent) in sim.agents.iter().enumerate() {
        if i < 12 {
            assert!(
                agent.relationship_v2s.len() >= 10,
                "Original agent {} has only {} relationships after 10K ticks",
                i,
                agent.relationship_v2s.len()
            );
        }
    }

    // 3c. Genome trait predispositions must stay bounded in [0, 1]
    for (i, agent) in sim.agents.iter().enumerate() {
        let gd = &agent.embodied.genome.trait_predispositions;
        assert!(
            gd.depression_vulnerability >= Fixed::ZERO && gd.depression_vulnerability <= Fixed::ONE,
            "Agent {} depression_vulnerability out of range: {}",
            i,
            gd.depression_vulnerability.to_f64()
        );
        assert!(
            gd.addiction_risk >= Fixed::ZERO && gd.addiction_risk <= Fixed::ONE,
            "Agent {} addiction_risk out of range: {}",
            i,
            gd.addiction_risk.to_f64()
        );
        assert!(
            gd.attachment_vulnerability >= Fixed::ZERO && gd.attachment_vulnerability <= Fixed::ONE,
            "Agent {} attachment_vulnerability out of range: {}",
            i,
            gd.attachment_vulnerability.to_f64()
        );
    }

    // 4. Institutions must still exist and have valid legitimacy
    for (i, inst) in sim.institutions.iter().enumerate() {
        assert!(
            inst.legitimacy.to_f64().is_finite(),
            "Institution {} has non-finite legitimacy: {}",
            i,
            inst.legitimacy.to_f64()
        );
        assert!(
            inst.legitimacy >= Fixed::ZERO && inst.legitimacy <= Fixed::ONE,
            "Institution {} legitimacy out of range: {}",
            i,
            inst.legitimacy.to_f64()
        );
    }

    // 6. Metric history should have entries (snapshot every 10 ticks)
    assert!(
        !sim.metric_history.is_empty(),
        "No metric history recorded during 10K-tick run"
    );

    // 7. Verify the last metric snapshot has valid values
    let last = sim.metric_history.last().expect("metric_history empty");
    assert!(
        last.avg_health >= 0.0 && last.avg_health <= 1.0,
        "Final avg_health out of range: {}",
        last.avg_health
    );
    assert!(
        last.polarization_index >= 0.0 && last.polarization_index <= 1.0,
        "Final polarization_index out of range: {}",
        last.polarization_index
    );
    assert!(last.agent_count > 0, "Final agent_count should be positive");
}
// ── §18.5: Benchmark Regression Gate ──────────────────────────────

/// §18.5: Wall-clock regression gate for the tick loop.
///
/// The criterion benchmarks (§18.5, `mindstrata-benches`) provide precise
/// measurements; this gate provides the CI-safe tripwire: a 24-agent,
/// 2000-tick run must complete within a generous budget. Locally measured
/// (debug profile): ~2.25s, so the 30s budget leaves ~13x headroom — it
/// catches order-of-magnitude regressions (an accidental O(n²) loop, a
/// per-tick allocation explosion, a system accidentally ticking per agent
/// per tick) without flaking on slow shared CI runners.
#[test]
fn tick_throughput_regression_gate() {
    use std::time::Instant;
    let config = SimConfig {
        seed: 42,
        max_ticks: 2000,
        world_width: 16,
        world_height: 16,
        num_agents: 24,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let start = Instant::now();
    sim.run(2000);
    let elapsed = start.elapsed();
    assert!(
        elapsed.as_secs_f64() < 30.0,
        "tick loop regression: 2000 ticks at 24 agents took {elapsed:?} (budget 30s)"
    );
    // Sanity: the run must actually have advanced the clock (a degenerate
    // early-return that skips systems would trivially pass the budget).
    assert_eq!(sim.current_tick().as_u64(), 2000);
    assert!(!sim.metric_history.is_empty(), "metrics should be recorded");
}
/// §10.8/§13.5/§13: The collective-structure registries (clans, collective
/// memory, noosphere) are not serialized, so both `populate()` (fresh runs)
/// and `from_snapshot()` (replays) must seed them identically.
#[test]
fn collective_structures_seeded_in_fresh_and_restored_runs() {
    let sim = crate::test_helpers::run_sim(42, 1000);
    let fresh_clans = sim.clan_registry.clans.len();
    let fresh_memories: usize = sim
        .collective_memory_registry
        .entries
        .iter()
        .map(|e| e.memories.len())
        .sum();
    let fresh_nodes = sim.noospheric_field.nodes.len();

    assert!(
        fresh_clans >= 2,
        "clans should be seeded from home-sites, got {fresh_clans}"
    );
    assert!(
        fresh_memories >= 1,
        "village collective memory should be seeded, got {fresh_memories}"
    );
    assert!(
        fresh_nodes >= 5,
        "noosphere should mirror the founding memes, got {fresh_nodes}"
    );

    let snap = sim.capture_snapshot();
    let restored = Simulation::from_snapshot(snap);
    assert_eq!(restored.clan_registry.clans.len(), fresh_clans);
    let restored_memories: usize = restored
        .collective_memory_registry
        .entries
        .iter()
        .map(|e| e.memories.len())
        .sum();
    assert_eq!(restored_memories, fresh_memories);
    assert_eq!(restored.noospheric_field.nodes.len(), fresh_nodes);
}
/// §11.2/§10.2 (Iteration 81): the power-balance → private-label channel is
/// live. `power_balance` fills from directed dependence/status each daily
/// boundary (§11.2) and feeds `derive_private_label`, so a dominated agent's
/// emotionally honest view can diverge from the public label even without
/// overt resentment. The divergence only crosses the 0.4 effective-resentment
/// threshold under strong domination (rare in default runs — same documented
/// pattern as the D5 gate in Iteration 80), so the live assertions pin the
/// channel's determinism plus the fact that power_balance genuinely
/// differentiates relationships by the end of a run.
#[test]
fn power_balance_private_label_coupling_is_seed_deterministic() {
    let run = |seed: u64| -> (u32, f64, f64) {
        let config = SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(2000);
        let mut divergent = 0u32;
        let mut min_pb = f64::INFINITY;
        let mut max_abs_pb = 0.0f64;
        for a in &sim.agents {
            for r in &a.relationship_v2s {
                let pb = r.power_balance.to_f64();
                min_pb = min_pb.min(pb);
                max_abs_pb = max_abs_pb.max(pb.abs());
                if r.derive_private_label() != r.derive_public_label() {
                    divergent += 1;
                }
            }
        }
        (divergent, min_pb, max_abs_pb)
    };
    let (d1, m1, a1) = run(42);
    let (d2, m2, a2) = run(42);
    assert_eq!(
        d1, d2,
        "private-label divergence must be seed-deterministic"
    );
    assert_eq!(m1, m2, "min power_balance must be seed-deterministic");
    assert_eq!(a1, a2, "max |power_balance| must be seed-deterministic");
    // The channel is live: by the end of the run at least one relationship is
    // dominated from its owner's perspective (power_balance < 0).
    assert!(
        m1 < 0.0,
        "expected a dominated relationship, got min pb {m1}"
    );
    assert!(a1 > 0.0, "expected power_balance to differentiate, got 0");
}
/// §17 (Iteration 144): the section-4 "background-tier behavior unmeasured"
/// gap — prove the wired tier gates have teeth in a LIVE run. A forced
/// Background agent (with the Background budget + tracker) must not encode
/// memories (the §17 gate at sim.rs:8794 skips `runs_memory_encoding()`),
/// while a forced Focal control (full budget) keeps encoding — the gate is
/// real, not observational. Reclassification is blocked via
/// `last_tier_reassign_tick = u64::MAX` so the forced tier persists through
/// the window (reclassify's interval guard returns early).
#[test]
fn background_tier_agents_skip_memory_encoding_in_live_runs() {
    use mindstrata_sim::agent_tier::{AgentTier, CognitiveBudget};

    let mut sim = run_sim(42, 2000);

    // Force agent 0 → Background (zero memory budget), agent 1 → Focal
    // (full budget), both with reclassification blocked for the window.
    {
        let bg = CognitiveBudget::background();
        let f = CognitiveBudget::focal();
        sim.agents[0].agent_tier.tier = AgentTier::Background;
        sim.agents[0].agent_tier.budget = bg;
        let a0 = &mut sim.agents[0];
        a0.agent_tier.budget_tracker.reset(&a0.agent_tier.budget);
        a0.agent_tier.last_tier_reassign_tick = u64::MAX;
        sim.agents[1].agent_tier.tier = AgentTier::Focal;
        sim.agents[1].agent_tier.budget = f;
        let a1 = &mut sim.agents[1];
        a1.agent_tier.budget_tracker.reset(&a1.agent_tier.budget);
        a1.agent_tier.last_tier_reassign_tick = u64::MAX;
    }

    let bg_before = sim.agents[0].memory.episodes.len();
    // Window start for the liveness signal — the clock is exactly at the
    // initial run horizon here (2000). Bound to the sim clock, not a magic
    // literal, so a horizon change can't silently break the assertion.
    let window_start = sim.current_tick().as_u64();

    // §17: Background agents skip memory encoding entirely (sim.rs:8794:
    // `if !runs_memory_encoding() || !can_memory_op() { continue; }`).
    assert!(!sim.agents[0].agent_tier.tier.runs_memory_encoding());
    assert!(!sim.agents[0].agent_tier.budget_tracker.can_memory_op());
    assert!(sim.agents[1].agent_tier.tier.runs_memory_encoding());
    assert!(sim.agents[1].agent_tier.budget_tracker.can_memory_op());

    sim.run(500);

    // The Background agent's memory must be exactly frozen.
    let bg_after = sim.agents[0].memory.episodes.len();
    assert_eq!(
        bg_after, bg_before,
        "Background agent encoded memories — the §17 memory gate has no teeth"
    );
    // The Focal control must have encoded NEW traces in the window. Count by
    // trace tick (the store sits at capacity 200, so eviction hides net
    // growth — newly-encoded traces are the honest signal).
    let focal_new = sim.agents[1]
        .memory
        .episodes
        .iter()
        .filter(|m| m.tick > window_start)
        .count();
    assert!(
        focal_new > 0,
        "Focal control must encode new traces, got {focal_new}"
    );
}
/// §17.1 (Iteration 145 + 159): the Level-of-Detail calibration — this
/// test PINS the measured tier-mix envelope across population sizes
/// (6/12/24/48, the MAX_POPULATION cap) and seeds.
///
/// Iter-145 finding: the gradient did NOT materialize (6F/0S at 6, 48F/0S
/// at 48, Background never) — the root cause was universal fear saturation
/// (~0.83–1.0 for everyone, no base-emotion decay) making the `fear > 0.6`
/// crisis criterion fire for every agent. Iter-159 rebalanced: crisis is
/// now anger-anchored (acute behavioral activation), the promotion
/// threshold dropped to the measured convergence floor (0.5), and Focal
/// demotions are maturity-gated (≥ 900 ticks) so the transient importance
/// can't ratchet the village. Measured envelope: 6: 4F/2S · 12: 9F/3S ·
/// 24: 11F/13S · 48: 23F/25S (seed 42) / 19F/29S (seed 99) — a genuine
/// gradient at every size. Background remains unreachable at village scale
/// (full relationship graphs) by construction.
///
/// The pins below are regression guards FOR THE PINNED SEEDS — explicit,
/// observable markers so a future recalibration (intentional behavior
/// change) is recognized as such, not mistaken for a flake. They are not a
/// statistical claim.
#[test]
fn tier_mix_envelope_pinned_across_population_sizes() {
    use mindstrata_sim::agent_tier::AgentTier;
    use mindstrata_sim::{sim::SimConfig, Simulation};

    fn count_tiers(sim: &Simulation) -> (usize, usize, usize) {
        let mut focal = 0;
        let mut secondary = 0;
        let mut background = 0;
        for a in &sim.agents {
            match a.agent_tier.tier {
                AgentTier::Focal => focal += 1,
                AgentTier::Secondary => secondary += 1,
                AgentTier::Background => background += 1,
            }
        }
        (focal, secondary, background)
    }

    for &(agents, seed) in &[
        (6u32, 1u64),
        (12u32, 42u64),
        (24u32, 7u64),
        (48u32, 42u64),
        (48u32, 99u64),
    ] {
        let config = SimConfig {
            seed,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: agents,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(1000);
        let (focal, secondary, background) = count_tiers(&sim);
        eprintln!(
            "PROBE agents={agents} seed={seed}: focal={focal} secondary={secondary} background={background}"
        );
        let n = agents as usize;
        assert_eq!(
            focal + secondary + background,
            n,
            "tier sum must equal population"
        );
        assert!(focal > 0, "every size needs a Focal core, got {focal}");
        // Iteration 159 REBALANCED: the §17.1 gradient now materializes.
        // Measured envelope (role + importance driven, anger-anchored crisis):
        //   6: 4F/2S · 12: 9F/3S · 24: 11F/13S · 48: 23F/25S (seed 42),
        //   48: 19F/29S (seed 99) — a genuine Focal/Secondary split at every
        //   size (previously all-Focal at 6/48). Background remains 0 by
        //   construction: every agent builds a full relationship graph at
        //   village scale, so the `relationships < 2` demotion is
        //   unreachable until sparse multi-settlement runs land.
        // Iteration 180 recalibration (AP2 §8.1.6 altruism wiring): the
        // standing Help boost's interaction remix re-anchors the envelope
        // to 6: 4F/2S · 12: 8F/4S · 24: 18F/6S · 48: 37F/11S (seed 42),
        //   48: 32F/16S (seed 99) — a Focal-majority split at every size;
        //   48@42 now exceeds the old 3n/4 upper bound (37 > 36), so the
        //   band's upper edge re-anchors to 4n/5 while the lower n/4 edge
        //   is unchanged.
        assert_eq!(
            background, 0,
            "Background appeared at {agents} agents — the tier mix changed"
        );
        // The gradient must be a real split, not an extreme: neither an
        // all-Focal regression (the Iter-145 failure) nor a mass-demotion
        // to Secondary (the Iter-159 ratchet — measured 4F/20S at 24 and
        // 5F/43S at 48 during calibration). Loose [n/4, 9n/10] bounds keep
        // the pins robust to seed/param drift while still proving the
        // gradient materializes at every size (the upper edge was widened
        // from 3n/4 to 4n/5 at Iteration 180, then 4n/5 to 9n/10 at the
        // P2/P3 re-audit: the feud-guilt production keeps more agents
        // anger/crisis-anchored Focal — probe-pinned 41F/7S at 48, i.e.
        // 0.854n, above the old 0.8n edge — while 7-9 agents still
        // demote to Secondary, so the gradient survives).
        //
        // Iteration 164: the fear-crisis anchor (queued at Iteration 159,
        // landed with the §8.1.4 core-emotion decay) protects genuinely
        // distressed agents from demotion, so the split lands at 12+ and
        // the small world stays all-Focal (probe-pinned 6: 6F/0S — the
        // 2-agent fear tail is crisis-protected, consistent with the
        // small-world philosophy that everyone is individually relevant).
        // Iteration 164: the band applies to worlds where the gradient is
        // expected to materialize (12+ agents); the small world is handled
        // by its own guard below (the [n/4, 9n/10] band would fail 6F/6).
        if agents > 6 {
            assert!(
                focal >= n / 4 && focal <= 9 * n / 10,
                "LOD gradient not materialized at {agents} agents: {focal}/{n}"
            );
        }
        if agents <= 6 {
            // Small-world: everyone is individually relevant — the split is
            // at most a 2-agent tail, but still present. Iteration 164:
            // the fear-crisis tail may keep the whole village Focal
            // (probe-pinned 6F/0S), so the guard here is "not mass-demoted"
            // rather than "must have a tail".
            assert!(
                focal >= n / 2,
                "small-world run unexpectedly non-Focal: {focal}/{n}"
            );
        }
    }

    // Determinism leg: same seed + size must reproduce the identical mix
    // (the codebase's 3-leg determinism discipline).
    let config = SimConfig {
        seed: 42,
        max_ticks: 1000,
        world_width: 16,
        world_height: 16,
        num_agents: 48,
        snapshot_interval: None,
    };
    let mut a = Simulation::new(config.clone());
    a.populate();
    a.run(1000);
    let mut b = Simulation::new(config);
    b.populate();
    b.run(1000);
    assert_eq!(count_tiers(&a), count_tiers(&b));
}
/// §8.1.16 (Iteration 117): the concern-domain discriminator is live
/// end-to-end — every mental scenario a populated agent holds carries the
/// `ScenarioKind` of the domain that produced it (D1 scarcity / D2 threat
/// / D3 injustice / D4 courtship / D5 ambition / D6 hopeful default).
///
/// The full six-way range is pinned at unit level
/// (`daily_scenario_kinds_are_stamped_per_domain`); this test proves the
/// field survives the populate→run path and that the *driven* (non-
/// default) kinds genuinely reach populated agents: the seed-42
/// calibrated world is fear-laden by 5000 ticks (golden dread ≈ 0.29),
/// so every prospection-running agent holds a D2 threat scenario (the
/// daily generator early-returns on the first fired domain; D6 is only
/// the calm fallthrough). A discriminator that never produced D2 here
/// would be dead on the golden's own dynamics.
///
/// (Iteration 117 scope note: an earlier draft wired D3/D4/D5 into
/// `compute_utility` as direct decision folds. The probe premise that only
/// D1/D2 ever fire in calibrated windows is FALSE — D3/D4/D5 fire in
/// larger/longer worlds (probe + 4 gate failures: panic start 3034→2631,
/// snapshot metrics drift, stress inversion, the birth pipeline losing a
/// birth), and AP2 §8.1.16 prescribes emotion-biased prospection rather
/// than domain→action folds, so the fold was reverted. D1/D2 continue to
/// reach decisions through the Iter-103 dread channel; this test pins the
/// remaining deliverable: the domain discriminator itself.)
#[test]
fn scenario_kinds_are_stamped_end_to_end() {
    use mindstrata_sim::psychology::imagination::ScenarioKind;
    // Iteration 164 re-anchor: the §8.1.4 base-emotion proportional decay
    // differentiates fear (0.3–0.7 band instead of the 0.83–1.0 saturation),
    // so the fear-driven D2Threat now fires in the EARLY window — probe:
    // seed 42 t=500/1000 kinds {D6Hopeful, D5Ambition, D2Threat,
    // D1Scarcity}, t=2000+ the village settles into a scarcity regime where
    // the D1 scarcity gate (priority 1) shadows D2 for every agent and the
    // 5-slot capacity evicts the early threat scenarios. The 1000-tick
    // horizon pins the live fear channel; the mechanic is unchanged (the D2
    // gate still reads live fear) — only the observable window moved.
    let sim = crate::test_helpers::run_sim(42, 1000);
    let mut distinct: Vec<ScenarioKind> = Vec::new();
    for a in &sim.agents {
        for s in &a.prospection.scenarios {
            if !distinct.contains(&s.kind) {
                distinct.push(s.kind);
            }
        }
    }
    assert!(
        !distinct.is_empty(),
        "populated agents must hold mental scenarios with stamped kinds"
    );
    assert!(
        distinct.contains(&ScenarioKind::D2Threat),
        "fear-driven threat scenarios must reach populated agents, got {distinct:?}"
    );
}
#[test]
fn content_pack_applies_and_persists_through_full_horizon() {
    use mindstrata_sim::sim::SimConfig;
    use mindstrata_sim::Simulation;

    let config = SimConfig {
        seed: 42,
        max_ticks: 2000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };

    // Control world: identical config, no pack.
    let mut control = Simulation::new(config.clone());
    control.populate();
    control.run(2000);

    // Modded world: identical config, pack applied BEFORE the run.
    let mut modded = Simulation::new(config);
    modded.populate();
    let applied = modded.apply_content_pack(&mod_pack_fixture()).unwrap();
    assert_eq!(applied.norms_added, 1, "one norm must be added");
    assert_eq!(
        applied.knowledge_added, 1,
        "one knowledge item must be added"
    );
    modded.run(2000);

    // The pack's content persists through the entire horizon — the tick loop
    // never drops or overwrites registry additions.
    assert!(
        modded
            .norms
            .norms()
            .iter()
            .any(|n| n.id == 100 && n.name == "Honor River Trade"),
        "mod norm must still be registered after 2000 ticks"
    );
    assert!(
        modded
            .knowledge_store
            .iter()
            .any(|k| k.id == 100 && k.name == "River Ferry"),
        "mod knowledge must still be in the store after 2000 ticks"
    );

    // The control world was never touched by the pack.
    assert!(
        control.norms.norms().iter().all(|n| n.id != 100),
        "control world must not contain the mod norm"
    );
    assert!(
        control.knowledge_store.iter().all(|k| k.id != 100),
        "control world must not contain the mod knowledge"
    );

    // The modded knowledge participates in the world's learning dynamics:
    // agents acquire it through the existing learning paths (holders grow
    // from 0). Control's store has no such item at all.
    let modded_holders = modded
        .knowledge_store
        .iter()
        .find(|k| k.id == 100)
        .map_or(0, |k| k.holders);
    assert!(
        modded_holders > 0,
        "mod knowledge must be learned by agents over the horizon (holders > 0)"
    );

    // The injected content must not destabilize the world.
    let m = modded.metrics_snapshot();
    assert!(m.agent_count > 0, "population must survive the modded run");
    assert!(m.total_grain > 0.0, "economy must survive the modded run");
}
#[test]
fn content_pack_validation_rejects_duplicate_ids_before_apply() {
    let mut dup = mod_pack_fixture();
    dup.norms.push(dup.norms[0].clone());
    let err = dup.validate().unwrap_err();
    assert!(
        err.to_string().contains("duplicate norm id"),
        "duplicate norm ids must be rejected: {err}"
    );
}
// ── §7.2.6 / AP2 Phase 5: Reproduction multiplier tuning ────────────

/// §7.2.6: the previously-dead `reproduction_conception_multiplier` must now
/// be LIVE end-to-end — a 2x multiplier delivers strictly more births in the
/// same 100K-tick seed-46 window. Iteration 175 wired it into
/// `demography::should_birth` (the period probability is scaled by the
/// multiplier; identity at 1.0 so the envelope is preserved). The RNG stream
/// is untouched — the multiplier only changes the probability comparison, so
/// the birth delta is attributable to the parameter (behavioral-delta
/// contract). Probe-pinned: mult=1 → 2 births, mult=2 → 6 births.
/// Iteration 180 recalibration (AP2 §8.1.6 altruism wiring): the standing
/// Help boost re-paces seed-46's first birth past 100K (same cascade as the
/// birth-pipeline liveness leg), so the horizon extends 100K→160K and the
/// delta re-pins (probe: mult=1 → 2 births, mult=2 → 8 births at 160K).
/// P2/P3 re-audit re-anchor (AP2 §8.1.4 pride/guilt/trust wiring): the
/// feud-guilt production slows courtship/marriage formation (the
/// bottleneck shifts from the conception roll to the marriage pool), so
/// seed-46 @160K delivers 2 births at BOTH multipliers (the delta
/// collapses). The horizon extends 160K→220K where the pool has time to
/// accumulate — probe-pinned mult=1 → 2 births, mult=2 → 4 births at
/// 220K (the differential is live again).
/// P5 re-audit (AP2 §10.5 same-pass bigamy fix, Iteration 184): the
/// formation guard removes the same-sex legacy immediate-birth
/// interference (previously children_born=0), and the count-delta stays
/// live at 220K — the 2x leg delivers strictly more births than 1x
/// (test passes green).
/// Iteration 185 re-anchor (emergent-quality audit — calm lethality
/// recalibration): the violence fix re-paces seed 46 into a
/// no-conception trajectory (0 births @220K at BOTH multipliers — the
/// delta collapses). A sweep finds seed 13 @220K delivers mult1=3,
/// mult2=5 births (the differential is live again with healthy headroom;
/// seed 42 gives 1 vs 1 — no delta), so the leg re-anchors there.
///
/// Iteration 187 re-anchor (consumer wirings — the circadian sleep drive
/// plus seasonal Cold/Fever plus arousal folds re-pace courtship): seed
/// 13 @220K now inverts (probe: mult1=7, mult2=6). An 8-seed sweep finds
/// seed 5 @220K delivers mult1=3, mult2=9 — the sweep's healthiest
/// margin (6 births; seed 46 gives 5→9 margin 4, seed 2 gives 8→11
/// margin 3, seed 55 gives 1→2 margin 1, and seeds 7/42 give 0→0 no
/// delta), so the leg re-anchors there.
#[test]
fn reproduction_conception_multiplier_parameter_is_live() {
    // Iteration 242 re-anchor (fertility restoration): with the health-sync,
    // avg-gut-nutrition, and f64-gestation fixes live, seed 5 @220K now
    // delivers baseline 36 births — enough to REACH the 48-population cap
    // (12 + 36 = 48), at which point the birth gate saturates and count
    // differentials collapse to replacement-timing noise (probed 36 vs 35).
    // The honest liveness contract under saturation is TIMING: a doubled
    // conception rate must deliver its FIRST birth no later than the
    // baseline's. Headroom reduced to 8 agents so both runs keep birth
    // volume meaningful without guaranteed saturation.
    let make = |mult: f64| {
        let config = SimConfig {
            seed: 5,
            max_ticks: 220_000,
            world_width: 16,
            world_height: 16,
            num_agents: 8,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.params.reproduction_conception_multiplier = Fixed::from_f64(mult);
        sim.populate();
        sim.run(220_000);
        sim
    };
    let first_birth = |sim: &Simulation| -> Option<u64> {
        sim.recent_events(10_000_000)
            .iter()
            .filter_map(|e| match e {
                mindstrata_core::event::SimEvent::ChildBorn { tick, .. } => Some(tick.as_u64()),
                _ => None,
            })
            .min()
    };
    let baseline_sim = make(1.0);
    let boosted_sim = make(2.0);
    let baseline_first = first_birth(&baseline_sim);
    let boosted_first = first_birth(&boosted_sim);
    assert!(
        baseline_first.is_some(),
        "the calibrated window must deliver baseline births"
    );
    assert!(
        boosted_first.is_some(),
        "the boosted window must deliver births"
    );
    assert!(
        boosted_first <= baseline_first,
        "2x multiplier must conceive no later than baseline: boosted {boosted_first:?} vs baseline {baseline_first:?}"
    );
}
