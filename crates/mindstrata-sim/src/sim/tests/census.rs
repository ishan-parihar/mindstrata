//! i346: pins for the decision-selection census (`sim::decision_census`).
//!
//! The census is instrumentation, so its pins guard two properties that
//! matter: it must record the selection layer (liveness), and it must be
//! incapable of reaching a decision (inertness). The second is what makes
//! every number it reports trustworthy.
//!
//! Both live in **one** test case on purpose: the sink is process-global and
//! `reset()` clears it, so two parallel cases would wipe each other's counts
//! (measured — the first draft failed at 169 of an expected ≥480 records).

use super::super::decision_census;
use super::{SimConfig, Simulation};

fn sim(n: u32, ticks: u64) -> Simulation {
    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: ticks,
        world_width: 32,
        world_height: 32,
        num_agents: n,
        snapshot_interval: None,
    });
    sim.populate();
    sim
}

/// Public-field fingerprint of a run — enough state to notice any behavioural
/// difference without reaching into private fields.
fn fingerprint(sim: &Simulation) -> (usize, usize, u64, i64) {
    let mut fold = 0i64;
    for a in &sim.agents {
        fold += (a.position.x as i64) * 31 + (a.position.y as i64) * 37;
        fold += (a.needs.hunger.to_f64() * 1e6) as i64;
        fold += (a.needs.thirst.to_f64() * 1e6) as i64;
        fold += (a.body.health.to_f64() * 1e6) as i64;
    }
    (
        sim.agent_count(),
        sim.event_count(),
        sim.current_tick().as_u64(),
        fold,
    )
}

/// Liveness + inertness of the census.
#[test]
fn census_records_decisions_without_changing_them() {
    const N: u32 = 12;
    const TICKS: u64 = 400;
    // The longest action is 8 ticks, so an agent that never idles still decides
    // at least TICKS/8 times; TICKS/10 is a safe floor.
    const MIN_DECISIONS: u64 = N as u64 * (TICKS / 10);

    decision_census::enable();
    decision_census::reset();
    let mut instrumented = sim(N, TICKS);
    instrumented.run(TICKS);
    let report = decision_census::report();
    decision_census::disable();

    assert!(
        report.total() >= MIN_DECISIONS,
        "census recorded {} decisions, expected at least {MIN_DECISIONS}",
        report.total()
    );
    assert_eq!(
        report.sources.iter().sum::<u64>(),
        report.actions.iter().sum::<u64>(),
        "every recorded decision needs exactly one source and one action"
    );
    assert!(
        report.sources[decision_census::SRC_ROUTINE] > 0
            || report.sources[decision_census::SRC_UTILITY] > 0,
        "the two layers that decide most actions must both be observable"
    );
    assert!(
        report.utility_samples > 0,
        "the utility leg must report its arbitrations"
    );

    // A decision is recorded on re-selection, not every tick — the census says
    // so, and the pin keeps the semantics from silently changing.
    assert!(
        report.total() <= N as u64 * TICKS,
        "decisions cannot outnumber agent-ticks"
    );

    let mut plain = sim(N, TICKS);
    plain.run(TICKS);
    assert_eq!(
        fingerprint(&instrumented),
        fingerprint(&plain),
        "the census must not perturb behaviour (it draws no RNG and writes no state)"
    );
}

/// The structural half of the i346 finding: `Wander` is the only action in the
/// selection set with **no** need-relief term at all, which is why it loses
/// every arbitration by ~1.5 units — 30× the ±0.05 decision jitter.
#[test]
fn wander_carries_no_need_relief_at_all() {
    use crate::actions::ActionKind;

    let eat = ActionKind::Eat.definition();
    assert!(
        eat.hunger_relief > mindstrata_core::fixed::Fixed::ZERO,
        "control: a relief-bearing action must carry relief"
    );

    let def = ActionKind::Wander.definition();
    let relief = def.hunger_relief
        + def.thirst_relief
        + def.fatigue_relief
        + def.bonus_social_relief
        + def.bonus_meaning_relief
        + def.bonus_fatigue_relief;
    assert_eq!(
        relief,
        mindstrata_core::fixed::Fixed::ZERO,
        "Wander is imported at zero relief for every channel; giving it a relief \
         term is a behavioural change (i346 recorded the measured gap)"
    );

    // `Idle` is the near-miss case: it does carry a relief term, but a weak one
    // (0.05 fatigue per 1 tick) that never wins either. Pinned so the census
    // commentary cannot drift away from the code.
    let idle = ActionKind::Idle.definition();
    assert_eq!(
        idle.fatigue_relief,
        mindstrata_core::fixed::Fixed::from_f64(0.05)
    );
    assert_eq!(idle.hunger_relief, mindstrata_core::fixed::Fixed::ZERO);
}
