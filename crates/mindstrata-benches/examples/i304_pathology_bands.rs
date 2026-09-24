//! i304 — Difficulty-lever row 3 probe: pathology growth / decay / ceiling.
//!
//! Promotes `docs/balance/difficulty-levers.md` row 3 ("Pathology
//! growth/ceiling — `PATHOLOGY_GROWTH_*` / `DECAY` / `CEILING` per 4
//! quadrants", candidate bands Resilient `0.5×` growth `1.2×` decay /
//! Standard `1.0×` / Brittle `1.8×` growth `0.7×` decay) from a DRAFT
//! catalog row to a measured runtime surface.
//!
//! Three legs, matching the promotion rule in `difficulty-levers.md`:
//!
//!   1. CONFIGURATION SURFACE: does the band profile reach the pathology
//!      operator at all? Measured as the number of pathology-carrying keys
//!      of `SimParameters` that differ between bands (0 before this
//!      iteration: the pass read `PROD_QUADRANT_PARAMS` consts and took no
//!      param input) plus the resolved per-quadrant tuple each band yields.
//!   2. STANDARD ≡ CANON: `with_difficulty(Standard)` must be byte-identical
//!      to today's canon — params serde equality AND same-seed end-state
//!      digest equality over a full run (the zero-blast contract; the row-2
//!      promotion carried no golden re-anchor and row 3 must carry none
//!      either, since pathology feeds every lineage/horizon pin).
//!   3. FAMILY DIFFERENTIAL: 12-seed family × 3 bands × 20K ticks (the
//!      catalog's own horizon for this lever is "100K lineage", and
//!      i269/i288 measured the working dark-addiction band 0.3–0.5 by
//!      5–20K). The catalog's predicted direction is measured, not assumed:
//!      a brittle village (fast growth, slow decay) carries MORE
//!      dark-addiction intensity than canon, a resilient one LESS.
//!
//! VERDICT CONTRACT (§4.4 re-contract, named not silently widened):
//!   LIVE ⇔ the pathology configuration surface is band-sensitive
//!          ∧ Standard ≡ canon (params + digest)
//!          ∧ the dark-addiction family direction is brittle > standard >
//!            resilient with a per-seed majority (≥ 8/12) agreeing
//!          ∧ all 12 seeds alive in all 3 bands.
//!
//! Per-seed majority rather than aggregate-only: pathology is driven by the
//! Threat catalyst diet, which is sparse and bursty per seed (a feud-heavy
//! seed accumulates, a calm one stays near neutral), so an aggregate mean can
//! be carried by two or three loud seeds. Requiring a majority of the family
//! to agree makes the verdict a statement about the mechanism rather than the
//! tail. The full per-quadrant table is printed either way (nothing hidden),
//! and the exact raw band mapping is pinned in `parameters::tests`.
//!
//! Run: cargo run --release -p mindstrata-benches --example i304_pathology_bands

use mindstrata_core::parameters::{DifficultyProfile, SimParameters};
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_sim::systems::development::{pathology_params, PROD_QUADRANT_PARAMS};

/// The i268 seed family (the calibration-audit stability instrument).
const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const TICKS: u64 = 20_000;
/// Per-seed majority threshold for the directional contract.
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

/// One seed's end state, pathology-relevant only.
#[derive(Clone, Copy, Default)]
struct SeedState {
    q1: f64,
    q2: f64,
    q3: f64,
    q4: f64,
    max_q1: f64,
    events: u64,
    alive: usize,
}

fn seed_state(profile: DifficultyProfile, seed: u64) -> SeedState {
    let mut sim = Simulation::new(config(seed));
    sim.params = SimParameters::with_difficulty(profile);
    sim.populate();
    sim.run(TICKS);
    let n = sim.agents.len();
    let inv = if n > 0 { 1.0 / n as f64 } else { 0.0 };
    let mut st = SeedState {
        events: sim.event_count() as u64,
        alive: n,
        ..SeedState::default()
    };
    for a in &sim.agents {
        let p = &a.development.pathology;
        st.q1 += p.dark_addiction.intensity;
        st.q2 += p.dark_allergy.intensity;
        st.q3 += p.golden_addiction.intensity;
        st.q4 += p.golden_allergy.intensity;
        st.max_q1 = st.max_q1.max(p.dark_addiction.intensity);
    }
    st.q1 *= inv;
    st.q2 *= inv;
    st.q3 *= inv;
    st.q4 *= inv;
    st
}

/// Per-band family measurement: the mean of each per-seed mean (and the
/// per-seed vector, so the direction test can be per-seed rather than
/// aggregate).
struct BandStats {
    states: Vec<SeedState>,
}

