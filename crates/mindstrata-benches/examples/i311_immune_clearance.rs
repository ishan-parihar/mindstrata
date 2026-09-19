//! i311 — immune-clearance probe (stage 1: measure before touching).
//!
//! i310 named the driver of the frailty population: `infection_load` pinned at
//! 1.0000 over a resistance that had collapsed to ~0.000, where
//! `ImmuneState::tick_update`'s clearance term
//!
//! ```ignore
//! fight = resistance × recovery_capacity × 0.005 × infection_load × (0.7 + 0.6·recovery_rate)
//! ```
//!
//! evaluates in `Fixed` (4-decimal) as ~1.4e-7 and therefore truncates to ZERO —
//! the AGENTS §5 sub-resolution class, in the immune path. Resistance is also
//! absorbing at zero (stress suppression 0.00075/tick against nutrition+~sleep
//! boosts ~0.0007), so an immunocompromised body can never recover competence.
//!
//! This probe measures the two axes the fix has to be judged on:
//!   * the frailty population (calm @50K: below-gate share, pinned infection),
//!   * the epidemic axis (the R0≈1 knife-edge recorded as systemic debt), on
//!     pestilence and collapse, at the horizons the tests and golden use.
//!
//! Run: cargo run --release -p mindstrata-benches --example i311_immune_clearance

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

#[derive(Default, Clone)]
struct Immune {
    agents: u64,
    below_gate: u64,
    pinned_infection: u64,
    collapsed_resistance: u64,
    infection_sum: f64,
    sickness_sum: f64,
    resistance_sum: f64,
    max_infection: f64,
    sick_above_half: u64,
    frozen_residual: u64,
    pinned_anatomy: Vec<String>,
}

fn scan(sim: &Simulation) -> Immune {
    let mut s = Immune::default();
    for a in sim.agents.iter() {
        s.agents += 1;
        let inf = a.embodied.immune.infection_load.to_f64();
        let res = a.embodied.immune.resistance.to_f64();
        let sick = a.embodied.immune.sickness_level().to_f64();
        if inf >= 0.999 && s.pinned_anatomy.len() < 6 {
            let c = a.embodied.immune.recovery_capacity.to_f64();
            let rr = a
                .embodied
                .genome
                .health_predispositions
                .recovery_rate
                .to_f64();
            let fight_f = res * c * 0.005 * inf * (0.7 + rr * 0.6);
            s.pinned_anatomy.push(format!(
                "R{res:.4} C{c:.4} rr{rr:.3} infl{:.3} fight_f{fight_f:.2e} stress{:.3} health{:.3}",
                a.embodied.immune.inflammation.to_f64(),
                a.embodied.endocrine.stress.level.to_f64(),
                a.body.health.to_f64(),
            ));
        }
        // Frozen-residual detector: no exposure channel (stress <= 0.4) and a
        // load in the range where the clearance term truncates to zero.
        if a.embodied.endocrine.stress.level.to_f64() <= 0.4 && inf > 0.001 && inf < 0.5 {
            s.frozen_residual += 1;
        }
        s.infection_sum += inf;
        s.sickness_sum += sick;
        s.resistance_sum += res;
        s.max_infection = s.max_infection.max(inf);
        if sick > 0.5 {
            s.sick_above_half += 1;
        }
        if inf >= 0.999 {
            s.pinned_infection += 1;
        }
        if res < 0.02 {
            s.collapsed_resistance += 1;
        }
        if a.body.health.to_f64() < HEALTH_GATE {
            s.below_gate += 1;
        }
    }
    s
}

impl Immune {
    fn merge(&mut self, o: &Immune) {
        self.agents += o.agents;
        self.below_gate += o.below_gate;
        self.pinned_infection += o.pinned_infection;
        self.collapsed_resistance += o.collapsed_resistance;
        self.infection_sum += o.infection_sum;
        self.sickness_sum += o.sickness_sum;
        self.resistance_sum += o.resistance_sum;
        self.max_infection = self.max_infection.max(o.max_infection);
        self.sick_above_half += o.sick_above_half;
        self.frozen_residual += o.frozen_residual;
    }
}

