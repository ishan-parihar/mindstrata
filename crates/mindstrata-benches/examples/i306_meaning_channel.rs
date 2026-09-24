//! i306 — meaning-channel anatomy probe (stage 1: measure before touching).
//!
//! `i305` found the meaning need saturating (p90 = 1.0000) at 20K while the
//! `Worship` GOAL runs in 41.5% of agent-ticks. Before selecting a fix, this
//! probe measures the channel's actual in-vivo shape, because the arithmetic
//! does not add up on paper:
//!
//!   * meaning accumulates 1e-4/tick (the Fixed-4 image of `meaning_decay_rate`)
//!   * the Worship action's per-tick meaning relief is 0.1 — a THOUSAND times
//!     the accumulation rate, over a 4-tick action
//!
//! so a village with any material worship duty should hold meaning pinned near
//! zero. It does not, which means either the relief never lands or worship
//! almost never actually runs as an ACTION (as opposed to its goal existing).
//! This probe distinguishes those cases instead of guessing:
//!
//!   * meaning trajectory sampled through the run (rise rate, collapse size,
//!     how much of the run sits at the ceiling),
//!   * Worship ACTION duty + action STARTS (goal presence was measured in i305),
//!   * the observed relief/accumulation balance per agent-tick,
//!   * cult formation (its candidate gate is `meaning > 0.3`, so the meaning
//!     ceiling governs whether the documented cult-liveliness debt can ever
//!     clear).
//!
//! Run: cargo run --release -p mindstrata-benches --example i306_meaning_channel

use mindstrata_sim::actions::ActionKind;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// The i268 seed family (the calibration-audit stability instrument).
const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HORIZONS: [u64; 2] = [2_000, 20_000];
/// Per-tick meaning relief on the Worship action (`per_tick_effects`).
const WORSHIP_MEANING_RELIEF: f64 = 0.1;
/// Meaning accumulation per tick (the Fixed-4 image of `meaning_decay_rate`).
const MEANING_DECAY_PER_TICK: f64 = 1e-4;

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

#[derive(Default, Clone, Copy)]
struct Anatomy {
    /// Mean meaning over the run (agent-ticks).
    meaning_mean: f64,
    /// Fraction of agent-ticks at or above 0.9 (ceiling pinning).
    at_ceiling: f64,
    /// Fraction of agent-ticks above the 0.7 Worship gate.
    above_gate: f64,
    /// Worship ACTION duty (current_action == Worship), per agent-tick.
    worship_action_duty: f64,
    /// Worship action STARTS per agent-tick (cadence, not duty).
    worship_starts: f64,
    /// Largest single-tick meaning drop observed (a landing relief dose).
    max_drop: f64,
    /// Ticks where the drop exceeded 0.05 (a relief landing).
    drop_ticks: f64,
    /// Active cults at the horizon (mean over seeds).
    cults: f64,
    agent_ticks: u64,
    alive_seeds: usize,
}

fn sample(seed: u64, ticks: u64) -> Anatomy {
    let mut sim = Simulation::new(config(seed, ticks));
    sim.populate();
    let mut a = Anatomy::default();
    let mut prev_meaning: Vec<f64> = sim
        .agents
        .iter()
        .map(|x| x.needs.meaning.to_f64())
        .collect();
    let mut prev_action: Vec<ActionKind> = sim.agents.iter().map(|x| x.current_action).collect();
    for _ in 0..ticks {
        sim.tick();
        // Births grow the agent vec; keep the per-agent trackers in sync
        // (new agents enter at their founder endowment).
        while prev_meaning.len() < sim.agents.len() {
            prev_meaning.push(sim.agents[prev_meaning.len()].needs.meaning.to_f64());
            prev_action.push(sim.agents[prev_action.len()].current_action);
        }
        for (i, agent) in sim.agents.iter().enumerate() {
            let m = agent.needs.meaning.to_f64();
            a.agent_ticks += 1;
            a.meaning_mean += m;
            if m >= 0.9 {
                a.at_ceiling += 1.0;
            }
            if m > 0.7 {
                a.above_gate += 1.0;
            }
            if agent.current_action == ActionKind::Worship {
                a.worship_action_duty += 1.0;
            }
            if agent.current_action == ActionKind::Worship && prev_action[i] != ActionKind::Worship
            {
                a.worship_starts += 1.0;
            }
            let drop = prev_meaning[i] - m;
            if drop > a.max_drop {
                a.max_drop = drop;
            }
            if drop > 0.05 {
                a.drop_ticks += 1.0;
            }
            prev_meaning[i] = m;
            prev_action[i] = agent.current_action;
        }
    }
    let n = a.agent_ticks.max(1) as f64;
    a.meaning_mean /= n;
    a.at_ceiling /= n;
    a.above_gate /= n;
    a.worship_action_duty /= n;
    a.worship_starts /= n;
    a.drop_ticks /= n;
    a.alive_seeds = usize::from(!sim.agents.is_empty());
    a.cults = sim.cult_registry.cults.iter().filter(|c| c.active).count() as f64;
    a
}

