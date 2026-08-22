//! Technology tree — §5 (AP2, Iteration 148).
//!
//! The knowledge system stores a flat bag of ids with no ordering. This
//! module adds the two missing plan dimensions — *knowledge prerequisites*
//! and *innovation chains*:
//!
//! - **Prerequisites**: every node past tier 0 requires the population to
//!   have reached its prerequisite nodes. `can_learn` gates every
//!   transmission site (socialization, work-innovation, interaction
//!   diffusion), so nobody can learn Advanced Irrigation without Crop
//!   Rotation, nor Steelworking without Metalworking.
//! - **Innovation chains**: tier-1+ nodes are NOT in the world at start.
//!   A yearly discovery pass (every 4320 ticks — the sim's established
//!   ritual cadence) checks each undiscovered node's critical mass — the
//!   fraction of agents holding ALL its prerequisites — and a seeded draw
//!   (from the virgin Narrative RNG stream, the only consumer of that
//!   stream, so no other subsystem can be perturbed) adds the node to the
//!   knowledge store when the mass and the roll align. Steelworking
//!   therefore becomes learnable only after Metalworking has been discovered
//!   AND spread through the population — a genuine two-level chain.
//!
//! Blast contract: the discovery pass runs ONLY on the 4320-tick cadence, so
//! calibrated windows (golden @2000, snapshots ≤2000) contain ZERO passes —
//! byte-identical by construction, not by probe. The 5000-tick drought probe
//! contains one pass; its draws land on the virgin Narrative stream (no
//! other consumer), so even a successful discovery cannot shift any other
//! subsystem's RNG. The 10000-tick long-horizon snapshot contains two
//! passes — deterministic regeneration whether or not a discovery fires.

use std::collections::{HashMap, HashSet};

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

use super::knowledge::{Knowledge, KnowledgeCategory};

/// Fraction of the population that must hold ALL of a node's prerequisites
/// before the daily discovery pass may roll for it.
pub const INNOVATION_THRESHOLD: f64 = 0.5;
/// Yearly per-node discovery base rate, scaled by (mass − threshold) and
/// (1 − difficulty). With a universal prerequisite (mass 1.0, difficulty
/// 0.5) the per-node yearly chance is 0.1 × 0.5 × 0.5 = 0.025 — expected
/// ~40 years per node; the ANY-node expected first fire across the five
/// tier-1 nodes is ~11 years (~46K ticks), so tier-1 nodes are
/// rare-but-possible on decade horizons. Because the pass only runs every
/// 4320 ticks, no discovery can land inside any calibrated window (golden
/// @2000, snapshots ≤2000) — the safety is by construction, not calibration.
pub const INNOVATION_RATE: f64 = 0.1;

/// A node in the technology tree.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechNode {
    /// Id. (doc added at S3 extraction)
    pub id: u64,
    /// Name. (doc added at S3 extraction)
    pub name: String,
    /// Category. (doc added at S3 extraction)
    pub category: KnowledgeCategory,
    /// How difficult the knowledge is to acquire (0 = easy, 1 = very hard).
    pub difficulty: Fixed,
    /// How useful the knowledge is (0 = useless, 1 = essential).
    pub utility: Fixed,
    /// Knowledge ids that must be known before this node can be learned.
    pub prereqs: Vec<u64>,
    /// Tree depth (0 = seeded baseline, 1 = first innovation, ...).
    pub tier: u32,
}

impl TechNode {
    fn new(
        id: u64,
        name: &str,
        category: KnowledgeCategory,
        difficulty: f64,
        utility: f64,
        prereqs: Vec<u64>,
        tier: u32,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            category,
            difficulty: Fixed::from_f64(difficulty),
            utility: Fixed::from_f64(utility),
            prereqs,
            tier,
        }
    }
}

