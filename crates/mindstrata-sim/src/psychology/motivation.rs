//! Motivation and need system — layered architecture for agent drives.
//!
//! Expands the original 8 needs into a layered motivation architecture
//! (Architecture-plan-2 §8.1.5):
//!
//! ```text
//! Biological Needs:   hunger, thirst, sleep, warmth, health, safety
//! Psychological Needs: attachment, belonging, esteem, autonomy, competence,
//!                      meaning, certainty, novelty, play, care, romance,
//!                      justice, recognition
//! ```
//!
//! Motivation dynamics (the plan's exact formula):
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
//!
//! Iteration 44 upgrade: the three missing formula factors
//! (`emotional_amplification`, `cultural_legitimacy`, `situational_affordance`)
//! are now computed every tick from existing agent/world state (affect,
//! the legitimacy field, and world food/water stocks) and the roster is
//! completed with `care` + `romance`. Note on the plan's `personality_weight`
//! factor: it is represented by each need's per-need `urgency_weight`
//! (pre-existing design — competition strength is a property of the need,
//! not the agent), so the base term is `deficit × urgency_weight`.
//! `dominant_need`/`dominant_pressure` feed `compute_utility()` (the
//! `dominant_pressure` × `urgency_weight` bonus biases action selection
//! toward the need-matched action — §8.1.5). The richer formula is
//! drift-free by construction.

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

impl Default for MotiveNeed {
    fn default() -> Self {
        Self {
            deficit: Fixed::ZERO,
            growth_rate: Fixed::ZERO,
            urgency_weight: Fixed::ZERO,
            cap: Fixed::ONE,
        }
    }
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
    /// Need to care for and nurture others.
    pub care: MotiveNeed,
    /// Need for intimacy and pair-bonding.
    pub romance: MotiveNeed,
    /// Need for justice and fairness.
    pub justice: MotiveNeed,
    /// Need for recognition and status.
    pub recognition: MotiveNeed,

    /// Dominant need index (updated each tick).
    pub dominant_need: MotiveCategory,

    // ── §8.1.5 full-formula context (feeds compute_utility) ──
    // Per-category modulation is what makes the formula genuinely live: a
    // fearful agent's safety pressure out-competes, an angry agent's justice
    // need rises, scarcity amplifies hunger/thirst — not a uniform multiplier
    // that would leave the argmax untouched.
    /// Fear context (0–1).
    #[serde(default)]
    pub fear: Fixed,
    /// Anger context (0–1).
    #[serde(default)]
    pub anger: Fixed,
    /// Joy context (0–1).
    #[serde(default)]
    pub joy: Fixed,
    /// Sadness context (0–1).
    #[serde(default)]
    pub sadness: Fixed,
    /// Institutional legitimacy context (mean-zero anchor 0.5).
    #[serde(default)]
    pub cultural_legitimacy: Fixed,
    /// Food scarcity context (0–1).
    #[serde(default)]
    pub food_scarcity: Fixed,
    /// Water scarcity context (0–1).
    #[serde(default)]
    pub water_scarcity: Fixed,
}

/// Categories of motivation needs.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
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
    Care,
    Romance,
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
            care: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.25)),
            romance: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.25)),
            justice: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            recognition: MotiveNeed::new(Fixed::from_f64(0.001), Fixed::from_f64(0.3)),
            dominant_need: MotiveCategory::Hunger,
            fear: Fixed::ZERO,
            anger: Fixed::ZERO,
            joy: Fixed::ZERO,
            sadness: Fixed::ZERO,
            cultural_legitimacy: Fixed::from_f64(0.5),
            food_scarcity: Fixed::ZERO,
            water_scarcity: Fixed::ZERO,
        }
    }
}

