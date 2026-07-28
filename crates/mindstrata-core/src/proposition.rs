//! Formal proposition system for agent beliefs.
//!
//! Agents hold beliefs about propositions. Propositions are machine-processable
//! facts about the world that can be true or false with varying confidence.
//! This enables ideology, propaganda, rumor, and polarization to emerge.
//!
//! §22.10 of the architecture spec.

use crate::fixed::Fixed;
use crate::id::AgentId;
use serde::{Deserialize, Serialize};
use std::fmt;

/// A proposition that agents can believe.
///
/// Each variant represents a specific claim about the world.
/// Agents hold beliefs about these propositions with varying confidence.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Proposition {
    /// The market treats everyone fairly.
    MarketIsFair,
    /// The ruler/governing body is legitimate.
    RulerIsLegitimate,
    /// A specific neighbor can be trusted.
    NeighborTrustworthy { neighbor: AgentId },
    /// Foreigners/outside agents are dangerous.
    ForeignersDangerous,
    /// Hard work leads to wealth and prosperity.
    HardWorkLeadsToWealth,
    /// The current harvest will be good/bad.
    HarvestWillFail,
    /// The temple/religious institution is corrupt.
    TempleIsCorrupt,
    /// The council governing body protects the people.
    CouncilProtectsUs,
    /// Grain prices are too high (unfair).
    GrainPricesTooHigh,
    /// The guards/law enforcement are unjust.
    GuardsUnjust,
    /// Violence is sometimes necessary.
    ViolenceNecessary,
    /// Sharing resources is a moral duty.
    SharingIsDuty,
    /// The community is strong and will survive.
    CommunityStrong,
    /// Stranger agents are generally honest.
    StrangersHonest,
    /// The well water is clean and safe.
    WellWaterSafe,
}

