//! i351 — event-level panic sweep (governance family {1, 7, 42} + margins):
//! counts `ConflictKind::MoralPanic` events over the pestilence 20K window,
//! post-Wander-driver.
use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

fn run_scenario(scenario: &Scenario, seed: u64, ticks: u64) -> Simulation {
    let mut sc = scenario.clone();
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(ticks);
    sim
}

fn main() {
    for seed in [1u64, 7, 42, 11, 5, 43, 44] {
        let sim = run_scenario(&Scenario::pestilence(), seed, 20000);
        let panics = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::ConflictOccurred {
                        kind: ConflictKind::MoralPanic,
                        ..
                    }
                )
            })
            .count();
        println!("seed {seed:>3}: MoralPanic events {panics}");
    }
}
