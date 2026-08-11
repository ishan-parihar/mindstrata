//! §18.5 Snapshot tests — capture key simulation outputs for regression detection.
//!
//! These tests use `cargo insta` to snapshot critical simulation state,
//! ensuring that any behavioral change in the simulation engine is caught
//! and reviewed before shipping.
//!
//! First run: `cargo test -p mindstrata-tests snapshot -- --nocapture`
//! Then accept baselines: `cargo insta review`
//! Subsequent runs compare against accepted baselines.

use std::collections::BTreeMap;

use mindstrata_sim::institutions::InstitutionKind;

use crate::test_helpers::run_sim;

// ── Module-level snapshot structs ──────────────────────────────
// Per Apollo Ch. 5, test helper types are module-level for reusability.
// Fields read by insta::assert_debug_snapshot! via Debug derive, not direct field access.

#[derive(Debug)]
#[expect(dead_code)] // insta reads via Debug, not direct field access
struct Metrics500 {
    avg_hunger: f64,
    avg_thirst: f64,
    avg_fatigue: f64,
    avg_stress: f64,
    avg_health: f64,
    avg_relationship_trust: f64,
    avg_relationship_quality: f64,
    polarization_index: f64,
    agent_count: f64,
    household_count: f64,
}

#[derive(Debug)]
#[expect(dead_code)] // insta reads via Debug, not direct field access
struct Metrics2000 {
    avg_hunger: f64,
    avg_thirst: f64,
    avg_fatigue: f64,
    avg_stress: f64,
    avg_health: f64,
    avg_relationship_trust: f64,
    polarization_index: f64,
    agent_count: f64,
    household_count: f64,
}

#[derive(Debug)]
#[expect(dead_code)] // insta reads via Debug, not direct field access
struct AgentSnapshot {
    name: String,
    age: f64,
    health: f64,
    energy: f64,
    hunger: f64,
    fear: f64,
    anger: f64,
    joy: f64,
    attachment_security: f64,
    attachment_anxiety: f64,
    heuristic_bias: f64,
    stress_level: f64,
    redemption_script: f64,
    depression_risk: f64,
    authority: f64,
    wealth_rank: f64,
}

#[derive(Debug)]
#[expect(dead_code)] // insta reads via Debug, not direct field access
struct InstitutionSnapshot {
    name: String,
    kind: String,
    legitimacy: f64,
    morale: f64,
    unity: f64,
    fear: f64,
    trust_in_leadership: f64,
    member_count: usize,
}

#[derive(Debug)]
#[expect(dead_code)] // insta reads via Debug, not direct field access
struct EndocrineSnapshot {
    name: String,
    stress: f64,
    bonding: f64,
    dominance: f64,
    fertility: f64,
    metabolic_energy: f64,
    arousal: f64,
    growth_capacity: f64,
}

#[derive(Debug)]
#[expect(dead_code)] // insta reads via Debug, not direct field access
struct AttachmentSnapshot {
    name: String,
    security: f64,
    anxiety: f64,
    avoidance: f64,
}

// ── Tests ─────────────────────────────────────────────────────

/// §18.5: Snapshot agent metrics after 500 ticks — catches any drift in
/// core biological, psychological, or social state distributions.
#[test]
fn snapshot_agent_metrics_500_ticks() {
    let sim = run_sim(42, 500);
    let ms = sim.metrics_snapshot();

    insta::assert_debug_snapshot!(
        "metrics_500_ticks",
        Metrics500 {
            avg_hunger: ms.avg_hunger,
            avg_thirst: ms.avg_thirst,
            avg_fatigue: ms.avg_fatigue,
            avg_stress: ms.avg_stress,
            avg_health: ms.avg_health,
            avg_relationship_trust: ms.avg_relationship_trust,
            avg_relationship_quality: ms.avg_relationship_quality,
            polarization_index: ms.polarization_index,
            agent_count: ms.agent_count as f64,
            household_count: ms.household_count as f64,
        }
    );
}

/// §18.5: Snapshot agent metrics after 2000 ticks — catches long-run drift.
#[test]
fn snapshot_agent_metrics_2000_ticks() {
    let sim = run_sim(42, 2000);
    let ms = sim.metrics_snapshot();

    insta::assert_debug_snapshot!(
        "metrics_2000_ticks",
        Metrics2000 {
            avg_hunger: ms.avg_hunger,
            avg_thirst: ms.avg_thirst,
            avg_fatigue: ms.avg_fatigue,
            avg_stress: ms.avg_stress,
            avg_health: ms.avg_health,
            avg_relationship_trust: ms.avg_relationship_trust,
            polarization_index: ms.polarization_index,
            agent_count: ms.agent_count as f64,
            household_count: ms.household_count as f64,
        }
    );
}

