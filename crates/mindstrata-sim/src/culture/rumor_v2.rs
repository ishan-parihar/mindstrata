//! Rumor v2 — rumors as memes with uncertainty and social stakes (§13.3).
//!
//! Rumors differ from generic memes because they:
//! - Target specific agents or institutions
//! - Carry accusation severity and evidence quality
//! - Track source chains (who told whom)
//! - Can trigger moral panic and scapegoating
//! - Degrade accuracy with each hop (telephone game)
//!
//! ```text
//! Rumor effects:
//!   - reputation damage
//!   - panic
//!   - scapegoating
//!   - market distortion
//!   - faction formation
//!   - institutional crisis
//!
//! Evidence quality degrades:
//!   evidence_quality × fidelity^hops
//!
//! Prevalence grows then decays:
//!   prevalence = host_count / population × emotional_contagion
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A rumor — a meme with a target, evidence chain, and social stakes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RumorV2 {
    /// Unique rumor identifier.
    pub id: usize,
    /// Description of the rumor content.
    pub description: String,
    /// Index of the agent this rumor targets (if any).
    pub target: Option<usize>,
    /// Index of the institution this rumor targets (if any).
    pub institution_target: Option<usize>,
    /// Severity of the accusation (0 = trivial, 1 = life-destroying).
    pub accusation_severity: Fixed,
    /// Quality of evidence supporting the rumor (0 = pure fabrication, 1 = eyewitness).
    pub evidence_quality: Fixed,
    /// Chain of agents who have relayed this rumor (source chain).
    pub source_chain: Vec<usize>,
    /// Emotional charge accumulated through transmission.
    pub emotional_charge: Fixed,
    /// How prevalent this rumor is in the population (0 = unknown, 1 = everyone knows).
    pub prevalence: Fixed,
    /// Moral panic potential (how easily this triggers collective fear/anger).
    pub moral_panic_potential: Fixed,
    /// Tick when this rumor was created.
    pub created_tick: u64,
    /// Tick when this rumor was last transmitted.
    pub last_transmitted_tick: u64,
    /// Number of agents who currently believe this rumor.
    pub believer_count: u32,
    /// Whether this rumor is still active (not debunked or forgotten).
    pub active: bool,
    /// Whether this rumor has been officially debunked.
    pub debunked: bool,
}

impl RumorV2 {
    /// Create a new rumor from an accusation.
    pub fn new(
        id: usize,
        description: String,
        target: Option<usize>,
        accusation_severity: Fixed,
        evidence_quality: Fixed,
        emotional_charge: Fixed,
        tick: u64,
    ) -> Self {
        Self {
            id,
            description,
            target,
            institution_target: None,
            accusation_severity,
            evidence_quality,
            source_chain: Vec::new(),
            emotional_charge,
            prevalence: Fixed::ZERO,
            moral_panic_potential: accusation_severity * evidence_quality,
            created_tick: tick,
            last_transmitted_tick: tick,
            believer_count: 0,
            active: true,
            debunked: false,
        }
    }

    /// Compute transmission probability from source to listener.
    ///
    /// ```text
    /// chance = evidence_quality × (1 - hops_fidelity_loss)
    ///        × emotional_charge × listener_susceptibility
    ///        × (1 - skepticism) × target_proximity
    /// ```
    pub fn transmission_chance(
        &self,
        source_trust: Fixed,
        listener_susceptibility: Fixed,
        skepticism: Fixed,
        population: u32,
    ) -> Fixed {
        // Evidence degrades with hops
        let hops_fidelity = Fixed::from_f64(0.85)
            .powi(self.source_chain.len() as u32);
        let effective_evidence = self.evidence_quality * hops_fidelity;

        // Prevalence creates social proof (bandwagon effect)
        let social_proof = if population > 0 {
            self.prevalence * Fixed::from_f64(0.2)
        } else {
            Fixed::ZERO
        };

        let base = effective_evidence
            * source_trust
            * self.emotional_charge
            * listener_susceptibility
            * (Fixed::ONE + social_proof)
            * (Fixed::ONE - skepticism);

        // Target proximity: rumors about known agents spread faster
        let target_bonus = if self.target.is_some() {
            Fixed::from_f64(0.1)
        } else {
            Fixed::ZERO
        };

        (base + target_bonus).clamp_01()
    }

    /// Record a transmission hop (adds agent to source chain, degrades evidence).
    pub fn record_transmission(&mut self, agent_id: usize, tick: u64) {
        self.source_chain.push(agent_id);
        self.last_transmitted_tick = tick;
        // Emotional charge amplifies with each retelling
        self.emotional_charge =
            (self.emotional_charge + Fixed::from_f64(0.02)).clamp_01();
    }

    /// Compute moral panic pressure from this rumor.
    ///
    /// Panic rises with severity, evidence, and prevalence, then decays.
    pub fn panic_pressure(&self, population: u32) -> Fixed {
        if population == 0 {
            return Fixed::ZERO;
        }
        let prevalence_factor = self.prevalence;
        let severity_factor = self.accusation_severity * self.evidence_quality;
        let contagion = self.moral_panic_potential * prevalence_factor;
        (severity_factor * (Fixed::ONE + contagion)).clamp_01()
    }

    /// Tick update — prevalence decays over time if not reinforced.
    pub fn tick_decay(&mut self, tick: u64) {
        let ticks_since_transmission = tick.saturating_sub(self.last_transmitted_tick);
        // Decay prevalence slowly (rumors fade if not repeated)
        if ticks_since_transmission > 50 {
            let decay_factor =
                Fixed::from_f64(0.995).powi((ticks_since_transmission / 50) as u32);
            self.prevalence = (self.prevalence * decay_factor).clamp_01();
        }
        // Very old rumors become inactive
        if ticks_since_transmission > 1000 && self.prevalence < Fixed::from_f64(0.05) {
            self.active = false;
        }
    }

