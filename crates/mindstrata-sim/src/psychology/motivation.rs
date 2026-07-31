//! Motivation and need system — layered architecture for agent drives.
//!
//! Expands the original 8 needs into a layered motivation architecture:
//!
//! ```text
//! Biological Needs:   hunger, thirst, sleep, warmth, health, safety
//! Psychological Needs: attachment, belonging, esteem, autonomy, competence,
//!                      meaning, certainty, novelty, play, justice, recognition
//! ```
//!
//! Motivation dynamics:
//! ```text
//! motivation_pressure =
//!     need_deficit
//!   × personality_weight
//!   × emotional_amplification
//!   × cultural_legitimacy
//!   × situational_affordance
//! ```
//!
//! Each need has:
//! - current deficit (0–1, higher = more pressing)
//! - baseline deficit (how fast the need naturally grows)
//! - urgency weight (how strongly it competes for attention)
//! - satisfaction from relevant actions
//!
//! Emergent effects:
//! - Starving agents prioritize food over social needs
//! - Isolated agents seek belonging even at cost of safety
//! - High-esteems agents avoid humiliation more than hunger
//! - Meaning-seeking agents take risks for ideological goals

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A single motivation need with deficit and urgency.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct MotiveNeed {
    /// Current deficit level (0 = satisfied, 1 = critical).
    pub deficit: Fixed,
    /// How fast this need grows each tick (baseline pressure).
    pub growth_rate: Fixed,
    /// How much this need competes for attention (personality-modulated).
    pub urgency_weight: Fixed,
    /// Maximum deficit cap.
    pub cap: Fixed,
}

impl MotiveNeed {
    /// Create a new motive need.
    pub fn new(growth_rate: Fixed, urgency_weight: Fixed) -> Self {
        Self {
            deficit: Fixed::ZERO,
            growth_rate,
            urgency_weight,
            cap: Fixed::ONE,
        }
    }

    /// Tick the need — increase deficit by growth rate.
    pub fn tick(&mut self) {
        self.deficit = (self.deficit + self.growth_rate).min(self.cap);
    }

    /// Apply relief from an action (e.g., eating reduces hunger deficit).
    pub fn relieve(&mut self, amount: Fixed) {
        self.deficit = (self.deficit - amount).max(Fixed::ZERO);
    }

    /// Pressure = deficit × urgency_weight.
    pub fn pressure(&self) -> Fixed {
        self.deficit * self.urgency_weight
    }
}

/// Layered motivation state for an agent.
///
/// Biological needs compete with psychological needs. Under extreme deprivation,
/// biological needs dominate. When biological needs are met, psychological
/// needs emerge and drive behavior.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MotivationState {
    // ── Biological needs ──
    /// Hunger deficit.
    pub hunger: MotiveNeed,
    /// Thirst deficit.
    pub thirst: MotiveNeed,
    /// Sleep deficit.
    pub sleep: MotiveNeed,
    /// Warmth deficit.
    pub warmth: MotiveNeed,
    /// Health concern.
    pub health: MotiveNeed,
    /// Safety concern.
    pub safety: MotiveNeed,
    // ── Psychological needs ──
    /// Need for attachment and close bonds.
    pub attachment: MotiveNeed,
    /// Need for social belonging.
    pub belonging: MotiveNeed,
    /// Need for esteem and respect.
    pub esteem: MotiveNeed,
    /// Need for autonomy and agency.
    pub autonomy: MotiveNeed,
    /// Need for competence and mastery.
    pub competence: MotiveNeed,
    /// Need for meaning and purpose.
    pub meaning: MotiveNeed,
    /// Need for certainty and predictability.
    pub certainty: MotiveNeed,
    /// Need for novelty and stimulation.
    pub novelty: MotiveNeed,
    /// Need for play and recreation.
    pub play: MotiveNeed,
    /// Need for justice and fairness.
    pub justice: MotiveNeed,
    /// Need for recognition and status.
    pub recognition: MotiveNeed,

    /// Dominant need index (updated each tick).
    pub dominant_need: MotiveCategory,
}

/// Categories of motivation needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MotiveCategory {
    Hunger,
    Thirst,
    Sleep,
    Warmth,
    Health,
    Safety,
    Attachment,
    Belonging,
    Esteem,
    Autonomy,
    Competence,
    Meaning,
    Certainty,
    Novelty,
    Play,
    Justice,
    Recognition,
}

