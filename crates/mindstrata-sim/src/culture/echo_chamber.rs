//! Echo chambers and polarization — network-level belief structure (§13.6).
//!
//! Echo chambers emerge when agents sort into homophilous networks,
//! institutions compete, fear rises, gossip punishes dissent, identity
//! fusion increases, and trusted bridges collapse.
//!
//! ```text
//! Echo chamber dynamics:
//!   - Agents interact mostly with similar others
//!   - Dissenting views are suppressed or punished
//!   - Narrative dominance increases within clusters
//!   - Cross-cutting ties weaken over time
//!   - Polarization index rises
//!
//! Polarization emerges from:
//!   - institutional competition
//!   - fear amplification
//!   - gossip punishing dissent
//!   - identity fusion
//!   - trusted bridge collapse
//!   - social sorting
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A cluster of agents who share similar beliefs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefCluster {
    /// Unique cluster identifier.
    pub id: usize,
    /// Dominant narrative or belief in this cluster.
    pub dominant_narrative: String,
    /// Agent IDs in this cluster.
    pub members: Vec<usize>,
    /// Internal cohesion — how strongly members agree (0 = fragmented, 1 = unanimous).
    pub cohesion: Fixed,
    /// Average emotional charge of the cluster's beliefs.
    pub emotional_charge: Fixed,
    /// How hostile the cluster is toward outgroup views (0 = tolerant, 1 = hostile).
    pub outgroup_hostility: Fixed,
    /// Degree of identity fusion (how much membership defines personal identity).
    pub identity_fusion: Fixed,
    /// Number of cross-cutting ties to other clusters.
    pub cross_cutting_ties: u32,
}

impl BeliefCluster {
    /// Create a new belief cluster.
    pub fn new(id: usize, dominant_narrative: String) -> Self {
        Self {
            id,
            dominant_narrative,
            members: Vec::new(),
            cohesion: Fixed::from_f64(0.5),
            emotional_charge: Fixed::ZERO,
            outgroup_hostility: Fixed::ZERO,
            identity_fusion: Fixed::ZERO,
            cross_cutting_ties: 0,
        }
    }

    /// Add a member to this cluster.
    pub fn add_member(&mut self, agent_id: usize) {
        if !self.members.contains(&agent_id) {
            self.members.push(agent_id);
        }
    }

    /// Remove a member from this cluster.
    pub fn remove_member(&mut self, agent_id: usize) {
        self.members.retain(|&id| id != agent_id);
    }

    /// Compute the cluster's echo chamber strength.
    ///
    /// ```text
    /// echo_strength = cohesion × identity_fusion × outgroup_hostility
    ///               × (1 / (1 + cross_cutting_ties))
    /// ```
    pub fn echo_strength(&self) -> Fixed {
        let tie_penalty = Fixed::ONE
            / (Fixed::ONE + Fixed::from_int(self.cross_cutting_ties as i64));
        (self.cohesion * self.identity_fusion * self.outgroup_hostility * tie_penalty)
            .clamp_01()
    }

    /// Tick update — cohesion changes slowly, ties may weaken.
    pub fn tick_update(&mut self) {
        // Outgroup hostility can grow if emotional charge is high
        if self.emotional_charge > Fixed::from_f64(0.6) {
            self.outgroup_hostility =
                (self.outgroup_hostility + Fixed::from_f64(0.002)).clamp_01();
        }
        // Identity fusion grows with emotional charge
        self.identity_fusion =
            (self.identity_fusion + self.emotional_charge * Fixed::from_f64(0.001))
                .clamp_01();
    }
}

/// System-level echo chamber and polarization state.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EchoChamberState {
    /// All belief clusters.
    pub clusters: Vec<BeliefCluster>,
    /// Overall polarization index (0 = consensus, 1 = fully polarized).
    pub polarization_index: Fixed,
    /// Total echo chamber strength across all clusters.
    pub echo_chamber_strength: Fixed,
    /// Total cross-cutting ties across all clusters.
    pub total_cross_cutting_ties: u32,
    /// Next cluster id.
    next_id: usize,
}

impl Default for EchoChamberState {
    fn default() -> Self {
        Self {
            clusters: Vec::new(),
            polarization_index: Fixed::ZERO,
            echo_chamber_strength: Fixed::ZERO,
            total_cross_cutting_ties: 0,
            next_id: 0,
        }
    }
}

