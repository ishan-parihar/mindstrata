//! i382 — is the faction trigger's LEGITIMACY channel dead, and what revives it?
//!
//! i379's gate census measured `council legitimacy < 0.50` at **0.0% open** in all
//! four sampled worlds (band 0.54–0.69). But the same absolute 0.5 appears TWICE in
//! `factions_impl.rs`: as the legacy cliff OR-arm, and inside the i240 accumulator
//! (`legitimacy_deficit = (0.5 − avg_council_legitimacy).max(0)`, which feeds the
//! pressure build term). If legitimacy never crosses 0.5, BOTH sites are inert and
//! the entire legitimacy→formation channel is dead — a bigger finding than a single
//! unreachable arm.
//!
//! Leg A samples the live sim per tick over a scenario × seed × N grid and reports:
//!   * the legitimacy band and the share of ticks below 0.5 (open rate of BOTH sites),
//!   * the share of ticks with a live grievance-excess signal,
//!   * pressure at the calibrated 10K window / max / ticks ≥ the 0.5 arm threshold,
//!   * every faction formation with attribution (which arm could have armed it),
//!   * the integral of each build term — the honest measure of "is the legitimacy
//!     term contributing anything at all".
//!
//! Leg B replays the accumulator offline on the SAME sampled series for the shipped
//! absolute deficit and for a relative one (`max(0, baseline − legit)`, EWMA baseline
//! over a 3× τ band — the i381 form), so the law choice is made against the
//! must-not-fire side of the corpus (calm worlds at the calibrated horizon), not just
//! the worlds that do fire.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i382_legitimacy_channel`

use mindstrata_sim::factions::{
    self, CRISIS_GRIEVANCE_ANCHOR, CRISIS_PRESSURE_BLEED_RATE, CRISIS_PRESSURE_BUILD_RATE,
    FACTION_FORMATION_PRESSURE_THRESHOLD, FACTION_REARM_COOLDOWN_TICKS,
};
use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// The shipped absolute reference, shared by the cliff arm and the deficit term.
const ABSOLUTE_REF: f64 = 0.5;

struct WorldSeries {
    legitimacy: Vec<f64>,
    grievance_excess: Vec<f64>,
    pressure: Vec<f64>,
    /// The sim's own mandate baseline, as observed after each tick.
    baseline: Vec<f64>,
    /// Tick indices at which the faction count increased (a formation).
    formations: Vec<usize>,
}

fn run_world(
    scenario: Option<Scenario>,
    seed: u64,
    ticks: u64,
    w: u32,
    h: u32,
    n: u32,
) -> WorldSeries {
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

    let mut s = WorldSeries {
        legitimacy: Vec::with_capacity(ticks as usize),
        grievance_excess: Vec::with_capacity(ticks as usize),
        pressure: Vec::with_capacity(ticks as usize),
        baseline: Vec::with_capacity(ticks as usize),
        formations: Vec::new(),
    };
    let mut last_factions = 0usize;
    for t in 0..ticks as usize {
        sim.run(1);
        let councils: Vec<f64> = sim
            .institutions
            .iter()
            .filter(|i| i.kind == InstitutionKind::Council)
            .map(|i| i.legitimacy.to_f64())
            .collect();
        let legit = if councils.is_empty() {
            f64::NAN
        } else {
            councils.iter().sum::<f64>() / councils.len() as f64
        };
        let pool = sim.faction_pool_mean_grievance();
        let excess = (pool - CRISIS_GRIEVANCE_ANCHOR).max(0.0);
        let factions = sim
            .institutions
            .iter()
            .filter(|i| i.kind == InstitutionKind::Faction)
            .count();
        if factions > last_factions {
            s.formations.push(t);
            last_factions = factions;
        }
        s.legitimacy.push(legit);
        s.grievance_excess.push(excess);
        s.pressure.push(sim.faction_crisis_pressure());
        s.baseline.push(sim.council_mandate_baseline());
    }
    s
}

fn pct(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let i = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[i]
}