impl Default for MotivationState {
    fn default() -> Self {
        Self {
            hunger: MotiveNeed::new(Fixed::from_f64(0.008), Fixed::from_f64(1.0)),
            thirst: MotiveNeed::new(Fixed::from_f64(0.010), Fixed::from_f64(1.1)),
            sleep: MotiveNeed::new(Fixed::from_f64(0.006), Fixed::from_f64(0.9)),
            warmth: MotiveNeed::new(Fixed::from_f64(0.003), Fixed::from_f64(0.6)),
            health: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.5)),
            safety: MotiveNeed::new(Fixed::from_f64(0.002), Fixed::from_f64(0.8)),
            attachment: MotiveNeed::new(Fixed::from_f64(0.002), Fixed::from_f64(0.6)),
            belonging: MotiveNeed::new(Fixed::from_f64(0.002), Fixed::from_f64(0.5)),
            esteem: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.4)),
            autonomy: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            competence: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            meaning: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            certainty: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            novelty: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.2)),
            play: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.2)),
            justice: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            recognition: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            dominant_need: MotiveCategory::Hunger,
        }
    }
}

impl MotivationState {
    /// Create from personality, modulating urgency weights.
    pub fn from_personality(personality: &crate::person::Personality) -> Self {
        let mut m = Self::default();
        // High extraversion → stronger belonging/attachment needs
        m.belonging.urgency_weight = (m.belonging.urgency_weight
            + personality.extraversion * Fixed::from_f64(0.3)).clamp_01();
        m.attachment.urgency_weight = (m.attachment.urgency_weight
            + personality.extraversion * Fixed::from_f64(0.2)).clamp_01();
        // High ambition → stronger esteem/recognition needs
        m.esteem.urgency_weight = (m.esteem.urgency_weight
            + personality.ambition * Fixed::from_f64(0.3)).clamp_01();
        m.recognition.urgency_weight = (m.recognition.urgency_weight
            + personality.ambition * Fixed::from_f64(0.2)).clamp_01();
        // High openness → stronger novelty/meaning needs
        m.novelty.urgency_weight = (m.novelty.urgency_weight
            + personality.openness * Fixed::from_f64(0.3)).clamp_01();
        m.meaning.urgency_weight = (m.meaning.urgency_weight
            + personality.openness * Fixed::from_f64(0.2)).clamp_01();
        // High neuroticism → stronger safety/certainty needs
        m.safety.urgency_weight = (m.safety.urgency_weight
            + personality.neuroticism * Fixed::from_f64(0.3)).clamp_01();
        m.certainty.urgency_weight = (m.certainty.urgency_weight
            + personality.neuroticism * Fixed::from_f64(0.2)).clamp_01();
        // High agreeableness → stronger justice/attachment needs
        m.justice.urgency_weight = (m.justice.urgency_weight
            + personality.agreeableness * Fixed::from_f64(0.2)).clamp_01();
        m
    }

    /// Tick all needs forward by one tick.
    pub fn tick_all(&mut self) {
        self.hunger.tick();
        self.thirst.tick();
        self.sleep.tick();
        self.warmth.tick();
        self.health.tick();
        self.safety.tick();
        self.attachment.tick();
        self.belonging.tick();
        self.esteem.tick();
        self.autonomy.tick();
        self.competence.tick();
        self.meaning.tick();
        self.certainty.tick();
        self.novelty.tick();
        self.play.tick();
        self.justice.tick();
        self.recognition.tick();
    }

    /// Update the dominant need based on current pressures.
    ///
    /// Biological needs dominate when they are critical (deficit > 0.5).
    /// Otherwise, the highest-pressure need wins.
    pub fn update_dominant(&mut self) {
        // Check biological urgency first
        // Biological needs dominate when their pressure exceeds a critical
        // threshold. Using pressure (deficit × urgency) rather than raw
        // deficit ensures high-urgency needs (e.g., thirst) dominate before
        // low-urgency ones (e.g., warmth) even at similar deficit levels.
        let bio_critical = self.hunger.pressure() > Fixed::from_f64(0.4)
            || self.thirst.pressure() > Fixed::from_f64(0.4)
            || self.sleep.pressure() > Fixed::from_f64(0.5)
            || self.safety.pressure() > Fixed::from_f64(0.4);

        if bio_critical {
            // Pick the most critical biological need
            let max_bio = [
                (MotiveCategory::Hunger, self.hunger.pressure()),
                (MotiveCategory::Thirst, self.thirst.pressure()),
                (MotiveCategory::Sleep, self.sleep.pressure()),
                (MotiveCategory::Safety, self.safety.pressure()),
            ];
            self.dominant_need = max_bio.iter()
                .max_by_key(|(_, p)| p.to_raw())
                .map(|(c, _)| *c)
                .unwrap_or(MotiveCategory::Hunger);
        } else {
            // All needs compete equally
            let all = [
                (MotiveCategory::Hunger, self.hunger.pressure()),
                (MotiveCategory::Thirst, self.thirst.pressure()),
                (MotiveCategory::Sleep, self.sleep.pressure()),
                (MotiveCategory::Warmth, self.warmth.pressure()),
                (MotiveCategory::Health, self.health.pressure()),
                (MotiveCategory::Safety, self.safety.pressure()),
                (MotiveCategory::Attachment, self.attachment.pressure()),
                (MotiveCategory::Belonging, self.belonging.pressure()),
                (MotiveCategory::Esteem, self.esteem.pressure()),
                (MotiveCategory::Autonomy, self.autonomy.pressure()),
                (MotiveCategory::Competence, self.competence.pressure()),
                (MotiveCategory::Meaning, self.meaning.pressure()),
                (MotiveCategory::Certainty, self.certainty.pressure()),
                (MotiveCategory::Novelty, self.novelty.pressure()),
                (MotiveCategory::Play, self.play.pressure()),
                (MotiveCategory::Justice, self.justice.pressure()),
                (MotiveCategory::Recognition, self.recognition.pressure()),
            ];
            self.dominant_need = all.iter()
                .max_by_key(|(_, p)| p.to_raw())
                .map(|(c, _)| *c)
                .unwrap_or(MotiveCategory::Hunger);
        }
    }