impl EchoChamberState {
    /// Create a new echo chamber state.
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a new belief cluster and return its id.
    pub fn create_cluster(&mut self, dominant_narrative: String) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        self.clusters
            .push(BeliefCluster::new(id, dominant_narrative));
        id
    }

    /// Get a cluster by id.
    pub fn get_cluster(&self, id: usize) -> Option<&BeliefCluster> {
        self.clusters.iter().find(|c| c.id == id)
    }

    /// Get a mutable reference to a cluster by id.
    pub fn get_cluster_mut(&mut self, id: usize) -> Option<&mut BeliefCluster> {
        self.clusters.iter_mut().find(|c| c.id == id)
    }

    /// Compute overall polarization from cluster disagreements.
    ///
    /// ```text
    /// polarization = average_inter_cluster_distance
    ///              × (1 - average_cross_cutting_ties / max_possible_ties)
    /// ```
    pub fn compute_polarization(&mut self) {
        if self.clusters.len() < 2 {
            self.polarization_index = Fixed::ZERO;
            self.echo_chamber_strength = Fixed::ZERO;
            return;
        }

        // Average echo chamber strength across clusters
        let total_echo: Fixed = self
            .clusters
            .iter()
            .map(BeliefCluster::echo_strength)
            .fold(Fixed::ZERO, |acc, e| acc + e);
        let n_clusters = Fixed::from_int(self.clusters.len() as i64);
        self.echo_chamber_strength = (total_echo / n_clusters).clamp_01();

        // Polarization = emotional charge disparity + identity fusion
        let total_emotional_charge: Fixed = self
            .clusters
            .iter()
            .map(|c| c.emotional_charge)
            .fold(Fixed::ZERO, |acc, e| acc + e);
        let avg_emotional_charge = total_emotional_charge / n_clusters;

        let total_fusion: Fixed = self
            .clusters
            .iter()
            .map(|c| c.identity_fusion)
            .fold(Fixed::ZERO, |acc, f| acc + f);
        let avg_fusion = total_fusion / n_clusters;

        // Cross-cutting tie ratio (higher ties = less polarized)
        self.total_cross_cutting_ties = self
            .clusters
            .iter()
            .map(|c| c.cross_cutting_ties)
            .sum();
        let tie_ratio = if self.clusters.len() > 1 {
            let max_possible = (self.clusters.len() * (self.clusters.len() - 1)) as u32;
            Fixed::from_int(self.total_cross_cutting_ties as i64)
                / Fixed::from_int(max_possible as i64)
        } else {
            Fixed::ZERO
        };

        // Additive blend: emotional_charge + fusion + echo chamber contribute,
        // while cross-cutting ties reduce polarization. Multiplicative alone would
        // zero the output if any single factor were zero.
        let w_charge = Fixed::from_f64(0.3);
        let w_fusion = Fixed::from_f64(0.3);
        let w_echo = Fixed::from_f64(0.3);
        self.polarization_index = (w_charge * avg_emotional_charge
            + w_fusion * avg_fusion
            + w_echo * self.echo_chamber_strength)
            * (Fixed::ONE - tie_ratio)
            * (Fixed::ONE / n_clusters).max(Fixed::from_f64(0.5))
            * Fixed::from_f64(0.67) // normalize blend to [0,1]
            ;
        self.polarization_index = self.polarization_index.clamp_01();
    }

    /// Move an agent from one cluster to another.
    /// Returns `false` if either cluster ID is invalid (no-op in that case).
    pub fn move_agent(&mut self, agent_id: usize, from: usize, to: usize) -> bool {
        // Validate both clusters exist before mutating to avoid losing the agent.
        if self.get_cluster(from).is_none() || self.get_cluster(to).is_none() {
            return false;
        }
        if let Some(src) = self.get_cluster_mut(from) {
            src.remove_member(agent_id);
        }
        if let Some(dst) = self.get_cluster_mut(to) {
            dst.add_member(agent_id);
        }
        true
    }

    /// Tick update on all clusters and recompute polarization.
    pub fn tick_update(&mut self) {
        for cluster in &mut self.clusters {
            cluster.tick_update();
        }
        self.compute_polarization();
    }

    /// Number of clusters.
    pub fn cluster_count(&self) -> usize {
        self.clusters.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_cluster_has_sane_defaults() {
        let c = BeliefCluster::new(0, "the crown is just".into());
        assert_eq!(c.id, 0);
        assert!(c.members.is_empty());
        assert!(c.cohesion > Fixed::ZERO);
    }

    #[test]
    fn add_and_remove_member() {
        let mut c = BeliefCluster::new(0, "test".into());
        c.add_member(5);
        c.add_member(5); // duplicate ignored
        assert_eq!(c.members.len(), 1);
        c.remove_member(5);
        assert!(c.members.is_empty());
    }

    #[test]
    fn echo_strength_scales_with_fusion_and_hostility() {
        let mut c = BeliefCluster::new(0, "test".into());
        c.cohesion = Fixed::ONE;
        c.identity_fusion = Fixed::ONE;
        c.outgroup_hostility = Fixed::ONE;
        c.cross_cutting_ties = 0;
        let high_echo = c.echo_strength();

        let mut c2 = BeliefCluster::new(1, "test".into());
        c2.cohesion = Fixed::from_f64(0.3);
        c2.identity_fusion = Fixed::from_f64(0.1);
        c2.outgroup_hostility = Fixed::from_f64(0.1);
        c2.cross_cutting_ties = 5;
        let low_echo = c2.echo_strength();

        assert!(high_echo > low_echo);
    }

    #[test]
    fn cross_cutting_ties_reduce_echo_strength() {
        let mut c1 = BeliefCluster::new(0, "test".into());
        c1.cohesion = Fixed::ONE;
        c1.identity_fusion = Fixed::ONE;
        c1.outgroup_hostility = Fixed::ONE;
        c1.cross_cutting_ties = 0;
        let no_ties = c1.echo_strength();

        let mut c2 = c1.clone();
        c2.cross_cutting_ties = 10;
        let many_ties = c2.echo_strength();

        assert!(no_ties > many_ties);
    }

    #[test]
    fn polarization_zero_with_single_cluster() {
        let mut state = EchoChamberState::new();
        state.create_cluster("narrative A".into());
        state.compute_polarization();
        assert_eq!(state.polarization_index, Fixed::ZERO);
    }

    #[test]
    fn polarization_increases_with_more_clusters() {
        let mut state = EchoChamberState::new();
        let c1 = state.create_cluster("narrative A".into());
        let c2 = state.create_cluster("narrative B".into());
        // Set high emotional charge and fusion, no cross-cutting ties
        if let Some(cluster) = state.get_cluster_mut(c1) {
            cluster.emotional_charge = Fixed::from_f64(0.9);
            cluster.identity_fusion = Fixed::from_f64(0.9);
            cluster.outgroup_hostility = Fixed::from_f64(0.8);
            cluster.cohesion = Fixed::from_f64(0.9);
        }
        if let Some(cluster) = state.get_cluster_mut(c2) {
            cluster.emotional_charge = Fixed::from_f64(0.9);
            cluster.identity_fusion = Fixed::from_f64(0.9);
            cluster.outgroup_hostility = Fixed::from_f64(0.8);
            cluster.cohesion = Fixed::from_f64(0.9);
        }
        state.compute_polarization();
        assert!(state.polarization_index > Fixed::ZERO);
    }

    #[test]
    fn move_agent_between_clusters() {
        let mut state = EchoChamberState::new();
        let c1 = state.create_cluster("A".into());
        let c2 = state.create_cluster("B".into());
        state.get_cluster_mut(c1).unwrap().add_member(5);
        assert_eq!(state.get_cluster(c1).unwrap().members.len(), 1);
        state.move_agent(5, c1, c2);
        assert!(state.get_cluster(c1).unwrap().members.is_empty());
        assert_eq!(state.get_cluster(c2).unwrap().members.len(), 1);
    }

    #[test]
    fn registry_create_and_get() {
        let mut state = EchoChamberState::new();
        let id = state.create_cluster("test".into());
        assert_eq!(id, 0);
        assert!(state.get_cluster(0).is_some());
        assert!(state.get_cluster(1).is_none());
    }
}
