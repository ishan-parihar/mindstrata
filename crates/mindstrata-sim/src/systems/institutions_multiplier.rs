//! Institutions read-side multiplier (SIM 4.25 groundwork → WP-J live).
//!
//! WP-J (AP3 waves, Era IV): "Institution behavior parameters become
//! functions of governance/economic-systems line stages (read-side
//! multipliers, midpoint-neutral)." The groundwork type below stays inert
//! (reserved for per-agent work/rest axes); the live coupling is
//! [`compliance_multiplier`]: the §12.3 institution-morale compliance
//! channel scales with the village's governance/economic-systems collective
//! stage — a village whose governance line has differentiated (band III+,
//! stage ≥ 4.0 per the WP-I tetra-arising bands) transmits institutional
//! morale into member compliance more effectively.
//!
//! Zero-blast is structural, not calibrated: governance/economic-systems sit
//! at exactly 1.0/2.0/3.0 at the 2K/5K/10K pinned horizons (i280 probe,
//! seed 42) — all below the 4.0 gate — so pinned horizons get the identity
//! multiplier unconditionally. At 20K (stage 6) the ramp is live.

use mindstrata_core::fixed::Fixed;

/// Stage where the compliance ramp begins (WP-I band III, "differentiated").
const COUPLING_GATE_STAGE: f64 = 4.0;

/// Ramp slope per stage above the gate, capped at 6 stages of headroom.
/// At the i280-measured 20K stage (6.0) the multiplier is 1.0 + 2×0.05
/// = 1.10 — a 10% deepening of the existing morale channel, nudge-class
/// (dread/hope scale), not a reordering lever.
const RAMP_SLOPE: f64 = 0.05;

/// WP-J read-side compliance multiplier from the governance and
/// economic-systems collective line stages.
///
/// Identity (1.0) at or below the band-III gate; linear ramp above, capped.
/// Both stages enter by mean — no single line can drive the coupling alone.
/// Computed in f64 and quantized once (§5 fixed-4 truncation rule).
pub fn compliance_multiplier(governance_stage: f64, economic_systems_stage: f64) -> Fixed {
    let mean_stage = f64::midpoint(governance_stage, economic_systems_stage);
    if mean_stage <= COUPLING_GATE_STAGE {
        return Fixed::ONE;
    }
    let ramp = (mean_stage - COUPLING_GATE_STAGE).min(6.0) * RAMP_SLOPE;
    Fixed::from_f64(1.0 + ramp)
}

/// Work-utility bonus for institution members from a live institution
/// (morale > 0), scaled by the WP-J compliance multiplier.
///
/// WP-J read-side channel #2 (i280): the §12.3 morale→compliance surface
/// is provably dead at N=12 (zero NormViolated across 6 seeds × 5K/20K,
/// i280 violations sweep) — a live multiplier on a dead channel has no
/// observable effect. The Work axis is the live read-side surface instead:
/// a functioning institution (positive morale) makes provisioning more
/// attractive to its members, scaled by the governance/economic band (a
/// village whose governance line has differentiated — band III+, stage ≥
/// 4.0 — transmits institutional vitality into member provisioning).
///
/// Zero below the band-III gate (all pinned horizons: goldens at 1000/
/// 4320, snapshot at 10K — governance sits at 1.0/2.0/3.0 there, i280
/// probe seed 42), so pinned horizons are byte-identical. At the 20K
/// live horizon (mult 1.10, morale ~0.11) the bonus is ~0.012 — dread/
/// hope-class nudge, not a reordering lever. Computed in f64, quantized
/// once (§5 fixed-4 truncation rule).
pub fn member_work_bonus(institution_morale: Fixed, multiplier: Fixed) -> Fixed {
    if multiplier <= Fixed::ONE {
        return Fixed::ZERO;
    }
    Fixed::from_f64(institution_morale.to_f64() * 0.1 * multiplier.to_f64())
}

/// Inert institutions multiplier — neutral at `1.0` (identity), `0.0` delta.
///
/// Shape mirrors `development` stage gating: `neutral()` is the zero-at-zero
/// anchor; future `apply` will be `value * multiplier` with `multiplier == 1.0`
/// at neutral institutions.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct InstitutionsMultiplier {
    /// Work multiplier (neutral `1.0`).
    pub work: Fixed,
    /// Rest multiplier (neutral `1.0`).
    pub rest: Fixed,
}

impl InstitutionsMultiplier {
    /// Neutral — identity at zero institutions signal.
    pub fn neutral() -> Self {
        Self {
            work: Fixed::ONE,
            rest: Fixed::ONE,
        }
    }

    /// True when both axes are identity (no institutions pressure).
    pub fn is_neutral(self) -> bool {
        self.work == Fixed::ONE && self.rest == Fixed::ONE
    }
}

impl Default for InstitutionsMultiplier {
    fn default() -> Self {
        Self::neutral()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compliance_multiplier_identity_below_band_iii() {
        // Pinned horizons (i280 probe, seed 42): stages 1/2/3 — identity.
        for stage in [1.0_f64, 2.0, 3.0, 4.0] {
            assert_eq!(
                compliance_multiplier(stage, stage),
                Fixed::ONE,
                "stage {stage} must be identity"
            );
        }
        // Asymmetric stages average: (6+2)/2 = 4 → still identity.
        assert_eq!(compliance_multiplier(6.0, 2.0), Fixed::ONE);
    }

    #[test]
    fn compliance_multiplier_ramps_above_band_iii() {
        // i280-measured 20K stage: 6.0 → 1.0 + 2×0.05 = 1.10.
        let m = compliance_multiplier(6.0, 6.0);
        assert_eq!(m, Fixed::from_f64(1.10));
        // Cap: 20 stages of headroom → same as 6 stages.
        assert_eq!(compliance_multiplier(24.0, 24.0), Fixed::from_f64(1.30));
        // Monotone between gate and cap.
        assert!(compliance_multiplier(5.0, 5.0) > Fixed::ONE);
        assert!(compliance_multiplier(6.0, 6.0) > compliance_multiplier(5.0, 5.0));
    }

    #[test]
    fn neutral_is_identity() {
        let m = InstitutionsMultiplier::neutral();
        assert!(m.is_neutral());
        assert_eq!(m.work, Fixed::ONE);
        assert_eq!(m.rest, Fixed::ONE);
    }

    #[test]
    fn default_is_neutral() {
        assert!(InstitutionsMultiplier::default().is_neutral());
    }

    #[test]
    fn member_work_bonus_zero_below_gate() {
        // Identity multiplier (at/below band III) → zero bonus regardless
        // of morale: pinned horizons stay byte-identical.
        assert_eq!(
            member_work_bonus(Fixed::from_f64(0.11), Fixed::ONE),
            Fixed::ZERO
        );
    }

    #[test]
    fn member_work_bonus_scales_with_morale_and_mult() {
        let m = compliance_multiplier(6.0, 6.0); // 1.10 at i280's 20K stage
        let b = member_work_bonus(Fixed::from_f64(0.11), m);
        // 0.11 * 0.1 * 1.10 = 0.0121 — dread-class nudge magnitude.
        assert!((b.to_f64() - 0.0121).abs() < 1e-6);
        // Monotone in morale; zero morale → zero bonus (zero-at-zero).
        assert!(member_work_bonus(Fixed::from_f64(0.5), m) > b);
        assert_eq!(member_work_bonus(Fixed::ZERO, m), Fixed::ZERO);
    }
}
