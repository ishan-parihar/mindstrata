//! Iteration 186 — final-audit orphan/write-only probe; Iteration 187
//! extends it to verify the consumer wirings that closed the findings:
//!
//! Original findings:
//!   1. **endocrine arousal** — the S2-2-4 fix wired the saturating GAIN
//!      (update is called), but did the LEVEL feed any decision? Compare it
//!      against `affect.arousal` (the live psychological quantity).
//!   2. **circadian behavioral queries** — `should_sleep()`/`sleep_deprived()`
//!      had zero callers; sleep was action-choice-driven.
//!   3. **disease kinds** — Cold/Fever/WoundInfection never appeared in real
//!      runs (only Epidemic, seeded by the pestilence shock).
//!   4. **market price trend** — `price_trend()`/`trend()` had zero consumers.
//!
//! Iteration 187 closures (all now wired, this probe re-verifies liveness):
//!   - endocrine arousal → `attention.recompute_biases` threat fold (hyper-
//!     vigilance): the mean threat_bias must now track the arousal level.
//!   - circadian → sleep drive: `sleep_pressure` builds AND the Rest action
//!     responds (probe prints the pressure; the sim floors `needs.fatigue`
//!     when `should_sleep()`/`sleep_deprived()` fire).
//!   - seasonal Cold/Fever vector: over a full year+ horizon (≈35K ticks),
//!     calm runs must show Cold (and Fever) circulating — the kinds are no
//!     longer unreachable outside the injected mix probe.
//!   - price trend → trade price; logistics → local scarcity price (structural;
//!     verified by grep + the trade-path unit tests, not sampled here).
//!
//! Run: `cargo run -p mindstrata-benches --example p6_orphan_probe --release`
//! Knobs: `P5_HORIZON` (default 50_000 — spans ~1.4 years so Winter fires),
//! `P5_SEED` (default 42).

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
        let mut threat_bias = 0.0f64;
        let (mut needs_hunger, mut needs_thirst) = (0.0f64, 0.0f64);
        let (mut body_hunger, mut body_thirst) = (0.0f64, 0.0f64);
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
            threat_bias += a.attention.threat_bias.to_f64();
            needs_hunger += a.needs.hunger.to_f64();
            needs_thirst += a.needs.thirst.to_f64();
            body_hunger += a.body.hunger.to_f64();
            body_thirst += a.body.thirst.to_f64();
            total_diseases += sim.agent_diseases[i].len();
            for d in &sim.agent_diseases[i] {
                kinds.insert(format!("{d:?}"));
            }
        }
        let n = sim.agents.len() as f64;
        println!(
            "[{scen_name} @{horizon}] n={n:.0} | endo_arousal={:.3} affect_arousal={:.3} \
             | threat_bias={:.3} | endo_stress={:.3} endo_bonding={:.3} endo_dominance={:.3} \
             | sleep_pressure={:.3} sleep_debt={:.3} | needs_hunger={:.3} needs_thirst={:.3} \
             | body_hunger={:.3} body_thirst={:.3} | diseases={total_diseases} kinds={kinds:?}",
            endo_arousal / n,
            affect_arousal / n,
            threat_bias / n,
            endo_stress / n,
            endo_bonding / n,
            endo_dominance / n,
            sleep_pressure / n,
            sleep_debt / n,
            needs_hunger / n,
            needs_thirst / n,
            body_hunger / n,
            body_thirst / n,
        );
    }
}
