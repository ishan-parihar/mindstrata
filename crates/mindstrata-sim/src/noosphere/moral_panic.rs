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
//!
//! Iteration 115 wired this previously unit-test-only lifecycle into the
//! simulation: a §7.2 trigger now registers the panic in the registry, the
//! daily driver (`tick_moral_panic_lifecycle` in sim.rs) escalates or
//! resolves it from `escalation_pressure` + panic duration (`panic_fatigue`),
//! and an ACTIVE panic erodes its target institution's legitimacy each day
//! (`panic_legitimacy_drain`) — the self-reinforcing "panic → distrust →
//! more panic" loop until fatigue resolves the crisis.

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
        let legitimacy_vacuum = (Fixed::ONE - institutional_legitimacy) * Fixed::from_f64(0.25);
        let community_fear_pressure = community_fear * Fixed::from_f64(0.25);
        (fear_pressure + gossip_pressure + legitimacy_vacuum + community_fear_pressure).clamp_01()
    }

    /// Should the panic resolve?
    pub fn should_resolve(&self, institutional_response: Fixed, community_fatigue: Fixed) -> bool {
        // Panic resolves when institutions crack down, or when people are tired
        institutional_response > Fixed::from_f64(0.6)
            || community_fatigue > Fixed::from_f64(0.7)
            || self.intensity < Fixed::from_f64(0.1)
    }

    /// Escalate the panic.
    pub fn escalate(&mut self, tick: u64) {
        self.intensity = (self.intensity + Fixed::from_f64(0.05)).clamp_01();
        self.fear_level = (self.fear_level + Fixed::from_f64(0.03)).clamp_01();
        self.gossip_amplification = (self.gossip_amplification + Fixed::from_f64(0.02)).clamp_01();
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

/// Ticks per in-world day (scheduler: 144 ticks = 1 day). Used by the
/// daily panic lifecycle to derive panic duration in days.
pub const TICKS_PER_DAY: u64 = 144;

/// §13.5 (Iteration 115): escalation-pressure threshold above which an
/// ACTIVE panic intensifies each daily tick — matches
/// `MoralPanic::escalation_pressure`'s documented "> 0.5 suggests the
/// panic should intensify" guidance.
pub const PANIC_ESCALATION_THRESHOLD: f64 = 0.5;

/// §13.5 (Iteration 115): panic fatigue accrued per elapsed day — a panic
/// exhausts its participants. At 0.02/day fatigue crosses `should_resolve`'s
/// 0.7 threshold after 35 days, ending the panic.
pub const PANIC_FATIGUE_RATE: f64 = 0.02;

/// §13.5 (Iteration 115): legitimacy erosion per unit of panic intensity
/// per day. An ACTIVE panic keeps corroding the institution it erupted
/// about ("panic → distrust → more panic") until fatigue resolves it. At
/// intensity 0.3 a fresh panic erodes 0.03% legitimacy/day; escalated to
/// 1.0 it erodes 0.1%/day — a genuine but modest per-panic crisis effect
/// (~0.03 total legitimacy across a 54-day panic). NOTE the cumulative
/// regime: in panic-FREQUENT worlds (e.g. the fear-heavy pestilence
/// scenario, which fires the §7.2 trigger every cooldown — 20 panics by
/// 12K ticks) the repeated drains accumulate toward legitimacy collapse
/// and shift downstream conflict dynamics (the epidemic settles to a low
/// endemic plateau — documented in-test) — the self-reinforcing
/// "panic → distrust → more panic" spiral working as designed. Calibrated
/// at 0.001 (down from an initial 0.005): the probe showed 0.005 cascaded
/// through the knife-edge courtship→pregnancy→birth pipeline and killed
/// ALL long-horizon births (seed-1 80K: 0 births vs 2 pre-Iter-115),
/// while 0.001 preserves the 2-birth chain (re-paced to [28230, 35960]);
/// the golden (1,000) and snapshot (≤2,000) horizons never fire a panic,
/// so they are provably untouched either way.
pub const PANIC_LEGITIMACY_DRAIN_RATE: f64 = 0.001;

/// §13.5 (Iteration 115): community fatigue from panic duration — panics
/// exhaust their participants. `fatigue = clamp01(elapsed_days × rate)`;
/// at `PANIC_FATIGUE_RATE` (0.02/day) fatigue crosses the 0.7 resolve
/// threshold after 35 days. Pure and deterministic (no RNG) so replay
/// determinism holds; used by the daily lifecycle to resolve a panic once
/// participants tire of the crisis.
pub fn panic_fatigue(elapsed_days: u64, rate: f64) -> Fixed {
    (Fixed::from_int(elapsed_days as i64) * Fixed::from_f64(rate)).clamp_01()
}

/// §13.5 (Iteration 115): legitimacy erosion from an ACTIVE panic —
/// `intensity × rate`. The daily lifecycle subtracts this from the
/// institution the panic erupted about, the self-reinforcing
/// "panic → distrust → more panic" loop until fatigue or an institutional
/// response resolves the crisis. Pure and deterministic; the caller reads
/// the panic's post-update intensity, so the drain tracks escalation
/// exactly.
pub fn panic_legitimacy_drain(intensity: Fixed, rate: f64) -> Fixed {
    intensity * Fixed::from_f64(rate)
}

/// Registry of all moral panics in the simulation.
///
/// Iteration 115: the simulation drives the panic lifecycle directly
/// (`Simulation::tick_moral_panic_lifecycle` — resolve/escalate/decay plus
/// the legitimacy drain) and registers panics here at the §7.2 trigger.
/// The registry is a passive store; `active()` is a query accessor for
/// tests and future consumers (the sim iterates `panics` mutably itself).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MoralPanicRegistry {
    /// All moral panics (active and resolved).
    pub panics: Vec<MoralPanic>,
}

