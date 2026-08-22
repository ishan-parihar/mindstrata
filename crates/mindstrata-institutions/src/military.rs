//! §5 (Iteration 153): Military system — organized defense and
//! conscription.
//!
//! Individual violence (§conflict.rs: feuds, injuries, trauma, casualties)
//! and off-map raids (§diplomacy.rs: caravans and raids from neighboring
//! settlements) are live; what the plan row ("Military system — organized
//! warfare, conscription") calls for is an *organized* military: a
//! conscripted militia, drilled yearly into collective readiness, that
//! genuinely dampens hostile raids.
//!
//! A world opts in by placing a `SiteKind::Barracks` — no default world
//! does, so every calibrated window (golden, snapshots) is untouched. The
//! pass is fully deterministic (fixed conscription and drill rules, no RNG
//! drawn anywhere), so it can never perturb any RNG stream, even in a
//! world with a barracks.
//!
//! Known limitation: roster membership is keyed by agent slot, and the
//! generational replacement path fills a dead agent's slot in place without
//! clearing its record — so in a barracks-bearing long run a replacement is
//! born a conscript with a stale `enlisted_since`. Moot in every calibrated
//! window (no barracks is ever placed there); a follow-up could hook the
//! replacement path to reset the slot's record.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// The militia musters and drills on the yearly 4320-tick cadence (the
/// ritual cadence shared with technology, diplomacy, schools, theology).
pub const MILITARY_CADENCE: u64 = 4320;

/// Conscription age window: adults 18–39 (40+ are elders exempt from the
/// muster).
pub const CONSCRIPT_AGE_MIN: f64 = 18.0;
/// Const. (doc added at S3 extraction)
pub const CONSCRIPT_AGE_MAX: f64 = 40.0;

/// The largest militia a single barracks supports.
pub const MUSTER_CAP: usize = 8;

/// Readiness gained per drilled member (as a share of the cap) and the
/// yearly decay when the militia stands down.
pub const DRILL_GAIN_PER_MEMBER: f64 = 0.15;
/// Const. (doc added at S3 extraction)
pub const DRILL_YEARLY_DECAY: f64 = 0.05;

/// How strongly readiness dampens raid grain loss (0.5 → up to −50%).
pub const RAID_READINESS_DAMPENING: f64 = 0.5;

/// One conscript's military record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MilitiaMember {
    /// Tick the agent was conscripted.
    pub enlisted_since: u64,
    /// The dominance that earned them a place in the muster.
    pub dominance_at_enlistment: Fixed,
}

/// World-level military state: the roster, collective readiness, tallies.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MilitaryRegistry {
    /// Per-agent militia membership, indexed by agent slot.
    pub roster: Vec<Option<MilitiaMember>>,
    /// Collective defensive readiness in [0, 1] — built by drills, decaying
    /// yearly, dampening raid losses.
    pub readiness: Fixed,
    /// Total conscriptions.
    pub conscripts: u64,
    /// Muster passes that recruited at least one conscript.
    pub musters: u64,
    /// Drill passes held with a non-empty militia.
    pub drills: u64,
}

impl MilitaryRegistry {
    /// Create an empty registry (fresh on both simulation constructors —
    /// the `weather` / `legal` / `diplomacy` / `school` / `theology`
    /// precedent: no calibrated window places a barracks, so a fresh
    /// restore is faithful).
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Whether the military system is dormant (no one mustered, no
    /// readiness).
    #[must_use]
    pub fn is_dormant(&self) -> bool {
        self.roster.iter().all(Option::is_none) && self.readiness <= Fixed::ZERO
    }

    /// Number of conscripts on the roster.
    #[must_use]
    pub fn militia_size(&self) -> u64 {
        self.roster.iter().filter(|m| m.is_some()).count() as u64
    }
}

/// Whether an agent is conscription-eligible: a non-elder adult in the
/// 18–39 window.
#[must_use]
pub fn conscription_eligible(age: Fixed) -> bool {
    age >= Fixed::from_f64(CONSCRIPT_AGE_MIN) && age < Fixed::from_f64(CONSCRIPT_AGE_MAX)
}

/// One year of militia training: readiness rises with the drilled share of
/// the barracks cap and decays every year, clamped to [0, 1].
#[must_use]
pub fn drill_readiness(current: Fixed, attenders: usize, cap: usize) -> Fixed {
    let share = Fixed::from_f64(attenders as f64 / cap.max(1) as f64);
    (current + share * Fixed::from_f64(DRILL_GAIN_PER_MEMBER) - Fixed::from_f64(DRILL_YEARLY_DECAY))
        .clamp_01()
}

/// Raid loss multiplier: 1.0 at zero readiness, down to (1 − dampening) at
/// full readiness.
#[must_use]
pub fn raid_loss_multiplier(readiness: Fixed) -> Fixed {
    (Fixed::ONE - readiness * Fixed::from_f64(RAID_READINESS_DAMPENING)).clamp_01()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn f(x: f64) -> Fixed {
        Fixed::from_f64(x)
    }

    #[test]
    fn new_registry_is_dormant() {
        let reg = MilitaryRegistry::new();
        assert!(reg.is_dormant());
        assert_eq!(reg.militia_size(), 0);
        assert_eq!(reg.conscripts, 0);
        assert_eq!(reg.drills, 0);
        assert_eq!(reg.readiness, Fixed::ZERO);
    }

    #[test]
    fn conscription_eligible_window_is_18_to_39() {
        assert!(
            !conscription_eligible(f(17.9)),
            "minors are not conscripted"
        );
        assert!(conscription_eligible(f(18.0)), "18 is eligible");
        assert!(conscription_eligible(f(39.9)), "39 is eligible");
        assert!(!conscription_eligible(f(40.0)), "40+ elders are exempt");
    }

    #[test]
    fn drill_readiness_rises_with_attendance_and_decays_when_empty() {
        // Full house: gain 0.15 − decay 0.05 = +0.10.
        let full = drill_readiness(Fixed::ZERO, 8, 8);
        assert!((full - f(0.1)).abs() < f(0.001), "got {}", full.to_f64());
        // Empty stands-down: −0.05, floored at zero.
        assert_eq!(drill_readiness(f(0.02), 0, 8), Fixed::ZERO);
        // Clamped at one.
        assert!(drill_readiness(f(0.98), 8, 8) <= Fixed::ONE);
    }

    #[test]
    fn raid_loss_multiplier_dampens_with_readiness() {
        assert_eq!(raid_loss_multiplier(Fixed::ZERO), Fixed::ONE);
        assert_eq!(raid_loss_multiplier(Fixed::ONE), f(0.5));
        assert!(raid_loss_multiplier(f(0.6)) < raid_loss_multiplier(f(0.2)));
    }

    #[test]
    fn militia_size_counts_the_roster() {
        let mut reg = MilitaryRegistry::new();
        reg.roster.resize(3, None);
        reg.roster[1] = Some(MilitiaMember {
            enlisted_since: 4320,
            dominance_at_enlistment: f(0.7),
        });
        assert_eq!(reg.militia_size(), 1);
        assert!(!reg.is_dormant());
    }
}
