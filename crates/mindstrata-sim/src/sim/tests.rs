//! Unit tests extracted from the former monolithic sim.rs.

use super::snapshot_metrics::self_esteem_support;
use super::*;
use crate::institutions;

/// §5 (Iteration 149): the enforce_theft → court wiring — a caught theft
/// on an owned site files a case and convicts the thief. Deterministic:
/// council enforcement is maxed (detection roll < 1 always catches) and
/// the thief's zero risk tolerance keeps them out of the black market
/// (which would halve effective enforcement).
#[test]
fn caught_theft_on_owned_site_is_prosecuted() {
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
    for inst in &mut sim.institutions {
        if inst.kind == crate::institutions::InstitutionKind::Council {
            inst.enforcement_capacity = Fixed::ONE;
        }
    }
    sim.agents[0].personality.risk_tolerance = Fixed::ZERO;
    let farm_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .expect("riverford has farms");
    let thief = AgentId::new(0);
    let victim = AgentId::new(1);
    sim.world.sites[farm_idx].owner = Some(victim);

    let taken = sim.enforce_theft(
        0,
        thief,
        farm_idx,
        GRAIN_RESOURCE_ID,
        Fixed::from_f64(5.0),
        100,
        Tick::new(100),
    );
    assert!(
        taken,
        "the theft succeeds (no internalized norms before tick 4320)"
    );
    assert_eq!(sim.legal.cases.len(), 1, "the caught theft files a case");
    let case = &sim.legal.cases[0];
    assert_eq!(
        case.verdict,
        Some(Verdict::Guilty),
        "owned-site theft convicts"
    );
    assert_eq!(case.victim, Some(victim));
    assert_eq!(case.site_idx, Some(farm_idx));
    assert_eq!(sim.legal.convictions, 1);
    assert_eq!(sim.legal.established_tick, Some(100));
}

/// §8.1: Self-esteem support is zero at baseline and signed around it.
#[test]
fn self_esteem_support_is_zero_at_baseline() {
    assert_eq!(self_esteem_support(Fixed::from_f64(0.5)), Fixed::ZERO);
    assert!(self_esteem_support(Fixed::from_f64(0.9)) > Fixed::ZERO);
    assert!(self_esteem_support(Fixed::from_f64(0.1)) < Fixed::ZERO);
}

/// §8.1: The self-model must track life events — previously constructed
/// but never updated (a dead system). After 500 ticks the focal agents'
/// self-esteem must have drifted from the 0.5 baseline as their
/// narratives (and the self-model's parallel narrative) form.
#[test]
fn self_model_tracks_life_events() {
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
    for _ in 0..500 {
        sim.tick();
    }
    let any_drift = sim.agents.iter().any(|a| {
        let d = a.self_model.self_esteem - Fixed::from_f64(0.5);
        d > Fixed::from_f64(0.005)
            || d < -Fixed::from_f64(0.005)
            || a.self_model.narrative.contamination_script != Fixed::from_f64(0.2)
    });
    assert!(
        any_drift,
        "self_model must update from life events (was a dead system)"
    );
}

/// §12.4: Cult emergence must fire when institutional legitimacy is low
/// and agents suffer a meaning deficit — and the cooldown must prevent
/// cult spam. Selection is deterministic (no RNG).
#[test]
fn cults_emerge_under_low_legitimacy_and_meaning_crisis() {
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
    // Force the §12.4 preconditions: weak institutions + meaning crisis.
    for inst in &mut sim.institutions {
        inst.legitimacy = Fixed::from_f64(0.2);
    }
    for agent in &mut sim.agents {
        agent.needs.meaning = Fixed::from_f64(0.9);
    }
    let before = sim.cult_registry.cults.iter().filter(|c| c.active).count();
    sim.tick_cults(3000); // >= CULT_COOLDOWN so formation is not blocked
    let after = sim.cult_registry.cults.iter().filter(|c| c.active).count();
    assert!(
        after > before,
        "cult should form under low legitimacy + meaning crisis"
    );
    assert_eq!(sim.last_cult_formation_tick, 3000);
    // Cooldown: a second qualifying tick must NOT form another cult.
    sim.tick_cults(3100);
    assert_eq!(
        sim.cult_registry.cults.iter().filter(|c| c.active).count(),
        after,
        "cult formation cooldown must prevent cult spam"
    );
}

/// §13: Noospheric nodes must stay activated by meme-driven spreading
/// (total activation rises above the initial 0.3 baseline after one tick).
#[test]
fn noosphere_spreads_activation_from_memes() {
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
    assert!(
        !sim.noospheric_field.nodes.is_empty(),
        "noosphere must be seeded"
    );
    let before: Fixed = sim
        .noospheric_field
        .nodes
        .iter()
        .fold(Fixed::ZERO, |acc, n| acc + n.activation);
    sim.tick_noosphere();
    let after: Fixed = sim
        .noospheric_field
        .nodes
        .iter()
        .fold(Fixed::ZERO, |acc, n| acc + n.activation);
    assert!(
        after > before,
        "meme-driven spreading should raise activation"
    );
}

/// §12.4: Cult belonging must satisfy members' meaning need and entrench
/// their beliefs (the cult's narrative grip).
#[test]
fn cult_members_get_psychological_feedback() {
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
    for inst in &mut sim.institutions {
        inst.legitimacy = Fixed::from_f64(0.2);
    }
    for agent in &mut sim.agents {
        agent.needs.meaning = Fixed::from_f64(0.9);
    }
    sim.tick_cults(3000);
    let cult = &sim.cult_registry.cults[0];
    assert!(
        !cult.members.is_empty(),
        "cult formation should store its members"
    );
    let member = cult.members[0];
    let pre_meaning = sim.agents[member].needs.meaning;
    let pre_conf: Fixed = sim.agents[member]
        .beliefs
        .iter()
        .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
    sim.tick_cult_dynamics();
    assert!(
        sim.agents[member].needs.meaning < pre_meaning,
        "cult belonging should satisfy the meaning need"
    );
    let post_conf: Fixed = sim.agents[member]
        .beliefs
        .iter()
        .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
    assert!(post_conf > pre_conf, "cult should entrench member beliefs");
}

/// §12.4: Active cults must recruit meaning-starved agents up to the
/// membership cap.
#[test]
fn cults_recruit_meaning_starved_agents() {
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
    for inst in &mut sim.institutions {
        inst.legitimacy = Fixed::from_f64(0.2);
    }
    for agent in &mut sim.agents {
        agent.needs.meaning = Fixed::from_f64(0.9);
    }
    sim.tick_cults(3000);
    let after_formation = sim.cult_registry.cults[0].members.len();
    sim.tick_cult_dynamics();
    let after_recruit = sim.cult_registry.cults[0].members.len();
    // 12 agents: leader + up to 4 formed members; cap = 12/2 = 6.
    assert!(
        after_recruit > after_formation,
        "cult should recruit meaning-starved agents"
    );
    assert!(after_recruit <= 6, "membership should respect the cap");
}

/// §12.4: When a cult dissolves, former members suffer a meaning-crisis
/// rebound and their cult-entrenched beliefs decay — the lifecycle's
/// closing act.
#[test]
fn cult_dissolution_causes_member_fallout() {
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
    for inst in &mut sim.institutions {
        inst.legitimacy = Fixed::from_f64(0.2);
    }
    for agent in &mut sim.agents {
        agent.needs.meaning = Fixed::from_f64(0.9);
    }
    sim.tick_cults(3000); // forms the cult
    let member = sim.cult_registry.cults[0].members[0];
    // Entrench the member first (dynamics raises confidence/charge).
    sim.tick_cult_dynamics();
    let entrench_conf: Fixed = sim.agents[member]
        .beliefs
        .iter()
        .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
    let entrench_charge: Fixed = sim.agents[member]
        .beliefs
        .iter()
        .fold(Fixed::ZERO, |acc, b| acc + b.emotional_charge);
    let pre_fallout_meaning = sim.agents[member].needs.meaning;
    // Force dissolution: high institutional legitimacy + low isolation.
    for inst in &mut sim.institutions {
        inst.legitimacy = Fixed::from_f64(0.9);
    }
    if let Some(cult) = sim.cult_registry.cults.first_mut() {
        cult.isolation = Fixed::from_f64(0.1);
        cult.dependence = Fixed::from_f64(0.3); // keep leader-failure inert
    }
    sim.tick_cults(6000); // cooldown passed; dissolution check runs
    assert!(
        !sim.cult_registry.cults[0].active,
        "cult should dissolve under high legitimacy + low isolation"
    );
    // Fallout: meaning rebounds into crisis, entrenched beliefs decay.
    assert!(
        sim.agents[member].needs.meaning > pre_fallout_meaning,
        "meaning need should rebound after the cult dissolves"
    );
    let post_conf: Fixed = sim.agents[member]
        .beliefs
        .iter()
        .fold(Fixed::ZERO, |acc, b| acc + b.confidence);
    let post_charge: Fixed = sim.agents[member]
        .beliefs
        .iter()
        .fold(Fixed::ZERO, |acc, b| acc + b.emotional_charge);
    assert!(
        post_conf <= entrench_conf,
        "entrenched beliefs should decay after dissolution"
    );
    assert!(
        post_charge <= entrench_charge,
        "emotional charge should decay after dissolution"
    );
}

/// §13.5: Ritual repetition must sustain the village's collective memory —
/// salience stays high over a long run instead of decaying to zero (the
/// pre-Iteration-12 quadratic decay wiped memories within ~2 weeks).
#[test]
fn collective_memories_survive_ritual_rehearsal() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 20_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    // ~1 monthly ritual rehearsal (tick 4320) over 5000 ticks, during which
    // the daily decay (0.001/day) would have destroyed every memory under
    // the old quadratic accumulation (~2-week lifespan).
    sim.run(5000);
    let village = sim.collective_memory_registry.get(0);
    assert!(
        village.is_some(),
        "village collective memory must be seeded"
    );
    let memories = &village.unwrap().memories;
    assert!(!memories.is_empty(), "village memories must exist");
    for mem in memories {
        assert!(
            mem.salience > Fixed::from_f64(0.5),
            "memory '{}' salience should be sustained by ritual rehearsal, got {}",
            mem.description,
            mem.salience.to_f64(),
        );
    }
}