impl MotivationState {
    /// Create from personality, modulating urgency weights.
    pub fn from_personality(personality: &crate::person::Personality) -> Self {
        let mut m = Self::default();
        // High extraversion → stronger belonging/attachment/romance needs
        m.belonging.urgency_weight = (m.belonging.urgency_weight
            + personality.extraversion * Fixed::from_f64(0.3))
        .clamp_01();
        m.attachment.urgency_weight = (m.attachment.urgency_weight
            + personality.extraversion * Fixed::from_f64(0.2))
        .clamp_01();
        m.romance.urgency_weight = (m.romance.urgency_weight
            + personality.extraversion * Fixed::from_f64(0.15))
        .clamp_01();
        // High ambition → stronger esteem/recognition needs
        m.esteem.urgency_weight =
            (m.esteem.urgency_weight + personality.ambition * Fixed::from_f64(0.3)).clamp_01();
        m.recognition.urgency_weight =
            (m.recognition.urgency_weight + personality.ambition * Fixed::from_f64(0.2)).clamp_01();
        // High openness → stronger novelty/meaning needs
        m.novelty.urgency_weight =
            (m.novelty.urgency_weight + personality.openness * Fixed::from_f64(0.3)).clamp_01();
        m.meaning.urgency_weight =
            (m.meaning.urgency_weight + personality.openness * Fixed::from_f64(0.2)).clamp_01();
        // High neuroticism → stronger safety/certainty needs
        m.safety.urgency_weight =
            (m.safety.urgency_weight + personality.neuroticism * Fixed::from_f64(0.3)).clamp_01();
        m.certainty.urgency_weight = (m.certainty.urgency_weight
            + personality.neuroticism * Fixed::from_f64(0.2))
        .clamp_01();
        // High agreeableness → stronger justice/care needs
        m.justice.urgency_weight = (m.justice.urgency_weight
            + personality.agreeableness * Fixed::from_f64(0.2))
        .clamp_01();
        m.care.urgency_weight =
            (m.care.urgency_weight + personality.agreeableness * Fixed::from_f64(0.2)).clamp_01();
        m
    }

    /// Read-only access to a need by category.
    pub fn need(&self, category: MotiveCategory) -> &MotiveNeed {
        match category {
            MotiveCategory::Hunger => &self.hunger,
            MotiveCategory::Thirst => &self.thirst,
            MotiveCategory::Sleep => &self.sleep,
            MotiveCategory::Warmth => &self.warmth,
            MotiveCategory::Health => &self.health,
            MotiveCategory::Safety => &self.safety,
            MotiveCategory::Attachment => &self.attachment,
            MotiveCategory::Belonging => &self.belonging,
            MotiveCategory::Esteem => &self.esteem,
            MotiveCategory::Autonomy => &self.autonomy,
            MotiveCategory::Competence => &self.competence,
            MotiveCategory::Meaning => &self.meaning,
            MotiveCategory::Certainty => &self.certainty,
            MotiveCategory::Novelty => &self.novelty,
            MotiveCategory::Play => &self.play,
            MotiveCategory::Care => &self.care,
            MotiveCategory::Romance => &self.romance,
            MotiveCategory::Justice => &self.justice,
            MotiveCategory::Recognition => &self.recognition,
        }
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
        self.care.tick();
        self.romance.tick();
        self.justice.tick();
        self.recognition.tick();
    }

    /// §8.1.5: set the full-formula context for this tick. The context
    /// feeds `compute_utility()` — the sim derives the values from affect,
    /// the legitimacy field, and world food/water stocks.
    pub fn set_context(
        &mut self,
        fear: Fixed,
        anger: Fixed,
        joy: Fixed,
        sadness: Fixed,
        cultural_legitimacy: Fixed,
        food_scarcity: Fixed,
        water_scarcity: Fixed,
    ) {
        self.fear = fear.clamp_01();
        self.anger = anger.clamp_01();
        self.joy = joy.clamp_01();
        self.sadness = sadness.clamp_01();
        self.cultural_legitimacy = cultural_legitimacy.clamp_01();
        self.food_scarcity = food_scarcity.clamp_01();
        self.water_scarcity = water_scarcity.clamp_01();
    }