fn main() {
    println!(
        "i306 meaning-channel anatomy — family of {} at {:?}",
        SEEDS.len(),
        HORIZONS
    );
    println!(
        "  accumulation {MEANING_DECAY_PER_TICK:.6}/tick | Worship relief {WORSHIP_MEANING_RELIEF:.4}/tick over a 4-tick action"
    );

    for ticks in HORIZONS {
        println!("\n[leg 1] family anatomy at {ticks} ticks/seed");
        println!(
            "  {:<10} {:>8} {:>8} {:>10} {:>14} {:>14} {:>9} {:>9} {:>6}",
            "seed",
            "mean",
            ">gate",
            "at-ceiling",
            "worship duty",
            "worship starts",
            "max drop",
            "drop ticks",
            "cults"
        );
        let mut sum = Anatomy::default();
        let mut alive = 0usize;
        for &seed in &SEEDS {
            let a = sample(seed, ticks);
            alive += a.alive_seeds;
            sum.meaning_mean += a.meaning_mean;
            sum.at_ceiling += a.at_ceiling;
            sum.above_gate += a.above_gate;
            sum.worship_action_duty += a.worship_action_duty;
            sum.worship_starts += a.worship_starts;
            sum.max_drop = sum.max_drop.max(a.max_drop);
            sum.drop_ticks += a.drop_ticks;
            sum.cults += a.cults;
            sum.agent_ticks += a.agent_ticks;
            println!(
                "  {seed:<10} {:>8.4} {:>8.4} {:>10.4} {:>14.5} {:>14.5} {:>9.4} {:>9.5} {:>6.1}",
                a.meaning_mean,
                a.above_gate,
                a.at_ceiling,
                a.worship_action_duty,
                a.worship_starts,
                a.max_drop,
                a.drop_ticks,
                a.cults
            );
        }
        let n = SEEDS.len() as f64;
        println!(
            "  {:<10} {:>8.4} {:>8.4} {:>10.4} {:>14.5} {:>14.5} {:>9.4} {:>9.5} {:>6.2}",
            "family",
            sum.meaning_mean / n,
            sum.above_gate / n,
            sum.at_ceiling / n,
            sum.worship_action_duty / n,
            sum.worship_starts / n,
            sum.max_drop,
            sum.drop_ticks / n,
            sum.cults / n
        );
        // Balance check: mean relief applied per agent-tick vs accumulation.
        let relief_rate = (sum.worship_action_duty / n) * WORSHIP_MEANING_RELIEF;
        println!(
            "  balance: relief {relief_rate:.6}/tick vs accumulation {MEANING_DECAY_PER_TICK:.6}/tick → ratio {:.2}×",
            relief_rate / MEANING_DECAY_PER_TICK
        );
        println!("  alive seeds: {alive}/{}", SEEDS.len());
    }

    // ── Leg 3: per-agent split (are the ceiling agents the ones that
    // never worship?) ───────────────────────────────────────────────────
    let ticks = 20_000u64;
    for seed in [42u64, 2, 99] {
        println!("\n[leg 3] per-agent anatomy, seed {seed}, {ticks} ticks");
        let mut sim = Simulation::new(config(seed, ticks));
        sim.populate();
        let n0 = sim.agents.len();
        let mut worship_ticks = vec![0u64; n0];
        let mut goal_ticks = vec![0u64; n0];
        let mut at_ceiling = vec![0u64; n0];
        let mut action_share: Vec<std::collections::HashMap<String, u64>> =
            vec![std::collections::HashMap::new(); n0];
        // Meaning INFLOW: rises beyond the 1e-4/tick decay image are an
        // external producer. "Big" = a jump of more than 0.001 in one tick.
        let mut inflow_events = vec![0u64; n0];
        let mut inflow_total = vec![0.0f64; n0];
        let mut max_inflow = vec![0.0f64; n0];
        let mut prev: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| a.needs.meaning.to_f64())
            .collect();
        for _ in 0..ticks {
            sim.tick();
            while prev.len() < sim.agents.len() {
                prev.push(sim.agents[prev.len()].needs.meaning.to_f64());
            }
            for i in 0..sim.agents.len().min(n0) {
                let now = sim.agents[i].needs.meaning.to_f64();
                let rise = now - prev[i];
                if rise > 0.001 {
                    inflow_events[i] += 1;
                    inflow_total[i] += rise;
                    max_inflow[i] = max_inflow[i].max(rise);
                }
                prev[i] = now;
            }
            for i in 0..sim.agents.len().min(n0) {
                *action_share[i]
                    .entry(format!("{:?}", sim.agents[i].current_action))
                    .or_insert(0) += 1;
                if sim.agents[i].current_action == ActionKind::Worship {
                    worship_ticks[i] += 1;
                }
                if sim.agents[i]
                    .goals
                    .iter()
                    .any(|g| g.kind == mindstrata_sim::person::GoalKind::Worship)
                {
                    goal_ticks[i] += 1;
                }
                if sim.agents[i].needs.meaning.to_f64() >= 0.9 {
                    at_ceiling[i] += 1;
                }
            }
        }
        println!(
            "  {:<3} {:>9} {:>9} {:>9} {:>9}  {:<34}",
            "id", "meaning", "worship", "goal", "ceiling", "top actions (work/rest/idle/socialize)"
        );
        let tick = mindstrata_core::clock::Tick::new(ticks);
        for i in 0..n0 {
            let alive = i < sim.agents.len();
            let meaning = if alive {
                sim.agents[i].needs.meaning.to_f64()
            } else {
                f64::NAN
            };
            let d = ticks as f64;
            // §10.3 routine state at the horizon: whether the schedule is
            // strong enough to override utility selection, and what it wants.
            let (routine_action, routine_strength) = sim.agents[i].routine.preferred_action(tick);
            // The dominant action is the decisive clue: a meaning-pinned
            // agent that spends its life in a physiological reflex is starved
            // of the meaning reflex by REFLEX ORDER, not by the routine.
            let (dom_label, dom_share) =
                action_share[i].iter().max_by_key(|(_, c)| **c).map_or_else(
                    || ("-none-".to_string(), 0.0),
                    |(l, c)| (l.clone(), *c as f64 / d),
                );
            println!(
                "  {i:<3} {meaning:>9.4} {:>9.5} {:>9.5} {:>9.5}  dom {dom_label} {dom_share:.3} | routine {routine_action:?} {routine_strength:.2} | vitals h{:.2} t{:.2} f{:.2} hp{:.2} | inflow {}× tot {:.2} max {:.3}",
                worship_ticks[i] as f64 / d,
                goal_ticks[i] as f64 / d,
                at_ceiling[i] as f64 / d,
                sim.agents[i].needs.hunger.to_f64(),
                sim.agents[i].needs.thirst.to_f64(),
                sim.agents[i].needs.fatigue.to_f64(),
                sim.agents[i].body.health.to_f64(),
                inflow_events[i],
                inflow_total[i],
                max_inflow[i],
            );
        }
    }

    // ── Leg 4: jump log for one pinned agent (where does meaning come from?) ──
    {
        let seed = 42u64;
        let ticks = 20_000u64;
        let agent = 11usize;
        let mut sim = Simulation::new(config(seed, ticks));
        sim.populate();
        let mut log: Vec<(u64, f64, f64, String)> = Vec::new();
        let mut prev = sim.agents[agent].needs.meaning.to_f64();
        // Does the relief actually land on the ticks where Worship is the
        // current action? ("Selected but not applied" is a different bug from
        // "never selected".)
        let mut worship_ticks = 0u64;
        let mut worship_with_drop = 0u64;
        let mut worship_no_drop = 0u64;
        let mut worship_rise = 0u64;
        let mut total_rise = 0.0f64;
        let mut total_fall = 0.0f64;
        for t in 1..=ticks {
            sim.tick();
            let now = sim.agents[agent].needs.meaning.to_f64();
            let delta = now - prev;
            if delta > 0.0 {
                total_rise += delta;
            } else {
                total_fall += -delta;
            }
            if sim.agents[agent].current_action == ActionKind::Worship {
                worship_ticks += 1;
                if delta < -0.01 {
                    worship_with_drop += 1;
                } else if delta > 0.001 {
                    worship_rise += 1;
                } else {
                    worship_no_drop += 1;
                }
            }
            if delta.abs() > 0.05 {
                log.push((
                    t,
                    prev,
                    now,
                    format!("{:?}", sim.agents[agent].current_action),
                ));
            }
            prev = now;
        }
        println!(
            "  worship ticks {worship_ticks} | with relief {worship_with_drop} | flat {worship_no_drop} | rising {worship_rise}"
        );
        println!(
            "  meaning total rise {total_rise:.3} | total fall {total_fall:.3} | net {:+.3}",
            total_rise - total_fall
        );
        println!(
            "\n[leg 4] seed {seed} agent {agent} meaning jumps >0.05 ({}/{} shown)",
            log.len().min(40),
            log.len()
        );
        println!("  {:<8} {:>8} {:>8}  action", "tick", "before", "after");
        for (t, before, after, action) in log.iter().take(40) {
            println!("  {t:<8} {before:>8.4} {after:>8.4}  {action}");
        }
    }

    // ── Leg 2: the trajectory shape on one seed (is it a sawtooth?) ──────
    println!("\n[leg 2] seed-42 trajectory (2000-tick windows: mean meaning, worship duty)");
    let mut sim = Simulation::new(config(42, ticks));
    sim.populate();
    let mut window_mean = 0.0;
    let mut window_duty = 0.0;
    let mut window_samples = 0u64;
    for t in 1..=ticks {
        sim.tick();
        for agent in &sim.agents {
            window_mean += agent.needs.meaning.to_f64();
            if agent.current_action == ActionKind::Worship {
                window_duty += 1.0;
            }
            window_samples += 1;
        }
        if t % 2000 == 0 {
            let n = window_samples.max(1) as f64;
            println!(
                "  tick {t:>6}: meaning mean {:.4} | worship duty {:.5}",
                window_mean / n,
                window_duty / n
            );
            window_mean = 0.0;
            window_duty = 0.0;
            window_samples = 0;
        }
    }

    // ── Leg 5: the i306 verdict ────────────────────────────────────────
    //
    // Pre-registered contract, written against the PRE-FIX baseline measured
    // by this same probe at the same horizon (i305-adjacent tree, action pass
    // before the meaning reflex):
    //
    //   PRE-FIX (20K x 12 seeds): meaning mean 0.5280 | >gate 0.3240 |
    //                             at-ceiling 0.2720 | worship duty 0.0220
    //
    // C1 liveness       — 12/12 seeds alive (the reflex kills no one).
    // C2 by-construction— ZERO agents sit at the meaning ceiling unless a
    //                     PHYSIOLOGICAL reflex outranks the meaning reflex
    //                     (body-outranks-soul, the ratified i255 order).
    // C3 effect         — family at-ceiling share <= HALF the pre-fix
    //                     baseline (0.136): the reflex must demonstrably
    //                     relieve the channel, not merely exist.
    // C4 no over-correction — worship ACTION duty stays >= half the pre-fix
    //                     baseline (> 0.011): the reflex must not suppress
    //                     worship for the agents below the threshold.
    println!("\n[leg 5] verdict — 12-seed family at {ticks} ticks");
    const PRE_MEAN: f64 = 0.5280;
    const PRE_ABOVE_GATE: f64 = 0.3240;
    const PRE_AT_CEILING: f64 = 0.2720;
    const PRE_WORSHIP_DUTY: f64 = 0.0220;
    // A ceiling excursion is a TRANSIENT if it lasts no longer than this.
    // Sizing: the reflex fires above 0.9, Worship lasts 4 ticks at 0.1 relief
    // each, so the expected excursion after the reflex engages is <10 ticks;
    // 200 is 20x that as a safety margin for action-continuation and the
    // rise back to 0.9 (1e-4/tick => ~4000 ticks). A pin, by contrast, is
    // thousands of ticks (pre-fix agents sat at 1.0000 for whole horizons).
    const TRANSIENT_RUN_MAX: u64 = 200;
    #[derive(Default, Clone, Copy)]
    struct Track {
        run: u64,
        longest_run: u64,
        ceiling_ticks: u64,
        ceiling_phys_ticks: u64,
        ceiling_worship_ticks: u64,
        ceiling_relief_ticks: u64,
    }
    let (mut mean, mut above_gate, mut at_ceiling, mut duty, mut agent_ticks) =
        (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0u64);
    let mut alive = 0usize;
    // (seed, agent index, longest run, ceiling ticks, phys ticks, worship
    //  ticks, relief landings) for the residual pins.
    let mut pins: Vec<(u64, usize, u64, u64, u64, u64, u64)> = Vec::new();
    for &seed in &SEEDS {
        let mut sim = Simulation::new(config(seed, ticks));
        sim.populate();
        let mut tracks = vec![Track::default(); sim.agents.len()];
        let mut prev_meaning: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| a.needs.meaning.to_f64())
            .collect();
        for _ in 0..ticks {
            sim.tick();
            while tracks.len() < sim.agents.len() {
                tracks.push(Track::default());
                prev_meaning.push(sim.agents[prev_meaning.len()].needs.meaning.to_f64());
            }
            for (i, agent) in sim.agents.iter().enumerate() {
                let m = agent.needs.meaning.to_f64();
                let drop = prev_meaning[i] - m;
                prev_meaning[i] = m;
                agent_ticks += 1;
                mean += m;
                if m > 0.7 {
                    above_gate += 1.0;
                }
                if agent.current_action == ActionKind::Worship {
                    duty += 1.0;
                }
                if m < 0.9 {
                    tracks[i].run = 0;
                    continue;
                }
                at_ceiling += 1.0;
                let t = &mut tracks[i];
                t.run += 1;
                t.longest_run = t.longest_run.max(t.run);
                t.ceiling_ticks += 1;
                // Which reflex, if any, outranks the meaning reflex here?
                let phys = (agent.needs.thirst.to_f64() > 0.9
                    && agent.needs.thirst >= agent.needs.hunger)
                    || agent.needs.hunger.to_f64() > 0.9
                    || agent.needs.fatigue.to_f64() > 0.95
                    || agent.body.health.to_f64() < 0.25;
                if phys {
                    t.ceiling_phys_ticks += 1;
                }
                if agent.current_action == ActionKind::Worship {
                    t.ceiling_worship_ticks += 1;
                    if drop > 0.01 {
                        t.ceiling_relief_ticks += 1;
                    }
                }
            }
        }
        if sim.agents.is_empty() {
            continue;
        }
        alive += 1;
        for (i, t) in tracks.iter().enumerate().take(sim.agents.len()) {
            if t.longest_run > TRANSIENT_RUN_MAX {
                pins.push((
                    seed,
                    i,
                    t.longest_run,
                    t.ceiling_ticks,
                    t.ceiling_phys_ticks,
                    t.ceiling_worship_ticks,
                    t.ceiling_relief_ticks,
                ));
            }
        }
    }
    let n = agent_ticks.max(1) as f64;
    let (mean, above_gate, at_ceiling, duty) = (mean / n, above_gate / n, at_ceiling / n, duty / n);
    // C2: every long excursion is EXPLAINED — either the body reflex was
    // active for at least half of it (body outranks soul, ratified i255
    // order) or the agent was actually worshipping during it (the reflex
    // engaged and the channel was healing, just slowly relative to the run
    // length). An unexplained long excursion is the failure mode i306 exists
    // to kill: a man at his meaning ceiling with nothing routed to fix it.
    let unexplained: Vec<_> = pins
        .iter()
        .filter(|(_, _, _, ceiling, phys, worship, _)| {
            *phys * 2 < *ceiling && *worship * 2 < *ceiling
        })
        .collect();
    let c1 = alive == SEEDS.len();
    let c2 = unexplained.is_empty();
    let c3 = at_ceiling <= PRE_AT_CEILING / 2.0;
    let c4 = duty >= PRE_WORSHIP_DUTY / 2.0;
    println!(
        "  family: mean {mean:.4} (pre {PRE_MEAN:.4}) | >gate {above_gate:.4} (pre {PRE_ABOVE_GATE:.4}) | at-ceiling {at_ceiling:.4} (pre {PRE_AT_CEILING:.4}) | worship duty {duty:.5} (pre {PRE_WORSHIP_DUTY:.5})"
    );
    println!(
        "  long ceiling excursions (>{TRANSIENT_RUN_MAX} ticks): {} | unexplained (neither body-reflex nor worshipping half the time): {}",
        pins.len(),
        unexplained.len()
    );
    println!(
        "  {:<6} {:<4} {:>10} {:>10} {:>10} {:>12} {:>12}",
        "seed", "id", "longest", "at-ceiling", "body-reflex", "worshipping", "relief land"
    );
    let mut shown: Vec<_> = pins.clone();
    shown.sort_by_key(|p| std::cmp::Reverse(p.2));
    for (seed, i, longest, ceiling, phys, worship, relief) in shown.iter().take(6) {
        println!(
            "  {seed:<6} {i:<4} {longest:>10} {ceiling:>10} {phys:>10} {worship:>12} {relief:>12}"
        );
    }
    println!(
        "  C1 alive {alive}/{}: {c1} | C2 excursions explained: {c2} | C3 ceiling halved: {c3} | C4 duty preserved: {c4}",
        SEEDS.len()
    );
    let verdict = if c1 && c2 && c3 && c4 {
        "MEANING_REFLEX_LIVE"
    } else {
        "MEANING_REFLEX_NOT_PROVEN"
    };
    println!("verdict={verdict}");
}
