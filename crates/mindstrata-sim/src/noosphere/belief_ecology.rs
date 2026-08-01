//! §13.6: Belief ecology — narrative clusters and ideological polarization.
//!
//! Architecture §13.6: Polarization emerges when agents sort into homophilous
//! networks, institutions compete, fear rises, gossip punishes dissent,
//! identity fusion increases, and trusted bridges collapse.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A narrative cluster — a group of agents sharing similar beliefs.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeCluster {
    /// Center of the cluster in belief space (simplified: proportion along one axis).
    pub center: Fixed,
    /// Number of agents in this cluster.
    pub size: u32,
    /// Internal cohesion (0–1).
    pub cohesion: Fixed,
    /// Emotional intensity of the cluster's narrative.
    pub emotional_intensity: Fixed,
    /// Institutional backing (0–1).
    pub institutional_backing: Fixed,
}

/// Population-level belief ecology tracking polarization.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BeliefEcologyNoosphere {
    /// Active narrative clusters.
    pub clusters: Vec<NarrativeCluster>,
    /// Overall polarization index (0 = unified, 1 = maximally polarized).
    pub polarization_index: Fixed,
    /// Cross-cutting ties between clusters (0 = none, 1 = many).
    pub cross_cutting_ties: Fixed,
    /// Total meme propagation events this tick.
    pub propagation_events: u32,
}

impl BeliefEcologyNoosphere {
    /// Create a new belief ecology.
    pub fn new() -> Self {
        Self {
            clusters: Vec::new(),
            polarization_index: Fixed::from_f64(0.2),
            cross_cutting_ties: Fixed::from_f64(0.5),
            propagation_events: 0,
        }
    }

    /// Update polarization based on cluster positions and sizes.
    pub fn update_polarization(&mut self) {
        if self.clusters.len() < 2 {
            self.polarization_index = Fixed::ZERO;
            return;
        }

        // Weighted inter-cluster distance
        let mut total_weighted_distance = Fixed::ZERO;
        let mut total_weight = Fixed::ZERO;
        for i in 0..self.clusters.len() {
            for j in (i + 1)..self.clusters.len() {
                let dist = (self.clusters[i].center - self.clusters[j].center).abs();
                let weight =
                    Fixed::from_int(self.clusters[i].size as i64 + self.clusters[j].size as i64);
                total_weighted_distance = total_weighted_distance + dist * weight;
                total_weight = total_weight + weight;
            }
        }

        if total_weight > Fixed::ZERO {
            self.polarization_index =
                (total_weighted_distance / total_weight).clamp_01();
        }

        // Cross-cutting ties reduce effective polarization
        self.polarization_index = (self.polarization_index
            - self.cross_cutting_ties * Fixed::from_f64(0.15))
            .clamp_01();
    }

    /// Record a propagation event.
    pub fn record_propagation(&mut self) {
        self.propagation_events += 1;
    }

    /// Reset per-tick counters.
    pub fn reset_tick(&mut self) {
        self.propagation_events = 0;
    }

    /// Daily update — decay cohesion, adjust clusters.
    pub fn daily_update(&mut self) {
        for cluster in &mut self.clusters {
            // Cohesion slowly decays without reinforcement
            cluster.cohesion = (cluster.cohesion * Fixed::from_f64(0.99)).max(Fixed::ZERO);
            // Emotional intensity decays
            cluster.emotional_intensity =
                (cluster.emotional_intensity * Fixed::from_f64(0.98)).max(Fixed::ZERO);
        }
        // Remove empty clusters
        self.clusters.retain(|c| c.size > 0);
        // Update polarization
        self.update_polarization();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn polarization_increases_with_separated_clusters() {
        let mut ecology = BeliefEcologyNoosphere::new();
        ecology.clusters.push(NarrativeCluster {
            center: Fixed::ZERO,
            size: 5,
            cohesion: Fixed::from_f64(0.8),
            emotional_intensity: Fixed::from_f64(0.5),
            institutional_backing: Fixed::from_f64(0.3),
        });
        ecology.clusters.push(NarrativeCluster {
            center: Fixed::ONE,
            size: 5,
            cohesion: Fixed::from_f64(0.8),
            emotional_intensity: Fixed::from_f64(0.5),
            institutional_backing: Fixed::from_f64(0.3),
        });
        ecology.update_polarization();
        assert!(ecology.polarization_index > Fixed::from_f64(0.3));
    }

    #[test]
    fn single_cluster_has_zero_polarization() {
        let mut ecology = BeliefEcologyNoosphere::new();
        ecology.clusters.push(NarrativeCluster {
            center: Fixed::from_f64(0.5),
            size: 10,
            cohesion: Fixed::from_f64(0.9),
            emotional_intensity: Fixed::from_f64(0.3),
            institutional_backing: Fixed::from_f64(0.5),
        });
        ecology.update_polarization();
        assert_eq!(ecology.polarization_index, Fixed::ZERO);
    }
}