    /// §8.1.5: the plan's full pressure formula for one need:
    /// `need_deficit × personality_weight × emotional_amplification ×
    /// cultural_legitimacy × situational_affordance`.
    ///
    /// - `personality_weight` = the need's `urgency_weight` (personality-
    ///   modulated at init).
    /// - `emotional_amplification` is per-category: fear amplifies
    ///   safety/certainty, anger amplifies justice, joy amplifies
    ///   play/belonging/romance, sadness dampens meaning.
    /// - `cultural_legitimacy` is per-category: low legitimacy amplifies
    ///   justice/certainty/safety (an illegitimate order is more threatening).
    /// - `situational_affordance` is per-category: scarcity amplifies
    ///   hunger/thirst (you cannot act, so the drive grows more pressing).
    pub fn pressure_full(&self, category: MotiveCategory) -> Fixed {
        let base = self.need(category).pressure();

        // Emotional amplification (1.0 neutral).
        let emotional = match category {
            MotiveCategory::Safety | MotiveCategory::Certainty => {
                Fixed::ONE + self.fear * Fixed::from_f64(0.4)
            }
            MotiveCategory::Justice => Fixed::ONE + self.anger * Fixed::from_f64(0.4),
            MotiveCategory::Play | MotiveCategory::Belonging | MotiveCategory::Romance => {
                Fixed::ONE + self.joy * Fixed::from_f64(0.3)
            }
            MotiveCategory::Meaning => {
                (Fixed::ONE - self.sadness * Fixed::from_f64(0.3)).max(Fixed::from_f64(0.5))
            }
            _ => Fixed::ONE,
        };

        // Cultural legitimacy (1.0 at the 0.5 mean-zero anchor). Live runs
        // carry ~0 (agents hold no legitimacy sources), so a wide slope would
        // halve pressure and cancel fear amplification; 0.4 gives a mild 0.8
        // dampener at legit=0 that amplification still out-weighs.
        let legitimacy = match category {
            MotiveCategory::Justice | MotiveCategory::Certainty | MotiveCategory::Safety => {
                (Fixed::ONE
                    + (self.cultural_legitimacy - Fixed::from_f64(0.5)) * Fixed::from_f64(0.4))
                .clamp(Fixed::from_f64(0.7), Fixed::from_f64(1.3))
            }
            _ => Fixed::ONE,
        };

        // Situational affordance (1.0 neutral; scarcity amplifies pressure).
        let affordance = match category {
            MotiveCategory::Hunger => Fixed::ONE + self.food_scarcity * Fixed::from_f64(0.5),
            MotiveCategory::Thirst => Fixed::ONE + self.water_scarcity * Fixed::from_f64(0.5),
            _ => Fixed::ONE,
        };

        (base * emotional * legitimacy * affordance).clamp_01()
    }