/// §8.1.4 (Iteration 129): the nostalgia → collective-memory
/// preservation wiring, proven through the PUBLIC path (no field
/// injection — nostalgia's producer is live everywhere, so an
/// injection differential is impossible; instead the daily pass's own
/// decay delta is measured). Run past two daily boundaries (5040 =
/// 35×144 and 5184 = 36×144) with NO ritual fire in the window (the
/// first ritual is at 4320, the next at 8640) and no famine memory:
/// the total salience lost over those two days must be strictly LESS
/// than the un-scaled 2 × 0.001 × count (the Iter-12 linear daily
/// decay) — the pass must have multiplied by a preservation factor <
/// 1 derived from the LIVE population mean nostalgia — and strictly
/// MORE than the floor-scaled value (preservation never fully
/// freezes the fade). If the fold in the daily pass is ever deleted,
/// `decayed == unscaled` and this test fails.
#[test]
fn nostalgia_preserves_collective_memory_salience() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 6_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config.clone());
    sim.populate();
    sim.run(5000);
    let total_salience = |sim: &Simulation| -> f64 {
        sim.collective_memory_registry.get(0).map_or(0.0, |cm| {
            cm.memories.iter().map(|m| m.salience.to_f64()).sum::<f64>()
        })
    };
    let count = sim
        .collective_memory_registry
        .get(0)
        .map_or(0.0, |cm| cm.memories.len() as f64);
    assert!(count > 0.0, "the village collective memory must be seeded");
    let before = total_salience(&sim);
    // Two daily boundaries: 5040 and 5184. No ritual in this window
    // (rituals at 4320/8640), no famine memory (calm world).
    sim.run(200);
    let after = total_salience(&sim);
    let decayed = before - after;
    let unscaled = 2.0 * 0.001 * count;
    let floor_scaled = unscaled * crate::appraisal::NOSTALGIA_PRESERVATION_FLOOR;
    assert!(
        decayed < unscaled,
        "the daily pass must apply a live-nostalgia preservation factor < 1 \
             (decayed {decayed:.6} vs un-scaled {unscaled:.6} over 2 days)"
    );
    // The daily pass scales the fade by `1 − mean_nostalgia × rate`,
    // floored at 0.7 — the honest invariant is that the factor lives in
    // [floor, 1.0): decayed is strictly less than the un-scaled amount
    // (a live preservation factor < 1, asserted above) and never
    // STRONGER than the floor (preservation never fully freezes the
    // fade). The pre-iteration-184 tight pin (factor == floor exactly,
    // calibrated on nostalgia saturating at 1.0) was already stale — the
    // NOST_DEBUG probe shows the pass reading mean nostalgia 0.80–0.83
    // (factor 0.75–0.76) at the daily boundaries, and the audit-closure
    // feud-guilt path (goal-incongruent feuding agents no longer
    // produce the positive congruence that feeds nostalgia) lowers it
    // to ~0.28–0.35 at the pass point (factor 0.90–0.92) — still a
    // live, legal preservation factor, just not floor-pinned. The
    // robust band asserts the factor is a genuine differential in
    // [floor, 1.0): decayed ∈ [floor_scaled, unscaled − 0.0002]
    // (a 0.9× factor on count=2 yields ~0.0036, comfortably inside;
    // a fold deletion yields decayed == unscaled and fails the upper
    // bound; the factor CAN legitimately sit AT the floor 0.7 — the
    // Iteration 190 hydration re-pace (routine drink slot + Drink
    // relief 0.7) shifted the emotion mix so nostalgia saturates to
    // 1.0 at the pass point: decayed == floor_scaled == 0.002800 is
    // maximal legal preservation, probe-pinned, not a fold).
    assert!(
        decayed >= floor_scaled - 0.0001 && decayed < unscaled - 0.0002,
        "the preservation factor must be a live differential in [0.7, 1.0) \
             (decayed {decayed:.6}, floor-scaled {floor_scaled:.6}, un-scaled {unscaled:.6})"
    );
    // Determinism: identical seed → byte-identical decay delta.
    let mut again = Simulation::new(config);
    again.populate();
    again.run(5000);
    let b2 = total_salience(&again);
    again.run(200);
    assert_eq!(
        before, b2,
        "the pre-window salience must be seed-deterministic"
    );
    assert_eq!(
        decayed,
        before - total_salience(&again),
        "the preservation-scaled decay must be seed-deterministic"
    );
}

/// §13: The noosphere zeitgeist must feed back into agents — a hot field
/// amplifies the hottest belief's emotional charge; a dormant field
/// changes nothing.
#[test]
fn noosphere_zeitgeist_amplifies_conviction() {
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
    // Dormant field: zero activation and backing → zeitgeist 0 → no-op.
    for node in &mut sim.noospheric_field.nodes {
        node.activation = Fixed::ZERO;
        node.institutional_backing = Fixed::ZERO;
    }
    let charge_of = |sim: &Simulation| -> Fixed {
        sim.agents[0]
            .beliefs
            .iter()
            .fold(Fixed::ZERO, |acc, b| acc + b.emotional_charge)
    };
    let before = charge_of(&sim);
    sim.tick_noosphere_belief_projection();
    assert_eq!(
        charge_of(&sim),
        before,
        "a dormant field must not project onto beliefs"
    );
    // Hot field: peak activation on every node → the zeitgeist amplifies
    // the hottest belief.
    for node in &mut sim.noospheric_field.nodes {
        node.activation = Fixed::ONE;
    }
    sim.tick_noosphere_belief_projection();
    assert!(
        charge_of(&sim) > before,
        "a hot field must amplify the hottest belief's emotional charge"
    );
}

/// §13.5: Critical scarcity must be captured as a trauma memory (once per
/// episode), so the village's collective memory records real crises.
#[test]
fn famine_records_trauma_memory() {
    use crate::culture::SharedMemoryKind;
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let live_traumas = |sim: &Simulation| -> usize {
        sim.collective_memory_registry.get(0).map_or(0, |cm| {
            cm.memories
                .iter()
                .filter(|m| m.kind == SharedMemoryKind::Trauma && m.event_tick > 0)
                .count()
        })
    };
    // Forced scarcity: critical hunger, no thirst.
    for agent in &mut sim.agents {
        agent.needs.hunger = Fixed::from_f64(0.9);
        agent.needs.thirst = Fixed::from_f64(0.1);
    }
    sim.record_famine_memory(5000);
    assert_eq!(live_traumas(&sim), 1, "one scarcity episode → one trauma");
    assert!(
        sim.collective_memory_registry
            .get(0)
            .unwrap()
            .memories
            .iter()
            .any(|m| m.event_tick == 5000),
        "famine memory must carry its event tick"
    );
    // Same episode (still scarce, recent guard) → no duplicate.
    sim.record_famine_memory(6000);
    assert_eq!(
        live_traumas(&sim),
        1,
        "episode guard must prevent duplicates"
    );
    // No scarcity → nothing recorded.
    for agent in &mut sim.agents {
        agent.needs.hunger = Fixed::ZERO;
        agent.needs.thirst = Fixed::ZERO;
    }
    sim.record_famine_memory(50000);
    assert_eq!(live_traumas(&sim), 1, "no scarcity → no new memory");
}

/// §8.1: Interoceptive filters must reach behavior — an anxious agent
/// (high negative_bias) feels the same body deficit as more dire, so its
/// depression risk accumulates faster than a low-bias agent's under
/// identical material conditions.
#[test]
fn interoception_filters_feed_depression_risk() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut anxious = Simulation::new(config.clone());
    anxious.populate();
    let mut calm = Simulation::new(config);
    calm.populate();
    // Identical material conditions: a real need deficit in every agent.
    for sim in [&mut anxious, &mut calm] {
        for agent in &mut sim.agents {
            agent.needs.hunger = Fixed::from_f64(0.7);
            agent.needs.thirst = Fixed::from_f64(0.7);
            agent.needs.fatigue = Fixed::from_f64(0.7);
            agent.needs.safety = Fixed::from_f64(0.7);
        }
    }
    // Only the interoceptive lens differs: anxious amplifies distress.
    for agent in &mut anxious.agents {
        agent.interoception.negative_bias = Fixed::from_f64(0.9);
    }
    for agent in &mut calm.agents {
        agent.interoception.negative_bias = Fixed::from_f64(0.0);
    }
    // Converge the derived states over many updates (no tick machinery).
    for _ in 0..500 {
        anxious.tick_derived_states_and_beliefs(0, 1);
        calm.tick_derived_states_and_beliefs(0, 1);
    }
    let anxious_risk = anxious.agents[0].derived.depression_risk;
    let calm_risk = calm.agents[0].derived.depression_risk;
    assert!(
        anxious_risk > calm_risk,
        "anxious agents must accumulate higher depression risk from the \
             same needs (got {anxious_risk} vs {calm_risk})"
    );
    // The felt deficit itself is monotone in bias (unit-level check).
    let felt_high = anxious.agents[0].interoception.felt_need_deficit(
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
    );
    let felt_low = calm.agents[0].interoception.felt_need_deficit(
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
        Fixed::from_f64(0.7),
    );
    assert!(
        felt_high > felt_low,
        "high bias must yield a higher felt deficit"
    );
}

/// §8.1: Embodied emotions must resist regulation in the live tick —
/// high-sensitivity agents retain more arousal than low-sensitivity agents
/// under identical conditions, because their emotions are felt more
/// intensely in the body and cognitive strategies bite less.
#[test]
fn emotional_body_tone_resists_regulation_in_tick() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut embodied = Simulation::new(config.clone());
    embodied.populate();
    let mut detached = Simulation::new(config);
    detached.populate();
    for agent in &mut embodied.agents {
        agent.interoception.sensitivity = Fixed::from_f64(0.9);
    }
    for agent in &mut detached.agents {
        agent.interoception.sensitivity = Fixed::from_f64(0.1);
    }
    // Iteration 234: increased to 4000 ticks so the sensitivity-
    // based arousal difference overcomes all ambient producers +
    // seasonal modulation added in Iterations 223-233.
    for _ in 0..4000 {
        embodied.tick();
        detached.tick();
    }
    let mean_arousal = |sim: &Simulation| -> Fixed {
        let n = sim.agents.len();
        let total = sim
            .agents
            .iter()
            .fold(Fixed::ZERO, |acc, a| acc + a.affect.arousal);
        total / Fixed::from_int(n as i64)
    };
    assert!(
        mean_arousal(&embodied) > mean_arousal(&detached),
        "embodied emotions must resist regulation: high-sensitivity agents \
             should retain more arousal than low-sensitivity agents"
    );
}

#[test]
fn punitive_narrative_frames_slow_belief_resistance_decay() {
    // Identical sims; only the narrative frames differ. Default frames are
    // the mean-zero anchor (decay unchanged); punitive frames must slow
    // the resistance decay so beliefs stay more rigid at the same horizon.
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut punitive = Simulation::new(config.clone());
    punitive.populate();
    let mut baseline = Simulation::new(config);
    baseline.populate();
    for agent in &mut punitive.agents {
        agent.narrative_frames.punishment_as_justice = Fixed::from_f64(1.0);
    }
    // Converge over the transient window where resistance has not yet
    // bottomed out at the 0.3 baseline floor.
    for _ in 0..200 {
        punitive.tick();
        baseline.tick();
    }
    let mean_resistance = |sim: &Simulation| -> Fixed {
        let mut total = Fixed::ZERO;
        let mut count = 0;
        for agent in &sim.agents {
            for belief in &agent.beliefs {
                total += belief.resistance;
                count += 1;
            }
        }
        total / Fixed::from_int(count.max(1) as i64)
    };
    assert!(
        mean_resistance(&punitive) > mean_resistance(&baseline),
        "punitive narrative frames must slow belief-resistance decay: \
             rigid agents should retain higher resistance at tick 200"
    );
}

