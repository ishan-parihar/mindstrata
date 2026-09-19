//! i308 — Rest-dominance anatomy probe (stage 1: measure before touching).
//!
//! i307 gated the stress-habit fallback off the survival reflex. That freed a
//! population whose habits had been pinning them — and the freed agents do not
//! return to Work/Trade, they settle into `Rest`: seed 123 agent 6 @50K sits in
//! `Rest` for **81.3%** of its life with a fatigue mean of 0.040 (88% of ticks
//! below 0.05). Sleeping with nothing to sleep off is a *stuck producer* of the
//! same family as i306/i307, so this probe measures which input is driving the
//! Rest utility before anything is changed.
//!
//! Candidate mechanisms:
//!   A. ENERGY      — energy is depleted and only Rest recovers it.
//!   B. SLEEP       — sleep_pressure/sleep_debt stay high (Rest doesn't repay).
//!   C. PRESSURE    — the motivation argmax reports Sleep with real pressure.
//!   D. NO-ALTERNATIVE — every other action's utility is negative/low.
//!
//! Run: cargo run --release -p mindstrata-benches --example i308_rest_dominance

use mindstrata_sim::actions::ActionKind;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HORIZON: u64 = 50_000;

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
struct Track {
    rest_ticks: u64,
    /// Rest ticks taken while fatigue is LOW (nothing to sleep off).
    rest_low_fatigue: u64,
    /// Rest ticks taken while energy is HIGH and sleep pressure LOW.
    rest_low_drive: u64,
    fatigue_sum: f64,
    energy_sum: f64,
    sleep_pressure_sum: f64,
    sleep_debt_sum: f64,
    alertness_sum: f64,
    /// Ticks where the motivation argmax was the Sleep category.
    sleep_dominant_ticks: u64,
    samples: u64,
}

