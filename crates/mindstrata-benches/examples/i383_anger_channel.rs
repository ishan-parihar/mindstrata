//! i383 — is the ANGER channel dead, and where?
//!
//! i379's gate census flagged `emotions.anger > 0.50` as near-dead at town scale
//! (p95 0.005–0.071) and queued it "re-anchor or delete, explicitly". The same
//! absolute 0.5 appears at TWO sites, and they have different jobs:
//!
//!   * `pass_action.rs` — the intention-abandonment SHOCK, where it is ORed with
//!     `fear > 0.5`. If anger never opens but fear does, the composite is live and
//!     the anger arm is a **false affordance** (the i372 class), not a dead
//!     producer;
//!   * `systems/mod.rs` — the "high anger → Work (aggressive productivity)" goal
//!     emitter, where anger is the ONLY gate. If it never opens, that goal source
//!     is **dead** — a producer death that no test can see, because nothing pins
//!     the goal's existence.
//!
//! This probe measures, per world and per agent: the anger distribution (p50 / p95
//! / p99 / MAX — does the bar EVER open?), both gates' open rates, and the share of
//! the anger tail a population-relative candidate would open (anger above the
//! population's own p90), so the repair is sized against the corpus that proves
//! the defect (the §4.11 rule i382 paid for).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i383_anger_channel`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

struct AngerStats {
    /// Every sampled agent-anger value (distribution behind the bar).
    values: Vec<f64>,
    /// Ticks where at least one agent cleared the anger arm of the shock gate.
    anger_shock_ticks: usize,
    /// Ticks where at least one agent cleared the FEAR arm of the same gate.
    fear_shock_ticks: usize,
    /// Ticks where at least one agent cleared the Work-emitter gate
    /// (`anger > 0.5 && hunger < 0.8`).
    work_emitter_ticks: usize,
    /// Share of ticks where some agent sits in the population's own top decile
    /// (the candidate relative form — measured per sample, not pooled).
    tail_ticks: usize,
    samples: usize,
    agents: usize,
    // ── candidate relative forms over state the engine ALREADY has ──
    /// agent-ticks where `fear > 1.25 × derived.trauma_risk` (the chronic
    /// threat index — the i381 anomaly multiple, per-agent reference).
    fear_anomaly: usize,
    /// agent-ticks where `anger > 1.25 × derived.resentment` (chronic injustice).
    anger_anomaly: usize,
    /// agent-ticks where `anger > 1.25 × the POPULATION's mean anger this tick`
    /// (the self-normalizing reference — no per-agent chronic anger state exists).
    anger_pop_anomaly: usize,
    /// agent-ticks where `anger > 0.5` — the like-for-like comparison to the
    /// candidate (the tick-level share above counts ANY agent, not the share of
    /// agents).
    anger_abs_agents: usize,
    /// agent-ticks where `fear > 0.5` (same reason).
    fear_abs_agents: usize,
    /// i383 shipped law, in situ: agent-ticks carrying an ANGER-sourced Work goal
    /// (the emitter the absolute bar left dead in 7 of 10 worlds).
    anger_work_goals: usize,
    /// The same for the fear-sourced SeekSafety goal — the live control.
    fear_safety_goals: usize,
    /// agent-ticks sampled (for per-agent open rates).
    agent_samples: usize,
    /// pooled fear / trauma_risk / anger / resentment sums (ratio sanity).
    fear_sum: f64,
    trauma_sum: f64,
    anger_sum: f64,
    resentment_sum: f64,
}

fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    sorted[(((sorted.len() - 1) as f64) * q).round() as usize]
}