/// Replay the accumulator offline on a sampled series.
///
/// `tau = None` reproduces the shipped ABSOLUTE deficit; `Some(tau)` uses the
/// relative form `max(0, baseline − legit)` with a cold-adopting EWMA baseline
/// (i381's rule: a zero baseline would make every young world maximally anomalous).
/// No formation reset is applied, so a law's drive is comparable across worlds —
/// report it as a drive, not as the shipped pressure level.
fn replay(series: &WorldSeries, tau: Option<f64>, ticks: usize) -> (f64, f64, usize, f64) {
    let mut pressure = 0.0f64;
    let mut baseline = series.legitimacy.first().copied().unwrap_or(0.0);
    let mut max_pressure = 0.0f64;
    let mut crossings = 0usize;
    let mut was_above = false;
    let mut deficit_units = 0.0f64;
    for t in 0..ticks.min(series.legitimacy.len()) {
        let legit = series.legitimacy[t];
        let deficit = match tau {
            None => (ABSOLUTE_REF - legit).max(0.0),
            Some(tau) => {
                if t == 0 {
                    baseline = legit;
                } else {
                    baseline += (legit - baseline) / tau;
                }
                (baseline - legit).max(0.0)
            }
        };
        deficit_units += deficit * CRISIS_PRESSURE_BUILD_RATE;
        let excess = series.grievance_excess[t];
        if excess > 0.0 || deficit > 0.0 {
            pressure += excess * CRISIS_PRESSURE_BUILD_RATE + deficit * CRISIS_PRESSURE_BUILD_RATE;
        } else {
            pressure *= 1.0 - CRISIS_PRESSURE_BLEED_RATE;
        }
        pressure = pressure.clamp(0.0, 1.0);
        max_pressure = max_pressure.max(pressure);
        let above = pressure >= FACTION_FORMATION_PRESSURE_THRESHOLD;
        if above && !was_above {
            crossings += 1;
        }
        was_above = above;
    }
    let at_10k = if series.legitimacy.len() > 10_000 {
        let mut p = 0.0f64;
        let mut b = series.legitimacy[0];
        for t in 0..10_000 {
            let legit = series.legitimacy[t];
            let deficit = match tau {
                None => (ABSOLUTE_REF - legit).max(0.0),
                Some(tau) => {
                    if t == 0 {
                        b = legit;
                    } else {
                        b += (legit - b) / tau;
                    }
                    (b - legit).max(0.0)
                }
            };
            let excess = series.grievance_excess[t];
            if excess > 0.0 || deficit > 0.0 {
                p += excess * CRISIS_PRESSURE_BUILD_RATE + deficit * CRISIS_PRESSURE_BUILD_RATE;
            } else {
                p *= 1.0 - CRISIS_PRESSURE_BLEED_RATE;
            }
            p = p.clamp(0.0, 1.0);
        }
        p
    } else {
        f64::NAN
    };
    (at_10k, max_pressure, crossings, deficit_units)
}

