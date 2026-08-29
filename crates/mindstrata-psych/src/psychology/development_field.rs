//! AP3 attractor-field state carried by every agent (DC-1 task 3.1).
//!
//! The per-person half of the development substrate: altitude shadows over
//! the registered line set plus the ratified 4-fold pathology field. The
//! pure engine that CONSUMES this state lives in `mindstrata-development`
//! (placement, dynamics, gating); this struct is inert data — nothing in
//! the tick pipeline reads it yet (task 3.2 wires the first consumer), so
//! golden runs stay byte-identical by the zero-at-zero law.
//!
//! Founder altitudes are drawn at the END of populate()'s local generator
//! sequence with order comments at the draw site (RNG discipline §5);
//! newborns inherit via [`DevelopmentFieldState::inherited`] (one draw per
//! line, blended mid-parent + noise) with pathology reset to neutral.

use mindstrata_development::dynamics::PathologyField;
use rand::Rng;
use serde::{Deserialize, Serialize};

/// Per-agent attractor-field state.
///
/// `Default` yields an EMPTY altitude vec with a neutral pathology field —
/// the serde(default) shape used when loading pre-3.1 snapshots (v12); the
/// inert field is re-sized on first consumption by the development pass.
#[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DevelopmentFieldState {
    /// Altitude shadow per registered line, index-aligned with the stable
    /// `mindstrata_development::line::all_lines()` order (frozen canon
    /// tables ⇒ order and count are build-stable).
    #[serde(default)]
    pub altitudes: Vec<f64>,
    /// Ratified 4-fold pathology quadrants; neutral at founder init and
    /// at birth.
    #[serde(default)]
    pub pathology: PathologyField,
}

impl DevelopmentFieldState {
    /// Fully neutral state for `line_count` registered lines — the newborn
    /// and test default (FR-023).
    #[must_use]
    pub fn neutral(line_count: usize) -> Self {
        Self {
            altitudes: vec![0.0; line_count],
            pathology: PathologyField::neutral(),
        }
    }

    /// Founder endowment: one U(0,1) altitude per registered line, drawn
    /// in registry order from the caller's generator. Callers must invoke
    /// this at a documented, fixed point in the seeding sequence.
    pub fn founder_drawn<R: Rng>(rng: &mut R, line_count: usize) -> Self {
        let altitudes = (0..line_count).map(|_| rng.random::<f64>()).collect();
        Self {
            altitudes,
            pathology: PathologyField::neutral(),
        }
    }

