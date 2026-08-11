//! Developmental system — life stages, aging, puberty progression, developmental milestones.
//!
//! Life stages: infant → child → adolescent → young_adult → adult → mature → elder.
//! Developmental processes: attachment formation, temperament expression, socialization,
//! identity formation, skill acquisition, aging/decline.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Life stage classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum LifeStage {
    Infant,
    Child,
    Adolescent,
    YoungAdult,
    Adult,
    Mature,
    Elder,
}

impl LifeStage {
    /// Convert age in years to life stage.
    pub fn from_age(age: Fixed) -> Self {
        let a = age.to_f64();
        if a < 2.0 {
            Self::Infant
        } else if a < 12.0 {
            Self::Child
        } else if a < 18.0 {
            Self::Adolescent
        } else if a < 30.0 {
            Self::YoungAdult
        } else if a < 50.0 {
            Self::Adult
        } else if a < 65.0 {
            Self::Mature
        } else {
            Self::Elder
        }
    }

    /// Developmental modifier for physical potential (child=high growth, elder=decline).
    pub fn physical_modifier(&self) -> Fixed {
        match self {
            Self::Infant => Fixed::from_f64(0.2),
            Self::Child => Fixed::from_f64(0.6),
            Self::Adolescent => Fixed::from_f64(0.9),
            Self::YoungAdult => Fixed::ONE,
            Self::Adult => Fixed::from_f64(0.95),
            Self::Mature => Fixed::from_f64(0.8),
            Self::Elder => Fixed::from_f64(0.5),
        }
    }

    /// Cognitive modifier (children learn fast, elders have wisdom but slower processing).
    pub fn cognitive_modifier(&self) -> Fixed {
        match self {
            Self::Infant => Fixed::from_f64(0.1),
            Self::Child => Fixed::from_f64(0.7),
            Self::Adolescent => Fixed::from_f64(0.9),
            Self::YoungAdult => Fixed::ONE,
            Self::Adult => Fixed::from_f64(0.95),
            Self::Mature => Fixed::from_f64(0.9),
            Self::Elder => Fixed::from_f64(0.7),
        }
    }

    /// Social influence susceptibility (adolescents most susceptible).
    pub fn social_susceptibility(&self) -> Fixed {
        match self {
            Self::Infant => Fixed::from_f64(0.1),
            Self::Child => Fixed::from_f64(0.8),
            Self::Adolescent => Fixed::from_f64(0.95),
            Self::YoungAdult => Fixed::from_f64(0.7),
            Self::Adult => Fixed::from_f64(0.5),
            Self::Mature => Fixed::from_f64(0.4),
            Self::Elder => Fixed::from_f64(0.3),
        }
    }
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
    /// Educational experiences count.
    pub education_events: u32,
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
            education_events: 0,
            positive_socialization: 0,
            negative_socialization: 0,
        }
    }
}

/// Developmental state — tracks current life stage and developmental modifiers.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DevelopmentalState {
    /// Current life stage.
    pub life_stage: LifeStage,
    /// Age in years.
    pub age: Fixed,
    /// Accumulated developmental history.
    pub history: DevelopmentalHistory,
    /// Developmental environment quality (0–1). Derived from nutrition + safety + social.
    pub environment_quality: Fixed,
    /// Whether puberty has been reached.
    pub pubertal_onset: bool,
    /// Whether key developmental milestones have been achieved.
    pub milestones_achieved: u32,
}

impl Default for DevelopmentalState {
    fn default() -> Self {
        Self {
            life_stage: LifeStage::Adult,
            age: Fixed::from_f64(25.0),
            history: DevelopmentalHistory::default(),
            environment_quality: Fixed::from_f64(0.5),
            pubertal_onset: true,
            milestones_achieved: 0,
        }
    }
}

impl DevelopmentalState {
    /// Advance age by one tick (called once per day or per appropriate timescale).
    pub fn advance_age(&mut self, years_per_tick: Fixed) {
        self.age += years_per_tick; // age is in years, not 0-1
        let new_stage = LifeStage::from_age(self.age);
        if new_stage != self.life_stage {
            self.life_stage = new_stage;
            self.milestones_achieved += 1;
            if self.life_stage == LifeStage::Adolescent && !self.pubertal_onset {
                self.pubertal_onset = true;
                self.milestones_achieved += 1;
            }
        }
    }

    /// Compute environment quality from current conditions.
    pub fn update_environment(&mut self, nutrition: Fixed, safety: Fixed, social_support: Fixed) {
        self.environment_quality = (nutrition * Fixed::from_f64(0.4)
            + safety * Fixed::from_f64(0.3)
            + social_support * Fixed::from_f64(0.3))
        .clamp_01();
    }

    /// Developmental modifier for trait expression (environment moderates genetics).
    pub fn trait_expression_modifier(&self) -> Fixed {
        self.environment_quality
    }

    /// Physical development modifier based on life stage.
    pub fn physical_modifier(&self) -> Fixed {
        self.life_stage.physical_modifier()
    }

    /// Cognitive development modifier.
    pub fn cognitive_modifier(&self) -> Fixed {
        self.life_stage.cognitive_modifier()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn life_stage_from_age() {
        assert_eq!(LifeStage::from_age(Fixed::from_f64(5.0)), LifeStage::Child);
        assert_eq!(
            LifeStage::from_age(Fixed::from_f64(15.0)),
            LifeStage::Adolescent
        );
        assert_eq!(
            LifeStage::from_age(Fixed::from_f64(25.0)),
            LifeStage::YoungAdult
        );
        assert_eq!(
            LifeStage::from_age(Fixed::from_f64(55.0)),
            LifeStage::Mature
        );
        assert_eq!(LifeStage::from_age(Fixed::from_f64(70.0)), LifeStage::Elder);
    }

    #[test]
    fn age_advances() {
        let mut dev = DevelopmentalState {
            age: Fixed::from_f64(17.0),
            ..Default::default()
        };
        dev.advance_age(Fixed::from_f64(2.0)); // 17 + 2 = 19
        assert_eq!(dev.life_stage, LifeStage::YoungAdult);
    }

    #[test]
    fn environment_quality_from_inputs() {
        let mut dev = DevelopmentalState::default();
        dev.update_environment(Fixed::ONE, Fixed::ONE, Fixed::ONE);
        assert_eq!(dev.environment_quality, Fixed::ONE);
    }
}