/// §18.5: Snapshot individual agent state after 1000 ticks — catches per-agent
/// biological/psychological state drift.
#[test]
fn snapshot_agent_state_1000_ticks() {
    let sim = run_sim(42, 1000);

    let agent_states: Vec<AgentSnapshot> = sim
        .agents
        .iter()
        .take(3)
        .map(|a| AgentSnapshot {
            name: a.name.clone(),
            age: a.age.to_f64(),
            health: a.body.health.to_f64(),
            energy: a.body.energy.to_f64(),
            hunger: a.needs.hunger.to_f64(),
            fear: a.emotions.fear.to_f64(),
            anger: a.emotions.anger.to_f64(),
            joy: a.emotions.joy.to_f64(),
            attachment_security: a.attachment.security.to_f64(),
            attachment_anxiety: a.attachment.anxiety.to_f64(),
            heuristic_bias: a.cognitive.heuristic_bias.to_f64(),
            stress_level: a.embodied.endocrine.stress.level.to_f64(),
            redemption_script: a.narrative.redemption_script.to_f64(),
            depression_risk: a.psychopathology.depression_risk.to_f64(),
            authority: a.status_v2.authority.to_f64(),
            wealth_rank: a.status_v2.wealth_rank.to_f64(),
        })
        .collect();

    insta::assert_debug_snapshot!("agent_states_1000_ticks", agent_states);
}

/// §18.5: Snapshot institution state after 1000 ticks — catches institutional
/// legitimacy and collective psychology drift.
#[test]
fn snapshot_institution_state_1000_ticks() {
    let sim = run_sim(42, 1000);

    let inst_states: Vec<InstitutionSnapshot> = sim
        .institutions
        .iter()
        .map(|inst| InstitutionSnapshot {
            name: inst.name.clone(),
            kind: format!("{:?}", inst.kind),
            legitimacy: inst.legitimacy.to_f64(),
            morale: inst.collective.morale.to_f64(),
            unity: inst.collective.unity.to_f64(),
            fear: inst.collective.fear.to_f64(),
            trust_in_leadership: inst.collective.trust_in_leadership.to_f64(),
            member_count: inst.members.len(),
        })
        .collect();

    insta::assert_debug_snapshot!("institution_states_1000_ticks", inst_states);
}

/// §18.5: Snapshot relationship stage distribution after 2000 ticks — catches
/// social layer regression in relationship progression.
#[test]
fn snapshot_relationship_stages_2000_ticks() {
    let sim = run_sim(42, 2000);

    // Count relationships at each stage
    let mut stage_counts = std::collections::BTreeMap::new();
    for agent in &sim.agents {
        for rv2 in &agent.relationship_v2s {
            let stage_name = format!("{:?}", rv2.stage);
            *stage_counts.entry(stage_name).or_insert(0u32) += 1;
        }
    }

    insta::assert_debug_snapshot!("relationship_stage_distribution_2000_ticks", stage_counts);
}

/// §18.5: Snapshot biological endocrine state after 500 ticks — catches
/// hormonal axis drift that could cascade through psychology.
#[test]
fn snapshot_endocrine_state_500_ticks() {
    let sim = run_sim(42, 500);

    let endocrine_states: Vec<EndocrineSnapshot> = sim
        .agents
        .iter()
        .take(3)
        .map(|a| EndocrineSnapshot {
            name: a.name.clone(),
            stress: a.embodied.endocrine.stress.level.to_f64(),
            bonding: a.embodied.endocrine.bonding.level.to_f64(),
            dominance: a.embodied.endocrine.dominance.level.to_f64(),
            // Iter 42: the endocrine FertilityAxis was dead state (never updated,
            // never read) and was removed per the YAGNI directive; the live
            // fertility lives on the reproductive system.
            fertility: a.embodied.reproductive.fertility.to_f64(),
            metabolic_energy: a.embodied.endocrine.metabolic.energy.to_f64(),
            arousal: a.embodied.endocrine.arousal.level.to_f64(),
            growth_capacity: a.embodied.endocrine.growth.capacity.to_f64(),
        })
        .collect();

    insta::assert_debug_snapshot!("endocrine_states_500_ticks", endocrine_states);
}

