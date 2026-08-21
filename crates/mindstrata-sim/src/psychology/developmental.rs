//! Developmental psychology system — agents develop across life.
//!
//! Life stages: infant → child → adolescent → young_adult → adult → mature → elder.
//! Developmental processes: attachment formation, temperament expression,
//! socialization, identity formation, skill acquisition, aging/decline.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Socialization style received during upbringing.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum SocializationStyle {
    /// Authoritative: warm but structured.
    #[default]
    Authoritative,
    /// Authoritarian: strict, low warmth.
    Authoritarian,
    /// Permissive: warm but unstructured.
    Permissive,
    /// Neglectful: low warmth, low structure.
    Neglectful,
}

/// Educational event — a learning experience.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EducationEvent {
    /// What was learned.
    pub skill: String,
    /// Quality of instruction (0–1).
    pub quality: Fixed,
    /// Tick when learned.
    pub tick: u64,
}

/// Cultural imprint — a value or belief absorbed from culture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CulturalImprint {
    /// What was imprinted.
    pub content: String,
    /// Strength of imprint (0–1).
    pub strength: Fixed,
    /// Source agent (who taught this).
    pub source: Option<u64>,
}

/// Developmental history — accumulated experiences that shape the agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentalHistory {
    /// Quality of caregiving received in childhood (0–1).
    pub caregiver_security: Fixed,
    /// Cumulative nutrition quality during development (0–1).
    pub nutrition_history: Fixed,
    /// Trauma experienced during development (0–1).
    pub trauma_history: Fixed,
    /// Socialization style received.
    pub socialization_style: SocializationStyle,
    /// Educational events.
    pub education_events: Vec<EducationEvent>,
    /// Cultural imprints received.
    pub cultural_imprints: Vec<CulturalImprint>,
    /// Number of positive socialization events.
    pub positive_socialization: u32,
    /// Number of negative socialization events.
    pub negative_socialization: u32,
}

impl Default for DevelopmentalHistory {
    fn default() -> Self {
        Self {
            caregiver_security: Fixed::from_f64(0.5),
            nutrition_history: Fixed::from_f64(0.5),
            trauma_history: Fixed::ZERO,
            socialization_style: SocializationStyle::default(),
            education_events: Vec::new(),
            cultural_imprints: Vec::new(),
            positive_socialization: 0,
            negative_socialization: 0,
        }
    }
}

/// Developmental state — tracks developmental processes across the lifespan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentalPsychState {
    /// Accumulated developmental history.
    pub history: DevelopmentalHistory,
    /// Whether key developmental milestones have been achieved.
    pub milestones_achieved: u32,
    /// Identity formation progress (0 = no identity, 1 = fully formed).
    pub identity_formation: Fixed,
    /// Moral development level (0 = preconventional, 1 = postconventional).
    pub moral_development: Fixed,
    /// Socialization completeness (0 = unsocialized, 1 = fully socialized).
    pub socialization_completeness: Fixed,
    /// Cognitive development level (0–1, maps to life stage).
    pub cognitive_development: Fixed,
}

impl Default for DevelopmentalPsychState {
    fn default() -> Self {
        Self {
            history: DevelopmentalHistory::default(),
            milestones_achieved: 0,
            identity_formation: Fixed::from_f64(0.5),
            moral_development: Fixed::from_f64(0.4),
            socialization_completeness: Fixed::from_f64(0.5),
            cognitive_development: Fixed::from_f64(0.7),
        }
    }
}

impl DevelopmentalPsychState {
    /// Update developmental state based on age and experiences.
    pub fn tick_update(
        &mut self,
        age_years: Fixed,
        social_support: Fixed,
        education_quality: Fixed,
    ) {
        let age = age_years.to_f64();

        // Identity formation progresses through adolescence and young adulthood
        if age > 12.0 && age < 30.0 {
            let progress_rate = Fixed::from_f64(0.001) * education_quality;
            self.identity_formation = (self.identity_formation + progress_rate).clamp_01();
        }

        // Moral development progresses throughout life
        let moral_rate = Fixed::from_f64(0.0005) * social_support;
        self.moral_development = (self.moral_development + moral_rate).clamp_01();

        // Socialization completeness builds through childhood and adolescence
        if age < 18.0 {
            let social_rate = Fixed::from_f64(0.001) * social_support;
            self.socialization_completeness =
                (self.socialization_completeness + social_rate).clamp_01();
        }

        // Cognitive development follows life stage
        self.cognitive_development = if age < 2.0 {
            Fixed::from_f64(0.1)
        } else if age < 12.0 {
            Fixed::from_f64(0.1 + (age - 2.0) * 0.06)
        } else if age < 18.0 {
            Fixed::from_f64(0.7 + (age - 12.0) * 0.04)
        } else if age < 50.0 {
            Fixed::ONE
        } else {
            (Fixed::ONE - Fixed::from_f64((age - 50.0) * 0.01)).max(Fixed::from_f64(0.5))
        };
    }

    /// Record a negative socialization event (trauma, abuse, neglect).
    pub fn negative_socialization(&mut self, severity: Fixed) {
        self.history.negative_socialization += 1;
        self.history.trauma_history =
            (self.history.trauma_history + severity * Fixed::from_f64(0.02)).clamp_01();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_forms_during_adolescence() {
        let mut dev = DevelopmentalPsychState {
            identity_formation: Fixed::from_f64(0.3),
            ..Default::default()
        };
        dev.tick_update(
            Fixed::from_f64(16.0),
            Fixed::from_f64(0.7),
            Fixed::from_f64(0.6),
        );
        assert!(dev.identity_formation > Fixed::from_f64(0.3));
    }

    #[test]
    fn socialization_builds_during_childhood() {
        let mut dev = DevelopmentalPsychState {
            socialization_completeness: Fixed::from_f64(0.3),
            ..Default::default()
        };
        dev.tick_update(
            Fixed::from_f64(8.0),
            Fixed::from_f64(0.8),
            Fixed::from_f64(0.5),
        );
        assert!(dev.socialization_completeness > Fixed::from_f64(0.3));
    }

    #[test]
    fn negative_socialization_increases_trauma() {
        let mut dev = DevelopmentalPsychState::default();
        dev.negative_socialization(Fixed::from_f64(0.8));
        assert!(dev.history.trauma_history > Fixed::ZERO);
    }

    /// Iteration 196: education quality (the agent's learning aptitude) must
    /// scale identity-formation progress — higher-quality education forms
    /// identity faster during the 12–30 window.
    #[test]
    fn education_quality_scales_identity_formation() {
        let mut low = DevelopmentalPsychState {
            identity_formation: Fixed::from_f64(0.3),
            ..Default::default()
        };
        let mut high = DevelopmentalPsychState {
            identity_formation: Fixed::from_f64(0.3),
            ..Default::default()
        };
        for _ in 0..10 {
            low.tick_update(
                Fixed::from_f64(16.0),
                Fixed::from_f64(0.7),
                Fixed::from_f64(0.2),
            );
            high.tick_update(
                Fixed::from_f64(16.0),
                Fixed::from_f64(0.7),
                Fixed::from_f64(0.9),
            );
        }
        assert!(
            high.identity_formation > low.identity_formation,
            "higher education quality must form identity faster ({} vs {})",
            high.identity_formation,
            low.identity_formation
        );
    }
}