impl MoralPanicRegistry {
    pub fn new() -> Self {
        Self { panics: Vec::new() }
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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn panic_escalation_pressure() {
        let panic = MoralPanic::new(PanicTrigger::MoralViolation, Some(0), 0);
        let pressure = panic.escalation_pressure(
            Fixed::from_f64(0.3), // low legitimacy
            Fixed::from_f64(0.7), // high community fear
        );
        assert!(pressure > Fixed::from_f64(0.2));
    }

    #[test]
    fn panic_resolves_with_institutional_response() {
        let panic = MoralPanic::new(PanicTrigger::MoralViolation, Some(0), 0);
        assert!(panic.should_resolve(
            Fixed::from_f64(0.8), // strong institutional response
            Fixed::from_f64(0.3), // moderate fatigue
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

    #[test]
    fn panic_fatigue_accrues_with_elapsed_days() {
        // §13.5 (Iteration 115): a fresh panic has no fatigue; at
        // 0.02/day it crosses the 0.7 resolve threshold after 35 days and
        // saturates at 1.0 in the far tail.
        assert_eq!(panic_fatigue(0, PANIC_FATIGUE_RATE), Fixed::ZERO);
        assert_eq!(
            panic_fatigue(10, PANIC_FATIGUE_RATE),
            Fixed::from_f64(0.2),
            "10 days at 0.02/day must be exactly 0.2"
        );
        assert_eq!(
            panic_fatigue(35, PANIC_FATIGUE_RATE),
            Fixed::from_f64(0.7),
            "35 days must cross the resolve threshold exactly"
        );
        assert_eq!(
            panic_fatigue(100, PANIC_FATIGUE_RATE),
            Fixed::ONE,
            "fatigue must saturate at 1.0"
        );
    }

    #[test]
    fn panic_legitimacy_drain_scales_with_intensity() {
        // §13.5 (Iteration 115): a fresh panic (intensity 0.3) erodes
        // exactly 0.03% legitimacy per day; at full intensity 1.0 it
        // erodes 0.1%/day; zero intensity erodes nothing.
        assert_eq!(
            panic_legitimacy_drain(Fixed::from_f64(0.3), PANIC_LEGITIMACY_DRAIN_RATE),
            Fixed::from_f64(0.0003),
            "fresh panic intensity 0.3 × 0.001 must be exactly 0.0003"
        );
        assert_eq!(
            panic_legitimacy_drain(Fixed::ONE, PANIC_LEGITIMACY_DRAIN_RATE),
            Fixed::from_f64(0.001),
            "full intensity × 0.001 must be exactly 0.001"
        );
        assert_eq!(
            panic_legitimacy_drain(Fixed::ZERO, PANIC_LEGITIMACY_DRAIN_RATE),
            Fixed::ZERO
        );
    }

    #[test]
    fn panic_lifecycle_escalates_then_resolves_via_fatigue() {
        // §13.5 (Iteration 115): the resolve-first lifecycle — a fresh
        // panic under low legitimacy + high community fear escalates
        // (self-reinforcing), and once fatigue crosses 0.7 it resolves
        // even while escalation pressure is still high.
        let mut panic = MoralPanic::new(PanicTrigger::InstitutionalCorruption, None, 0);
        let pressure = panic.escalation_pressure(Fixed::from_f64(0.2), Fixed::from_f64(0.8));
        assert!(
            pressure > Fixed::from_f64(PANIC_ESCALATION_THRESHOLD),
            "low legitimacy + high fear must exceed the escalation threshold"
        );
        panic.escalate(TICKS_PER_DAY);
        assert!(
            panic.intensity > Fixed::from_f64(0.3),
            "escalation must raise intensity above the initial 0.3"
        );
        let fatigue = panic_fatigue(40, PANIC_FATIGUE_RATE);
        assert!(
            panic.should_resolve(Fixed::ZERO, fatigue),
            "fatigue 0.8 (> 0.7) must resolve the panic"
        );
        panic.deescalate();
        assert!(
            panic.intensity < Fixed::from_f64(0.4),
            "de-escalation must lower intensity"
        );
    }
}
