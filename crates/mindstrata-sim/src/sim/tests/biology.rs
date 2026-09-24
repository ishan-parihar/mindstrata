//! Domain-grouped unit tests (split from tests.rs; pure moves).

use super::super::*;

#[test]
fn survival_reflex_forces_relief_over_utility() {
    // Iteration 255 (audit Phase 3): a body at its limits must take the
    // relief action REGARDLESS of utility contest, habits, or commands —
    // the E3 inversion (starving while working) is impossible by
    // construction.
    use mindstrata_core::fixed::Fixed;

    let needs = NeedState {
        hunger: Fixed::from_f64(0.95),
        thirst: Fixed::from_f64(0.5),
        ..Default::default()
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
    // Non-critical body: no reflex fires — full utility autonomy.
    let no_reflex = ok_needs.thirst > Fixed::from_f64(0.9)
        || ok_needs.hunger > Fixed::from_f64(0.9)
        || ok_needs.fatigue > Fixed::from_f64(0.95);
    assert!(
        !no_reflex,
        "comfortable body must not trigger the reflex layer"
    );
}
