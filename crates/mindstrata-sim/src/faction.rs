//! Faction entities — emergent political groups.
//!
//! §26: A faction has collective identity, leadership, cohesion,
//! grievance, mobilization capacity, and an informal communication network.
//!
//! Factions should EMERGE from the simulation, not be hardcoded.
//! But we provide the data structures so that coalition formation,
//! polarization, and collective action can arise.

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// An emergent political/social faction.
///
/// §26: "A faction has collective identity, leadership, cohesion,
/// grievance, mobilization capacity, and informal communication network."
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Faction {
    /// Unique identifier.
    pub id: u64,
    /// Human-readable name (emerges from member identities).
    pub name: String,
    /// Core belief or grievance that unites the faction.
    pub core_belief_proposition_id: u64,
    /// Leaders of the faction (ordered by influence).
    pub leaders: Vec<AgentId>,
    /// All members.
    pub members: Vec<AgentId>,
    /// Internal cohesion: how united are the members? (0..1)
    pub cohesion: Fixed,
    /// Collective grievance: how much does the faction feel wronged? (0..1)
    pub grievance: Fixed,
    /// Mobilization capacity: can the faction act collectively? (0..1)
    pub mobilization_capacity: Fixed,
    /// Ideological rigidity: resistance to changing core beliefs.
    pub ideological_rigidity: Fixed,
    /// Creation tick.
    pub formed_tick: u64,
    /// Is this faction still active?
    pub active: bool,
}

impl Faction {
    /// Create a new faction.
    pub fn new(
        id: u64,
        name: String,
        core_belief_proposition_id: u64,
        founder: AgentId,
        formed_tick: u64,
    ) -> Self {
        Self {
            id,
            name,
            core_belief_proposition_id,
            leaders: vec![founder],
            members: vec![founder],
            cohesion: Fixed::from_f64(0.7),
            grievance: Fixed::from_f64(0.3),
            mobilization_capacity: Fixed::from_f64(0.5),
            ideological_rigidity: Fixed::from_f64(0.5),
            formed_tick,
            active: true,
        }
    }

    /// Add a member to the faction.
    pub fn add_member(&mut self, agent: AgentId) {
        if !self.members.contains(&agent) {
            self.members.push(agent);
        }
    }

    /// Remove a member from the faction.
    pub fn remove_member(&mut self, agent: AgentId) {
        self.members.retain(|m| *m != agent);
        self.leaders.retain(|l| *l != agent);
        if self.members.is_empty() {
            self.active = false;
        }
    }

    /// Check if an agent is a member.
    pub fn has_member(&self, agent: AgentId) -> bool {
        self.members.contains(&agent)
    }

    /// Derive mobilization capacity from member count and cohesion.
    pub fn derive_mobilization(&mut self) {
        let member_factor = Fixed::from_f64((self.members.len() as f64 / 20.0).min(1.0));
        self.mobilization_capacity = (member_factor * self.cohesion).clamp_01();
    }

    /// Increase grievance (e.g., from perceived injustice).
    pub fn increase_grievance(&mut self, amount: Fixed) {
        self.grievance = (self.grievance + amount).clamp_01();
    }

    /// Decrease grievance (e.g., from reform or satisfaction).
    pub fn decrease_grievance(&mut self, amount: Fixed) {
        self.grievance = (self.grievance - amount).max(Fixed::ZERO);
    }

    /// Increase cohesion (e.g., from shared experience or threat).
    pub fn increase_cohesion(&mut self, amount: Fixed) {
        self.cohesion = (self.cohesion + amount).clamp_01();
    }

    /// Decrease cohesion (e.g., from internal conflict).
    pub fn decrease_cohesion(&mut self, amount: Fixed) {
        self.cohesion = (self.cohesion - amount).max(Fixed::ZERO);
        if self.cohesion < Fixed::from_f64(0.1) {
            self.active = false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn faction_creation() {
        let faction = Faction::new(0, "Reformers".into(), 6, AgentId::new(0), 100);
        assert_eq!(faction.leaders.len(), 1);
        assert_eq!(faction.members.len(), 1);
        assert!(faction.active);
    }

    #[test]
    fn faction_member_management() {
        let mut faction = Faction::new(0, "Test".into(), 0, AgentId::new(0), 0);
        faction.add_member(AgentId::new(1));
        assert!(faction.has_member(AgentId::new(1)));
        assert_eq!(faction.members.len(), 2);

        faction.remove_member(AgentId::new(0));
        assert!(!faction.has_member(AgentId::new(0)));
        assert_eq!(faction.members.len(), 1);
    }

    #[test]
    fn faction_dissolves_when_empty() {
        let mut faction = Faction::new(0, "Test".into(), 0, AgentId::new(0), 0);
        faction.remove_member(AgentId::new(0));
        assert!(!faction.active);
    }

    #[test]
    fn grievance_and_cohesion() {
        let mut faction = Faction::new(0, "Test".into(), 0, AgentId::new(0), 0);
        // Initial grievance is 0.3
        assert!(faction.grievance > Fixed::ZERO, "Initial grievance should be positive");
        faction.increase_grievance(Fixed::from_f64(0.2));
        let after_increase = faction.grievance;
        faction.decrease_grievance(Fixed::from_f64(0.1));
        assert!(faction.grievance < after_increase, "Grievance should decrease after decrease_grievance");
        faction.increase_cohesion(Fixed::from_f64(0.1));
        assert!(faction.cohesion > Fixed::from_f64(0.7), "Cohesion should increase");
    }

    #[test]
    fn mobilization_derivation() {
        let mut faction = Faction::new(0, "Test".into(), 0, AgentId::new(0), 0);
        for i in 1..10 {
            faction.add_member(AgentId::new(i));
        }
        faction.derive_mobilization();
        assert!(faction.mobilization_capacity > Fixed::ZERO);
    }
}