    /// Vertical transmission: blended mid-parent altitudes plus per-line
    /// noise, one RNG draw per trait in registry order (stream discipline
    /// §5, matching `Personality::inherit`). Pathology resets to neutral
    /// at birth — it is experiential, not inherited. Deterministic for a
    /// given `(mother, father, rng)` tuple.
    #[allow(
        clippy::manual_midpoint,
        reason = "midpoint via *0.5 keeps f64 precision explicit"
    )]
    pub fn inherited<R: Rng>(mother: &Self, father: Option<&Self>, rng: &mut R) -> Self {
        let n = mother.altitudes.len();
        let mut altitudes = Vec::with_capacity(n);
        for i in 0..n {
            let m = mother.altitudes[i];
            let f = father.map_or(m, |p| p.altitudes.get(i).copied().unwrap_or(m));
            let noise = (rng.random::<f64>() - 0.5) * 0.1;
            altitudes.push(((m + f) * 0.5 + noise).clamp(0.0, 1.0));
        }
        Self {
            altitudes,
            pathology: PathologyField::neutral(),
        }
    }

    /// True when every altitude sits at exact zero AND all pathology
    /// quadrants are neutral — the pre-consumption golden invariant.
    #[must_use]
    pub fn is_fully_neutral(&self) -> bool {
        self.altitudes.iter().all(|&a| a == 0.0) && self.pathology.is_neutral()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;

    #[test]
    fn neutral_state_is_fully_neutral_at_any_line_count() {
        assert!(DevelopmentFieldState::neutral(0).is_fully_neutral());
        assert!(DevelopmentFieldState::neutral(49).is_fully_neutral());
    }

    #[test]
    fn founder_drawn_spans_unit_interval_in_registry_order() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        let state = DevelopmentFieldState::founder_drawn(&mut rng, 49);
        assert_eq!(state.altitudes.len(), 49);
        assert!(state.altitudes.iter().all(|a| (0.0..=1.0).contains(a)));
        // Pathology stays neutral under altitude draws.
        assert!(state.pathology.is_neutral());
        // Determinism: same seed ⇒ identical endowment.
        let mut rng2 = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        let again = DevelopmentFieldState::founder_drawn(&mut rng2, 49);
        assert_eq!(state, again);
    }

    #[test]
    fn serde_round_trip_preserves_state() {
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(9);
        let state = DevelopmentFieldState::founder_drawn(&mut rng, 5);
        let bytes = serde_json::to_vec(&state).expect("serialize");
        let back: DevelopmentFieldState = serde_json::from_slice(&bytes).expect("deserialize");
        assert_eq!(back, state);
    }

    #[test]
    fn inherited_blends_parents_with_one_draw_per_trait() {
        let mother = DevelopmentFieldState {
            altitudes: vec![0.2, 0.8, 0.5],
            pathology: PathologyField::neutral(),
        };
        let father = DevelopmentFieldState {
            altitudes: vec![0.4, 0.6, 0.7],
            pathology: PathologyField::neutral(),
        };
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(11);
        let child = DevelopmentFieldState::inherited(&mother, Some(&father), &mut rng);
        assert_eq!(child.altitudes.len(), 3);
        // Mid-parent without noise would be [0.3, 0.7, 0.6]; noise ±0.05 keeps child near.
        for (c, mid) in child.altitudes.iter().zip([0.3, 0.7, 0.6]) {
            assert!((c - mid).abs() <= 0.06, "child {c} far from mid {mid}");
        }
        assert!(child.pathology.is_neutral());
    }

    #[test]
    fn inherited_is_deterministic_and_consumes_one_draw_per_line() {
        let mother = DevelopmentFieldState {
            altitudes: vec![0.1; 5],
            pathology: PathologyField::neutral(),
        };
        let father = DevelopmentFieldState {
            altitudes: vec![0.9; 5],
            pathology: PathologyField::neutral(),
        };
        let mut rng1 = rand_chacha::ChaCha8Rng::seed_from_u64(99);
        let mut rng2 = rand_chacha::ChaCha8Rng::seed_from_u64(99);
        let a = DevelopmentFieldState::inherited(&mother, Some(&father), &mut rng1);
        let b = DevelopmentFieldState::inherited(&mother, Some(&father), &mut rng2);
        assert_eq!(a, b, "same seed must give identical child");
        // Verify stream discipline: after 5 draws, the next draw is the 6th in the sequence.
        let next1: f64 = rng1.random();
        let mut rng3 = rand_chacha::ChaCha8Rng::seed_from_u64(99);
        for _ in 0..5 {
            let _: f64 = rng3.random();
        }
        let next3: f64 = rng3.random();
        assert_eq!(next1, next3, "must consume exactly one draw per line");
    }

    #[test]
    fn inherited_from_neutral_parents_stays_near_neutral() {
        let neutral = DevelopmentFieldState::neutral(4);
        let mut rng = rand_chacha::ChaCha8Rng::seed_from_u64(7);
        let child = DevelopmentFieldState::inherited(&neutral, Some(&neutral), &mut rng);
        // Noise ±0.05 keeps child within [0,0.05] when parents 0.
        assert!(child.altitudes.iter().all(|&v| v <= 0.06));
        assert!(child.pathology.is_neutral());
    }
}