impl Proposition {
    /// Get a human-readable description of this proposition.
    pub fn description(&self) -> &'static str {
        match self {
            Proposition::MarketIsFair => "The market is fair",
            Proposition::RulerIsLegitimate => "The ruler is legitimate",
            Proposition::NeighborTrustworthy { .. } => "Neighbor is trustworthy",
            Proposition::ForeignersDangerous => "Foreigners are dangerous",
            Proposition::HardWorkLeadsToWealth => "Hard work leads to wealth",
            Proposition::HarvestWillFail => "The harvest will fail",
            Proposition::TempleIsCorrupt => "The temple is corrupt",
            Proposition::CouncilProtectsUs => "The council protects us",
            Proposition::GrainPricesTooHigh => "Grain prices are too high",
            Proposition::GuardsUnjust => "The guards are unjust",
            Proposition::ViolenceNecessary => "Violence is sometimes necessary",
            Proposition::SharingIsDuty => "Sharing resources is a moral duty",
            Proposition::CommunityStrong => "The community is strong",
            Proposition::StrangersHonest => "Strangers are generally honest",
            Proposition::WellWaterSafe => "Well water is safe",
        }
    }

    /// Get the stable numeric ID for this proposition.
    /// Used for serialization and belief lookup.
    pub fn id(&self) -> u64 {
        match self {
            Proposition::MarketIsFair => 0,
            Proposition::RulerIsLegitimate => 1,
            Proposition::NeighborTrustworthy { .. } => 2,
            Proposition::ForeignersDangerous => 3,
            Proposition::HardWorkLeadsToWealth => 4,
            Proposition::HarvestWillFail => 5,
            Proposition::TempleIsCorrupt => 6,
            Proposition::CouncilProtectsUs => 7,
            Proposition::GrainPricesTooHigh => 8,
            Proposition::GuardsUnjust => 9,
            Proposition::ViolenceNecessary => 10,
            Proposition::SharingIsDuty => 11,
            Proposition::CommunityStrong => 12,
            Proposition::StrangersHonest => 13,
            Proposition::WellWaterSafe => 14,
        }
    }

    /// Get the base emotional charge for believing this proposition.
    /// Positive = emotionally positive belief, negative = negative.
    pub fn base_emotional_charge(&self) -> Fixed {
        match self {
            Proposition::MarketIsFair => Fixed::from_f64(0.2),
            Proposition::RulerIsLegitimate => Fixed::from_f64(0.1),
            Proposition::NeighborTrustworthy { .. } => Fixed::from_f64(0.3),
            Proposition::ForeignersDangerous => Fixed::from_f64(-0.4),
            Proposition::HardWorkLeadsToWealth => Fixed::from_f64(0.3),
            Proposition::HarvestWillFail => Fixed::from_f64(-0.5),
            Proposition::TempleIsCorrupt => Fixed::from_f64(-0.3),
            Proposition::CouncilProtectsUs => Fixed::from_f64(0.2),
            Proposition::GrainPricesTooHigh => Fixed::from_f64(-0.2),
            Proposition::GuardsUnjust => Fixed::from_f64(-0.4),
            Proposition::ViolenceNecessary => Fixed::from_f64(-0.1),
            Proposition::SharingIsDuty => Fixed::from_f64(0.2),
            Proposition::CommunityStrong => Fixed::from_f64(0.4),
            Proposition::StrangersHonest => Fixed::from_f64(0.1),
            Proposition::WellWaterSafe => Fixed::from_f64(0.1),
        }
    }

    /// Get the base identity linkage for this proposition.
    /// High identity linkage = beliefs resist change (§22.9).
    pub fn base_identity_linkage(&self) -> Fixed {
        match self {
            Proposition::MarketIsFair => Fixed::from_f64(0.3),
            Proposition::RulerIsLegitimate => Fixed::from_f64(0.5),
            Proposition::NeighborTrustworthy { .. } => Fixed::from_f64(0.2),
            Proposition::ForeignersDangerous => Fixed::from_f64(0.6),
            Proposition::HardWorkLeadsToWealth => Fixed::from_f64(0.4),
            Proposition::HarvestWillFail => Fixed::from_f64(0.1),
            Proposition::TempleIsCorrupt => Fixed::from_f64(0.5),
            Proposition::CouncilProtectsUs => Fixed::from_f64(0.4),
            Proposition::GrainPricesTooHigh => Fixed::from_f64(0.2),
            Proposition::GuardsUnjust => Fixed::from_f64(0.3),
            Proposition::ViolenceNecessary => Fixed::from_f64(0.5),
            Proposition::SharingIsDuty => Fixed::from_f64(0.4),
            Proposition::CommunityStrong => Fixed::from_f64(0.5),
            Proposition::StrangersHonest => Fixed::from_f64(0.2),
            Proposition::WellWaterSafe => Fixed::from_f64(0.1),
        }
    }

    /// Get the base resistance to change for this proposition.
    pub fn base_resistance(&self) -> Fixed {
        match self {
            Proposition::MarketIsFair => Fixed::from_f64(0.5),
            Proposition::RulerIsLegitimate => Fixed::from_f64(0.6),
            Proposition::NeighborTrustworthy { .. } => Fixed::from_f64(0.3),
            Proposition::ForeignersDangerous => Fixed::from_f64(0.7),
            Proposition::HardWorkLeadsToWealth => Fixed::from_f64(0.5),
            Proposition::HarvestWillFail => Fixed::from_f64(0.2),
            Proposition::TempleIsCorrupt => Fixed::from_f64(0.6),
            Proposition::CouncilProtectsUs => Fixed::from_f64(0.5),
            Proposition::GrainPricesTooHigh => Fixed::from_f64(0.3),
            Proposition::GuardsUnjust => Fixed::from_f64(0.4),
            Proposition::ViolenceNecessary => Fixed::from_f64(0.7),
            Proposition::SharingIsDuty => Fixed::from_f64(0.5),
            Proposition::CommunityStrong => Fixed::from_f64(0.6),
            Proposition::StrangersHonest => Fixed::from_f64(0.3),
            Proposition::WellWaterSafe => Fixed::from_f64(0.2),
        }
    }

    /// All generic propositions (no agent-specific parameters).
    pub fn all_generic() -> Vec<Proposition> {
        vec![
            Proposition::MarketIsFair,
            Proposition::RulerIsLegitimate,
            Proposition::ForeignersDangerous,
            Proposition::HardWorkLeadsToWealth,
            Proposition::HarvestWillFail,
            Proposition::TempleIsCorrupt,
            Proposition::CouncilProtectsUs,
            Proposition::GrainPricesTooHigh,
            Proposition::GuardsUnjust,
            Proposition::ViolenceNecessary,
            Proposition::SharingIsDuty,
            Proposition::CommunityStrong,
            Proposition::StrangersHonest,
            Proposition::WellWaterSafe,
        ]
    }
}

