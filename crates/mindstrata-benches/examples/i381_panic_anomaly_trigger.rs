//! i381 — size the relative/anomaly moral-panic trigger before shipping it.
//!
//! i378 measured the trigger's headroom and found the shipped absolute bar
//! (`avg_charge ≥ 0.55 AND panic_ratio ≥ 0.30`) sitting *inside* the body of its
//! own input distribution: firing legs cleared it by 2.5–9.4%, one swept crisis
//! seed missed by 0.5%. That is why the firing seed family has been renamed in
//! three separate iterations. The structural fix (i379's Class-4 ruling) is a
//! trigger whose bar is **relative to the population's own slow baseline**, plus
//! an absolute floor so a quiet world stays dark. The floor and the anomaly ratio
//! are law constants, so they must be sized by evidence, not guessed.
//!
//! This probe samples the trigger's two raw inputs every tick (per proposition),
//! for the crisis family AND calm worlds, then evaluates candidate laws
//! **offline** against the same series. Offline because the constants must be
//! chosen before the code changes — and because a replayed series lets many
//! candidates be scored on one expensive run. The shipped law is then re-measured
//! end-to-end by the integration suite (which discovers which crisis members
//! register rather than naming them).
//!
//! What the table must show for a law to be shippable:
//!   * every calm world stays dark (0 panics) — a relative bar must not fire on
//!     a stably-warm population;
//!   * the crisis family still produces at least one firing member (liveness);
//!   * the firing members' margin is no longer ~1–9% (knife-edge), i.e. the
//!     anomaly leg discriminates rather than the distribution's edge.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i381_panic_anomaly_trigger`

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::Simulation;

const CRISIS_SEEDS: [u64; 10] = [7, 1, 23, 11, 46, 5, 42, 13, 99, 3];
const CALM_SEEDS: [u64; 3] = [42, 7, 1];
const TICKS: u64 = 20_000;
/// The shipped panic cooldown — a panic is discrete, and the offline evaluator
/// must mirror it or its counts are not comparable to the registry's.
const COOLDOWN: u64 = 300;
/// The ratio leg (`panic_ratio ≥ 0.30`) is unchanged by i381.
const RATIO_LEG: f64 = 0.30;

/// Per-tick `(avg_charge, panic_ratio)` for one proposition in one world.
struct Series {
    label: String,
    prop: u64,
    calm: bool,
    points: Vec<(f64, f64)>,
}

fn sample(scenario: Scenario, seed: u64, label: &str, calm: bool) -> Vec<Series> {
    let mut sc = scenario;
    sc.seed = seed;
    sc.ticks = TICKS;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    let mut series: Vec<Series> = (0..=1u64)
        .map(|prop| Series {
            label: format!("{label}/{seed}#{prop}"),
            prop,
            calm,
            points: Vec::with_capacity(TICKS as usize),
        })
        .collect();
    for _ in 0..TICKS {
        sim.tick();
        for s in series.iter_mut() {
            let charges: Vec<f64> = sim
                .agents
                .iter()
                .filter_map(|a| {
                    a.beliefs
                        .iter()
                        .find(|b| b.proposition_id == s.prop)
                        .map(|b| b.emotional_charge.to_f64())
                })
                .collect();
            if charges.is_empty() {
                continue;
            }
            let avg = charges.iter().sum::<f64>() / charges.len() as f64;
            let ratio = charges.iter().filter(|c| **c > 0.4).count() as f64 / charges.len() as f64;
            s.points.push((avg, ratio));
        }
    }
    series
}

/// The pre-i381 law, evaluated on a series — the reference row every candidate
/// is compared against.
fn old_law(series: &Series) -> usize {
    let mut count = 0usize;
    let mut last: Option<usize> = None;
    for (i, (avg, ratio)) in series.points.iter().enumerate() {
        if *avg >= 0.55 && *ratio >= RATIO_LEG {
            if last.is_none_or(|l| i - l >= COOLDOWN as usize) {
                count += 1;
                last = Some(i);
            }
        }
    }
    count
}

