//! Self-model system — identity, roles, values, and narrative.
//!
//! The self-model is essential for "mind-like" agents. It contains:
//! - Identity claims ("I am a farmer", "I am a protector")
//! - Role identities (Elder, Priest, Merchant)
//! - Value commitments (honesty, loyalty, justice)
//! - Sacred values (protected from change at any cost)
//! - Self-esteem and identity security
//! - Narrative identity (life theme, redemption/contamination scripts)
//!
//! Identity-linked beliefs resist change — this is how ideology,
//! propaganda, and polarization emerge naturally.

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A claim about who the agent is.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityClaim {
    /// What the claim is (e.g., "I am a farmer").
    pub claim: String,
    /// How strongly the agent holds this identity (0–1).
    pub strength: Fixed,
    /// How much this identity is threatened by current events.
    pub threat_level: Fixed,
    /// Whether this identity is sacred (protected from any change).
    pub sacred: bool,
}

/// A role the agent occupies in society.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoleIdentity {
    /// Role name (e.g., "Elder", "Priest", "Parent").
    pub role: String,
    /// How central this role is to the agent's self-concept.
    pub centrality: Fixed,
    /// How competent the agent feels in this role.
    pub competence: Fixed,
}

/// A value the agent commits to.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValueCommitment {
    /// Value name (e.g., "honesty", "loyalty", "justice").
    pub value: String,
    /// Strength of commitment (0–1).
    pub commitment: Fixed,
    /// Whether violations of this value produce moral outrage.
    pub outrage_on_violation: Fixed,
}

/// Life narrative theme — how the agent interprets their life story.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifeTheme {
    /// Life is a journey of growth.
    Growth,
    /// Life is a struggle against adversity.
    Struggle,
    /// Life is a gift to be enjoyed.
    Gift,
    /// Life is a test of character.
    Test,
    /// Life is meaningless (depressive narrative).
    Meaningless,
}

/// Narrative identity — the story the agent tells about themselves.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NarrativeIdentity {
    /// Dominant life theme.
    pub life_theme: LifeTheme,
    /// Redemption script strength (finding meaning in suffering).
    pub redemption_script: Fixed,
    /// Contamination script strength (good things are ruined).
    pub contamination_script: Fixed,
    /// Victimhood script strength (bad things always happen to me).
    pub victimhood_script: Fixed,
    /// Heroism script strength (I overcome challenges).
    pub heroism_script: Fixed,
}

impl Default for NarrativeIdentity {
    fn default() -> Self {
        Self {
            life_theme: LifeTheme::Growth,
            redemption_script: Fixed::from_f64(0.5),
            contamination_script: Fixed::from_f64(0.2),
            victimhood_script: Fixed::from_f64(0.2),
            heroism_script: Fixed::from_f64(0.5),
        }
    }
}

/// The agent's self-model — their internal representation of who they are.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelfModel {
    /// Identity claims the agent holds.
    pub claims: Vec<IdentityClaim>,
    /// Roles the agent occupies.
    pub roles: Vec<RoleIdentity>,
    /// Values the agent commits to.
    pub values: Vec<ValueCommitment>,
    /// Sacred values — cannot be changed without existential crisis.
    pub sacred_values: Vec<String>,
    /// Current self-esteem (0–1).
    pub self_esteem: Fixed,
    /// Identity coherence — how consistent the agent's self-concept is.
    pub coherence: Fixed,
    /// Identity security — how threatened the agent feels.
    pub security: Fixed,
    /// Shame proneness.
    pub shame_proneness: Fixed,
    /// Guilt proneness.
    pub guilt_proneness: Fixed,
    /// Narrative identity — the story the agent tells about themselves.
    pub narrative: NarrativeIdentity,
}

impl Default for SelfModel {
    fn default() -> Self {
        Self {
            claims: Vec::new(),
            roles: Vec::new(),
            values: Vec::new(),
            sacred_values: Vec::new(),
            self_esteem: Fixed::from_f64(0.5),
            coherence: Fixed::from_f64(0.6),
            security: Fixed::from_f64(0.5),
            shame_proneness: Fixed::from_f64(0.4),
            guilt_proneness: Fixed::from_f64(0.4),
            narrative: NarrativeIdentity::default(),
        }
    }
}

