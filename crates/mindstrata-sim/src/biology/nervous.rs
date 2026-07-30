//! Neurological / Nervous system — arousal, autonomic regulation, pain, trauma.
//!
//! Central to making agents feel alive. The nervous system modulates:
//! - attention narrowing under stress
//! - threat sensitivity
//! - social engagement capacity
//! - pain processing
//! - trauma accumulation and effects

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Pain meaning — what pain signifies to the agent.
/// Pain meaning affects emotional response: battle pain → pride, punishment → resentment.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PainMeaning {
    /// Pain from labor/effort — expected, tolerable.
    Labor,
    /// Pain from battle — may produce pride or trauma.
    Battle,
    /// Pain from punishment — produces resentment.
    Punishment,
    /// Pain from childbirth — bonding or trauma.
    Childbirth,
    /// Pain from illness — produces fear.
    Illness,
    /// Unknown/unattributed pain.
    Unknown,
}

/// Pain state — more than just injury level.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PainState {
    /// Acute pain (0 = none, 1 = excruciating).
    pub acute: Fixed,
    /// Chronic pain (persists after acute resolves).
    pub chronic: Fixed,
    /// How much attention pain commands (0 = ignored, 1 = dominating).
    pub attention_demand: Fixed,
    /// What the agent believes caused the pain.
    pub meaning: PainMeaning,
}

impl Default for PainState {
    fn default() -> Self {
        Self {
            acute: Fixed::ZERO,
            chronic: Fixed::ZERO,
            attention_demand: Fixed::ZERO,
            meaning: PainMeaning::Unknown,
        }
    }
}

impl PainState {
    /// Update pain levels each tick.
    pub fn update(&mut self, injury_input: Fixed) {
        // Acute pain from injury
        self.acute = (self.acute * Fixed::from_f64(0.95) + injury_input * Fixed::from_f64(0.3)).clamp_01();
        // Chronic pain accumulates from repeated acute pain
        if self.acute > Fixed::from_f64(0.5) {
            self.chronic = (self.chronic + Fixed::from_f64(0.0005)).clamp_01();
        } else {
            self.chronic = (self.chronic - Fixed::from_f64(0.0002)).max(Fixed::ZERO);
        }
        // Attention demand follows pain level
        self.attention_demand = (self.acute * Fixed::from_f64(0.7) + self.chronic * Fixed::from_f64(0.3)).clamp_01();
    }

    /// Total effective pain for cognitive modulation.
    pub fn effective_pain(&self) -> Fixed {
        (self.acute + self.chronic * Fixed::from_f64(0.5)).clamp_01()
    }
}

/// Sensory acuity — how sensitive the agent's senses are.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensoryAcuity {
    /// Visual acuity.
    pub visual: Fixed,
    /// Auditory acuity.
    pub auditory: Fixed,
    /// Overall perceptual sensitivity.
    pub general: Fixed,
}

impl Default for SensoryAcuity {
    fn default() -> Self {
        Self {
            visual: Fixed::from_f64(0.5),
            auditory: Fixed::from_f64(0.5),
            general: Fixed::from_f64(0.5),
        }
    }
}

/// Nervous system state — autonomic regulation, arousal, pain, trauma.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NervousSystemState {
    /// Sympathetic activation (fight/flight). High = narrowed attention, threat-biased.
    pub sympathetic_arousal: Fixed,
    /// Parasympathetic tone (rest/digest). High = social engagement, trust, recovery.
    pub parasympathetic_tone: Fixed,
    /// Baseline arousal level.
    pub baseline_arousal: Fixed,
    /// Sensory acuity.
    pub sensory: SensoryAcuity,
    /// Current pain state.
    pub pain: PainState,
    /// Pain sensitivity — genetic + developmental.
    pub pain_sensitivity: Fixed,
    /// Accumulated trauma load.
    pub trauma_load: Fixed,
    /// Dissociation risk (extreme trauma response).
    pub dissociation_risk: Fixed,
    /// Startle sensitivity — elevated by trauma.
    pub startle_sensitivity: Fixed,
    /// Neuroplasticity — capacity for learning and change.
    pub neuroplasticity: Fixed,
    /// Sleep pressure — builds without sleep.
    pub sleep_pressure: Fixed,
}

impl Default for NervousSystemState {
    fn default() -> Self {
        Self {
            sympathetic_arousal: Fixed::from_f64(0.3),
            parasympathetic_tone: Fixed::from_f64(0.5),
            baseline_arousal: Fixed::from_f64(0.3),
            sensory: SensoryAcuity::default(),
            pain: PainState::default(),
            pain_sensitivity: Fixed::from_f64(0.5),
            trauma_load: Fixed::ZERO,
            dissociation_risk: Fixed::ZERO,
            startle_sensitivity: Fixed::from_f64(0.3),
            neuroplasticity: Fixed::from_f64(0.6),
            sleep_pressure: Fixed::ZERO,
        }
    }
}

