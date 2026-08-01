//! §5.2, §29.2: Faction v2 — upgraded faction system with mobilization, morale,
//! supplies, leadership dynamics, casualties, and legitimacy of violence.
//!
//! Architecture §29.2: "Factions and institutions need mobilization capacity,
//! morale, supplies, leadership, cohesion, casualties, legitimacy of violence."
//!
//! This module extends the existing `factions.rs` (grievance, recruitment,
//! protest, revolution) with richer faction-internal dynamics.
//!
//! ```text
//! Faction v2 dynamics:
//!   - Mobilization capacity: how many agents can the faction field?
//!   - Morale: collective emotional state driving willingness to act
//!   - Supplies: material resources sustaining faction activity
//!   - Leadership: leader competence, trust, succession pressure
//!   - Cohesion: internal unity vs fragmentation
//!   - Casualties: losses from conflict reduce capacity and morale
//!   - Legitimacy of violence: perceived rightfulness of force
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Internal state of a faction's mobilization and combat readiness.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FactionV2 {
    /// Index of the faction leader agent.
    pub leader: usize,
    /// Member agent indices.
    pub members: Vec<usize>,
    /// Overall grievance level driving the faction (0–1).
    pub grievance: Fixed,
    /// Mobilization capacity — fraction of members willing to act (0–1).
    pub mobilization_capacity: Fixed,
    /// Collective morale — willingness to fight/sacrifice (0–1).
    pub morale: Fixed,
    /// Supply level — material resources available (0–1).
    pub supplies: Fixed,
    /// Supply consumption rate per tick.
    pub supply_consumption: Fixed,
    /// Leadership competence (0–1).
    pub leader_competence: Fixed,
    /// Trust in leadership from members (0–1).
    pub trust_in_leadership: Fixed,
    /// Internal cohesion — unity vs fragmentation (0–1).
    pub cohesion: Fixed,
    /// Fragmentation pressure — dissent, internal rivalry (0–1).
    pub fragmentation: Fixed,
    /// Cumulative casualties (0–1 of original strength).
    pub casualties: Fixed,
    /// Legitimacy of violence — perceived rightfulness of force (0–1).
    pub legitimacy_of_violence: Fixed,
    /// Whether the faction is currently active.
    pub active: bool,
    /// Tick when the faction was formed.
    pub formed_tick: u64,
    /// Tick of last significant event (protest, conflict, etc.).
    pub last_event_tick: u64,
}

impl FactionV2 {
    /// Create a new faction v2 from grievance and initial member count.
    pub fn new(
        leader: usize,
        members: Vec<usize>,
        grievance: Fixed,
        tick: u64,
    ) -> Self {
        let member_count = Fixed::from_int(members.len() as i64);
        // Higher grievance = higher initial morale and mobilization
        let morale = (grievance * Fixed::from_f64(0.8) + Fixed::from_f64(0.2)).clamp_01();
        let mobilization = (grievance * Fixed::from_f64(0.6) + Fixed::from_f64(0.1)).clamp_01();
        // Legitimacy of violence starts low — grows with perceived injustice
        let legitimacy_of_violence = (grievance * Fixed::from_f64(0.3)).clamp_01();

        Self {
            leader,
            members,
            grievance,
            mobilization_capacity: mobilization,
            morale,
            supplies: Fixed::from_f64(0.7), // start with moderate supplies
            supply_consumption: Fixed::from_f64(0.001),
            leader_competence: Fixed::from_f64(0.5),
            trust_in_leadership: Fixed::from_f64(0.6),
            cohesion: (grievance * Fixed::from_f64(0.7) + Fixed::from_f64(0.2)).clamp_01(),
            fragmentation: Fixed::from_f64(0.1),
            casualties: Fixed::ZERO,
            legitimacy_of_violence,
            active: true,
            formed_tick: tick,
            last_event_tick: tick,
        }
    }

    /// Compute effective fighting strength = mobilization × morale × (1 − casualties).
    pub fn fighting_strength(&self) -> Fixed {
        let base = self.mobilization_capacity * self.morale;
        let casualty_penalty = Fixed::ONE - self.casualties;
        (base * casualty_penalty).clamp_01()
    }

    /// Compute faction threat level to institutions.
    ///
    /// Threat = fighting_strength × cohesion × grievance × (1 − fragmentation).
    pub fn threat_level(&self) -> Fixed {
        let strength = self.fighting_strength();
        let cohesion_factor = self.cohesion * (Fixed::ONE - self.fragmentation);
        (strength * cohesion_factor * self.grievance).clamp_01()
    }

    /// Record casualties from a conflict or crackdown.
    ///
    /// Also increases grievance and legitimacy_of_violence (martyrdom effect):
    /// casualties radicalize survivors and make violence feel more justified.
    pub fn record_casualties(&mut self, loss_fraction: Fixed) {
        self.casualties = (self.casualties + loss_fraction).clamp_01();
        // Casualties reduce morale and mobilization
        let morale_hit = loss_fraction * Fixed::from_f64(0.5);
        self.morale = (self.morale - morale_hit).max(Fixed::ZERO);
        self.mobilization_capacity = (self.mobilization_capacity - loss_fraction * Fixed::from_f64(0.3))
            .max(Fixed::ZERO);
        // But casualties can increase grievance and legitimacy of violence (martyrdom effect)
        self.grievance = (self.grievance + loss_fraction * Fixed::from_f64(0.1)).clamp_01();
        self.legitimacy_of_violence = (self.legitimacy_of_violence
            + loss_fraction * Fixed::from_f64(0.05))
            .clamp_01();
    }

