//! i389 — the directive channel: does an in-sim authority ever ask?
//!
//! DC-5's G1 lists the authority channel as **nominal**: the offices exist
//! (Elder, Guard Captain), the directive API exists (`Simulation::command_agent`,
//! five unit tests), the consumption path exists (`command_goal_action` overrides
//! routine and utility, gated so a pressing need still wins), and the census
//! counts it (`SRC_COMMAND`) — but **no shipped producer emits a
//! `GoalSource::Command` goal**, so the layer's share is 0.00% in every world
//! (i385's re-measured census: `Command channel 0% / 0%`).
//!
//! This probe is the producer census the item requires, plus the *authority
//! context* a decree emitter would have to read: legitimacy (does the office have
//! a mandate to ask), panic state, the famine window, and the population's own
//! hunger/fear. A decree producer that cannot name its crisis condition is a
//! decoration; this measures the conditions before any of them is wired.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i389_command_channel`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::decision_census::{self, SOURCE_NAMES, SRC_COMMAND};
use mindstrata_sim::sim::{SimConfig, Simulation};

const WARMUP: u64 = 500;
const WINDOW: u64 = 20_000;

enum World {
    Calm,
    Collapse,
    Pestilence,
}

fn build(world: World, seed: u64) -> Simulation {
    let mut sim = match world {
        World::Calm => Simulation::new(SimConfig {
            seed,
            max_ticks: WARMUP + WINDOW,
            world_width: 32,
            world_height: 32,
            num_agents: 12,
            snapshot_interval: None,
        }),
        World::Collapse => {
            let mut sc = Scenario::collapse();
            sc.seed = seed;
            sc.ticks = WARMUP + WINDOW;
            Simulation::from_scenario(sc)
        }
        World::Pestilence => {
            let mut sc = Scenario::pestilence();
            sc.seed = seed;
            sc.ticks = WARMUP + WINDOW;
            Simulation::from_scenario(sc)
        }
    };
    sim.populate();
    sim
}

fn run(label: &str, world: World, seed: u64) {
    let mut sim = build(world, seed);
    sim.run(WARMUP);
    decision_census::reset();
    decision_census::enable();
    // i389 §4.13: the crisis conditions must be measured, not assumed. The
    // decree system is sampled at the cadence, so the *frequency* of each
    // condition matters as much as its existence: a condition that lives for 30
    // ticks cannot carry a 250-tick cadence.
    let mut ticks_panic = 0u64;
    let mut ticks_fear = 0u64;
    let mut ticks_hunger = 0u64;
    let mut decrees_seen = 0u64;
    for _ in 0..WINDOW {
        sim.run(1);
        let n = sim.agents.len().max(1) as f64;
        let fear = sim
            .agents
            .iter()
            .map(|a| a.emotions.fear.to_f64())
            .sum::<f64>()
            / n;
        let hunger = sim
            .agents
            .iter()
            .map(|a| a.needs.hunger.to_f64())
            .sum::<f64>()
            / n;
        if !sim.moral_panic_registry.panics.is_empty() {
            ticks_panic += 1;
        }
        if fear > 0.5 {
            ticks_fear += 1;
        }
        if hunger > 0.6 {
            ticks_hunger += 1;
        }
        decrees_seen += sim
            .agents
            .iter()
            .map(|a| {
                a.goals
                    .iter()
                    .filter(|g| g.source == mindstrata_sim::person::GoalSource::Decree)
                    .count()
            })
            .sum::<usize>() as u64;
    }
    decision_census::disable();
    let r = decision_census::report();
    let total = r.total().max(1) as f64;

    let command_goals = sim
        .agents
        .iter()
        .filter(|a| {
            a.goals
                .iter()
                .any(|g| g.source == mindstrata_sim::person::GoalSource::Command)
        })
        .count();

    let council = sim
        .institutions
        .iter()
        .find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Council);
    let legitimacy = council.map(|c| c.legitimacy.to_f64()).unwrap_or(f64::NAN);
    let panics = sim.moral_panic_registry.panics.len();
    let n = sim.agents.len().max(1) as f64;
    let mean_hunger = sim
        .agents
        .iter()
        .map(|a| a.needs.hunger.to_f64())
        .sum::<f64>()
        / n;
    let mean_fear = sim
        .agents
        .iter()
        .map(|a| a.emotions.fear.to_f64())
        .sum::<f64>()
        / n;

    println!("══ {label} (seed {seed}) ══");
    print!("  deciding layer:");
    for (i, name) in SOURCE_NAMES.iter().enumerate() {
        print!("  {name} {:.2}%", r.sources[i] as f64 / total * 100.0);
    }
    println!();
    println!(
        "  directive holders at window end: {command_goals}/{} agents · census command selections: {}",
        sim.agents.len(),
        r.sources[SRC_COMMAND]
    );
    let elder = council.and_then(|c| c.get_role_holder("Elder")).is_some();
    println!(
        "  authority context: council legitimacy {legitimacy:.4} · Elder office filled {elder} · \
         active panics {panics} · mean hunger {mean_hunger:.4} · mean fear {mean_fear:.4}"
    );
    println!(
        "  crisis duty cycles over {WINDOW} ticks: panic active {:.2}% · mean fear > 0.5 {:.2}% · \
         mean hunger > 0.6 {:.2}% · decree-goal agent-ticks {decrees_seen}",
        ticks_panic as f64 / WINDOW as f64 * 100.0,
        ticks_fear as f64 / WINDOW as f64 * 100.0,
        ticks_hunger as f64 / WINDOW as f64 * 100.0
    );
    println!();
}

fn main() {
    println!("i389 — the directive channel (producer census, warmup {WARMUP}, window {WINDOW})\n");
    run("calm village", World::Calm, 42);
    run("collapse", World::Collapse, 42);
    run("pestilence", World::Pestilence, 7);
}
