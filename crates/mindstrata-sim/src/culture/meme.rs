//! Meme system — cultural units that propagate, mutate, and compete.
//!
//! Memes are the atoms of culture. They spread through gossip, ritual,
//! institutional broadcasting, and social interaction. They mutate through
//! memory distortion, emotional exaggeration, and identity bias.
//!
//! ```text
//! Meme propagation:
//!   transmission_chance =
//!       source_credibility
//!     × emotional_arousal
//!     × identity_relevance
//!     × novelty
//!     × listener_susceptibility
//!     - skepticism
//!     - belief_contradiction
//!     - suppression
//!
//! Meme mutation:
//!   mutation =
//!     memory_error
//!   + emotional_exaggeration
//!   + identity_bias
//!   + narrative_simplification
//! ```
//!
//! Emergent effects:
//! - Rumors mutate as they spread through gossip networks
//! - Institutional propaganda competes with民间 narratives
//! - Sacred memes resist mutation
//! - Emotional memes spread faster than neutral ones
//! - Echo chambers form around identity-linked memes

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Content type of a meme.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemeContent {
    /// Accusation or blame.
    Accusation,
    /// Praise or honor.
    Praise,
    /// Theological or spiritual claim.
    Theological,
    /// Political accusation or claim.
    Political,
    /// Moral norm or value statement.
    Moral,
    /// Conspiracy theory.
    Conspiracy,
    /// Prophecy or prediction.
    Prophecy,
    /// Historical narrative or myth.
    Historical,
    /// Practical knowledge or skill.
    Practical,
    /// Taboo or prohibition.
    Taboo,
    /// Joke or humor.
    Joke,
    /// Slogan or rallying cry.
    Slogan,
}

/// A meme — a self-replicating cultural unit.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Meme {
    /// Unique meme identifier.
    pub id: usize,
    /// Brief description of the meme's content.
    pub description: String,
    /// Content type.
    pub content_type: MemeContent,
    /// Emotional charge (0 = neutral, 1 = highly emotional).
    pub emotional_charge: Fixed,
    /// Identity relevance (0 = irrelevant, 1 = core identity).
    pub identity_relevance: Fixed,
    /// Moral charge (0 = amoral, 1 = morally charged).
    pub moral_charge: Fixed,
    /// Base credibility (0 = incredible, 1 = universally trusted).
    pub credibility: Fixed,
    /// Novelty (0 = stale, 1 = brand new). Decays over time.
    pub novelty: Fixed,
    /// Virality — how easily this meme spreads (0 = inert, 1 = highly viral).
    pub virality: Fixed,
    /// Mutation rate (0 = stable, 1 = mutates every transmission).
    pub mutation_rate: Fixed,
    /// Number of agents currently hosting this meme.
    pub host_count: u32,
    /// Sacredness (0 = mundane, 1 = sacred). Sacred memes resist mutation.
    pub sacredness: Fixed,
    /// Tick when this meme was created.
    pub created_tick: u64,
    /// Whether this meme is currently active (can be suppressed).
    pub active: bool,
}

impl Meme {
    /// Create a new meme.
    pub fn new(
        id: usize,
        description: String,
        content_type: MemeContent,
        emotional_charge: Fixed,
        identity_relevance: Fixed,
        tick: u64,
    ) -> Self {
        Self {
            id,
            description,
            content_type,
            emotional_charge,
            identity_relevance,
            moral_charge: Fixed::ZERO,
            credibility: Fixed::from_f64(0.5),
            novelty: Fixed::ONE,
            virality: (emotional_charge + identity_relevance) * Fixed::from_f64(0.5),
            mutation_rate: Fixed::from_f64(0.1),
            host_count: 0,
            sacredness: Fixed::ZERO,
            created_tick: tick,
            active: true,
        }
    }

    /// Compute transmission probability from source to listener.
    ///
    /// ```text
    /// chance = credibility × emotional_charge × identity_relevance
    ///        × novelty × virality × susceptibility
    ///        × (1 - skepticism) × (1 - suppression)
    /// ```
    pub fn transmission_chance(
        &self,
        source_trust: Fixed,
        listener_susceptibility: Fixed,
        skepticism: Fixed,
    ) -> Fixed {
        let base = self.credibility * source_trust
            * self.emotional_charge
            * self.identity_relevance
            * self.novelty
            * self.virality
            * listener_susceptibility;
        (base * (Fixed::ONE - skepticism)).clamp_01()
    }

