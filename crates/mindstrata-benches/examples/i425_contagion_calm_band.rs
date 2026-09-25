//! i425 — calm-world occupancy of the fear-contagion sub-quantum band.
//!
//! The i403 arc exposed a §5 Fixed-4 truncation hazard inside the daily
//! fear-contagion fold (`relational_field.rs::contagion_delta`):
//! `Fixed::mul` floors at SCALE=10_000, so with `FEAR_CONTAGION_RATE = 0.05`
//! every product from `perceived_stress < 0.002` (one output quantum 1e-4
//! ÷ rate) truncated to exactly zero — the whole low-stress range was silent.
//! The f64 fix quantizes once; its repaired band is `old == 0 && new > 0`,
//! which arithmetic puts at `[0.001, 0.002)` (the f64 product must also
//! clear `from_f64`'s 5e-5 rounding threshold) — this probe measures that
//! band EMPIRICALLY, never by the edge constant.
//!
//! Corpus: the riverford_minor golden world (seed 42, 1000 ticks, 16×16,
//! N=12) — the corpus whose golden the fix moves — plus calm seeds 7 and 11
//! so the occupancy is not a one-seed artefact. Samples every agent's
//! `perceived_stress` at each daily fold boundary (tick % 144 == 0, the
//! `TickPhases::is_daily` cadence the decay pass runs on), computes both
//! deltas per agent-day, and accumulates the first-order fear contribution
//! difference per agent.
//!
//! Attribution honesty (§4.13): the accumulated difference is FIRST-ORDER —
//! fear feeds back into stress, so the exact end-state trajectory difference
//! is what the golden hashes and snapshot diffs measure, not this sum. This
//! probe characterizes the MECHANISM (band occupancy + magnitude) that makes
//! the custody re-baseline expected, per §4.2.
//!
//! Run: cargo run --release -p mindstrata-benches --example i425_contagion_calm_band
use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::{SimConfig, Simulation};

const RATE: f64 = mindstrata_sim::social::relational_field::FEAR_CONTAGION_RATE;

/// The pre-fix code path verbatim: `Fixed::mul` truncation + clamp.
fn old_delta(perceived_stress: Fixed) -> Fixed {
    (perceived_stress * Fixed::from_f64(RATE)).clamp_01()
}

/// The post-fix code path verbatim: f64 product, quantize once, clamp.
fn new_delta(perceived_stress: Fixed) -> Fixed {
    Fixed::from_f64(perceived_stress.to_f64() * RATE).clamp_01()
}

