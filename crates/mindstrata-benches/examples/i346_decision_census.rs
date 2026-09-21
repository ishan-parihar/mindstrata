//! i346 — the decision census: which selection branch actually decides the village?
//!
//! i338 measured `Wander`/`Move` at **0.00%** of agent-ticks and recorded the
//! whole §6 locomotion subsystem as inert (ledger A8). That measurement cannot
//! say *why*, because `pass_action` picks an action through a five-deep branch
//! chain — survival reflex → external command → daily routine → feud approach →
//! utility AI — and then applies two post-transforms (the §8.1.19 stress-habit
//! substitution and the i309/i314 exertion veto). An action can be dead because
//! the branch that owns it never runs, or because the branch runs and the action
//! loses. Those two failures have different fixes.
//!
//! The i346 census (`mindstrata_sim::sim::decision_census`) records the deciding
//! source per agent-tick, the finalized action, the cross-tab, and — on the
//! utility leg — the realized utility gap between the winner and `Wander`, which
//! is the only way to tell a *structural* wall (gap above the ±0.05 per-candidate
//! jitter) from a *calibration* one (gap inside it).
//!
//! Run: cargo run --release -p mindstrata-benches --example i346_decision_census -- [N...]

use mindstrata_sim::sim::decision_census::{self, ACTION_NAMES, NOISE_AMPLITUDE, SOURCE_NAMES};
use mindstrata_sim::sim::{SimConfig, Simulation};

const WARMUP: u64 = 500;
const WINDOW: u64 = 20_000;

fn sim_for(n: u32) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: WARMUP + WINDOW,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

/// A public-field fingerprint of the whole run, used to prove the census is
/// inert: counters can never reach a decision, so an instrumented run must be
/// byte-equivalent to an uninstrumented one.
fn fingerprint(sim: &Simulation) -> String {
    let mut acc = 0.0f64;
    for a in &sim.agents {
        acc += a.position.x as f64 * 3.0
            + a.position.y as f64 * 7.0
            + a.needs.hunger.to_f64() * 11.0
            + a.needs.thirst.to_f64() * 13.0
            + a.body.health.to_f64() * 17.0;
    }
    format!(
        "agents={} events={} journal={} tick={} fold={acc:.9}",
        sim.agent_count(),
        sim.event_count(),
        sim.journal_len(),
        sim.current_tick().as_u64(),
    )
}

fn census_leg(n: u32) {
    let mut sim = sim_for(n);
    sim.run(WARMUP);

    let tick_before = sim.current_tick().as_u64();
    decision_census::enable();
    decision_census::reset();
    sim.run(WINDOW);
    decision_census::disable();
    let r = decision_census::report();
    let total = r.total().max(1) as f64;
    let ticks = sim.current_tick().as_u64() - tick_before;

    println!(
        "\n=== N={n} (seed 42, 32x32, {WINDOW} requested / {ticks} advanced after {WARMUP} warmup) ==="
    );
    let agent_ticks = ticks * u64::from(n);
    println!(
        "    decisions {} over {agent_ticks} agent-ticks ({:.1}%) — mean action duration {:.2} ticks",
        r.total(),
        r.total() as f64 / agent_ticks.max(1) as f64 * 100.0,
        agent_ticks as f64 / r.total().max(1) as f64
    );
    println!("\n-- selection source (share of DECISIONS, not agent-ticks) --");
    println!("{:>10} {:>12} {:>8}", "source", "decisions", "share");
    for (i, name) in SOURCE_NAMES.iter().enumerate() {
        println!(
            "{:>10} {:>12} {:>7.2}%",
            name,
            r.sources[i],
            r.sources[i] as f64 / total * 100.0
        );
    }

    println!("\n-- final action (share of decisions, and which source produced it) --");
    println!(
        "{:>10} {:>12} {:>8}   {}",
        "action", "decisions", "share", "top source"
    );
    for (a, name) in ACTION_NAMES.iter().enumerate() {
        let mut best = (0usize, 0u64);
        for (s, src) in r.cross.iter().enumerate() {
            if src[a] > best.1 {
                best = (s, src[a]);
            }
        }
        println!(
            "{:>10} {:>12} {:>7.2}%   {} ({})",
            name,
            r.actions[a],
            r.actions[a] as f64 / total * 100.0,
            SOURCE_NAMES[best.0],
            best.1
        );
    }

    println!("\n-- utility AI arbitration (the `utility` row above) --");
    println!("samples {}", r.utility_samples);
    for (name, g, samples) in [
        ("Wander", r.wander, r.utility_samples),
        ("Idle", r.idle, r.utility_samples),
    ] {
        println!(
            "  {:>6}: at argmax {} ({:.3}%) · within ±{NOISE_AMPLITUDE} {} ({:.2}% of losses) · gap mean {:.4} max {:.4}",
            name,
            g.wins,
            if samples == 0 {
                0.0
            } else {
                g.wins as f64 / samples as f64 * 100.0
            },
            g.within_noise,
            if samples == g.wins {
                0.0
            } else {
                g.within_noise as f64 / (samples - g.wins) as f64 * 100.0
            },
            g.mean_loss(samples),
            g.max
        );
    }

    println!("\n-- cross-tab (source × action, non-zero) --");
    for (s, src) in r.cross.iter().enumerate() {
        let parts: Vec<String> = src
            .iter()
            .enumerate()
            .filter(|(_, c)| **c > 0)
            .map(|(a, c)| format!("{}={c}", ACTION_NAMES[a]))
            .collect();
        if !parts.is_empty() {
            println!("{:>10}: {}", SOURCE_NAMES[s], parts.join(" "));
        }
    }
}