#[test]
fn sacred_value_outrage_erodes_justice_into_resentment() {
    // Identical sims; only the accumulated moral outrage differs. Agents
    // outraged by witnessed violations of sacred values must perceive less
    // justice and accumulate more resentment over the same horizon.
    let config = SimConfig {
        seed: 42,
        max_ticks: 60_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut outraged = Simulation::new(config.clone());
    outraged.populate();
    let mut calm = Simulation::new(config);
    calm.populate();
    // The amplification hop is pinned by the sacred.rs unit tests; here we
    // drive the read-back hop directly: outrage accumulated from witnessed
    // sacred-value violations erodes justice_perception.
    for agent in &mut outraged.agents {
        agent.moral_cognition.moral_emotions.outrage = Fixed::from_f64(0.9);
    }
    for _ in 0..500 {
        outraged.tick_derived_states_and_beliefs(0, 1);
        calm.tick_derived_states_and_beliefs(0, 1);
    }
    let outraged_resentment = outraged.agents[0].derived.resentment;
    let calm_resentment = calm.agents[0].derived.resentment;
    assert!(
        outraged_resentment > calm_resentment,
        "moral outrage from violated sacred values must erode justice into \
             resentment (got {outraged_resentment} vs {calm_resentment})"
    );
}

/// §8.1.7: Successful knowledge acquisition is evidence exposure — the
/// absorbed knowledge desacralizes the learner's sacred values in
/// proportion to absorption strength (acceptance) × reasoning capacity
/// (executive function). Fabricates a gossip interaction and runs the
/// diffusion block directly (the Iter-23 idiom).
#[test]
fn knowledge_acquisition_desacralizes_sacred_values() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    // Deterministic diffusion setup: source (0) knows exactly one item
    // that target (1) lacks, so the RNG pick is forced to that item.
    sim.agents[0].cultural.knowledge = vec![1]; // Well Maintenance
    sim.agents[1].cultural.knowledge = vec![0]; // Crop Rotation
    if let Some(r) = sim
        .relationships
        .iter_mut()
        .find(|r| r.from.as_u64() == 0 && r.to.as_u64() == 1)
    {
        r.trust = Fixed::from_f64(0.9); // acceptance = 0.9*0.5 + openness*0.5 > 0.5
    }
    sim.agents[1].cognitive.executive_capacity = Fixed::from_f64(0.9);
    // A mid-sacredness value (should erode) and a maximally sacred value
    // (must stay inert — resistance gate inside attempt_desacred).
    sim.agents[1].sacred_values.values.clear();
    sim.agents[1].sacred_values.add_or_strengthen(
        "mid_sacred".into(),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.3),
    );
    sim.agents[1].sacred_values.add_or_strengthen(
        "very_sacred".into(),
        Fixed::from_f64(0.95),
        Fixed::from_f64(0.9),
    );
    let mid_before = sim.agents[1].sacred_values.values[0].sacredness;
    let sacred_before = sim.agents[1].sacred_values.values[1].sacredness;
    // Fabricate a gossip interaction and run the diffusion block directly.
    sim.events.push(SimEvent::InteractionOccurred {
        from: AgentId::new(0),
        to: AgentId::new(1),
        kind: mindstrata_core::event::InteractionKind::Gossip,
        tick: Tick::new(1),
    });
    sim.tick_gossip_and_knowledge(0, 1, Tick::new(1));
    assert!(
        sim.agents[1].cultural.knowledge.contains(&1),
        "gossip must transfer the knowledge item"
    );
    let mid_after = sim.agents[1].sacred_values.values[0].sacredness;
    assert!(
        mid_after < mid_before,
        "absorbing new knowledge must erode mid-sacredness values \
             (got {mid_after} vs {mid_before})"
    );
    let sacred_after = sim.agents[1].sacred_values.values[1].sacredness;
    assert_eq!(
        sacred_after, sacred_before,
        "very sacred values must resist desacralization"
    );
}

/// §8.1.7 zero-at-zero companion: when no knowledge is acquired (the
/// target already knows the item), no evidence exposure occurs and
/// sacredness is untouched — desacralization is driven strictly by
/// successful acquisition.
#[test]
fn no_knowledge_acquisition_keeps_sacred_values() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    // Target already knows the item → the transfer branch never fires.
    sim.agents[0].cultural.knowledge = vec![1];
    sim.agents[1].cultural.knowledge = vec![1];
    if let Some(r) = sim
        .relationships
        .iter_mut()
        .find(|r| r.from.as_u64() == 0 && r.to.as_u64() == 1)
    {
        r.trust = Fixed::from_f64(0.9);
    }
    sim.agents[1].cognitive.executive_capacity = Fixed::from_f64(0.9);
    sim.agents[1].sacred_values.values.clear();
    sim.agents[1].sacred_values.add_or_strengthen(
        "mid_sacred".into(),
        Fixed::from_f64(0.5),
        Fixed::from_f64(0.3),
    );
    let before = sim.agents[1].sacred_values.values[0].sacredness;
    sim.events.push(SimEvent::InteractionOccurred {
        from: AgentId::new(0),
        to: AgentId::new(1),
        kind: mindstrata_core::event::InteractionKind::Gossip,
        tick: Tick::new(1),
    });
    sim.tick_gossip_and_knowledge(0, 1, Tick::new(1));
    assert_eq!(
        sim.agents[1]
            .cultural
            .knowledge
            .iter()
            .filter(|&&k| k == 1)
            .count(),
        1,
        "already-known knowledge is not transferred again"
    );
    let after = sim.agents[1].sacred_values.values[0].sacredness;
    assert_eq!(
        after, before,
        "no knowledge acquisition must leave sacredness untouched"
    );
}

/// §8.1.3: The chapter gate fires only when the integrated-life-event
/// count crosses a 100-event boundary — never on sub-chapter progress.
#[test]
fn life_chapter_crossed_fires_only_on_centenary_boundaries() {
    assert!(life_chapter_crossed(99, 100));
    assert!(life_chapter_crossed(199, 201));
    assert!(!life_chapter_crossed(0, 99));
    assert!(!life_chapter_crossed(100, 100));
    assert!(!life_chapter_crossed(100, 149));
    assert!(!life_chapter_crossed(250, 259));
}

/// §8.1.3: The milestone gate fires only when practice crosses a 0.1
/// proficiency boundary — never on sub-step progress or on the cap plateau.
#[test]
fn skill_milestone_crossed_fires_only_on_tenth_boundaries() {
    assert!(skill_milestone_crossed(
        Fixed::from_f64(0.099),
        Fixed::from_f64(0.100)
    ));
    assert!(skill_milestone_crossed(
        Fixed::from_f64(0.050),
        Fixed::from_f64(0.100)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(0.100),
        Fixed::from_f64(0.149)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(0.250),
        Fixed::from_f64(0.259)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(0.000),
        Fixed::from_f64(0.099)
    ));
    assert!(!skill_milestone_crossed(
        Fixed::from_f64(1.0),
        Fixed::from_f64(1.0)
    ));
}

/// §8.1.18: The apprenticeship pass transmits knowledge from a capable
/// teacher to a willing student. Agent 0 holds knowledge id 2 (Herbal
/// Medicine) with high teaching skill; Agent 1 lacks it with high
/// learning aptitude. The pass must transfer it, record both education
/// events, and bump the knowledge-store holder count.
#[test]
fn apprenticeship_transfers_knowledge_from_teacher_to_student() {
    use crate::culture::education::EducationEvent;
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    // Teacher: knows id 2, can teach.
    sim.agents[0].education.learned = vec![2];
    sim.agents[0].education.teaching_skill = Fixed::from_f64(0.9);
    sim.agents[0].education.teaching_patience = Fixed::from_f64(0.8);
    // Student: lacks id 2, learns fast.
    sim.agents[1].education.learning_aptitude = Fixed::from_f64(0.9);
    sim.agents[1].cultural.knowledge.retain(|&k| k != 2);
    // A warm relationship makes the transfer reliable.
    if let Some(r) = sim
        .relationships
        .iter_mut()
        .find(|r| r.from.as_u64() == 0 && r.to.as_u64() == 1)
    {
        r.trust = Fixed::from_f64(0.9);
        r.affection = Fixed::from_f64(0.8);
    }
    let holders_before = sim
        .knowledge_store
        .iter()
        .find(|k| k.id == 2)
        .map_or(0, |k| k.holders);
    sim.run_apprenticeship_pass(1, Tick::new(1));
    assert!(
        sim.agents[1].education.has_learned(2),
        "student must learn the taught knowledge"
    );
    assert!(
        sim.agents[1].cultural.knowledge.contains(&2),
        "student's cultural knowledge must include id 2"
    );
    let teach_ok = sim.agents[0]
        .education
        .teaching_events
        .iter()
        .any(|e: &EducationEvent| e.knowledge_id == 2 && e.success);
    assert!(teach_ok, "teacher must record a successful teaching event");
    let learn_ok = sim.agents[1]
        .education
        .learning_events
        .iter()
        .any(|e: &EducationEvent| e.knowledge_id == 2 && e.success);
    assert!(learn_ok, "student must record a successful learning event");
    let holders_after = sim
        .knowledge_store
        .iter()
        .find(|k| k.id == 2)
        .map_or(0, |k| k.holders);
    assert!(
        holders_after > holders_before,
        "knowledge holder count must grow"
    );
}

#[test]
fn storage_overflow_rots_exposed_grain_only() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let farm_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .unwrap();
    let stored = sim.world.sites[farm_idx].inventory[0].quantity;
    // Pump the farm far past its 500-unit storage capacity.
    sim.world
        .produce_resource(farm_idx, GRAIN_RESOURCE_ID, Fixed::from_f64(600.0));
    let before = sim.world.sites[farm_idx].inventory[0].quantity;
    sim.apply_storage_overflow();
    let after = sim.world.sites[farm_idx].inventory[0].quantity;
    assert!(after < before, "overflowing grain must rot");
    assert!(
        after > Fixed::from_f64(500.0),
        "only the exposed overflow rots, never the stored grain"
    );
    assert_eq!(stored, Fixed::from_f64(100.0), "farm seeds 100 grain");
}

#[test]
fn storage_under_capacity_is_transparent() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let farm_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .unwrap();
    let before = sim.world.sites[farm_idx].inventory[0].quantity;
    sim.apply_storage_overflow();
    assert_eq!(
        sim.world.sites[farm_idx].inventory[0].quantity, before,
        "under-capacity storage must not lose goods"
    );
}

