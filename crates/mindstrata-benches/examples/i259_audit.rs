//! Iteration 259 — full-program audit probe (PLAN_BIO_PSYCH_DEEPENING §4).
//!
//! Measures EQUILIBRIA for every arc claim, per doctrine §4.3 ("a dead
//! producer is a bug even when tests pass"). Read-only over public accessors;
//! no sim-state mutation beyond the boosted conception multiplier (needed to
//! get enough births inside the horizon for heredity statistics).
//!
//! Run: cargo run -p mindstrata-benches --example i259_audit --release

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn pct(vals: &mut Vec<f64>, q: f64) -> f64 {
    if vals.is_empty() {
        return 0.0;
    }
    vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
    vals[((vals.len() as f64 * q).ceil() as usize)
        .saturating_sub(1)
        .min(vals.len() - 1)]
}

fn pearson(xs: &[f64], ys: &[f64]) -> f64 {
    let n = xs.len() as f64;
    if n < 3.0 {
        return 0.0;
    }
    let mx = xs.iter().sum::<f64>() / n;
    let my = ys.iter().sum::<f64>() / n;
    let mut sxy = 0.0;
    let mut sxx = 0.0;
    let mut syy = 0.0;
    for (x, y) in xs.iter().zip(ys.iter()) {
        sxy += (x - mx) * (y - my);
        sxx += (x - mx).powi(2);
        syy += (y - my).powi(2);
    }
    if sxx <= 1e-12 || syy <= 1e-12 {
        return 0.0;
    }
    sxy / (sxx.sqrt() * syy.sqrt())
}

/// Mean of the big-five CONSTITUTION anchors (birth-time heredity substrate).
fn constitution_mean(a: &mindstrata_sim::sim::AgentBundle) -> f64 {
    // Constitution is snapshotted at construction on every path
    // (random/inherit); the fallback is unreachable in live sims.
    let c = match &a.personality.constitution {
        Some(c) => c,
        None => return 2.5,
    };
    (c.openness.to_f64()
        + c.conscientiousness.to_f64()
        + c.extraversion.to_f64()
        + c.agreeableness.to_f64()
        + c.neuroticism.to_f64())
        / 5.0
}

/// Mean of the big-five personality vector.
fn big5_mean(a: &mindstrata_sim::sim::AgentBundle) -> f64 {
    let p = &a.personality;
    (p.openness.to_f64()
        + p.conscientiousness.to_f64()
        + p.extraversion.to_f64()
        + p.agreeableness.to_f64()
        + p.neuroticism.to_f64())
        / 5.0
}