/// The tree: the full knowledge catalog (seeded tier 0 + innovatable tiers)
/// and the set of nodes actually present in the world.
///
/// `Serialize`/`Deserialize` are reserved for a future snapshot field — the
/// current restore path (`Simulation::from_snapshot`) reconstructs the tree
/// via `reconcile_discovered` instead (the weather precedent), so the tree
/// is never serialized today.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TechnologyTree {
    /// Nodes. (doc added at S3 extraction)
    pub nodes: HashMap<u64, TechNode>,
    /// Node ids present in the world (seeded = tier 0; tiers 1+ enter only
    /// through the discovery pass).
    pub discovered: HashSet<u64>,
}

impl TechnologyTree {
    /// The riverford catalog: five seeded tier-0 nodes mirror the sim's
    /// `knowledge_store` seeding, five tier-1 innovations (one per seed), and
    /// one tier-2 node (`Steelworking`, requiring Well Maintenance + the
    /// innovated Metalworking) that proves multi-level chains.
    pub fn riverford() -> Self {
        let mut nodes = HashMap::new();
        let mut discovered = HashSet::new();
        let mut add = |node: TechNode, seeded: bool| {
            if seeded {
                discovered.insert(node.id);
            }
            nodes.insert(node.id, node);
        };
        // Tier 0 — the seeded baseline (mirrors knowledge_store ids 0..4).
        add(
            TechNode::new(
                0,
                "Crop Rotation",
                KnowledgeCategory::Agricultural,
                0.3,
                0.8,
                vec![],
                0,
            ),
            true,
        );
        add(
            TechNode::new(
                1,
                "Well Maintenance",
                KnowledgeCategory::Craft,
                0.4,
                0.6,
                vec![],
                0,
            ),
            true,
        );
        add(
            TechNode::new(
                2,
                "Herbal Medicine",
                KnowledgeCategory::Medical,
                0.7,
                0.9,
                vec![],
                0,
            ),
            true,
        );
        add(
            TechNode::new(
                3,
                "Harvest Prayer",
                KnowledgeCategory::Philosophical,
                0.2,
                0.4,
                vec![],
                0,
            ),
            true,
        );
        add(
            TechNode::new(
                4,
                "Grain Storage",
                KnowledgeCategory::Agricultural,
                0.3,
                0.7,
                vec![],
                0,
            ),
            true,
        );
        // Tier 1 — each requires its tier-0 seed.
        add(
            TechNode::new(
                5,
                "Advanced Irrigation",
                KnowledgeCategory::Agricultural,
                0.6,
                0.9,
                vec![0],
                1,
            ),
            false,
        );
        add(
            TechNode::new(
                6,
                "Metalworking",
                KnowledgeCategory::Craft,
                0.55,
                0.8,
                vec![1],
                1,
            ),
            false,
        );
        add(
            TechNode::new(
                7,
                "Surgery",
                KnowledgeCategory::Medical,
                0.8,
                0.95,
                vec![2],
                1,
            ),
            false,
        );
        add(
            TechNode::new(
                8,
                "Astronomy",
                KnowledgeCategory::Philosophical,
                0.5,
                0.6,
                vec![3],
                1,
            ),
            false,
        );
        add(
            TechNode::new(
                9,
                "Fermentation",
                KnowledgeCategory::Agricultural,
                0.5,
                0.75,
                vec![4],
                1,
            ),
            false,
        );
        // Tier 2 — the innovation chain: requires the seeded craft AND the
        // innovated Metalworking.
        add(
            TechNode::new(
                10,
                "Steelworking",
                KnowledgeCategory::Craft,
                0.75,
                0.95,
                vec![1, 6],
                2,
            ),
            false,
        );
        Self { nodes, discovered }
    }

    /// Whether a learner holding `known` may acquire `id` — every
    /// prerequisite must be known. Tier-0 nodes (empty prerequisites) are
    /// always learnable, which keeps calibrated windows unchanged.
    #[must_use]
    pub fn can_learn(&self, known: &[u64], id: u64) -> bool {
        self.nodes
            .get(&id)
            .is_none_or(|n| n.prereqs.iter().all(|p| known.contains(p)))
    }