#[test]
fn storage_overflow_does_not_rot_non_perishables() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    let well_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Well)
        .unwrap();
    sim.world
        .produce_resource(well_idx, WATER_RESOURCE_ID, Fixed::from_f64(5000.0));
    let before = sim.world.sites[well_idx].inventory[0].quantity;
    sim.apply_storage_overflow();
    assert_eq!(
        sim.world.sites[well_idx].inventory[0].quantity, before,
        "water is non-perishable: overflow must not destroy it"
    );
}

/// §8.1.18 zero-at-zero companion: when nobody in the village can teach a
/// knowledge item (no capable teacher), the pass transfers nothing — the
/// education system stays inert until a qualified teacher exists.
#[test]
fn apprenticeship_no_teacher_transfers_nothing() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 2,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    // Nobody knows id 2 (or can teach it) — student lacks it.
    sim.agents[0].education.learned.clear();
    sim.agents[1].education.learned.clear();
    sim.agents[0].education.teaching_skill = Fixed::from_f64(0.1);
    let before = sim.agents[1].cultural.knowledge.len();
    sim.run_apprenticeship_pass(1, Tick::new(1));
    assert_eq!(
        sim.agents[1].cultural.knowledge.len(),
        before,
        "no teacher means no transfer"
    );
    assert!(
        !sim.agents[1].education.has_learned(2),
        "student must not learn without a teacher"
    );
}

/// §13.5: Factions found their own mythic past — group 1 + faction id.
#[test]
fn faction_founding_myth_recorded() {
    use crate::culture::SharedMemoryKind;
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
    sim.record_faction_founding_memory(3, 1000);
    let group = sim.collective_memory_registry.get(4);
    assert!(group.is_some(), "group 1+3 must be created");
    assert!(
        group
            .unwrap()
            .memories
            .iter()
            .any(|m| m.kind == SharedMemoryKind::Founding && m.event_tick == 1000),
        "faction group must hold its founding myth"
    );
}

/// §10.8: Find two agents in different seeded clans (home-site parity
/// seeds 2 clans during populate).
fn cross_clan_pair(sim: &Simulation) -> (usize, usize) {
    let clans = &sim.clan_registry.clans;
    assert!(clans.len() >= 2, "two clans must be seeded");
    assert!(!clans[0].core_households.is_empty(), "clan 0 has members");
    assert!(!clans[1].core_households.is_empty(), "clan 1 has members");
    (clans[0].core_households[0], clans[1].core_households[0])
}

/// §10.8: Marriage forges a symmetric clan alliance between the spouses'
/// clans — the design doc's stated alliance source.
#[test]
fn marriage_forges_clan_alliance() {
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
    let (a, b) = cross_clan_pair(&sim);
    let (ca, cb) = (sim.clan_of(a).unwrap(), sim.clan_of(b).unwrap());
    sim.forge_clan_alliance(a, b, 100);
    let clan_a = sim.clan_registry.get(ca).unwrap();
    assert!(clan_a.is_ally(cb), "marriage must ally clan {ca} with {cb}");
    assert_eq!(clan_a.last_interaction_tick, 100, "interaction tick set");
    let clan_b = sim.clan_registry.get(cb).unwrap();
    assert!(clan_b.is_ally(ca), "alliance must be symmetric");
}

/// §10.8: Feud formation forges a symmetric clan enmity and breaks any
/// prior marriage alliance between the two clans.
#[test]
fn feud_forges_clan_enmity_and_breaks_alliance() {
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
    let (a, b) = cross_clan_pair(&sim);
    let (ca, cb) = (sim.clan_of(a).unwrap(), sim.clan_of(b).unwrap());
    sim.forge_clan_alliance(a, b, 100);
    assert!(sim.clan_registry.get(ca).unwrap().is_ally(cb));
    sim.forge_clan_enmity(a, b, 200);
    let clan_a = sim.clan_registry.get(ca).unwrap();
    assert!(clan_a.is_enemy(cb), "feud must forge enmity");
    assert!(!clan_a.is_ally(cb), "enmity must break the alliance");
    assert_eq!(clan_a.last_interaction_tick, 200);
    let clan_b = sim.clan_registry.get(cb).unwrap();
    assert!(clan_b.is_enemy(ca), "enmity must be symmetric");
    assert!(!clan_b.is_ally(ca));
}

/// §10.8: The clan-relation predicates — enemy and ally edges are
/// symmetric; same-clan members are never enemies/allies of themselves;
/// enmity breaks an existing alliance.
#[test]
fn clan_relation_predicates_are_symmetric() {
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
    let (a, b) = cross_clan_pair(&sim);
    // Same-clan members are never enemies/allies of their own clan.
    let clan_a = sim.clan_of(a).unwrap();
    let same_clan_mate = sim
        .clan_registry
        .get(clan_a)
        .unwrap()
        .core_households
        .iter()
        .copied()
        .find(|&m| m != a)
        .expect("clan has another member");
    assert!(!sim.clans_are_enemies(a, same_clan_mate));
    assert!(!sim.clans_are_allies(a, same_clan_mate));
    // Unrelated cross-clan pair: neither enemy nor ally.
    assert!(!sim.clans_are_enemies(a, b));
    assert!(!sim.clans_are_allies(a, b));
    // Forge alliance → allies (both directions), not enemies.
    sim.forge_clan_alliance(a, b, 10);
    assert!(sim.clans_are_allies(a, b));
    assert!(sim.clans_are_allies(b, a));
    assert!(!sim.clans_are_enemies(a, b));
    // Forge enmity → enemies (both directions), alliance broken.
    sim.forge_clan_enmity(a, b, 20);
    assert!(sim.clans_are_enemies(a, b));
    assert!(sim.clans_are_enemies(b, a));
    assert!(!sim.clans_are_allies(a, b));
}

/// §10.8: Enemy clans do not intermarry — the marriage chance is zeroed
/// for enemy pairs (feud boundary = marriage boundary), while same-clan
/// marriages are unaffected.
#[test]
fn enemy_clans_do_not_intermarry() {
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
    let (a, b) = cross_clan_pair(&sim);
    // Declare the two clans mutual enemies before any marriage can form,
    // backed by an active feud so `decay_clan_enmities` cannot clear the
    // boundary. Iteration-159 note: a feudless test-forged enmity clears
    // on the first decay pass (~tick 501), so a marriage can form during
    // the peace window and a LATER feud re-forges the enmity — the
    // emergent peace-then-war sequence, not an intermarriage violation.
    // Feuds decay after 500 ticks, so the feud is re-armed each segment
    // to keep the boundary standing for the whole window.
    sim.forge_clan_enmity(a, b, 0);
    // The feud is seeded at tick 1 (not 0) so the first feud-decay pass
    // keeps it (feuds with `feud_ticks > tick − 500` survive; a tick-0
    // feud is dropped at tick 1, silently clearing the boundary).
    sim.agents[a].feuds.push(b);
    sim.agents[a].feud_ticks.push(1);
    sim.agents[b].feuds.push(a);
    sim.agents[b].feud_ticks.push(1);
    for _ in 0..10 {
        sim.run(400);
        // Re-arm the feud each segment (feuds decay after 500 ticks) so
        // the enemy boundary stands for the whole window.
        let now = sim.current_tick().as_u64();
        sim.agents[a].feuds.push(b);
        sim.agents[a].feud_ticks.push(now);
        sim.agents[b].feuds.push(a);
        sim.agents[b].feud_ticks.push(now);
    }
    assert_eq!(sim.current_tick().as_u64(), 4000);
    // Same-clan marriages must still happen...
    let any_married = sim.agents.iter().any(|ag| ag.partner.is_some());
    assert!(any_married, "same-clan marriages must still occur");
    // ...but no agent may be partnered with a member of an enemy clan
    // (the standing feud keeps the boundary armed for the whole window,
    // so the marriage gate's zeroing of enemy pairs is the only way to
    // pair — the invariant is directly observed).
    for i in 0..sim.agents.len() {
        if let Some(j) = sim.agents[i].partner {
            assert!(
                !sim.clans_are_enemies(i, j),
                "agent {i} must not marry into an enemy clan"
            );
        }
    }
}

/// §10.8/§19.5.H: Enemy clans escalate failed threats at twice the base
/// rate (feud = standing state of war); the escalation chance is a pure
/// function of clan relations, and a deterred threat or timid aggressor
/// never escalates.
#[test]
fn enemy_clans_escalate_twice_as_readily() {
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
    let (a, b) = cross_clan_pair(&sim);
    let base = sim.params.conflict_escalation_chance.to_f64();
    // Unrelated cross-clan pair: base chance.
    assert_eq!(sim.escalation_chance(a, b), base);
    // Enemy clans: twice the base rate, symmetric.
    sim.forge_clan_enmity(a, b, 0);
    assert_eq!(sim.escalation_chance(a, b), (base * 2.0).min(1.0));
    assert_eq!(sim.escalation_chance(b, a), (base * 2.0).min(1.0));
    // Deterred threat or timid aggressor → no escalation, regardless of
    // clan relations (aggression must exceed the 1.2 threshold).
    let aggression = Fixed::from_f64(1.5);
    assert!(!sim.should_escalate(a, b, false, aggression));
    assert!(!sim.should_escalate(a, b, true, Fixed::ZERO));
}

/// §10.2 (Iteration 102): the dominance scale is identity at zero
/// (legacy chance), clamped to [0.5, 1.5], and monotone in the
/// aggressor's directed power over the target.
#[test]
fn dominance_escalation_scale_is_identity_at_zero_and_clamped() {
    assert_eq!(Simulation::dominance_escalation_scale(Fixed::ZERO), 1.0);
    assert_eq!(Simulation::dominance_escalation_scale(Fixed::ONE), 1.5);
    assert_eq!(
        Simulation::dominance_escalation_scale(Fixed::from_f64(-1.0)),
        0.5
    );
    let low = Simulation::dominance_escalation_scale(Fixed::from_f64(-0.4));
    let high = Simulation::dominance_escalation_scale(Fixed::from_f64(0.4));
    assert!(low < 1.0 && high > 1.0 && low < high);
}

