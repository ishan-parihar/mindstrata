//! i307 — physiological-saturation anatomy probe (stage 1: measure before
//! touching).
//!
//! i306's leg 5 surfaced a residual: 21 long ceiling excursions, ALL of them
//! agents pinned at thirst 1.00 **and** fatigue 1.00 for 12–15K consecutive
//! ticks, whose physiological reflex (i255) selects `Drink`/`Rest` and yet
//! never clears. That is the same failure class as the i306 meaning channel one
//! layer down — a relief route that is selected but does not land — so this
//! probe measures which of the three candidate mechanisms it is, instead of
//! guessing:
//!
//!   A. SUPPLY     — the well has no water (village-wide exhaustion).
//!   B. ACCESS     — a well with water exists but `can_access_resource` is
//!                   false for the agent (rights/permit gate).
//!   C. EXECUTION  — Drink/Rest is selected and runs, but the per-tick effect
//!                   does not land (the "selected but not applied" class that
//!                   i306 proved for Worship).
//!
//! Run: cargo run --release -p mindstrata-benches --example i307_physio_saturation

use mindstrata_core::id::AgentId;
use mindstrata_core::Fixed;
use mindstrata_sim::actions::ActionKind;
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_sim::world::{SiteKind, WATER_RESOURCE_ID};

/// The i268 seed family (the calibration-audit stability instrument).
const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HORIZONS: [u64; 3] = [2_000, 20_000, 50_000];

/// The i255 physiological reflex thresholds (must mirror `pass_action`).
const THIRST_REFLEX: f64 = 0.9;
const HUNGER_REFLEX: f64 = 0.9;
const FATIGUE_REFLEX: f64 = 0.95;

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

/// Per-agent physiology track.
#[derive(Default, Clone, Copy)]
struct Track {
    /// Current consecutive run of ticks in ANY physiological reflex zone.
    run: u64,
    longest_run: u64,
    /// Ticks with thirst above the reflex threshold (and thirst >= hunger).
    thirst_reflex_ticks: u64,
    /// Ticks with fatigue above the reflex threshold.
    fatigue_reflex_ticks: u64,
    /// Ticks where the agent's current action was Drink / Rest.
    drink_ticks: u64,
    rest_ticks: u64,
    /// Drink/Rest ticks where the need actually DROPPED (relief landed).
    drink_relief_ticks: u64,
    rest_relief_ticks: u64,
    /// Ticks where a well holding water EXISTED but was inaccessible.
    water_inaccessible_ticks: u64,
    /// Ticks where NO well anywhere held water.
    water_absent_ticks: u64,
    /// Ticks where an accessible well with water existed.
    water_available_ticks: u64,
    index: usize,
}

