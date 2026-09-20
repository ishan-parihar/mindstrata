//! Shared test utilities for Mindstrata integration and snapshot tests.

use mindstrata_sim::parameters::SimParameters;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::{sim::SimConfig, Simulation};

/// Run a simulation with given seed and tick count, returning it for inspection.
///
/// Uses a 16×16 world with 12 agents — the standard test configuration.
pub fn run_sim(seed: u64, ticks: u64) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    sim.run(ticks);
    sim
}

/// Collect every `ChildBorn` tick over a long run **without relying on an
/// unbounded event history**.
///
/// i327 bounded the rolling event buffer (`MAX_EVENTS`, amortized bulk drop),
/// so `recent_events(usize::MAX)` no longer spans a 175K–220K-tick run — the
/// old whole-run scans in the conception/birth pins silently lost their early
/// births and read 0. Bounded journals are the charter contract (§ASSET-
/// PIPELINE-v0 rule 4), so the pins observe **incrementally** instead: the run
/// is stepped in segments, and each segment's window is read straight after it
/// and filtered to that segment's tick range. Segment event volume (~15–30
/// events/tick × `step`) stays far under the buffer bound, and the tick-range
/// filter makes the result independent of any trim.
pub fn collect_child_born_ticks(sim: &mut Simulation, total: u64, step: u64) -> Vec<u64> {
    let mut out = Vec::new();
    let mut done = 0u64;
    while done < total {
        let seg = step.min(total - done);
        let lo = done;
        sim.run(seg);
        done += seg;
        for e in sim.recent_events(usize::MAX) {
            if let mindstrata_core::event::SimEvent::ChildBorn { tick, .. } = e {
                let t = tick.as_u64();
                if t > lo && t <= done {
                    out.push(t);
                }
            }
        }
    }
    out.sort_unstable();
    out
}

/// Run a simulation with mutated tuning parameters, returning it for inspection.
///
/// The mutation is applied to `sim.params` BEFORE `populate`/`run`, so every
/// consumer that reads a parameter during the run sees the override. Same seed
/// and parameter values produce the identical RNG stream and outcome — the
/// contract the behavioral-delta harness (`behavioral_delta` module) relies on.
///
/// Uses a 16×16 world with 12 agents — the standard test configuration.
pub fn run_sim_with_params(
    seed: u64,
    ticks: u64,
    modify: impl FnOnce(&mut SimParameters),
) -> Simulation {
    let config = SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    modify(&mut sim.params);
    sim.populate();
    sim.run(ticks);
    sim
}

/// Run a simulation from a scenario with mutated parameters, returning it.
///
/// Clones the scenario (overriding ticks to the requested horizon), constructs
/// a `Simulation` via `from_scenario`, applies the parameter mutation BEFORE
/// `populate`/`run` (same-seed determinism: same clone → identical RNG
/// stream), and runs with the scenario's shocks firing at their `at_tick`
/// times. Use this to probe consumer liveness in non-calm contexts (drought,
/// famine) where calm-window-inert consumers actually fire.
pub fn run_scenario(scenario: &Scenario, seed: u64, ticks: u64) -> Simulation {
    let mut sc = scenario.clone();
    sc.seed = seed;
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(ticks);
    sim
}

/// Run a simulation from a scenario with mutated parameters, returning it.
pub fn run_scenario_with_params(
    scenario: &Scenario,
    ticks: u64,
    modify: impl FnOnce(&mut SimParameters),
) -> Simulation {
    let mut sc = scenario.clone();
    sc.ticks = ticks;
    let mut sim = Simulation::from_scenario(sc);
    modify(&mut sim.params);
    sim.populate();
    sim.run(ticks);
    sim
}
