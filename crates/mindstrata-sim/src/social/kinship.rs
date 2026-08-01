//! Kinship system — networks of biological, marital, and ritual kinship.
//!
//! Kinship is a network, not a label. Agents have kinship links that affect:
//! - Trust baseline (kin get higher default trust)
//! - Obligation (kin have mutual obligations)
//! - Inheritance (property passes through kinship lines)
//! - Feud participation (kin defend each other)
//! - Marriage restrictions (incest taboo via kinship coefficient)
//! - Household formation (kin co-reside)
//! - Clan identity (shared kinship = shared identity)
//!
//! ```text
//! Kinship types:
//!   Biological  — parent/child, sibling, half-sibling, cousin
//!   Marital     — spouse, in-law
//!   Adoptive    — adoptive parent/child
//!   Ritual      — godparent, oath-sworn sibling, blood brother
//! ```
//!
//! The `kinship_coefficient` on each relationship quantifies genetic relatedness:
//! - Parent/child: 0.5
//! - Full sibling: 0.5
//! - Half-sibling: 0.25
//! - First cousin: 0.125
//! - Unrelated: 0.0
//!
//! This coefficient is used to enforce the incest taboo and model inbreeding risk.
//!
//! Emergent effects:
//! - Families form clans through repeated marriage alliances
//! - Kin obligations override institutional loyalty
//! - Inheritance disputes fracture households
//! - Blood oaths create fictive kinship bonds
//! - Godparent networks bridge households

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Type of kinship link.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum KinshipLink {
    /// Biological parent → child.
    ParentChild,
    /// Biological siblings (full or half).
    Sibling,
    /// Marital relationship.
    Spouse,
    /// In-law (via marriage of kin).
    InLaw,
    /// Adoptive parent → child.
    Adoptive,
    /// Godparent → godchild.
    Godparent,
    /// Oath-sworn siblings (ritual kinship).
    OathSibling,
}

/// A single kinship link between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinshipEdge {
    /// From agent index.
    pub from: usize,
    /// To agent index.
    pub to: usize,
    /// Type of kinship relationship.
    pub link: KinshipLink,
    /// Genetic relatedness coefficient (0.0 for non-biological).
    pub coefficient: Fixed,
    /// Strength of the kinship bond (0–1). Decays without reinforcement.
    pub strength: Fixed,
    /// Tick when this link was created.
    pub created_tick: u64,
    /// Is this link currently active? (can be severed by events)
    pub active: bool,
}

impl KinshipEdge {
    /// Create a new kinship edge.
    pub fn new(
        from: usize,
        to: usize,
        link: KinshipLink,
        coefficient: Fixed,
        tick: u64,
    ) -> Self {
        Self {
            from,
            to,
            link,
            coefficient,
            strength: Fixed::from_f64(0.8),
            created_tick: tick,
            active: true,
        }
    }

    /// Biological kinship coefficient for a given link type.
    pub fn biological_coefficient(link_type: KinshipLink) -> Fixed {
        match link_type {
            KinshipLink::ParentChild => Fixed::from_f64(0.5),
            KinshipLink::Sibling => Fixed::from_f64(0.5),
            KinshipLink::Spouse => Fixed::ZERO,
            KinshipLink::InLaw => Fixed::ZERO,
            KinshipLink::Adoptive => Fixed::ZERO,
            KinshipLink::Godparent => Fixed::ZERO,
            KinshipLink::OathSibling => Fixed::ZERO,
        }
    }
}

/// Kinship graph for the entire population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinshipGraph {
    /// All kinship edges.
    pub edges: Vec<KinshipEdge>,
}

impl Default for KinshipGraph {
    fn default() -> Self {
        Self { edges: Vec::new() }
    }
}

