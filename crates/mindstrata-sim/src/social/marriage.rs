//! Marriage system — PairBond, courtship stages, marriage as institution.
//!
//! Architecture §10.4-§10.5: Marriage is not just a relationship label.
//! It is an institution that affects inheritance, household economics,
//! faction alliances, legitimacy of children, status, sexual norms,
//! religious standing, and kin networks.
//!
//! ```text
//! Romantic Stages:
//!   Awareness → Attraction → Flirtation → Courtship → Exclusivity
//!   → Betrothal → Marriage → Household Formation → Parenthood
//!   → Stabilization/Strain → Reconciliation/Separation/Widowhood
//! ```
//!
//! Marriage dissolution: death, abandonment, annulment, divorce, exile, religious sanction.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Stage of a romantic relationship.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RomanticStage {
    /// No awareness of each other.
    Unaware,
    /// Aware of each other's existence.
    Awareness,
    /// Initial attraction detected.
    Attraction,
    /// Subtle signaling of interest.
    Flirtation,
    /// Active courtship — pursuing the relationship.
    ActiveCourtship,
    /// Testing whether interest is mutual.
    TestingReciprocity,
    /// Social validation — gossip, community awareness.
    SocialValidation,
    /// Mutually agreed exclusivity.
    Exclusivity,
    /// Formal betrothal / engagement.
    Betrothal,
    /// Married / formally bonded.
    Married,
    /// Household formed around the couple.
    HouseholdFormed,
    /// Children present — parenthood phase.
    Parenthood,
    /// Stable partnership with strain accumulating.
    Stabilization,
    /// High strain — reconciliation or separation possible.
    Strain,
}

impl RomanticStage {
    /// Advance to the next stage if conditions are met.
    pub fn next_positive(&self) -> Option<Self> {
        match self {
            Self::Unaware => Some(Self::Awareness),
            Self::Awareness => Some(Self::Attraction),
            Self::Attraction => Some(Self::Flirtation),
            Self::Flirtation => Some(Self::ActiveCourtship),
            Self::ActiveCourtship => Some(Self::TestingReciprocity),
            Self::TestingReciprocity => Some(Self::SocialValidation),
            Self::SocialValidation => Some(Self::Exclusivity),
            Self::Exclusivity => Some(Self::Betrothal),
            Self::Betrothal => Some(Self::Married),
            Self::Married => Some(Self::HouseholdFormed),
            Self::HouseholdFormed => Some(Self::Parenthood),
            Self::Parenthood => Some(Self::Stabilization),
            Self::Stabilization => Some(Self::Strain),
            Self::Strain => None, // terminal — branches to reconciliation or separation
        }
    }

    /// Get the base trust for this stage.
    pub fn base_trust(&self) -> Fixed {
        match self {
            Self::Unaware => Fixed::ZERO,
            Self::Awareness => Fixed::from_f64(0.1),
            Self::Attraction => Fixed::from_f64(0.2),
            Self::Flirtation => Fixed::from_f64(0.3),
            Self::ActiveCourtship => Fixed::from_f64(0.35),
            Self::TestingReciprocity => Fixed::from_f64(0.4),
            Self::SocialValidation => Fixed::from_f64(0.45),
            Self::Exclusivity => Fixed::from_f64(0.55),
            Self::Betrothal => Fixed::from_f64(0.65),
            Self::Married => Fixed::from_f64(0.75),
            Self::HouseholdFormed => Fixed::from_f64(0.8),
            Self::Parenthood => Fixed::from_f64(0.85),
            Self::Stabilization => Fixed::from_f64(0.7),
            Self::Strain => Fixed::from_f64(0.4),
        }
    }
}

/// Type of marriage arrangement.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MarriageType {
    /// Arranged by families for alliance.
    Arranged,
    /// Chose each other, socially approved.
    Chosen,
    /// Quick ceremony due to pregnancy.
    Shotgun,
    /// Second marriage after widowhood or divorce.
    Remarriage,
}

/// A PairBond — the core romantic/sexual relationship between two agents.
///
/// Architecture §10.4: Pair-bonding emerges from repeated positive interaction,
/// intimacy, exclusivity, shared vulnerability, social recognition, cohabitation,
/// children, and economic cooperation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PairBond {
    /// Index of partner A.
    pub partner_a: usize,
    /// Index of partner B.
    pub partner_b: usize,
    /// Overall bond strength (0–1).
    pub bond_strength: Fixed,
    /// How exclusive the relationship is (0 = open, 1 = fully exclusive).
    pub exclusivity: Fixed,
    /// Public recognition of the bond (0 = secret, 1 = widely known).
    pub public_recognition: Fixed,
    /// Emotional intimacy level.
    pub emotional_intimacy: Fixed,
    /// Economic cooperation level.
    pub economic_cooperation: Fixed,
    /// Shared children (agent indices).
    pub shared_children: Vec<usize>,
    /// Current strain on the relationship.
    pub strain: Fixed,
    /// Accumulated jealousy load.
    pub jealousy_load: Fixed,
    /// Current romantic stage.
    pub stage: RomanticStage,
    /// Tick when the bond was formed.
    pub formed_tick: u64,
    /// Tick of last positive interaction within the bond.
    pub last_positive_tick: u64,
    /// Tick of last negative interaction within the bond.
    pub last_negative_tick: u64,
}