    /// Debunk this rumor (mark as false, reduce prevalence).
    pub fn debunk(&mut self) {
        self.debunked = true;
        self.prevalence = (self.prevalence * Fixed::from_f64(0.3)).clamp_01();
        self.active = false;
    }
}

/// Registry of all rumors in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RumorRegistry {
    /// All rumors.
    pub rumors: Vec<RumorV2>,
    /// Next available rumor id.
    next_id: usize,
}

impl Default for RumorRegistry {
    fn default() -> Self {
        Self {
            rumors: Vec::new(),
            next_id: 0,
        }
    }
}

impl RumorRegistry {
    /// Register a new rumor and return its id.
    pub fn register(&mut self, rumor: RumorV2) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut rumor = rumor;
        rumor.id = id;
        self.rumors.push(rumor);
        id
    }

    /// Get a rumor by id.
    pub fn get(&self, id: usize) -> Option<&RumorV2> {
        self.rumors.iter().find(|r| r.id == id)
    }

    /// Get a mutable reference to a rumor by id.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut RumorV2> {
        self.rumors.iter_mut().find(|r| r.id == id)
    }

    /// Get all active rumors targeting a specific agent.
    pub fn rumors_targeting(&self, agent_id: usize) -> Vec<&RumorV2> {
        self.rumors
            .iter()
            .filter(|r| r.active && r.target == Some(agent_id))
            .collect()
    }

    /// Get all active rumors about an institution.
    pub fn rumors_about_institution(&self, inst_id: usize) -> Vec<&RumorV2> {
        self.rumors
            .iter()
            .filter(|r| r.active && r.institution_target == Some(inst_id))
            .collect()
    }

    /// Tick decay on all active rumors.
    pub fn tick_all(&mut self, tick: u64) {
        for rumor in &mut self.rumors {
            if rumor.active {
                rumor.tick_decay(tick);
            }
        }
    }

    /// Number of active rumors.
    pub fn active_count(&self) -> usize {
        self.rumors.iter().filter(|r| r.active).count()
    }

    /// Sum the panic pressure from all active rumors.
    pub fn total_panic_pressure(&self, population: u32) -> Fixed {
        self.rumors
            .iter()
            .filter(|r| r.active)
            .map(|r| r.panic_pressure(population))
            .fold(Fixed::ZERO, |acc, p| (acc + p).clamp_01())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_rumor_has_sane_defaults() {
        let r = RumorV2::new(
            0,
            "the baker stole grain".into(),
            Some(5),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            Fixed::from_f64(0.7),
            0,
        );
        assert_eq!(r.id, 0);
        assert!(r.active);
        assert!(!r.debunked);
        assert_eq!(r.target, Some(5));
    }

    #[test]
    fn transmission_degrades_with_hops() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.5),
            0,
        );
        let chance_no_hops =
            r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::ZERO, 100);
        r.record_transmission(1, 1);
        r.record_transmission(2, 2);
        let chance_2_hops =
            r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::ZERO, 100);
        assert!(chance_no_hops > chance_2_hops);
    }

    #[test]
    fn skepticism_reduces_transmission() {
        let r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.6),
            0,
        );
        let no_skepticism =
            r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::ZERO, 100);
        let high_skepticism =
            r.transmission_chance(Fixed::ONE, Fixed::ONE, Fixed::from_f64(0.8), 100);
        assert!(no_skepticism > high_skepticism);
    }

    #[test]
    fn prevalence_decays_over_time() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        );
        r.prevalence = Fixed::from_f64(0.5);
        r.last_transmitted_tick = 0;
        r.tick_decay(200);
        assert!(r.prevalence < Fixed::from_f64(0.5));
    }

    #[test]
    fn debunk_reduces_prevalence() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        );
        r.prevalence = Fixed::from_f64(0.8);
        r.debunk();
        assert!(r.debunked);
        assert!(r.prevalence < Fixed::from_f64(0.3));
        assert!(!r.active);
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = RumorRegistry::default();
        let r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.5),
            0,
        );
        let id = reg.register(r);
        assert_eq!(id, 0);
        assert!(reg.get(0).is_some());
        assert!(reg.get(1).is_none());
    }

    #[test]
    fn rumors_targeting_filters_correctly() {
        let mut reg = RumorRegistry::default();
        reg.register(RumorV2::new(
            0, "about alice".into(), Some(0),
            Fixed::from_f64(0.5), Fixed::from_f64(0.5),
            Fixed::from_f64(0.5), 0,
        ));
        reg.register(RumorV2::new(
            0, "about bob".into(), Some(1),
            Fixed::from_f64(0.5), Fixed::from_f64(0.5),
            Fixed::from_f64(0.5), 0,
        ));
        let about_0 = reg.rumors_targeting(0);
        assert_eq!(about_0.len(), 1);
        assert_eq!(about_0[0].target, Some(0));
    }

    #[test]
    fn panic_pressure_scales_with_prevalence() {
        let mut r = RumorV2::new(
            0,
            "test".into(),
            None,
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.9),
            Fixed::from_f64(0.5),
            0,
        );
        r.prevalence = Fixed::from_f64(0.1);
        let low_panic = r.panic_pressure(100);
        r.prevalence = Fixed::from_f64(0.9);
        let high_panic = r.panic_pressure(100);
        assert!(high_panic > low_panic);
    }
}
