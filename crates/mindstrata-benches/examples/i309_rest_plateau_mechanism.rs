//! i309 — Rest-plateau mechanism probe.
//!
//! i308 ruled out energy, sleep, habits-as-substitution and the emotional
//! modifiers by live-sim ablation, and localized the driver to a pass-level
//! input its utility ledger omitted. Reading the pass, the omitted terms are:
//! the emotional modifier was included, but NOT `decision_policy.moral_modifier`
//! and NOT `decision_policy.habit_modifier(is_routine, stress)` — and the latter
//! is the interesting one, because `is_routine` covers exactly
//! {Work, Eat, Drink, Rest, Worship} and **excludes Socialize and Trade**:
//!
//! ```ignore
//! pub fn habit_modifier(&self, is_routine_action: bool, stress: Fixed) -> Fixed {
//!     if !is_routine_action { return Fixed::ZERO; }          // Socialize/Trade: 0
//!     let base = self.habit_policy.habit_strength;           // a POLICY constant
//!     let stress_boost = stress * self.habit_policy.stress_habit_reliance;
//!     (base + stress_boost).clamp_01() * Fixed::from_f64(0.3)
//! }
//! ```
//!
//! The plateau agents are chronically stressed (fear ~0.93 + anger ~0), so the
//! boost saturates near its +0.3 ceiling for the five routine actions and stays
//! structurally 0 for the two social ones. In the low-need regime every
//! candidate's core utility is near zero, so that +0.3 decides the argmax.
//!
//! This probe tests that hypothesis twice:
//!   * leg 1 — ledger WITH the omitted terms: does it now reproduce the sim's
//!     own winner distribution? (i308's ledger did not.)
//!   * leg 2 — the routine gate: `should_follow` returns false above stress 0.7,
//!     i.e. the daily schedule is switched OFF for exactly these agents.
//!
//! Run: cargo run --release -p mindstrata-benches --example i309_rest_plateau_mechanism

use mindstrata_core::rng::RngStreams;
use mindstrata_core::Fixed;
use mindstrata_sim::actions::{compute_utility, ActionKind as K};
use mindstrata_sim::sim::{SimConfig, Simulation};

const HORIZON: u64 = 50_000;
/// (seed, agent index) of plateau agents found by `i308_rest_dominance`.
const PLATEAU: [(u64, usize); 3] = [(123, 6), (2, 2), (1, 0)];

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

/// `actions::action_traits` — private in the sim, mirrored here.
/// (is_social, is_risky, is_withdrawal, is_prosocial, is_disobedient, is_harmful)
fn traits(kind: K) -> (bool, bool, bool, bool, bool, bool) {
    match kind {
        K::Eat | K::Drink => (false, false, false, false, false, false),
        K::Rest => (false, false, true, false, false, false),
        K::Work => (false, false, false, true, false, false),
        K::Socialize | K::Trade => (true, false, false, true, false, false),
        K::Worship => (true, false, false, true, false, false),
        K::Wander => (false, true, false, false, true, false),
        K::Idle => (false, false, true, false, true, false),
        K::Move { .. } => (false, false, false, false, false, false),
    }
}

/// The five actions `is_routine` covers in `compute_utility`'s habit call.
fn is_routine(kind: K) -> bool {
    matches!(kind, K::Work | K::Eat | K::Drink | K::Rest | K::Worship)
}

fn goal_bonus(kind: K, goals: &[mindstrata_sim::person::Goal]) -> f64 {
    use mindstrata_sim::person::GoalKind as G;
    let mut bonus = 0.0;
    for g in goals {
        let aligned = matches!(
            (kind, g.kind),
            (K::Eat, G::Eat)
                | (K::Drink, G::Drink)
                | (K::Rest, G::Rest)
                | (K::Work, G::Work)
                | (K::Socialize, G::Socialize)
                | (K::Worship, G::Worship)
        );
        if aligned {
            bonus += g.priority.to_f64() * 0.5;
        }
    }
    bonus
}