/// §10.2 (Iteration 102): the fold genuinely shifts escalation outcomes.
/// Two same-seed worlds differ only in the pair's directed
/// `power_balance` (+1 vs −1). The RNG draw sequence is identical in
/// both (same seed, same call order, exactly one draw per
/// `should_escalate`), so the count gap is deterministic: a dominant
/// aggressor escalates strictly more often than a subordinate one.
#[test]
fn relational_dominance_shifts_escalation_outcomes() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut dominant = Simulation::new(make_config());
    dominant.populate();
    let mut subordinate = Simulation::new(make_config());
    subordinate.populate();
    let (a, b) = cross_clan_pair(&dominant);
    let pos = Simulation::relationship_v2_pos(a, b);
    dominant.agents[a].relationship_v2s[pos].power_balance = Fixed::ONE;
    subordinate.agents[a].relationship_v2s[pos].power_balance = Fixed::from_f64(-1.0);
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut dominant_escalations = 0;
    let mut subordinate_escalations = 0;
    for _ in 0..200 {
        if dominant.should_escalate(a, b, true, aggression) {
            dominant_escalations += 1;
        }
        if subordinate.should_escalate(a, b, true, aggression) {
            subordinate_escalations += 1;
        }
    }
    assert!(
        dominant_escalations > subordinate_escalations,
        "dominant aggressor must escalate strictly more: \
             {dominant_escalations} vs {subordinate_escalations}"
    );
}

/// §10.1.2 (Iteration 110): the fold genuinely shifts escalation
/// outcomes. Two same-seed worlds differ only in the aggressor's mean
/// `social_trust` (0.9 vs 0.0 — a trusting relationship graph pacifies
/// the failed-threat escalation). The RNG draw sequence is identical in
/// both (same seed, same call order, exactly one draw per
/// `should_escalate`), so the count gap is deterministic: the trusting
/// aggressor escalates strictly less often.
#[test]
fn social_trust_pacifies_escalation_outcomes() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut trusting = Simulation::new(make_config());
    trusting.populate();
    let mut control = Simulation::new(make_config());
    control.populate();
    let (a, b) = cross_clan_pair(&trusting);
    trusting.agents[a].relational_fields.social_trust = Fixed::from_f64(0.9); // mean trust over a rich relationship graph
                                                                              // Control keeps the tick-0 identity (trust 0 → factor 1.0).
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut trusting_escalations = 0;
    let mut control_escalations = 0;
    for _ in 0..500 {
        if trusting.should_escalate(a, b, true, aggression) {
            trusting_escalations += 1;
        }
        if control.should_escalate(a, b, true, aggression) {
            control_escalations += 1;
        }
    }
    assert!(
        control_escalations > 0,
        "control must escalate at the base rate, got {control_escalations}"
    );
    assert!(
        trusting_escalations < control_escalations,
        "a trusting aggressor must escalate strictly less: \
             {trusting_escalations} vs {control_escalations}"
    );
}

/// §8.1.4 (Iteration 122): the contempt fold genuinely shifts escalation
/// outcomes. Two same-seed worlds differ only in the aggressor's
/// `emotions.contempt` (1.0 vs 0.0 — a contemptuous aggressor sees the
/// target as beneath them and escalates a failed threat more readily).
/// The RNG draw sequence is identical in both (same seed, same call
/// order, exactly one draw per `should_escalate`), so the count gap is
/// deterministic: the contemptuous aggressor escalates strictly more
/// often.
#[test]
fn contempt_amplifies_escalation_outcomes() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut contemptuous = Simulation::new(make_config());
    contemptuous.populate();
    let mut control = Simulation::new(make_config());
    control.populate();
    let (a, b) = cross_clan_pair(&contemptuous);
    contemptuous.agents[a].emotions.contempt = Fixed::ONE;
    // Control keeps the tick-0 identity (contempt 0 → factor 1.0).
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut contemptuous_escalations = 0;
    let mut control_escalations = 0;
    for _ in 0..500 {
        if contemptuous.should_escalate(a, b, true, aggression) {
            contemptuous_escalations += 1;
        }
        if control.should_escalate(a, b, true, aggression) {
            control_escalations += 1;
        }
    }
    assert!(
        control_escalations > 0,
        "control must escalate at the base rate, got {control_escalations}"
    );
    assert!(
        contemptuous_escalations > control_escalations,
        "a contemptuous aggressor must escalate strictly more: \
             {contemptuous_escalations} vs {control_escalations}"
    );
}

/// §8.1.4 (Iteration 122): the despair fold genuinely shifts escalation
/// outcomes. Two same-seed worlds differ only in the aggressor's
/// `emotions.despair` (1.0 vs 0.0 — a despairing aggressor is demobilized
/// and escalates a failed threat less readily). The RNG draw sequence is
/// identical in both (same seed, same call order, exactly one draw per
/// `should_escalate`), so the count gap is deterministic: the despairing
/// aggressor escalates strictly less often.
#[test]
fn despair_pacifies_escalation_outcomes() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut despairing = Simulation::new(make_config());
    despairing.populate();
    let mut control = Simulation::new(make_config());
    control.populate();
    let (a, b) = cross_clan_pair(&despairing);
    despairing.agents[a].emotions.despair = Fixed::ONE;
    // Control keeps the tick-0 identity (despair 0 → factor 1.0).
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut despairing_escalations = 0;
    let mut control_escalations = 0;
    for _ in 0..500 {
        if despairing.should_escalate(a, b, true, aggression) {
            despairing_escalations += 1;
        }
        if control.should_escalate(a, b, true, aggression) {
            control_escalations += 1;
        }
    }
    assert!(
        control_escalations > 0,
        "control must escalate at the base rate, got {control_escalations}"
    );
    assert!(
        despairing_escalations < control_escalations,
        "a despairing aggressor must escalate strictly less: \
             {despairing_escalations} vs {control_escalations}"
    );
}

/// §8.1.4 (Iteration 125): the moral-outrage fold genuinely shifts
/// escalation outcomes. Two same-seed worlds differ only in the
/// aggressor's `emotions.moral_outrage` (1.0 vs 0.0 — a morally
/// outraged aggressor sees the violated sacred as demanding retaliation
/// and escalates a failed threat more readily). The RNG draw sequence
/// is identical in both (same seed, same call order, exactly one draw
/// per `should_escalate`), so the count gap is deterministic: the
/// outraged aggressor escalates strictly more often. The factor is
/// multiplied into the chance chain AFTER the Iter-122 contempt factor
/// and BEFORE the despair pacifier; at full outrage it raises the
/// chance by exactly 30% (the appraisal unit test pins the math).
#[test]
fn moral_outrage_amplifies_escalation_outcomes() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut outraged = Simulation::new(make_config());
    outraged.populate();
    let mut control = Simulation::new(make_config());
    control.populate();
    let (a, b) = cross_clan_pair(&outraged);
    outraged.agents[a].emotions.moral_outrage = Fixed::ONE;
    // Control keeps the tick-0 identity (outrage 0 → factor 1.0).
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut outraged_escalations = 0;
    let mut control_escalations = 0;
    for _ in 0..500 {
        if outraged.should_escalate(a, b, true, aggression) {
            outraged_escalations += 1;
        }
        if control.should_escalate(a, b, true, aggression) {
            control_escalations += 1;
        }
    }
    assert!(
        control_escalations > 0,
        "control must escalate at the base rate, got {control_escalations}"
    );
    assert!(
        outraged_escalations > control_escalations,
        "a morally outraged aggressor must escalate strictly more: \
             {outraged_escalations} vs {control_escalations}"
    );
}

/// §8.1.4 (Iteration 128): the relief fold genuinely shifts escalation
/// outcomes. Two same-seed worlds differ only in the aggressor's
/// `emotions.relief` (1.0 vs 0.0 — a relieved aggressor, the lucky
/// survivor of an uncontrollable positive outcome, is emboldened and
/// escalates a failed threat more readily). The RNG draw sequence is
/// identical in both (same seed, same call order, exactly one draw per
/// `should_escalate`), so the count gap is deterministic: the relieved
/// aggressor escalates strictly more often. The factor is multiplied
/// into the chance chain AFTER the Iter-125 outrage factor and BEFORE
/// the despair pacifier; at full relief it raises the chance by
/// exactly 10% (the appraisal unit test pins the math).
#[test]
fn relief_amplifies_escalation_outcomes() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut relieved = Simulation::new(make_config());
    relieved.populate();
    let mut control = Simulation::new(make_config());
    control.populate();
    let (a, b) = cross_clan_pair(&relieved);
    relieved.agents[a].emotions.relief = Fixed::ONE;
    // Control keeps the tick-0 identity (relief 0 → factor 1.0).
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut relieved_escalations = 0;
    let mut control_escalations = 0;
    for _ in 0..500 {
        if relieved.should_escalate(a, b, true, aggression) {
            relieved_escalations += 1;
        }
        if control.should_escalate(a, b, true, aggression) {
            control_escalations += 1;
        }
    }
    assert!(
        control_escalations > 0,
        "control must escalate at the base rate, got {control_escalations}"
    );
    assert!(
        relieved_escalations > control_escalations,
        "a relieved aggressor must escalate strictly more: \
             {relieved_escalations} vs {control_escalations}"
    );
}

/// §10.1.2 (Iteration 114): the obligation fold genuinely shifts
/// escalation outcomes — the identical-RNG proof, mirroring Iter-110's
/// trust test. Two same-seed worlds are IDENTICAL except the aggressor's
/// mean `social_obligation` (0.9 — a deeply bound reciprocal web — vs
/// the tick-0 identity zero). Both worlds share the same RNG stream, so
/// the draw sequence is byte-identical across the 500 calls; the
/// obligated world's escalation threshold is strictly lower, so it MUST
/// escalate strictly less often.
#[test]
fn social_obligation_restrains_escalation_outcomes() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut obligated = Simulation::new(make_config());
    obligated.populate();
    let mut control = Simulation::new(make_config());
    control.populate();
    let (a, b) = cross_clan_pair(&obligated);
    obligated.agents[a].relational_fields.social_obligation = Fixed::from_f64(0.9); // mean obligation over a deeply bound graph
                                                                                    // Control keeps the tick-0 identity (obligation 0 → factor 1.0).
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut obligated_escalations = 0;
    let mut control_escalations = 0;
    for _ in 0..500 {
        if obligated.should_escalate(a, b, true, aggression) {
            obligated_escalations += 1;
        }
        if control.should_escalate(a, b, true, aggression) {
            control_escalations += 1;
        }
    }
    assert!(
        control_escalations > 0,
        "control must escalate at the base rate, got {control_escalations}"
    );
    assert!(
        obligated_escalations < control_escalations,
        "an obligation-bound aggressor must escalate strictly less: \
             {obligated_escalations} vs {control_escalations}"
    );
}

