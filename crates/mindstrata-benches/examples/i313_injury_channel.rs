//! i313 — the injury channel liveness probe.
//!
//! i312 found that two of the five derived-health penalty channels (`pain`,
//! `shock`) contribute exactly 0.0000, and traced both to a single upstream
//! producer: `EmbodiedState.injury`. This probe measures the whole chain:
//!
//!   injury → pain (`nervous.pain.update`)         → health `pain_penalty`
//!   injury → blood loss (`cardiovascular`)        → `shock_risk` → `shock_penalty`
//!   injury → immune wound exposure + `chronic_damage`
//!
//! It reports the max ever seen for each link, plus the count of violent
//! norm violations that SHOULD be the upstream source.
//!
//! Run: cargo run --release -p mindstrata-benches --example i313_injury_channel

use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::sim::{SimConfig, Simulation};

const SEEDS: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];

#[derive(Default, Clone, Copy)]
struct Axes {
    max_injury: f64,
    max_pain: f64,
    max_shock: f64,
    min_blood_volume: f64,
    max_chronic_pain: f64,
    max_sickness: f64,
}

impl Axes {
    fn new() -> Self {
        Self {
            min_blood_volume: f64::INFINITY,
            ..Default::default()
        }
    }

    fn observe(&mut self, sim: &Simulation) {
        for a in &sim.agents {
            let e = &a.embodied;
            self.max_injury = self.max_injury.max(e.injury.to_f64());
            self.max_pain = self.max_pain.max(e.nervous.pain.effective_pain().to_f64());
            self.max_chronic_pain = self.max_chronic_pain.max(e.skeletal.chronic_pain.to_f64());
            self.max_shock = self.max_shock.max(e.cardiovascular.shock_risk.to_f64());
            self.min_blood_volume = self
                .min_blood_volume
                .min(e.cardiovascular.blood_volume.to_f64());
            self.max_sickness = self.max_sickness.max(e.immune.sickness_level().to_f64());
        }
    }

    fn report(&self, label: &str, violations: u64) {
        println!(
            "  {label:<18} max injury {:.5} | max pain {:.5} | max chronic-pain {:.5} | \
             max shock {:.5} | min blood-vol {:.4} | max sickness {:.4} | violent violations {violations}",
            self.max_injury,
            self.max_pain,
            self.max_chronic_pain,
            self.max_shock,
            self.min_blood_volume,
            self.max_sickness,
        );
    }
}

/// Count public violence events in the retained event window.
///
/// i327 note: `recent_events` is a **bounded** window now, so at long
/// horizons this undercounts the cumulative total. It is a liveness signal
/// (violence fires), not a rate — the verdict-bearing figures in this probe
/// (injury, pain, shock, blood-volume) are state-based and unaffected.
fn count_violence(sim: &Simulation) -> u64 {
    sim.recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                mindstrata_core::event::SimEvent::ConflictOccurred {
                    kind: mindstrata_core::conflict::ConflictKind::Violence,
                    ..
                }
            )
        })
        .count() as u64
}

fn main() {
    println!("i313 injury-channel liveness");
    let scenarios: [(&str, Scenario); 3] = [
        ("pestilence", Scenario::pestilence()),
        ("collapse", Scenario::collapse()),
        ("drought", Scenario::drought()),
    ];
    for (label, sc) in scenarios {
        for ticks in [4_320u64, 20_000] {
            let mut axes = Axes::new();
            let mut violations = 0u64;
            for &seed in &SEEDS {
                let mut s = sc.clone();
                s.seed = seed;
                s.ticks = ticks;
                let mut sim = Simulation::from_scenario(s);
                sim.populate();
                for _ in 0..ticks {
                    sim.tick();
                    axes.observe(&sim);
                }
                violations += count_violence(&sim);
            }
            axes.report(&format!("{label} @{ticks}"), violations);
        }
    }
    for ticks in [20_000u64, 50_000] {
        let mut axes = Axes::new();
        let mut violations = 0u64;
        for &seed in &SEEDS {
            let mut sim = Simulation::new(SimConfig {
                seed,
                max_ticks: ticks,
                num_agents: 12,
                snapshot_interval: None,
                ..SimConfig::default()
            });
            sim.populate();
            for _ in 0..ticks {
                sim.tick();
                axes.observe(&sim);
            }
            violations += count_violence(&sim);
        }
        axes.report(&format!("calm family @{ticks}"), violations);
    }
}
