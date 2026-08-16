//! Iteration 186 — calm-vs-famine inversion diagnostic.
//!
//! Post-Iteration-186, the coin dividend fixed the poverty channel (0/12
//! under the 3-coin line, market treasury ~172) yet CALM still shows
//! HIGHER grievance (0.62) than famine (0.46–0.50) and a perpetual
//! revolution clock (28 coups/100K vs famine-13's 0 until 88K). The
//! grievance drivers decompose to resentment 0.44 + fear 0.39 +
//! meaning-deficit 0.73 + gini 0.75 in calm. This probe compares the
//! psychology, economy, and negative-interaction counts per scenario to
//! find WHY a calm village is more fearful/resentful than a famine one.
//!
//! Run: `cargo run -p mindstrata-benches --example p5_calm_diag --release`
//! Knob: `P5_HORIZON` (default 20_000).

use mindstrata_core::event::{InteractionKind, SimEvent};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::collections::BTreeMap;

fn main() {
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(20_000);
    let filter = std::env::var("P5_ONLY").unwrap_or_default();

    for (scenario, seed) in [
        ("calm", 42u64),
        ("calm", 7),
        ("famine", 42),
        ("famine", 13),
        ("pestilence", 42),
    ] {
        if !filter.is_empty() && scenario != filter {
            continue;
        }
        let mut sc = match scenario {
            "calm" => Scenario::calm(),
            "famine" => Scenario::famine(),
            "pestilence" => Scenario::pestilence(),
            _ => unreachable!(),
        };
        sc.seed = seed;
        sc.ticks = horizon;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        sim.run(horizon);

        let n = sim.agents.len().max(1) as f64;
        let (mut joy, mut fear, mut anger, mut sad, mut trust) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let (mut res, mut val, mut stress, mut auton, mut meaning) = (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for a in &sim.agents {
            joy += a.emotions.joy.to_f64();
            fear += a.emotions.fear.to_f64();
            anger += a.emotions.anger.to_f64();
            sad += a.emotions.sadness.to_f64();
            trust += a.emotions.trust.to_f64();
            res += a.derived.resentment.to_f64();
            val += a.affect.valence.to_f64();
            stress += a.embodied.endocrine.stress.level.to_f64();
            auton += a.needs.autonomy.to_f64();
            meaning += a.needs.meaning.to_f64();
        }

        // Negative interaction + conflict counts
        let mut kinds: BTreeMap<&'static str, u32> = BTreeMap::new();
        for e in sim.recent_events(10_000_000) {
            match e {
                SimEvent::InteractionOccurred { kind, .. } => match kind {
                    InteractionKind::Help => *kinds.entry("help").or_insert(0) += 1,
                    InteractionKind::Comfort => *kinds.entry("comfort").or_insert(0) += 1,
                    InteractionKind::Threaten => *kinds.entry("threaten").or_insert(0) += 1,
                    InteractionKind::Insult => *kinds.entry("insult").or_insert(0) += 1,
                    InteractionKind::Gossip => *kinds.entry("gossip").or_insert(0) += 1,
                    _ => *kinds.entry("other").or_insert(0) += 1,
                },
                SimEvent::ConflictOccurred { kind, .. } => {
                    *kinds.entry(match kind {
                        mindstrata_core::conflict::ConflictKind::Threat => "c_threat",
                        mindstrata_core::conflict::ConflictKind::Violence => "c_violence",
                        mindstrata_core::conflict::ConflictKind::Intimidation => "c_intim",
                        mindstrata_core::conflict::ConflictKind::Feud => "c_feud",
                        mindstrata_core::conflict::ConflictKind::MoralPanic => "c_panic",
                        mindstrata_core::conflict::ConflictKind::Revolution => "c_rev",
                        mindstrata_core::conflict::ConflictKind::Combat => "c_combat",
                    })
                    .or_insert(0) += 1;
                }
                _ => {}
            }
        }

        let council = sim
            .institutions
            .iter()
            .find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Council);
        println!(
            "=== {scenario} seed {seed} @{horizon} ===\n  emo joy={:.3} fear={:.3} anger={:.3} sad={:.3} trust={:.3} | res={:.3} val={:.3} stress={:.3} auton_need={:.3} meaning_need={:.3}\n  council legit={:.3} morale={:.3} members={} | gini={:.3} grain={:.0} water={:.0}\n  events: {kinds:?}",
            joy / n, fear / n, anger / n, sad / n, trust / n,
            res / n, val / n, stress / n, auton / n, meaning / n,
            council.map_or(0.0, |c| c.legitimacy.to_f64()),
            council.map_or(0.0, |c| c.collective.morale.to_f64()),
            council.map_or(0, |c| c.members.len()),
            sim.market.inequality.to_f64(),
            sim.total_grain().to_f64(),
            sim.total_water().to_f64(),
        );
    }
}