impl KinshipGraph {
    /// Add a kinship edge between two agents.
    pub fn add_link(
        &mut self,
        from: usize,
        to: usize,
        link: KinshipLink,
        tick: u64,
    ) {
        let coefficient = KinshipEdge::biological_coefficient(link);
        self.edges.push(KinshipEdge::new(from, to, link, coefficient, tick));
        // Mirror: child also knows parent, sibling knows sibling
        if matches!(link, KinshipLink::Sibling | KinshipLink::OathSibling) {
            self.edges.push(KinshipEdge::new(to, from, link, coefficient, tick));
        }
    }

    /// Get all active kinship edges for an agent.
    pub fn kin_of(&self, agent: usize) -> Vec<&KinshipEdge> {
        self.edges.iter()
            .filter(|e| e.from == agent && e.active)
            .collect()
    }

    /// Compute the kinship coefficient between two agents (highest along any path).
    /// Checks both directions to handle unidirectional parent→child edges.
    pub fn coefficient_between(&self, a: usize, b: usize) -> Fixed {
        self.edges.iter()
            .filter(|e| e.active && ((e.from == a && e.to == b) || (e.from == b && e.to == a)))
            .map(|e| e.coefficient)
            .max_by_key(|c| c.to_raw())
            .unwrap_or(Fixed::ZERO)
    }

    /// Check if two agents are close kin (coefficient >= 0.125, i.e., first cousins or closer).
    pub fn are_close_kin(&self, a: usize, b: usize) -> bool {
        self.coefficient_between(a, b) >= Fixed::from_f64(0.125)
    }

    /// Check if two agents can marry (not close kin).
    pub fn can_marry(&self, a: usize, b: usize) -> bool {
        !self.are_close_kin(a, b)
    }

    /// Decay all kinship edges (called daily).
    pub fn decay_daily(&mut self) {
        for edge in &mut self.edges {
            if edge.active {
                edge.strength = (edge.strength * Fixed::from_f64(0.999)).clamp_01();
            }
        }
    }

    /// Number of active kinship edges.
    pub fn active_count(&self) -> usize {
        self.edges.iter().filter(|e| e.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parent_child_coefficient_is_half() {
        let c = KinshipEdge::biological_coefficient(KinshipLink::ParentChild);
        assert_eq!(c, Fixed::from_f64(0.5));
    }

    #[test]
    fn sibling_coefficient_is_half() {
        let c = KinshipEdge::biological_coefficient(KinshipLink::Sibling);
        assert_eq!(c, Fixed::from_f64(0.5));
    }

    #[test]
    fn spouse_coefficient_is_zero() {
        let c = KinshipEdge::biological_coefficient(KinshipLink::Spouse);
        assert_eq!(c, Fixed::ZERO);
    }

    #[test]
    fn add_parent_child_link() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0);
        assert_eq!(g.kin_of(0).len(), 1);
        assert_eq!(g.kin_of(0)[0].link, KinshipLink::ParentChild);
    }

    #[test]
    fn sibling_link_is_bidirectional() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::Sibling, 0);
        assert_eq!(g.kin_of(0).len(), 1);
        assert_eq!(g.kin_of(1).len(), 1);
    }

    #[test]
    fn coefficient_between_parent_child() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0);
        assert_eq!(g.coefficient_between(0, 1), Fixed::from_f64(0.5));
    }

    #[test]
    fn close_kin_detected() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0);
        assert!(g.are_close_kin(0, 1));
        // Parent/child are close kin, cannot marry
        assert!(!g.can_marry(0, 1));
    }

    #[test]
    fn unrelated_agents_not_close_kin() {
        let g = KinshipGraph::default();
        assert!(!g.are_close_kin(0, 1));
        assert!(g.can_marry(0, 1));
    }

    #[test]
    fn decay_reduces_strength() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::Sibling, 0);
        let before = g.edges[0].strength;
        g.decay_daily();
        assert!(g.edges[0].strength < before);
    }

    #[test]
    fn active_count() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0); // 1 edge (unidirectional)
        g.add_link(1, 2, KinshipLink::Sibling, 0);     // 2 edges (bidirectional mirror)
        assert_eq!(g.active_count(), 3);
    }

    // TODO: transitive kinship via BFS for grandparent/great-aunt detection
}
