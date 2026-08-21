//! §13: Noospheric field — the symbolic space where concepts, beliefs, and narratives live.
//!
//! Architecture §9.2: The noosphere uses concept embeddings and spreading
//! activation to model how ideas propagate, associate, and compete.
//!
//! ```text
//! Noospheric dynamics:
//!   - Concept nodes with low-dimensional fixed-point vectors
//!   - Spreading activation through associative edges
//!   - Emotional charge amplifies activation
//!   - Identity relevance protects activation from decay
//!   - Institutional backing boosts activation
//!   - Suppression reduces activation
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Dimension count for concept vectors.
pub const CONCEPT_DIMS: usize = 12;

/// A concept vector in the noospheric space.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConceptVector {
    /// Safety dimension.
    pub safety: Fixed,
    /// Sacredness dimension.
    pub sacredness: Fixed,
    /// Status dimension.
    pub status: Fixed,
    /// Kinship dimension.
    pub kinship: Fixed,
    /// Scarcity dimension.
    pub scarcity: Fixed,
    /// Threat dimension.
    pub threat: Fixed,
    /// Purity dimension.
    pub purity: Fixed,
    /// Freedom dimension.
    pub freedom: Fixed,
    /// Loyalty dimension.
    pub loyalty: Fixed,
    /// Pleasure dimension.
    pub pleasure: Fixed,
    /// Shame dimension.
    pub shame: Fixed,
    /// Hope dimension.
    pub hope: Fixed,
}

impl Default for ConceptVector {
    fn default() -> Self {
        Self {
            safety: Fixed::ZERO,
            sacredness: Fixed::ZERO,
            status: Fixed::ZERO,
            kinship: Fixed::ZERO,
            scarcity: Fixed::ZERO,
            threat: Fixed::ZERO,
            purity: Fixed::ZERO,
            freedom: Fixed::ZERO,
            loyalty: Fixed::ZERO,
            pleasure: Fixed::ZERO,
            shame: Fixed::ZERO,
            hope: Fixed::ZERO,
        }
    }
}

impl ConceptVector {
    /// Compute dot product with another concept vector.
    pub fn dot(&self, other: &ConceptVector) -> Fixed {
        self.safety * other.safety
            + self.sacredness * other.sacredness
            + self.status * other.status
            + self.kinship * other.kinship
            + self.scarcity * other.scarcity
            + self.threat * other.threat
            + self.purity * other.purity
            + self.freedom * other.freedom
            + self.loyalty * other.loyalty
            + self.pleasure * other.pleasure
            + self.shame * other.shame
            + self.hope * other.hope
    }

    /// Compute the norm (magnitude) of this vector.
    pub fn norm(&self) -> Fixed {
        let sum = self.safety * self.safety
            + self.sacredness * self.sacredness
            + self.status * self.status
            + self.kinship * self.kinship
            + self.scarcity * self.scarcity
            + self.threat * self.threat
            + self.purity * self.purity
            + self.freedom * self.freedom
            + self.loyalty * self.loyalty
            + self.pleasure * self.pleasure
            + self.shame * self.shame
            + self.hope * self.hope;
        Fixed::from_f64(sum.to_f64().sqrt())
    }

    /// Cosine similarity with another concept vector.
    pub fn similarity(&self, other: &ConceptVector) -> Fixed {
        let dot = self.dot(other);
        let norm_a = self.norm();
        let norm_b = other.norm();
        let denom = norm_a * norm_b;
        if denom == Fixed::ZERO {
            Fixed::ZERO
        } else {
            (dot / denom).clamp_01()
        }
    }
}

/// A symbolic node in the noospheric field — a concept, belief, or meme.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicNode {
    /// Unique node id.
    pub id: u32,
    /// Concept vector defining this node's position in conceptual space.
    pub concept: ConceptVector,
    /// Current activation level (0 = dormant, 1 = maximally activated).
    pub activation: Fixed,
    /// Emotional charge — how emotionally loaded this concept is (0–1).
    pub emotional_charge: Fixed,
    /// Identity relevance — how tied to agent identities this concept is (0–1).
    pub identity_relevance: Fixed,
    /// Sacredness — how sacred/untouchable this concept is (0–1).
    pub sacredness: Fixed,
    /// Institutional backing — boosted by institutional propagation (0–1).
    pub institutional_backing: Fixed,
    /// Suppression level — how much this concept is being suppressed (0–1).
    pub suppression: Fixed,
}

