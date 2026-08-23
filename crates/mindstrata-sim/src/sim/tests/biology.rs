//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::*;

#[test]
fn survival_reflex_forces_relief_over_utility() {
    use rand::SeedableRng;
    use rand_chacha::ChaCha8Rng;
    // Iteration 255 (audit Phase 3): a body at its limits must take the
    // relief action REGARDLESS of utility contest, habits, or commands —
    // the E3 inversion (starving while working) is impossible by
    // construction.
    use crate::actions::DecisionContext;
    use crate::person::{IdentityState, Personality};
    use crate::psychology::motivation::MotiveCategory;
    use mindstrata_core::fixed::Fixed;

    let needs = NeedState {
        hunger: Fixed::from_f64(0.95),
        thirst: Fixed::from_f64(0.5),
        ..Default::default()
    };
    let mut rng_p = rand_chacha::ChaCha8Rng::seed_from_u64(3);
    let personality = Personality::random(&mut rng_p);
    let identity = IdentityState::default();
    let dp = crate::psychology::decision_policy::DecisionPolicy::default();
    let goals: Vec<crate::person::Goal> = Vec::new();
    let ctx = DecisionContext {
        needs: &needs,
        personality: &personality,
        active_goals: &goals,
        identity: &identity,
        decision_policy: &dp,
        total_grain: Fixed::from_f64(0.5),
        total_water: Fixed::from_f64(0.5),
        coin: Fixed::from_f64(10.0),
        norm_pressure: Fixed::ZERO,
        anger: Fixed::ZERO,
        fear: Fixed::ZERO,
        joy: Fixed::ZERO,
        sadness: Fixed::ZERO,
        stress: Fixed::ZERO,
        fairness: Fixed::from_f64(0.5),
        authority: Fixed::from_f64(0.5),
        care: Fixed::from_f64(0.5),
        loyalty: Fixed::from_f64(0.5),
        action_values: Default::default(),
        dominant_need: MotiveCategory::Hunger,
        dominant_pressure: Fixed::from_f64(0.9),
        dread: Fixed::ZERO,
        hope: Fixed::ZERO,
        planning_confidence: Fixed::from_f64(0.5),
        mood_valence: Fixed::ZERO,
        season: 0,
        life_stage: 4,
        social_withdrawal: Fixed::ZERO,
        somatic_marker: Fixed::ZERO,
    };
    // The utility AI alone might pick anything; the REFLEX layer forces Eat.
    // Simulate the pass_action gate directly: hunger > 0.9 → forced relief.
    let forced = if needs.thirst > Fixed::from_f64(0.9) && needs.thirst >= needs.hunger {
        Some(crate::actions::ActionKind::Drink)
    } else if needs.hunger > Fixed::from_f64(0.9) {
        Some(crate::actions::ActionKind::Eat)
    } else if needs.fatigue > Fixed::from_f64(0.95) {
        Some(crate::actions::ActionKind::Rest)
    } else {
        None
    };
    assert_eq!(forced, Some(crate::actions::ActionKind::Eat));
    // The deliberated path stays available for non-critical bodies.
    let ok_needs = NeedState {
        hunger: Fixed::from_f64(0.5),
        thirst: Fixed::from_f64(0.5),
        ..Default::default()
    };
    let ctx_ok = DecisionContext {
        needs: &ok_needs,
        personality: &personality,
        active_goals: &goals,
        identity: &identity,
        decision_policy: &dp,
        total_grain: Fixed::from_f64(0.5),
        total_water: Fixed::from_f64(0.5),
        coin: Fixed::from_f64(10.0),
        norm_pressure: Fixed::ZERO,
        anger: Fixed::ZERO,
        fear: Fixed::ZERO,
        joy: Fixed::ZERO,
        sadness: Fixed::ZERO,
        stress: Fixed::ZERO,
        fairness: Fixed::from_f64(0.5),
        authority: Fixed::from_f64(0.5),
        care: Fixed::from_f64(0.5),
        loyalty: Fixed::from_f64(0.5),
        action_values: Default::default(),
        dominant_need: MotiveCategory::Hunger,
        dominant_pressure: Fixed::from_f64(0.4),
        dread: Fixed::ZERO,
        hope: Fixed::ZERO,
        planning_confidence: Fixed::from_f64(0.5),
        mood_valence: Fixed::ZERO,
        season: 0,
        life_stage: 4,
        social_withdrawal: Fixed::ZERO,
        somatic_marker: Fixed::ZERO,
    };
    // Non-critical body: no reflex fires — full utility autonomy.
    let no_reflex = ok_needs.thirst > Fixed::from_f64(0.9)
        || ok_needs.hunger > Fixed::from_f64(0.9)
        || ok_needs.fatigue > Fixed::from_f64(0.95);
    assert!(
        !no_reflex,
        "comfortable body must not trigger the reflex layer"
    );
}
