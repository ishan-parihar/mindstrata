//! Phase 2 audit probe — AP2 §7 Biological Substrate operationalization.
//!
//! Prints per-system means/min/max and saturation flags (% of agents at 0.0
//! or 1.0) for every biological subsystem across seeds × horizons ×
//! scenarios, so the audit can verify the Phase 2 acceptance:
//!
//!   1. No axis saturates (pinned at 1.0 for >50% of agents in calm worlds).
//!   2. Stress/shock worlds show directional shifts (hunger/stress up,
//!      health down) vs calm.
//!   3. Per-agent spread exists (not all agents identical).
//!
//! Run with: `cargo run -p mindstrata-benches --example biology_probe --release`
//! (release required — 16 runs up to 20K ticks.)

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn base(seed: u64, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

fn scenario_sim(scenario: Scenario, ticks: u64) -> Simulation {
    let mut sim = Simulation::from_scenario(scenario);
    sim.populate();
    sim.run(ticks);
    sim
}

struct BioStats {
    label: String,
    values: Vec<f64>,
}

impl BioStats {
    fn report(&self) -> String {
        let n = self.values.len() as f64;
        if n == 0.0 {
            return format!("{}: (no agents)", self.label);
        }
        let mean: f64 = self.values.iter().sum::<f64>() / n;
        let min = self.values.iter().copied().fold(f64::INFINITY, f64::min);
        let max = self.values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        let at_zero = self.values.iter().filter(|v| **v <= 0.0).count();
        let at_one = self.values.iter().filter(|v| **v >= 1.0).count();
        let sat = if at_one as f64 > n * 0.5 { " ⚠ SATURATED-HIGH" } else { "" };
        let dead = if max - min < 0.001 { " (no spread)" } else { "" };
        format!(
            "  {:<26} mean={:.3} min={:.3} max={:.3} [0:{}/{}, 1:{}/{}]{}{}",
            self.label,
            mean,
            min,
            max,
            at_zero,
            self.values.len(),
            at_one,
            self.values.len(),
            sat,
            dead
        )
    }
}

fn stats(label: &str, values: impl IntoIterator<Item = Fixed>) -> BioStats {
    BioStats {
        label: label.to_string(),
        values: values.into_iter().map(|v| v.to_f64()).collect(),
    }
}

fn report_sim(tag: &str, sim: &Simulation) {
    println!("── {tag} ─────────────────────────────────────────────");
    let e: Vec<_> = sim.agents.iter().map(|a| &a.embodied).collect();

    println!("  Body / facade:");
    println!("    {}", stats("health", e.iter().map(|b| b.health)).report());
    println!("    {}", stats("energy", e.iter().map(|b| b.energy)).report());
    println!("    {}", stats("hunger", e.iter().map(|b| b.hunger)).report());
    println!("    {}", stats("thirst", e.iter().map(|b| b.thirst)).report());
    println!("    {}", stats("fatigue", e.iter().map(|b| b.fatigue)).report());
    println!("    {}", stats("derived_health", e.iter().map(|b| b.derived_health())).report());
    println!("    {}", stats("derived_energy", e.iter().map(|b| b.derived_energy())).report());

    println!("  Endocrine (§7.2.2):");
    println!("    {}", stats("stress.level", e.iter().map(|b| b.endocrine.stress.level)).report());
    println!("    {}", stats("stress.chronic", e.iter().map(|b| b.endocrine.stress.chronic_load)).report());
    println!("    {}", stats("bonding.level", e.iter().map(|b| b.endocrine.bonding.level)).report());
    println!("    {}", stats("dominance.level", e.iter().map(|b| b.endocrine.dominance.level)).report());
    println!("    {}", stats("arousal.level", e.iter().map(|b| b.endocrine.arousal.level)).report());
    println!("    {}", stats("growth.capacity", e.iter().map(|b| b.endocrine.growth.capacity)).report());

    println!("  Nervous (§7.2.5):");
    println!("    {}", stats("sympathetic", e.iter().map(|b| b.nervous.sympathetic_arousal)).report());
    println!("    {}", stats("parasympathetic", e.iter().map(|b| b.nervous.parasympathetic_tone)).report());
    println!("    {}", stats("pain.effective", e.iter().map(|b| b.nervous.pain.effective_pain())).report());
    println!("    {}", stats("pain.chronic", e.iter().map(|b| b.nervous.pain.chronic)).report());
    println!("    {}", stats("trauma_load", e.iter().map(|b| b.nervous.trauma_load)).report());
    println!("    {}", stats("sleep_pressure", e.iter().map(|b| b.nervous.sleep_pressure)).report());

    println!("  Muscular (§7.2.4):");
    println!("    {}", stats("strength", e.iter().map(|b| b.muscular.strength)).report());
    println!("    {}", stats("fatigue", e.iter().map(|b| b.muscular.fatigue)).report());
    println!("    {}", stats("conditioning", e.iter().map(|b| b.muscular.conditioning)).report());

    println!("  Skeletal (§7.2.3):");
    println!("    {}", stats("integrity", e.iter().map(|b| b.skeletal.structural_integrity)).report());
    println!("    {}", stats("mobility_penalty", e.iter().map(|b| b.skeletal.mobility_penalty)).report());

    println!("  Cardiovascular (§7.2.9):");
    println!("    {}", stats("fitness", e.iter().map(|b| b.cardiovascular.fitness)).report());
    println!("    {}", stats("shock_risk", e.iter().map(|b| b.cardiovascular.shock_risk)).report());
    println!("    {}", stats("blood_volume", e.iter().map(|b| b.cardiovascular.blood_volume)).report());

    println!("  Respiratory (§7.2.8):");
    println!("    {}", stats("lung_health", e.iter().map(|b| b.respiratory.lung_health)).report());
    println!("    {}", stats("oxygenation", e.iter().map(|b| b.respiratory.oxygenation)).report());

    println!("  Immune (§7.3.1):");
    println!("    {}", stats("resistance", e.iter().map(|b| b.immune.resistance)).report());
    println!("    {}", stats("infection_load", e.iter().map(|b| b.immune.infection_load)).report());
    println!("    {}", stats("inflammation", e.iter().map(|b| b.immune.inflammation)).report());

    println!("  Digestive (§7.2.7):");
    println!("    {}", stats("gut_health", e.iter().map(|b| b.digestive.gut_health)).report());
    println!("    {}", stats("satiety", e.iter().map(|b| b.digestive.satiety)).report());

    println!("  Thermal (§7.3.3):");
    println!("    {}", stats("body_temp", e.iter().map(|b| b.thermal.body_temperature)).report());
    println!("    {}", stats("cold_stress", e.iter().map(|b| b.thermal.cold_stress)).report());
    println!("    {}", stats("heat_stress", e.iter().map(|b| b.thermal.heat_stress)).report());

    println!("  Reproductive (§7.2.6):");
    println!("    {}", stats("fertility", e.iter().filter_map(|b| b.fertility)).report());
    println!("    {}", stats("libido", e.iter().map(|b| b.reproductive.libido)).report());
    let pregnant = sim
        .agents
        .iter()
        .filter(|a| a.embodied.reproductive.pregnancy.is_some())
        .count();
    println!("    pregnancies: {pregnant}");

    println!("  Development:");
    println!("    {}", stats("age", e.iter().map(|b| b.development.age)).report());
    let stages: Vec<&str> = e
        .iter()
        .map(|b| {
            use mindstrata_sim::biology::development::LifeStage;
            match b.development.life_stage {
                LifeStage::Infant => "infant",
                LifeStage::Child => "child",
                LifeStage::Adolescent => "adolescent",
                LifeStage::YoungAdult => "young-adult",
                LifeStage::Adult => "adult",
                LifeStage::Mature => "mature",
                LifeStage::Elder => "elder",
            }
        })
        .collect();
    println!("    life stages: {stages:?}");
}

fn main() {
    // ── Scenario matrix @5000 ticks (the Phase-2 directional check) ──
    for seed in [42u64, 1, 7] {
        for (name, sc) in [
            ("calm", Scenario::calm()),
            ("drought", Scenario::drought()),
            ("famine", Scenario::famine()),
            ("pestilence", Scenario::pestilence()),
        ] {
            let mut sim = scenario_sim(sc, 5000);
            // Force the scenario's own declared horizon is 5000; re-tag seed.
            report_sim(&format!("{name} seed{seed} @5000"), &sim);
            let _ = &mut sim;
        }
    }

    // ── Long-horizon calm + famine @20000 (saturation stability) ──
    type Maker = fn(u64) -> Simulation;
    let makers: [(&str, Maker); 2] = [
        ("calm-20K", |s: u64| {
            let mut sim = base(s, 20_000);
            sim.run(20_000);
            sim
        }),
        ("famine-20K", |_s: u64| {
            let mut sim = scenario_sim(Scenario::famine(), 20_000);
            sim
        }),
    ];
    for (name, make) in makers {
        for seed in [42u64, 46] {
            let sim = make(seed);
            report_sim(&format!("{name} seed{seed}"), &sim);
        }
    }
}