impl SymbolicNode {
    /// Create a new symbolic node.
    pub fn new(id: u32, concept: ConceptVector) -> Self {
        Self {
            id,
            concept,
            activation: Fixed::from_f64(0.3),
            emotional_charge: Fixed::ZERO,
            identity_relevance: Fixed::ZERO,
            sacredness: Fixed::ZERO,
            institutional_backing: Fixed::ZERO,
            suppression: Fixed::ZERO,
        }
    }

    /// Effective activation after accounting for suppression and institutional backing.
    pub fn effective_activation(&self) -> Fixed {
        let boosted = self.activation + self.institutional_backing * Fixed::from_f64(0.2);
        let suppressed = boosted * (Fixed::ONE - self.suppression * Fixed::from_f64(0.5));
        suppressed.clamp_01()
    }
}

/// A directed edge between symbolic nodes (associative link).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolicEdge {
    /// Source node id.
    pub from: u32,
    /// Target node id.
    pub to: u32,
    /// Weight of the association (0–1).
    pub weight: Fixed,
    /// Decay rate of this edge (0 = permanent, 1 = instant decay).
    pub decay: Fixed,
}

/// The noospheric field — the symbolic space where concepts live and propagate.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NoosphericField {
    /// All symbolic nodes.
    pub nodes: Vec<SymbolicNode>,
    /// All associative edges.
    pub edges: Vec<SymbolicEdge>,
}

impl NoosphericField {
    /// Create a new empty noospheric field.
    pub fn new() -> Self {
        Self {
            nodes: Vec::new(),
            edges: Vec::new(),
        }
    }

    /// Add a node and return its id.
    pub fn add_node(&mut self, node: SymbolicNode) -> u32 {
        let id = node.id;
        self.nodes.push(node);
        id
    }

    /// Add an edge between two nodes.
    pub fn add_edge(&mut self, edge: SymbolicEdge) {
        self.edges.push(edge);
    }

    /// Spreading activation — propagate activation from a source node through edges.
    ///
    /// Architecture §9.2:
    /// ```text
    /// activation(node) += input_strength
    /// for edge in associations:
    ///     activation(target) += activation(source) * edge_weight * decay
    /// ```
    pub fn spread_activation(&mut self, source_id: u32, input_strength: Fixed) {
        // Find source node
        if let Some(source_idx) = self.nodes.iter().position(|n| n.id == source_id) {
            self.nodes[source_idx].activation =
                (self.nodes[source_idx].activation + input_strength).clamp_01();
            let source_activation = self.nodes[source_idx].activation;

            // Propagate through edges
            let edges_snapshot: Vec<(usize, Fixed, Fixed)> = self
                .edges
                .iter()
                .enumerate()
                .filter(|(_, e)| e.from == source_id)
                .map(|(i, e)| (i, e.weight, e.decay))
                .collect();

            for (edge_idx, weight, decay) in edges_snapshot {
                let target_id = self.edges[edge_idx].to;
                let propagation = source_activation * weight * (Fixed::ONE - decay);
                if let Some(target_idx) = self.nodes.iter().position(|n| n.id == target_id) {
                    self.nodes[target_idx].activation =
                        (self.nodes[target_idx].activation + propagation).clamp_01();
                }
            }
        }
    }

    /// Natural decay of all activations.
    pub fn decay_all(&mut self, decay_rate: Fixed) {
        for node in &mut self.nodes {
            // Sacred concepts decay slower
            let effective_decay =
                decay_rate * (Fixed::ONE - node.sacredness * Fixed::from_f64(0.8));
            node.activation = (node.activation - effective_decay).max(Fixed::ZERO);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn concept_similarity_identical() {
        let a = ConceptVector {
            safety: Fixed::from_f64(0.5),
            sacredness: Fixed::from_f64(0.3),
            ..ConceptVector::default()
        };
        assert!(a.similarity(&a) > Fixed::from_f64(0.9));
    }

    #[test]
    fn spreading_activation_increases_target() {
        let mut field = NoosphericField::new();
        let n0 = SymbolicNode::new(0, ConceptVector::default());
        let n1 = SymbolicNode::new(1, ConceptVector::default());
        field.add_node(n0);
        field.add_node(n1);
        field.add_edge(SymbolicEdge {
            from: 0,
            to: 1,
            weight: Fixed::from_f64(0.5),
            decay: Fixed::from_f64(0.1),
        });

        field.spread_activation(0, Fixed::from_f64(0.8));
        assert!(field.nodes[1].activation > Fixed::ZERO);
    }

    #[test]
    fn effective_activation_suppressed() {
        let node = SymbolicNode {
            activation: Fixed::from_f64(0.8),
            suppression: Fixed::from_f64(0.5),
            ..SymbolicNode::new(0, ConceptVector::default())
        };
        assert!(node.effective_activation() < node.activation);
    }
}