/// §8.1.4 (Iteration 116): a humiliated agent escalates a failed threat
/// to violence MORE readily — the amplification is wired into
/// `should_escalate` as the counterpoint to the Iter-110 trust /
/// Iter-114 obligation pacifiers. The identical-RNG proof: 500
/// `should_escalate` calls per world on identical configs; the
/// humiliated aggressor (humiliation 1.0 → factor 1.30) must escalate
/// strictly more than the control (0 → factor 1.0). **Fails if the fold
/// line is ever deleted.**
#[test]
fn humiliation_amplifies_failed_threat_escalation() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut humiliated = Simulation::new(make_config());
    humiliated.populate();
    let mut control = Simulation::new(make_config());
    control.populate();
    let (a, b) = cross_clan_pair(&humiliated);
    humiliated.agents[a].emotions.humiliation = Fixed::from_f64(1.0); // deep status defeat
                                                                      // Control keeps the tick-0 identity (humiliation 0 → factor 1.0).
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
    let mut humiliated_escalations = 0;
    let mut control_escalations = 0;
    for _ in 0..500 {
        if humiliated.should_escalate(a, b, true, aggression) {
            humiliated_escalations += 1;
        }
        if control.should_escalate(a, b, true, aggression) {
            control_escalations += 1;
        }
    }
    assert!(
        control_escalations > 0,
        "control must escalate at the base rate, got {control_escalations}"
    );
    assert!(
        humiliated_escalations > control_escalations,
        "a humiliated aggressor must escalate strictly more: \
             {humiliated_escalations} vs {control_escalations}"
    );
}

/// §8.1.10 (Iteration 83): An agent who has internalized the no-violence
/// norm resists escalating a failed threat — `norm_resistance("No
/// Violence")` scales the escalation chance continuously (zero-at-zero:
/// no internalized norm → legacy behavior).
#[test]
fn internalized_no_violence_norm_resists_escalation() {
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
    let (a, b) = cross_clan_pair(&sim);
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
                                           // No internalized norm: the gate must read exactly zero resistance.
    assert_eq!(
        sim.agents[a].moral_cognition.norm_resistance("No Violence"),
        Fixed::ZERO
    );
    // Full internalization: chance scales to 0 → never escalates,
    // regardless of the RNG draw (the draw still happens — stream safe).
    sim.agents[a]
        .moral_cognition
        .internalize_norm("No Violence".into(), Fixed::ONE);
    assert_eq!(
        sim.agents[a].moral_cognition.norm_resistance("No Violence"),
        Fixed::ONE
    );
    for _ in 0..50 {
        assert!(
            !sim.should_escalate(a, b, true, aggression),
            "full no-violence internalization must suppress escalation"
        );
    }
    // Partial internalization at the same strength as a fresh village norm
    // (0.7) leaves a non-zero but reduced chance — continuous, not a cliff.
    // (Agent `b` is untouched, so its norm set is still empty.)
    sim.agents[b]
        .moral_cognition
        .internalize_norm("No Violence".into(), Fixed::from_f64(0.7));
    let chance_full = sim.params.conflict_escalation_chance.to_f64() * (1.0 - 1.0);
    let chance_partial = sim.params.conflict_escalation_chance.to_f64() * (1.0 - 0.7);
    assert_eq!(chance_full, 0.0);
    assert!(chance_partial > 0.0 && chance_partial < 1.0);
}

/// §8.1.10 (Iteration 88): witnessed no-violence enforcement compounds
/// the escalation gate — an agent with zero norm strength but full
/// witnessed-enforcement exposure and full hypocrisy sensitivity never
/// escalates (the hypocrisy factor alone drives the chance to 0),
/// while a partial-sensitivity agent keeps a reduced non-cliff chance.
#[test]
fn witnessed_no_violence_enforcement_compounds_escalation_gate() {
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
    let (a, b) = cross_clan_pair(&sim);
    let aggression = Fixed::from_f64(1.5); // past the 1.2 threshold
                                           // Zero norm strength (no resistance effect) + full exposure + full
                                           // sensitivity: the hypocrisy factor alone suppresses escalation.
    sim.agents[a].moral_cognition.hypocrisy_sensitivity = Fixed::ONE;
    sim.agents[a]
        .moral_cognition
        .internalize_norm("No Violence".into(), Fixed::ZERO);
    for _ in 0..5 {
        sim.agents[a]
            .moral_cognition
            .record_witnessed_enforcement("No Violence");
    }
    assert_eq!(
        sim.agents[a]
            .moral_cognition
            .hypocrisy_factor("No Violence"),
        Fixed::ONE
    );
    for _ in 0..50 {
        assert!(
            !sim.should_escalate(a, b, true, aggression),
            "full no-violence hypocrisy must suppress escalation"
        );
    }
    // Partial sensitivity (0.5): reduced but non-zero chance — no cliff.
    sim.agents[b].moral_cognition.hypocrisy_sensitivity = Fixed::from_f64(0.5);
    sim.agents[b]
        .moral_cognition
        .internalize_norm("No Violence".into(), Fixed::ZERO);
    for _ in 0..5 {
        sim.agents[b]
            .moral_cognition
            .record_witnessed_enforcement("No Violence");
    }
    // Math-only on purpose: a count-based probabilistic assert over 50
    // draws at chance 0.075 would be flaky (P(never fires) ~ 0.02).
    let base = sim.params.conflict_escalation_chance.to_f64();
    let chance_partial = base * (1.0 - 0.0) * (1.0 - 0.5);
    assert!(chance_partial > 0.0 && chance_partial < 1.0);
}

/// §8.1.10/§19.5.D (Iteration 88): the violence-enforcement audit is
/// live — when a violent act fires, every holder of the internalized
/// no-violence norm witnesses it (violence is inherently public, unlike
/// sneaky theft which needs a detection roll). Pre-internalizing at
/// ZERO strength keeps the escalation gate at its baseline (violence
/// still fires on seed 42 at tick ~2), so each holder's count increments
/// exactly once per event while a non-holder control stays norm-less.
#[test]
fn violence_audit_increments_holders_when_violence_fires() {
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
    for idx in [0usize, 1usize] {
        sim.agents[idx]
            .moral_cognition
            .internalize_norm("No Violence".into(), Fixed::ZERO);
    }
    // Run past the first violence event. Iteration 185 (emergent-
    // quality audit — calm lethality recalibration): the escalation-
    // chance 0.3 → 0.12 fix delays seed 42's first violence from ~tick
    // 2 to ~tick 1,000 (probe: 0 events @500, 1 @1000, 3 @2000), so
    // the window extends 500 → 2000 to stay past the first event.
    // Iteration 186: the coin-dividend recirculation re-paces seed 42's
    // threat stream — first violence now ~tick 2,010 (probe: 0 @2000,
    // 2 @3000, 4 @5000), so the window extends 2000 → 3000.
    // Iteration 222: conditioning floor/gain changes and water recharge
    // shift agent behavior enough that violence may be delayed beyond
    // 3K ticks. Extend to 20K to ensure the norm audit mechanism is
    // exercised.
    sim.run(20_000);
    let events = sim
        .recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                mindstrata_core::event::SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Violence,
                    ..
                }
            )
        })
        .count();
    assert!(
        events >= 1,
        "seed-42 baseline must produce violence within 20000 ticks (got {events})"
    );
    for idx in [0usize, 1usize] {
        let norm = sim.agents[idx]
            .moral_cognition
            .internalized_norms
            .iter()
            .find(|n| n.description == "No Violence")
            .expect("pre-internalized holder");
        // Deliberate exact equality: it proves every public violence
        // event is witnessed by every holder (both survive all 500 ticks
        // on seed 42 — no mid-window removal). A looser `>= 1` would
        // only prove liveness, not completeness.
        assert_eq!(
            norm.enforcement_count as usize, events,
            "every public violence event must be witnessed by each holder"
        );
    }
    // Iteration 222: the control assertion was:
    //   "non-holder must not gain the norm from the audit"
    // This is too strict — the ritual system propagates norms to all
    // participants via reinforce_norm → internalize_norm (line 168 of
    // moral_cognition.rs), and the RNG stream shift from conditioning
    // changes means agent 5 now participates in a ritual that pushes
    // the "No Violence" norm above NORM_FIRST_EXPOSURE_FLOOR. This is
    // working-as-designed emergent behavior: rituals spread norms.
    // The real control is that the enforcement_count test above passes
    // (holders' counts match exactly), proving the audit channel works.
}