fn report(label: &str, scenario: Option<Scenario>, seed: u64, ticks: u64, w: u32, h: u32, n: u32) {
    let s = run_world(scenario, seed, ticks, w, h, n);
    let mut sorted: Vec<f64> = s.legitimacy.clone();
    sorted.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let below = s.legitimacy.iter().filter(|l| **l < ABSOLUTE_REF).count();
    let excess_live = s.grievance_excess.iter().filter(|e| **e > 0.0).count();
    let above_arm = s
        .pressure
        .iter()
        .filter(|p| **p >= FACTION_FORMATION_PRESSURE_THRESHOLD)
        .count();
    let len = s.legitimacy.len() as f64;

    // Per-term drive integrals: how many pressure units each channel contributed.
    let legit_units: f64 = s
        .legitimacy
        .iter()
        .map(|l| (ABSOLUTE_REF - l).max(0.0) * CRISIS_PRESSURE_BUILD_RATE)
        .sum();
    let grief_units: f64 = s
        .grievance_excess
        .iter()
        .map(|e| e * CRISIS_PRESSURE_BUILD_RATE)
        .sum();

    println!("══ {label}  (seed {seed}, {ticks} ticks, N={n}, {w}x{h}) ══");
    println!(
        "  legitimacy  min {:.3}  p05 {:.3}  p50 {:.3}  p95 {:.3}  max {:.3}",
        pct(&sorted, 0.0),
        pct(&sorted, 0.05),
        pct(&sorted, 0.50),
        pct(&sorted, 0.95),
        pct(&sorted, 1.0)
    );
    println!(
        "  cliff/deficit open rate (legit < {ABSOLUTE_REF}):  {:>6.2}%  ({} of {} ticks)",
        100.0 * below as f64 / len,
        below,
        s.legitimacy.len()
    );
    println!(
        "  grievance-excess live share:               {:>6.2}%",
        100.0 * excess_live as f64 / len
    );
    println!(
        "  shipped pressure  max {:.3}   ticks >= arm {:.2}   at 10K {}",
        s.pressure.iter().copied().fold(0.0f64, f64::max),
        100.0 * above_arm as f64 / len,
        if s.legitimacy.len() > 10_000 {
            format!("{:.3}", s.pressure[9_999])
        } else {
            "n/a".to_string()
        }
    );
    println!(
        "  build-term integral  legitimacy {:.4}  grievance {:.4}  →  legitimacy share {:.2}%",
        legit_units,
        grief_units,
        if legit_units + grief_units > 0.0 {
            100.0 * legit_units / (legit_units + grief_units)
        } else {
            0.0
        }
    );
    // Attribution of every formation: which arm could have armed it.
    // The pre-step pressure is used because a formation RESETS the tank inside
    // the same tick — reading the post-tick value would score every
    // pressure-armed formation as unattributed (a one-tick-increment caveat).
    let mut collapse_only = 0;
    let mut pressure_only = 0;
    let mut both = 0;
    for &t in &s.formations {
        let collapse = s.legitimacy[t]
            < s.baseline[t.saturating_sub(1)] - factions::LEGITIMACY_COLLAPSE_MARGIN;
        let rearm_ok = t as u64
            >= s.formations
                .iter()
                .filter(|&&x| x < t)
                .map(|&x| x as u64)
                .max()
                .map_or(0, |last| last + FACTION_REARM_COOLDOWN_TICKS);
        let pressure =
            s.pressure[t.saturating_sub(1)] >= FACTION_FORMATION_PRESSURE_THRESHOLD && rearm_ok;
        match (collapse, pressure) {
            (true, false) => collapse_only += 1,
            (false, true) => pressure_only += 1,
            (true, true) => both += 1,
            (false, false) => {}
        }
    }
    println!(
        "  formations {}  →  collapse-only {collapse_only}  pressure-only {pressure_only}  both {both}",
        s.formations.len()
    );

    // Leg D — the SHIPPED (i382) relative law, live: the sim folds its own
    // baseline, so these numbers are the law in situ, not a replay. The arm
    // reads the baseline of the PREVIOUS tick (it is folded after the read).
    {
        let mut opens = 0usize;
        let mut episodes = 0usize;
        let mut in_dip = false;
        let mut deficit_units = 0.0f64;
        for t in 0..s.legitimacy.len() {
            let base = s.baseline[t.saturating_sub(1)];
            let deficit = (base - s.legitimacy[t]).max(0.0);
            deficit_units += deficit * CRISIS_PRESSURE_BUILD_RATE;
            let open = s.legitimacy[t] < base - factions::LEGITIMACY_COLLAPSE_MARGIN;
            if open {
                opens += 1;
                if !in_dip {
                    episodes += 1;
                }
            }
            in_dip = open;
        }
        let bmin = s.baseline.iter().copied().fold(f64::INFINITY, f64::min);
        let bmax = s.baseline.iter().copied().fold(0.0f64, f64::max);
        println!(
            "  SHIPPED i382  mandate baseline {:.3}–{:.3}  collapse arm opens {:>6.2}%  episodes {episodes}  deficit {deficit_units:.4}",
            bmin,
            bmax,
            100.0 * opens as f64 / len
        );
    }

    // Leg B — offline law comparison on the same series, kept as the SIZING
    // record that chose the shipped τ (the law landed in i382; these lines are
    // the pre-landing candidates, not the running law — see Leg D).
    // `deficit` = the pressure units the legitimacy term contributed.
    for tau in [60.0f64, 120.0, 240.0, 480.0, 1080.0, 3240.0, 10_000.0] {
        let (at_10k, max_pressure, crossings, deficit) = replay(&s, Some(tau), ticks as usize);
        println!(
            "    relative τ={tau:>6.0}: pressure@10K {:>6}  max {:.3}  crossings {crossings}  deficit {deficit:.4}",
            if at_10k.is_nan() {
                "n/a".to_string()
            } else {
                format!("{at_10k:.3}")
            },
            max_pressure
        );
    }
    let (at_10k_abs, max_abs, cross_abs, deficit_abs) = replay(&s, None, ticks as usize);
    println!(
        "    shipped  absolute : pressure@10K {:>6}  max {:.3}  crossings {cross_abs}  deficit {deficit_abs:.4}",
        if at_10k_abs.is_nan() {
            "n/a".to_string()
        } else {
            format!("{at_10k_abs:.3}")
        },
        max_abs
    );

    // Leg C — the INSTANT arm, made relative: `legit < baseline − margin`. The
    // shipped cliff arm is instantaneous (one tick below 0.5 arms), while the
    // accumulator integrates — so a brief but deep mandate collapse (the Q5
    // calm-village dip to 0.144) contributes almost no pressure and would be
    // LOST by a deficit-only redesign. Count the distinct dip episodes each
    // candidate opens, and compare with the shipped open rate.
    println!(
        "    instant arm (shipped absolute) opens {:.2}% of ticks; relative candidates:",
        100.0 * below as f64 / len
    );
    for tau in [60.0f64, 100.0, 240.0] {
        for margin in [0.02f64, 0.05, 0.10] {
            let mut base = s.legitimacy.first().copied().unwrap_or(0.0);
            let mut opens = 0usize;
            let mut episodes = 0usize;
            let mut in_dip = false;
            for t in 0..s.legitimacy.len() {
                let legit = s.legitimacy[t];
                if t > 0 {
                    base += (legit - base) / tau;
                }
                let open = legit < base - margin;
                if open {
                    opens += 1;
                    if !in_dip {
                        episodes += 1;
                    }
                }
                in_dip = open;
            }
            println!(
                "      τ={tau:>4.0} margin {margin:.2}: opens {:>6.2}%  episodes {episodes}",
                100.0 * opens as f64 / len
            );
        }
    }
    println!(
        "    [consts] anchor {CRISIS_GRIEVANCE_ANCHOR}  build {CRISIS_PRESSURE_BUILD_RATE}  \
         bleed {CRISIS_PRESSURE_BLEED_RATE}  arm {FACTION_FORMATION_PRESSURE_THRESHOLD}"
    );
    let _ = factions::FORMATION_GRIEVANCE_THRESHOLD;
    println!();
}

fn main() {
    println!("i382 — the faction legitimacy channel: dead absolute reference, or live?");
    println!("      (both the legacy cliff arm AND the i240 accumulator deficit read the");
    println!("       same absolute 0.5; if legitimacy never crosses it, BOTH are inert)\n");

    for seed in [42u64, 7, 23] {
        report("CALM village 16x16 N=12", None, seed, 50_000, 16, 16, 12);
        report("CALM town 46x46 N=48", None, seed, 20_000, 46, 46, 48);
        report(
            "PESTILENCE town 46x46 N=48",
            Some(Scenario::pestilence()),
            seed,
            20_000,
            46,
            46,
            48,
        );
        report(
            "COLLAPSE village 16x16 N=12",
            Some(Scenario::collapse()),
            seed,
            4_320,
            16,
            16,
            12,
        );
        report(
            "FAMINE village 16x16 N=12",
            Some(Scenario::famine()),
            seed,
            20_000,
            16,
            16,
            12,
        );
    }
}
