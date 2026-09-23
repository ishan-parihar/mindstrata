//! i392 row 3 — `chronic_pain_risk` against its consumer.
//!
//! The i391 census found the gene drawn U(0.0, 0.5), defaulted (0.2), blended at
//! `inherit`, and read by nothing outside `genome.rs`. Its consumer is the
//! skeletal accumulation law (`SkeletalState::tick_update`), whose state feeds
//! real places: `mobility_penalty` (×0.3), `health_factor` (×0.2), and
//! psychopathology (×0.15/×0.1).
//!
//! **The law the gene would scale:**
//!
//! ```rust
//! self.chronic_pain = (self.chronic_pain + self.fracture_risk * Fixed::from_f64(0.005)).clamp_01();
//! ```
//!
//! with a comment claiming that `0.005` *is* `chronic_pain_accumulation_rate`
//! "from organs.ron". **It is not**: `specs/biology/organs.ron:16` declares
//! **0.001**, the identifier appears nowhere in Rust except those two comments,
//! and 0.001 could not work anyway — its product with a moderate `fracture_risk`
//! (0.01 × 0.001 = 1e-5) is below `Fixed`'s 1e-4 resolution and would quantize to
//! zero (§5). So the code is right and the spec is inert, but nothing says so.
//!
//! Four legs, in the order the two gates of §4.15 require — the band first, then
//! the response's proportionality:
//!
//! * **A — manipulate**: the gene's realised distribution in the corpus.
//! * **B — is the band even open?** The state only accumulates above
//!   `injury > 0.5 → fracture_risk > 0`. If calibrated corpora have no such
//!   injury, the wiring is invisible to them (the row-1 pattern); if they do,
//!   the blast radius is real and has to be measured before touching anything.
//! * **C — the response's shape**: the law's increment is **quantized**
//!   (1e-4), so a proportional gene multiplier can silently become a threshold
//!   at low fracture risk. This leg prints, in raw Fixed units, what each
//!   carrier's increment actually is across the risk range — the §5 hazard check.
//! * **D — the manipulation check**: accumulate the state in a direct harness and
//!   compare gene extremes, which is what the wiring's pin will assert.
//!
//! Run: `cargo run --release -p mindstrata-benches --example i393_chronic_pain_gene`

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

/// Candidate law: the multiplier on the accumulation rate, anchored at the
/// **draw's midpoint (0.25)**, so the population-mean multiplier is 1.0 by
/// construction (§4.6) and gene 0.25 reproduces today's hardcoded 0.005 exactly.
///
/// The anchor is the draw midpoint and **not** the gene's `Default` (0.2), even
/// though `reproductive.rs`'s row-1 precedent anchored on the constant: the draw
/// is U(0.0, 0.5) with mean 0.25, so anchoring at 0.2 would leave a **systematic
/// +8%** population drift (1 + 1.6 × 0.05) for every future injury world. Row 1
/// had no such asymmetry — its draw was centred on the constant. Every agent
/// carries a drawn gene here, so population neutrality is the property that
/// matters; leg A measures the realised mean.
fn scale(gene: f64) -> f64 {
    (1.0 + (gene - 0.25) * CHRONIC_PAIN_GENE_SPAN).max(0.0)
}

/// How far the gene swings the rate. Sized to the response's *shape*: chronic
/// pain is a **linear accumulator** (no threshold, no gate), so unlike the
/// elastic Wander coefficient (i392 row 2, elasticity ≈4) a ±40% rate change
/// buys a ±40% state change. That linearity is the §4.15 gate-2 pass.
const CHRONIC_PAIN_GENE_SPAN: f64 = 1.6;

const RATE: f64 = 0.005;

fn village(n: u32, seed: u64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

/// Leg A — the manipulation: what the gene actually varies.
fn leg_a() {
    println!("══ A — the gene's realised distribution ══");
    for (label, n, seed) in [("village", 12u32, 42u64), ("town", 48, 42)] {
        let sim = village(n, seed, 0);
        let mut genes: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| {
                a.embodied
                    .genome
                    .health_predispositions
                    .chronic_pain_risk
                    .to_f64()
            })
            .collect();
        genes.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
        let mean = genes.iter().sum::<f64>() / genes.len().max(1) as f64;
        let scales: Vec<f64> = genes.iter().map(|g| scale(*g)).collect();
        let mean_scale = scales.iter().sum::<f64>() / scales.len().max(1) as f64;
        println!(
            "{label:<8} n={n:<3} gene {:.4}–{:.4} mean {:.4} (draw U(0.0,0.5), default 0.2) · \
             multiplier {:.4}–{:.4} mean {:.4}",
            genes.first().copied().unwrap_or(0.0),
            genes.last().copied().unwrap_or(0.0),
            mean,
            scales.first().copied().unwrap_or(1.0),
            scales.last().copied().unwrap_or(1.0),
            mean_scale
        );
    }
    println!(
        "reference points: gene 0.0 → ×{:.3} · 0.2 (Default) → ×{:.3} · 0.25 (draw mid, neutral) → ×{:.3} · 0.5 → ×{:.3}",
        scale(0.0),
        scale(0.2),
        scale(0.25),
        scale(0.5)
    );
    println!();
}