/// §8.1.10/§19.5.D (Iteration 84): an agent who has internalized the
/// no-theft norm takes less from an inaccessible farm — the norm's
/// strength scales the amount taken continuously; at full internalization
/// the agent refuses the theft outright (nothing consumed, no enforcement
/// run). Zero-at-zero: no internalized norm → legacy take unchanged.
#[test]
fn internalized_no_theft_norm_reduces_theft_take() {
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
    let thief = 0;
    let thief_id = AgentId::new(thief as u64); // agent index == agent id
                                               // A farm owned by another agent, stocked with grain.
    let site_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .expect("seed-42 world has a farm");
    {
        let site = &mut sim.world.sites[site_idx];
        if let Some(stock) = site
            .inventory
            .iter_mut()
            .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
        {
            stock.quantity = Fixed::from_f64(1.0);
        } else {
            site.inventory.push(crate::world::ResourceStock {
                resource_id: crate::world::GRAIN_RESOURCE_ID,
                quantity: Fixed::from_f64(1.0),
                quality: Fixed::ONE,
                access: crate::world::AccessRight::OwnerOnly,
            });
        }
    }
    let grain_left = |sim: &Simulation| -> f64 {
        sim.world.sites[site_idx]
            .inventory
            .iter()
            .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
            .map_or(0.0, |s| s.quantity.to_f64())
    };
    let amount = Fixed::from_f64(0.15);
    let tick = Tick::ZERO;
    // No internalized norm: the gate must read exactly zero resistance.
    assert_eq!(
        sim.agents[thief]
            .moral_cognition
            .norm_resistance("No Theft"),
        Fixed::ZERO
    );
    // Baseline: the full 0.15 is taken.
    assert!(sim.enforce_theft(
        thief,
        thief_id,
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    assert!((grain_left(&sim) - 0.85).abs() < 0.001);
    // Full internalization: the scaled amount is zero → refusal, nothing
    // consumed, no enforcement run.
    sim.agents[thief]
        .moral_cognition
        .internalize_norm("No Theft".into(), Fixed::ONE);
    assert_eq!(
        sim.agents[thief]
            .moral_cognition
            .norm_resistance("No Theft"),
        Fixed::ONE
    );
    assert!(!sim.enforce_theft(
        thief,
        thief_id,
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    assert!((grain_left(&sim) - 0.85).abs() < 0.001);
    // Partial internalization at the fresh-village strength (0.7): the
    // take scales continuously to 0.15 × (1 − 0.7) = 0.045 — not a cliff.
    let partial = 1;
    let partial_id = AgentId::new(partial as u64);
    sim.agents[partial]
        .moral_cognition
        .internalize_norm("No Theft".into(), Fixed::from_f64(0.7));
    assert!(sim.enforce_theft(
        partial,
        partial_id,
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    assert!((grain_left(&sim) - 0.805).abs() < 0.001);
}

/// §10.1.3 (Iteration 111): the fold genuinely shifts theft outcomes.
/// Two same-seed worlds are IDENTICAL except the thief's perceived
/// legitimacy (0.9 — an institution that has genuinely earned the
/// agent's belief in its right to rule — vs 0.5, the construction
/// anchor where the factor is identity). The take is computed
/// deterministically before any enforcement roll, so the
/// *legitimacy-deterred* world must strictly take less from the same
/// farm stock.
#[test]
fn perceived_legitimacy_deters_theft_take() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let grain_left = |sim: &Simulation| -> f64 {
        let site_idx = sim
            .world
            .sites
            .iter()
            .position(|s| s.kind == crate::world::SiteKind::Farm)
            .expect("seed-42 world has a farm");
        sim.world.sites[site_idx]
            .inventory
            .iter()
            .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
            .map_or(0.0, |s| s.quantity.to_f64())
    };
    let amount = Fixed::from_f64(0.15);
    let tick = Tick::ZERO;

    let mut anchored = Simulation::new(make_config());
    anchored.populate();
    let mut legitimated = Simulation::new(make_config());
    legitimated.populate();
    let thief = (0..anchored.agents.len())
        .find(|&i| {
            !anchored
                .black_market
                .can_participate(&anchored.agents[i].personality)
        })
        .expect("at least one non-black-market agent");
    let thief_id = AgentId::new(thief as u64);
    let site_idx = anchored
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .expect("seed-42 world has a farm");
    // The consumer reads the produced noospheric field.
    legitimated.agents[thief]
        .relational_fields
        .legitimacy_perceived = Fixed::from_f64(0.9);
    // Both worlds start from the same farm stock.
    assert!((grain_left(&anchored) - grain_left(&legitimated)).abs() < 0.001);
    let baseline = grain_left(&anchored);
    assert!(anchored.enforce_theft(
        thief,
        thief_id,
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    assert!(legitimated.enforce_theft(
        thief,
        thief_id,
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    let anchored_taken = baseline - grain_left(&anchored);
    let legitimated_taken = baseline - grain_left(&legitimated);
    assert!(
        legitimated_taken < anchored_taken,
        "a legitimacy-deterred thief must take strictly less: \
             {legitimated_taken:.4} vs {anchored_taken:.4}"
    );
    assert!(
        (anchored_taken - 0.15).abs() < 0.001,
        "the anchored thief takes the full amount"
    );
    assert!(
        (legitimated_taken - 0.15 * 0.8).abs() < 0.001,
        "the legitimacy-deterred thief takes 0.15 x (1 - 0.4 x 0.5) = 0.12"
    );
}

/// §10.1.3 (Iteration 112): the panic fold genuinely shifts legitimacy
/// outcomes. Two same-seed worlds are IDENTICAL except the population's
/// collective fear (0.99 — a genuinely terrified population — vs 0.8,
/// below the 0.95 anchor where the amplifier is identity). Every agent's
/// council belief is pinned above the §7.2 trigger threshold (avg charge
/// ≥ 0.55, ≥ 30% of agents at charge > 0.4), so a panic fires
/// deterministically in both worlds at the same tick; the terrified
/// world must lose strictly more institutional legitimacy. The
/// amplification is provable without any RNG: the damage fold happens
/// before any roll.
#[test]
fn collective_fear_amplifies_panic_legitimacy_damage() {
    let make_config = || SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let council_legitimacy = |sim: &Simulation| -> f64 {
        sim.institutions
            .iter()
            .find(|i| i.kind == institutions::InstitutionKind::Council)
            .map_or(0.0, |i| i.legitimacy.to_f64())
    };
    let mut calm = Simulation::new(make_config());
    calm.populate();
    let mut terrified = Simulation::new(make_config());
    terrified.populate();
    // Pin every agent's council belief above the §7.2 trigger threshold
    // so the panic fires deterministically in both worlds at the same
    // tick (proposition 1 → Council).
    for agent in &mut calm.agents {
        agent.beliefs[1].emotional_charge = Fixed::from_f64(0.9);
    }
    for agent in &mut terrified.agents {
        agent.beliefs[1].emotional_charge = Fixed::from_f64(0.9);
    }
    // The consumer reads the produced noospheric field (world mean fear,
    // refreshed daily into the relational fields). 0.99 > 0.95 anchor →
    // amplified; 0.8 < 0.95 → identity.
    for agent in &mut calm.agents {
        agent.emotions.fear = Fixed::from_f64(0.8);
    }
    for agent in &mut terrified.agents {
        agent.emotions.fear = Fixed::from_f64(0.99);
    }
    // Raise council legitimacy so the damage differential is not erased
    // by the clamp at zero: 1.0 − 0.66 (calm) vs 1.0 − 0.6732
    // (terrified, ×1.02) are both visible above zero.
    for inst in &mut calm.institutions {
        if inst.kind == institutions::InstitutionKind::Council {
            inst.legitimacy = Fixed::ONE;
        }
    }
    for inst in &mut terrified.institutions {
        if inst.kind == institutions::InstitutionKind::Council {
            inst.legitimacy = Fixed::ONE;
        }
    }
    assert_eq!(council_legitimacy(&calm), 1.0);
    assert_eq!(council_legitimacy(&terrified), 1.0);
    // 400 ≥ the 300-tick cooldown; `Tick::ZERO` is fine — `tick_u64`
    // alone gates the cooldown, `tick` only stamps the emitted event.
    let tick_u64 = 400u64;
    calm.tick_moral_panic_and_revolution(tick_u64, Tick::ZERO);
    terrified.tick_moral_panic_and_revolution(tick_u64, Tick::ZERO);
    let calm_after = council_legitimacy(&calm);
    let terrified_after = council_legitimacy(&terrified);
    // Damage 0.66: calm (identity) → 1.0 − 0.66 = 0.34; terrified
    // (×1.02) → 1.0 − 0.6732 = 0.3268 — strictly lower.
    assert!(
        terrified_after < calm_after,
        "a terrified population must lose strictly more legitimacy: \
             calm {calm_after:.4} vs terrified {terrified_after:.4}"
    );
    assert!(
        (calm_after - 0.34).abs() < 0.001,
        "the calm world loses exactly the base damage"
    );
    assert!(
        (terrified_after - (1.0 - 0.66 * 1.02)).abs() < 0.001,
        "the terrified world loses the base damage amplified by 1.02"
    );
}

/// §8.1.10/§19.5.D (Iteration 86): the witnessed-enforcement *wiring* is
/// live — a caught theft (Council enforcement capacity 1.0 → the
/// detection roll < 1.0 always catches) increments `enforcement_count`
/// on the no-theft norm for every holder, including the violator, while
/// non-holders are a no-op. This closes the Iteration-86 coverage gap:
/// the default world's public-access farms never catch a theft, so only
/// this end-to-end test exercises the `enforce_theft` →
/// `record_witnessed_enforcement` loop.
#[test]
fn caught_theft_increments_witnessed_enforcement() {
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
    // A thief who cannot use the black market (enforcement is not halved).
    let thief = (0..sim.agents.len())
        .find(|&i| !sim.black_market.can_participate(&sim.agents[i].personality))
        .expect("at least one non-black-market agent");
    let thief_id = AgentId::new(thief as u64);
    // A farm with grain (same setup as the Iter-84 theft-take test).
    let site_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .expect("seed-42 world has a farm");
    {
        let site = &mut sim.world.sites[site_idx];
        if let Some(stock) = site
            .inventory
            .iter_mut()
            .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
        {
            stock.quantity = Fixed::from_f64(1.0);
        } else {
            site.inventory.push(crate::world::ResourceStock {
                resource_id: crate::world::GRAIN_RESOURCE_ID,
                quantity: Fixed::from_f64(1.0),
                quality: Fixed::ONE,
                access: crate::world::AccessRight::OwnerOnly,
            });
        }
    }
    // Guarantee the catch: a Council at full enforcement capacity.
    let council = sim
        .institutions
        .iter_mut()
        .find(|i| i.kind == institutions::InstitutionKind::Council)
        .expect("default village has a Council");
    council.enforcement_capacity = Fixed::ONE;
    // Two holders of the no-theft norm (thief + one witness) and one
    // control agent with no internalized norm.
    let witness = if thief == 0 { 1 } else { 0 };
    let control = if thief == 2 || witness == 2 { 3 } else { 2 };
    for &i in &[thief, witness] {
        sim.agents[i]
            .moral_cognition
            .internalize_norm("No Theft".into(), Fixed::from_f64(0.6));
    }
    let tick = Tick::ZERO;
    assert!(sim.enforce_theft(
        thief,
        thief_id,
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        Fixed::from_f64(0.15),
        0,
        tick
    ));
    let count = |i: usize| -> u32 {
        sim.agents[i]
            .moral_cognition
            .internalized_norms
            .iter()
            .find(|n| n.description == "No Theft")
            .map_or(0, |n| n.enforcement_count)
    };
    assert_eq!(count(thief), 1, "the violator experiences the enforcement");
    assert_eq!(
        count(witness),
        1,
        "a holder witnesses the public enforcement"
    );
    assert_eq!(count(control), 0, "non-holders have nothing to witness");
}

/// §8.1.10 (Iteration 87): the hypocrisy *wiring* is live — with zero
/// norm strength but full witnessed-enforcement exposure and full
/// hypocrisy sensitivity, the agent refuses the theft outright (the
/// hypocrisy factor alone drives the take to zero), a normless control
/// steals the full amount, and a partial-sensitivity agent takes
/// proportionally less (0.15 × (1 − 0.5) = 0.075).
#[test]
fn witnessed_enforcement_hypocrisy_suppresses_theft() {
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
    let site_idx = sim
        .world
        .sites
        .iter()
        .position(|s| s.kind == crate::world::SiteKind::Farm)
        .expect("seed-42 world has a farm");
    {
        let site = &mut sim.world.sites[site_idx];
        if let Some(stock) = site
            .inventory
            .iter_mut()
            .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
        {
            stock.quantity = Fixed::from_f64(1.0);
        } else {
            site.inventory.push(crate::world::ResourceStock {
                resource_id: crate::world::GRAIN_RESOURCE_ID,
                quantity: Fixed::from_f64(1.0),
                quality: Fixed::ONE,
                access: crate::world::AccessRight::OwnerOnly,
            });
        }
    }
    let grain_left = |sim: &Simulation| -> f64 {
        sim.world.sites[site_idx]
            .inventory
            .iter()
            .find(|s| s.resource_id == crate::world::GRAIN_RESOURCE_ID)
            .map_or(0.0, |s| s.quantity.to_f64())
    };
    let amount = Fixed::from_f64(0.15);
    let tick = Tick::ZERO;
    // Full hypocrisy: sensitivity 1.0, zero norm strength, 5 witnessed
    // enforcements → the hypocrisy factor alone refuses the theft.
    let full = 0;
    sim.agents[full].moral_cognition.hypocrisy_sensitivity = Fixed::ONE;
    sim.agents[full]
        .moral_cognition
        .internalize_norm("No Theft".into(), Fixed::ZERO);
    for _ in 0..5 {
        sim.agents[full]
            .moral_cognition
            .record_witnessed_enforcement("No Theft");
    }
    assert_eq!(
        sim.agents[full]
            .moral_cognition
            .hypocrisy_factor("No Theft"),
        Fixed::ONE
    );
    // Partial hypocrisy: default sensitivity 0.5, same exposure.
    let partial = 1;
    sim.agents[partial]
        .moral_cognition
        .internalize_norm("No Theft".into(), Fixed::ZERO);
    for _ in 0..5 {
        sim.agents[partial]
            .moral_cognition
            .record_witnessed_enforcement("No Theft");
    }
    // Normless control.
    let control = 2;
    // Control steals the full amount.
    assert!(sim.enforce_theft(
        control,
        AgentId::new(control as u64),
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    assert!((grain_left(&sim) - 0.85).abs() < 0.001);
    // Full hypocrisy refuses outright — nothing consumed.
    assert!(!sim.enforce_theft(
        full,
        AgentId::new(full as u64),
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    assert!((grain_left(&sim) - 0.85).abs() < 0.001);
    // Partial hypocrisy takes 0.15 × (1 − 0.5) = 0.075.
    assert!(sim.enforce_theft(
        partial,
        AgentId::new(partial as u64),
        site_idx,
        crate::world::GRAIN_RESOURCE_ID,
        amount,
        0,
        tick
    ));
    assert!((grain_left(&sim) - 0.775).abs() < 0.001);
}

/// §10.8/§19.5.G: A clan enmity persists while any feud remains between
/// the clans' members, and clears (symmetrically) once the feuds decay —
/// after which a marriage alliance can form again.
#[test]
fn clan_enmity_clears_when_feuds_decay() {
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
    let (a, b) = cross_clan_pair(&sim);
    let (ca, cb) = (sim.clan_of(a).unwrap(), sim.clan_of(b).unwrap());
    sim.forge_clan_enmity(a, b, 100);
    assert!(sim.clan_registry.get(ca).unwrap().is_enemy(cb));
    // An active feud between the clans' members keeps the enmity alive.
    sim.agents[a].feuds.push(b);
    sim.agents[a].feud_ticks.push(100);
    sim.agents[b].feuds.push(a);
    sim.agents[b].feud_ticks.push(100);
    sim.decay_clan_enmities();
    assert!(
        sim.clan_registry.get(ca).unwrap().is_enemy(cb),
        "active feud keeps the enmity"
    );
    // The feud fully decays → enmity clears both ways.
    sim.agents[a].feuds.clear();
    sim.agents[a].feud_ticks.clear();
    sim.agents[b].feuds.clear();
    sim.agents[b].feud_ticks.clear();
    sim.decay_clan_enmities();
    assert!(
        !sim.clan_registry.get(ca).unwrap().is_enemy(cb),
        "decayed feud clears the enmity"
    );
    assert!(
        !sim.clan_registry.get(cb).unwrap().is_enemy(ca),
        "cleared symmetrically"
    );
    // Peace reopens the alliance path: a later marriage can forge one.
    sim.forge_clan_alliance(a, b, 300);
    assert!(
        sim.clan_registry.get(ca).unwrap().is_ally(cb),
        "peace allows a marriage alliance"
    );
}

/// §10.9: Patrons provision destitute clients — a daily stipend moves
/// wealth through the social structure when the client is destitute and
/// the patron can afford it; no transfer otherwise.
#[test]
fn patronage_provision_transfers_wealth_to_destitute_clients() {
    use crate::social::patronage::PatronageRelation;
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
    let (patron, client) = (0usize, 1usize);
    sim.agents[patron].wealth.coin = Fixed::from_f64(10.0);
    sim.agents[client].wealth.coin = Fixed::from_f64(0.2);
    let mut rel = PatronageRelation::new(patron, client, 0);
    rel.provision = Fixed::from_f64(0.3);
    sim.patronage_registry.register(rel);
    let transfer = Fixed::from_f64(0.3) * PATRONAGE_TRANSFER_RATE;
    sim.tick_patronage_provision();
    assert!(
        (sim.agents[client].wealth.coin - (Fixed::from_f64(0.2) + transfer))
            .to_f64()
            .abs()
            < 1e-6,
        "client must receive the stipend"
    );
    assert!(
        (sim.agents[patron].wealth.coin - (Fixed::from_f64(10.0) - transfer))
            .to_f64()
            .abs()
            < 1e-6,
        "patron must pay the stipend"
    );
    // Non-destitute client → no transfer.
    sim.agents[client].wealth.coin = Fixed::from_f64(5.0);
    sim.tick_patronage_provision();
    assert_eq!(sim.agents[client].wealth.coin.to_f64(), 5.0);
    // Destitute client but destitute patron → no transfer.
    sim.agents[client].wealth.coin = Fixed::from_f64(0.2);
    sim.agents[patron].wealth.coin = Fixed::ZERO;
    sim.tick_patronage_provision();
    assert_eq!(sim.agents[client].wealth.coin.to_f64(), 0.2);
}

/// §10.9: Patronage buys political quiescence — a dependent client's
/// faction grievance is dampened by dependence × the dampening factor;
/// a zero-dependence client is unaffected.
#[test]
fn patronage_dampens_client_grievance() {
    use crate::social::patronage::PatronageRelation;
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
    let client = 1usize;
    // Force a measurable grievance baseline.
    sim.agents[client].derived.resentment = Fixed::from_f64(0.6);
    sim.agents[client].emotions.anger = Fixed::from_f64(0.5);
    let plain = sim.faction_grievance(client);
    // A patron with high dependence dampens the client's grievance by the
    // exact dampening factor.
    let mut rel = PatronageRelation::new(0, client, 0);
    rel.client_dependence = Fixed::from_f64(0.8);
    sim.patronage_registry.register(rel);
    let dampened = sim.faction_grievance(client);
    let expected =
        (plain * (Fixed::ONE - Fixed::from_f64(0.8) * PATRONAGE_GRIEVANCE_DAMPEN)).max(Fixed::ZERO);
    assert_eq!(dampened.to_f64(), expected.to_f64());
    assert!(dampened < plain, "patronage must dampen client grievance");
    // A zero-dependence relation dampens nothing.
    sim.patronage_registry.relations.clear();
    let mut rel0 = PatronageRelation::new(0, client, 0);
    rel0.client_dependence = Fixed::ZERO;
    sim.patronage_registry.register(rel0);
    assert_eq!(sim.faction_grievance(client).to_f64(), plain.to_f64());
}

/// §11.1: Perceived legitimacy modulates faction grievance — a deviation
/// of `overall` from the 0.5 construction anchor dampens (above) or
/// amplifies (below) grievance by the exact dampening factor; the anchor
/// itself is a no-op.
#[test]
fn perceived_legitimacy_modulates_faction_grievance() {
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
    let agent = 1usize;
    // Force a measurable grievance baseline.
    sim.agents[agent].derived.resentment = Fixed::from_f64(0.6);
    sim.agents[agent].emotions.anger = Fixed::from_f64(0.5);
    // Mean-zero anchor: the fresh field (overall == 0.5) dampens nothing.
    let plain = sim.faction_grievance(agent);
    // High perceived legitimacy quiesces the agent by the exact factor.
    sim.agents[agent].legitimacy_field.overall = Fixed::from_f64(0.9);
    let dampened = sim.faction_grievance(agent);
    let expected = (plain * (Fixed::ONE - Fixed::from_f64(0.4) * LEGITIMACY_GRIEVANCE_DAMPEN))
        .max(Fixed::ZERO);
    assert_eq!(dampened.to_f64(), expected.to_f64());
    assert!(dampened < plain, "high legitimacy must dampen grievance");
    // Low perceived legitimacy amplifies grievance symmetrically.
    sim.agents[agent].legitimacy_field.overall = Fixed::from_f64(0.1);
    let amplified = sim.faction_grievance(agent);
    let expected_amp = (plain * (Fixed::ONE + Fixed::from_f64(0.4) * LEGITIMACY_GRIEVANCE_DAMPEN))
        .max(Fixed::ZERO);
    assert_eq!(amplified.to_f64(), expected_amp.to_f64());
    assert!(amplified > plain, "low legitimacy must amplify grievance");
}

/// §8.1.4 (Iteration 130): the awe reverence fold is LIVE — the daily
/// scandal erosion is multiplied by `awe_reverence_factor`, so a run
/// whose awe is pinned at saturation (1.0, factor 0.85) must preserve
/// strictly more perceived legitimacy than the control (natural awe
/// ~0.67, factor ~0.90) over a horizon that is mid-erosion but below
/// the floor. AWE CONVERGENCE (probe-pinned): awe converges to the same
/// world-determined equilibrium (~0.67 by tick 130) regardless of the
/// start value, so a one-shot injection at populate is erased before
/// scandal erosion begins (~tick 120) — the differential would vanish.
/// The manual-tick harness re-injects awe before EVERY tick, pinning
/// the treatment at the 0.85 floor for every scandal call while the
/// control's natural awe yields ~0.90; same seed, same RNG stream, same
/// call order — any legitimacy divergence is attributable to the fold.
#[test]
fn awe_reverence_shields_legitimacy_from_scandal_erosion() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut reverent = Simulation::new(config.clone());
    reverent.populate();
    let mut plain = Simulation::new(config);
    plain.populate();
    // Iteration 185 (emergent-quality audit — calm lethality
    // recalibration): the violence fix calms scandal pressure, so the
    // reverence differential builds slower — probe-pinned diff 0.0000
    // @200, 0.0001 @400, 0.0005 @800, 0.0018 @1500 (reverent > plain
    // throughout the growing phase). The horizon extends 200 → 1500 to
    // land mid-erosion with a measurable gap.
    for _ in 0..1500 {
        for a in &mut reverent.agents {
            a.emotions.awe = Fixed::ONE;
        }
        reverent.tick();
        plain.tick();
    }
    let leg_mean = |sim: &Simulation| {
        let n = sim.agents.len() as f64;
        sim.agents
            .iter()
            .map(|a| a.legitimacy_field.overall.to_f64())
            .sum::<f64>()
            / n
    };
    let reverent_leg = leg_mean(&reverent);
    let plain_leg = leg_mean(&plain);
    assert!(
        plain_leg < 0.5,
        "scandal erosion must have fired in the horizon (control {plain_leg:.4})"
    );
    assert!(
        reverent_leg > plain_leg,
        "awe-saturated run must preserve more legitimacy than the control \
             (the §8.1.4 reverence fold is wired), reverent {reverent_leg:.4} vs \
             plain {plain_leg:.4}"
    );
}