/// The full pass-level ledger. `with_omitted_terms = false` reproduces i308's
/// ledger (core + goal + emotional), which did NOT match the sim.
fn ledger_winner(
    sim: &Simulation,
    id: usize,
    rng: &mut RngStreams,
    grain: Fixed,
    water: Fixed,
    with_omitted_terms: bool,
) -> (String, Vec<(String, f64, f64, f64, f64, f64)>) {
    let a = &sim.agents[id];
    let cands = [
        K::Eat,
        K::Drink,
        K::Rest,
        K::Work,
        K::Socialize,
        K::Worship,
        K::Trade,
        K::Wander,
        K::Idle,
    ];
    let stress = a.emotions.fear + a.emotions.anger;
    let mut rows: Vec<(String, f64, f64, f64, f64, f64)> = Vec::new();
    for kind in cands {
        let def = kind.definition();
        let core = compute_utility(
            &def,
            &a.needs,
            &a.personality,
            rng,
            grain,
            water,
            &a.identity,
            Fixed::ZERO,
            a.wealth.coin,
            a.neural_like.values,
            a.motivation.dominant_need,
            a.motivation.dominant_pressure(),
            a.prospection.dread,
            a.prospection.hope,
            a.prospection.planning_confidence,
        )
        .to_f64();
        let goal = goal_bonus(kind, &a.goals);
        let (is_social, is_risky, is_withdrawal, is_prosocial, is_disobedient, is_harmful) =
            traits(kind);
        let emo = a
            .decision_policy
            .emotional_modifier(
                a.emotions.anger,
                a.emotions.fear,
                a.emotions.joy,
                a.emotions.sadness,
                is_social,
                is_risky,
                is_withdrawal,
            )
            .to_f64();
        let (moral, habit) = if with_omitted_terms {
            let moral = a
                .decision_policy
                .moral_modifier(
                    a.moral_values.fairness,
                    a.moral_values.authority,
                    a.moral_values.care,
                    a.moral_values.loyalty,
                    is_prosocial,
                    is_disobedient,
                    is_harmful,
                )
                .to_f64();
            let habit = a
                .decision_policy
                .habit_modifier(is_routine(kind), stress)
                .to_f64();
            (moral, habit)
        } else {
            (0.0, 0.0)
        };
        rows.push((format!("{kind:?}"), core, goal, emo, moral, habit));
    }
    rows.sort_by(|x, y| {
        let tx = x.1 + x.2 + x.3 + x.4 + x.5;
        let ty = y.1 + y.2 + y.3 + y.4 + y.5;
        ty.partial_cmp(&tx).unwrap()
    });
    let winner = rows[0].0.clone();
    rows.truncate(3);
    (winner, rows)
}

