//! Iter-286 probe part 2 — sizing the norm-proposal channel.
//!
//! (a) FIRST-SYNTHESIS TICK via Refuted-census growth: refutation requires
//!     an Integrated claim (i284 scope), and part 1 proved syntheses are
//!     consumed intra-pass — so the first tick with Refuted > 0 ≈ the first
//!     synthesis tick. This sets the proposal channel's earliest trigger.
//! (b) CLUSTER QUORUM: distinct agents holding ActiveTension per
//!     (referent, line) slot at 20K — sizes the consensus gate for norm
//!     proposals (a village-wide prescription needs more than one holder).

use mindstrata_development::polarity::PolarityState;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn refuted_count(sim: &Simulation) -> usize {
    sim.agents
        .iter()
        .map(|a| {
            a.polarity_claims
                .iter()
                .filter(|c| c.polarity == PolarityState::Refuted)
                .count()
        })
        .sum()
}

fn quorum(sim: &Simulation) -> Vec<((u8, String), usize)> {
    use std::collections::BTreeMap;
    let mut slots: BTreeMap<(u8, String), std::collections::BTreeSet<usize>> = BTreeMap::new();
    for (i, a) in sim.agents.iter().enumerate() {
        for c in &a.polarity_claims {
            if c.polarity == PolarityState::ActiveTension {
                slots
                    .entry((c.referent as u8, c.line.slug().to_string()))
                    .or_default()
                    .insert(i);
            }
        }
    }
    slots.into_iter().map(|(k, v)| (k, v.len())).collect()
}

fn main() {
    for (name, sc) in [
        ("calm", Scenario::calm()),
        ("drought", Scenario::drought()),
        ("collapse", Scenario::collapse()),
    ] {
        let mut sc = sc;
        sc.ticks = 20_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        let mut first_refuted = None;
        let step = 500u64;
        let mut t = step;
        while t <= 20_000 {
            sim.run(step); // advances BY 500 — totals stay 20K
            let r = refuted_count(&sim);
            if r > 0 && first_refuted.is_none() {
                first_refuted = Some(t);
                break;
            }
            t += step;
        }
        // Continue to 20K for the quorum census.
        if t < 20_000 {
            sim.run(20_000 - t);
        }
        let q = quorum(&sim);
        let summary = q
            .iter()
            .map(|((r, l), n)| format!("slot{r}/{l}×{n}"))
            .collect::<Vec<_>>()
            .join(" ");
        println!(
            "{name:>8}: first_refuted@={first_refuted:?}  tension_quorum@20K={}",
            if summary.is_empty() {
                "none".into()
            } else {
                summary
            }
        );
    }
}
