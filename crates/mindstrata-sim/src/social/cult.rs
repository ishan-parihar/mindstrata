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

/// Grace period before prophecy disconfirmation can dissolve a cult (§12.4).
/// Fresh cults have not yet made falsifiable prophecies — a disconfirmed
/// prophecy only matters once the narrative has had time to be tested.
/// One week at 144 ticks/day.
pub const PROPHECY_GRACE_TICKS: u64 = 144 * 7;

/// Signals that can drive cult dissolution (§12.4).
///
/// The plan lists seven dissolution conditions: leader failure, prophecy
/// disconfirmation, internal betrayal, external repression, member
/// exhaustion, rival narrative, and economic collapse. This struct carries
/// the per-cult signal values; `CultDynamics::should_dissolve` decides.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CultDissolutionSignals {
    /// Leader competence (0–1). Below threshold with high member dependence =
    /// leader failure.
    pub leader_competence: Fixed,
    /// Institutional legitimacy (0–1). High legitimacy + low isolation =
    /// external repression.
    pub institutional_legitimacy: Fixed,
    /// Average member fatigue (0–1). High = member exhaustion.
    pub member_average_fatigue: Fixed,
    /// Age of the cult in ticks (tick − formed_tick) — gates the prophecy
    /// disconfirmation branch (see `PROPHECY_GRACE_TICKS`).
    pub cult_age_ticks: u64,
    /// Prophecy disconfirmation — the sacred narrative's credibility collapsed.
    pub prophecy_disconfirmed: bool,
    /// Internal betrayal — most members no longer depend on the cult.
    pub internal_betrayal: bool,
    /// Rival narrative pressure (0–1) — a competing narrative out-charms the
    /// sacred one.
    pub rival_narrative_pressure: Fixed,
    /// Economic stress (0–1) — resource scarcity undermining the cult.
    pub economic_stress: Fixed,
}

impl CultDissolutionSignals {
    /// Neutral signals — no dissolution driver active. Useful for tests and
    /// as the base for asserting on single conditions.
    pub fn benign() -> Self {
        Self {
            leader_competence: Fixed::from_f64(0.8),
            institutional_legitimacy: Fixed::from_f64(0.3),
            member_average_fatigue: Fixed::from_f64(0.2),
            cult_age_ticks: 10_000,
            prophecy_disconfirmed: false,
            internal_betrayal: false,
            rival_narrative_pressure: Fixed::ZERO,
            economic_stress: Fixed::ZERO,
        }
    }
}

/// Cult formation and dissolution dynamics.
///
/// Architecture §12.4: Cults are a special high-intensity group type
/// that emerges under specific social-psychological conditions.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CultDynamics {
    /// Index of the charismatic leader.
    pub charismatic_leader: usize,
    /// Agent indices of the cult's members (excluding the leader).
    pub members: Vec<usize>,
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
            members: Vec::new(),
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

    /// Add a member to the cult (deduplicated).
    pub fn add_member(&mut self, agent: usize) {
        if !self.members.contains(&agent) {
            self.members.push(agent);
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
    /// Architecture §12.4: all seven dissolution conditions — leader failure,
    /// prophecy disconfirmation, internal betrayal, external repression,
    /// member exhaustion, rival narrative, and economic collapse.
    pub fn should_dissolve(&self, signals: &CultDissolutionSignals) -> bool {
        // Leader failure — competence collapses while members depend on the leader.
        if signals.leader_competence < Fixed::from_f64(0.2)
            && self.dependence > Fixed::from_f64(0.5)
        {
            return true;
        }
        // Prophecy disconfirmation — the sacred narrative was falsified. A
        // grace period protects fresh cults (no prophecies made yet), and high
        // identity fusion resists disconfirmation (Festinger's "when prophecy
        // fails" — fused members double down instead of leaving).
        if signals.prophecy_disconfirmed
            && signals.cult_age_ticks >= PROPHECY_GRACE_TICKS
            && self.identity_fusion < Fixed::from_f64(0.6)
        {
            return true;
        }
        // Internal betrayal — most members have stopped depending on the cult.
        if signals.internal_betrayal && self.dependence < Fixed::from_f64(0.5) {
            return true;
        }
        // Rival narrative — a competing story out-charms the sacred one faster
        // than the cult's thought-terminating clichés can shut it down.
        if signals.rival_narrative_pressure > self.thought_terminating_cliches {
            return true;
        }
        // External repression — institutional legitimacy overwhelms a cult
        // that has not isolated itself.
        if signals.institutional_legitimacy > Fixed::from_f64(0.8)
            && self.isolation < Fixed::from_f64(0.3)
        {
            return true;
        }
        // Member exhaustion — fatigue undermines cohesion.
        if signals.member_average_fatigue > Fixed::from_f64(0.8) {
            return true;
        }
        // Economic collapse — scarcity makes membership unsustainable.
        if signals.economic_stress > Fixed::from_f64(0.85) {
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
        let mut s = CultDissolutionSignals::benign();
        s.leader_competence = Fixed::from_f64(0.1); // low leader competence
        assert!(cult.should_dissolve(&s));
    }

    #[test]
    fn cult_does_not_dissolve_when_strong() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.dependence = Fixed::from_f64(0.9);
        assert!(!cult.should_dissolve(&CultDissolutionSignals::benign()));
    }

    #[test]
    fn cult_dissolves_on_prophecy_disconfirmation() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.identity_fusion = Fixed::from_f64(0.3); // unfused members defect
        let mut s = CultDissolutionSignals::benign();
        s.prophecy_disconfirmed = true;
        assert!(cult.should_dissolve(&s)); // age >= grace
    }

    #[test]
    fn prophecy_disconfirmation_resisted_by_identity_fusion() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.identity_fusion = Fixed::from_f64(0.9); // fused members double down
        let mut s = CultDissolutionSignals::benign();
        s.prophecy_disconfirmed = true;
        assert!(!cult.should_dissolve(&s));
    }

    #[test]
    fn fresh_cults_get_prophecy_grace_period() {
        let cult = CultDynamics::new(0, 1, 0);
        let mut s = CultDissolutionSignals::benign();
        s.prophecy_disconfirmed = true;
        s.cult_age_ticks = 100; // < grace
        assert!(!cult.should_dissolve(&s));
    }

    #[test]
    fn cult_dissolves_on_internal_betrayal() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.dependence = Fixed::from_f64(0.3); // members no longer dependent
        let mut s = CultDissolutionSignals::benign();
        s.internal_betrayal = true;
        assert!(cult.should_dissolve(&s));
    }

    #[test]
    fn cult_dissolves_on_rival_narrative() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.thought_terminating_cliches = Fixed::from_f64(0.2);
        let mut s = CultDissolutionSignals::benign();
        s.rival_narrative_pressure = Fixed::from_f64(0.5); // rival out-charms sacred
        assert!(cult.should_dissolve(&s));
    }

    #[test]
    fn cult_dissolves_on_economic_collapse() {
        let cult = CultDynamics::new(0, 1, 0);
        let mut s = CultDissolutionSignals::benign();
        s.economic_stress = Fixed::from_f64(0.9); // scarcity
        assert!(cult.should_dissolve(&s));
    }

    #[test]
    fn high_dependence_resists_internal_betrayal() {
        let mut cult = CultDynamics::new(0, 1, 0);
        cult.dependence = Fixed::from_f64(0.9); // still dependent on leader
        let mut s = CultDissolutionSignals::benign();
        s.internal_betrayal = true;
        assert!(!cult.should_dissolve(&s));
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