/// A candidate anomaly law: `avg ≥ max(baseline × ratio_mult, floor)`, where the
/// baseline is a slow EWMA (time constant `tau` ticks) of the population's own
/// mean charge. Returns `(panics, best_margin, anomalous_ticks)` — the margin
/// being the largest `avg / bar` ever reached, which is the headroom reading i378
/// used to call the old law knife-edge.
///
/// `seed_first` is the cold-start rule under test: an unmeasured population's
/// first observation **is** its baseline, so a fresh world is never assumed
/// quiet. Without it a young world's baseline is 0, the floor governs, and every
/// world whose level merely exceeds the floor fires before its own history has
/// been learned (measured: calm/42 fires once at floor 0.40 with a cold start,
/// yet its plateau 0.405 is below its own level × 1.25 forever after).
fn anomaly_law(
    series: &Series,
    floor: f64,
    ratio_mult: f64,
    tau: f64,
    seed_first: bool,
) -> (usize, f64, usize) {
    let alpha = 1.0 / tau;
    let mut baseline = 0.0f64;
    let mut count = 0usize;
    let mut last: Option<usize> = None;
    let mut best_margin = 0.0f64;
    let mut anomalous_ticks = 0usize;
    for (i, (avg, ratio)) in series.points.iter().enumerate() {
        if seed_first && baseline == 0.0 && *avg > 0.0 {
            baseline = *avg;
        }
        let bar = (baseline * ratio_mult).max(floor);
        best_margin = best_margin.max(*avg / bar);
        if *avg >= bar && *ratio >= RATIO_LEG {
            anomalous_ticks += 1;
            if last.is_none_or(|l| i - l >= COOLDOWN as usize) {
                count += 1;
                last = Some(i);
            }
        }
        baseline += (avg - baseline) * alpha;
    }
    (count, best_margin, anomalous_ticks)
}

