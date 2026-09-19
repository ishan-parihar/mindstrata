//! i315 — Difficulty-lever row 3 probe: the pathology CEILING band.
//!
//! i304 promoted row 3's growth/decay axes and explicitly deferred the ceiling
//! axis ("a per-quadrant ceiling band is a separate hypothesis"). This probe
//! tests that hypothesis IN ISOLATION: growth and decay are held at canon
//! (1.0×) and only `pathology_ceiling_scale` varies, so any measured difference
//! is attributable to the ceiling alone.
//!
//! The ceiling governs because growth is `headroom = ceiling − intensity`
//! (`mindstrata-development/src/dynamics.rs`), so it sets each quadrant's
//! equilibrium. It is *live* only where intensity approaches the cap — i304
//! measured Q2 (dark-allergy) 0.63 standard / 0.73 brittle against ceiling 0.80
//! and Q4 0.50/0.63 against 0.75, while Q1 0.34/0.48 and Q3 0.11/0.19 sit far
//! below theirs. The probe reports every quadrant, so a ceiling that only binds
//! on Q2/Q4 is visible rather than smoothed.
//!
//! Run: cargo run --release -p mindstrata-benches --example i315_pathology_ceiling

use mindstrata_core::parameters::{DifficultyProfile, SimParameters};
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_sim::systems::development::{pathology_params, PROD_QUADRANT_PARAMS};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 20_000;
const MAJORITY: usize = 8;

fn config(seed: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: TICKS,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

#[derive(Clone, Copy, Default)]
struct SeedState {
    q1: f64,
    q2: f64,
    q3: f64,
    q4: f64,
    alive: usize,
}

/// Run one seed with ONLY the ceiling scale changed (growth/decay = canon).
fn seed_state(ceiling_scale: f64, seed: u64) -> SeedState {
    let mut params = SimParameters::default();
    params.pathology_ceiling_scale = mindstrata_core::fixed::Fixed::from_f64(ceiling_scale);
    let mut sim = Simulation::new(config(seed));
    sim.params = params;
    sim.populate();
    sim.run(TICKS);
    let n = sim.agents.len();
    let inv = if n > 0 { 1.0 / n as f64 } else { 0.0 };
    let mut st = SeedState {
        alive: n,
        ..SeedState::default()
    };
    for a in &sim.agents {
        let p = &a.development.pathology;
        st.q1 += p.dark_addiction.intensity;
        st.q2 += p.dark_allergy.intensity;
        st.q3 += p.golden_addiction.intensity;
        st.q4 += p.golden_allergy.intensity;
    }
    st.q1 *= inv;
    st.q2 *= inv;
    st.q3 *= inv;
    st.q4 *= inv;
    st
}

fn measure(ceiling_scale: f64) -> Vec<SeedState> {
    SEEDS
        .iter()
        .map(|&s| seed_state(ceiling_scale, s))
        .collect()
}

fn mean(v: &[SeedState], f: fn(&SeedState) -> f64) -> f64 {
    if v.is_empty() {
        return 0.0;
    }
    v.iter().map(f).sum::<f64>() / v.len() as f64
}

fn main() {
    println!("i315 row-3 CEILING band (growth/decay held at canon), {TICKS} ticks/seed");

    // ── Surface: resolved ceilings per band ──────────────────────────────
    let lenient_p = SimParameters::with_difficulty(DifficultyProfile::Lenient);
    let standard_p = SimParameters::with_difficulty(DifficultyProfile::Standard);
    let harsh_p = SimParameters::with_difficulty(DifficultyProfile::Harsh);
    println!("\n[leg 1] resolved ceilings (Q1/Q2/Q3/Q4) per band");
    for (name, p) in [
        ("resilient", &lenient_p),
        ("standard", &standard_p),
        ("brittle", &harsh_p),
    ] {
        let q = pathology_params(p);
        let cells: Vec<String> = q.iter().map(|x| format!("{:.4}", x.ceiling)).collect();
        println!("  {name:<10} {}", cells.join("  "));
    }
    // Standard identity: ceilings bit-for-bit to the canon consts.
    let std_q = pathology_params(&standard_p);
    let identity = std_q
        .iter()
        .zip(PROD_QUADRANT_PARAMS.iter())
        .all(|(a, b)| a.ceiling == b.ceiling && a.growth == b.growth && a.decay == b.decay);
    println!("  standard ceilings == canon consts (bit-for-bit): {identity}");

    // ── Isolation differential ───────────────────────────────────────────
    println!("\n[leg 2] ceiling-only family differential (12 seeds)");
    let low = measure(0.85);
    let canon = measure(1.0);
    let high = measure(1.15);
    println!(
        "{:<10} {:>9} {:>9} {:>9} {:>9} {:>8}",
        "ceiling", "Q1", "Q2", "Q3", "Q4", "alive"
    );
    for (name, v) in [("0.85x", &low), ("1.00x", &canon), ("1.15x", &high)] {
        println!(
            "{name:<10} {:>9.4} {:>9.4} {:>9.4} {:>9.4} {:>4}/12",
            mean(v, |s| s.q1),
            mean(v, |s| s.q2),
            mean(v, |s| s.q3),
            mean(v, |s| s.q4),
            v.iter().filter(|s| s.alive > 0).count()
        );
    }

    // Per-seed direction on the quadrant the ceiling actually binds.
    let dir = |higher: &[SeedState], lower: &[SeedState], f: fn(&SeedState) -> f64| -> usize {
        higher
            .iter()
            .zip(lower.iter())
            .filter(|(h, l)| f(h) > f(l))
            .count()
    };
    let q2_high = dir(&high, &canon, |s| s.q2);
    let q2_low = dir(&canon, &low, |s| s.q2);
    let q4_high = dir(&high, &canon, |s| s.q4);
    let q4_low = dir(&canon, &low, |s| s.q4);
    println!("\n  per-seed direction (of {})", SEEDS.len());
    println!("    Q2 high>canon {q2_high} | canon>low {q2_low}");
    println!("    Q4 high>canon {q4_high} | canon>low {q4_low}");
    println!("  per-seed Q2 (0.85x | 1.00x | 1.15x):");
    for i in 0..SEEDS.len() {
        println!(
            "    seed {:>6}: {:.4} | {:.4} | {:.4}",
            SEEDS[i], low[i].q2, canon[i].q2, high[i].q2
        );
    }

    // ── Verdict ──────────────────────────────────────────────────────────
    let alive = low.iter().all(|s| s.alive > 0)
        && canon.iter().all(|s| s.alive > 0)
        && high.iter().all(|s| s.alive > 0);
    let binding = q2_high >= MAJORITY && q2_low >= MAJORITY;
    println!("\nleg1_standard_identity={identity}");
    println!("leg2_family_stable={alive}");
    println!("leg2_ceiling_binds_q2={binding} (high>canon {q2_high}, canon>low {q2_low})");
    if identity && alive && binding {
        println!("verdict=PATHOLOGY_CEILING_BAND_LIVE");
    } else {
        println!("verdict=PATHOLOGY_CEILING_BAND_PARTIAL");
    }
}