impl BandStats {
    fn mean(&self, f: fn(&SeedState) -> f64) -> f64 {
        if self.states.is_empty() {
            return 0.0;
        }
        self.states.iter().map(f).sum::<f64>() / self.states.len() as f64
    }
    fn alive_seeds(&self) -> usize {
        self.states.iter().filter(|s| s.alive > 0).count()
    }
}

fn measure(profile: DifficultyProfile) -> BandStats {
    BandStats {
        states: SEEDS.iter().map(|&s| seed_state(profile, s)).collect(),
    }
}

/// Deterministic end-state digest over the four pathology quadrants + one
/// need channel + the event count. `PathologyField` is f64, so intensities
/// travel as bit patterns — any divergence shows, including the last-bit
/// ones a `Fixed`-rounded digest would hide.
fn state_digest(sim: &Simulation) -> String {
    let rows: Vec<(u64, u64, u64, u64, i64)> = sim
        .agents
        .iter()
        .map(|a| {
            (
                a.development.pathology.dark_addiction.intensity.to_bits(),
                a.development.pathology.dark_allergy.intensity.to_bits(),
                a.development.pathology.golden_addiction.intensity.to_bits(),
                a.development.pathology.golden_allergy.intensity.to_bits(),
                a.needs.meaning.to_raw(),
            )
        })
        .collect();
    format!("{}|{rows:?}", sim.event_count())
}

/// Pathology-carrying keys of `SimParameters` that differ between two bands:
/// the direct measure of whether the band profile reaches this lever at all.
fn pathology_keys_differing(a: &SimParameters, b: &SimParameters) -> usize {
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
        .filter(|((ka, _), (kb, _))| ka == kb && ka.contains("pathology"))
        .filter(|((_, va), (_, vb))| va != vb)
        .count()
}