    /// Should this meme mutate during transmission?
    ///
    /// Sacred memes are resistant to mutation.
    pub fn should_mutate(&self, rng_val: Fixed) -> bool {
        let effective_rate = self.mutation_rate * (Fixed::ONE - self.sacredness);
        rng_val < effective_rate
    }

    /// Decay novelty each tick (memes become less novel over time).
    pub fn tick_decay(&mut self) {
        self.novelty = (self.novelty * Fixed::from_f64(0.998)).max(Fixed::ZERO);
        // Host count slowly decays (memes fade if not reinforced)
        if self.host_count > 0 {
            self.host_count = self.host_count.saturating_sub(1);
        }
    }
}

/// Registry of all memes in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemeRegistry {
    /// All memes, indexed by id.
    pub memes: Vec<Meme>,
    /// Next available meme id.
    next_id: usize,
}

impl Default for MemeRegistry {
    fn default() -> Self {
        Self {
            memes: Vec::new(),
            next_id: 0,
        }
    }
}

impl MemeRegistry {
    /// Register a new meme and return its id.
    pub fn register(&mut self, meme: Meme) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut meme = meme;
        meme.id = id;
        self.memes.push(meme);
        id
    }

    /// Get a meme by id.
    pub fn get(&self, id: usize) -> Option<&Meme> {
        self.memes.iter().find(|m| m.id == id)
    }

    /// Get a mutable reference to a meme by id.
    pub fn get_mut(&mut self, id: usize) -> Option<&mut Meme> {
        self.memes.iter_mut().find(|m| m.id == id)
    }

    /// Tick decay on all active memes.
    pub fn tick_all(&mut self) {
        for meme in &mut self.memes {
            if meme.active {
                meme.tick_decay();
            }
        }
    }

    /// Number of active memes.
    pub fn active_count(&self) -> usize {
        self.memes.iter().filter(|m| m.active).count()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_meme_has_sane_defaults() {
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.3), 0);
        assert_eq!(m.id, 0);
        assert!(m.novelty > Fixed::ZERO);
        assert!(m.active);
    }

    #[test]
    fn transmission_chance_scales_with_trust() {
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.8), Fixed::from_f64(0.6), 0);
        let high_trust = m.transmission_chance(Fixed::from_f64(0.9), Fixed::ONE, Fixed::ZERO);
        let low_trust = m.transmission_chance(Fixed::from_f64(0.2), Fixed::ONE, Fixed::ZERO);
        assert!(high_trust > low_trust);
    }

    #[test]
    fn skepticism_reduces_transmission() {
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.8), Fixed::from_f64(0.6), 0);
        let no_skepticism = m.transmission_chance(Fixed::from_f64(0.5), Fixed::ONE, Fixed::ZERO);
        let high_skepticism = m.transmission_chance(Fixed::from_f64(0.5), Fixed::ONE, Fixed::from_f64(0.8));
        assert!(no_skepticism > high_skepticism);
    }

    #[test]
    fn sacred_meme_resists_mutation() {
        let mut sacred = Meme::new(0, "sacred".into(), MemeContent::Theological, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0);
        sacred.sacredness = Fixed::ONE;
        sacred.mutation_rate = Fixed::ONE;
        // Even with max mutation rate, sacred memes resist
        assert!(!sacred.should_mutate(Fixed::from_f64(0.5)));
    }

    #[test]
    fn novelty_decays() {
        let mut m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0);
        let before = m.novelty;
        m.tick_decay();
        assert!(m.novelty < before);
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = MemeRegistry::default();
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0);
        let id = reg.register(m);
        assert_eq!(id, 0);
        assert!(reg.get(0).is_some());
        assert!(reg.get(1).is_none());
    }

    #[test]
    fn registry_active_count() {
        let mut reg = MemeRegistry::default();
        let m1 = Meme::new(0, "a".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0);
        let mut m2 = Meme::new(0, "b".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0);
        m2.active = false;
        reg.register(m1);
        reg.register(m2);
        assert_eq!(reg.active_count(), 1);
    }
}