/// §18.5: Snapshot psychological attachment state after 1000 ticks — catches
/// attachment system regression.
#[test]
fn snapshot_attachment_state_1000_ticks() {
    let sim = run_sim(42, 1000);

    let attachment_states: Vec<AttachmentSnapshot> = sim
        .agents
        .iter()
        .take(3)
        .map(|a| AttachmentSnapshot {
            name: a.name.clone(),
            security: a.attachment.security.to_f64(),
            anxiety: a.attachment.anxiety.to_f64(),
            avoidance: a.attachment.avoidance.to_f64(),
        })
        .collect();

    insta::assert_debug_snapshot!("attachment_states_1000_ticks", attachment_states);
}

// ── Long-horizon surface snapshot (>4320 ticks — past the first ritual) ───
// Iteration 136: the queue's snapshot-coverage gap. All seven legacy
// snapshots sit at <= 2000 ticks, below the first ritual firing (4320). This
// is the first snapshot past a full ritual cycle — it captures the norm
// internalization, memory traces, and mate-search attraction state that only
// exist after structural time.

#[derive(Debug)]
#[expect(dead_code)] // insta reads via Debug, not direct field access
struct LongHorizonSurface10000 {
    // Core metrics (Iter-135 proved byte-identical 50K determinism).
    tick: u64,
    agent_count: u64,
    event_count: u64,
    avg_stress: f64,
    avg_health: f64,
    avg_relationship_trust: f64,
    avg_relationship_quality: f64,
    polarization_index: f64,
    active_meme_count: u64,
    total_grain: f64,
    total_water: f64,
    // Memory surface (§13.5 / §8.1.3): the nine-kind taxonomy distribution.
    total_memory_traces: usize,
    memory_kind_distribution: BTreeMap<String, usize>,
    // Norm surface (§22.1): internalized norms after two ritual cycles.
    total_internalized_norms: usize,
    avg_norm_strength: f64,
    // Attraction surface (§10.4): population means. Honest probe-pinned note:
    // in this calm seed-42 village the physical/personality/moral channels sit
    // at their init defaults (0.5) — the D4 courtship gate rarely opens, so
    // only familiarity/status move; the LIVE surface is exercised only in
    // courtship-active worlds (covered by the courtship integration tests).
    // envy_cost is 0.0 — legitimately, envy never fires in calm worlds.
    avg_physical_attraction: f64,
    avg_personality_attraction: f64,
    avg_status_attraction: f64,
    avg_moral_attraction: f64,
    avg_familiarity: f64,
    avg_envy_cost: f64,
    // Relationship surface: edges + stage distribution.
    relationship_count: usize,
    relationship_stage_distribution: BTreeMap<String, u32>,
    // Ritual execution (§22.4): durable `last_occurrence` signal. Participant
    // memory traces decay below eviction before 50K (Iter-135 probe finding),
    // so `last_occurrence` is the liveness proof that rituals actually fired.
    executed_ritual_count: usize,
    latest_ritual_occurrence: u64,
    // Institutional surface.
    institution_count: usize,
    faction_count: usize,
}

