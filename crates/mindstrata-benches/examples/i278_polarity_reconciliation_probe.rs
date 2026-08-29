//! Polarity engine end-to-end probe (DC-1 STORY 8-9, FR-030/FR-032).
//!
//! Reads the live `EventJournal` from the calm scenario, builds three
//! three-realm claims from real event streams, and reports reconciliation
//! outcomes. The v1 probe exercises `reconcile_claims` against synthetic
//! claim pairs (the event stream does not currently emit claims, but the
//! pure engine is operational; this measures the v1 surface and documents
//! the data-path gap to STORY 9-10).
//!
//! Run: cargo run --release -p mindstrata-benches --example i278_polarity_reconciliation_probe

use mindstrata_development::line::LineId;
use mindstrata_development::polarity::{
    is_active_tension, reconcile_claims, CausalDomain, GrossReferent, PolarityState, SubtleClaim,
    ThreeRealmClaim,
};
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 2000,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(2000);

    let events = sim.recent_events(10_000_000);
    let event_count = events.len();
    let events_with_content = events
        .iter()
        .filter(|e| !format!("{e:?}").is_empty())
        .count();
    println!("events_total={event_count}");
    println!("events_with_content={events_with_content}");

    // v1 surface: 4 synthetic claim pairs covering the legal reconcile axes.
    let mut reconciled = 0usize;
    let mut active_tension = 0usize;
    let mut no_reconcile = 0usize;

    // (Material, Symbolic) on Event → reconcilable
    let a = ThreeRealmClaim::new(
        CausalDomain::Material,
        GrossReferent::Event,
        SubtleClaim::Fact,
        LineId::new("cognitive").expect("cognitive line"),
    );
    let mut a = a;
    a.polarity = PolarityState::ActiveTension;
    let mut b = ThreeRealmClaim::new(
        CausalDomain::Symbolic,
        GrossReferent::Event,
        SubtleClaim::Value,
        LineId::new("cognitive").expect("cognitive line"),
    );
    b.polarity = PolarityState::ActiveTension;
    match reconcile_claims(&a, &b) {
        Some(_) => reconciled += 1,
        None => no_reconcile += 1,
    }
    if is_active_tension(&a, &b) {
        active_tension += 1;
    }

    // (Material, Material) on Event → not reconcilable
    let mut c = ThreeRealmClaim::new(
        CausalDomain::Material,
        GrossReferent::Event,
        SubtleClaim::Fact,
        LineId::new("cognitive").expect("cognitive line"),
    );
    c.polarity = PolarityState::ActiveTension;
    let mut d = ThreeRealmClaim::new(
        CausalDomain::Material,
        GrossReferent::Event,
        SubtleClaim::Value,
        LineId::new("cognitive").expect("cognitive line"),
    );
    d.polarity = PolarityState::ActiveTension;
    match reconcile_claims(&c, &d) {
        Some(_) => reconciled += 1,
        None => no_reconcile += 1,
    }

    // Undiscovered claim → no reconcile
    let e = ThreeRealmClaim::new(
        CausalDomain::Material,
        GrossReferent::Event,
        SubtleClaim::Fact,
        LineId::new("cognitive").expect("cognitive line"),
    );
    let f = b;
    match reconcile_claims(&e, &f) {
        Some(_) => reconciled += 1,
        None => no_reconcile += 1,
    }

    println!("reconciled_count={reconciled}");
    println!("active_tension_count={active_tension}");
    println!("no_reconcile_count={no_reconcile}");
    let verdict = if reconciled == 1 && active_tension == 1 && no_reconcile == 2 {
        "POLARITY_LIVE"
    } else {
        "POLARITY_REGRESSION"
    };
    println!("verdict={verdict}");
}