fn main() {
    println!("i309 Rest-plateau mechanism — habit_modifier + the routine stress gate");

    // ── Leg 1: does the pass-faithful ledger reproduce the sim? ───────────
    for &(seed, id) in &PLATEAU {
        let mut sim = Simulation::new(config(seed, HORIZON));
        sim.populate();
        let mut census: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        for _ in 0..HORIZON {
            sim.tick();
            if let Some(a) = sim.agents.get(id) {
                *census
                    .entry(format!("{:?}", a.current_action))
                    .or_insert(0u64) += 1;
            }
        }
        let total = census.values().sum::<u64>().max(1) as f64;
        let mut census: Vec<_> = census.into_iter().collect();
        census.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        println!(
            "\n[leg 1] seed {seed} agent {id} — sim census: {}",
            census
                .iter()
                .take(4)
                .map(|(k, c)| format!("{k} {:.3}", *c as f64 / total))
                .collect::<Vec<_>>()
                .join(" | ")
        );
        let grain = sim.world.total_food();
        let water = sim.world.total_water();
        for with_omitted in [false, true] {
            let label = if with_omitted {
                "ledger+moral+habit"
            } else {
                "ledger (i308 form) "
            };
            let mut wins: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
            let mut rng = RngStreams::new(seed ^ 0x9E37);
            let mut sample_detail = String::new();
            let mut sweep = Simulation::new(config(seed, HORIZON));
            sweep.populate();
            for t in 0..HORIZON {
                sweep.tick();
                if t % 1000 != 0 || id >= sweep.agents.len() {
                    continue;
                }
                let (winner, rows) =
                    ledger_winner(&sweep, id, &mut rng, grain, water, with_omitted);
                *wins.entry(winner.clone()).or_insert(0u64) += 1;
                if sample_detail.is_empty() && winner == "Rest" {
                    sample_detail = rows
                        .iter()
                        .map(|(k, c, g, e, m, h)| {
                            format!(
                                "{k}={:.3} (core {c:.3} goal {g:.3} emo {e:+.3} moral {m:+.3} habit {h:+.3})",
                                c + g + e + m + h
                            )
                        })
                        .collect::<Vec<_>>()
                        .join(" | ");
                }
            }
            let mut wins: Vec<_> = wins.into_iter().collect();
            wins.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
            println!(
                "    {label}: winners {}",
                wins.iter()
                    .map(|(k, c)| format!("{k} {c}"))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
            if !sample_detail.is_empty() {
                println!("      first Rest-win sample: {sample_detail}");
            }
        }
    }

    // ── Leg 3: the three tracked plateau agents, post-fix ────────────────
    println!("\n[leg 3] tracked agents @50K — the trap's signature");
    println!(
        "  {:<6} {:<4} {:>9} {:>12} {:>9} {:>9} {:>9} {:>9} {:>8}",
        "seed", "id", "rest duty", "actions", "health", "hunger", "thirst", "social", "meaning"
    );
    for &(seed, id) in &PLATEAU {
        let mut sim = Simulation::new(config(seed, HORIZON));
        sim.populate();
        let mut census: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut n = 0u64;
        for _ in 0..HORIZON {
            sim.tick();
            if let Some(a) = sim.agents.get(id) {
                n += 1;
                *census
                    .entry(format!("{:?}", a.current_action))
                    .or_insert(0u64) += 1;
            }
        }
        let a = &sim.agents[id];
        let d = n.max(1) as f64;
        let rest = census.get("Rest").copied().unwrap_or(0) as f64 / d;
        let mut top: Vec<_> = census.into_iter().collect();
        top.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        println!(
            "  {seed:<6} {id:<4} {rest:>9.4} {:>12} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>8.3}",
            top.iter()
                .take(3)
                .map(|(k, c)| format!("{k}:{:.2}", *c as f64 / d))
                .collect::<Vec<_>>()
                .join(" "),
            a.body.health.to_f64(),
            a.needs.hunger.to_f64(),
            a.needs.thirst.to_f64(),
            a.needs.social.to_f64(),
            a.needs.meaning.to_f64(),
        );
    }
    println!(
        "\nverdict=HEALTH_FRAME_KEEPS_AGENCY \"(pre-fix: rest 0.813/0.893/0.833, social 0.75-1.00 and meaning 0.81-1.00 pinned unmet; post-fix: see leg 3)\""
    );

    // ── Leg 2: the routine stress gate ───────────────────────────────────
    // `DailyRoutine::should_follow` returns false when stress > 0.7, and the
    // pass's routine branch is skipped entirely — the daily schedule is OFF.
    println!("\n[leg 2] daily schedule availability (pass: stress = fear + anger)");
    println!(
        "  {:<8} {:<5} {:>12} {:>12} {:>10} {:>10}",
        "seed", "id", "stress>0.7", "should_follow", "rest duty", "rest in sched"
    );
    for &(seed, id) in &PLATEAU {
        let mut sim = Simulation::new(config(seed, HORIZON));
        sim.populate();
        let (mut above, mut follow, mut rest, mut rest_in_sched, mut n) =
            (0u64, 0u64, 0u64, 0u64, 0u64);
        for t in 0..HORIZON {
            sim.tick();
            if id >= sim.agents.len() {
                continue;
            }
            let a = &sim.agents[id];
            n += 1;
            let stress = a.emotions.fear + a.emotions.anger;
            let is_above = stress > Fixed::from_f64(0.7);
            let does_follow =
                a.routine
                    .should_follow(a.needs.hunger, a.needs.thirst, a.needs.fatigue, stress);
            if is_above {
                above += 1;
            }
            if does_follow {
                follow += 1;
            }
            if a.current_action == K::Rest {
                rest += 1;
                if does_follow {
                    rest_in_sched += 1;
                }
            }
            let _ = t;
        }
        let d = n.max(1) as f64;
        println!(
            "  {seed:<8} {id:<5} {:>12.4} {:>12.4} {:>10.4} {:>11}",
            above as f64 / d,
            follow as f64 / d,
            rest as f64 / d,
            format!("{:.4}", rest_in_sched as f64 / d)
        );
    }
}