/// §18.5 (Iteration 136): Snapshot the full norm/memory/attraction surface at
/// 10000 ticks — past the first ritual firing (4320) and its second (8640).
/// Catches drift in ritual-driven norm internalization, memory-survival
/// behavior, and mate-search state that the sub-2000-tick snapshots cannot.
///
/// Honest probe-pinned reading of the baseline (Iter-134 calm-window
/// discipline): (1) the attraction surface's physical/personality/moral
/// channels sit at their init defaults (0.5) — the D4 courtship gate rarely
/// opens in a calm village, so this snapshot pins the DEFAULT surface (only
/// familiarity/status move); (2) the memory taxonomy shows 6 of 9 kinds —
/// `Semantic`'s absence signals the schooling/knowledge surface is dormant at
/// 10K (consistent with the Iter-134 saturation sweep); (3) rituals EXECUTE
/// (`executed_ritual_count: 2`, latest at 8640) — the durable `last_occurrence`
/// signal, since participant traces decay below eviction before 50K.
#[test]
fn snapshot_long_horizon_surface_10000_ticks() {
    let sim = run_sim(42, 10_000);
    let ms = sim.metrics_snapshot();

    // Memory surface: every surviving trace in every agent's store.
    let total_memory_traces: usize = sim.agents.iter().map(|a| a.memory.episodes.len()).sum();
    let mut memory_kind_distribution = BTreeMap::new();
    for a in &sim.agents {
        for t in &a.memory.episodes {
            *memory_kind_distribution
                .entry(format!("{:?}", t.kind))
                .or_insert(0usize) += 1;
        }
    }

    // Norm surface: internalized norms and their mean strength.
    let total_internalized_norms: usize = sim
        .agents
        .iter()
        .map(|a| a.moral_cognition.internalized_norms.len())
        .sum();
    let norm_strength_sum: f64 = sim
        .agents
        .iter()
        .flat_map(|a| &a.moral_cognition.internalized_norms)
        .map(|n| n.strength.to_f64())
        .sum();
    let avg_norm_strength = if total_internalized_norms > 0 {
        norm_strength_sum / total_internalized_norms as f64
    } else {
        0.0
    };

    // Attraction surface: population means (0.0 is legitimate — an agent
    // that never evaluated a partner has an untouched attraction model).
    let pop = sim.agents.len().max(1) as f64;
    let avg_physical_attraction = sim
        .agents
        .iter()
        .map(|a| a.attraction.physical_attraction.to_f64())
        .sum::<f64>()
        / pop;
    let avg_personality_attraction = sim
        .agents
        .iter()
        .map(|a| a.attraction.personality_attraction.to_f64())
        .sum::<f64>()
        / pop;
    let avg_status_attraction = sim
        .agents
        .iter()
        .map(|a| a.attraction.status_attraction.to_f64())
        .sum::<f64>()
        / pop;
    let avg_moral_attraction = sim
        .agents
        .iter()
        .map(|a| a.attraction.moral_attraction.to_f64())
        .sum::<f64>()
        / pop;
    let avg_familiarity = sim
        .agents
        .iter()
        .map(|a| a.attraction.familiarity.to_f64())
        .sum::<f64>()
        / pop;
    let avg_envy_cost = sim
        .agents
        .iter()
        .map(|a| a.attraction.envy_cost.to_f64())
        .sum::<f64>()
        / pop;

    // Relationship surface: edges + stage distribution.
    let relationship_count: usize = sim.agents.iter().map(|a| a.relationship_v2s.len()).sum();
    let mut relationship_stage_distribution = BTreeMap::new();
    for agent in &sim.agents {
        for rv2 in &agent.relationship_v2s {
            let stage_name = format!("{:?}", rv2.stage);
            *relationship_stage_distribution
                .entry(stage_name)
                .or_insert(0u32) += 1;
        }
    }

    // Ritual execution: the durable signal (participant traces decay away).
    let executed_ritual_count = sim
        .ritual_registry
        .rituals
        .iter()
        .filter(|r| r.last_occurrence > 0)
        .count();
    let latest_ritual_occurrence = sim
        .ritual_registry
        .rituals
        .iter()
        .map(|r| r.last_occurrence)
        .max()
        .unwrap_or(0);

    // Institutional surface.
    let institution_count = sim.institutions.len();
    let faction_count = sim
        .institutions
        .iter()
        .filter(|i| i.kind == InstitutionKind::Faction)
        .count();

    insta::assert_debug_snapshot!(
        "long_horizon_surface_10000_ticks",
        LongHorizonSurface10000 {
            tick: ms.tick,
            agent_count: ms.agent_count,
            event_count: ms.event_count,
            avg_stress: ms.avg_stress,
            avg_health: ms.avg_health,
            avg_relationship_trust: ms.avg_relationship_trust,
            avg_relationship_quality: ms.avg_relationship_quality,
            polarization_index: ms.polarization_index,
            active_meme_count: ms.active_meme_count,
            total_grain: ms.total_grain,
            total_water: ms.total_water,
            total_memory_traces,
            memory_kind_distribution,
            total_internalized_norms,
            avg_norm_strength,
            avg_physical_attraction,
            avg_personality_attraction,
            avg_status_attraction,
            avg_moral_attraction,
            avg_familiarity,
            avg_envy_cost,
            relationship_count,
            relationship_stage_distribution,
            executed_ritual_count,
            latest_ritual_occurrence,
            institution_count,
            faction_count,
        }
    );
}