impl SelfModel {
    /// Check if a belief is linked to any identity claim.
    /// Identity-linked beliefs resist change.
    pub fn identity_linkage(&self, identity_keywords: &[&str]) -> Fixed {
        let mut max_linkage = Fixed::ZERO;
        for claim in &self.claims {
            for keyword in identity_keywords {
                if claim.claim.to_lowercase().contains(&keyword.to_lowercase()) {
                    max_linkage = max_linkage.max(claim.strength);
                }
            }
        }
        max_linkage
    }

    /// Compute belief resistance based on identity linkage and sacredness.
    /// `base_resistance` is the belief's inherent resistance.
    pub fn belief_resistance(
        &self,
        base_resistance: Fixed,
        identity_linkage: Fixed,
        social_reinforcement: u32,
    ) -> Fixed {
        let identity_boost = identity_linkage * Fixed::from_f64(0.4);
        let social_boost = Fixed::from_f64(social_reinforcement as f64 * 0.02);
        (base_resistance + identity_boost + social_boost).clamp_01()
    }

    /// Threaten an identity claim — reduces security.
    pub fn threaten_identity(&mut self, claim_index: usize, severity: Fixed) {
        if let Some(claim) = self.claims.get_mut(claim_index) {
            claim.threat_level = (claim.threat_level + severity).clamp_01();
            // Threatened identities reduce security
            self.security = (self.security - severity * Fixed::from_f64(0.1)).max(Fixed::ZERO);
        }
    }

    /// Update narrative scripts based on recent events.
    pub fn update_narrative(
        &mut self,
        negative_events: Fixed,
        positive_events: Fixed,
        social_support: Fixed,
    ) {
        // Contamination script strengthens with negative events
        self.narrative.contamination_script =
            (self.narrative.contamination_script + negative_events * Fixed::from_f64(0.01))
                .clamp_01();
        // Redemption script strengthens with positive events + social support
        self.narrative.redemption_script =
            (self.narrative.redemption_script + positive_events * Fixed::from_f64(0.01)
                + social_support * Fixed::from_f64(0.005))
                .clamp_01();
        // Heroism script strengthens with overcoming adversity
        if negative_events > Fixed::from_f64(0.5) && social_support > Fixed::from_f64(0.3) {
            self.narrative.heroism_script =
                (self.narrative.heroism_script + Fixed::from_f64(0.005)).clamp_01();
        }
        // Victimhood script strengthens with negative events without support
        if negative_events > Fixed::from_f64(0.5) && social_support < Fixed::from_f64(0.2) {
            self.narrative.victimhood_script =
                (self.narrative.victimhood_script + Fixed::from_f64(0.01)).clamp_01();
        }
        // Life theme shifts based on balance
        let balance = positive_events - negative_events;
        if balance > Fixed::from_f64(0.2) {
            self.narrative.life_theme = LifeTheme::Growth;
        } else if balance < -Fixed::from_f64(0.3) {
            if self.narrative.heroism_script > Fixed::from_f64(0.5) {
                self.narrative.life_theme = LifeTheme::Test;
            } else {
                self.narrative.life_theme = LifeTheme::Struggle;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn identity_linkage_returns_max_strength() {
        let mut model = SelfModel::default();
        model.claims.push(IdentityClaim {
            claim: "I am a farmer".into(),
            strength: Fixed::from_f64(0.8),
            threat_level: Fixed::ZERO,
            sacred: false,
        });
        model.claims.push(IdentityClaim {
            claim: "I am a parent".into(),
            strength: Fixed::from_f64(0.3),
            threat_level: Fixed::ZERO,
            sacred: false,
        });
        let linkage = model.identity_linkage(&["farmer"]);
        assert_eq!(linkage, Fixed::from_f64(0.8));
    }

    #[test]
    fn belief_resistance_increases_with_identity() {
        let model = SelfModel::default();
        let base = Fixed::from_f64(0.3);
        let with_identity = model.belief_resistance(base, Fixed::from_f64(0.7), 5);
        assert!(with_identity > base);
    }

    #[test]
    fn threaten_identity_reduces_security() {
        let mut model = SelfModel {
            security: Fixed::from_f64(0.8),
            ..SelfModel::default()
        };
        model.claims.push(IdentityClaim {
            claim: "I am a protector".into(),
            strength: Fixed::from_f64(0.9),
            threat_level: Fixed::ZERO,
            sacred: false,
        });
        model.threaten_identity(0, Fixed::from_f64(0.5));
        assert!(model.security < Fixed::from_f64(0.8));
    }
}
