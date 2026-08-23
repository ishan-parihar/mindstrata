//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::*;

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
