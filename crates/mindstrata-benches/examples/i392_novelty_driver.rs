//! i392 row 2 — the `novelty_seeking` gene reaches the exploration driver.
//!
//! i391's census found five dead genes: drawn at `random()`, defaulted, blended
//! at `inherit`, and read by nothing outside `genome.rs`. `novelty_seeking` is
//! the second row to be wired (after `puberty_age`). Its consumer is the i351
//! exploration driver — an individual with a heritable tendency to seek
//! novelty should explore more, and until i392 the driver applied one
//! population-wide coefficient to every carrier.
//!
//! The candidate law this probe evaluates (it is **not** in the tree — this is
//! a measured rejection, see the evidence doc):
//!
//! ```text
//! coef = WANDER_NOVELTY_COEF × (1 + (gene − 0.5) × 0.4)
//! ```
//!
//! i.e. exactly 1.0 at the gene midpoint (0.5, the gene's `Default`), mapping
//! the gene's U(0.1, 0.9) range onto coefficient 1.68–2.32. A span of 0.8
//! (coefficient 1.36–2.64) was measured first and rejected as the more extreme
//! of the two; the span below is the *conservative* variant, and it is also
//! rejected, on the grounds legs B–E measure.
//!
//! Four legs, in the order §4.13 requires — manipulation first, response
//! second:
//!
//! * **A — the manipulation.** The gene's distribution in the corpus, and the
//!   multiplier it implies per agent (spread, population mean).
//! * **B — the causal leg.** Two worlds identical in seed and everything else,
//!   with every agent's gene pinned to the band's low end (0.1) and then its
//!   high end (0.9). If the channel is live, `Wander` wins strictly more
//!   arbitrations in the high-gene world; if it does not move, the wiring is
//!   inert in practice and has to be said so (the row-1 precedent).
//! * **C — the in-vivo leg.** The natural-gene world's census, so the shipped
//!   behaviour can be compared against i351's calibrated band (Wander share
//!   sub-1%, quiet-window wins ~4.9% of quiet windows).
//!
//! Run: `cargo run --release -p mindstrata-benches --example i392_novelty_driver`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::decision_census;
use mindstrata_sim::sim::{SimConfig, Simulation};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

const WARMUP: u64 = 500;
const WINDOW: u64 = 20_000;

/// `ACTION_NAMES` index of `Wander` (decision_census.rs).
const WANDER: usize = 7;
/// `SOURCE_NAMES` index of `utility` — the only source that arbitrates.
const UTILITY: usize = 4;

/// The evaluated candidate's span (see the module docs). Kept here, not in the
/// library: the law was measured and rejected, and an unused public helper in
/// the action layer would be exactly the false affordance this whole act exists
/// to remove.
const CANDIDATE_SPAN: f64 = 0.4;

/// The evaluated candidate law: the multiplier the gene would apply to the
/// i351 exploration coefficient. `1.0` at the gene midpoint, by construction.
fn novelty_driver_scale(gene: Fixed) -> Fixed {
    Fixed::from_f64(1.0 + (gene.to_f64() - 0.5) * CANDIDATE_SPAN)
}

