//! i305 — Difficulty-lever row 2 residual probe: fulfillment thresholds.
//!
//! `docs/balance/difficulty-levers.md` row 2 is "Need decay / fulfillment
//! thresholds". i303 promoted the DECAY half to a live band surface; the
//! THRESHOLD half — the five goal-generation gates that decide how much deficit
//! an agent tolerates before it acts (`retain 0.3`, `Eat/Drink 0.5`, `Rest 0.6`,
//! `Socialize/Worship 0.7`, all inline consts in `systems::system_goal_generation`)
//! — was hard-coded. This probe measures that surface, the headroom that sizes
//! the band, and the band's response.
//!
//! ## The horizon trap this probe deliberately avoids
//!
//! Need decay runs at 1e-4–2e-4 per tick, so a need starting near 0.1 only
//! CROSSES a 0.7 gate around tick 6000. Measured at 2000 ticks alone, three
//! need-driven producers (Eat, Socialize, Worship) read as dead — and at 20K
//! they are all live (Worship especially: 41.5% of agent-ticks). A single
//! horizon would have produced a false dead-producer finding, so every producer
//! claim here is made at BOTH horizons.
//!
//! ## Legs
//!
//!   1. CONFIGURATION SURFACE: goal-gate-carrying keys of `SimParameters` that
//!      differ between bands (0 before this iteration) + the inline gate table.
//!   2. HEADROOM: per horizon, the per-agent-tick rate at which each goal kind
//!      is live and each need sits above its gate — what a gate scale moves and
//!      what must not go to zero.
//!   3. BAND RESPONSE: Lenient/Standard/Harsh × 12 seeds × both horizons — the
//!      catalog's predicted direction (a low-gate village responds to smaller
//!      deficits, so its need-driven goals run more often) measured, not
//!      assumed, with per-kind tables printed in full.
//!
//! VERDICT CONTRACT: LIVE ⇔ the surface is band-sensitive
//!   ∧ Standard ≡ canon (params serde identity)
//!   ∧ at EACH horizon, every producer canon runs materially there (≥1% of
//!     agent-ticks) keeps a ≥0.1% duty in every band — a band may not kill a
//!     producer canon runs AT THAT HORIZON (measured honestly at 2K, the Harsh
//!     band does drop the Drink producer to ~0: thirst only crosses 0.7 in the
//!     tail at 2K. That is the harsh semantics, and the producer is live again
//!     at 20K — the horizon at which the calibration actually lives)
//!   ∧ the THRESHOLD half IN ISOLATION (gate scale swept with the row-2 decay
//!     rates pinned at canon) is monotone lenient > standard > harsh in the
//!     five need-driven channels' combined duty at both horizons
//!   ∧ 12/12 seeds alive in every band × horizon.
//!
//! Why the isolated leg is the contract and the combined band is the finding:
//! row 2 is one lever with two halves that push the SAME way (Harsh both
//! accumulates deficit faster and tolerates more of it), so the combined
//! band's duty cycle is the product of the two and is not expected to be
//! monotone — measured here at 2000 ticks as lenient 0.2304 > harsh 0.0943 >
//! standard 0.0587, and at 20K as 0.7045 > 0.6410 > 0.6361 (harsh marginally
//! over standard). The row's felt effect was promoted in i303 on the standing
//! deficit (aggregate +27%, monotone); THIS iteration promotes the threshold
//! half on an observable where only the threshold half moves.
//!
//! Run: cargo run --release -p mindstrata-benches --example i305_goal_gate_bands

use mindstrata_sim::parameters::{DifficultyProfile, SimParameters};
use mindstrata_sim::person::GoalKind;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// The i268 seed family (the calibration-audit stability instrument).
const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
/// Both horizons are measured (see the module doc: a gate that looks dead at
/// 2K can be live at 20K).
const HORIZONS: [u64; 2] = [2_000, 20_000];

/// The canon gates (inline consts in `system_goal_generation` before i305).
const CANON_GATES: [(&str, f64); 5] = [
    ("Eat/Drink", 0.5),
    ("Rest", 0.6),
    ("Socialize", 0.7),
    ("Worship", 0.7),
    ("retain", 0.3),
];

