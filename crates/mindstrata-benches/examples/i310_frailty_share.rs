//! i310 — chronic-frailty share probe (stage 1: measure before touching).
//!
//! i309 turned the health-critical reflex into an exertion veto and recorded a
//! calibration question: `derived_health` equilibrates at ~0.21–0.25 for the
//! most chronically stressed agents, so the 0.25 gate is in practice a CHRONIC
//! frailty state rather than an acute crisis detector. Before deciding whether
//! to split the gate (acute: pain / shock / injury / sickness) from chronic
//! frailty, measure how large that population actually is and what holds it
//! under the line.
//!
//! Run: cargo run --release -p mindstrata-benches --example i310_frailty_share

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HEALTH_GATE: f64 = 0.25;

fn config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        num_agents: 12,
        snapshot_interval: None,
        ..SimConfig::default()
    }
}

#[derive(Default, Clone, Copy)]
struct Sample {
    agents: u64,
    below_gate: u64,
    /// Component means over the BELOW-GATE population only.
    stress_level: f64,
    chronic_load: f64,
    pain: f64,
    sickness: f64,
    working: u64,
    resting: u64,
    trading: u64,
    socializing: u64,
}

impl Sample {
    fn merge(&mut self, o: &Sample) {
        self.agents += o.agents;
        self.below_gate += o.below_gate;
        self.stress_level += o.stress_level;
        self.chronic_load += o.chronic_load;
        self.pain += o.pain;
        self.sickness += o.sickness;
        self.working += o.working;
        self.resting += o.resting;
        self.trading += o.trading;
        self.socializing += o.socializing;
    }
}

fn sample(sim: &Simulation) -> Sample {
    let mut s = Sample::default();
    for a in &sim.agents {
        s.agents += 1;
        let health = a.body.health.to_f64();
        if health >= HEALTH_GATE {
            continue;
        }
        s.below_gate += 1;
        s.stress_level += a.embodied.endocrine.stress.level.to_f64();
        s.chronic_load += a.embodied.endocrine.stress.chronic_load.to_f64();
        s.pain += a.embodied.nervous.pain.effective_pain().to_f64();
        s.sickness += a.embodied.immune.sickness_level().to_f64();
        match a.current_action {
            mindstrata_sim::actions::ActionKind::Work => s.working += 1,
            mindstrata_sim::actions::ActionKind::Rest => s.resting += 1,
            mindstrata_sim::actions::ActionKind::Trade => s.trading += 1,
            mindstrata_sim::actions::ActionKind::Socialize => s.socializing += 1,
            _ => {}
        }
    }
    s
}

fn report(label: &str, s: &Sample) {
    let n = s.below_gate.max(1) as f64;
    println!(
        "  {label:<26} population {:>4} | below gate {:>4} ({:>5.1}%) | stress {:.3} chronic {:.3} pain {:.3} sick {:.3} | Work {:.2} Rest {:.2} Trade {:.2} Socialize {:.2}",
        s.agents,
        s.below_gate,
        100.0 * s.below_gate as f64 / s.agents.max(1) as f64,
        s.stress_level / n,
        s.chronic_load / n,
        s.pain / n,
        s.sickness / n,
        s.working as f64 / n,
        s.resting as f64 / n,
        s.trading as f64 / n,
        s.socializing as f64 / n,
    );
}

fn main() {
    println!("i310 chronic-frailty share — derived health below {HEALTH_GATE}");
    println!("  (component means are over the BELOW-GATE population only)\n");

    for ticks in [2_000u64, 20_000, 50_000] {
        let mut total = Sample::default();
        for &seed in &SEEDS {
            let mut sim = Simulation::new(config(seed, ticks));
            sim.populate();
            for _ in 0..ticks {
                sim.tick();
            }
            total.merge(&sample(&sim));
        }
        report(&format!("calm family @{ticks}"), &total);
    }

    // ── Leg 2: the immune anatomy of the below-gate population ───────────
    //
    // The component means say the gate is held by SICKNESS (0.84), not pain,
    // and `sickness_level = 0.6·infection + 0.4·inflammation`. This leg prints
    // the underlying immune state so the next iteration has the driver, not
    // the symptom.
    println!("\n[leg 2] immune anatomy of below-gate agents (calm, 50K)");
    println!(
        "  {:<6} {:<4} {:>8} {:>9} {:>10} {:>10} {:>9} {:>8} {:>7} {:>7}",
        "seed",
        "id",
        "health",
        "sickness",
        "infection",
        "inflam",
        "resistance",
        "recover",
        "injury",
        "stress"
    );
    let mut shown = 0usize;
    for &seed in &SEEDS {
        let mut sim = Simulation::new(config(seed, 50_000));
        sim.populate();
        for _ in 0..50_000u64 {
            sim.tick();
        }
        for (id, a) in sim.agents.iter().enumerate() {
            if a.body.health.to_f64() >= HEALTH_GATE {
                continue;
            }
            shown += 1;
            println!(
                "  {seed:<6} {id:<4} {:>8.4} {:>9.4} {:>10.4} {:>10.4} {:>9.4} {:>8.4} {:>7.3} {:>7.3}",
                a.body.health.to_f64(),
                a.embodied.immune.sickness_level().to_f64(),
                a.embodied.immune.infection_load.to_f64(),
                a.embodied.immune.inflammation.to_f64(),
                a.embodied.immune.resistance.to_f64(),
                a.embodied.immune.recovery_capacity.to_f64(),
                a.embodied.injury.to_f64(),
                a.embodied.endocrine.stress.level.to_f64(),
            );
        }
    }
    println!("  below-gate agents found: {shown}");

    for (label, sc) in [
        ("collapse (crisis)", Scenario::collapse()),
        ("pestilence", Scenario::pestilence()),
        ("drought", Scenario::drought()),
    ] {
        let mut total = Sample::default();
        let ticks = sc.ticks.min(20_000);
        for &seed in &SEEDS {
            let mut s = sc.clone();
            s.seed = seed;
            s.ticks = ticks;
            let mut sim = Simulation::from_scenario(s);
            sim.populate();
            for _ in 0..ticks {
                sim.tick();
            }
            total.merge(&sample(&sim));
        }
        report(&format!("{label} @{ticks}"), &total);
    }
}
