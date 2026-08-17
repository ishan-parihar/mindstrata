//! Iteration 192 — famine-window probe. The p6 probe samples the 50K
//! end-state, long after the 2000-tick famine window closed and the
//! village recovered (hunger 0.007 vs calm 0.011 — recovery, not a gap).
//! This probe samples INSIDE the window (500/1000/1500/2000) plus early
//! recovery (3000/5000/10000) to verify the famine genuinely starves and
//! that the Iter-188 acute-stress channel (live hunger/thirst -> cortisol)
//! elevates stress during the window.
//!
//! Run: `cargo run -p mindstrata-benches --example p12_famine_probe --release`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

fn main() {
    let seed: u64 = std::env::var("P12_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    for horizon in [500u64, 1000, 1500, 2000, 2500, 3000, 5000, 10000] {
        for (name, scen) in [
            ("calm", Scenario::calm()),
            ("famine", Scenario::famine()),
        ] {
            let mut sc = scen;
            sc.seed = seed;
            sc.ticks = horizon;
            let mut sim = Simulation::from_scenario(sc);
            sim.populate();
            sim.run(horizon);
            let n = sim.agents.len() as f64;
            let mut hunger = 0.0f64;
            let mut thirst = 0.0f64;
            let mut stress = 0.0f64;
            let mut arousal = 0.0f64;
            let mut valence = 0.0f64;
            let mut grain = 0.0f64;
            let mut max_hunger = 0.0f64;
            let mut over_gate = 0usize;
            let mut despair_sum = 0.0f64;
            let mut sadness_sum = 0.0f64;
            let mut depression_sum = 0.0f64;
            let mut chronic_sum = 0.0f64;
            let mut max_despair = 0.0f64;
            let mut max_depression = 0.0f64;
            let mut max_sadness = 0.0f64;
            let mut despair_over_gate = 0usize;
            for a in &sim.agents {
                hunger += a.needs.hunger.to_f64();
                thirst += a.needs.thirst.to_f64();
                stress += a.embodied.endocrine.stress.level.to_f64();
                arousal += a.embodied.endocrine.arousal.level.to_f64();
                valence += a.affect.valence.to_f64();
                max_hunger = max_hunger.max(a.needs.hunger.to_f64());
                if a.needs.hunger.to_f64() >= 0.5 {
                    over_gate += 1;
                }
                despair_sum += a.emotions.despair.to_f64();
                sadness_sum += a.emotions.sadness.to_f64();
                depression_sum += a.psychopathology.depression_risk.to_f64();
                chronic_sum += a.embodied.endocrine.stress.chronic_load.to_f64();
                max_despair = max_despair.max(a.emotions.despair.to_f64());
                max_depression = max_depression.max(a.psychopathology.depression_risk.to_f64());
                max_sadness = max_sadness.max(a.emotions.sadness.to_f64());
                if a.needs.hunger.to_f64() >= 0.5 && a.emotions.despair.to_f64() > 0.0 {
                    despair_over_gate += 1;
                }
            }
            for site in &sim.world.sites {
                for stock in &site.inventory {
                    if stock.resource_id == 0u64 {
                        grain += stock.quantity.to_f64();
                    }
                }
            }
            println!(
                "[{name} @{horizon:>6}] n={n:.0} hunger={hunger:.3} thirst={thirst:.3} stress={stress:.3} \
                 arousal={arousal:.3} valence={valence:.3} grain={grain:.2} maxH={max_hunger:.3} \
                 gate={over_gate} despair={despair:.3} maxD={max_despair:.3} dOverG={despair_over_gate} \
                 sadness={sadness:.3} maxSad={max_sadness:.3} depress={depression:.4} \
                 maxDep={max_depression:.4} chronic={chronic:.3}",
                hunger = hunger / n,
                thirst = thirst / n,
                stress = stress / n,
                arousal = arousal / n,
                valence = valence / n,
                grain = grain,
                despair = despair_sum / n,
                sadness = sadness_sum / n,
                depression = depression_sum / n,
                chronic = chronic_sum / n
            );
        }
        println!();
    }
}