const TRACKED_GOALS: [GoalKind; 7] = [
    GoalKind::Eat,
    GoalKind::Drink,
    GoalKind::Rest,
    GoalKind::Work,
    GoalKind::Socialize,
    GoalKind::Worship,
    GoalKind::SeekSafety,
];

/// The five need-driven producers whose duty cycle the band is expected to move
/// (`Work` is identity/emotion-driven, `SeekSafety` is fear-driven).
const NEED_DRIVEN: [usize; 5] = [0, 1, 2, 4, 5];

/// The five need channels, in the order the bands are felt.
const CHANNELS: [(&str, f64); 5] = [
    ("hunger", 0.5),
    ("thirst", 0.5),
    ("fatigue", 0.6),
    ("social", 0.7),
    ("meaning", 0.7),
];

fn goal_name(kind: GoalKind) -> &'static str {
    match kind {
        GoalKind::Eat => "Eat",
        GoalKind::Drink => "Drink",
        GoalKind::Rest => "Rest",
        GoalKind::Work => "Work",
        GoalKind::Socialize => "Socialize",
        GoalKind::Worship => "Worship",
        GoalKind::SeekSafety => "SeekSafety",
    }
}

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

struct Family {
    /// Per-agent-tick presence rate of each tracked goal kind.
    goal_rate: Vec<(&'static str, f64)>,
    /// Per-agent-tick rate at which each need channel sits above its CANON gate
    /// (the pre-i305 gate — kept as the reference line so the table is
    /// comparable across bands).
    above_gate: [f64; 5],
    /// Mean final deficits, in the channel order above.
    deficits: [f64; 5],
    alive_seeds: usize,
    agent_ticks: u64,
    /// Per-channel need samples of the whole family run.
    samples: [Vec<f64>; 5],
}

impl Family {
    /// Combined duty cycle of the five need-driven producers.
    fn need_driven_duty(&self) -> f64 {
        NEED_DRIVEN.iter().map(|i| self.goal_rate[*i].1).sum()
    }
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * q).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn measure(profile: DifficultyProfile, ticks: u64) -> Family {
    measure_params(SimParameters::with_difficulty(profile), ticks)
}

/// Measure with the ROW-2 DECAY RATES pinned at canon and only the gate scale
/// moved — the isolating experiment for the threshold half (row 2's other half
/// would otherwise mask its direction; see the module doc).
fn measure_gates_only(scale: f64, ticks: u64) -> Family {
    let mut p = SimParameters::default();
    p.goal_gate_scale = mindstrata_core::fixed::Fixed::from_f64(scale);
    measure_params(p, ticks)
}

fn measure_params(params: SimParameters, ticks: u64) -> Family {
    let mut goal_counts = vec![0u64; TRACKED_GOALS.len()];
    let mut above = [0u64; 5];
    let mut deficits = [0.0f64; 5];
    let mut alive_seeds = 0usize;
    let mut agent_ticks = 0u64;
    let mut samples: [Vec<f64>; 5] = Default::default();

    for &seed in &SEEDS {
        let mut sim = Simulation::new(config(seed, ticks));
        sim.params = params;
        sim.populate();
        for _ in 0..ticks {
            sim.tick();
            for a in &sim.agents {
                agent_ticks += 1;
                let needs = [
                    a.needs.hunger.to_f64(),
                    a.needs.thirst.to_f64(),
                    a.needs.fatigue.to_f64(),
                    a.needs.social.to_f64(),
                    a.needs.meaning.to_f64(),
                ];
                for (i, v) in needs.iter().enumerate() {
                    samples[i].push(*v);
                    if *v > CHANNELS[i].1 {
                        above[i] += 1;
                    }
                }
                for (i, kind) in TRACKED_GOALS.iter().enumerate() {
                    if a.goals.iter().any(|g| g.kind == *kind) {
                        goal_counts[i] += 1;
                    }
                }
            }
        }
        let n = sim.agents.len().max(1) as f64;
        if !sim.agents.is_empty() {
            alive_seeds += 1;
        }
        for (slot, f) in deficits.iter_mut().zip([
            |a: &mindstrata_sim::sim::AgentBundle| a.needs.hunger.to_f64(),
            |a: &mindstrata_sim::sim::AgentBundle| a.needs.thirst.to_f64(),
            |a: &mindstrata_sim::sim::AgentBundle| a.needs.fatigue.to_f64(),
            |a: &mindstrata_sim::sim::AgentBundle| a.needs.social.to_f64(),
            |a: &mindstrata_sim::sim::AgentBundle| a.needs.meaning.to_f64(),
        ]) {
            *slot += sim.agents.iter().map(f).sum::<f64>() / n;
        }
    }
    let denom = agent_ticks.max(1) as f64;
    let family_n = SEEDS.len() as f64;
    Family {
        goal_rate: TRACKED_GOALS
            .iter()
            .enumerate()
            .map(|(i, k)| (goal_name(*k), goal_counts[i] as f64 / denom))
            .collect(),
        above_gate: above.map(|c| c as f64 / denom),
        deficits: deficits.map(|d| d / family_n),
        alive_seeds,
        agent_ticks,
        samples,
    }
}

/// Goal-gate-carrying keys of `SimParameters` that differ between two bands:
/// the direct measure of whether the band profile reaches this lever at all.
fn goal_gate_keys_differing(a: &SimParameters, b: &SimParameters) -> usize {
    let parse = |s: &str| -> Vec<(String, String)> {
        s.trim_matches(|c| c == '{' || c == '}')
            .split(", ")
            .filter_map(|kv| {
                kv.split_once(": ")
                    .map(|(k, v)| (k.to_string(), v.to_string()))
            })
            .collect()
    };
    let (pa, pb) = (parse(&format!("{a:?}")), parse(&format!("{b:?}")));
    pa.iter()
        .zip(pb.iter())
        .filter(|((ka, _), (kb, _))| ka == kb && ka.contains("goal_gate"))
        .filter(|((_, va), (_, vb))| va != vb)
        .count()
}

fn gates_table(profile: DifficultyProfile) -> [i64; 6] {
    let p = SimParameters::with_difficulty(profile);
    let g = mindstrata_sim::systems::GoalGates::for_params(&p);
    [
        g.eat.to_raw(),
        g.drink.to_raw(),
        g.rest.to_raw(),
        g.socialize.to_raw(),
        g.worship.to_raw(),
        g.retain.to_raw(),
    ]
}

fn print_band(name: &str, f: &Family) {
    println!(
        "  {name:<10} {:>8.4} {:>8.4} {:>8.4} {:>9.4} {:>9.4} {:>11.4}",
        f.goal_rate[0].1,
        f.goal_rate[1].1,
        f.goal_rate[2].1,
        f.goal_rate[4].1,
        f.goal_rate[5].1,
        f.need_driven_duty()
    );
    println!(
        "             work {:.4} | seeksafety {:.4} | alive {}/{} | deficits h{:.4} t{:.4} f{:.4} s{:.4} m{:.4}",
        f.goal_rate[3].1,
        f.goal_rate[6].1,
        f.alive_seeds,
        SEEDS.len(),
        f.deficits[0],
        f.deficits[1],
        f.deficits[2],
        f.deficits[3],
        f.deficits[4]
    );
}

fn main() {
    println!(
        "i305 difficulty-lever row 2 residual — fulfillment thresholds, family of {} at {:?}",
        SEEDS.len(),
        HORIZONS
    );

    // ── Leg 1: configuration surface ─────────────────────────────────────
    println!("\n[leg 1] goal-gate configuration surface");
    println!("  inline gates the pass read before i305 (system_goal_generation):");
    for (name, v) in CANON_GATES {
        println!("    {name:<11} {v:.2}");
    }
    let lenient = SimParameters::with_difficulty(DifficultyProfile::Lenient);
    let standard = SimParameters::with_difficulty(DifficultyProfile::Standard);
    let harsh = SimParameters::with_difficulty(DifficultyProfile::Harsh);
    println!(
        "  SimParameters goal-gate keys (lenient vs standard): {}",
        goal_gate_keys_differing(&lenient, &standard)
    );
    println!(
        "  SimParameters goal-gate keys (standard vs harsh):   {}",
        goal_gate_keys_differing(&standard, &harsh)
    );
    println!("  resolved gates (raw units, 1 raw = 1e-4):");
    println!(
        "    {:<10} {:>6} {:>6} {:>6} {:>10} {:>8} {:>7}",
        "band", "eat", "drink", "rest", "socialize", "worship", "retain"
    );
    for (name, profile) in [
        ("lenient", DifficultyProfile::Lenient),
        ("standard", DifficultyProfile::Standard),
        ("harsh", DifficultyProfile::Harsh),
    ] {
        let g = gates_table(profile);
        println!(
            "    {name:<10} {:>6} {:>6} {:>6} {:>10} {:>8} {:>7}",
            g[0], g[1], g[2], g[3], g[4], g[5]
        );
    }
    let surface_band_sensitive = goal_gate_keys_differing(&lenient, &standard) > 0
        && goal_gate_keys_differing(&standard, &harsh) > 0;

    // ── Leg 2: canon-band headroom, per horizon ──────────────────────────
    let mut canon_by_horizon: Vec<(u64, Family)> = Vec::new();
    for ticks in HORIZONS {
        println!("\n[leg 2] canon-band headroom at {ticks} ticks/seed");
        let canon = measure(DifficultyProfile::Standard, ticks);
        println!("  agent-ticks sampled: {}", canon.agent_ticks);
        println!(
            "  {:<11} {:>12}  (per-agent-tick rate)",
            "goal kind", "live rate"
        );
        for (name, rate) in &canon.goal_rate {
            println!("    {name:<11} {rate:>12.4}");
        }
        println!(
            "  end deficits: hunger {:.4} | thirst {:.4} | fatigue {:.4} | social {:.4} | meaning {:.4}",
            canon.deficits[0],
            canon.deficits[1],
            canon.deficits[2],
            canon.deficits[3],
            canon.deficits[4]
        );
        println!("  alive seeds: {}/{}", canon.alive_seeds, SEEDS.len());
        println!("  per-channel need distribution (agent-ticks):");
        println!(
            "    {:<9} {:>7} {:>7} {:>7} {:>7} {:>7} {:>10}  {:<9} {:>7}",
            "channel", "p50", "p75", "p90", "p99", "max", "above_gate", "canon", "state"
        );
        for i in 0..5 {
            let mut s = canon.samples[i].clone();
            s.sort_by(|a, b| a.partial_cmp(b).unwrap());
            let above =
                s.iter().filter(|v| **v > CHANNELS[i].1).count() as f64 / s.len().max(1) as f64;
            println!(
                "    {:<9} {:>7.4} {:>7.4} {:>7.4} {:>7.4} {:>7.4} {:>10.4}  {:<9} {:>7}",
                CHANNELS[i].0,
                percentile(&s, 0.50),
                percentile(&s, 0.75),
                percentile(&s, 0.90),
                percentile(&s, 0.99),
                s.last().copied().unwrap_or(0.0),
                above,
                format!("{:.2}", CHANNELS[i].1),
                if above == 0.0 { "DEAD" } else { "live" }
            );
        }
        canon_by_horizon.push((ticks, canon));
    }

    // Producer-liveness matrix (dead only if dead at EVERY horizon).
    println!("\n[leg 2b] producer liveness per horizon (rate > 0)");
    print!("  {:<11}", "goal kind");
    for (ticks, _) in &canon_by_horizon {
        print!(" {ticks:>10}");
    }
    println!();
    let mut dead_everywhere = Vec::new();
    for (i, name) in canon_by_horizon[0]
        .1
        .goal_rate
        .iter()
        .map(|(n, _)| *n)
        .collect::<Vec<_>>()
        .iter()
        .enumerate()
        .map(|(i, n)| (i, *n))
    {
        let rates: Vec<f64> = canon_by_horizon
            .iter()
            .map(|(_, f)| f.goal_rate[i].1)
            .collect();
        print!("  {name:<11}");
        for r in &rates {
            print!(" {r:>10.5}");
        }
        println!();
        if rates.iter().all(|r| *r == 0.0) {
            dead_everywhere.push(name);
        }
    }
    println!(
        "  dead at every measured horizon: {}",
        if dead_everywhere.is_empty() {
            "none".to_string()
        } else {
            dead_everywhere.join(", ")
        }
    );

    // ── Leg 3: band response ─────────────────────────────────────────────
    let bands = [
        ("lenient", DifficultyProfile::Lenient),
        ("standard", DifficultyProfile::Standard),
        ("harsh", DifficultyProfile::Harsh),
    ];
    // §4.4 liveness invariant: a band may not kill a producer another band
    // runs. "Materially live" = canon duty ≥ 1% of agent-ticks; the floor a
    // band must respect is 0.1% (an order of magnitude below canon).
    const LIVE_THRESHOLD: f64 = 0.01;
    const KILL_FLOOR: f64 = 0.001;
    let mut alive_ok = true;
    let mut producers_alive = true;
    for ticks in HORIZONS {
        println!("\n[leg 3] row-2 band response (both halves) at {ticks} ticks/seed");
        println!(
            "  {:<10} {:>8} {:>8} {:>8} {:>9} {:>9} {:>11}        (per-agent-tick rates)",
            "band", "Eat", "Drink", "Rest", "Socialize", "Worship", "need-driven"
        );
        for (name, profile) in bands.iter() {
            let f = measure(*profile, ticks);
            print_band(name, &f);
            if f.alive_seeds != SEEDS.len() {
                alive_ok = false;
            }
        }
    }

    // ── Leg 3b: the threshold half IN ISOLATION (decay rates pinned) ─────
    println!("\n[leg 3b] threshold half in isolation (row-2 decay pinned at canon)");
    let gate_scales = [(0.6, "lenient"), (1.0, "standard"), (1.4, "harsh")];
    let mut duty: [Vec<f64>; 3] = Default::default();
    let mut isolated_direction = true;
    for ticks in HORIZONS {
        println!("  at {ticks} ticks/seed");
        println!(
            "  {:<10} {:>8} {:>8} {:>8} {:>9} {:>9} {:>11}",
            "gate scale", "Eat", "Drink", "Rest", "Socialize", "Worship", "need-driven"
        );
        let mut families: Vec<Family> = Vec::new();
        for (bi, (scale, name)) in gate_scales.iter().enumerate() {
            let f = measure_gates_only(*scale, ticks);
            print_band(name, &f);
            duty[bi].push(f.need_driven_duty());
            families.push(f);
        }
        // §4.4 liveness invariant, per horizon (the leg-2 lesson: a horizon at
        // which canon itself does not run a producer is not a horizon at which
        // a band can be accused of killing it). Candidates are the producers
        // the CANON family runs materially (≥1%) AT THIS HORIZON; each must
        // keep ≥0.1% in every band.
        let canon = &families[1];
        for (i, (kind, canon_rate)) in canon.goal_rate.iter().enumerate() {
            if *canon_rate < LIVE_THRESHOLD {
                continue;
            }
            for (bi, f) in families.iter().enumerate() {
                let rate = f.goal_rate[i].1;
                if rate < KILL_FLOOR {
                    producers_alive = false;
                    println!(
                        "    KILLED: {kind} canon {canon_rate:.4} → {} band {rate:.4}",
                        gate_scales[bi].1
                    );
                }
            }
        }
        let (l, s, h) = (
            duty[0].last().unwrap(),
            duty[1].last().unwrap(),
            duty[2].last().unwrap(),
        );
        println!(
            "    need-driven duty: lenient {l:.4} | standard {s:.4} | harsh {h:.4} → monotone {} (rows above show every band's full table)",
            l > s && s > h
        );
        isolated_direction &= l > s && s > h;
    }
    let direction = isolated_direction;

    // ── Verdict (pre-registered — see the module doc) ────────────────────
    let params_identical = serde_json::to_string(&SimParameters::default()).unwrap()
        == serde_json::to_string(&standard).unwrap();
    println!("\nleg1_surface_band_sensitive={surface_band_sensitive}");
    println!("leg1_standard_params_identical_to_canon={params_identical}");
    println!(
        "leg2_dead_at_every_horizon={}",
        if dead_everywhere.is_empty() {
            "none".to_string()
        } else {
            dead_everywhere.join(",")
        }
    );
    println!("leg3_no_producer_killed_by_a_band={producers_alive}");
    println!("leg3_all_seeds_alive_every_band={alive_ok}");
    println!("leg3_need_driven_duty_monotone={direction}");
    if surface_band_sensitive && params_identical && producers_alive && alive_ok && direction {
        println!("verdict=GOAL_GATE_BANDS_LIVE");
    } else {
        println!("verdict=GOAL_GATE_BANDS_PARTIAL (see failed leg above)");
    }
}