fn main() {
    println!(
        "i304 difficulty-lever row 3 — pathology growth/decay bands, {TICKS} ticks/seed, family of {}",
        SEEDS.len()
    );

    // ── Leg 1: configuration surface ─────────────────────────────────────
    println!("\n[leg 1] pathology configuration surface");
    let canon = PROD_QUADRANT_PARAMS;
    let (c1, c2, c3, c4) = (canon[0], canon[1], canon[2], canon[3]);
    println!("  module consts the pass reads today (PROD_QUADRANT_PARAMS):");
    for (name, q) in [
        ("Q1 dark-add", c1),
        ("Q2 dark-all", c2),
        ("Q3 golden-add", c3),
        ("Q4 golden-all", c4),
    ] {
        println!(
            "    {name:<14} growth={:.4} decay={:.4} ceiling={:.4}",
            q.growth, q.decay, q.ceiling
        );
    }
    let lenient_p = SimParameters::with_difficulty(DifficultyProfile::Lenient);
    let standard_p = SimParameters::with_difficulty(DifficultyProfile::Standard);
    let harsh_p = SimParameters::with_difficulty(DifficultyProfile::Harsh);
    let keys_ls = pathology_keys_differing(&lenient_p, &standard_p);
    let keys_sh = pathology_keys_differing(&standard_p, &harsh_p);
    println!("  SimParameters pathology-band keys (lenient vs standard): {keys_ls}");
    println!("  SimParameters pathology-band keys (standard vs harsh):   {keys_sh}");

    // The resolved operator params each band actually feeds the field engine.
    let resolved: [(
        DifficultyProfile,
        [mindstrata_development::dynamics::OperatorParams; 4],
    ); 3] = [
        (DifficultyProfile::Lenient, pathology_params(&lenient_p)),
        (DifficultyProfile::Standard, pathology_params(&standard_p)),
        (DifficultyProfile::Harsh, pathology_params(&harsh_p)),
    ];
    println!("  resolved per-quadrant params (growth / decay / ceiling):");
    for (name, q) in &resolved {
        let cells: Vec<String> = q
            .iter()
            .map(|p| format!("{:.4}/{:.4}/{:.3}", p.growth, p.decay, p.ceiling))
            .collect();
        println!("    {name:<10} {}", cells.join("  "));
    }
    let distinct = |a: &[mindstrata_development::dynamics::OperatorParams],
                    b: &[mindstrata_development::dynamics::OperatorParams]| {
        a.iter()
            .zip(b.iter())
            .any(|(x, y)| x.growth != y.growth || x.decay != y.decay || x.ceiling != y.ceiling)
    };
    let resolved_distinct = distinct(&resolved[0].1, &resolved[1].1)
        && distinct(&resolved[1].1, &resolved[2].1)
        && distinct(&resolved[0].1, &resolved[2].1);
    println!("  resolved tuples pairwise distinct across bands: {resolved_distinct}");
    let surface_band_sensitive = (keys_ls > 0 || keys_sh > 0) && resolved_distinct;

    // ── Leg 2: Standard ≡ canon identity (zero-blast contract) ───────────
    println!("\n[leg 2] Standard identity vs untouched canon defaults");
    let params_identical = serde_json::to_string(&SimParameters::default()).unwrap()
        == serde_json::to_string(&standard_p).unwrap();
    println!("  params serde identical: {params_identical}");

    let mut canon_sim = Simulation::new(config(42));
    canon_sim.populate();
    canon_sim.run(TICKS);
    let canon_digest = state_digest(&canon_sim);

    let mut standard_sim = Simulation::new(config(42));
    standard_sim.params = SimParameters::with_difficulty(DifficultyProfile::Standard);
    standard_sim.populate();
    standard_sim.run(TICKS);
    let state_identical = canon_digest == state_digest(&standard_sim);
    println!("  end-state digest identical (seed 42, {TICKS} ticks): {state_identical}");

    // ── Leg 3: family differential ───────────────────────────────────────
    println!("\n[leg 3] 12-seed family differential (mean quadrant intensity)");
    let resilient = measure(DifficultyProfile::Lenient);
    let standard = measure(DifficultyProfile::Standard);
    let brittle = measure(DifficultyProfile::Harsh);

    println!(
        "{:<11} {:>7} {:>7} {:>7} {:>7} {:>8} {:>8} {:>7}",
        "band", "Q1", "Q2", "Q3", "Q4", "max_Q1", "events", "alive"
    );
    for (name, s) in [
        ("resilient", &resilient),
        ("standard", &standard),
        ("brittle", &brittle),
    ] {
        println!(
            "{name:<11} {:>7.4} {:>7.4} {:>7.4} {:>7.4} {:>8.4} {:>8.0} {:>4}/12",
            s.mean(|x| x.q1),
            s.mean(|x| x.q2),
            s.mean(|x| x.q3),
            s.mean(|x| x.q4),
            s.mean(|x| x.max_q1),
            s.mean(|x| x.events as f64),
            s.alive_seeds()
        );
    }

    // Per-seed directional test on the catalog's named observable (Q1).
    let mut brittle_gt_standard = 0;
    let mut standard_gt_resilient = 0;
    let mut brittle_gt_resilient = 0;
    for i in 0..SEEDS.len() {
        let (r, s, b) = (resilient.states[i], standard.states[i], brittle.states[i]);
        if b.q1 > s.q1 {
            brittle_gt_standard += 1;
        }
        if s.q1 > r.q1 {
            standard_gt_resilient += 1;
        }
        if b.q1 > r.q1 {
            brittle_gt_resilient += 1;
        }
    }
    println!("\n  per-seed Q1 direction (of {}):", SEEDS.len());
    println!("    brittle > standard:      {brittle_gt_standard}");
    println!("    standard > resilient:    {standard_gt_resilient}");
    println!("    brittle > resilient:     {brittle_gt_resilient}");
    println!("  per-seed Q1 (resilient | standard | brittle):");
    for (i, seed) in SEEDS.iter().enumerate() {
        println!(
            "    seed {:>6}: {:.4} | {:.4} | {:.4}",
            seed, resilient.states[i].q1, standard.states[i].q1, brittle.states[i].q1
        );
    }

    // ── Verdict (pre-registered — see the module doc) ────────────────────
    let identity = params_identical && state_identical;
    let family_stable = resilient.alive_seeds() == SEEDS.len()
        && standard.alive_seeds() == SEEDS.len()
        && brittle.alive_seeds() == SEEDS.len();
    let direction = brittle_gt_standard >= MAJORITY
        && standard_gt_resilient >= MAJORITY
        && brittle_gt_resilient >= MAJORITY;

    println!("\nleg1_surface_band_sensitive={surface_band_sensitive}");
    println!("leg2_standard_identity={identity}");
    println!(
        "leg3_family_stable={family_stable} (alive {}/{}/{})",
        resilient.alive_seeds(),
        standard.alive_seeds(),
        brittle.alive_seeds()
    );
    println!("leg3_q1_direction_majority={direction}");
    if surface_band_sensitive && identity && family_stable && direction {
        println!("verdict=PATHOLOGY_BANDS_LIVE");
    } else {
        println!("verdict=PATHOLOGY_BANDS_PARTIAL (see failed leg above)");
    }
}