fn main() {
    println!(
        "i307 physiological-saturation anatomy — family of {} at {:?}",
        SEEDS.len(),
        HORIZONS
    );
    println!(
        "  reflex thresholds: thirst>{THIRST_REFLEX} Drink | hunger>{HUNGER_REFLEX} Eat | fatigue>{FATIGUE_REFLEX} Rest"
    );
    println!("  Drink: 2 ticks, thirst relief 0.7 | Rest relief via action def");

    for ticks in HORIZONS {
        println!("\n[leg 1] family physiology at {ticks} ticks/seed");
        println!(
            "  {:<8} {:>8} {:>8} {:>10} {:>10} {:>10} {:>10} {:>9} {:>9}",
            "seed",
            "thirst>p90",
            "fati>p90",
            "reflex run",
            "drink duty",
            "drink land",
            "rest duty",
            "no water",
            "unaccess"
        );
        let mut alive = 0usize;
        let (mut sum_t, mut sum_f, mut sum_run, mut sum_duty, mut sum_land, mut sum_rest) =
            (0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64, 0.0f64);
        let (mut sum_nowat, mut sum_unacc) = (0.0f64, 0.0f64);
        for &seed in &SEEDS {
            let mut sim = Simulation::new(config(seed, ticks));
            sim.populate();
            let mut tracks = vec![Track::default(); sim.agents.len()];
            for (i, t) in tracks.iter_mut().enumerate() {
                t.index = i;
            }
            let mut prev_thirst: Vec<f64> =
                sim.agents.iter().map(|a| a.needs.thirst.to_f64()).collect();
            let mut prev_fatigue: Vec<f64> = sim
                .agents
                .iter()
                .map(|a| a.needs.fatigue.to_f64())
                .collect();
            for _ in 0..ticks {
                sim.tick();
                // Births grow the vec; new agents start their own tracks.
                while tracks.len() < sim.agents.len() {
                    let i = tracks.len();
                    tracks.push(Track {
                        index: i,
                        ..Track::default()
                    });
                    prev_thirst.push(sim.agents[i].needs.thirst.to_f64());
                    prev_fatigue.push(sim.agents[i].needs.fatigue.to_f64());
                }
                // Village-level water state this tick (one probe per tick, not
                // per agent — the site list is small but this keeps it cheap).
                let mut any_water = false;
                let mut accessible_water = 0usize;
                for site in &sim.world.sites {
                    let holds = site
                        .inventory
                        .iter()
                        .any(|r| r.resource_id == WATER_RESOURCE_ID && r.quantity > Fixed::ZERO);
                    if holds && site.kind == SiteKind::Well {
                        any_water = true;
                        accessible_water += 1;
                    }
                }
                let _ = accessible_water;
                for (i, agent) in sim.agents.iter().enumerate() {
                    let thirst = agent.needs.thirst.to_f64();
                    let fatigue = agent.needs.fatigue.to_f64();
                    let drop_t = prev_thirst[i] - thirst;
                    let drop_f = prev_fatigue[i] - fatigue;
                    prev_thirst[i] = thirst;
                    prev_fatigue[i] = fatigue;
                    let in_thirst_zone =
                        thirst > THIRST_REFLEX && agent.needs.thirst >= agent.needs.hunger;
                    let in_fatigue_zone = fatigue > FATIGUE_REFLEX;
                    let t = &mut tracks[i];
                    if in_thirst_zone || in_fatigue_zone {
                        t.run += 1;
                        t.longest_run = t.longest_run.max(t.run);
                    } else {
                        t.run = 0;
                    }
                    if in_thirst_zone {
                        t.thirst_reflex_ticks += 1;
                    }
                    if in_fatigue_zone {
                        t.fatigue_reflex_ticks += 1;
                    }
                    match agent.current_action {
                        ActionKind::Drink => {
                            t.drink_ticks += 1;
                            if drop_t > 0.05 {
                                t.drink_relief_ticks += 1;
                            }
                        }
                        ActionKind::Rest => {
                            t.rest_ticks += 1;
                            if drop_f > 0.001 {
                                t.rest_relief_ticks += 1;
                            }
                        }
                        _ => {}
                    }
                    // Access anatomy: is a water-carrying well reachable?
                    let well = sim.world.accessible_well_with_water(
                        AgentId::new(i as u64),
                        sim.institutions.as_slice(),
                    );
                    if well.is_some() {
                        t.water_available_ticks += 1;
                    } else if any_water {
                        t.water_inaccessible_ticks += 1;
                    } else {
                        t.water_absent_ticks += 1;
                    }
                }
            }
            if sim.agents.is_empty() {
                continue;
            }
            alive += 1;
            let n_agents = sim.agents.len();
            let mut thirst_p90 = 0.0f64;
            let mut fatigue_p90 = 0.0f64;
            for i in 0..n_agents {
                thirst_p90 += sim.agents[i].needs.thirst.to_f64();
                fatigue_p90 += sim.agents[i].needs.fatigue.to_f64();
            }
            let d = (ticks * n_agents.max(1) as u64).max(1) as f64;
            let long = tracks
                .iter()
                .take(n_agents)
                .map(|t| t.longest_run)
                .max()
                .unwrap_or(0);
            println!(
                "  {seed:<8} {:>8.3} {:>8.3} {long:>10} {:>10.5} {:>10.5} {:>10.5} {:>9.4} {:>9.4}",
                thirst_p90 / n_agents.max(1) as f64,
                fatigue_p90 / n_agents.max(1) as f64,
                tracks
                    .iter()
                    .take(n_agents)
                    .map(|t| t.drink_ticks)
                    .sum::<u64>() as f64
                    / d,
                tracks
                    .iter()
                    .take(n_agents)
                    .map(|t| t.drink_relief_ticks)
                    .sum::<u64>() as f64
                    / d,
                tracks
                    .iter()
                    .take(n_agents)
                    .map(|t| t.rest_ticks)
                    .sum::<u64>() as f64
                    / d,
                tracks
                    .iter()
                    .take(n_agents)
                    .map(|t| t.water_absent_ticks)
                    .sum::<u64>() as f64
                    / d,
                tracks
                    .iter()
                    .take(n_agents)
                    .map(|t| t.water_inaccessible_ticks)
                    .sum::<u64>() as f64
                    / d,
            );
            sum_t += thirst_p90 / n_agents.max(1) as f64;
            sum_f += fatigue_p90 / n_agents.max(1) as f64;
            sum_run += long as f64;
            sum_duty += tracks
                .iter()
                .take(n_agents)
                .map(|t| t.drink_ticks)
                .sum::<u64>() as f64
                / d;
            sum_land += tracks
                .iter()
                .take(n_agents)
                .map(|t| t.drink_relief_ticks)
                .sum::<u64>() as f64
                / d;
            sum_rest += tracks
                .iter()
                .take(n_agents)
                .map(|t| t.rest_ticks)
                .sum::<u64>() as f64
                / d;
            sum_nowat += tracks
                .iter()
                .take(n_agents)
                .map(|t| t.water_absent_ticks)
                .sum::<u64>() as f64
                / d;
            sum_unacc += tracks
                .iter()
                .take(n_agents)
                .map(|t| t.water_inaccessible_ticks)
                .sum::<u64>() as f64
                / d;
        }
        let n = alive.max(1) as f64;
        println!(
            "  {:<8} {:>8.3} {:>8.3} {:>10.0} {:>10.5} {:>10.5} {:>10.5} {:>9.4} {:>9.4}   (alive {alive}/{})",
            "family",
            sum_t / n,
            sum_f / n,
            sum_run / n,
            sum_duty / n,
            sum_land / n,
            sum_rest / n,
            sum_nowat / n,
            sum_unacc / n,
            SEEDS.len()
        );
    }

    // ── Leg 2: the anatomy of one pinned agent (i306 seed 99 agent 2) ──────
    for (seed, id) in [(99u64, 2usize), (42, 2), (123, 6)] {
        let ticks = 50_000u64;
        println!("\n[leg 2] pinned-agent anatomy — seed {seed} agent {id}, {ticks} ticks");
        let mut sim = Simulation::new(config(seed, ticks));
        sim.populate();
        let mut prev_thirst = sim.agents[id].needs.thirst.to_f64();
        let mut prev_fatigue = sim.agents[id].needs.fatigue.to_f64();
        let mut drink = 0u64;
        let mut drink_land = 0u64;
        let mut rest = 0u64;
        let mut rest_land = 0u64;
        let mut no_water = 0u64;
        let mut inaccess = 0u64;
        let mut available = 0u64;
        let mut thirst_run = 0u64;
        let mut longest_thirst = 0u64;
        let mut total_water = 0.0f64;
        let mut min_water = f64::MAX;
        let mut census = std::collections::HashMap::new();
        // Fatigue distribution over the run: distinguishes "resting because
        // genuinely exhausted" from "resting on an already-zero need" (a
        // stuck-state in nicer clothes).
        let mut fatigue_sum = 0.0f64;
        let mut fatigue_near_zero = 0u64;
        let mut thirst_sum = 0.0f64;
        let mut samples = 0u64;
        for _ in 0..ticks {
            sim.tick();
            if id >= sim.agents.len() {
                break;
            }
            let a = &sim.agents[id];
            let thirst = a.needs.thirst.to_f64();
            let fatigue = a.needs.fatigue.to_f64();
            let dt = prev_thirst - thirst;
            let df = prev_fatigue - fatigue;
            prev_thirst = thirst;
            prev_fatigue = fatigue;
            if thirst > THIRST_REFLEX {
                thirst_run += 1;
                longest_thirst = longest_thirst.max(thirst_run);
            } else {
                thirst_run = 0;
            }
            match a.current_action {
                ActionKind::Drink => {
                    drink += 1;
                    if dt > 0.05 {
                        drink_land += 1;
                    }
                }
                ActionKind::Rest => {
                    rest += 1;
                    if df > 0.001 {
                        rest_land += 1;
                    }
                }
                _ => {}
            }
            *census
                .entry(format!("{:?}", a.current_action))
                .or_insert(0u64) += 1;
            fatigue_sum += fatigue;
            thirst_sum += thirst;
            samples += 1;
            if fatigue < 0.05 {
                fatigue_near_zero += 1;
            }
            let well = sim
                .world
                .accessible_well_with_water(AgentId::new(id as u64), sim.institutions.as_slice());
            let any_water = sim.world.sites.iter().any(|s| {
                s.kind == SiteKind::Well
                    && s.inventory
                        .iter()
                        .any(|r| r.resource_id == WATER_RESOURCE_ID && r.quantity > Fixed::ZERO)
            });
            if well.is_some() {
                available += 1;
            } else if any_water {
                inaccess += 1;
            } else {
                no_water += 1;
            }
            let stock: f64 = sim
                .world
                .sites
                .iter()
                .filter(|s| s.kind == SiteKind::Well)
                .flat_map(|s| s.inventory.iter())
                .filter(|r| r.resource_id == WATER_RESOURCE_ID)
                .map(|r| r.quantity.to_f64())
                .sum();
            total_water += stock;
            min_water = min_water.min(stock);
        }
        let mut census: Vec<_> = census.into_iter().collect();
        census.sort_by_key(|(_, c)| std::cmp::Reverse(*c));
        println!(
            "  thirst: longest run above reflex {longest_thirst} ticks | final {prev_thirst:.4}"
        );
        println!(
            "  drink ticks {drink} | relief landed {drink_land} | rest ticks {rest} | fatigue relief landed {rest_land}"
        );
        println!(
            "  water: accessible {available} | well-with-water-but-BLOCKED {inaccess} | NO well holds water {no_water}"
        );
        let s = samples.max(1) as f64;
        println!(
            "  needs over run: thirst mean {:.3} | fatigue mean {:.3} | fatigue<0.05 share {:.3}",
            thirst_sum / s,
            fatigue_sum / s,
            fatigue_near_zero as f64 / s
        );
        println!(
            "  well stock: mean {:.2} | min {:.2} (raw units)",
            total_water / ticks as f64,
            min_water
        );
        let top: Vec<String> = census
            .iter()
            .take(5)
            .map(|(k, v)| format!("{k} {:.3}", *v as f64 / ticks as f64))
            .collect();
        println!("  action census: {}", top.join(" | "));
    }

    // ── Leg 4: the 50K emergence leg (pestilence seed 5) ───────────────────
    //
    // `long_horizon_tests::long_horizon_50k_is_deterministic_and_emerges`
    // asserts the calm/crisis belief-charge band on pestilence seed 5 @50K.
    // The i307 gate changes action selection in exactly the stressed, habituated
    // population that scenario builds, so this leg measures the charge
    // equilibrium and the emergence counters the test guards.
    {
        use mindstrata_core::Fixed as F;
        use mindstrata_sim::scenario::Scenario;
        println!("\n[leg 4] pestilence seed 5 @50K (the 50K emergence leg)");
        // Mirror `run_scenario` exactly: seed 5 and a 50K horizon override the
        // scenario's own defaults, or this measures a different world.
        let mut sc = Scenario::pestilence();
        sc.seed = 5;
        sc.ticks = 50_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        let mut sums = [0.0f64; 6]; // eat, drink, rest, work, social, worship
        let mut agent_ticks = 0u64;
        let mut reflex_zone = 0u64;
        for _ in 0..50_000u64 {
            sim.tick();
            for a in &sim.agents {
                agent_ticks += 1;
                match a.current_action {
                    ActionKind::Eat => sums[0] += 1.0,
                    ActionKind::Drink => sums[1] += 1.0,
                    ActionKind::Rest => sums[2] += 1.0,
                    ActionKind::Work => sums[3] += 1.0,
                    ActionKind::Socialize => sums[4] += 1.0,
                    ActionKind::Worship => sums[5] += 1.0,
                    _ => {}
                }
                let t = a.needs.thirst.to_f64();
                let h = a.needs.hunger.to_f64();
                let f = a.needs.fatigue.to_f64();
                if t > THIRST_REFLEX || h > HUNGER_REFLEX || f > FATIGUE_REFLEX {
                    reflex_zone += 1;
                }
            }
        }
        let mut sum = 0.0f64;
        let mut n = 0usize;
        for a in &sim.agents {
            for b in a.beliefs.iter().filter(|b| b.proposition_id <= 1) {
                sum += b.emotional_charge.to_f64();
                n += 1;
            }
        }
        let m = sim.metrics_snapshot();
        println!(
            "  avg belief charge (prop<=1) {:.4} | agents {} | events {} | memes {} | stress {:.3} | health {:.3}",
            sum / n.max(1) as f64,
            m.agent_count,
            m.event_count,
            m.active_meme_count,
            m.avg_stress,
            m.avg_health
        );
        println!(
            "  factions v2 history {} | panics {} | reflex-zone duty {:.4}",
            sim.faction_v2_registry.factions.len(),
            sim.moral_panic_registry.panics.len(),
            reflex_zone as f64 / agent_ticks.max(1) as f64,
        );
        // Charge DISPERSION: a re-anchored band is only honest if the channel
        // is regulated — live spread, nothing pinned at saturation.
        {
            let mut charges: Vec<f64> = sim
                .agents
                .iter()
                .flat_map(|a| {
                    a.beliefs
                        .iter()
                        .filter(|b| b.proposition_id <= 1)
                        .map(|b| b.emotional_charge.to_f64())
                })
                .collect();
            charges.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let q = |p: f64| -> f64 {
                if charges.is_empty() {
                    f64::NAN
                } else {
                    charges[((charges.len() as f64 - 1.0) * p).round() as usize]
                }
            };
            let saturated = charges.iter().filter(|c| **c >= 0.99).count();
            println!(
                "  charge spread: n {} | p10 {:.3} | p50 {:.3} | p90 {:.3} | max {:.3} | saturated(>=0.99) {}",
                charges.len(),
                q(0.10),
                q(0.50),
                q(0.90),
                charges.last().copied().unwrap_or(f64::NAN),
                saturated
            );
        }
        println!(
            "  duty: Eat {:.4} | Drink {:.4} | Rest {:.4} | Work {:.4} | Socialize {:.4} | Worship {:.4}",
            sums[0] / agent_ticks.max(1) as f64,
            sums[1] / agent_ticks.max(1) as f64,
            sums[2] / agent_ticks.max(1) as f64,
            sums[3] / agent_ticks.max(1) as f64,
            sums[4] / agent_ticks.max(1) as f64,
            sums[5] / agent_ticks.max(1) as f64
        );
        let _ = F::ONE;
    }

    // ── Leg 3: the CRISIS path — the collapse golden's own scenario ────────
    //
    // The habit-substitution gate needs stress > 0.5 AND automaticity > 0.5,
    // so whether the i307 fix touches a calibration window is an empirical
    // question, not a structural one: a crisis is exactly where both climb.
    // This leg measures the collapse scenario (seed 42, 4320 ticks) so the
    // golden re-anchor (if any) is reported with numbers rather than assumed.
    {
        use mindstrata_sim::scenario::Scenario;
        let sc = Scenario::collapse();
        let ticks = sc.ticks;
        println!(
            "\n[leg 3] collapse scenario (seed {}, {ticks} ticks)",
            sc.seed
        );
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        let mut drink = 0u64;
        let mut eat = 0u64;
        let mut rest = 0u64;
        let mut reflex_zone = 0u64;
        let mut agent_ticks = 0u64;
        let mut stress_high = 0u64;
        let mut autos_high = 0u64;
        let mut both_high = 0u64;
        for _ in 0..ticks {
            sim.tick();
            for a in &sim.agents {
                agent_ticks += 1;
                match a.current_action {
                    ActionKind::Drink => drink += 1,
                    ActionKind::Eat => eat += 1,
                    ActionKind::Rest => rest += 1,
                    _ => {}
                }
                let t = a.needs.thirst.to_f64();
                let h = a.needs.hunger.to_f64();
                let f = a.needs.fatigue.to_f64();
                if t > THIRST_REFLEX || h > HUNGER_REFLEX || f > FATIGUE_REFLEX {
                    reflex_zone += 1;
                }
                // The i307 gate's own inputs, read at the horizon state.
                let stress = a.emotions.fear + a.emotions.anger;
                if stress > Fixed::from_f64(0.5) {
                    stress_high += 1;
                }
                if a.psych_skills.automaticity > Fixed::from_f64(0.5) {
                    autos_high += 1;
                }
                if stress > Fixed::from_f64(0.5)
                    && a.psych_skills.automaticity > Fixed::from_f64(0.5)
                {
                    both_high += 1;
                }
            }
        }
        let n = agent_ticks.max(1) as f64;
        println!(
            "  agent_ticks {agent_ticks} | final population {}",
            sim.agents.len()
        );
        println!(
            "  duty: Eat {:.5} | Drink {:.5} | Rest {:.5} | in-reflex-zone {:.5}",
            eat as f64 / n,
            drink as f64 / n,
            rest as f64 / n,
            reflex_zone as f64 / n
        );
        println!(
            "  habit gate inputs: stress>0.5 {:.5} | automaticity>0.5 {:.5} | BOTH {:.5}",
            stress_high as f64 / n,
            autos_high as f64 / n,
            both_high as f64 / n
        );
        let (mut t, mut h, mut f, mut hp) = (0.0f64, 0.0f64, 0.0f64, 0.0f64);
        for a in &sim.agents {
            t += a.needs.thirst.to_f64();
            h += a.needs.hunger.to_f64();
            f += a.needs.fatigue.to_f64();
            hp += a.body.health.to_f64();
        }
        let na = sim.agents.len().max(1) as f64;
        println!(
            "  final means: thirst {:.3} | hunger {:.3} | fatigue {:.3} | health {:.3}",
            t / na,
            h / na,
            f / na,
            hp / na
        );
    }
}
