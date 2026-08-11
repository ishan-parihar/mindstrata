//! §10.6: Kinship graph — biological, marital, adoptive, and ritual kinship.
//!
//! Kinship affects:
//! - trust baseline,
//! - obligation,
//! - inheritance,
//! - feud participation,
//! - marriage restrictions,
//! - household formation,
//! - clan identity.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};
use std::collections::{HashSet, VecDeque};

/// The type of kinship link between two agents.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KinshipLink {
    /// Biological parent → child (coefficient 0.5).
    ParentChild,
    /// Biological siblings (full or half, coefficient 0.5 or 0.25).
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

/// A directed kinship edge between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KinshipEdge {
    /// Source agent index.
    pub from: usize,
    /// Target agent index.
    pub to: usize,
    /// Type of kinship relationship.
    pub link: KinshipLink,
    /// Genetic relatedness coefficient (0.0 for non-biological).
    pub coefficient: Fixed,
    /// Bond strength (0–1).
    pub strength: Fixed,
    /// Creation tick.
    pub created_tick: u64,
    /// Whether the link is currently active.
    pub active: bool,
}

impl KinshipEdge {
    /// Compute the genetic relatedness coefficient for a given link type.
    pub fn biological_coefficient(link: KinshipLink) -> Fixed {
        match link {
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

/// The kinship graph — a network of kinship relationships.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct KinshipGraph {
    /// All kinship edges.
    pub edges: Vec<KinshipEdge>,
}

impl KinshipGraph {
    /// Add a kinship link between two agents.
    ///
    /// For bidirectional links (Sibling, OathSibling), both directions are created.
    pub fn add_link(&mut self, from: usize, to: usize, link: KinshipLink, tick: u64) {
        let coefficient = KinshipEdge::biological_coefficient(link);
        let strength = Fixed::from_f64(0.5);
        self.edges.push(KinshipEdge {
            from,
            to,
            link,
            coefficient,
            strength,
            created_tick: tick,
            active: true,
        });
        // Bidirectional links get mirrored
        match link {
            KinshipLink::Sibling | KinshipLink::OathSibling => {
                self.edges.push(KinshipEdge {
                    from: to,
                    to: from,
                    link,
                    coefficient,
                    strength,
                    created_tick: tick,
                    active: true,
                });
            }
            _ => {}
        }
    }

    /// Get all active outgoing kinship edges for a given agent.
    /// §10.3 (AP2): Find the kinship link between two agents, scanning both
    /// directions (edges are directed — populate writes parent→child, so a
    /// child→parent lookup needs the reverse scan). Returns the first matching
    /// link; deterministic given the deterministic edge insertion order.
    pub fn link_between(&self, a: usize, b: usize) -> Option<KinshipLink> {
        self.edges
            .iter()
            .find(|e| (e.from == a && e.to == b) || (e.from == b && e.to == a))
            .map(|e| e.link)
    }

    pub fn kin_of(&self, agent: usize) -> Vec<&KinshipEdge> {
        self.edges
            .iter()
            .filter(|e| e.active && e.from == agent)
            .collect()
    }

    /// Compute the kinship coefficient between two agents (highest along any path).
    /// Checks both directions to handle unidirectional parent→child edges.
    pub fn coefficient_between(&self, a: usize, b: usize) -> Fixed {
        self.edges
            .iter()
            .filter(|e| e.active && ((e.from == a && e.to == b) || (e.from == b && e.to == a)))
            .map(|e| e.coefficient)
            .max_by_key(|c| c.to_raw())
            .unwrap_or(Fixed::ZERO)
    }

    /// Compute transitive kinship coefficient via BFS.
    ///
    /// Architecture §10.6: Traverses the kinship graph breadth-first to find
    /// the maximum genetic relatedness along any path. Detects grandparents,
    /// great-aunts, first cousins, etc. Uses half-coefficient decay per hop
    /// for biological links.
    ///
    /// # Panics
    /// Panics if `a == b`.
    pub fn transitive_coefficient(&self, a: usize, b: usize) -> Fixed {
        debug_assert_ne!(a, b, "transitive_coefficient called with a == b");

        // BFS from agent a, tracking maximum coefficient seen for each reachable agent.
        let mut best: Vec<Fixed> = vec![Fixed::ZERO; std::cmp::max(a, b) + 2];
        let mut visited: HashSet<usize> = HashSet::new();
        let mut queue: VecDeque<(usize, Fixed)> = VecDeque::new();

        // Seed: agent a itself has coefficient 1.0 (self)
        visited.insert(a);
        queue.push_back((a, Fixed::ONE));

        while let Some((current, current_coeff)) = queue.pop_front() {
            // If we reached b, update best
            if current == b && current_coeff > best[b] {
                best[b] = current_coeff;
            }

            // Expand neighbors
            for edge in self.edges.iter().filter(|e| e.active && e.from == current) {
                if visited.contains(&edge.to) {
                    continue;
                }
                visited.insert(edge.to);

                // Decay coefficient by edge coefficient (halves for biological links)
                let neighbor_coeff = current_coeff * edge.coefficient;

                // Only continue BFS if coefficient is still meaningful (> 0.001)
                if neighbor_coeff > Fixed::from_f64(0.001) {
                    queue.push_back((edge.to, neighbor_coeff));

                    // Update best if this is b
                    if edge.to == b && neighbor_coeff > best[b] {
                        best[b] = neighbor_coeff;
                    }
                }
            }
        }

        best[b]
    }

    /// Check if two agents are close kin (above incest taboo threshold).
    pub fn are_close_kin(&self, a: usize, b: usize) -> bool {
        self.coefficient_between(a, b) >= Fixed::from_f64(0.125)
    }

    /// Check if two agents can marry (not close kin).
    pub fn can_marry(&self, a: usize, b: usize) -> bool {
        !self.are_close_kin(a, b)
    }

    /// §10.6 (AP2): Record the kinship consequences of a marriage.
    ///
    /// Creates the directed Spouse tie between the couple (both directions)
    /// and InLaw ties between each spouse and the other's **direct biological
    /// kin** — parents, children, and siblings. The graph stores ParentChild
    /// edges in BOTH directions (parent→child and child→parent), so a parent
    /// and a child are indistinguishable from a single edge; treating all
    /// direct biological kin as family-by-marriage is the faithful reading
    /// the data model supports (step-children are in-laws in this model). All
    /// marital edges carry coefficient ZERO: they are inert for the §10.6
    /// incest taboo (`coefficient_between` / `can_marry` — biological
    /// relatedness only) and for transitive relatedness, so the only
    /// behavioral reach is the §10.3 kin-stage assignment pass, which maps
    /// InLaw edges onto the InLaw relationship stage (observational labels).
    /// Deterministic edge insertion (deduped), no RNG.
    pub fn add_marital_links(&mut self, a: usize, b: usize, tick: u64) {
        // Capture each spouse's family BEFORE adding the new edges, so the
        // fresh Spouse/InLaw ties never appear in their own in-law sets.
        let fam_a = self.family_of(a);
        let fam_b = self.family_of(b);

        // Spouse ties (the graph is directed — add both directions).
        self.add_link(a, b, KinshipLink::Spouse, tick);
        self.add_link(b, a, KinshipLink::Spouse, tick);

        // In-law ties: each spouse is in-law to the other's direct kin.
        for &k in &fam_b {
            if k != a {
                self.add_link(a, k, KinshipLink::InLaw, tick);
                self.add_link(k, a, KinshipLink::InLaw, tick);
            }
        }
        for &k in &fam_a {
            if k != b {
                self.add_link(b, k, KinshipLink::InLaw, tick);
                self.add_link(k, b, KinshipLink::InLaw, tick);
            }
        }
    }

    /// Direct biological kin of `x` — parents, children, and siblings — the
    /// family-by-blood set a marriage makes into in-laws. Deterministic:
    /// scans the active edges in insertion order, deduped.
    fn family_of(&self, x: usize) -> Vec<usize> {
        let mut fam: Vec<usize> = Vec::new();
        for e in self.edges.iter().filter(|e| e.active) {
            let kin = match e.link {
                // Both directions exist (parent→child and child→parent), so
                // any ParentChild neighbor of x is direct biological kin.
                KinshipLink::ParentChild | KinshipLink::Sibling => {
                    if e.from == x {
                        Some(e.to)
                    } else if e.to == x {
                        Some(e.from)
                    } else {
                        None
                    }
                }
                _ => None,
            };
            if let Some(k) = kin {
                if k != x && !fam.contains(&k) {
                    fam.push(k);
                }
            }
        }
        fam
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
    fn coefficient_between_parent_child() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0);
        assert_eq!(g.coefficient_between(0, 1), Fixed::from_f64(0.5));
    }

    #[test]
    fn coefficient_between_siblings() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::Sibling, 0);
        assert_eq!(g.coefficient_between(0, 1), Fixed::from_f64(0.5));
        assert_eq!(g.coefficient_between(1, 0), Fixed::from_f64(0.5));
    }

    #[test]
    fn kinship_graph_active_count() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0); // 1 edge (unidirectional)
        g.add_link(1, 2, KinshipLink::Sibling, 0); // 2 edges (bidirectional mirror)
        assert_eq!(g.active_count(), 3);
    }

    #[test]
    fn transitive_grandparent() {
        // 0 → 1 (parent), 1 → 2 (parent). Grandparent: 0 → 2 coeff should be 0.25.
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0);
        g.add_link(1, 2, KinshipLink::ParentChild, 0);
        let coeff = g.transitive_coefficient(0, 2);
        assert_eq!(coeff, Fixed::from_f64(0.25));
    }

    #[test]
    fn transitive_great_aunt_via_sibling() {
        // 0 → 1 (parent), 1 ↔ 2 (siblings), 2 → 3 (parent).
        // 0 is grandparent of 3 via 1→2→3 path.
        // But 0 is also "great-aunt/uncle" via sibling path 0↔1→2→3.
        // The BFS finds the MAX coefficient along any path.
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0);
        g.add_link(1, 2, KinshipLink::Sibling, 0);
        g.add_link(2, 3, KinshipLink::ParentChild, 0);
        // Path 0→1→2→3: 0.5 * 0.5 * 0.5 = 0.125
        // Path 1→0 (sibling mirror) is also present: 0→1 coeff=0.5
        // BFS from 0: 0→1 (0.5), 1→2 via sibling (0.5*0.5=0.25), 2→3 (0.25*0.5=0.125)
        let coeff = g.transitive_coefficient(0, 3);
        assert_eq!(coeff, Fixed::from_f64(0.125));
    }

    #[test]
    fn transitive_no_path() {
        let mut g = KinshipGraph::default();
        g.add_link(0, 1, KinshipLink::ParentChild, 0);
        g.add_link(2, 3, KinshipLink::ParentChild, 0);
        assert_eq!(g.transitive_coefficient(0, 3), Fixed::ZERO);
    }

    // ── §10.6 Marital kinship links ──────────────────────────────────

    #[test]
    fn marital_links_connect_spouses_and_in_laws() {
        let mut g = KinshipGraph::default();
        // a = 0: parents 2,3 and sibling 4.
        for p in [2usize, 3] {
            g.add_link(p, 0, KinshipLink::ParentChild, 0);
            g.add_link(0, p, KinshipLink::ParentChild, 0);
        }
        g.add_link(0, 4, KinshipLink::Sibling, 0); // mirrored automatically
                                                   // b = 1: parent 5 and sibling 6.
        g.add_link(5, 1, KinshipLink::ParentChild, 0);
        g.add_link(1, 5, KinshipLink::ParentChild, 0);
        g.add_link(1, 6, KinshipLink::Sibling, 0); // mirrored automatically

        g.add_marital_links(0, 1, 100);

        assert_eq!(g.link_between(0, 1), Some(KinshipLink::Spouse));
        assert_eq!(g.link_between(1, 0), Some(KinshipLink::Spouse));
        // b is in-law to a's parents and sibling (both directions).
        for k in [2usize, 3, 4] {
            assert_eq!(g.link_between(1, k), Some(KinshipLink::InLaw), "1↔{k}");
            assert_eq!(g.link_between(k, 1), Some(KinshipLink::InLaw), "{k}↔1");
        }
        // a is in-law to b's parent and sibling (both directions).
        for k in [5usize, 6] {
            assert_eq!(g.link_between(0, k), Some(KinshipLink::InLaw), "0↔{k}");
            assert_eq!(g.link_between(k, 0), Some(KinshipLink::InLaw), "{k}↔0");
        }
        // Marital edges are non-biological: inert for the incest taboo.
        assert_eq!(g.coefficient_between(1, 2), Fixed::ZERO);
        assert!(
            g.can_marry(1, 2),
            "in-law ties must not block courtship via coefficient"
        );
    }

    #[test]
    fn marital_links_include_children_as_family_with_no_self_edges() {
        let mut g = KinshipGraph::default();
        // a = 0 has child 7 — family-by-marriage of b = 1 in this model (the
        // graph cannot distinguish parent from child: ParentChild edges are
        // stored both directions).
        g.add_link(0, 7, KinshipLink::ParentChild, 0);
        g.add_link(7, 0, KinshipLink::ParentChild, 0);

        g.add_marital_links(0, 1, 50);

        assert_eq!(g.link_between(0, 1), Some(KinshipLink::Spouse));
        assert_eq!(g.link_between(1, 7), Some(KinshipLink::InLaw));
        assert_eq!(g.link_between(7, 1), Some(KinshipLink::InLaw));
        // No self-edges.
        assert_eq!(g.link_between(0, 0), None);
        assert_eq!(g.link_between(1, 1), None);
    }
}
