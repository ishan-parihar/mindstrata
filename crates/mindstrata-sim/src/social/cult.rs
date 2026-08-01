//! §12.4: Cult formation model — emergent high-intensity group type.
//!
//! Cults emerge from high meaning need, high fear, low institutional legitimacy,
//! charismatic leaders, strong boundary markers, repeated ritual, social isolation,
//! and identity fusion.
//!
//! Cult dissolution: leader failure, prophecy disconfirmation, internal betrayal,
//! external repression, member exhaustion, rival narrative, economic collapse.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Cult formation and dissolution dynamics.
///
/// Architecture §12.4: Cults are a special high-intensity group type
/// that emerges under specific social-psychological conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultDynamics {
    /// Index of the charismatic leader.
    pub charismatic_leader: usize,
    /// Sacred narrative meme id that binds the cult.
    pub sacred_narrative_id: u32,
    /// Identity fusion — how much members fuse their self-concept with the group.
    pub identity_fusion: Fixed,
    /// Social isolation from the broader community.
    pub isolation: Fixed,
    /// Member dependence on the leader.
    pub dependence: Fixed,
    /// Fear load — fear of leaving or dissenting.
    pub fear_load: Fixed,
    /// Love bombing intensity — early recruitment tactic.
    pub love_bombing: Fixed,
    /// Strictness of in-group/out-group boundaries.
    pub boundary_strictness: Fixed,
    /// Thought-terminating clichés — phrases that shut down critical thinking.
    pub thought_terminating_cliches: Fixed,
    /// Cost of leaving the cult (social, material, psychological).
    pub exit_cost: Fixed,
    /// Tick when the cult was formed.
    pub formed_tick: u64,
    /// Whether the cult is currently active.
    pub active: bool,
}

impl CultDynamics {
    /// Create a new cult dynamics instance.
    pub fn new(leader: usize, sacred_narrative_id: u32, tick: u64) -> Self {
        Self {
            charismatic_leader: leader,
            sacred_narrative_id,
            identity_fusion: Fixed::from_f64(0.3),
            isolation: Fixed::from_f64(0.2),
            dependence: Fixed::from_f64(0.4),
            fear_load: Fixed::from_f64(0.1),
            love_bombing: Fixed::from_f64(0.5),
            boundary_strictness: Fixed::from_f64(0.3),
            thought_terminating_cliches: Fixed::from_f64(0.2),
            exit_cost: Fixed::from_f64(0.2),
            formed_tick: tick,
            active: true,
        }
    }

    /// Compute cult intensity — a composite measure of the cult's grip on members.
    pub fn intensity(&self) -> Fixed {
        (self.identity_fusion * Fixed::from_f64(0.2)
            + self.isolation * Fixed::from_f64(0.15)
            + self.dependence * Fixed::from_f64(0.2)
            + self.fear_load * Fixed::from_f64(0.15)
            + self.boundary_strictness * Fixed::from_f64(0.1)
            + self.thought_terminating_cliches * Fixed::from_f64(0.1)
            + self.exit_cost * Fixed::from_f64(0.1))
            .clamp_01()
    }

    /// Check if the cult should dissolve based on current conditions.
    ///
    /// Architecture §12.4: Cult dissolution conditions include leader failure,
    /// prophecy disconfirmation, internal betrayal, external repression,
    /// member exhaustion, rival narrative, and economic collapse.
    pub fn should_dissolve(
        &self,
        leader_competence: Fixed,
        institutional_legitimacy: Fixed,
        member_average_fatigue: Fixed,
    ) -> bool {
        // Leader failure — leader competence drops below threshold
        if leader_competence < Fixed::from_f64(0.2) && self.dependence > Fixed::from_f64(0.5) {
            return true;
        }
        // External repression — institutional legitimacy overwhelms cult
        if institutional_legitimacy > Fixed::from_f64(0.8) && self.isolation < Fixed::from_f64(0.3) {
            return true;
        }
        // Member exhaustion — fatigue undermines cohesion
        if member_average_fatigue > Fixed::from_f64(0.8) {
            return true;
        }
        false
    }

    /// Daily update — decay cult intensity if not reinforced.
    pub fn daily_update(&mut self) {
        // Love bombing decays — it's a recruitment phase, not sustainable
        self.love_bombing = (self.love_bombing * Fixed::from_f64(0.98)).max(Fixed::ZERO);
        // Fear load slowly decays if not reinforced by leader
        self.fear_load = (self.fear_load * Fixed::from_f64(0.99)).max(Fixed::ZERO);
        // Identity fusion is sticky but decays slowly without reinforcement
        self.identity_fusion = (self.identity_fusion * Fixed::from_f64(0.995)).max(Fixed::ZERO);
        // Isolation can grow if boundaries are strict
        self.isolation = (self.isolation + self.boundary_strictness * Fixed::from_f64(0.001))
            .clamp_01();
    }
}

/// Registry of all cults in the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CultRegistry {
    pub cults: Vec<CultDynamics>,
}

impl CultRegistry {
    pub fn new() -> Self {
        Self { cults: Vec::new() }
    }

    /// Register a new cult and return its index.
    pub fn register(&mut self, cult: CultDynamics) -> usize {
        let idx = self.cults.len();
        self.cults.push(cult);
        idx
    }

    /// Daily update for all active cults.
    pub fn daily_update(&mut self) {
        for cult in &mut self.cults {
            if cult.active {
                cult.daily_update();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cult_intensity_computed() {
        let cult = CultDynamics::new(0, 1, 0);
        let intensity = cult.intensity();
        assert!(intensity > Fixed::ZERO);
        assert!(intensity <= Fixed::ONE);
    }

    #[test]
    fn cult_dissolves_on_leader_failure() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.dependence = Fixed::from_f64(0.7); // high dependence on leader
        assert!(cult.should_dissolve(
            Fixed::from_f64(0.1),  // low leader competence
            Fixed::from_f64(0.5),  // moderate legitimacy
            Fixed::from_f64(0.3),  // moderate fatigue
        ));
    }

    #[test]
    fn cult_does_not_dissolve_when_strong() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.dependence = Fixed::from_f64(0.9);
        assert!(!cult.should_dissolve(
            Fixed::from_f64(0.8),  // strong leader
            Fixed::from_f64(0.3),  // low legitimacy
            Fixed::from_f64(0.2),  // low fatigue
        ));
    }

    #[test]
    fn cult_daily_update_decays() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.love_bombing = Fixed::from_f64(0.8);
        cult.fear_load = Fixed::from_f64(0.6);
        cult.daily_update();
        assert!(cult.love_bombing < Fixed::from_f64(0.8));
        assert!(cult.fear_load < Fixed::from_f64(0.6));
    }
}