impl PairBond {
    /// Create a new pair bond between two agents.
    pub fn new(partner_a: usize, partner_b: usize, tick: u64) -> Self {
        Self {
            partner_a,
            partner_b,
            bond_strength: Fixed::from_f64(0.3),
            exclusivity: Fixed::from_f64(0.5),
            public_recognition: Fixed::ZERO,
            emotional_intimacy: Fixed::from_f64(0.2),
            economic_cooperation: Fixed::ZERO,
            shared_children: Vec::new(),
            strain: Fixed::ZERO,
            jealousy_load: Fixed::ZERO,
            stage: RomanticStage::Attraction,
            formed_tick: tick,
            last_positive_tick: tick,
            last_negative_tick: tick,
        }
    }

    /// Record a positive interaction within the bond.
    pub fn record_positive(&mut self, tick: u64, magnitude: Fixed) {
        self.bond_strength = (self.bond_strength + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.emotional_intimacy = (self.emotional_intimacy + magnitude * Fixed::from_f64(0.015)).clamp_01();
        self.strain = (self.strain - magnitude * Fixed::from_f64(0.01)).max(Fixed::ZERO);
        self.last_positive_tick = tick;
    }

    /// Record a negative interaction within the bond.
    pub fn record_negative(&mut self, tick: u64, magnitude: Fixed) {
        self.bond_strength = (self.bond_strength - magnitude * Fixed::from_f64(0.03)).max(Fixed::ZERO);
        self.strain = (self.strain + magnitude * Fixed::from_f64(0.025)).clamp_01();
        self.last_negative_tick = tick;
    }

    /// Daily update — decay strain and jealousy, stabilize bond.
    pub fn daily_update(&mut self) {
        // Strain decays slowly
        self.strain = (self.strain * Fixed::from_f64(0.98)).max(Fixed::ZERO);
        // Jealousy decays slowly
        self.jealousy_load = (self.jealousy_load * Fixed::from_f64(0.97)).max(Fixed::ZERO);
        // Bond slowly converges toward 0.5 only when below it (prevent regression of strong bonds)
        if self.bond_strength < Fixed::from_f64(0.5) {
            let drift = (Fixed::from_f64(0.5) - self.bond_strength) * Fixed::from_f64(0.002);
            self.bond_strength = (self.bond_strength + drift).clamp_01();
        }
    }

    /// Check if the bond should dissolve.
    pub fn should_dissolve(&self) -> bool {
        self.bond_strength < Fixed::from_f64(0.1) && self.strain > Fixed::from_f64(0.8)
    }
}

/// Outcome of a marriage dissolution event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DissolutionCause {
    /// One partner died.
    Death(usize),
    /// Voluntary separation.
    Separation,
    /// Formal annulment (religious/institutional).
    Annulment,
    /// Exile of one partner.
    Exile(usize),
}

/// A Marriage — an institutional bond between agents.
///
/// Architecture §10.5: Marriage affects inheritance, household economics,
/// faction alliances, legitimacy of children, status, sexual norms,
/// religious standing, and kin networks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Marriage {
    /// Index of partner A.
    pub partner_a: usize,
    /// Index of partner B.
    pub partner_b: usize,
    /// Type of marriage arrangement.
    pub marriage_type: MarriageType,
    /// Legitimacy of the marriage in community eyes (0–1).
    pub legitimacy: Fixed,
    /// Household index (if a household has been formed).
    pub household_id: Option<usize>,
    /// Children produced by this marriage.
    pub children: Vec<usize>,
    /// Community recognition (0 = unknown, 1 = universally recognized).
    pub community_recognition: Fixed,
    /// Religious sanction (0 = none, 1 = blessed by temple).
    pub religious_sanction: Fixed,
    /// Current strain on the marriage.
    pub strain: Fixed,
    /// Tick when the marriage was formed.
    pub formed_tick: u64,
    /// Whether the marriage is currently active.
    pub active: bool,
}

