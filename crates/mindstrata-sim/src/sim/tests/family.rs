//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::*;
use super::cross_clan_pair;

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