fn main() {
    println!(
        "i308 Rest-dominance anatomy — family of {} at {HORIZON} ticks",
        SEEDS.len()
    );

    // ── Leg 1: family-level Rest duty and the state behind it ─────────────
    println!("\n[leg 1] family, {HORIZON} ticks/seed");
    println!(
        "  {:<8} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "seed", "rest duty", "fatigue", "energy", "sleepP", "sleepD", "alert", "sleepdom"
    );
    let mut worst: Vec<(u64, usize, f64)> = Vec::new();
    let (mut sum_rest, mut sum_f, mut sum_e, mut sum_p, mut sum_d) = (0.0f64, 0.0, 0.0, 0.0, 0.0);
    let mut alive = 0usize;
    for &seed in &SEEDS {
        let mut sim = Simulation::new(config(seed, HORIZON));
        sim.populate();
        let mut tracks = vec![Track::default(); sim.agents.len()];
        for _ in 0..HORIZON {
            sim.tick();
            while tracks.len() < sim.agents.len() {
                tracks.push(Track::default());
            }
            for (i, a) in sim.agents.iter().enumerate() {
                let t = &mut tracks[i];
                let fatigue = a.needs.fatigue.to_f64();
                let energy = a.body.energy.to_f64();
                let pressure = a.embodied.circadian.sleep_pressure.to_f64();
                t.samples += 1;
                t.fatigue_sum += fatigue;
                t.energy_sum += energy;
                t.sleep_pressure_sum += pressure;
                t.sleep_debt_sum += a.embodied.circadian.sleep_debt.to_f64();
                t.alertness_sum += a.embodied.circadian.alertness.to_f64();
                if matches!(
                    a.motivation.dominant_need,
                    mindstrata_sim::psychology::motivation::MotiveCategory::Sleep
                ) {
                    t.sleep_dominant_ticks += 1;
                }
                if a.current_action == ActionKind::Rest {
                    t.rest_ticks += 1;
                    if fatigue < 0.2 {
                        t.rest_low_fatigue += 1;
                    }
                    if fatigue < 0.2 && energy > 0.8 && pressure < 0.2 {
                        t.rest_low_drive += 1;
                    }
                }
            }
        }
        if sim.agents.is_empty() {
            continue;
        }
        alive += 1;
        let n = tracks.iter().map(|t| t.samples).sum::<u64>().max(1) as f64;
        let rest = tracks.iter().map(|t| t.rest_ticks).sum::<u64>() as f64 / n;
        let fat = tracks.iter().map(|t| t.fatigue_sum).sum::<f64>() / n;
        let eng = tracks.iter().map(|t| t.energy_sum).sum::<f64>() / n;
        let pre = tracks.iter().map(|t| t.sleep_pressure_sum).sum::<f64>() / n;
        let deb = tracks.iter().map(|t| t.sleep_debt_sum).sum::<f64>() / n;
        let ale = tracks.iter().map(|t| t.alertness_sum).sum::<f64>() / n;
        let sdo = tracks.iter().map(|t| t.sleep_dominant_ticks).sum::<u64>() as f64 / n;
        println!(
            "  {seed:<8} {rest:>9.4} {fat:>9.4} {eng:>9.4} {pre:>9.4} {deb:>9.4} {ale:>9.4} {sdo:>9.4}"
        );
        sum_rest += rest;
        sum_f += fat;
        sum_e += eng;
        sum_p += pre;
        sum_d += deb;
        for (i, t) in tracks.iter().enumerate() {
            let duty = t.rest_ticks as f64 / t.samples.max(1) as f64;
            if duty > 0.5 {
                worst.push((seed, i, duty));
            }
        }
    }
    let n = alive.max(1) as f64;
    println!(
        "  {:<8} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>9}   (alive {alive}/{})",
        "family",
        sum_rest / n,
        sum_f / n,
        sum_e / n,
        sum_p / n,
        sum_d / n,
        f64::NAN,
        "",
        SEEDS.len()
    );

    // ── Leg 3: where does the fear come from? ─────────────────────────────
    //
    // The high-Rest agents are not activated into Rest by a need — they are
    // ACTIVATED BY FEAR (0.92–0.94) with joy ~0.03 and valence −0.93. Before
    // touching selection, measure whether that fear is a legitimate response or
    // a ratchet: family fear/sadness at three horizons, plus one agent's fear
    // trajectory in 5K windows.
    println!("\n[leg 3] affect by horizon (calm family)");
    println!(
        "  {:<8} {:>9} {:>9} {:>9} {:>9} {:>9} {:>9}",
        "horizon", "fear", "sadness", "joy", "valence", "trauma", "depress"
    );
    for ticks in [2_000u64, 20_000, 50_000] {
        let (mut f, mut s, mut j, mut v, mut tr, mut de) = (0.0f64, 0.0, 0.0, 0.0, 0.0, 0.0);
        let mut n = 0u64;
        for &seed in &SEEDS {
            let mut sim = Simulation::new(config(seed, ticks));
            sim.populate();
            for _ in 0..ticks {
                sim.tick();
                for a in sim.agents.iter() {
                    f += a.emotions.fear.to_f64();
                    s += a.emotions.sadness.to_f64();
                    j += a.emotions.joy.to_f64();
                    v += a.affect.valence.to_f64();
                    tr += a.derived.trauma_risk.to_f64();
                    de += a.derived.depression_risk.to_f64();
                    n += 1;
                }
            }
        }
        let n = n.max(1) as f64;
        println!(
            "  {ticks:<8} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3} {:>9.3}",
            f / n,
            s / n,
            j / n,
            v / n,
            tr / n,
            de / n
        );
    }

    // ── Leg 4: the utility ledger + ablation for one Rest-pinned agent ────
    //
    // Same inputs the action pass feeds `select_action` (minus the habit layer
    // i307 gated, the somatic marker and the polarity bias, which are all zero
    // for an unstressed agent), so a negative result here is meaningful:
    //   core  = `actions::compute_utility`
    //   goal  = +priority × 0.5 for each goal-aligned action (§24.5)
    //   emo   = `DecisionPolicy::emotional_modifier` (clamped ±0.3)
    // Then ABLATE one input at a time and see which single change flips the
    // winner — the causal attribution, not an argument.
    {
        use mindstrata_core::rng::RngStreams;
        use mindstrata_sim::actions::{compute_utility, ActionKind as K};
        let (seed, id) = (123u64, 6usize);
        let mut sim = Simulation::new(config(seed, HORIZON));
        sim.populate();
        for _ in 0..HORIZON {
            sim.tick();
        }
        let a = &sim.agents[id];
        let grain = sim.world.total_food();
        let water = sim.world.total_water();
        // (is_social, is_risky, is_withdrawal) from `actions::action_traits`.
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
        let ledger = |needs: &mindstrata_sim::person::NeedState,
                      fear: f64,
                      sadness: f64,
                      joy: f64,
                      label: &str| {
            let mut rng = RngStreams::new(seed ^ 0x9E37);
            let mut rows: Vec<(String, f64, f64, f64, f64)> = Vec::new();
            for kind in cands {
                let def = kind.definition();
                let core = compute_utility(
                    &def,
                    needs,
                    &a.personality,
                    &mut rng,
                    grain,
                    water,
                    &a.identity,
                    mindstrata_core::Fixed::ZERO,
                    a.wealth.coin,
                    a.neural_like.values,
                    a.motivation.dominant_need,
                    a.motivation.dominant_pressure(),
                    a.prospection.dread,
                    a.prospection.hope,
                    a.prospection.planning_confidence,
                )
                .to_f64();
                let mut goal = 0.0f64;
                for g in a.goals.iter() {
                    let aligned = matches!(
                        (kind, g.kind),
                        (K::Eat, mindstrata_sim::person::GoalKind::Eat)
                            | (K::Drink, mindstrata_sim::person::GoalKind::Drink)
                            | (K::Rest, mindstrata_sim::person::GoalKind::Rest)
                            | (K::Work, mindstrata_sim::person::GoalKind::Work)
                            | (K::Socialize, mindstrata_sim::person::GoalKind::Socialize)
                            | (K::Worship, mindstrata_sim::person::GoalKind::Worship)
                    );
                    if aligned {
                        goal += g.priority.to_f64() * 0.5;
                    }
                }
                let (is_social, is_risky, is_withdrawal) = match kind {
                    K::Rest | K::Idle => (false, false, true),
                    K::Socialize | K::Trade | K::Worship => (true, false, false),
                    K::Wander => (false, true, false),
                    _ => (false, false, false),
                };
                let emo = a
                    .decision_policy
                    .emotional_modifier(
                        mindstrata_core::Fixed::from_f64(0.0),
                        mindstrata_core::Fixed::from_f64(fear),
                        mindstrata_core::Fixed::from_f64(joy),
                        mindstrata_core::Fixed::from_f64(sadness),
                        is_social,
                        is_risky,
                        is_withdrawal,
                    )
                    .to_f64();
                rows.push((format!("{kind:?}"), core, goal, emo, core + goal + emo));
            }
            rows.sort_by(|x, y| y.4.partial_cmp(&x.4).unwrap());
            println!(
                "  {label:<22} winner {:<10} | top3 {} ",
                rows[0].0,
                rows.iter()
                    .take(3)
                    .map(|r| format!(
                        "{}={:.4} (core {:.4} goal {:.4} emo {:+.4})",
                        r.0, r.4, r.1, r.2, r.3
                    ))
                    .collect::<Vec<_>>()
                    .join(" | ")
            );
        };
        let fear = a.emotions.fear.to_f64();
        let sadness = a.emotions.sadness.to_f64();
        let joy = a.emotions.joy.to_f64();
        let base = a.needs.clone();
        println!("\n[leg 4] utility ledger + ablation — seed {seed} agent {id} @{HORIZON}");
        println!(
            "  state: fear {fear:.3} sadness {sadness:.3} joy {joy:.3} | fatigue {:.3} social {:.3} meaning {:.3} hunger {:.3} thirst {:.3} | goals {}",
            base.fatigue.to_f64(),
            base.social.to_f64(),
            base.meaning.to_f64(),
            base.hunger.to_f64(),
            base.thirst.to_f64(),
            a.goals
                .iter()
                .map(|g| format!("{:?}:{:.2}", g.kind, g.priority.to_f64()))
                .collect::<Vec<_>>()
                .join(",")
        );
        // The horizon instant is ONE tick out of 50K — sweep the whole run so
        // the ledger explains the 81% Rest duty rather than a single moment.
        let mut wins: std::collections::HashMap<String, u64> = std::collections::HashMap::new();
        let mut shown = 0usize;
        let mut rng = RngStreams::new(seed ^ 0x9E37);
        // Own instance: keeps the `a`/`base` borrows above intact.
        let mut sweep = Simulation::new(config(seed, HORIZON));
        sweep.populate();
        for t in 0..HORIZON {
            sweep.tick();
            if t % 1000 != 0 || id >= sweep.agents.len() {
                continue;
            }
            let ag = &sweep.agents[id];
            let n = ag.needs.clone();
            let mut best = (String::new(), f64::MIN);
            let mut detail: Vec<(String, f64, f64, f64)> = Vec::new();
            for kind in cands {
                let def = kind.definition();
                let core = compute_utility(
                    &def,
                    &n,
                    &ag.personality,
                    &mut rng,
                    grain,
                    water,
                    &ag.identity,
                    mindstrata_core::Fixed::ZERO,
                    ag.wealth.coin,
                    ag.neural_like.values,
                    ag.motivation.dominant_need,
                    ag.motivation.dominant_pressure(),
                    ag.prospection.dread,
                    ag.prospection.hope,
                    ag.prospection.planning_confidence,
                )
                .to_f64();
                let mut goal = 0.0f64;
                for g in ag.goals.iter() {
                    let aligned = matches!(
                        (kind, g.kind),
                        (K::Eat, mindstrata_sim::person::GoalKind::Eat)
                            | (K::Drink, mindstrata_sim::person::GoalKind::Drink)
                            | (K::Rest, mindstrata_sim::person::GoalKind::Rest)
                            | (K::Work, mindstrata_sim::person::GoalKind::Work)
                            | (K::Socialize, mindstrata_sim::person::GoalKind::Socialize)
                            | (K::Worship, mindstrata_sim::person::GoalKind::Worship)
                    );
                    if aligned {
                        goal += g.priority.to_f64() * 0.5;
                    }
                }
                let total = core + goal;
                detail.push((format!("{kind:?}"), total, core, goal));
                if total > best.1 {
                    best = (format!("{kind:?}"), total);
                }
            }
            *wins.entry(best.0.clone()).or_insert(0) += 1;
            if best.0 == "Rest" && shown < 3 {
                shown += 1;
                detail.sort_by(|x, y| y.1.partial_cmp(&x.1).unwrap());
                println!(
                    "  REST-WIN sample @t{t}: fatigue {:.3} social {:.3} meaning {:.3} hunger {:.3} thirst {:.3} | fear {:.2} sad {:.2} | goals {}",
                    n.fatigue.to_f64(),
                    n.social.to_f64(),
                    n.meaning.to_f64(),
                    n.hunger.to_f64(),
                    n.thirst.to_f64(),
                    ag.emotions.fear.to_f64(),
                    ag.emotions.sadness.to_f64(),
                    ag.goals
                        .iter()
                        .map(|g| format!("{:?}:{:.2}", g.kind, g.priority.to_f64()))
                        .collect::<Vec<_>>()
                        .join(",")
                );
                println!(
                    "      top3: {}",
                    detail
                        .iter()
                        .take(3)
                        .map(|(k, t, c, g)| format!("{k}={t:.4} (core {c:.4} goal {g:.4})"))
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
        let mut wins: Vec<_> = wins.into_iter().collect();
        wins.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        println!(
            "  ledger winners over 50 samples: {}",
            wins.iter()
                .map(|(k, c)| format!("{k} {c}"))
                .collect::<Vec<_>>()
                .join(" | ")
        );

        ledger(&base, fear, sadness, joy, "as-is");
        ledger(&base, 0.0, sadness, joy, "ablate fear->0");
        ledger(&base, fear, 0.0, joy, "ablate sadness->0");
        let mut no_social = base.clone();
        no_social.social = mindstrata_core::Fixed::ZERO;
        ledger(&no_social, fear, sadness, joy, "ablate social->0");
        let mut no_meaning = base.clone();
        no_meaning.meaning = mindstrata_core::Fixed::ZERO;
        ledger(&no_meaning, fear, sadness, joy, "ablate meaning->0");
        let mut no_fatigue = base.clone();
        no_fatigue.fatigue = mindstrata_core::Fixed::ZERO;
        ledger(&no_fatigue, fear, sadness, joy, "ablate fatigue->0");
    }

    // ── Leg 5: input ablation inside the LIVE selection path ──────────────
    //
    // The ledger (leg 4) does not reproduce the sim's own winner, so the driver
    // is an input the ledger omits. Neutralize the candidates one at a time, in
    // the real sim, and watch the Rest duty — this is attribution, not
    // argument.
    {
        println!("\n[leg 5] input ablation at 20K (seed, variant) → Rest duty | census");
        for seed in [123u64, 2] {
            for variant in ["baseline", "no-habits", "no-fear/sad", "needs-max"] {
                let mut sim = Simulation::new(config(seed, 20_000));
                sim.populate();
                let mut rest = 0u64;
                let mut census = std::collections::HashMap::new();
                let mut ticks_seen = 0u64;
                for _ in 0..20_000u64 {
                    // Neutralize BEFORE the tick so the pass sees it.
                    for a in sim.agents.iter_mut() {
                        match variant {
                            "no-habits" => {
                                a.psych_skills.habits.clear();
                                a.psych_skills.automaticity = mindstrata_core::Fixed::ZERO;
                            }
                            "no-fear/sad" => {
                                a.emotions.fear = mindstrata_core::Fixed::ZERO;
                                a.emotions.sadness = mindstrata_core::Fixed::ZERO;
                            }
                            "needs-max" => {
                                a.needs.social = mindstrata_core::Fixed::from_f64(0.95);
                                a.needs.meaning = mindstrata_core::Fixed::from_f64(0.95);
                            }
                            _ => {}
                        }
                    }
                    sim.tick();
                    for a in sim.agents.iter() {
                        ticks_seen += 1;
                        if a.current_action == ActionKind::Rest {
                            rest += 1;
                        }
                        *census
                            .entry(format!("{:?}", a.current_action))
                            .or_insert(0u64) += 1;
                    }
                }
                let mut census: Vec<_> = census.into_iter().collect();
                census.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
                let d = ticks_seen.max(1) as f64;
                println!(
                    "  seed {seed:<5} {variant:<12} Rest {:.4} | {}",
                    rest as f64 / d,
                    census
                        .iter()
                        .take(4)
                        .map(|(k, v)| format!("{k} {:.2}", *v as f64 / d))
                        .collect::<Vec<_>>()
                        .join(" | ")
                );
            }
        }
    }

    // ── Leg 2: the high-Rest agents, with the discriminators ──────────────
    worst.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
    println!(
        "\n[leg 2] agents with Rest duty > 0.5: {} of the family",
        worst.len()
    );
    println!(
        "  {:<7} {:<4} {:>10} {:>10} {:>9} {:>9} {:>9} {:>9} {:>8} {:>8}",
        "seed",
        "id",
        "rest duty",
        "rest<0.2f",
        "rest-nodr",
        "fatigue",
        "energy",
        "sleepP",
        "sleepdom",
        "actions"
    );
    for &(seed, id, duty) in worst.iter().take(10) {
        let mut sim = Simulation::new(config(seed, HORIZON));
        sim.populate();
        let mut t = Track::default();
        let mut census = std::collections::HashMap::new();
        // Affect / derived-state means: `compute_utility` gives NO residual
        // value to any action once every need is met, so the winner among the
        // near-zero candidate set is decided by the DecisionPolicy emotional
        // (withdrawal → Rest) and moral modifiers. These columns say whether
        // that is what is happening.
        let (mut fear, mut anger, mut sadness, mut joy, mut valence) = (0.0f64, 0.0, 0.0, 0.0, 0.0);
        let (mut stress_l, mut load, mut depress, mut trauma) = (0.0f64, 0.0, 0.0, 0.0);
        let (mut social, mut meaning) = (0.0f64, 0.0);
        for _ in 0..HORIZON {
            sim.tick();
            if id >= sim.agents.len() {
                break;
            }
            let a = &sim.agents[id];
            let fatigue = a.needs.fatigue.to_f64();
            let energy = a.body.energy.to_f64();
            let pressure = a.embodied.circadian.sleep_pressure.to_f64();
            t.samples += 1;
            t.fatigue_sum += fatigue;
            t.energy_sum += energy;
            t.sleep_pressure_sum += pressure;
            t.sleep_debt_sum += a.embodied.circadian.sleep_debt.to_f64();
            t.alertness_sum += a.embodied.circadian.alertness.to_f64();
            if matches!(
                a.motivation.dominant_need,
                mindstrata_sim::psychology::motivation::MotiveCategory::Sleep
            ) {
                t.sleep_dominant_ticks += 1;
            }
            *census
                .entry(format!("{:?}", a.current_action))
                .or_insert(0u64) += 1;
            if a.current_action == ActionKind::Rest {
                t.rest_ticks += 1;
                if fatigue < 0.2 {
                    t.rest_low_fatigue += 1;
                }
                if fatigue < 0.2 && energy > 0.8 && pressure < 0.2 {
                    t.rest_low_drive += 1;
                }
            }
            fear += a.emotions.fear.to_f64();
            anger += a.emotions.anger.to_f64();
            sadness += a.emotions.sadness.to_f64();
            joy += a.emotions.joy.to_f64();
            valence += a.affect.valence.to_f64();
            stress_l += a.embodied.endocrine.stress.level.to_f64();
            load += a.embodied.endocrine.stress.chronic_load.to_f64();
            depress += a.derived.depression_risk.to_f64();
            trauma += a.derived.trauma_risk.to_f64();
            social += a.needs.social.to_f64();
            meaning += a.needs.meaning.to_f64();
        }
        let s = t.samples.max(1) as f64;
        let mut census: Vec<_> = census.into_iter().collect();
        census.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        let top: Vec<String> = census
            .iter()
            .take(4)
            .map(|(k, v)| format!("{k} {:.2}", *v as f64 / s))
            .collect();
        println!(
            "  {seed:<7} {id:<4} {duty:>10.4} {:>10.4} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>8.4}  {}",
            t.rest_low_fatigue as f64 / s,
            t.rest_low_drive as f64 / s,
            t.fatigue_sum / s,
            t.energy_sum / s,
            t.sleep_pressure_sum / s,
            t.sleep_dominant_ticks as f64 / s,
            top.join(" | ")
        );
        println!(
            "      affect: fear {:.3} anger {:.3} sad {:.3} joy {:.3} valence {:.3} | stress {:.3}/{:.3} depress {:.3} trauma {:.3} | social {:.3} meaning {:.3}",
            fear / s,
            anger / s,
            sadness / s,
            joy / s,
            valence / s,
            stress_l / s,
            load / s,
            depress / s,
            trauma / s,
            social / s,
            meaning / s
        );
    }

    // ── Verdict ───────────────────────────────────────────────────────────
    //
    // This iteration MEASURES; it does not change behaviour. Pre-registered
    // question: which input drives the Rest plateau i307 unmasked?
    //   A. ENERGY  RULED OUT — plateau agents sit at energy 0.94–0.99.
    //   B. SLEEP   RULED OUT — sleep pressure 0.04–0.11, debt 0.00, and the
    //              motivation argmax is Sleep in only 1.6–11% of their ticks.
    //   C. HABIT   RULED OUT — the ablation clears habits + automaticity every
    //              tick and Rest duty does not fall (0.2995 → 0.2672).
    //   D. AFFECT  RULED OUT — zeroing fear+sadness every tick RAISES Rest duty
    //              (0.2995 → 0.3341), so the emotional modifiers are not it.
    //   E. NEEDS   The plateau survives with social 0.71–0.84 and meaning
    //              0.81–0.89 UNMET, so the residual driver is the utility
    //              landscape itself. The leg-4 ledger fails to reproduce the
    //              sim's own winner (ledger: Trade 33 / Drink 16 / Rest 1 across
    //              50 samples; sim census: Rest 0.81), which localizes the
    //              missing term to a pass-level input the ledger omits.
    // Recorded as an open behavioural finding with its measurements; the
    // utility-landscape question belongs to its own iteration (see
    // docs/architecture/AP4-studio/evidence/i308_rest_plateau.md).
    println!("\nverdict=REST_PLATEAU_MEASURED_DRIVER_LOCALIZED_TO_UTILITY_LANDSCAPE");
}