fn run(label: &str, scenario: Option<Scenario>, seed: u64, ticks: u64, w: u32, h: u32, n: u32) {
    let mut sim = match scenario {
        Some(mut sc) => {
            sc.seed = seed;
            sc.ticks = ticks;
            sc.world_width = w;
            sc.world_height = h;
            sc.num_agents = n;
            Simulation::from_scenario(sc)
        }
        None => Simulation::new(SimConfig {
            seed,
            max_ticks: ticks,
            world_width: w,
            world_height: h,
            num_agents: n,
            snapshot_interval: None,
        }),
    };
    sim.populate();

    let mut st = AngerStats {
        values: Vec::new(),
        anger_shock_ticks: 0,
        fear_shock_ticks: 0,
        work_emitter_ticks: 0,
        anger_work_goals: 0,
        fear_safety_goals: 0,
        tail_ticks: 0,
        samples: 0,
        agents: 0,
        fear_anomaly: 0,
        anger_anomaly: 0,
        anger_pop_anomaly: 0,
        anger_abs_agents: 0,
        fear_abs_agents: 0,
        agent_samples: 0,
        fear_sum: 0.0,
        trauma_sum: 0.0,
        anger_sum: 0.0,
        resentment_sum: 0.0,
    };
    let step = 25u64;
    let mut done = 0u64;
    while done < ticks {
        let seg = step.min(ticks - done);
        sim.run(seg);
        done += seg;
        let mut anger_any = false;
        let mut fear_any = false;
        let mut work_any = false;
        let mut tick_anger: Vec<f64> = Vec::with_capacity(sim.agents.len());
        for a in &sim.agents {
            let anger = a.emotions.anger.to_f64();
            let fear = a.emotions.fear.to_f64();
            let hunger = a.needs.hunger.to_f64();
            let trauma = a.derived.trauma_risk.to_f64();
            let resentment = a.derived.resentment.to_f64();
            st.values.push(anger);
            tick_anger.push(anger);
            st.agent_samples += 1;
            st.fear_sum += fear;
            st.trauma_sum += trauma;
            st.anger_sum += anger;
            st.resentment_sum += resentment;
            if fear > 1.25 * trauma {
                st.fear_anomaly += 1;
            }
            if anger > 1.25 * resentment {
                st.anger_anomaly += 1;
            }
            if anger > 0.5 {
                anger_any = true;
                st.anger_abs_agents += 1;
            }
            if fear > 0.5 {
                st.fear_abs_agents += 1;
            }
            if fear > 0.5 {
                fear_any = true;
            }
            if anger > 0.5 && hunger < 0.8 {
                work_any = true;
            }
            for g in &a.goals {
                match (g.source, g.kind) {
                    (
                        mindstrata_sim::person::GoalSource::Emotion,
                        mindstrata_sim::person::GoalKind::Work,
                    ) => {
                        st.anger_work_goals += 1;
                    }
                    (
                        mindstrata_sim::person::GoalSource::Emotion,
                        mindstrata_sim::person::GoalKind::SeekSafety,
                    ) => {
                        st.fear_safety_goals += 1;
                    }
                    _ => {}
                }
            }
        }
        // Candidate relative form: the population's own top decile this tick.
        // The threshold is a QUANTILE of the live distribution, so it cannot go
        // dark the way an absolute bar can (it always opens for someone).
        let pop_mean_anger = tick_anger.iter().sum::<f64>() / tick_anger.len().max(1) as f64;
        st.anger_pop_anomaly += tick_anger
            .iter()
            .filter(|a| **a > 1.25 * pop_mean_anger)
            .count();
        let mut sorted = tick_anger.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let p90 = pct(&sorted, 0.90);
        if tick_anger.iter().any(|a| *a > p90) {
            st.tail_ticks += 1;
        }
        st.samples += 1;
        st.agents = sim.agents.len();
        if anger_any {
            st.anger_shock_ticks += 1;
        }
        if fear_any {
            st.fear_shock_ticks += 1;
        }
        if work_any {
            st.work_emitter_ticks += 1;
        }
    }

    let mut sorted = st.values.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let samples = st.samples as f64;
    println!(
        "══ {label} (seed {seed}, {ticks} ticks, N={}) ══",
        st.agents
    );
    println!(
        "  anger per agent  min {:.3}  p50 {:.3}  p90 {:.3}  p95 {:.3}  p99 {:.3}  MAX {:.3}",
        pct(&sorted, 0.0),
        pct(&sorted, 0.50),
        pct(&sorted, 0.90),
        pct(&sorted, 0.95),
        pct(&sorted, 0.99),
        pct(&sorted, 1.0)
    );
    println!(
        "  arm open rate   anger>0.5 {:>6.2}%   fear>0.5 {:>6.2}%   anger>0.5&hunger<0.8 {:>6.2}%   top-decile {:>6.2}%",
        100.0 * st.anger_shock_ticks as f64 / samples,
        100.0 * st.fear_shock_ticks as f64 / samples,
        100.0 * st.work_emitter_ticks as f64 / samples,
        100.0 * st.tail_ticks as f64 / samples
    );
    let anger_only = st
        .anger_shock_ticks
        .saturating_sub(st.anger_shock_ticks.min(st.fear_shock_ticks));
    println!(
        "  abandonment shock: live ticks {} (anger-only {anger_only}) → the composite is {}",
        st.anger_shock_ticks,
        if st.anger_shock_ticks > 0 {
            if anger_only > 0 {
                "carried by BOTH arms"
            } else {
                "carried by FEAR alone once anger is counted out"
            }
        } else if st.fear_shock_ticks > 0 {
            "carried by FEAR alone"
        } else {
            "dark"
        }
    );
    println!(
        "  Work-from-anger emitter (anger>0.5 & hunger<0.8): {} ({} sample ticks)",
        if st.work_emitter_ticks == 0 {
            "DEAD"
        } else {
            "live"
        },
        st.work_emitter_ticks
    );
    let n = st.agent_samples as f64;
    println!(
        "  pooled levels  fear {:.3} vs trauma_risk {:.3} (x{:.2})   anger {:.3} vs resentment {:.3} (x{:.2})",
        st.fear_sum / n,
        st.trauma_sum / n,
        st.fear_sum / st.trauma_sum.max(1e-9),
        st.anger_sum / n,
        st.resentment_sum / n,
        st.anger_sum / st.resentment_sum.max(1e-9)
    );
    println!("  per-AGENT open share  (absolute vs candidate)");
    println!(
        "    fear:  >0.5 {:>6.2}%   vs  >1.25*trauma {:>6.2}%",
        100.0 * st.fear_abs_agents as f64 / n,
        100.0 * st.fear_anomaly as f64 / n
    );
    println!(
        "    anger: >0.5 {:>6.2}%   vs  >1.25*pop_mean {:>6.2}%   vs  >1.25*resentment {:>6.2}%",
        100.0 * st.anger_abs_agents as f64 / n,
        100.0 * st.anger_pop_anomaly as f64 / n,
        100.0 * st.anger_anomaly as f64 / n
    );
    // The shipped law, in situ: emotion-sourced goals actually held per
    // agent-ticks (the emitter the probe above measured as DEAD on the absolute
    // bar). `SeekSafety` is the live control — fear's emitter never went dark.
    println!(
        "  SHIPPED i383 goals per agent-tick: anger-sourced Work {:.6} ({} total)   fear-sourced SeekSafety {:.6}",
        st.anger_work_goals as f64 / n,
        st.anger_work_goals,
        st.fear_safety_goals as f64 / n
    );
    println!();
}

fn main() {
    println!("i383 — the anger channel at both of its absolute-0.5 sites\n");
    for seed in [42u64, 7, 23] {
        run("CALM village 16x16 N=12", None, seed, 30_000, 16, 16, 12);
    }
    for seed in [42u64, 7] {
        run("CALM town 46x46 N=48", None, seed, 15_000, 46, 46, 48);
        run(
            "PESTILENCE town 46x46 N=48",
            Some(Scenario::pestilence()),
            seed,
            15_000,
            46,
            46,
            48,
        );
        run(
            "COLLAPSE village 16x16 N=12",
            Some(Scenario::collapse()),
            seed,
            4_320,
            16,
            16,
            12,
        );
        run(
            "FAMINE village 16x16 N=12",
            Some(Scenario::famine()),
            seed,
            15_000,
            16,
            16,
            12,
        );
    }
}