fn calm(n: u32, seed: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: WARMUP + WINDOW,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

/// Pin every agent's gene to one value — the same world with one dial moved.
fn pinned(n: u32, seed: u64, gene: f64) -> Simulation {
    let mut sim = calm(n, seed);
    for a in &mut sim.agents {
        a.embodied.genome.trait_predispositions.novelty_seeking = Fixed::from_f64(gene);
    }
    sim
}

fn pct(sorted: &[f64], p: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let idx = ((sorted.len() - 1) as f64 * p).round() as usize;
    sorted[idx]
}

/// The census reading every leg is stated in.
struct Reading {
    decisions: u64,
    wander_decisions: u64,
    wander_utility_wins: u64,
    arbitrations: u64,
    quiet_samples: u64,
    quiet_wander_wins: u64,
}

/// Run the window under the census and read the counters (silent).
fn read_census(mut sim: Simulation) -> Reading {
    sim.run(WARMUP);
    decision_census::reset();
    decision_census::enable();
    sim.run(WINDOW);
    decision_census::disable();
    let r = decision_census::report();
    Reading {
        decisions: r.total(),
        wander_decisions: r.cross[UTILITY][WANDER],
        wander_utility_wins: r.wander.wins,
        arbitrations: r.utility_samples,
        quiet_samples: r.quiet_samples,
        quiet_wander_wins: r.quiet_wander.wins,
    }
}

fn census_of(label: &str, sim: Simulation) -> Reading {
    let reading = read_census(sim);
    println!("══ {label} ══");
    println!(
        "decisions {} · Wander by deliberation {} ({:.4}% of decisions)",
        reading.decisions,
        reading.wander_decisions,
        reading.wander_decisions as f64 / reading.decisions.max(1) as f64 * 100.0
    );
    println!(
        "arbitrations {} · Wander wins {} ({:.4}%) · quiet windows {} · quiet Wander wins {} ({:.2}%)",
        reading.arbitrations,
        reading.wander_utility_wins,
        reading.wander_utility_wins as f64 / reading.arbitrations.max(1) as f64 * 100.0,
        reading.quiet_samples,
        reading.quiet_wander_wins,
        reading.quiet_wander_wins as f64 / reading.quiet_samples.max(1) as f64 * 100.0
    );
    println!();
    reading
}

/// Leg A — the manipulation: what the gene actually varies, and what the law
/// does to each carrier.
fn leg_a(n: u32, seed: u64) {
    let sim = calm(n, seed);
    let mut genes: Vec<f64> = sim
        .agents
        .iter()
        .map(|a| {
            a.embodied
                .genome
                .trait_predispositions
                .novelty_seeking
                .to_f64()
        })
        .collect();
    genes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let scales: Vec<f64> = genes
        .iter()
        .map(|g| novelty_driver_scale(Fixed::from_f64(*g)).to_f64())
        .collect();
    let mean = |v: &[f64]| v.iter().sum::<f64>() / v.len().max(1) as f64;
    println!("══ A — manipulation (N={n}, seed {seed}) ══");
    println!(
        "gene: min {:.4} p50 {:.4} max {:.4} · mean {:.4} (midpoint 0.5)",
        genes.first().copied().unwrap_or(0.0),
        pct(&genes, 0.5),
        genes.last().copied().unwrap_or(0.0),
        mean(&genes)
    );
    println!(
        "driver multiplier: min {:.4} max {:.4} · mean {:.4} (=1.0 at the gene midpoint, §4.6)",
        scales.first().copied().unwrap_or(1.0),
        scales.last().copied().unwrap_or(1.0),
        mean(&scales)
    );
    println!(
        "implied coefficient (i351 coef 2.0): {:.3}–{:.3}",
        scales.first().copied().unwrap_or(1.0) * 2.0,
        scales.last().copied().unwrap_or(1.0) * 2.0
    );
    // A pinned-gene world bypasses the draw entirely, so state the check the
    // same way row 1 did: the helper is exact at the three reference points.
    println!(
        "helper check: gene 0.1 → {:.4}, 0.5 → {:.4}, 0.9 → {:.4}",
        novelty_driver_scale(Fixed::from_f64(0.1)).to_f64(),
        novelty_driver_scale(Fixed::from_f64(0.5)).to_f64(),
        novelty_driver_scale(Fixed::from_f64(0.9)).to_f64()
    );
    println!();
}

/// Leg B — the causal leg: same seed, one dial (the gene) moved.
///
/// Three arms, and the middle one is the one that matters: **pinned 0.5 is the
/// pre-i392 engine exactly** (multiplier 1.0 → the i351 coefficient), so it is
/// the control every natural-gene reading has to be compared against. A leg
/// with only the two extremes would show the channel works but could not say
/// what the shipped world does.
fn leg_b(n: u32, seed: u64) {
    println!("── B — causal A/B: identical world, gene pinned (N={n}, seed {seed}) ──");
    let low = census_of(
        "gene pinned 0.1 (homebody, coef 1.68)",
        pinned(n, seed, 0.1),
    );
    let control = census_of(
        "gene pinned 0.5 (CONTROL = pre-i392 engine)",
        pinned(n, seed, 0.5),
    );
    let high = census_of("gene pinned 0.9 (seeker, coef 2.32)", pinned(n, seed, 0.9));
    let d = |a: &Reading, b: &Reading| b.wander_decisions as i64 - a.wander_decisions as i64;
    println!(
        "DELTA  Wander by deliberation: 0.1→0.5 {:+} · 0.5→0.9 {:+} · monotone {}",
        d(&low, &control),
        d(&control, &high),
        low.wander_decisions < control.wander_decisions
            && control.wander_decisions < high.wander_decisions
    );
    println!();
}

/// Leg D — the response *shape*: is the driver's response to a coefficient a
/// gradient or a threshold?
///
/// Three pinned points cannot answer that, and the answer decides whether a
/// heritable spread may ride this term at all. A gradient means the gene maps
/// onto a smooth trait difference; a threshold means any gene-driven spread
/// splits the population into two regimes and makes the aggregate share a
/// lottery over founder draws (§4.1 / §4.5 knife-edge).
///
/// Every arm is the same world and seed with one number pinned, so the curve is
/// the surface, not the lottery.
fn leg_d(n: u32, seed: u64) {
    println!("── D — response shape: gene pinned across its range (N={n}, seed {seed}) ──");
    println!("  gene  coef  mult   Wander decisions  quiet-window wins  quiet %");
    for gene in [0.1, 0.2, 0.3, 0.4, 0.5, 0.6, 0.7, 0.8, 0.9] {
        let mult = novelty_driver_scale(Fixed::from_f64(gene)).to_f64();
        let r = read_census(pinned(n, seed, gene));
        println!(
            "  {gene:.2}  {:.2}  {mult:.3}  {:>16}  {:>17}  {:>7.2}%",
            mult * 2.0,
            r.wander_decisions,
            r.quiet_wander_wins,
            r.quiet_wander_wins as f64 / r.quiet_samples.max(1) as f64 * 100.0
        );
    }
    println!();
}

/// Leg E — attribution: is the golden-window shift the gene, and is the
/// midpoint path *exactly* the pre-i392 engine?
///
/// §4.13 demands the manipulation be verified before the response is re-pinned.
/// The cleanest possible check is available here: the golden scenario itself.
/// Running `riverford_minor` (seed 42, 1 000 ticks, 12 agents, 16×16) with every
/// gene pinned to 0.5 must reproduce the **stored golden hash byte for byte** —
/// that is what "midpoint-neutral by construction" means at machine precision,
/// and it simultaneously proves the divergence in the natural-gene world is the
/// gene's spread and not a stray change in the wiring.
///
/// The leg then walks the two worlds in lockstep and reports the **first tick at
/// which any agent's position differs**, with those agents' genes — the
/// mechanism, not just the hash.
fn leg_e() {
    const GOLDEN_METRIC_HASH: u64 = 12_871_778_371_033_085_037;
    const TICKS: u64 = 1_000;

    fn scenario(n: u32, seed: u64) -> Simulation {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: TICKS,
            world_width: 16,
            world_height: 16,
            num_agents: n,
            snapshot_interval: None,
        });
        sim.populate();
        sim
    }
    fn metric_hash(sim: &Simulation) -> u64 {
        let ms = sim.metrics_snapshot();
        let metrics = vec![
            ms.avg_hunger,
            ms.avg_thirst,
            ms.avg_fatigue,
            ms.avg_valence,
            ms.avg_joy,
            ms.avg_fear,
            ms.total_grain,
            ms.total_water,
            ms.event_count as f64,
            ms.journal_len as f64,
            ms.agent_count as f64,
        ];
        let mut hasher = DefaultHasher::new();
        for m in &metrics {
            m.to_bits().hash(&mut hasher);
        }
        hasher.finish()
    }
    fn positions(sim: &Simulation) -> Vec<(i32, i32)> {
        sim.agents
            .iter()
            .map(|a| (a.position.x, a.position.y))
            .collect()
    }

    let mut control = scenario(12, 42);
    for a in &mut control.agents {
        a.embodied.genome.trait_predispositions.novelty_seeking = Fixed::from_f64(0.5);
    }
    let mut natural = scenario(12, 42);

    let mut first_divergence: Option<(u64, Vec<usize>)> = None;
    for tick in 1..=TICKS {
        control.run(1);
        natural.run(1);
        if first_divergence.is_none() {
            let (a, b) = (positions(&control), positions(&natural));
            let diff: Vec<usize> = a
                .iter()
                .zip(b.iter())
                .enumerate()
                .filter(|(_, (x, y))| x != y)
                .map(|(i, _)| i)
                .collect();
            if !diff.is_empty() {
                first_divergence = Some((tick, diff));
            }
        }
    }
    let control_hash = metric_hash(&control);
    let natural_hash = metric_hash(&natural);
    println!("══ E — attribution: the golden window (riverford_minor, seed 42, 1 000 ticks) ══");
    println!("control (gene pinned 0.5) metric_hash {control_hash} (0x{control_hash:016x})");
    println!(
        "stored golden              metric_hash {GOLDEN_METRIC_HASH} (0x{GOLDEN_METRIC_HASH:016x})  → {} ",
        if control_hash == GOLDEN_METRIC_HASH {
            "EXACT — the midpoint path IS the pre-i392 engine, byte for byte"
        } else {
            "MISMATCH — the wiring is not identity at the midpoint"
        }
    );
    println!("natural genes              metric_hash {natural_hash} (0x{natural_hash:016x})");
    match first_divergence {
        Some((tick, diff)) => {
            println!("first position divergence: tick {tick}, agents {diff:?}");
            for &i in &diff {
                let gene = natural.agents[i]
                    .embodied
                    .genome
                    .trait_predispositions
                    .novelty_seeking
                    .to_f64();
                println!(
                    "  agent {i}: gene {gene:.4} → multiplier {:.4} (control sits at 1.0)",
                    novelty_driver_scale(Fixed::from_f64(gene)).to_f64()
                );
            }
        }
        None => println!("no position divergence in the window"),
    }
    println!();
}

fn main() {
    println!(
        "i392 row 2 — novelty_seeking → the i351 exploration driver (warmup {WARMUP}, window {WINDOW})\n"
    );
    leg_a(12, 42);
    leg_a(48, 42);
    leg_b(12, 42);
    leg_b(48, 7);
    leg_d(12, 42);
    census_of("C — in-vivo, natural genes (N=12, seed 42)", calm(12, 42));
    census_of("C — in-vivo, natural genes (N=48, seed 42)", calm(48, 42));
    leg_e();
}
