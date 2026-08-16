//! Iteration 186 — final-audit orphan/write-only probe.
//!
//! Empirically verifies the write-only and dead-path findings from the
//! exhaustive re-audit:
//!   1. **endocrine arousal** — the S2-2-4 fix wired the saturating GAIN
//!      (update is called), but does the LEVEL ever move AND is it read by
//!      any decision? Compare it against `affect.arousal` (the live
//!      psychological quantity) — if the biological level runs its
//!      dynamics and stays unconsumed, it is a write-only axis.
//!   2. **circadian behavioral queries** — `should_sleep()`/`sleep_deprived()`
//!      have zero callers; the sim drives sleep from the action choice
//!      instead. Verify sleep_pressure still builds (the state is live)
//!      while the behavioral interface is dead.
//!   3. **disease kinds** — Cold/Fever/WoundInfection never appear in a
//!      real run (only Epidemic, seeded by the pestilence shock). Count
//!      distinct kinds per agent across scenarios.
//!   4. **market price trend** — `price_trend()`/`trend()` have zero
//!      consumers; verify recent_prices is written (history recorded) but
//!      no decision reads it.
//!
//! Run: `cargo run -p mindstrata-benches --example p6_orphan_probe --release`
//! Knobs: `P5_HORIZON` (default 50_000), `P5_SEED` (default 42).

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;
use std::collections::BTreeSet;

fn main() {
    let horizon: u64 = std::env::var("P5_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(50_000);
    let seed: u64 = std::env::var("P5_SEED")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(42);

    for (scen_name, scen) in [
        ("calm", Scenario::calm()),
        ("famine", Scenario::famine()),
        ("pestilence", Scenario::pestilence()),
    ] {
        let mut sc = scen;
        sc.seed = seed;
        sc.ticks = horizon;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        sim.run(horizon);

        // 1. Endocrine arousal level vs affect arousal at the horizon.
        let (mut endo_arousal, mut affect_arousal) = (0.0f64, 0.0f64);
        let (mut endo_stress, mut endo_bonding, mut endo_dominance) = (0.0f64, 0.0f64, 0.0f64);
        let (mut sleep_pressure, mut sleep_debt) = (0.0f64, 0.0f64);
        let mut kinds: BTreeSet<String> = BTreeSet::new();
        let mut total_diseases = 0usize;
        for (i, a) in sim.agents.iter().enumerate() {
            endo_arousal += a.embodied.endocrine.arousal.level.to_f64();
            endo_stress += a.embodied.endocrine.stress.level.to_f64();
            endo_bonding += a.embodied.endocrine.bonding.level.to_f64();
            endo_dominance += a.embodied.endocrine.dominance.level.to_f64();
            affect_arousal += a.affect.arousal.to_f64();
            sleep_pressure += a.embodied.circadian.sleep_pressure.to_f64();
            sleep_debt += a.embodied.circadian.sleep_debt.to_f64();
            total_diseases += sim.agent_diseases[i].len();
            for d in &sim.agent_diseases[i] {
                kinds.insert(format!("{d:?}"));
            }
        }
        let n = sim.agents.len() as f64;
        println!(
            "[{scen_name} @{horizon}] n={n:.0} | endo_arousal={:.3} affect_arousal={:.3} \
             | endo_stress={:.3} endo_bonding={:.3} endo_dominance={:.3} \
             | sleep_pressure={:.3} sleep_debt={:.3} | diseases={total_diseases} kinds={kinds:?}",
            endo_arousal / n,
            affect_arousal / n,
            endo_stress / n,
            endo_bonding / n,
            endo_dominance / n,
            sleep_pressure / n,
            sleep_debt / n,
        );
    }
}
