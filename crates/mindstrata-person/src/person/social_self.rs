//! Relational substrate: relationship kinds, status, directed ties.

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

// ── Relational substrate ─────────────────────────────────────────────────

/// §19.5.G: Type of relationship — determines interaction patterns.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RelationshipKind {
    /// Stranger. (doc added at S2 extraction)
    Stranger,
    /// Neighbor. (doc added at S2 extraction)
    Neighbor,
    /// Colleague. (doc added at S2 extraction)
    Colleague,
    /// Friend. (doc added at S2 extraction)
    Friend,
    /// Rival. (doc added at S2 extraction)
    Rival,
    /// Kin. (doc added at S2 extraction)
    Kin,
}

/// §19.5.G: Per-agent status — derived from wealth, institutional role, social connections.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusState {
    /// Base status from wealth (0..1).
    pub wealth_status: Fixed,
    /// Status from institutional role (0..1).
    pub role_status: Fixed,
    /// Status from social connections (0..1).
    pub social_status: Fixed,
    /// Combined effective status.
    pub effective: Fixed,
}

impl Default for StatusState {
    fn default() -> Self {
        Self {
            wealth_status: Fixed::from_f64(0.3),
            role_status: Fixed::ZERO,
            social_status: Fixed::from_f64(0.3),
            effective: Fixed::from_f64(0.3),
        }
    }
}

impl StatusState {
    /// Recompute effective status from components.
    pub fn recompute(&mut self) {
        self.effective = (self.wealth_status * Fixed::from_f64(0.4)
            + self.role_status * Fixed::from_f64(0.35)
            + self.social_status * Fixed::from_f64(0.25))
        .clamp_01();
    }
}

/// A directed relationship between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relationship {
    /// From. (doc added at S2 extraction)
    pub from: AgentId,
    /// To. (doc added at S2 extraction)
    pub to: AgentId,
    /// Trust. (doc added at S2 extraction)
    pub trust: Fixed,
    /// Affection. (doc added at S2 extraction)
    pub affection: Fixed,
    /// Respect. (doc added at S2 extraction)
    pub respect: Fixed,
    /// Fear. (doc added at S2 extraction)
    pub fear: Fixed,
    /// Obligation. (doc added at S2 extraction)
    pub obligation: Fixed,
    /// Last interaction tick. (doc added at S2 extraction)
    pub last_interaction_tick: u64,
    /// §19.5.G: Relationship type — evolves based on interaction history.
    pub kind: RelationshipKind,
    /// §19.5.G: Interaction count — drives relationship evolution.
    pub interaction_count: u32,
    /// §19.5.G: Last positive interaction tick (for friendship formation).
    pub last_positive_tick: u64,
    /// §19.5.G: Last negative interaction tick (for rivalry formation).
    pub last_negative_tick: u64,
}
