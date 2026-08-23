//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::*;
use crate::institutions;

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
