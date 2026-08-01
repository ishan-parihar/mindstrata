//! §12.2: Group formation mechanics — emergent group cohesion and dissociation.
//!
//! Groups form when agents share proximity, repeated interaction, emotional
//! intensity, common threat, common need, shared identity, shared grievance,
//! charismatic focal agent, ritual participation, kinship, and economic
//! interdependence.
//!
//! ```text
//! group_formation_pressure =
//!     shared_grievance
//!   + shared_identity
//!   + emotional_synchrony
//!   + repeated_interaction
//!   + leadership_gravity
//!   + external_threat
//!   - social_cost
//!   - institutional_suppression
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A candidate group that may form from emergent social dynamics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupCandidate {
    /// Indices of agents in the candidate group.
    pub members: Vec<usize>,
    /// Shared grievance that binds the group (0–1).
    pub shared_grievance: Fixed,
    /// Shared identity strength (0–1).
    pub shared_identity: Fixed,
    /// Emotional synchrony — how aligned members' emotions are (0–1).
    pub emotional_synchrony: Fixed,
    /// Frequency of interaction among members (0–1).
    pub repeated_interaction: Fixed,
    /// Strength of the focal leader's gravity (0–1).
    pub leadership_gravity: Fixed,
    /// External threat level perceived by the group (0–1).
    pub external_threat: Fixed,
    /// Social cost of forming/maintaining the group (0–1).
    pub social_cost: Fixed,
    /// Institutional suppression pressure (0–1).
    pub institutional_suppression: Fixed,
    /// Tick when this candidate was first identified.
    pub identified_tick: u64,
}

impl GroupCandidate {
    /// Compute the total formation pressure.
    ///
    /// Returns a value > 0.5 suggesting the group should formalize.
    pub fn formation_pressure(&self) -> Fixed {
        let positive = self.shared_grievance * Fixed::from_f64(0.2)
            + self.shared_identity * Fixed::from_f64(0.2)
            + self.emotional_synchrony * Fixed::from_f64(0.15)
            + self.repeated_interaction * Fixed::from_f64(0.15)
            + self.leadership_gravity * Fixed::from_f64(0.15)
            + self.external_threat * Fixed::from_f64(0.15);
        let negative =
            self.social_cost * Fixed::from_f64(0.15) + self.institutional_suppression * Fixed::from_f64(0.15);
        (positive - negative).clamp_01()
    }

    /// Should this group formalize?
    pub fn should_form(&self) -> bool {
        self.formation_pressure() > Fixed::from_f64(0.5)
    }
}

/// Group type classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum GroupType {
    /// Household-based group.
    Household,
    /// Kinship-based family.
    Family,
    /// Kinship + myth + honor clan.
    Clan,
    /// Grievance + interest faction.
    Faction,
    /// Charismatic + sacred narrative cult.
    Cult,
    /// Ritual + belief congregation.
    Congregation,
    /// Skill + economic guild.
    Guild,
    /// Obligation + protection patronage network.
    Patronage,
    /// Age + proximity + identity peer group.
    PeerGroup,
    /// Violence + loyalty + survival warband.
    Warband,
}

/// Evaluate whether a set of agents should form a peer group based on
/// proximity, age similarity, and shared identity.
pub fn evaluate_peer_group(
    agent_indices: &[usize],
    agent_ages: &[Fixed],
    agent_openness: &[Fixed],
    avg_interaction_frequency: Fixed,
    _tick: u64,
) -> Option<GroupCandidate> {
    if agent_indices.len() < 3 {
        return None;
    }

    // Age similarity — standard deviation of ages
    let mean_age: Fixed = agent_ages.iter().fold(Fixed::ZERO, |acc, a| acc + *a)
        / Fixed::from_int(agent_ages.len() as i64);
    let age_variance: Fixed = agent_ages
        .iter()
        .map(|a| {
            let diff = *a - mean_age;
            diff * diff
        })
        .fold(Fixed::ZERO, |acc, v| acc + v)
        / Fixed::from_int(agent_ages.len() as i64);

    // Low age variance means age-similar group
    let age_similarity = (Fixed::ONE - age_variance * Fixed::from_f64(0.1)).clamp_01();

    // Shared identity — average openness (open agents form peer groups more easily)
    let avg_openness: Fixed = agent_openness.iter().fold(Fixed::ZERO, |acc, o| acc + *o)
        / Fixed::from_int(agent_openness.len() as i64);

    Some(GroupCandidate {
        members: agent_indices.to_vec(),
        shared_grievance: Fixed::ZERO,
        shared_identity: avg_openness * age_similarity,
        emotional_synchrony: Fixed::from_f64(0.3),
        repeated_interaction: avg_interaction_frequency,
        leadership_gravity: Fixed::from_f64(0.2),
        external_threat: Fixed::ZERO,
        social_cost: Fixed::from_f64(0.1),
        institutional_suppression: Fixed::ZERO,
        identified_tick: 0,
    })
}

/// Compute group cohesion dynamics for an existing group.
///
/// Architecture §12.2-§12.3: Group cohesion depends on shared identity,
/// repeated interaction, external threat, and institutional support.
pub fn compute_group_cohesion(
    shared_identity: Fixed,
    repeated_interaction: Fixed,
    external_threat: Fixed,
    institutional_support: Fixed,
    time_since_formation: u64,
) -> Fixed {
    // Cohesion builds over time with reinforcement
    let base = shared_identity * Fixed::from_f64(0.3)
        + repeated_interaction * Fixed::from_f64(0.3)
        + external_threat * Fixed::from_f64(0.2) // shared threat bonds
        + institutional_support * Fixed::from_f64(0.2);

    // Cohesion builds slowly with time (up to a cap)
    let time_bonus = Fixed::from_f64(time_since_formation as f64 * 0.0001).min(Fixed::from_f64(0.1));

    (base + time_bonus).clamp_01()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formation_pressure_computed() {
        let candidate = GroupCandidate {
            members: vec![0, 1, 2],
            shared_grievance: Fixed::from_f64(0.7),
            shared_identity: Fixed::from_f64(0.6),
            emotional_synchrony: Fixed::from_f64(0.5),
            repeated_interaction: Fixed::from_f64(0.4),
            leadership_gravity: Fixed::from_f64(0.3),
            external_threat: Fixed::from_f64(0.6),
            social_cost: Fixed::from_f64(0.1),
            institutional_suppression: Fixed::from_f64(0.1),
            identified_tick: 0,
        };
        let pressure = candidate.formation_pressure();
        assert!(pressure > Fixed::from_f64(0.3));
    }

    #[test]
    fn group_cohesion_increases_with_shared_identity() {
        let low = compute_group_cohesion(
            Fixed::from_f64(0.2),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.1),
            100,
        );
        let high = compute_group_cohesion(
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.3),
            Fixed::from_f64(0.1),
            Fixed::from_f64(0.1),
            100,
        );
        assert!(high > low);
    }

    #[test]
    fn peer_group_needs_minimum_members() {
        let result = evaluate_peer_group(&[0, 1], &[Fixed::from_f64(25.0), Fixed::from_f64(26.0)], &[Fixed::from_f64(0.5), Fixed::from_f64(0.6)], Fixed::from_f64(0.3), 0);
        assert!(result.is_none());
    }
}