    /// Whether `id` is present in the world (seeded or innovated).
    #[must_use]
    pub fn is_discovered(&self, id: u64) -> bool {
        self.discovered.contains(&id)
    }

    /// Fraction of agents holding ALL of `id`'s prerequisites — the
    /// "competence pool" that gates discovery.
    #[must_use]
    pub fn prerequisite_mass(&self, id: u64, agents_known: &[Vec<u64>]) -> Fixed {
        let Some(node) = self.nodes.get(&id) else {
            return Fixed::ZERO;
        };
        if node.prereqs.is_empty() {
            return Fixed::ONE;
        }
        let n = agents_known.len();
        if n == 0 {
            return Fixed::ZERO;
        }
        let holding = agents_known
            .iter()
            .filter(|known| node.prereqs.iter().all(|p| known.contains(p)))
            .count();
        Fixed::from_int(holding as i64) / Fixed::from_int(n as i64)
    }

    /// The daily discovery chance for an undiscovered node at the current
    /// prerequisite mass: `(mass − threshold) × rate × (1 − difficulty)`,
    /// floored at zero — no mass, no discovery.
    #[must_use]
    pub fn discovery_chance(&self, id: u64, agents_known: &[Vec<u64>]) -> Fixed {
        let Some(node) = self.nodes.get(&id) else {
            return Fixed::ZERO;
        };
        if node.prereqs.is_empty() || self.discovered.contains(&id) {
            return Fixed::ZERO;
        }
        let mass = self.prerequisite_mass(id, agents_known);
        let excess = (mass - Fixed::from_f64(INNOVATION_THRESHOLD)).max(Fixed::ZERO);
        excess * Fixed::from_f64(INNOVATION_RATE) * (Fixed::ONE - node.difficulty)
    }

    /// Add a node to the world; returns the `Knowledge` entry to push into
    /// the sim's `knowledge_store` (with its discovery tick).
    pub fn discover(&mut self, id: u64, tick: u64) -> Option<Knowledge> {
        let node = self.nodes.get(&id)?;
        if self.discovered.contains(&id) {
            return None;
        }
        self.discovered.insert(id);
        Some(Knowledge {
            id: node.id,
            name: node.name.clone(),
            category: node.category,
            difficulty: node.difficulty,
            utility: node.utility,
            holders: 0,
            discovered_tick: tick,
        })
    }

    /// Reconcile the discovered set with a serialized knowledge store — used
    /// by `Simulation::from_snapshot` so a snapshot taken mid-run after a
    /// discovery restores the tree in the exact discovered state.
    pub fn reconcile_discovered(&mut self, store: &[Knowledge]) {
        for k in store {
            if self.nodes.contains_key(&k.id) {
                self.discovered.insert(k.id);
            }
        }
    }

    /// Iterate undiscovered node ids in ascending order (stable pass order).
    pub fn undiscovered(&self) -> impl Iterator<Item = u64> + '_ {
        let mut ids: Vec<u64> = self
            .nodes
            .keys()
            .copied()
            .filter(|id| !self.discovered.contains(id))
            .collect();
        ids.sort_unstable();
        ids.into_iter()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn known(ids: &[u64]) -> Vec<u64> {
        ids.to_vec()
    }

    #[test]
    fn tier_zero_is_always_learnable() {
        let tree = TechnologyTree::riverford();
        assert!(
            tree.can_learn(&[], 0),
            "tier-0 Crop Rotation needs no prereq"
        );
        assert!(
            tree.can_learn(&[], 4),
            "tier-0 Grain Storage needs no prereq"
        );
    }

