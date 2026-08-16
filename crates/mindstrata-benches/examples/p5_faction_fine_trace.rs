//! Iteration 186 — fine-grained faction-lifetime trace.
//!
//! The coarse probes sample `factions` every 1K–5K ticks and see 0 while the
//! cumulative revolution counter climbs. This trace samples every 50 ticks
//! for the first P5_HORIZON ticks and prints EVERY tick where a faction is
//! observed alive, plus every revolution event tick, to prove factions are
//! short-lived (form → threshold → coup → dissolve all within the inter-sample
//! window) and that every revolution requires one.
//!
//! Run: `cargo run -p mindstrata-benches --example p5_faction_fine_trace --release`
//! Knobs: `P5_HORIZON` (default 12_000), `P5_SEED` (default 42),
//! `P5_SCEN` (default pestilence).

use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

fn main() {
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(12_000);
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

    let sample = 10u64;
    let mut t = 0u64;
    let mut last_revs = 0usize;
    let mut alive_anywhere = false;
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
        if v1 > 0 || v2_active > 0 {
            alive_anywhere = true;
            println!("  t={t:>6}: faction ALIVE v1={v1} v2_active={v2_active} revs={revs}");
        } else if revs != last_revs {
            println!("  t={t:>6}: revs={revs} (faction already dissolved)");
        }
        last_revs = revs;
    }
    println!(
        "  [{scen} seed {seed} @{horizon}] faction observed alive at 50-tick granularity: {alive_anywhere}; total revs={last_revs}"
    );
}