fn main() {
    println!("i381 — sizing the relative/anomaly panic trigger (crisis @{TICKS}, calm @{TICKS})");
    let mut all: Vec<Series> = Vec::new();
    for seed in CRISIS_SEEDS {
        all.extend(sample(Scenario::pestilence(), seed, "crisis", false));
    }
    for seed in CALM_SEEDS {
        all.extend(sample(Scenario::calm(), seed, "calm", true));
    }

    // ── Leg 1: what the trigger's raw inputs actually look like ──────────────
    println!("\n== raw input distribution (per series) ==");
    println!(
        "{:>16} {:>7} {:>8} {:>8} {:>8} {:>8} {:>8} {:>7}",
        "series", "ticks", "avg_p50", "avg_p95", "avg_max", "r_p95", "r_max", "ratio>0.3"
    );
    for s in &all {
        if s.points.is_empty() {
            continue;
        }
        let mut avgs: Vec<f64> = s.points.iter().map(|p| p.0).collect();
        let mut ratios: Vec<f64> = s.points.iter().map(|p| p.1).collect();
        avgs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        ratios.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = avgs.len();
        let p = |v: &[f64], q: f64| v[((q * (n - 1) as f64) as usize).min(n - 1)];
        let r_leg = s.points.iter().filter(|p| p.1 >= RATIO_LEG).count() as f64 / n as f64;
        println!(
            "{:>16} {n:>7} {:>8.4} {:>8.4} {:>8.4} {:>8.4} {:>8.4} {:>6.1}%",
            s.label,
            p(&avgs, 0.50),
            p(&avgs, 0.95),
            avgs[n - 1],
            p(&ratios, 0.95),
            ratios[n - 1],
            r_leg * 100.0
        );
    }
    println!(
        "\nreading: the ratio leg is the clean discriminator when `ratio>0.3` is\n\
         rare-but-nonzero; the charge leg is the one that needs to stop being absolute."
    );

    // ── Leg 2: the pre-i381 reference law, on the same series ───────────────
    println!("\n== reference: the shipped absolute law (avg ≥ 0.55) ==");
    let mut calm_detail: Vec<String> = Vec::new();
    for s in all.iter().filter(|s| s.calm) {
        let n = old_law(s);
        if n > 0 {
            calm_detail.push(format!("{}:{n}", s.label));
        }
    }
    let mut crisis_fired = 0usize;
    for seed in CRISIS_SEEDS {
        let n: usize = all
            .iter()
            .filter(|s| !s.calm && s.label.starts_with(&format!("crisis/{seed}#")))
            .map(old_law)
            .sum();
        if n > 0 {
            crisis_fired += 1;
        }
        print!("seed {seed}: {n}  ");
    }
    println!("\ncrisis seeds firing: {crisis_fired}/10");
    let calm_old: usize = all.iter().filter(|s| s.calm).map(old_law).sum();
    println!(
        "calm panics: {calm_old} — {} (pre-existing: a calm seed spikes to 0.55)",
        calm_detail.join(" ")
    );

    // ── Leg 3: the candidate grid ───────────────────────────────────────────
    // ── Leg 4: why the ratio leg cannot be the discriminator ────────────────
    //
    // i378 called `panic_ratio ≥ 0.30` "the clean discriminator" on the strength
    // of crisis seeds alone. The calm worlds in this corpus are *warm* — the leg
    // is met for the majority of their ticks — so the claim was never measured
    // where it mattered. Both legs are levels, and the calm world's level is
    // higher than most crisis worlds'. What separates them is not level but
    // **spikiness**: a crisis population's own p95 sits far above its own p50.
    println!("\n== level vs spikiness: p95/p50 of avg_charge, and the ratio-leg share ==");
    println!(
        "{:>16} {:>9} {:>10} {:>13}",
        "series", "p50", "p95/p50", "ratio_leg_share"
    );
    for s in &all {
        if s.points.is_empty() {
            continue;
        }
        let mut avgs: Vec<f64> = s.points.iter().map(|p| p.0).collect();
        avgs.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let n = avgs.len();
        let q = |v: &[f64], q: f64| v[((q * (n - 1) as f64) as usize).min(n - 1)];
        let share = s.points.iter().filter(|p| p.1 >= RATIO_LEG).count() as f64 / n as f64;
        println!(
            "{:>16} {:>9.4} {:>10.2} {:>12.1}%",
            s.label,
            q(&avgs, 0.50),
            q(&avgs, 0.95) / q(&avgs, 0.50),
            share * 100.0
        );
    }

    println!("\n== candidate anomaly laws (corpus: crisis ×10 + calm ×3) ==");
    println!(
        "{:>7} {:>6} {:>6} {:>9} {:>8} {:>9} {:>10} {:>16}",
        "floor",
        "mult",
        "tau",
        "crisis_fir",
        "calm_pan",
        "calm_worlds",
        "cr_margin",
        "firing_seeds"
    );
    type Row = (f64, f64, f64, usize, usize, Vec<String>, f64, Vec<u64>);
    let mut rows: Vec<Row> = Vec::new();
    for floor in [0.40f64, 0.44, 0.47, 0.50, 0.55] {
        for ratio_mult in [1.10f64, 1.25, 1.5] {
            for tau in [3000.0f64, 10_000.0] {
                let mut fired = 0usize;
                let mut seeds: Vec<u64> = Vec::new();
                let mut worst_margin = 0.0f64;
                let mut calm_panics = 0usize;
                let mut calm_worlds: Vec<String> = Vec::new();
                for seed in CRISIS_SEEDS {
                    let (mut n, mut m) = (0usize, 0.0f64);
                    for s in all
                        .iter()
                        .filter(|s| !s.calm && s.label.starts_with(&format!("crisis/{seed}#")))
                    {
                        let (c, margin, _) = anomaly_law(s, floor, ratio_mult, tau, true);
                        n += c;
                        m = m.max(margin);
                    }
                    if n > 0 {
                        fired += 1;
                        seeds.push(seed);
                    }
                    worst_margin = worst_margin.max(m);
                }
                for s in all.iter().filter(|s| s.calm) {
                    let n = anomaly_law(s, floor, ratio_mult, tau, true).0;
                    calm_panics += n;
                    if n > 0 {
                        calm_worlds.push(format!("{}:{n}", s.label));
                    }
                }
                // The old law's own calm firing set is the honest baseline to
                // compare against: calm/1 spikes to 0.55 in a calm world and
                // already fires twice pre-i381.
                let crisis_margin = if fired > 0 { worst_margin } else { f64::NAN };
                println!(
                    "{floor:>7.2} {ratio_mult:>6.2} {tau:>6.0} {fired:>6}/10 {calm_panics:>8} {:>9} {crisis_margin:>10.3} {:>16}",
                    calm_worlds.join(" "),
                    seeds.iter().map(|s| s.to_string()).collect::<Vec<_>>().join(",")
                );
                rows.push((
                    floor,
                    ratio_mult,
                    tau,
                    fired,
                    calm_panics,
                    calm_worlds,
                    crisis_margin,
                    seeds,
                ));
            }
        }
    }

    println!("\n== per-world detail: rows whose calm firing set ⊆ the old law's {{calm/1:2}} ==");
    for (floor, ratio_mult, tau, _fired, _cp, calm_worlds, margin, seeds) in &rows {
        let regression = calm_worlds.iter().any(|w| !w.starts_with("calm/1#"));
        if regression || seeds.is_empty() {
            continue;
        }
        print!("floor={floor:.2} mult={ratio_mult:.2} tau={tau:.0} (margin {margin:.2}): crisis");
        for seed in CRISIS_SEEDS {
            let n: usize = all
                .iter()
                .filter(|s| !s.calm && s.label.starts_with(&format!("crisis/{seed}#")))
                .map(|s| anomaly_law(s, *floor, *ratio_mult, *tau, true).0)
                .sum();
            print!(" {seed}:{n}");
        }
        print!("  | calm");
        for seed in CALM_SEEDS {
            let n: usize = all
                .iter()
                .filter(|s| s.calm && s.label.starts_with(&format!("calm/{seed}#")))
                .map(|s| anomaly_law(s, *floor, *ratio_mult, *tau, true).0)
                .sum();
            print!(" {seed}:{n}");
        }
        println!();
    }
    println!(
        "\nreading: choose by (a) no calm seed fires that the old law did not already\n\
         fire (calm/1 spikes to 0.55 today — pre-existing, not a regression),\n\
         (b) at least one crisis member fires, (c) cr_margin well above the old\n\
         1.02–1.09 knife-edge, (d) the crossover point floor/mult sits inside the\n\
         measured gap [highest calm plateau, lowest crisis spike]."
    );

    // ── Leg 5: what the ANOMALY arm does that no floor can ───────────────────
    //
    // In this corpus the floor is the operating point (the warmest *stable*
    // population, calm/42, plateaus at 0.373 and the crossover is floor/mult =
    // 0.376). So the arm must be sized on the shapes a floor provably cannot
    // handle — a population that *lives* above the bar, and one that drifts up to
    // it. Both are synthetic so the property is unambiguous.
    println!("\n== synthetic shapes: the anomaly arm's own job ==");
    let shapes: Vec<(&str, Vec<(f64, f64)>, f64, f64)> = vec![
        // (label, series, floor, mult) — floor/mult columns below are the pre-i381 law.
        ("flat 0.20 (quiet)", flat(vec![0.20], 20_000), 0.47, 1.25),
        (
            "flat 0.40 (warm, below floor)",
            flat(vec![0.40], 20_000),
            0.47,
            1.25,
        ),
        (
            "flat 0.65 (sustained, above floor)",
            flat(vec![0.65], 20_000),
            0.47,
            1.25,
        ),
        (
            "0.20 → 0.65 step at 5K",
            step(vec![0.20, 0.65], 5_000),
            0.47,
            1.25,
        ),
        ("0.20 with a 300-tick 0.65 spike at 5K", spike(), 0.47, 1.25),
    ];
    println!(
        "{:>38} {:>15} {:>18}",
        "shape", "old law (0.55)", "anomaly (0.47×1.25)"
    );
    for (label, points, floor, mult) in &shapes {
        let s = Series {
            label: (*label).to_string(),
            prop: 1,
            calm: false,
            points: points.clone(),
        };
        let old = old_law(&s);
        let (new, margin, anomalous) = anomaly_law(&s, *floor, *mult, 3000.0, true);
        println!(
            "{label:>38} {old:>15} {new:>10} (margin {margin:.2}, {anomalous} anomalous ticks)"
        );
    }
    println!(
        "reading: a FLAT population above the bar must fire a BURST and then go quiet\n\
         (the baseline rises to meet it) — that is what makes a panic an event rather\n\
         than a new equilibrium, and it is invisible to a fixed threshold. A spike on\n\
         a quiet population must fire exactly as the old law does."
    );
}

/// A flat population at `levels[0]` for `ticks`, with a charged share matching it.
fn flat(levels: Vec<f64>, ticks: usize) -> Vec<(f64, f64)> {
    let level = levels[0];
    let ratio = if level > 0.4 { 0.5 } else { 0.0 };
    (0..ticks).map(|_| (level, ratio)).collect()
}

/// A population that steps to `levels[1]` after `run_in` ticks at `levels[0]`.
fn step(levels: Vec<f64>, run_in: usize) -> Vec<(f64, f64)> {
    (0..20_000)
        .map(|i| {
            let level = if i < run_in { levels[0] } else { levels[1] };
            (level, if level > 0.4 { 0.5 } else { 0.0 })
        })
        .collect()
}

/// A quiet population with one 300-tick charged spike at 5K.
fn spike() -> Vec<(f64, f64)> {
    (0..20_000)
        .map(|i| {
            if (5000..5300).contains(&i) {
                (0.65, 0.5)
            } else {
                (0.20, 0.0)
            }
        })
        .collect()
}
