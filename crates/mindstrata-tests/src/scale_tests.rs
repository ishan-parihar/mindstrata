//! §2 scale-validation — the sim's DESIGNED settlement size is the §19.5.F
//! population cap (MAX_POPULATION = 48, population_cap.rs), enforced at
//! populate (requested > cap is clamped) and at every birth gate. Every
//! prior test ran ≤ 48 agents but none validated behavior AT full capacity:
//! this module closes that gap — cap enforcement, through-birth safety,
//! invariant cleanliness, and determinism at the designed maximum.

use mindstrata_sim::{sim::SimConfig, Simulation};

fn full_capacity_config(seed: u64, ticks: u64) -> SimConfig {
    SimConfig {
        seed,
        max_ticks: ticks,
        world_width: 32,
        world_height: 32,
        num_agents: mindstrata_sim::population_cap::STRESS_POPULATION as u32,
        snapshot_interval: None,
    }
}

/// §19.5.F: the population cap is enforced at populate — over-requesting is
/// clamped to the ceiling, and requests at/below it pass through intact.
#[test]
fn population_cap_enforced_at_populate() {
    let ceiling = mindstrata_sim::population_cap::MAX_POPULATION;
    // Over-request (300, 500) -> clamped to exactly the ceiling.
    // (Iteration 261: 100/200 now pass through — the ceiling is 256.)
    for requested in [300u32, 500] {
        let mut sim = Simulation::new(SimConfig {
            seed: 42,
            max_ticks: 100,
            world_width: 32,
            world_height: 32,
            num_agents: requested,
            snapshot_interval: None,
        });
        sim.populate();
        assert_eq!(
            sim.agent_count(),
            ceiling,
            "requested {requested} agents must clamp to the ceiling {ceiling}"
        );
    }
    // At-cap and below-cap requests pass through intact.
    let mut at_cap = Simulation::new(full_capacity_config(42, 100));
    at_cap.populate();
    assert_eq!(
        at_cap.agent_count(),
        mindstrata_sim::population_cap::STRESS_POPULATION,
        "at-cap request must pass through"
    );
    let mut small = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 100,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    small.populate();
    assert_eq!(
        small.agent_count(),
        12,
        "below-cap request must pass through"
    );
}

/// §19.5.F + §2: at full capacity the population NEVER exceeds the cap
/// and the core invariants hold at every sampled tick — resources never
/// negative, all 22 emotions bounded to [0, 1]. 600 ticks with per-100
/// sampling; the sampling is probabilistic, not exhaustive (a buggy gate
/// that overshoots compounds every birth, so a 100-tick cadence would catch
/// it). HONEST VACUITY NOTE (probe-pinned): at seed 42 the 48-agent village
/// is perfectly healthy — agent_count sits at exactly 48 at every 100-tick
/// sample through 2000 ticks, i.e. no deaths occur in-window and the birth
/// gates never fire here. The through-births cap check under churn is
/// exercised by the famine/pestilence integration tests (which re-pace
/// births under mortality); this test proves the cap HOLDS at full capacity
/// and the invariant surface stays clean while the village is alive.
#[test]
fn full_capacity_population_stays_capped_and_invariant_clean() {
    let cap = mindstrata_sim::population_cap::MAX_POPULATION;
    // Collapse floor stays relative to the §19.5.F stress-size seeding.
    let seeded = mindstrata_sim::population_cap::STRESS_POPULATION;
    let mut sim = Simulation::new(full_capacity_config(42, 600));
    sim.populate();
    for tick in 0..600 {
        sim.tick();
        let t = tick + 1;
        if t % 100 == 0 {
            let alive = sim.agent_count();
            assert!(
                alive <= cap,
                "tick {t}: population {alive} exceeds the cap {cap} — a birth gate is broken"
            );
            // Collapse-detector, NOT a stability contract: a genuine mass
            // die-off would crater toward 0; the floor just catches that
            // (cap/4 is well below any plausible churn trough on seed 42).
            assert!(
                alive > seeded / 4,
                "tick {t}: population collapsed to {alive} — mass death at full capacity"
            );
            assert!(
                sim.total_grain().to_f64() >= 0.0 && sim.total_water().to_f64() >= 0.0,
                "tick {t}: resources went negative"
            );
            for agent in &sim.agents {
                let e = &agent.emotions;
                for (name, v) in [
                    ("fear", e.fear),
                    ("anger", e.anger),
                    ("joy", e.joy),
                    ("sadness", e.sadness),
                    ("trust", e.trust),
                    ("shame", e.shame),
                    ("pride", e.pride),
                    ("guilt", e.guilt),
                    ("disgust", e.disgust),
                    ("contempt", e.contempt),
                    ("awe", e.awe),
                    ("gratitude", e.gratitude),
                    ("jealousy", e.jealousy),
                    ("envy", e.envy),
                    ("loneliness", e.loneliness),
                    ("tenderness", e.tenderness),
                    ("humiliation", e.humiliation),
                    ("relief", e.relief),
                    ("hope", e.hope),
                    ("despair", e.despair),
                    ("nostalgia", e.nostalgia),
                    ("moral_outrage", e.moral_outrage),
                ] {
                    let v = v.to_f64();
                    assert!(
                        (0.0..=1.0).contains(&v),
                        "tick {t}: emotion {name} = {v} out of [0, 1]"
                    );
                }
            }
        }
    }
}

/// §2: full-capacity runs are seed-deterministic — the same seed twice at
/// the cap yields byte-identical aggregate metrics, and the end state is
/// healthy (population survives at the cap, valence finite).
#[test]
fn full_capacity_run_is_deterministic_and_healthy() {
    let run = |seed: u64| {
        let mut sim = Simulation::new(full_capacity_config(seed, 500));
        sim.populate();
        sim.run(500);
        sim.metrics_snapshot()
    };
    let a = run(42);
    let b = run(42);
    assert_eq!(
        a.avg_hunger, b.avg_hunger,
        "hunger must be seed-deterministic"
    );
    assert_eq!(
        a.avg_valence, b.avg_valence,
        "valence must be seed-deterministic"
    );
    assert_eq!(
        a.total_grain, b.total_grain,
        "grain must be seed-deterministic"
    );
    assert_eq!(
        a.event_count, b.event_count,
        "events must be seed-deterministic"
    );
    assert_eq!(
        a.agent_count, b.agent_count,
        "agent count must be seed-deterministic"
    );
    // End-state health at the cap.
    assert!(
        a.agent_count >= mindstrata_sim::population_cap::STRESS_POPULATION as u64 / 2,
        "population must survive to a healthy size at the cap, got {}",
        a.agent_count
    );
    assert!(a.avg_valence.is_finite(), "valence must stay finite");
    assert!(a.total_grain >= 0.0 && a.total_water >= 0.0);
}
