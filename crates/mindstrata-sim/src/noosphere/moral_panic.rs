//! §13.5: Moral panic — collective fear-driven response to perceived threat.
//!
//! Architecture §13.5: Moral panics emerge when:
//! - Fear is high
//! - Institutional legitimacy is low
//! - A scapegoat is identified
//! - Gossip amplifies the threat
//! - Media/institutional messaging amplifies the threat
//! - The perceived threat violates sacred values
//!
//! Moral panics are self-reinforcing: fear → gossip → more fear → more gossip.
//! They resolve when:
//! - The scapegoat is punished
//! - Institutional authority reasserts control
//! - Fatigue reduces participation
//! - Counter-narratives emerge

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Trigger for a moral panic.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PanicTrigger {
    /// Perceived moral violation.
    MoralViolation,
    /// Perceived threat to children or family.
    ThreatToFamily,
    /// Perceived violation of sacred values.
    SacredViolation,
    /// Perceived outsider threat.
    OutsiderThreat,
    /// Perceived institutional corruption.
    InstitutionalCorruption,
    /// Perceived disease or contamination.
    Contamination,
}

/// A moral panic event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MoralPanic {
    /// What triggered the panic.
    pub trigger: PanicTrigger,
    /// Index of the scapegoat agent (if any).
    pub scapegoat: Option<usize>,
    /// Current intensity of the panic (0–1).
    pub intensity: Fixed,
    /// Fear level among affected agents (0–1).
    pub fear_level: Fixed,
    /// Gossip amplification factor (0–1).
    pub gossip_amplification: Fixed,
    /// Institutional response (0 = no response, 1 = strong crackdown).
    pub institutional_response: Fixed,
    /// Number of agents participating.
    pub participants: u32,
    /// Tick when the panic started.
    pub start_tick: u64,
    /// Tick of last escalation.
    pub last_escalation_tick: u64,
    /// Whether the panic is still active.
    pub active: bool,
}

impl MoralPanic {
    /// Create a new moral panic.
    pub fn new(trigger: PanicTrigger, scapegoat: Option<usize>, tick: u64) -> Self {
        Self {
            trigger,
            scapegoat,
            intensity: Fixed::from_f64(0.3),
            fear_level: Fixed::from_f64(0.5),
            gossip_amplification: Fixed::from_f64(0.4),
            institutional_response: Fixed::ZERO,
            participants: 0,
            start_tick: tick,
            last_escalation_tick: tick,
            active: true,
        }
    }

    /// Compute the panic escalation pressure.
    ///
    /// Returns a value > 0.5 suggesting the panic should intensify.
    pub fn escalation_pressure(
        &self,
        institutional_legitimacy: Fixed,
        community_fear: Fixed,
    ) -> Fixed {
        let fear_pressure = self.fear_level * Fixed::from_f64(0.3);
        let gossip_pressure = self.gossip_amplification * Fixed::from_f64(0.2);
        let legitimacy_vacuum =
            (Fixed::ONE - institutional_legitimacy) * Fixed::from_f64(0.25);
        let community_fear_pressure = community_fear * Fixed::from_f64(0.25);
        (fear_pressure + gossip_pressure + legitimacy_vacuum + community_fear_pressure).clamp_01()
    }

    /// Should the panic resolve?
    pub fn should_resolve(
        &self,
        institutional_response: Fixed,
        community_fatigue: Fixed,
    ) -> bool {
        // Panic resolves when institutions crack down, or when people are tired
        institutional_response > Fixed::from_f64(0.6)
            || community_fatigue > Fixed::from_f64(0.7)
            || self.intensity < Fixed::from_f64(0.1)
    }

    /// Escalate the panic.
    pub fn escalate(&mut self, tick: u64) {
        self.intensity = (self.intensity + Fixed::from_f64(0.05)).clamp_01();
        self.fear_level = (self.fear_level + Fixed::from_f64(0.03)).clamp_01();
        self.gossip_amplification =
            (self.gossip_amplification + Fixed::from_f64(0.02)).clamp_01();
        self.last_escalation_tick = tick;
    }

    /// De-escalate the panic.
    pub fn deescalate(&mut self) {
        self.intensity = (self.intensity - Fixed::from_f64(0.05)).max(Fixed::ZERO);
        self.fear_level = (self.fear_level - Fixed::from_f64(0.04)).max(Fixed::ZERO);
        self.gossip_amplification =
            (self.gossip_amplification - Fixed::from_f64(0.03)).max(Fixed::ZERO);
        if self.intensity < Fixed::from_f64(0.05) {
            self.active = false;
        }
    }

    /// Daily update — natural decay.
    pub fn daily_update(&mut self) {
        // Natural fatigue decay
        self.intensity = (self.intensity * Fixed::from_f64(0.95)).max(Fixed::ZERO);
        self.fear_level = (self.fear_level * Fixed::from_f64(0.96)).max(Fixed::ZERO);
        self.gossip_amplification =
            (self.gossip_amplification * Fixed::from_f64(0.94)).max(Fixed::ZERO);
        if self.intensity < Fixed::from_f64(0.05) {
            self.active = false;
        }
    }
}

/// Registry of all moral panics in the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MoralPanicRegistry {
    /// All moral panics (active and resolved).
    pub panics: Vec<MoralPanic>,
}

impl MoralPanicRegistry {
    pub fn new() -> Self {
        Self {
            panics: Vec::new(),
        }
    }

    /// Register a new moral panic and return its index.
    pub fn register(&mut self, panic: MoralPanic) -> usize {
        let idx = self.panics.len();
        self.panics.push(panic);
        idx
    }

    /// Get all active panics.
    pub fn active(&self) -> Vec<&MoralPanic> {
        self.panics.iter().filter(|p| p.active).collect()
    }

    /// Daily update for all active panics.
    pub fn daily_update(&mut self) {
        for panic in &mut self.panics {
            if panic.active {
                panic.daily_update();
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_escalation_pressure() {
        let panic = MoralPanic::new(PanicTrigger::MoralViolation, Some(0), 0);
        let pressure = panic.escalation_pressure(
            Fixed::from_f64(0.3),  // low legitimacy
            Fixed::from_f64(0.7),  // high community fear
        );
        assert!(pressure > Fixed::from_f64(0.2));
    }

    #[test]
    fn panic_resolves_with_institutional_response() {
        let panic = MoralPanic::new(PanicTrigger::MoralViolation, Some(0), 0);
        assert!(panic.should_resolve(
            Fixed::from_f64(0.8),  // strong institutional response
            Fixed::from_f64(0.3),  // moderate fatigue
        ));
    }

    #[test]
    fn panic_escalation_increases_intensity() {
        let mut panic = MoralPanic::new(PanicTrigger::MoralViolation, Some(0), 0);
        let initial = panic.intensity;
        panic.escalate(10);
        assert!(panic.intensity > initial);
    }

    #[test]
    fn panic_daily_update_decays() {
        let mut panic = MoralPanic::new(PanicTrigger::MoralViolation, Some(0), 0);
        panic.intensity = Fixed::from_f64(0.8);
        panic.daily_update();
        assert!(panic.intensity < Fixed::from_f64(0.8));
    }
}
