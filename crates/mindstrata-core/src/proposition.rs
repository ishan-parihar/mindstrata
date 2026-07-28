//! Proposition identifiers for machine-processable beliefs.
//!
//! Each belief in the simulation references a proposition ID that can be
//! evaluated against world state.

use serde::{Deserialize, Serialize};

/// Unique identifier for a proposition in the registry.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PropositionId(pub u64);

impl std::fmt::Debug for PropositionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Prop({})", self.0)
    }
}

impl std::fmt::Display for PropositionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "prop_{}", self.0)
    }
}

/// Well-known proposition IDs used across the simulation.
impl PropositionId {
    /// "The market is fair."
    pub const MARKET_IS_FAIR: Self = Self(0);
    /// "The ruler is legitimate."
    pub const RULER_IS_LEGITIMATE: Self = Self(1);
    /// "My neighbor can be trusted."
    pub const NEIGHBOR_TRUSTWORTHY: Self = Self(2);
    /// "Foreigners are dangerous."
    pub const FOREIGNERS_DANGEROUS: Self = Self(3);
    /// "Hard work leads to wealth."
    pub const HARD_WORK_WEALTH: Self = Self(4);
}
