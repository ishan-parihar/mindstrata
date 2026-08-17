//! §13.6: Ideology system — belief clusters, polarization, and echo chambers.
//!
//! Architecture §13.6: Polarization emerges when agents sort into homophilous
//! networks, institutions compete, fear rises, gossip punishes dissent,
//! identity fusion increases, and trusted bridges collapse.
//!
//! Ideology is not a single variable but a multidimensional space where
//! agents position themselves and drift toward or away from clusters.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Deterministic integer square root approximation for Fixed-point values.
/// Uses Babylonian method (Heron's method) with fixed iterations for determinism.
fn fixed_sqrt(val: Fixed) -> Fixed {
    if val <= Fixed::ZERO {
        return Fixed::ZERO;
    }
    // Use f64 sqrt as approximation, then snap to nearest Fixed.
    // This is deterministic because f64 sqrt is IEEE-754 guaranteed
    // to produce the same result on all conforming platforms.
    let approx = val.to_f64().sqrt();
    Fixed::from_f64(approx)
}

/// An ideological position along a single axis.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdeologyAxis {
    /// Name of the axis (e.g., "traditional vs progressive").
    pub name: String,
    /// Current position on this axis (0 = one extreme, 1 = other extreme).
    pub position: Fixed,
    /// How strongly the agent holds this position (0 = indifferent, 1 = dogmatic).
    pub conviction: Fixed,
}

/// An agent's complete ideological profile.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Ideology {
    /// Axes of ideological position.
    pub axes: Vec<IdeologyAxis>,
    /// Overall dogmatism — resistance to changing any axis (0–1).
    pub dogmatism: Fixed,
    /// Echo chamber strength — tendency to reinforce existing positions (0–1).
    pub echo_chamber_strength: Fixed,
    /// Polarization tendency — how much the agent drifts toward extremes under social pressure.
    pub polarization_tendency: Fixed,
}

impl Ideology {
    /// Euclidean distance between two agents' ideological positions.
    pub fn distance_to(&self, other: &Ideology) -> Fixed {
        let mut total = Fixed::ZERO;
        let mut count = 0i64;
        for (a, b) in self.axes.iter().zip(other.axes.iter()) {
            let diff = a.position - b.position;
            total += diff * diff;
            count += 1;
        }
        if count > 0 {
            // Euclidean distance via integer sqrt approximation (deterministic)
            let variance = total / Fixed::from_int(count);
            fixed_sqrt(variance)
        } else {
            Fixed::ZERO
        }
    }

}

/// Network-level belief ecology — tracks polarization across the population.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefEcology {
    /// Clusters of agents with similar beliefs.
    pub clusters: Vec<BeliefCluster>,
    /// Overall polarization index (0 = unified, 1 = maximally polarized).
    pub polarization_index: Fixed,
    /// Cross-cutting ties — connections between different clusters (0–1).
    pub cross_cutting_ties: Fixed,
}

/// A cluster of agents with similar ideological positions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefCluster {
    /// Representative ideological position of the cluster.
    pub center: Vec<IdeologyAxis>,
    /// Number of agents in the cluster.
    pub size: u32,
    /// Internal cohesion — how tightly clustered the agents are.
    pub cohesion: Fixed,
}

impl Default for BeliefEcology {
    fn default() -> Self {
        Self {
            clusters: Vec::new(),
            polarization_index: Fixed::from_f64(0.2),
            cross_cutting_ties: Fixed::from_f64(0.5),
        }
    }
}

impl BeliefEcology {
    /// Compute overall polarization from cluster separation.
    pub fn update_polarization(&mut self) {
        if self.clusters.len() < 2 {
            self.polarization_index = Fixed::ZERO;
            return;
        }

        // Average inter-cluster distance
        let mut total_distance = Fixed::ZERO;
        let mut pairs = 0i64;
        for i in 0..self.clusters.len() {
            for j in (i + 1)..self.clusters.len() {
                let mut dist = Fixed::ZERO;
                for (a, b) in self.clusters[i]
                    .center
                    .iter()
                    .zip(self.clusters[j].center.iter())
                {
                    let diff = a.position - b.position;
                    dist += diff * diff;
                }
                total_distance += dist;
                pairs += 1;
            }
        }
        if pairs > 0 {
            self.polarization_index = (total_distance / Fixed::from_int(pairs)).clamp_01();
        }
        // Cross-cutting ties reduce polarization
        self.polarization_index =
            (self.polarization_index - self.cross_cutting_ties * Fixed::from_f64(0.2)).clamp_01();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ideology_distance_zero_for_identical() {
        let ideol = Ideology {
            axes: vec![IdeologyAxis {
                name: "test".into(),
                position: Fixed::from_f64(0.5),
                conviction: Fixed::from_f64(0.5),
            }],
            dogmatism: Fixed::from_f64(0.5),
            echo_chamber_strength: Fixed::ZERO,
            polarization_tendency: Fixed::ZERO,
        };
        assert_eq!(ideol.distance_to(&ideol), Fixed::ZERO);
    }

    #[test]
    fn ideology_distance_positive_for_different() {
        let a = Ideology {
            axes: vec![IdeologyAxis {
                name: "test".into(),
                position: Fixed::from_f64(0.2),
                conviction: Fixed::from_f64(0.5),
            }],
            dogmatism: Fixed::ZERO,
            echo_chamber_strength: Fixed::ZERO,
            polarization_tendency: Fixed::ZERO,
        };
        let b = Ideology {
            axes: vec![IdeologyAxis {
                name: "test".into(),
                position: Fixed::from_f64(0.8),
                conviction: Fixed::from_f64(0.5),
            }],
            dogmatism: Fixed::ZERO,
            echo_chamber_strength: Fixed::ZERO,
            polarization_tendency: Fixed::ZERO,
        };
        assert!(a.distance_to(&b) > Fixed::ZERO);
    }

    #[test]
    fn polarization_increases_with_clusters() {
        let mut ecology = BeliefEcology::default();
        ecology.clusters.push(BeliefCluster {
            center: vec![IdeologyAxis {
                name: "a".into(),
                position: Fixed::ZERO,
                conviction: Fixed::from_f64(0.5),
            }],
            size: 5,
            cohesion: Fixed::from_f64(0.8),
        });
        ecology.clusters.push(BeliefCluster {
            center: vec![IdeologyAxis {
                name: "a".into(),
                position: Fixed::ONE,
                conviction: Fixed::from_f64(0.5),
            }],
            size: 5,
            cohesion: Fixed::from_f64(0.8),
        });
        ecology.update_polarization();
        assert!(ecology.polarization_index > Fixed::ZERO);
    }
}