fn report(label: &str, s: &Immune) {
    let n = s.agents.max(1) as f64;
    println!(
        "  {label:<24} agents {:>4} | below gate {:>3} ({:>5.1}%) | pinned inf {:>3} | R<0.02 {:>3} | inf {:.3} sick {:.3} R {:.3} maxI {:.3} sick>0.5 {:>3}",
        s.agents,
        s.below_gate,
        100.0 * s.below_gate as f64 / n,
        s.pinned_infection,
        s.collapsed_resistance,
        s.infection_sum / n,
        s.sickness_sum / n,
        s.resistance_sum / n,
        s.max_infection,
        s.sick_above_half,
    );
    if s.frozen_residual > 0 {
        println!(
            "      frozen-residual (stress<=0.4, 0.001<load<0.5): {}",
            s.frozen_residual
        );
    }
}

fn main() {
    println!("i311 immune-clearance probe");

    // ── Leg 1: calm family, the frailty population ───────────────────────
    for ticks in [2_000u64, 20_000, 50_000] {
        let mut total = Immune::default();
        for &seed in &SEEDS {
            let mut sim = Simulation::new(config(seed, ticks));
            sim.populate();
            for _ in 0..ticks {
                sim.tick();
            }
            let one = scan(&sim);
            if ticks == 50_000 {
                for (i, line) in one.pinned_anatomy.iter().enumerate() {
                    println!("    seed {seed} pinned #{i}: {line}");
                }
            }
            total.merge(&one);
        }
        report(&format!("calm family @{ticks}"), &total);
    }

    // ── Leg 2: the crisis axis (pestilence / collapse) ───────────────────
    for (label, sc) in [
        ("pestilence", Scenario::pestilence()),
        ("collapse", Scenario::collapse()),
    ] {
        for ticks in [4_320u64, 20_000] {
            let mut total = Immune::default();
            for &seed in &SEEDS {
                let mut s = sc.clone();
                s.seed = seed;
                s.ticks = ticks;
                let mut sim = Simulation::from_scenario(s);
                sim.populate();
                for _ in 0..ticks {
                    sim.tick();
                }
                total.merge(&scan(&sim));
            }
            report(&format!("{label} @{ticks}"), &total);
        }
    }

    // ── Leg 3: the 50K emergence leg's world (pestilence seed 5) ─────────
    // `long_horizon_50k_is_deterministic_and_emerges` runs exactly this.
    {
        let mut sc = Scenario::pestilence();
        sc.seed = 5;
        sc.ticks = 50_000;
        let mut sim = Simulation::from_scenario(sc);
        sim.populate();
        // Epidemic trace: how much of the run has any infection at all, and
        // whether the infection dies out (transient) or persists (endemic).
        let mut infected_ticks = 0u64;
        let mut ticks = 0u64;
        let mut peak = 0.0f64;
        for _ in 0..50_000u64 {
            sim.tick();
            ticks += 1;
            let mut any = false;
            for a in sim.agents.iter() {
                let inf = a.embodied.immune.infection_load.to_f64();
                peak = peak.max(inf);
                if inf > 0.05 {
                    any = true;
                }
            }
            if any {
                infected_ticks += 1;
            }
        }
        let s = scan(&sim);
        report("pestilence 5 @50K", &s);
        println!(
            "  epidemic trace: infected-tick share {:.3} | peak infection {:.4} | panics {} | factions {} | agents {}",
            infected_ticks as f64 / ticks.max(1) as f64,
            peak,
            sim.moral_panic_registry.panics.len(),
            sim.faction_v2_registry.factions.len(),
            sim.agents.len()
        );
    }
}