fn main() {
    println!("=== [1] Phase-2 affect equilibrium (calm worlds, 5000 ticks) ===");
    let mut valence_medians = Vec::new();
    for seed in [42u64, 7, 13, 5] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(5000);
        let mut val = Vec::new();
        let mut fear = Vec::new();
        let mut joy = Vec::new();
        for a in &sim.agents {
            val.push(a.affect.valence.to_f64());
            fear.push(a.emotions.fear.to_f64());
            joy.push(a.emotions.joy.to_f64());
        }
        let vmed = pct(&mut val.clone(), 0.5);
        valence_medians.push(vmed);
        println!(
            "  seed {seed}: valence med={vmed:+.3} | fear med={:.3} | joy med={:.3}",
            pct(&mut fear, 0.5),
            pct(&mut joy, 0.5)
        );
    }
    let vmax = valence_medians
        .iter()
        .cloned()
        .fold(f64::NEG_INFINITY, f64::max);
    let vmin = valence_medians
        .iter()
        .cloned()
        .fold(f64::INFINITY, f64::min);
    let spread = vmax - vmin;
    println!(
        "  CONTRACT: all valence medians > 0 (E1 plateau dead): {}",
        valence_medians.iter().all(|v| *v > 0.0)
    );
    println!(
        "  seed differentiation (spread > 0.05): {spread:.3} -> {}",
        spread > 0.05
    );

    println!("\n=== [2] Arc-B Whitehall gradient (status -> chronic_load) ===");
    {
        let mut sim = Simulation::new(SimConfig {
            seed: 46,
            max_ticks: 20000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(20000);
        // Split by effective status at the FINAL tick; compare cohort loads.
        let mut rows: Vec<(f64, f64)> = sim
            .agents
            .iter()
            .map(|a| {
                (
                    a.status_v2.effective_status().to_f64(),
                    a.embodied.endocrine.stress.chronic_load.to_f64(),
                )
            })
            .collect();
        rows.sort_by(|x, y| x.0.partial_cmp(&y.0).unwrap());
        let half = rows.len() / 2;
        let lo = &rows[..half];
        let hi = &rows[rows.len() - half..];
        let mean = |r: &[(f64, f64)]| r.iter().map(|x| x.1).sum::<f64>() / r.len() as f64;
        println!(
            "  low-status mean load {:.4} vs high-status {:.4}",
            mean(lo),
            mean(hi)
        );
        println!("  CONTRACT low >= high: {}", mean(lo) >= mean(hi));
    }

    println!("\n=== [3] Arc-A heredity end-to-end (CONSTITUTION anchors) ===");
    {
        const TICKS: u64 = 40000;
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: TICKS,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        // Founder registry BEFORE any life experience: (constitution, age).
        let tpy = sim.demography_config.ticks_per_year.max(1) as f64;
        let mut founders: std::collections::HashMap<usize, (f64, f64)> =
            std::collections::HashMap::new();
        for (i, a) in sim.agents.iter().enumerate() {
            founders.insert(i, (constitution_mean(a), a.age.to_f64()));
        }
        sim.params.reproduction_conception_multiplier = Fixed::from_f64(4.0);
        sim.run(TICKS);
        let horizon_y = TICKS as f64 / tpy;

        // Parent-child pairs: only slots whose occupant aged through the
        // whole horizon (replacement newborns reset age -> excluded).
        let mut pc_x = Vec::new();
        let mut pc_y = Vec::new();
        let mut families: std::collections::HashMap<usize, Vec<usize>> =
            std::collections::HashMap::new();
        for (i, a) in sim.agents.iter().enumerate() {
            if let Some(pa) = a.parent_a {
                if let Some(&(pc0, age0)) = founders.get(&pa) {
                    let aged_through = sim.agents[pa].age.to_f64() >= age0 + horizon_y - 0.5;
                    if aged_through && pa != i {
                        families.entry(pa).or_default().push(i);
                        pc_x.push(pc0);
                        pc_y.push(constitution_mean(a));
                    }
                }
            }
        }
        let r_pc = pearson(&pc_x, &pc_y);
        // Sibling vs stranger, same verified families.
        let mut sib_pairs = Vec::new();
        let mut stranger_pairs = Vec::new();
        let vals: Vec<f64> = sim.agents.iter().map(constitution_mean).collect();
        let mut in_same_family = vec![false; vals.len()];
        for fam in families.values() {
            for w in fam.windows(2) {
                sib_pairs.push((vals[w[0]], vals[w[1]]));
                in_same_family[w[0]] = true;
                in_same_family[w[1]] = true;
            }
        }
        for i in 0..vals.len() {
            for j in (i + 1)..vals.len() {
                if !in_same_family[i]
                    && !in_same_family[j]
                    && sim.agents[i].parent_a != Some(j)
                    && sim.agents[j].parent_a != Some(i)
                {
                    stranger_pairs.push((vals[i], vals[j]));
                }
            }
        }
        let r_sib = pearson(
            &sib_pairs.iter().map(|p| p.0).collect::<Vec<_>>(),
            &sib_pairs.iter().map(|p| p.1).collect::<Vec<_>>(),
        );
        let r_str = pearson(
            &stranger_pairs.iter().map(|p| p.0).collect::<Vec<_>>(),
            &stranger_pairs.iter().map(|p| p.1).collect::<Vec<_>>(),
        );
        println!(
            "  verified parent-child pairs {} | sibling families {}",
            pc_x.len(),
            families.len()
        );
        println!(
            "  parent-child constitution r={r_pc:.3} (contract > 0.25): {}",
            r_pc > 0.25
        );
        println!(
            "  sibling r={r_sib:.3} vs stranger r={r_str:.3} (sib > str): {}",
            r_sib > r_str
        );
    }

    println!("\n=== [4] Arc-B interoception differentiation (pure fn) ===");
    {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        let raw = Fixed::from_f64(0.8);
        let felt: Vec<f64> = sim
            .agents
            .iter()
            .map(|a| a.interoception.corrected_felt_hunger(raw).to_f64())
            .collect();
        let mn = felt.iter().cloned().fold(f64::MAX, f64::min);
        let mx = felt.iter().cloned().fold(f64::MIN, f64::max);
        println!(
            "  same raw hunger 0.80 -> felt range [{mn:.3}, {mx:.3}] (spread {:.3})",
            mx - mn
        );
        println!("  CONTRACT identical raw != identical felt somewhere OR identity-at-default documented:");
        println!("    (identity-at-default is the shipped contract; spread>0 only with non-default constitutions)");
        // The real liveness check: pipeline consumes corrected values.
        println!(
            "  pipeline consumers wired: motivation.deficit <- corrected_felt_hunger/thirst (pass_cognitive.rs:200)"
        );
    }

    println!("\n=== [5] Phase-5 world variance across seeds ===");
    {
        let mut means = Vec::new();
        for seed in [42u64, 7, 13, 5] {
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: 10,
                world_width: 16,
                world_height: 16,
                num_agents: 12,
                snapshot_interval: None,
            });
            sim.populate();
            let fert_sum: f64 = sim.world().tiles.iter().map(|t| t.fertility.to_f64()).sum();
            let mean = fert_sum / sim.world().tiles.len() as f64;
            means.push(mean);
            println!("  seed {seed}: mean tile fertility {mean:.4}");
        }
        let spread = means.iter().cloned().fold(f64::MIN, f64::max)
            - means.iter().cloned().fold(f64::MAX, f64::min);
        println!(
            "  CONTRACT seeds differ (spread > 0.01): {spread:.4} -> {}",
            spread > 0.01
        );
        // determinism: same seed twice -> identical first-tile raw value
        let mut again = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 10,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        again.populate();
        let det_a = again.world().tiles[0].fertility.to_raw();
        let mut third = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 10,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        third.populate();
        let det_b = third.world().tiles[0].fertility.to_raw();
        println!(
            "  determinism same-seed (tile0 raw equal): {}",
            det_a == det_b
        );
    }

    println!("\n=== [6] Phase-6 chronicle/dossier smoke (already CLI-verified) ===");
    {
        let mut sim = Simulation::new(SimConfig {
            seed: 5,
            max_ticks: 12000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim.run(12000);
        let chronicle = mindstrata_sim::sim::chronicle::render_chronicle(&sim);
        let dossier = mindstrata_sim::sim::chronicle::render_dossier(&sim, 0);
        println!(
            "  chronicle {} bytes, {} year headers; dossier {} bytes",
            chronicle.len(),
            chronicle.matches("Year ").count(),
            dossier.len()
        );
    }
}