    #[test]
    fn tier_one_requires_its_seed() {
        let tree = TechnologyTree::riverford();
        // Irrigation needs Crop Rotation.
        assert!(!tree.can_learn(&[], 5));
        assert!(tree.can_learn(&known(&[0]), 5));
        assert!(
            !tree.can_learn(&known(&[1]), 5),
            "wrong prereq must not satisfy"
        );
    }

    #[test]
    fn tier_two_chain_requires_innovated_tier_one() {
        let tree = TechnologyTree::riverford();
        // Steelworking needs Well Maintenance AND Metalworking.
        assert!(!tree.can_learn(&known(&[1]), 10));
        assert!(!tree.can_learn(&known(&[6]), 10));
        assert!(tree.can_learn(&known(&[1, 6]), 10));
    }

    #[test]
    fn tier_one_is_not_discovered_at_start() {
        let tree = TechnologyTree::riverford();
        assert!(tree.is_discovered(0));
        assert!(!tree.is_discovered(5));
        assert!(!tree.is_discovered(10));
        assert_eq!(tree.undiscovered().count(), 6, "five tier-1 + one tier-2");
    }

    #[test]
    fn discovery_chance_requires_critical_mass() {
        let tree = TechnologyTree::riverford();
        // No one holds Crop Rotation → zero chance.
        let empty: Vec<Vec<u64>> = vec![vec![]; 10];
        assert_eq!(tree.discovery_chance(5, &empty), Fixed::ZERO);
        // 4 of 10 hold it → mass 0.4 < 0.5 threshold → zero chance.
        let partial: Vec<Vec<u64>> = (0..10)
            .map(|i| if i < 4 { known(&[0]) } else { vec![] })
            .collect();
        assert_eq!(tree.discovery_chance(5, &partial), Fixed::ZERO);
        // 8 of 10 → mass 0.8 → excess 0.3 × rate 0.1 × (1 − 0.6) = 0.012.
        let enough: Vec<Vec<u64>> = (0..10)
            .map(|i| if i < 8 { known(&[0]) } else { vec![] })
            .collect();
        let c = tree.discovery_chance(5, &enough);
        assert!(
            c > Fixed::ZERO && c < Fixed::from_f64(0.02),
            "got {}",
            c.to_f64()
        );
    }

    #[test]
    fn reconcile_discovered_marks_store_ids_present_in_catalog() {
        let mut tree = TechnologyTree::riverford();
        assert!(!tree.is_discovered(5), "precondition: tier-1 starts hidden");
        let store = vec![
            Knowledge {
                id: 5,
                name: "Advanced Irrigation".into(),
                category: KnowledgeCategory::Agricultural,
                difficulty: Fixed::from_f64(0.6),
                utility: Fixed::from_f64(0.9),
                holders: 0,
                discovered_tick: 4320,
            },
            Knowledge {
                id: 99,
                name: "Unknown".into(),
                category: KnowledgeCategory::Philosophical,
                difficulty: Fixed::from_f64(0.5),
                utility: Fixed::from_f64(0.5),
                holders: 0,
                discovered_tick: 4320,
            },
        ];
        tree.reconcile_discovered(&store);
        assert!(
            tree.is_discovered(5),
            "a catalog node present in the store is reconciled"
        );
        assert!(
            !tree.is_discovered(99),
            "an id absent from the catalog must not be marked discovered"
        );
    }

    #[test]
    fn discover_adds_node_once_and_returns_knowledge() {
        let mut tree = TechnologyTree::riverford();
        let k = tree
            .discover(5, 1234)
            .expect("discovery returns the Knowledge entry");
        assert_eq!(k.id, 5);
        assert_eq!(k.discovered_tick, 1234);
        assert!(tree.is_discovered(5));
        assert!(
            tree.discover(5, 2000).is_none(),
            "second discovery is a no-op"
        );
        // The chain unlocks: now Steelworking needs [1, 6] — 6 still hidden.
        assert!(!tree.is_discovered(6));
    }
}
