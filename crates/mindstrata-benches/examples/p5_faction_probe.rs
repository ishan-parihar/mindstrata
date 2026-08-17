//! Iteration 186 — pestilence faction-formation probe.
//!
//! The Iteration-186 legitimacy-equilibrium fix moved faction formation
//! from every base-world village (permanently-armed trigger) to genuine
//! grievance crises (grievance > ~0.67, i.e. pestilence/collapse). The
//! base-world faction tests must re-anchor to the crisis scenario. This
//! probe finds, per seed, when the first faction forms, whether an ACTIVE
//! faction exists at candidate snapshot horizons, and how rich the v2
//! registry history is (the attachment-styles test reads the history).
//!
//! Run: `cargo run -p mindstrata-benches --example p5_faction_probe --release`
//! Knobs: `P5_HORIZON` (default 30_000), `P5_SEED` (default 42),
//! `P5_SCEN` (default pestilence).

use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

fn main() {
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30_000);
    let seed: u64 = std::env::var("P5_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);
    let scen = std::env::var("P5_SCEN").unwrap_or_else(|_| "pestilence".into());

    let mut sc = match scen.as_str() {
        "pestilence" => Scenario::pestilence(),
        "famine" => Scenario::famine(),
        "calm" => Scenario::calm(),
        _ => unreachable!(),
    };
    sc.seed = seed;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();

    let mut first_faction: Option<u64> = None;
    let sample = 1000u64;
    let mut t = 0u64;
    while t < horizon {
        sim.run(sample.min(horizon - t));
        t += sample.min(horizon - t);
        let v1 = sim
            .institutions
            .iter()
            .filter(|i| i.kind == InstitutionKind::Faction)
            .count();
        let v2_active = sim
            .faction_v2_registry
            .factions
            .iter()
            .filter(|f| f.active)
            .count();
        if first_faction.is_none() && (v1 > 0 || v2_active > 0) {
            first_faction = Some(t);
        }
        if t.is_multiple_of(5000) {
            let legit = sim
                .institutions
                .iter()
                .filter(|i| i.kind == InstitutionKind::Council)
                .map(|i| i.legitimacy)
                .fold(mindstrata_core::fixed::Fixed::ZERO, std::cmp::Ord::max);
            let revs = sim
                .recent_events(10_000_000)
                .iter()
                .filter(|e| {
                    matches!(
                        e,
                        mindstrata_core::event::SimEvent::ConflictOccurred {
                            kind: mindstrata_core::conflict::ConflictKind::Revolution,
                            ..
                        }
                    )
                })
                .count();
            println!(
                "  t={t:>6}: v1_factions={v1} v2_active={v2_active} legit={:.3} revs={revs}",
                legit.to_f64()
            );
        }
    }
    let v2_history = sim.faction_v2_registry.factions.len();
    let non_secure = sim
        .faction_v2_registry
        .factions
        .iter()
        .filter(|f| f.attachment_style != mindstrata_sim::social::group_formation::GroupAttachmentStyle::Secure)
        .count();
    println!(
        "  [{scen} seed {seed} @{horizon}] first_faction={first_faction:?} v2_history={v2_history} non_secure_in_history={non_secure}"
    );
}
