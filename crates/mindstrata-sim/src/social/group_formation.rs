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
    Household,
    Family,
    Clan,
    Faction,
    Cult,
    Congregation,
    Guild,
    Patronage,
    PeerGroup,
    Warband,
}

/// Type alias for group registry IDs.
pub type GroupId = usize;

/// A formalized peer group that emerged from group formation pressure.
///
/// Architecture §12.2: Groups form when agents share proximity, repeated
/// interaction, emotional synchrony, shared grievance, or external threat.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerGroup {
    pub id: GroupId,
    pub members: Vec<usize>,
    pub leader: Option<usize>,
    pub group_type: GroupType,
    pub shared_grievance: Fixed,
    pub shared_identity: Fixed,
    /// Current group cohesion (decays without reinforcement).
    pub cohesion: Fixed,
    pub formed_tick: u64,
    pub last_interaction_tick: u64,
    pub active: bool,
}

impl PeerGroup {
    /// Create a new peer group from a GroupCandidate.
    pub fn from_candidate(candidate: &GroupCandidate, id: GroupId, tick: u64) -> Self {
        let leader = candidate.members.first().copied();
        let cohesion = compute_group_cohesion(
            candidate.shared_identity,
            candidate.repeated_interaction,
            candidate.external_threat,
            Fixed::ZERO,
            0,
        );
        Self {
            id,
            members: candidate.members.clone(),
            leader,
            group_type: GroupType::PeerGroup,
            shared_grievance: candidate.shared_grievance,
            shared_identity: candidate.shared_identity,
            cohesion,
            formed_tick: tick,
            last_interaction_tick: tick,
            active: true,
        }
    }

    /// Daily update — decay cohesion and check dissolution.
    pub fn daily_update(&mut self) {
        self.cohesion = (self.cohesion * Fixed::from_f64(0.995)).clamp_01();
        if self.cohesion < Fixed::from_f64(0.1) || self.members.len() < 2 {
            self.active = false;
        }
    }
}

/// Registry for all active peer groups in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GroupRegistry {
    pub groups: Vec<PeerGroup>,
}

impl GroupRegistry {
    pub fn new() -> Self {
        Self { groups: Vec::new() }
    }

    pub fn register(&mut self, group: PeerGroup) -> GroupId {
        let id = self.groups.len();
        self.groups.push(group);
        id
    }

    pub fn get(&self, id: GroupId) -> Option<&PeerGroup> {
        self.groups.get(id)
    }

    pub fn get_mut(&mut self, id: GroupId) -> Option<&mut PeerGroup> {
        self.groups.get_mut(id)
    }

    /// Return all active groups.
    pub fn active(&self) -> Vec<(GroupId, &PeerGroup)> {
        self.groups.iter().enumerate()
            .filter(|(_, g)| g.active)
            .map(|(i, g)| (i, g))
            .collect()
    }

    /// Daily update for all groups — decay cohesion.
    pub fn daily_update(&mut self) {
        for group in &mut self.groups {
            if group.active {
                group.daily_update();
            }
        }
    }

    /// Check if an agent is already in an active group.
    pub fn agent_in_active_group(&self, agent_idx: usize) -> bool {
        self.groups.iter()
            .filter(|g| g.active)
            .any(|g| g.members.contains(&agent_idx))
    }
}

/// Evaluate whether a set of agents should form a peer group.
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
    let mean_age: Fixed = agent_ages.iter().fold(Fixed::ZERO, |acc, a| acc + *a)
        / Fixed::from_int(agent_ages.len() as i64);
    let age_variance: Fixed = agent_ages.iter()
        .map(|a| { let diff = *a - mean_age; diff * diff })
        .fold(Fixed::ZERO, |acc, v| acc + v)
        / Fixed::from_int(agent_ages.len() as i64);
    let age_similarity = (Fixed::ONE - age_variance * Fixed::from_f64(0.1)).clamp_01();
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
pub fn compute_group_cohesion(
    shared_identity: Fixed,
    repeated_interaction: Fixed,
    external_threat: Fixed,
    institutional_support: Fixed,
    time_since_formation: u64,
) -> Fixed {
    let base = shared_identity * Fixed::from_f64(0.3)
        + repeated_interaction * Fixed::from_f64(0.3)
        + external_threat * Fixed::from_f64(0.2)
        + institutional_support * Fixed::from_f64(0.2);
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
        assert!(candidate.formation_pressure() > Fixed::from_f64(0.3));
    }

    #[test]
    fn group_cohesion_increases_with_shared_identity() {
        let low = compute_group_cohesion(
            Fixed::from_f64(0.2), Fixed::from_f64(0.3),
            Fixed::from_f64(0.1), Fixed::from_f64(0.1), 100,
        );
        let high = compute_group_cohesion(
            Fixed::from_f64(0.8), Fixed::from_f64(0.3),
            Fixed::from_f64(0.1), Fixed::from_f64(0.1), 100,
        );
        assert!(high > low);
    }

    #[test]
    fn peer_group_needs_minimum_members() {
        let result = evaluate_peer_group(
            &[0, 1],
            &[Fixed::from_f64(25.0), Fixed::from_f64(26.0)],
            &[Fixed::from_f64(0.5), Fixed::from_f64(0.6)],
            Fixed::from_f64(0.3), 0,
        );
        assert!(result.is_none());
    }

    #[test]
    fn group_registry_register_and_active() {
        let mut reg = GroupRegistry::new();
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
        let group = PeerGroup::from_candidate(&candidate, 0, 100);
        let id = reg.register(group);
        assert_eq!(id, 0);
        assert_eq!(reg.active().len(), 1);
    }

    #[test]
    fn peer_group_dissolves_on_low_cohesion() {
        let mut group = PeerGroup {
            id: 0,
            members: vec![0, 1],
            leader: Some(0),
            group_type: GroupType::PeerGroup,
            shared_grievance: Fixed::ZERO,
            shared_identity: Fixed::ZERO,
            cohesion: Fixed::from_f64(0.05),
            formed_tick: 0,
            last_interaction_tick: 0,
            active: true,
        };
        group.daily_update();
        assert!(!group.active);
    }
}