    /// Daily update — decay morale, consume supplies, adjust cohesion.
    pub fn daily_update(&mut self) {
        // Supplies decay with consumption
        self.supplies = (self.supplies - self.supply_consumption).max(Fixed::ZERO);
        // Low supplies reduce morale
        if self.supplies < Fixed::from_f64(0.3) {
            let supply_penalty = (Fixed::from_f64(0.3) - self.supplies) * Fixed::from_f64(0.1);
            self.morale = (self.morale - supply_penalty).max(Fixed::ZERO);
        }
        // Morale slowly decays without reinforcement
        self.morale = (self.morale * Fixed::from_f64(0.998)).max(Fixed::ZERO);
        // Fragmentation grows slowly if cohesion is low
        if self.cohesion < Fixed::from_f64(0.4) {
            self.fragmentation = (self.fragmentation + Fixed::from_f64(0.001)).clamp_01();
        }
        // Cohesion slowly decays without shared activity
        self.cohesion = (self.cohesion * Fixed::from_f64(0.999)).max(Fixed::ZERO);
        // Trust in leadership decays if leader is incompetent
        if self.leader_competence < Fixed::from_f64(0.3) {
            self.trust_in_leadership =
                (self.trust_in_leadership - Fixed::from_f64(0.002)).max(Fixed::ZERO);
        }
    }

    /// Check if the faction should dissolve.
    pub fn should_dissolve(&self) -> bool {
        // Dissolve if: too few members, morale collapsed, or casualties too high
        self.members.len() < 2
            || self.morale < Fixed::from_f64(0.1)
            || self.casualties > Fixed::from_f64(0.7)
    }

    /// Compute the recruitment chance for a potential recruit.
    ///
    /// Returns a probability (0–1) that the recruit will join.
    /// The caller should roll against this probability.
    pub fn recruitment_chance(
        &self,
        recruit_grievance: Fixed,
        max_members: usize,
    ) -> Fixed {
        if self.members.len() >= max_members {
            return Fixed::ZERO;
        }
        if recruit_grievance < Fixed::from_f64(0.4) {
            return Fixed::ZERO;
        }
        // Recruitment chance based on faction morale and recruit grievance
        (self.morale * recruit_grievance).clamp_01()
    }
}

/// Registry of all factions v2 in the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct FactionV2Registry {
    pub factions: Vec<FactionV2>,
}

impl FactionV2Registry {
    /// Create an empty registry.
    pub fn new() -> Self {
        Self {
            factions: Vec::new(),
        }
    }

    /// Register a new faction and return its index.
    pub fn register(&mut self, faction: FactionV2) -> usize {
        let idx = self.factions.len();
        self.factions.push(faction);
        idx
    }

    /// Get all active factions.
    pub fn active(&self) -> Vec<(usize, &FactionV2)> {
        self.factions
            .iter()
            .enumerate()
            .filter(|(_, f)| f.active)
            .collect()
    }

    /// Daily update for all active factions.
    pub fn daily_update(&mut self) {
        for faction in &mut self.factions {
            if faction.active {
                faction.daily_update();
                if faction.should_dissolve() {
                    faction.active = false;
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fighting_strength_computed() {
        let f = FactionV2::new(0, vec![0, 1, 2], Fixed::from_f64(0.7), 0);
        let strength = f.fighting_strength();
        assert!(strength > Fixed::ZERO);
        assert!(strength <= Fixed::ONE);
    }

    #[test]
    fn threat_level_includes_cohesion() {
        let mut f1 = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let mut f2 = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        f1.cohesion = Fixed::from_f64(0.9);
        f2.cohesion = Fixed::from_f64(0.2);
        assert!(f1.threat_level() > f2.threat_level());
    }

    #[test]
    fn casualties_reduce_strength() {
        let mut f = FactionV2::new(0, vec![0, 1, 2], Fixed::from_f64(0.7), 0);
        let before = f.fighting_strength();
        f.record_casualties(Fixed::from_f64(0.3));
        assert!(f.fighting_strength() < before);
    }

    #[test]
    fn casualties_increase_grievance() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.5), 0);
        let before = f.grievance;
        f.record_casualties(Fixed::from_f64(0.2));
        assert!(f.grievance > before);
    }

    #[test]
    fn dissolve_on_low_morale() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        f.morale = Fixed::from_f64(0.05);
        assert!(f.should_dissolve());
    }

    #[test]
    fn daily_update_consumes_supplies() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        let before = f.supplies;
        f.daily_update();
        assert!(f.supplies < before);
    }

    #[test]
    fn recruit_requires_grievance() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        // Low grievance recruit should fail
        assert!(!f.attempt_recruit(Fixed::from_f64(0.2), 10));
    }

    #[test]
    fn recruit_fails_at_capacity() {
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        assert!(!f.attempt_recruit(Fixed::from_f64(0.8), 2));
    }

    #[test]
    fn registry_daily_update_deactivates_dead_factions() {
        let mut reg = FactionV2Registry::new();
        let mut f = FactionV2::new(0, vec![0, 1], Fixed::from_f64(0.7), 0);
        f.morale = Fixed::from_f64(0.05); // will dissolve
        reg.register(f);
        reg.daily_update();
        assert!(!reg.factions[0].active);
    }
}