impl fmt::Display for Proposition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.description())
    }
}

/// A belief held by an agent about a proposition.
///
/// §22.9 of the architecture spec: beliefs have provenance, source trust,
/// identity linkage, emotional charge, contradiction tracking, and resistance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PropositionBelief {
    /// Which proposition this belief is about.
    pub proposition: Proposition,
    /// How confident the agent is (0.0 = disbelieve, 1.0 = certain).
    pub confidence: Fixed,
    /// Emotional charge associated with this belief.
    pub emotional_charge: Fixed,
    /// How linked this belief is to the agent's identity.
    /// High identity linkage = beliefs resist change.
    pub identity_linkage: Fixed,
    /// Resistance to belief change (decays over time).
    pub resistance: Fixed,
    /// Tick when this belief was last reinforced by evidence or social contact.
    pub last_reinforced_tick: u64,
    /// Social reinforcement count — how many times this belief was confirmed by others.
    pub social_reinforcement: u32,
}

impl PropositionBelief {
    /// Create a new belief about a proposition with default values.
    pub fn new(proposition: Proposition) -> Self {
        Self {
            confidence: Fixed::from_f64(0.5),
            emotional_charge: proposition.base_emotional_charge(),
            identity_linkage: proposition.base_identity_linkage(),
            resistance: proposition.base_resistance(),
            last_reinforced_tick: 0,
            social_reinforcement: 0,
            proposition,
        }
    }

    /// Create a belief with specific initial confidence.
    pub fn with_confidence(proposition: Proposition, confidence: Fixed) -> Self {
        Self {
            confidence,
            ..Self::new(proposition)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn proposition_ids_are_unique() {
        let props = Proposition::all_generic();
        let mut ids: Vec<u64> = props.iter().map(|p| p.id()).collect();
        ids.sort();
        ids.dedup();
        assert_eq!(ids.len(), props.len(), "All proposition IDs should be unique");
    }

    #[test]
    fn proposition_display() {
        assert_eq!(format!("{}", Proposition::MarketIsFair), "The market is fair");
        assert_eq!(format!("{}", Proposition::ForeignersDangerous), "Foreigners are dangerous");
    }

    #[test]
    fn belief_default_values() {
        let belief = PropositionBelief::new(Proposition::MarketIsFair);
        assert_eq!(belief.confidence, Fixed::from_f64(0.5));
        assert_eq!(belief.proposition, Proposition::MarketIsFair);
        assert!(belief.resistance > Fixed::ZERO);
    }

    #[test]
    fn belief_with_confidence() {
        let belief = PropositionBelief::with_confidence(
            Proposition::HarvestWillFail,
            Fixed::from_f64(0.8),
        );
        assert_eq!(belief.confidence, Fixed::from_f64(0.8));
    }

    #[test]
    fn high_identity_linkage_beliefs_resist_change() {
        let foreign = PropositionBelief::new(Proposition::ForeignersDangerous);
        let water = PropositionBelief::new(Proposition::WellWaterSafe);
        assert!(foreign.identity_linkage > water.identity_linkage,
            "ForeignersDangerous should have higher identity linkage than WellWaterSafe");
    }
}
