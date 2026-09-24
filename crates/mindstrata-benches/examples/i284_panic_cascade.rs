//! Iter-284 probe — WP-H2 panic cascade: refutation storms → norm churn →
//! re-crystallization (the wave-brief's i288 behavioral test).
//!
//! Mechanism under test: Threat catalysts refute living claims on the
//! (Event, cognitive) slot. A violence-heavy regime (conflict-dense world)
//! should therefore show:
//!   1. REFUTATION: Refuted claims accumulate (norm-free window),
//!   2. CHURN: the ActiveTension→Refuted→(re-advance)→… cycle keeps the
//!      claim mix churning vs a calm control (more Undiscovered/tension
//!      turnover at the contested slot),
//!   3. RE-CRYSTALLIZATION: when the storm passes (fewer threats), claims
//!      re-advance and integrate — Integrated count recovers.
//!
//! Method: same seed, two regimes via scenario (calm vs a conflict-dense
//! scenario); measure polarity-state mix at the contested slot across
//! time. If refutation is live, the calm/drought conflict delta and the
//! Refuted census diverge measurably.

use mindstrata_development::line::LineId;
use mindstrata_development::polarity::{GrossReferent, PolarityState};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn census(sim: &Simulation) -> (usize, usize, usize, usize) {
    let cog = LineId::new("cognitive").expect("registered");
    let mut integrated = 0;
    let mut tension = 0;
    let mut refuted = 0;
    let mut event_slot = 0usize;
    for a in &sim.agents {
        for c in &a.polarity_claims {
            // The contested slot: Event referent on the cognitive line.
            if c.referent == GrossReferent::Event && c.line == cog {
                event_slot += 1;
                match c.polarity {
                    PolarityState::Integrated => integrated += 1,
                    PolarityState::ActiveTension => tension += 1,
                    PolarityState::Refuted => refuted += 1,
                    PolarityState::Undiscovered => {}
                }
            }
        }
    }
    (integrated, tension, refuted, event_slot)
}

fn run_named(sc: Scenario, label: &str) {
    let horizon = 20_000u64;
    let mut sc = sc;
    sc.ticks = horizon;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(horizon);
    let (integrated, tension, refuted, slot) = census(&sim);
    let events = sim.event_count();
    println!(
        "{label:>8}: slot_claims={slot:>4} integrated={integrated:>3} tension={tension:>3} refuted={refuted:>3} | events={events}"
    );
}

fn main() {
    run_named(Scenario::calm(), "calm");
    run_named(Scenario::drought(), "drought");
    run_named(Scenario::collapse(), "collapse");
}
