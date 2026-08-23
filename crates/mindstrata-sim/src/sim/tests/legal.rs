//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::*;
use crate::institutions;
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