/// Leg B — is the band open in the corpora the goldens and pins use?
fn leg_b() {
    println!("══ B — is the band open? chronic-pain state occupancy ══");
    println!("corpus                agent-ticks  ever fr>0  ever cp>0  max chronic_pain  max fracture_risk");
    // Sampled **every tick over the whole run**, not at the end: a single final
    // snapshot would miss an injury spike that decayed (fracture_risk −0.0005/tick,
    // chronic_pain −0.0005/tick), and this is the measurement the wiring's blast
    // radius rests on.
    let check = |label: &str, mut sim: Simulation, ticks: u64| {
        let mut max_cp = 0.0f64;
        let mut max_fr = 0.0f64;
        let mut any_cp = false;
        let mut any_fr = false;
        let mut agent_ticks = 0u64;
        for _ in 0..ticks {
            sim.run(1);
            for a in &sim.agents {
                agent_ticks += 1;
                let f = a.embodied.skeletal.fracture_risk.to_f64();
                let c = a.embodied.skeletal.chronic_pain.to_f64();
                if f > 0.0 {
                    any_fr = true;
                }
                if c > 0.0 {
                    any_cp = true;
                }
                max_cp = max_cp.max(c);
                max_fr = max_fr.max(f);
            }
        }
        println!(
            "{label:<20}  {:>11}  {:>10}  {:>10}  {:>15.4}  {:>16.4}",
            agent_ticks,
            if any_fr { "yes" } else { "no" },
            if any_cp { "yes" } else { "no" },
            max_cp,
            max_fr
        );
    };
    check("village 12 s42 @20K", village(12, 42, 20_000), 20_000);
    check("town 48 s42 @20K", village(48, 42, 20_000), 20_000);
    check("town 48 s7 @20K", village(48, 7, 20_000), 20_000);
    // The crisis scenarios are where severe injury actually happens.
    for (label, seed) in [("pestilence s42", 42u64), ("collapse s42", 42)] {
        let mut sc = if label.starts_with("pest") {
            Scenario::pestilence()
        } else {
            Scenario::collapse()
        };
        sc.seed = seed;
        let ticks = sc.ticks;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        check(label, sim, ticks);
    }
    println!();
    println!(
        "(fracture_risk only rises while injury > 0.5, so `fracture_risk>0` counts agents who \
         have taken a severe injury at least once — the band the gene would govern)"
    );
    println!();
}

/// Leg C — the response's shape under Fixed's 1e-4 quantization.
fn leg_c() {
    println!("══ C — §5 hazard check: does the gene-scaled increment survive quantization? ══");
    println!("fracture_risk   rate      raw increment   ×10⁴    stored?");
    for fr in [0.01f64, 0.02, 0.05, 0.1, 0.3] {
        for gene in [0.0f64, 0.2, 0.5] {
            let rate = RATE * scale(gene);
            let increment = Fixed::from_f64(fr) * Fixed::from_f64(rate);
            println!(
                "{fr:>13.2}   {rate:.5}   {increment:>13}   {:>4}    {}   (gene {gene})",
                (increment.to_f64() * 10_000.0) as i64,
                if increment > Fixed::ZERO {
                    "kept"
                } else {
                    "**LOST**"
                }
            );
        }
    }
    println!();
    println!(
        "reading: below fracture_risk 0.02 every carrier loses the increment — that is a \
         property of the *state's stored resolution*, not of the gene (it is already true \
         today at 0.005), so the gene differentiates carriers in the regime where the state \
         can move at all"
    );
    println!();
}

/// Leg D — the manipulation check: same injury, different gene.
fn leg_d() {
    println!("══ D — manipulation check: the accumulation law at a fixed fracture risk ══");
    println!("gene   multiplier   chronic_pain after N steps   Δ vs neutral");
    // A **fixed** fracture risk isolates the gene: the first draft drove injury
    // through `tick_update`, which pushes risk to its 1.0 ceiling and clamps
    // every carrier to chronic_pain 1.0 — a saturated harness measures nothing,
    // and reading the saturation as "no separation" would have been the §4.13
    // trap. Holding risk at 0.2 and letting the accumulator run 500 steps is
    // inside the range the state can actually move through.
    //
    // This leg tests the **candidate law**, not the tree (the tree still hardcodes
    // 0.005, so driving `tick_update` today would show zero separation by
    // construction). The wiring's unit test is what asserts the same separation
    // through the real `tick_update` path.
    const RISK: f64 = 0.2;
    const STEPS: u32 = 500;
    let mut neutral_value = 0.0;
    for gene in [0.0f64, 0.2, 0.25, 0.5] {
        let rate = RATE * scale(gene);
        let mut value = 0.0f64;
        for _ in 0..STEPS {
            value = (value + RISK * rate).min(1.0);
        }
        value = Fixed::from_f64(value).to_f64(); // the state's own 1e-4 resolution
        if (gene - 0.25).abs() < 1e-9 {
            neutral_value = value;
        }
        println!(
            "{gene:>4.2}   ×{:.3}      {:>10.4}               {:>+9.4}",
            scale(gene),
            value,
            value - neutral_value
        );
    }
    println!(
        "   (Δ measured against the neutral gene 0.25 ≡ today's constant; risk held at {RISK} \
         for {STEPS} steps — the regime where `tick_update` moves the state)"
    );
    println!();
}

fn main() {
    println!("i392 row 3 — chronic_pain_risk → the skeletal accumulation law\n");
    leg_a();
    leg_b();
    leg_c();
    leg_d();
}