impl NervousSystemState {
    /// Update nervous system each tick.
    pub fn update(
        &mut self,
        threat_input: Fixed,
        safety_input: Fixed,
        injury_input: Fixed,
        sleep_tick: bool,
    ) {
        // Sympathetic activation from threat
        let sympathetic_delta = threat_input * Fixed::from_f64(0.2);
        let sympathetic_recovery = self.parasympathetic_tone * Fixed::from_f64(0.1);
        self.sympathetic_arousal = (self.sympathetic_arousal + sympathetic_delta - sympathetic_recovery).clamp_01();

        // Parasympathetic recovery from safety
        let parasympathetic_buildup = safety_input * Fixed::from_f64(0.06);
        let parasympathetic_drain = self.sympathetic_arousal * Fixed::from_f64(0.05);
        self.parasympathetic_tone = (self.parasympathetic_tone + parasympathetic_buildup - parasympathetic_drain).clamp_01();

        // Pain update
        self.pain.update(injury_input);

        // Trauma load accumulation from sustained high sympathetic arousal
        if self.sympathetic_arousal > Fixed::from_f64(0.7) {
            self.trauma_load = (self.trauma_load + Fixed::from_f64(0.0003)).clamp_01();
        }
        // Trauma load decays very slowly
        self.trauma_load = (self.trauma_load - Fixed::from_f64(0.00005)).max(Fixed::ZERO);

        // Dissociation risk from extreme trauma
        self.dissociation_risk = (self.trauma_load * Fixed::from_f64(0.8)
            + self.pain.effective_pain() * Fixed::from_f64(0.2))
            .clamp_01();

        // Startle sensitivity elevated by trauma
        self.startle_sensitivity = (Fixed::from_f64(0.3) + self.trauma_load * Fixed::from_f64(0.5)).clamp_01();

        // Sleep pressure
        if sleep_tick {
            self.sleep_pressure = (self.sleep_pressure - Fixed::from_f64(0.1)).max(Fixed::ZERO);
        } else {
            self.sleep_pressure = (self.sleep_pressure + Fixed::from_f64(0.001)).clamp_01();
        }

        // Neuroplasticity reduced by chronic stress
        self.neuroplasticity = (Fixed::from_f64(0.6)
            - self.trauma_load * Fixed::from_f64(0.3)
            - self.pain.chronic * Fixed::from_f64(0.1))
            .clamp_01();
    }

    /// Effective arousal for cognitive modulation.
    /// High arousal narrows attention, increases threat bias.
    pub fn effective_arousal(&self) -> Fixed {
        self.sympathetic_arousal
    }

    /// Social engagement capacity — requires parasympathetic dominance.
    pub fn social_engagement_capacity(&self) -> Fixed {
        self.parasympathetic_tone
    }

    /// Attention narrowing factor from arousal.
    /// Returns 0–1 where 1 = fully narrowed (tunnel vision).
    pub fn attention_narrowing(&self) -> Fixed {
        (self.sympathetic_arousal * Fixed::from_f64(0.8)).clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pain_decays_without_input() {
        let mut pain = PainState::default();
        pain.acute = Fixed::from_f64(0.8);
        pain.update(Fixed::ZERO);
        assert!(pain.acute < Fixed::from_f64(0.8));
    }

    #[test]
    fn pain_increases_with_injury() {
        let mut pain = PainState::default();
        pain.update(Fixed::from_f64(0.9));
        assert!(pain.acute > Fixed::ZERO);
    }

    #[test]
    fn trauma_accumulates_from_sustained_arousal() {
        let mut ns = NervousSystemState::default();
        // Run 100 ticks with high threat
        for _ in 0..100 {
            ns.update(Fixed::from_f64(0.9), Fixed::ZERO, Fixed::ZERO, false);
        }
        assert!(ns.trauma_load > Fixed::ZERO);
    }

    #[test]
    fn parasympathetic_recoveres_in_safety() {
        let mut ns = NervousSystemState::default();
        ns.parasympathetic_tone = Fixed::from_f64(0.1);
        ns.sympathetic_arousal = Fixed::from_f64(0.8);
        // Run 50 ticks in safety
        for _ in 0..50 {
            ns.update(Fixed::ZERO, Fixed::from_f64(0.8), Fixed::ZERO, false);
        }
        assert!(ns.parasympathetic_tone > Fixed::from_f64(0.1));
    }

    #[test]
    fn social_engagement_requires_parasympathetic() {
        let mut ns = NervousSystemState::default();
        ns.parasympathetic_tone = Fixed::from_f64(0.8);
        assert!(ns.social_engagement_capacity() > Fixed::from_f64(0.5));
        ns.parasympathetic_tone = Fixed::from_f64(0.1);
        assert!(ns.social_engagement_capacity() < Fixed::from_f64(0.3));
    }
}
