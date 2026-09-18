//! Iter-286 probe — WP-H3 second half: norm proposals from reconciled
//! polarity clusters. Reachability census BEFORE wiring.
//!
//! Design target (wave brief): "norm proposals generated from reconciled
//! polarity clusters". The reconciliation pass (DC-2.1) synthesizes
//! `Integrated` claims with the more-encompassing subtle claim — a
//! synthesis landing on SubtleClaim::Value or ::Norm is precisely a
//! crystallized communal prescription. The proposal channel would turn
//! such syntheses into registry norms.
//!
//! Questions this census answers first:
//! 1. How many Integrated Value/Norm claims exist per agent by horizon?
//! 2. How DISTINCT are their (referent, line) slots? (A proposal channel
//!    keyed on slots should not emit dozens of duplicate norms.)
//! 3. Do they form EARLY (before the first monthly ritual at 4320, when
//!    reinforcement consumers start reading the registry)?

use mindstrata_development::line::LineId;
use mindstrata_development::polarity::{GrossReferent, PolarityState, SubtleClaim};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn census(sim: &Simulation) -> Vec<((u8, String, u8), usize)> {
    // Integrated claims grouped by (referent, line, subtle-claim) slot →
    // distinct holder count. The consensus signal for norm proposals.
    use std::collections::BTreeMap;
    let mut slots: BTreeMap<(u8, String, u8), (usize, std::collections::BTreeSet<usize>)> =
        BTreeMap::new();
    for (i, a) in sim.agents.iter().enumerate() {
        for c in &a.polarity_claims {
            if c.polarity == PolarityState::Integrated {
                let key = (c.referent as u8, c.line.slug().to_string(), c.claim as u8);
                let e = slots.entry(key).or_insert((0, Default::default()));
                e.0 += 1;
                e.1.insert(i);
            }
        }
    }
    slots
        .into_iter()
        .map(|(k, (claims, holders))| {
            (
                k,
                holders.len().max(claims.min(0)) + holders.len() * 0 + holders.len(),
            )
        })
        .map(|(k, h)| ((k.0, k.1, k.2), h))
        .collect()
}

fn run_named(name: &str, sc: Scenario, horizon: u64) {
    let mut sc = sc;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(horizon);
    let slots = census(&sim);
    let summary = slots
        .iter()
        .map(|((r, l, c), h)| format!("slot{r}/{l}/c{c}×{h}"))
        .collect::<Vec<_>>()
        .join(" ");
    println!(
        "{name:>11} @{horizon:>5}: integrated_slots={}",
        if summary.is_empty() {
            "none".into()
        } else {
            summary
        }
    );
}

fn main() {
    // The cognitive/values/justice line slugs must be registered.
    for slug in ["cognitive", "values", "justice"] {
        assert!(LineId::new(slug).is_some(), "{slug} must be in registry");
    }
    for horizon in [1_000u64, 4_320, 20_000] {
        run_named("calm", Scenario::calm(), horizon);
        run_named("drought", Scenario::drought(), horizon);
        run_named("collapse", Scenario::collapse(), horizon);
    }
}