    /// Get the current dominant need's pressure level.
    pub fn dominant_pressure(&self) -> Fixed {
        match self.dominant_need {
            MotiveCategory::Hunger => self.hunger.pressure(),
            MotiveCategory::Thirst => self.thirst.pressure(),
            MotiveCategory::Sleep => self.sleep.pressure(),
            MotiveCategory::Warmth => self.warmth.pressure(),
            MotiveCategory::Health => self.health.pressure(),
            MotiveCategory::Safety => self.safety.pressure(),
            MotiveCategory::Attachment => self.attachment.pressure(),
            MotiveCategory::Belonging => self.belonging.pressure(),
            MotiveCategory::Esteem => self.esteem.pressure(),
            MotiveCategory::Autonomy => self.autonomy.pressure(),
            MotiveCategory::Competence => self.competence.pressure(),
            MotiveCategory::Meaning => self.meaning.pressure(),
            MotiveCategory::Certainty => self.certainty.pressure(),
            MotiveCategory::Novelty => self.novelty.pressure(),
            MotiveCategory::Play => self.play.pressure(),
            MotiveCategory::Justice => self.justice.pressure(),
            MotiveCategory::Recognition => self.recognition.pressure(),
        }
    }

    /// Full tick: grow all needs, update dominant.
    pub fn update(&mut self) {
        self.tick_all();
        self.update_dominant();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn motive_need_tick_increases_deficit() {
        let mut n = MotiveNeed::new(Fixed::from_f64(0.1), Fixed::ONE);
        n.tick();
        assert!(n.deficit > Fixed::ZERO);
    }

    #[test]
    fn motive_need_relieve_decreases_deficit() {
        let mut n = MotiveNeed::new(Fixed::from_f64(0.1), Fixed::ONE);
        n.tick();
        n.tick();
        let before = n.deficit;
        n.relieve(Fixed::from_f64(0.05));
        assert!(n.deficit < before);
    }

    #[test]
    fn motive_need_pressure_scales_with_urgency() {
        let mut n1 = MotiveNeed::new(Fixed::from_f64(0.1), Fixed::ONE);
        let mut n2 = MotiveNeed::new(Fixed::from_f64(0.1), Fixed::from_f64(0.5));
        n1.tick();
        n2.tick();
        assert!(n1.pressure() > n2.pressure());
    }

    #[test]
    fn hunger_dominates_when_starving() {
        let mut m = MotivationState::default();
        m.hunger.deficit = Fixed::from_f64(0.8);
        m.update_dominant();
        assert_eq!(m.dominant_need, MotiveCategory::Hunger);
    }

    #[test]
    fn default_dominant_is_hunger() {
        let m = MotivationState::default();
        assert_eq!(m.dominant_need, MotiveCategory::Hunger);
    }

    #[test]
    fn from_personality_extraversion_boosts_belonging() {
        use rand::SeedableRng;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        let mut p = crate::person::Personality::random(&mut rng);
        p.extraversion = Fixed::from_f64(0.9);
        let m = MotivationState::from_personality(&p);
        // Belonging urgency should be higher than default (0.5 + 0.9*0.3 = 0.77)
        assert!(m.belonging.urgency_weight > Fixed::from_f64(0.5));
    }

    #[test]
    fn update_grows_all_needs() {
        let mut m = MotivationState::default();
        m.update();
        assert!(m.hunger.deficit > Fixed::ZERO);
        assert!(m.thirst.deficit > Fixed::ZERO);
        assert!(m.sleep.deficit > Fixed::ZERO);
    }

    #[test]
    fn deficit_caps_at_one() {
        let mut n = MotiveNeed::new(Fixed::ONE, Fixed::ONE);
        for _ in 0..10 {
            n.tick();
        }
        assert!(n.deficit <= Fixed::ONE);
    }
}