/// Does the census itself move the simulation? Run the same configuration twice
/// in one process — counters on, then off — and compare the fingerprints.
fn inertness_leg(n: u32) {
    let mut on = sim_for(n);
    decision_census::enable();
    decision_census::reset();
    on.run(WARMUP + 1_000);

    let mut off = sim_for(n);
    decision_census::disable();
    off.run(WARMUP + 1_000);

    let (a, b) = (fingerprint(&on), fingerprint(&off));
    println!(
        "\n=== inertness (N={n}, 1500 ticks) ===\n  census on : {a}\n  census off: {b}\n  {}",
        if a == b {
            "IDENTICAL — the census cannot reach a decision"
        } else {
            "DIVERGED — instrumentation is behavioural (BUG)"
        }
    );
}

/// `Move` has exactly one producer in the whole selection chain — the §19.5.G
/// feud-approach branch — so the branch's own preconditions decide whether the
/// action is reachable *by construction*.
fn feud_leg(n: u32) {
    let mut sim = sim_for(n);
    sim.run(WARMUP);
    let mut feud_ticks = 0u64;
    let mut angry_ticks = 0u64;
    let mut both_ticks = 0u64;
    let mut max_feuds = 0usize;
    let mut max_anger = 0.0f64;
    for _ in 0..2_000 {
        sim.run(1);
        for a in &sim.agents {
            let feuding = !a.feuds.is_empty();
            let angry = a.emotions.anger.to_f64() > 0.4;
            if feuding {
                feud_ticks += 1;
            }
            if angry {
                angry_ticks += 1;
            }
            if feuding && angry {
                both_ticks += 1;
            }
            max_feuds = max_feuds.max(a.feuds.len());
            max_anger = max_anger.max(a.emotions.anger.to_f64());
        }
    }
    let n_ticks = (2_000.0 * sim.agents.len().max(1) as f64) as u64;
    println!(
        "  N={n}: non-empty feuds {feud_ticks}/{n_ticks} agent-ticks ({:.3}%), anger>0.4 {angry_ticks} ({:.3}%), BOTH (the §19.5.G gate) {both_ticks} ({:.4}%), max feuds {max_feuds}, max anger {max_anger:.4}",
        feud_ticks as f64 / n_ticks as f64 * 100.0,
        angry_ticks as f64 / n_ticks as f64 * 100.0,
        both_ticks as f64 / n_ticks as f64 * 100.0,
    );
}

/// The observable consequence: if the decisions above never produce `Wander` or
/// `Move`, position must be frozen.
fn movement_leg(n: u32) {
    let mut sim = sim_for(n);
    sim.run(WARMUP);
    let start: Vec<(i32, i32)> = sim
        .agents
        .iter()
        .map(|a| (a.position.x, a.position.y))
        .collect();
    // A site teleport (the §Phase 1.5 ecology migration path) lands exactly on
    // a site tile; the §6 movement pass takes a single clamped step. Classifying
    // each change is what says WHICH code path moved the agent.
    let site_tiles: std::collections::HashSet<(i32, i32)> = (0..sim.world().sites.len())
        .filter_map(|i| sim.world().site_position(i))
        .collect();

    let mut changes = 0u64;
    let mut teleports = 0u64;
    let mut steps = 0u64;
    let mut first_changes: Vec<String> = Vec::new();
    for _ in 0..2_000 {
        let before: Vec<(i32, i32)> = sim
            .agents
            .iter()
            .map(|a| (a.position.x, a.position.y))
            .collect();
        sim.run(1);
        for (i, a) in sim.agents.iter().enumerate() {
            let now = (a.position.x, a.position.y);
            if now == before[i] {
                continue;
            }
            changes += 1;
            if first_changes.len() < 6 {
                first_changes.push(format!(
                    "agent {i} {:?} -> {:?} action={:?}",
                    before[i], now, a.current_action
                ));
            }
            if site_tiles.contains(&now) {
                teleports += 1;
            }
            let manhattan = (now.0 - before[i].0).abs() + (now.1 - before[i].1).abs();
            if manhattan <= 2 {
                steps += 1;
            }
        }
    }
    let moved_agents = sim
        .agents
        .iter()
        .enumerate()
        .filter(|(i, a)| (a.position.x, a.position.y) != start[*i])
        .count();
    println!(
        "  N={n}: position changes {changes} over 2000 ticks ({:.4}% of agent-ticks), agents that ever moved {moved_agents}/{}; landed on a site tile {teleports}, single-step (<=2 manhattan) {steps}",
        changes as f64 / (2_000.0 * sim.agents.len().max(1) as f64) * 100.0,
        sim.agents.len()
    );
    for line in &first_changes {
        println!("      {line}");
    }
}

fn main() {
    let ns: Vec<u32> = {
        let args: Vec<u32> = std::env::args()
            .skip(1)
            .filter_map(|s| s.parse().ok())
            .collect();
        if args.is_empty() {
            vec![12, 48]
        } else {
            args
        }
    };

    println!("i346 — the decision census (seed 42, 32x32, housing default)");
    for &n in &ns {
        census_leg(n);
    }

    println!("\n\n=== §19.5.G feud-branch reachability (the only `Move` producer) ===");
    for &n in &ns {
        feud_leg(n);
    }

    println!("\n=== position-update leg ===");
    for &n in &ns {
        movement_leg(n);
    }

    for &n in &ns {
        inertness_leg(n);
    }
}
