//! Conflict types shared across crates.
//!
//! `ConflictKind` lives in core so that `SimEvent::ConflictOccurred` can carry
//! the enum directly instead of a heap-allocated `String`.

use crate::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Types of conflict interactions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ConflictKind {
    /// Verbal threat — fear-inducing but no physical harm.
    Threat,
    /// Intimidation — repeated threats that erode will.
    Intimidation,
    /// Physical violence — direct harm.
    Violence,
    /// Organized combat — between factions or groups.
    Combat,
    /// Feud — ongoing conflict between two parties.
    Feud,
    /// Moral panic event — community-level fear cascade.
    MoralPanic,
    /// Revolution — overthrow of institutional authority.
    Revolution,
}

impl ConflictKind {
    /// Base injury severity from this conflict type.
    pub fn injury_severity(self) -> Fixed {
        match self {
            Self::Threat | Self::MoralPanic | Self::Revolution => Fixed::ZERO,
            Self::Intimidation => Fixed::from_f64(0.05),
            // Iteration 185 (P5 emergent re-audit): 0.3 → 0.12. At 0.3 a
            // single violence hit did ~1/3 of a healthy agent's health and
            // ~3 hits killed — with the pre-fix escalation rate that made
            // beatings routinely lethal and every calm seed's population
            // collapse (5/6 seeds ≤4/12 alive @20K, 100% of deaths =
            // Violence). 0.12 makes a fight hurt (~7 hits to kill) while
            // the 0.001/tick resting recovery (≈0.1/day) lets most injuries
            // heal between conflicts — violence is now trauma/drama, with
            // death reserved for sustained, repeated aggression.
            Self::Violence => Fixed::from_f64(0.12),
            Self::Combat => Fixed::from_f64(0.5),
            Self::Feud => Fixed::from_f64(0.2),
        }
    }

    /// Trauma accumulation rate per incident.
    pub fn trauma_rate(self) -> Fixed {
        match self {
            Self::Threat => Fixed::from_f64(0.02),
            Self::Intimidation => Fixed::from_f64(0.05),
            Self::Violence => Fixed::from_f64(0.15),
            Self::Combat => Fixed::from_f64(0.25),
            Self::Feud => Fixed::from_f64(0.08),
            Self::MoralPanic => Fixed::from_f64(0.1),
            Self::Revolution => Fixed::from_f64(0.2),
        }
    }

    /// Fear induced in the target.
    pub fn fear_induction(self) -> Fixed {
        match self {
            Self::Threat => Fixed::from_f64(0.15),
            Self::Intimidation => Fixed::from_f64(0.25),
            Self::Violence => Fixed::from_f64(0.4),
            Self::Combat => Fixed::from_f64(0.6),
            Self::Feud => Fixed::from_f64(0.2),
            Self::MoralPanic => Fixed::from_f64(0.5),
            Self::Revolution => Fixed::from_f64(0.6),
        }
    }
}

impl std::fmt::Display for ConflictKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Threat => write!(f, "Threat"),
            Self::Intimidation => write!(f, "Intimidation"),
            Self::Violence => write!(f, "Violence"),
            Self::Combat => write!(f, "Combat"),
            Self::Feud => write!(f, "Feud"),
            Self::MoralPanic => write!(f, "MoralPanic"),
            Self::Revolution => write!(f, "Revolution"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn injury_severity_ordering() {
        assert!(ConflictKind::Threat.injury_severity() < ConflictKind::Violence.injury_severity());
        assert!(ConflictKind::Violence.injury_severity() < ConflictKind::Combat.injury_severity());
    }

    #[test]
    fn non_physical_conflicts_have_zero_injury() {
        assert_eq!(ConflictKind::Threat.injury_severity(), Fixed::ZERO);
        assert_eq!(ConflictKind::MoralPanic.injury_severity(), Fixed::ZERO);
        assert_eq!(ConflictKind::Revolution.injury_severity(), Fixed::ZERO);
    }

    #[test]
    fn display_matches_variant_names() {
        assert_eq!(ConflictKind::Threat.to_string(), "Threat");
        assert_eq!(ConflictKind::Revolution.to_string(), "Revolution");
    }
}
