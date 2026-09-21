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
use crate::actions::ActionKind;

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

/// Liveness + inertness of the census, and the i347 acceptance criterion for the
/// revived §19.5.G producer.
///
/// One test case deliberately: the sink is process-global and `reset()` clears
/// it, so a second parallel case would wipe this one's counts (measured — an
/// earlier two-case version read 169 of an expected ≥480 records).
#[test]
fn census_records_decisions_without_changing_them() {
    const N: u32 = 12;
    const TICKS: u64 = 5_000;
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

    // i347 acceptance: before the fix this row was exactly zero (i346 measured
    // 0 of 197 146 decisions at N=48), because the §19.5.G gate sat above the
    // anger channel's reach AND the branch was shadowed by the daily routine.
    // `Move` has this one producer, so the two facts are the same fact.
    let feud = report.sources[decision_census::SRC_FEUD];
    let move_actions = report.actions[decision_census::action_index(ActionKind::Move {
        target_x: 0,
        target_y: 0,
    })];
    assert!(
        feud > 0 && move_actions > 0,
        "the feud-approach producer must be live (feud {feud}, Move {move_actions} over \
         {TICKS} ticks at N={N})"
    );
    assert_eq!(
        report.cross[decision_census::SRC_FEUD][decision_census::action_index(ActionKind::Move {
            target_x: 0,
            target_y: 0,
        })],
        feud,
        "every feud-sourced decision must produce a Move (it is the branch's only output)"
    );

    // NOTE (upper bound deliberately absent): a decision is recorded on
    // re-selection, not every tick, so in isolation `total() <= N·TICKS` would
    // hold — but the harness runs 295 test cases in parallel threads and the
    // sink is process-global, so every other running simulation adds records
    // while this test holds the census open (measured: 1 failing run read a
    // total above N·TICKS). Exact per-run attribution is pinned by the
    // single-process probe `i346_decision_census`, not here.

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