impl Marriage {
    /// Create a new marriage.
    pub fn new(
        partner_a: usize,
        partner_b: usize,
        marriage_type: MarriageType,
        tick: u64,
    ) -> Self {
        Self {
            partner_a,
            partner_b,
            marriage_type,
            legitimacy: Fixed::from_f64(0.6),
            household_id: None,
            children: Vec::new(),
            community_recognition: Fixed::from_f64(0.3),
            religious_sanction: Fixed::ZERO,
            strain: Fixed::ZERO,
            formed_tick: tick,
            active: true,
        }
    }

    /// Daily update — decay strain, stabilize recognition.
    pub fn daily_update(&mut self) {
        self.strain = (self.strain * Fixed::from_f64(0.98)).max(Fixed::ZERO);
        // Recognition slowly grows if active
        if self.active {
            self.community_recognition =
                (self.community_recognition + Fixed::from_f64(0.001)).clamp_01();
        }
    }

    /// Dissolve the marriage.
    pub fn dissolve(&mut self, cause: &DissolutionCause) {
        self.active = false;
        self.strain = Fixed::ONE;
        match cause {
            DissolutionCause::Death(_) => {
                // Legitimacy preserved — community respects widowhood
                self.legitimacy = (self.legitimacy * Fixed::from_f64(0.8)).clamp_01();
            }
            DissolutionCause::Separation => {
                // Legitimacy damaged
                self.legitimacy = (self.legitimacy - Fixed::from_f64(0.2)).max(Fixed::ZERO);
            }
            DissolutionCause::Annulment => {
                // Religious sanction revoked
                self.religious_sanction = Fixed::ZERO;
                self.legitimacy = (self.legitimacy - Fixed::from_f64(0.3)).max(Fixed::ZERO);
            }
            DissolutionCause::Exile(_) => {
                self.legitimacy = (self.legitimacy * Fixed::from_f64(0.5)).clamp_01();
            }
        }
    }
}

/// Registry of all pair bonds and marriages in the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MarriageRegistry {
    /// All active pair bonds.
    pub pair_bonds: Vec<PairBond>,
    /// All marriages (active and dissolved).
    pub marriages: Vec<Marriage>,
}

impl MarriageRegistry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            pair_bonds: Vec::new(),
            marriages: Vec::new(),
        }
    }

    /// Find a pair bond between two agents (either direction).
    pub fn find_bond(&self, a: usize, b: usize) -> Option<&PairBond> {
        self.pair_bonds.iter().find(|pb| {
            (pb.partner_a == a && pb.partner_b == b) || (pb.partner_a == b && pb.partner_b == a)
        })
    }

    /// Find an active marriage between two agents.
    pub fn find_marriage(&self, a: usize, b: usize) -> Option<&Marriage> {
        self.marriages.iter().find(|m| {
            m.active
                && ((m.partner_a == a && m.partner_b == b)
                    || (m.partner_a == b && m.partner_b == a))
        })
    }

    /// Daily update for all bonds and marriages.
    pub fn daily_update(&mut self) {
        for bond in &mut self.pair_bonds {
            bond.daily_update();
        }
        for marriage in &mut self.marriages {
            if marriage.active {
                marriage.daily_update();
            }
        }
        // Remove dissolved pair bonds where the bond has fully decayed
        self.pair_bonds
            .retain(|pb| pb.bond_strength > Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn romantic_stage_advances() {
        assert_eq!(
            RomanticStage::Unaware.next_positive(),
            Some(RomanticStage::Awareness)
        );
        assert_eq!(
            RomanticStage::Married.next_positive(),
            Some(RomanticStage::HouseholdFormed)
        );
        assert_eq!(RomanticStage::Strain.next_positive(), None);
    }

    #[test]
    fn pair_bond_positive_increases_strength() {
        let mut bond = PairBond::new(0, 1, 0);
        let initial = bond.bond_strength;
        bond.record_positive(10, Fixed::from_f64(0.5));
        assert!(bond.bond_strength > initial);
    }

    #[test]
    fn pair_bond_negative_increases_strain() {
        let mut bond = PairBond::new(0, 1, 0);
        bond.record_negative(10, Fixed::from_f64(0.5));
        assert!(bond.strain > Fixed::ZERO);
    }

    #[test]
    fn marriage_dissolution_sets_inactive() {
        let mut marriage = Marriage::new(0, 1, MarriageType::Chosen, 100);
        assert!(marriage.active);
        marriage.dissolve(&DissolutionCause::Separation);
        assert!(!marriage.active);
    }

    #[test]
    fn marriage_registry_find_bond() {
        let mut registry = MarriageRegistry::new();
        registry.pair_bonds.push(PairBond::new(0, 1, 0));
        assert!(registry.find_bond(0, 1).is_some());
        assert!(registry.find_bond(1, 0).is_some());
        assert!(registry.find_bond(0, 2).is_none());
    }
}
