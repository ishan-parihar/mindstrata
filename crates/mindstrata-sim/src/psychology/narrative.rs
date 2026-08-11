//! Narrative and meaning-making system — agents interpret life as a story.
//!
//! Events are interpreted through narrative frames:
//! - punishment as justice,
//! - suffering as test,
//! - loss as curse,
//! - success as blessing,
//! - betrayal as proof of unworthiness,
//! - survival as destiny.
//!
//! This feeds religion, ideology, and resilience.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Life narrative theme — the dominant story the agent tells about their life.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum LifeTheme {
    /// Life is a journey of growth and learning.
    #[default]
    Growth,
    /// Life is a struggle against adversity.
    Struggle,
    /// Life is a gift to be enjoyed.
    Gift,
    /// Life is a test of character and faith.
    Test,
    /// Life is meaningless (depressive narrative).
    Meaningless,
    /// Life is a punishment for past wrongs.
    Punishment,
    /// Life is a mission with purpose.
    Mission,
}

/// Narrative identity — the story the agent tells about themselves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentity {
    /// Dominant life theme.
    pub life_theme: LifeTheme,
    /// Redemption script strength (finding meaning in suffering). 0–1.
    pub redemption_script: Fixed,
    /// Contamination script strength (good things are ruined). 0–1.
    pub contamination_script: Fixed,
    /// Victimhood script strength (bad things always happen to me). 0–1.
    pub victimhood_script: Fixed,
    /// Heroism script strength (I overcome challenges). 0–1.
    pub heroism_script: Fixed,
    /// Chosenness script strength (I am special/chosen). 0–1.
    pub chosenness_script: Fixed,
    /// Shame script strength (I am fundamentally flawed). 0–1.
    pub shame_script: Fixed,
    /// Overall narrative coherence (how consistent the story feels). 0–1.
    pub coherence: Fixed,
    /// Number of major life events incorporated into narrative.
    pub events_integrated: u32,
}

impl Default for NarrativeIdentity {
    fn default() -> Self {
        Self {
            life_theme: LifeTheme::Growth,
            redemption_script: Fixed::from_f64(0.5),
            contamination_script: Fixed::from_f64(0.2),
            victimhood_script: Fixed::from_f64(0.2),
            heroism_script: Fixed::from_f64(0.5),
            chosenness_script: Fixed::from_f64(0.1),
            shame_script: Fixed::from_f64(0.1),
            coherence: Fixed::from_f64(0.6),
            events_integrated: 0,
        }
    }
}

impl NarrativeIdentity {
    /// Interpret a negative event through the narrative frame.
    pub fn interpret_negative_event(
        &mut self,
        severity: Fixed,
        social_support: Fixed,
        has_blame_target: bool,
    ) {
        // Contamination script strengthens with negative events
        self.contamination_script =
            (self.contamination_script + severity * Fixed::from_f64(0.01)).clamp_01();

        // Victimhood strengthens without support
        if social_support < Fixed::from_f64(0.3) {
            self.victimhood_script =
                (self.victimhood_script + severity * Fixed::from_f64(0.008)).clamp_01();
        }

        // Heroism strengthens when there's support to overcome
        if social_support > Fixed::from_f64(0.4) {
            self.heroism_script =
                (self.heroism_script + severity * Fixed::from_f64(0.005)).clamp_01();
        }

        // Redemption script strengthens when blame can be externalized
        if has_blame_target {
            self.redemption_script = (self.redemption_script + Fixed::from_f64(0.003)).clamp_01();
        }

        // Shame script strengthens from personal failure without external blame
        if !has_blame_target && social_support < Fixed::from_f64(0.2) {
            self.shame_script = (self.shame_script + severity * Fixed::from_f64(0.005)).clamp_01();
        }

        self.events_integrated += 1;
    }

    /// Interpret a positive event through the narrative frame.
    pub fn interpret_positive_event(&mut self, magnitude: Fixed, social_recognition: Fixed) {
        // Redemption script strengthens with positive recovery
        self.redemption_script =
            (self.redemption_script + magnitude * Fixed::from_f64(0.005)).clamp_01();

        // Heroism strengthens with recognition
        self.heroism_script =
            (self.heroism_script + social_recognition * Fixed::from_f64(0.003)).clamp_01();

        // Chosenness strengthens with exceptional success
        if magnitude > Fixed::from_f64(0.7) && social_recognition > Fixed::from_f64(0.6) {
            self.chosenness_script = (self.chosenness_script + Fixed::from_f64(0.002)).clamp_01();
        }

        // Contamination weakens with positive events
        self.contamination_script =
            (self.contamination_script - magnitude * Fixed::from_f64(0.003)).max(Fixed::ZERO);

        self.events_integrated += 1;
    }

    /// Update the dominant life theme based on current script balance.
    pub fn update_theme(&mut self) {
        let positive_balance =
            self.redemption_script + self.heroism_script + self.chosenness_script;
        let negative_balance =
            self.contamination_script + self.victimhood_script + self.shame_script;

        self.life_theme = if positive_balance > negative_balance * Fixed::from_f64(1.5) {
            if self.heroism_script > self.redemption_script {
                LifeTheme::Mission
            } else {
                LifeTheme::Growth
            }
        } else if negative_balance > positive_balance * Fixed::from_f64(1.5) {
            if self.victimhood_script > self.shame_script {
                LifeTheme::Struggle
            } else if self.shame_script > Fixed::from_f64(0.6) {
                LifeTheme::Punishment
            } else {
                LifeTheme::Meaningless
            }
        } else {
            LifeTheme::Test // balanced — life is a test
        };

        // Coherence is how consistent the narrative scripts are
        let max_script = positive_balance.max(negative_balance);
        let min_script = positive_balance.min(negative_balance);
        self.coherence = if max_script > Fixed::ZERO {
            (Fixed::ONE - (max_script - min_script) / max_script).clamp_01()
        } else {
            Fixed::from_f64(0.5)
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_event_increases_contamination() {
        let mut n = NarrativeIdentity::default();
        let initial = n.contamination_script;
        n.interpret_negative_event(Fixed::from_f64(0.7), Fixed::from_f64(0.3), false);
        assert!(n.contamination_script > initial);
    }

    #[test]
    fn positive_event_increases_redemption() {
        let mut n = NarrativeIdentity::default();
        let initial = n.redemption_script;
        n.interpret_positive_event(Fixed::from_f64(0.5), Fixed::from_f64(0.5));
        assert!(n.redemption_script > initial);
    }

    #[test]
    fn theme_updates_based_on_balance() {
        let mut n = NarrativeIdentity {
            heroism_script: Fixed::from_f64(0.9),
            redemption_script: Fixed::from_f64(0.7),
            contamination_script: Fixed::from_f64(0.1),
            victimhood_script: Fixed::from_f64(0.1),
            ..Default::default()
        };
        n.update_theme();
        assert_eq!(n.life_theme, LifeTheme::Mission);
    }
}