fn main() {
    // Sanity header: the band on synthetic values, so the empirical
    // definition is seen to discriminate (old==0 && new>0) before any
    // simulation runs.
    println!("i425 contagion sub-quantum band (rate = {RATE}):");
    for probe in [0.0f64, 0.0005, 0.001, 0.0015, 0.0019, 0.002, 0.01] {
        let s = Fixed::from_f64(probe);
        let (o, n) = (old_delta(s), new_delta(s));
        println!(
            "  stress {probe:.4}: old {o:.4} new {n:.4} {}",
            if o == Fixed::ZERO && n > Fixed::ZERO {
                "<- BAND"
            } else {
                ""
            }
        );
    }

    println!("\nper-day occupancy over the calm corpora (1000 ticks, daily folds):");
    let seeds = [42u64, 7, 11];
    for &seed in &seeds {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 1000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        let n_agents = sim.agents.len();

        let mut agent_days = 0u64;
        let mut zero_stress = 0u64; // identity-at-zero rows: unaffected either way
        let mut band_days = 0u64; // old == 0 && new > 0: the repaired sub-quantum band
        let mut round_up_days = 0u64; // old > 0 && new == old + 1 quantum: floor → nearest
        let mut equal_days = 0u64; // old == new > 0
        let mut band_new_sum = 0.0f64; // total NEW contribution inside the band
        let mut band_new_max = 0.0f64;
        let mut per_agent_gain = vec![0.0f64; n_agents]; // first-order Σ(new − old)

        // Sample at every daily fold boundary: state after tick t−1 is the
        // input the fold reads during tick t (t % 144 == 0 && t > 0).
        // Loop runs the full golden horizon (1000 tick() calls).
        for t in 0u64..1000 {
            if t % 144 == 0 && t > 0 {
                for (i, a) in sim.agents.iter().enumerate() {
                    let s = a.relational_fields.perceived_stress;
                    let (o, n) = (old_delta(s), new_delta(s));
                    agent_days += 1;
                    if s == Fixed::ZERO {
                        zero_stress += 1;
                    }
                    if o == Fixed::ZERO && n > Fixed::ZERO {
                        band_days += 1;
                        band_new_sum += n.to_f64();
                        band_new_max = band_new_max.max(n.to_f64());
                    } else if o > Fixed::ZERO && n > o {
                        // from_f64 rounds to nearest; Fixed::mul floors — any
                        // product with residue ≥ 0.5 quantum gains +1 q.
                        round_up_days += 1;
                    } else if o > Fixed::ZERO {
                        equal_days += 1;
                    }
                    per_agent_gain[i] += (n - o).to_f64();
                }
            }
            sim.tick();
        }
        assert_eq!(
            sim.agents.len(),
            n_agents,
            "population must be stable at this scale"
        );

        let (mean_gain, max_gain) = {
            let total: f64 = per_agent_gain.iter().sum();
            (
                total / n_agents as f64,
                per_agent_gain.iter().copied().fold(0.0, f64::max),
            )
        };
        let band_share = band_days as f64 / agent_days as f64 * 100.0;
        let zero_share = zero_stress as f64 / agent_days as f64 * 100.0;
        let band_new_mean = if band_days > 0 {
            band_new_sum / band_days as f64
        } else {
            0.0
        };
        let avg_fear = sim.metrics_snapshot().avg_fear;

        println!(
            "seed {seed:>2}: agent-days {agent_days}, zero {zero_share:.2}%, \
             band {band_days} ({band_share:.2}%), round-up {round_up_days}, \
             equal {equal_days}; band new-delta mean {band_new_mean:.6} \
             max {band_new_max:.6}; first-order fear gain/agent mean {mean_gain:.6} \
             max {max_gain:.6}; end avg_fear {avg_fear:.6}"
        );
    }

    // ── Fold-window A/B legs ───────────────────────────────────────
    // Does the fold's +1-quantum increment PERSIST or wash? The tick
    // pipeline runs the decay fold at pass 7 and the relational-field
    // refresh at the end of the same tick; whether appraisal recompute
    // overwrites fear later is an empirical question. Sample per-tick
    // mean fear around the first fold (tick 144) on riverford, and
    // around the collapse horizon's final tick (4320 = 30×144, where a
    // fold fires inside the golden's last tick).
    println!("\nfold-window, riverford seed 42 (mean fear per tick, t=138..152):");
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 1000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    for t in 0u64..152 {
        if (138..=152).contains(&t) {
            let mean_fear = mean_fear(&sim);
            println!("  t={t:>4} mean_fear {mean_fear:.6}");
        }
        sim.tick();
    }

    use mindstrata_sim::scenario::Scenario;
    let sc = Scenario::collapse();
    let ticks = sc.ticks;
    let window_start = ticks - 12;
    println!(
        "\nfold-window, collapse scenario (mean fear per tick, t={window_start}..{}):",
        ticks - 1
    );
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    // Crisis fold-boundary stress census: MEASURES the crisis product
    // scale the calm/crisis asymmetry claim rests on (min/mean stress at
    // fold boundaries, and the share of fold agent-days whose contagion
    // product is small enough that the ≤1-quantum class could matter,
    // i.e. below 20 quanta = 2% relative).
    let mut fold_min_stress = f64::INFINITY;
    let mut fold_stress_sum = 0.0f64;
    let mut fold_below_20q = 0u64;
    let mut fold_agent_days = 0u64;
    for t in 0u64..ticks {
        if t % 144 == 0 && t > 0 {
            for a in &sim.agents {
                let s = a.relational_fields.perceived_stress.to_f64();
                fold_min_stress = fold_min_stress.min(s);
                fold_stress_sum += s;
                if s * RATE < 20e-4 {
                    fold_below_20q += 1;
                }
                fold_agent_days += 1;
            }
        }
        if t + 12 >= ticks {
            let mean_fear = mean_fear(&sim);
            println!("  t={t:>4} mean_fear {mean_fear:.6}");
        }
        sim.tick();
    }
    let ms = sim.metrics_snapshot();
    println!(
        "collapse end: avg_fear {:.6}, agent_count {}, event_count {}",
        ms.avg_fear, ms.agent_count, ms.event_count
    );
    println!(
        "collapse fold-boundary stress: min {fold_min_stress:.4}, mean {:.4}, \
         agent-days {fold_agent_days}, products < 20 quanta: {fold_below_20q}",
        fold_stress_sum / fold_agent_days as f64
    );
}

fn mean_fear(sim: &Simulation) -> f64 {
    let n = sim.agents.len();
    let total: Fixed = sim
        .agents
        .iter()
        .fold(Fixed::ZERO, |acc, a| acc + a.emotions.fear);
    (total / Fixed::from_int(n as i64)).to_f64()
}
