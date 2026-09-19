//! i312 — health-reflex liveness re-audit (post-i311 downstream check).
//!
//! i309 added the health-critical frame: a body with derived health below 0.25
//! is forbidden from exerting itself (Work/Wander downgraded after every
//! selection path). i311 fixed the immune clearance pin that held a ~3% frailty
//! population below that gate, so the AGENTS §4.3 question stands: is the veto
//! now dead state?
//!
//! This probe samples EVERY tick and reports, per scenario:
//!   * the share of agent-ticks below the gate and the worst single agent,
//!   * how many agents ever crossed it,
//!   * the health distribution tails (p0.1 / p1 / p5 / median) — if no band
//!     separates crisis from calm, no absolute threshold can restore liveness,
//!   * the penalty decomposition of the worst-health agent, to see which
//!     derived-health terms can still reach an extreme.
//!
//! Run: cargo run --release -p mindstrata-benches --example i312_health_reflex_liveness

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HEALTH_GATE: f64 = 0.25;

#[derive(Default)]
struct Anatomy {
    health: f64,
    stress: f64,
    chronic: f64,
    pain: f64,
    sickness: f64,
    shock: f64,
    base_x_immune: f64,
}

fn percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return f64::NAN;
    }
    let idx = ((sorted.len() as f64 - 1.0) * q).round() as usize;
    sorted[idx]
}

fn run<F>(mut make: F, ticks: u64) -> (u64, u64, u64, Vec<f64>, Anatomy)
where
    F: FnMut(u64) -> Simulation,
{
    let mut agent_ticks = 0u64;
    let mut below_ticks = 0u64;
    let mut ever = 0u64;
    let mut samples: Vec<f64> = Vec::new();
    let mut worst = Anatomy {
        health: f64::INFINITY,
        ..Default::default()
    };
    for &seed in &SEEDS {
        let mut sim = make(seed);
        sim.populate();
        let mut crossed: Vec<bool> = Vec::new();
        for tick in 0..ticks {
            sim.tick();
            if crossed.len() != sim.agents.len() {
                crossed.resize(sim.agents.len(), false);
            }
            for (i, a) in sim.agents.iter().enumerate() {
                let h = a.body.health.to_f64();
                agent_ticks += 1;
                if tick % 20 == 0 {
                    samples.push(h);
                }
                if h < HEALTH_GATE {
                    below_ticks += 1;
                    if !crossed[i] {
                        crossed[i] = true;
                        ever += 1;
                    }
                }
                if h < worst.health {
                    let e = &a.embodied;
                    worst = Anatomy {
                        health: h,
                        stress: e.endocrine.stress.level.to_f64(),
                        chronic: e.endocrine.stress.chronic_load.to_f64(),
                        pain: e.nervous.pain.effective_pain().to_f64(),
                        sickness: e.immune.sickness_level().to_f64(),
                        shock: e.cardiovascular.shock_risk.to_f64(),
                        base_x_immune: (e.health.to_f64()
                            * (0.7
                                + e.genome.health_predispositions.immune_strength.to_f64() * 0.3))
                            * e.skeletal.health_factor().to_f64(),
                    };
                }
            }
        }
    }
    samples.sort_by(f64::total_cmp);
    (agent_ticks, below_ticks, ever, samples, worst)
}

fn report(label: &str, agent_ticks: u64, below: u64, ever: u64, samples: &[f64], w: &Anatomy) {
    let share = 100.0 * below as f64 / agent_ticks.max(1) as f64;
    println!(
        "  {label:<16} agent-ticks {:<8} below {:<6} ({:.5}%) | ever {:<3} | \
         p0.1 {:.4} p1 {:.4} p5 {:.4} p50 {:.4}",
        agent_ticks,
        below,
        share,
        ever,
        percentile(samples, 0.001),
        percentile(samples, 0.01),
        percentile(samples, 0.05),
        percentile(samples, 0.50),
    );
    println!(
        "      worst agent: health {:.4} = base×immune {:.4} − stress {:.4} − chronic {:.4} \
         − pain {:.4} − sickness {:.4} − shock {:.4}",
        w.health,
        w.base_x_immune,
        0.2 * w.stress,
        0.15 * w.chronic,
        0.1 * w.pain,
        0.15 * w.sickness,
        0.1 * w.shock,
    );
}

fn main() {
    println!("i312 health-reflex liveness (health < {HEALTH_GATE})");
    let scenarios: [(&str, Scenario); 3] = [
        ("pestilence", Scenario::pestilence()),
        ("collapse", Scenario::collapse()),
        ("drought", Scenario::drought()),
    ];
    for (label, sc) in scenarios {
        for ticks in [4_320u64, 20_000] {
            let sc2 = sc.clone();
            let (at, below, ever, samples, w) = run(
                move |seed| {
                    let mut s = sc2.clone();
                    s.seed = seed;
                    s.ticks = ticks;
                    Simulation::from_scenario(s)
                },
                ticks,
            );
            report(&format!("{label} @{ticks}"), at, below, ever, &samples, &w);
        }
    }
    for ticks in [20_000u64, 50_000] {
        let (at, below, ever, samples, w) = run(
            |seed| {
                Simulation::new(SimConfig {
                    seed,
                    max_ticks: ticks,
                    num_agents: 12,
                    snapshot_interval: None,
                    ..SimConfig::default()
                })
            },
            ticks,
        );
        report(
            &format!("calm family @{ticks}"),
            at,
            below,
            ever,
            &samples,
            &w,
        );
    }
}
