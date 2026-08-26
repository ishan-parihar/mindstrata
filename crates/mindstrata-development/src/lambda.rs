//! Lambda gating (WP-D): whether resonated pressure actually enters field
//! dynamics. Completes the pure engine — placement (WP-B) produces
//! coordinates, resonance + pathology (WP-C) process catalysts, gating
//! decides admission.
//!
//! Gating is a deterministic threshold transform: sub-threshold pressure
//! admits exactly zero (no partial leakage), supra-threshold pressure maps
//! linearly from the threshold band into [0,1]. Zero-at-zero holds by
//! construction. Threshold is CALIBRATION-PENDING(AP3).

use crate::canon::FIELD_UPTAKE_QUANTUM_F64;

/// Admission gate over resonated pressure.
#[derive(Debug, Clone, Copy)]
pub struct Gate {
    /// Minimum resonated pressure that registers at all, in (0, 1].
    pub engagement_threshold: f64,
}

impl Gate {
    /// Placeholder gate — threshold pending probe evidence
    /// (i<iter>_gate_admission).
    #[must_use]
    pub const fn pending() -> Self {
        Self {
            engagement_threshold: 0.05,
        }
    }

    /// Transform raw resonated pressure into admitted pressure.
    ///
    /// - `p < threshold` ⇒ 0.0 (dead-zone, no leakage)
    /// - otherwise ⇒ `(p - t) / (1 - t)` renormalized into (0, 1]
    #[must_use]
    pub fn admit(&self, p: f64) -> f64 {
        let p = if p.is_finite() {
            p.clamp(0.0, 1.0)
        } else {
            0.0
        };
        if p <= self.engagement_threshold {
            return 0.0;
        }
        (p - self.engagement_threshold) / (1.0 - self.engagement_threshold)
    }

    /// Sub-resolution guard: admissions below the Fixed quantum are floored
    /// to zero here so downstream quantize-once persistence never records
    /// phantom deltas (IC-3 numeric discipline).
    #[must_use]
    pub fn admit_quantized(&self, p: f64) -> f64 {
        let admitted = self.admit(p);
        if admitted < FIELD_UPTAKE_QUANTUM_F64 {
            0.0
        } else {
            admitted
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dead_zone_admits_exactly_zero() {
        let g = Gate::pending();
        assert_eq!(g.admit(0.0), 0.0);
        assert_eq!(g.admit(0.04), 0.0);
        assert_eq!(g.admit(g.engagement_threshold), 0.0);
    }

    #[test]
    fn supra_threshold_renormalizes_to_full_span() {
        let g = Gate::pending();
        assert!((g.admit(1.0) - 1.0).abs() < 1e-12);
        // Midpoint of the remaining span maps to ~half.
        let mid = g.engagement_threshold + (1.0 - g.engagement_threshold) / 2.0;
        assert!((g.admit(mid) - 0.5).abs() < 1e-12);
    }

    #[test]
    fn zero_at_zero_holds_through_the_gate() {
        let g = Gate::pending();
        assert_eq!(g.admit_quantized(0.0), 0.0);
    }

    #[test]
    fn non_finite_pressure_admits_zero() {
        let g = Gate::pending();
        assert_eq!(g.admit(f64::NAN), 0.0);
        assert_eq!(g.admit(f64::INFINITY), 0.0);
    }

    #[test]
    fn quantum_guard_floors_phantom_deltas() {
        let g = Gate::pending();
        // Just above threshold: admitted value is tiny; below the 1e-4
        // quantum it must floor to zero rather than record a phantom.
        let tiny = g.admit(0.05005);
        assert!(tiny > 0.0 && tiny < 1e-4, "tiny={tiny}");
        assert_eq!(g.admit_quantized(0.05005), 0.0);
    }

    #[test]
    fn gating_is_monotone() {
        let g = Gate::pending();
        let mut prev = -1.0;
        for i in 0..=100 {
            let p = i as f64 / 100.0;
            let a = g.admit(p);
            assert!(a >= prev, "gate regressed at p={p}");
            prev = a;
        }
    }
}