    /// Update the dominant need based on current pressures.
    ///
    /// Biological needs dominate when they are critical (pressure > 0.4).
    /// Otherwise, the highest-pressure need wins — evaluated with the plan's
    /// full five-factor pressure formula.
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
            self.dominant_need = max_bio
                .iter()
                .max_by_key(|(_, p)| p.to_raw())
                .map_or(MotiveCategory::Hunger, |(c, _)| *c);
        } else {
            // All needs compete equally with the full pressure formula
            let all = [
                (
                    MotiveCategory::Hunger,
                    self.pressure_full(MotiveCategory::Hunger),
                ),
                (
                    MotiveCategory::Thirst,
                    self.pressure_full(MotiveCategory::Thirst),
                ),
                (
                    MotiveCategory::Sleep,
                    self.pressure_full(MotiveCategory::Sleep),
                ),
                (
                    MotiveCategory::Warmth,
                    self.pressure_full(MotiveCategory::Warmth),
                ),
                (
                    MotiveCategory::Health,
                    self.pressure_full(MotiveCategory::Health),
                ),
                (
                    MotiveCategory::Safety,
                    self.pressure_full(MotiveCategory::Safety),
                ),
                (
                    MotiveCategory::Attachment,
                    self.pressure_full(MotiveCategory::Attachment),
                ),
                (
                    MotiveCategory::Belonging,
                    self.pressure_full(MotiveCategory::Belonging),
                ),
                (
                    MotiveCategory::Esteem,
                    self.pressure_full(MotiveCategory::Esteem),
                ),
                (
                    MotiveCategory::Autonomy,
                    self.pressure_full(MotiveCategory::Autonomy),
                ),
                (
                    MotiveCategory::Competence,
                    self.pressure_full(MotiveCategory::Competence),
                ),
                (
                    MotiveCategory::Meaning,
                    self.pressure_full(MotiveCategory::Meaning),
                ),
                (
                    MotiveCategory::Certainty,
                    self.pressure_full(MotiveCategory::Certainty),
                ),
                (
                    MotiveCategory::Novelty,
                    self.pressure_full(MotiveCategory::Novelty),
                ),
                (
                    MotiveCategory::Play,
                    self.pressure_full(MotiveCategory::Play),
                ),
                (
                    MotiveCategory::Care,
                    self.pressure_full(MotiveCategory::Care),
                ),
                (
                    MotiveCategory::Romance,
                    self.pressure_full(MotiveCategory::Romance),
                ),
                (
                    MotiveCategory::Justice,
                    self.pressure_full(MotiveCategory::Justice),
                ),
                (
                    MotiveCategory::Recognition,
                    self.pressure_full(MotiveCategory::Recognition),
                ),
            ];
            self.dominant_need = all
                .iter()
                .max_by_key(|(_, p)| p.to_raw())
                .map_or(MotiveCategory::Hunger, |(c, _)| *c);
        }
    }

    /// Get the current dominant need's pressure level (full formula).
    pub fn dominant_pressure(&self) -> Fixed {
        self.pressure_full(self.dominant_need)
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
        assert!(m.care.deficit > Fixed::ZERO);
        assert!(m.romance.deficit > Fixed::ZERO);
    }

    #[test]
    fn deficit_caps_at_one() {
        let mut n = MotiveNeed::new(Fixed::ONE, Fixed::ONE);
        for _ in 0..10 {
            n.tick();
        }
        assert!(n.deficit <= Fixed::ONE);
    }

    // ── §8.1.5 full-formula tests ─────────────────────────────────────

    #[test]
    fn context_defaults_to_neutral() {
        let m = MotivationState::default();
        assert_eq!(m.fear, Fixed::ZERO);
        assert_eq!(m.anger, Fixed::ZERO);
        assert_eq!(m.joy, Fixed::ZERO);
        assert_eq!(m.sadness, Fixed::ZERO);
        assert_eq!(m.cultural_legitimacy, Fixed::from_f64(0.5));
        assert_eq!(m.food_scarcity, Fixed::ZERO);
        assert_eq!(m.water_scarcity, Fixed::ZERO);
    }

    #[test]
    fn set_context_clamps_to_bounds() {
        let mut m = MotivationState::default();
        m.set_context(
            Fixed::from_f64(3.0),
            Fixed::from_f64(-1.0),
            Fixed::from_f64(0.7),
            Fixed::ZERO,
            Fixed::from_f64(0.2),
            Fixed::from_f64(2.0),
            Fixed::ZERO,
        );
        assert_eq!(m.fear, Fixed::ONE);
        assert_eq!(m.anger, Fixed::ZERO);
        assert_eq!(m.joy, Fixed::from_f64(0.7));
        assert_eq!(m.cultural_legitimacy, Fixed::from_f64(0.2));
        assert_eq!(m.food_scarcity, Fixed::ONE);
    }

    #[test]
    fn neutral_context_pressure_full_equals_base() {
        let mut m = MotivationState::default();
        m.belonging.deficit = Fixed::from_f64(0.5);
        let base = m.need(MotiveCategory::Belonging).pressure();
        assert_eq!(m.pressure_full(MotiveCategory::Belonging), base);
    }

    #[test]
    fn fear_amplifies_safety_pressure() {
        let mut m = MotivationState::default();
        m.safety.deficit = Fixed::from_f64(0.5);
        let calm = m.pressure_full(MotiveCategory::Safety);
        m.set_context(
            Fixed::from_f64(0.9),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::ZERO,
        );
        let afraid = m.pressure_full(MotiveCategory::Safety);
        assert!(afraid > calm, "fear must amplify safety pressure");
    }

    #[test]
    fn anger_amplifies_justice_pressure() {
        let mut m = MotivationState::default();
        m.justice.deficit = Fixed::from_f64(0.5);
        let calm = m.pressure_full(MotiveCategory::Justice);
        m.set_context(
            Fixed::ZERO,
            Fixed::from_f64(0.9),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::ZERO,
        );
        let angry = m.pressure_full(MotiveCategory::Justice);
        assert!(angry > calm, "anger must amplify justice pressure");
    }

    #[test]
    fn joy_amplifies_belonging_and_scarcity_amplifies_hunger() {
        let mut m = MotivationState::default();
        m.belonging.deficit = Fixed::from_f64(0.5);
        m.hunger.deficit = Fixed::from_f64(0.5);
        let calm = m.pressure_full(MotiveCategory::Belonging);
        let fed = m.pressure_full(MotiveCategory::Hunger);
        m.set_context(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.9),
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9),
            Fixed::ZERO,
        );
        assert!(
            m.pressure_full(MotiveCategory::Belonging) > calm,
            "joy must amplify belonging"
        );
        assert!(
            m.pressure_full(MotiveCategory::Hunger) > fed,
            "food scarcity must amplify hunger"
        );
    }

    #[test]
    fn care_and_romance_participate_in_dominance() {
        let mut m = MotivationState::default();
        m.care.deficit = Fixed::from_f64(0.9);
        m.romance.deficit = Fixed::from_f64(0.9);
        m.update_dominant();
        // With no bio-critical pressure and all else at zero, the highest
        // full pressure wins — care/romance must be eligible.
        assert!(
            m.dominant_need == MotiveCategory::Care || m.dominant_need == MotiveCategory::Romance,
            "care/romance must participate in dominance, got {:?}",
            m.dominant_need
        );
    }

    #[test]
    fn pressure_full_amplifies_unsaturated_but_caps_at_one_when_saturated() {
        let mut m = MotivationState::default();
        // Unsaturated regime: hunger deficit small, scarcity high — the
        // five-factor amplification must be visible above the base pressure.
        m.hunger.deficit = Fixed::from_f64(0.2);
        m.set_context(
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::from_f64(0.9),
            Fixed::ZERO,
        );
        let base = m.hunger.pressure();
        let amplified = m.pressure_full(MotiveCategory::Hunger);
        assert!(
            amplified > base,
            "amplification must be visible in the unsaturated regime: full={} base={}",
            amplified.to_f64(),
            base.to_f64()
        );
        // Saturated regime: a 1.0-deficit, 1.0-urgency need clamps at 1.0
        // regardless of the context factors (the formula must not overflow).
        m.hunger.deficit = Fixed::ONE;
        m.hunger.urgency_weight = Fixed::ONE;
        let saturated = m.pressure_full(MotiveCategory::Hunger);
        assert_eq!(
            saturated,
            Fixed::ONE,
            "saturated pressure must cap at 1.0, got {}",
            saturated.to_f64()
        );
    }

    #[test]
    fn fearful_agent_dominance_shifts_to_safety() {
        let mut m = MotivationState::default();
        // Belonging urgency (0.5) is below safety's (0.8), so belonging needs a
        // higher deficit to win at neutral: 0.7*0.5 = 0.35 > 0.4*0.8 = 0.32.
        m.belonging.deficit = Fixed::from_f64(0.7);
        m.safety.deficit = Fixed::from_f64(0.4);
        m.update_dominant();
        assert_eq!(m.dominant_need, MotiveCategory::Belonging);
        // Fear (0.9) amplifies safety pressure by 1.36x -> 0.435, flipping the
        // argmax — the formula is genuinely live.
        m.set_context(
            Fixed::from_f64(0.9),
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::ZERO,
            Fixed::from_f64(0.5),
            Fixed::ZERO,
            Fixed::ZERO,
        );
        m.update_dominant();
        assert_eq!(
            m.dominant_need,
            MotiveCategory::Safety,
            "fear must flip dominance to safety"
        );
    }
}
