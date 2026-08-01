//! §8.1.17: Narrative frames — how events are interpreted through meaning-making lenses.
//!
//! Architecture §8.1.17: Events are interpreted through narrative frames:
//! punishment as justice, suffering as test, loss as curse, success as blessing,
//! betrayal as proof of unworthiness, survival as destiny.
//!
//! Narrative frames are cultural constructs that shape how agents make sense
//! of their experiences. They feed religion, ideology, and resilience.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A narrative frame that shapes event interpretation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeFrame {
    /// Name of the narrative frame.
    pub name: String,
    /// How strongly this frame is held by the agent (0–1).
    pub strength: Fixed,
    /// Emotional valence of the frame (negative = punitive, positive = redemptive).
    pub valence: Fixed,
    /// Identity relevance — how much this frame is tied to self-concept (0–1).
    pub identity_relevance: Fixed,
}

/// Registry of common narrative frames in the culture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeFrameSet {
    /// Events are punishments for wrongdoing.
    pub punishment_as_justice: Fixed,
    /// Suffering is a test of faith or character.
    pub suffering_as_test: Fixed,
    /// Loss is a curse or punishment from the divine.
    pub loss_as_curse: Fixed,
    /// Success is a blessing from the divine or the universe.
    pub success_as_blessing: Fixed,
    /// Betrayal proves one's unworthiness.
    pub betrayal_as_unworthiness: Fixed,
    /// Survival proves destiny or chosenness.
    pub survival_as_destiny: Fixed,
    /// Hardship builds character and strength.
    pub hardship_builds_character: Fixed,
    /// The world is fundamentally fair (just-world hypothesis).
    pub just_world: Fixed,
}

impl Default for NarrativeFrameSet {
    fn default() -> Self {
        Self {
            punishment_as_justice: Fixed::from_f64(0.3),
            suffering_as_test: Fixed::from_f64(0.2),
            loss_as_curse: Fixed::from_f64(0.15),
            success_as_blessing: Fixed::from_f64(0.3),
            betrayal_as_unworthiness: Fixed::from_f64(0.2),
            survival_as_destiny: Fixed::from_f64(0.2),
            hardship_builds_character: Fixed::from_f64(0.25),
            just_world: Fixed::from_f64(0.4),
        }
    }
}

impl NarrativeFrameSet {
    /// Interpret a negative event through the agent's narrative frames.
    ///
    /// Returns an emotional modulation (0 = no reinterpretation, 1 = full reinterpretation).
    pub fn interpret_negative(&self) -> Fixed {
        // Negative frames amplify the sting of bad events
        (self.punishment_as_justice * Fixed::from_f64(0.2)
            + self.loss_as_curse * Fixed::from_f64(0.2)
            + self.betrayal_as_unworthiness * Fixed::from_f64(0.2)
            + self.just_world * Fixed::from_f64(0.15))
            .clamp_01()
    }

    /// Interpret a positive event through the agent's narrative frames.
    ///
    /// Returns an emotional boost from narrative meaning-making.
    pub fn interpret_positive(&self) -> Fixed {
        // Positive frames amplify good fortune
        (self.success_as_blessing * Fixed::from_f64(0.2)
            + self.survival_as_destiny * Fixed::from_f64(0.2)
            + self.hardship_builds_character * Fixed::from_f64(0.15))
            .clamp_01()
    }

    /// Resilience factor — how much narrative meaning-making buffers against adversity.
    pub fn resilience_factor(&self) -> Fixed {
        (self.suffering_as_test * Fixed::from_f64(0.3)
            + self.hardship_builds_character * Fixed::from_f64(0.3)
            + self.survival_as_destiny * Fixed::from_f64(0.2)
            + self.success_as_blessing * Fixed::from_f64(0.2))
            .clamp_01()
    }

    /// Compute how much this narrative frame set resists countervailing evidence.
    pub fn resistance_to_update(&self) -> Fixed {
        // Frames with high identity relevance and emotional charge resist change
        let max_frame = self
            .punishment_as_justice
            .max(self.suffering_as_test)
            .max(self.just_world)
            .max(self.survival_as_destiny);
        (max_frame * Fixed::from_f64(0.5) + Fixed::from_f64(0.3)).clamp_01()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn negative_interpretation_uses_frames() {
        let frames = NarrativeFrameSet {
            punishment_as_justice: Fixed::from_f64(0.8),
            ..NarrativeFrameSet::default()
        };
        let result = frames.interpret_negative();
        assert!(result > Fixed::from_f64(0.1));
    }

    #[test]
    fn positive_interpretation_uses_frames() {
        let frames = NarrativeFrameSet {
            success_as_blessing: Fixed::from_f64(0.9),
            ..NarrativeFrameSet::default()
        };
        let result = frames.interpret_positive();
        assert!(result > Fixed::from_f64(0.1));
    }

    #[test]
    fn resilience_factor_increases_with_suffering_as_test() {
        let low = NarrativeFrameSet {
            suffering_as_test: Fixed::from_f64(0.1),
            ..NarrativeFrameSet::default()
        };
        let high = NarrativeFrameSet {
            suffering_as_test: Fixed::from_f64(0.9),
            ..NarrativeFrameSet::default()
        };
        assert!(high.resilience_factor() > low.resilience_factor());
    }
}
