//! §11.1: Legitimacy field — how institutional and social power is perceived.
//!
//! Architecture §11.1: Legitimacy is power through perceived rightfulness.
//! It is distinct from dominance (fear/coercion), prestige (admiration/competence),
//! and authority (institutional role). Legitimacy emerges from:
//! - Ritual participation
//! - Moral reputation
//! - Narrative coherence
//! - Community recognition
//! - Historical precedent

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A source of legitimacy for an entity (institution, leader, norm).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegitimacySource {
    /// Name of the source (e.g., "divine mandate", "popular consent", "tradition").
    pub name: String,
    /// Strength of this legitimacy source (0–1).
    pub strength: Fixed,
    /// How quickly this source decays without reinforcement.
    pub decay_rate: Fixed,
    /// Whether this source requires ongoing ritual to maintain.
    pub requires_ritual: bool,
}

/// The legitimacy field for an entity.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegitimacyField {
    /// All legitimacy sources for this entity.
    pub sources: Vec<LegitimacySource>,
    /// Overall legitimacy score (0–1).
    pub overall: Fixed,
    /// How quickly legitimacy decays per tick.
    pub base_decay_rate: Fixed,
    /// Threshold below which challenges become likely.
    pub challenge_threshold: Fixed,
    /// Threshold above which the entity is considered fully legitimate.
    pub stability_threshold: Fixed,
}

impl Default for LegitimacyField {
    fn default() -> Self {
        Self {
            sources: Vec::new(),
            overall: Fixed::from_f64(0.5),
            base_decay_rate: Fixed::from_f64(0.0001),
            challenge_threshold: Fixed::from_f64(0.3),
            stability_threshold: Fixed::from_f64(0.7),
        }
    }
}

impl LegitimacyField {
    /// Create a new legitimacy field with a given base level.
    pub fn new(base: Fixed) -> Self {
        Self {
            overall: base,
            ..Self::default()
        }
    }

    /// Add a legitimacy source.
    pub fn add_source(&mut self, source: LegitimacySource) {
        self.sources.push(source);
        self.recompute();
    }

    /// Recompute overall legitimacy from all sources.
    pub fn recompute(&mut self) {
        if self.sources.is_empty() {
            return;
        }
        let total: Fixed = self
            .sources
            .iter()
            .map(|s| s.strength)
            .fold(Fixed::ZERO, |acc, s| acc + s);
        let count = Fixed::from_int(self.sources.len() as i64);
        self.overall = (total / count).clamp_01();
    }

    /// Check if the entity is under legitimacy pressure (challenges likely).
    pub fn under_pressure(&self) -> bool {
        self.overall < self.challenge_threshold
    }

    /// Daily update — decay all sources, recompute.
    pub fn daily_update(&mut self) {
        let mut needs_ritual_decay = false;
        for source in &mut self.sources {
            // Non-ritual sources decay at their own rate
            if !source.requires_ritual {
                source.strength = (source.strength - source.decay_rate).max(Fixed::ZERO);
            } else {
                // Ritual sources decay faster without reinforcement
                source.strength =
                    (source.strength - source.decay_rate * Fixed::from_f64(2.0)).max(Fixed::ZERO);
                needs_ritual_decay = true;
            }
        }
        if needs_ritual_decay {
            // Base decay also applies
            self.overall = (self.overall - self.base_decay_rate).max(Fixed::ZERO);
        }
        self.recompute();
    }

    /// Boost legitimacy from a ritual event.
    pub fn ritual_boost(&mut self, magnitude: Fixed) {
        for source in &mut self.sources {
            if source.requires_ritual {
                source.strength = (source.strength + magnitude * Fixed::from_f64(0.02)).clamp_01();
            }
        }
        self.overall = (self.overall + magnitude * Fixed::from_f64(0.01)).clamp_01();
    }

    /// Damage legitimacy from a scandal or violation.
    pub fn scandal_damage(&mut self, severity: Fixed) {
        for source in &mut self.sources {
            // "popular consent" and "moral reputation" sources are most affected
            if source.name == "popular consent" || source.name == "moral reputation" {
                source.strength =
                    (source.strength - severity * Fixed::from_f64(0.05)).max(Fixed::ZERO);
            }
        }
        self.overall = (self.overall - severity * Fixed::from_f64(0.03)).max(Fixed::ZERO);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn legitimacy_computed_from_sources() {
        let mut field = LegitimacyField::default();
        field.add_source(LegitimacySource {
            name: "tradition".into(),
            strength: Fixed::from_f64(0.8),
            decay_rate: Fixed::from_f64(0.001),
            requires_ritual: false,
        });
        field.add_source(LegitimacySource {
            name: "popular consent".into(),
            strength: Fixed::from_f64(0.6),
            decay_rate: Fixed::from_f64(0.002),
            requires_ritual: true,
        });
        assert!(field.overall > Fixed::from_f64(0.5));
    }

    #[test]
    fn legitimacy_under_pressure() {
        let field = LegitimacyField::new(Fixed::from_f64(0.2));
        assert!(field.under_pressure());
    }

    #[test]
    fn scandal_reduces_legitimacy() {
        let mut field = LegitimacyField::new(Fixed::from_f64(0.7));
        field.add_source(LegitimacySource {
            name: "moral reputation".into(),
            strength: Fixed::from_f64(0.8),
            decay_rate: Fixed::from_f64(0.001),
            requires_ritual: false,
        });
        let before = field.overall;
        field.scandal_damage(Fixed::from_f64(0.5));
        assert!(field.overall < before);
    }
}
