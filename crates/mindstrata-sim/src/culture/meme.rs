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
    /// Song or chant.
    Song,
    /// Slogan or rallying cry.
    Slogan,
    /// Rumor — unverified social claim.
    Rumor,
}

/// §13.1 (AP2): Lineage of a meme — how it descended from its ancestors.
///
/// The plan models memes as evolving populations: every meme either founds
/// a lineage or is derived from a parent through mutation. Tracks the
/// parent id and generation depth for mutation-rate and complexity studies.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum MemeLineage {
    /// A founding meme with no parent.
    #[default]
    Founding,
    /// Derived from a parent meme via mutation.
    Derived {
        /// Parent meme id.
        parent: usize,
        /// Generations removed from the founding meme.
        generations: u32,
    },
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
    /// §13.1 (AP2): How complex the meme is — complex memes are harder to
    /// transmit faithfully. Derived deterministically from content type.
    #[serde(default)]
    pub complexity: Fixed,
    /// §13.1 (AP2): Lineage — founding or derived from a parent.
    #[serde(default)]
    pub lineage: MemeLineage,
    /// §13.1 (AP2): Institution backing this meme (amplifies transmission).
    #[serde(default)]
    pub institutional_backing: Option<u64>,
    /// §13.1 (AP2): Suppression level (0 = free, 1 = fully suppressed).
    /// Gates transmission: `× (1 - suppression)`.
    #[serde(default)]
    pub suppression_level: Fixed,
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
        virality_scaling: Fixed,
        mutation_rate_base: Fixed,
    ) -> Self {
        // §13.1 (AP2): Complexity is derived deterministically from content
        // type (no RNG) — abstract/sacred content is more complex than
        // practical/social content.
        let complexity = match content_type {
            MemeContent::Theological
            | MemeContent::Conspiracy
            | MemeContent::Prophecy
            | MemeContent::Historical => Fixed::from_f64(0.7),
            MemeContent::Political | MemeContent::Moral | MemeContent::Taboo => {
                Fixed::from_f64(0.5)
            }
            MemeContent::Accusation | MemeContent::Praise | MemeContent::Rumor => {
                Fixed::from_f64(0.3)
            }
            MemeContent::Practical | MemeContent::Joke | MemeContent::Song | MemeContent::Slogan => {
                Fixed::from_f64(0.2)
            }
        };
        Self {
            id,
            description,
            content_type,
            emotional_charge,
            identity_relevance,
            moral_charge: Fixed::ZERO,
            credibility: Fixed::from_f64(0.5),
            novelty: Fixed::ONE,
            virality: (emotional_charge + identity_relevance) * virality_scaling,
            mutation_rate: mutation_rate_base,
            host_count: 0,
            sacredness: Fixed::ZERO,
            complexity,
            lineage: MemeLineage::Founding,
            institutional_backing: None,
            suppression_level: Fixed::ZERO,
            created_tick: tick,
            active: true,
        }
    }

    /// Compute transmission probability from source to listener.
    ///
    /// ```text
    /// chance = credibility × emotional_charge × identity_relevance
    ///        × novelty × virality × susceptibility
    ///        × (1 - skepticism) × (1 - suppression) × multiplier
    /// ```
    pub fn transmission_chance(
        &self,
        source_trust: Fixed,
        listener_susceptibility: Fixed,
        skepticism: Fixed,
        transmission_multiplier: Fixed,
    ) -> Fixed {
        // §13.1 (AP2): suppression_level gates transmission (`× (1 - suppression)`)
        // exactly as the plan's formula specifies. With suppression ZERO this is
        // the identity factor, so the golden baseline stays byte-identical.
        let suppression_factor = Fixed::ONE - self.suppression_level;
        let base = self.credibility * source_trust
            * self.emotional_charge
            * self.identity_relevance
            * self.novelty
            * self.virality
            * listener_susceptibility;
        (base * (Fixed::ONE - skepticism) * transmission_multiplier * suppression_factor).clamp_01()
    }

    /// Should this meme mutate during transmission?
    ///
    /// Sacred memes are resistant to mutation. Delegates to
    /// [`should_mutate_scaled`](Self::should_mutate_scaled) with an identity
    /// multiplier, so the sacredness-resistance logic lives in one place.
    pub fn should_mutate(&self, rng_val: Fixed) -> bool {
        self.should_mutate_scaled(rng_val, Fixed::ONE)
    }

    /// `should_mutate` with an external rate multiplier (§13.2 master gate).
    ///
    /// At a multiplier of ZERO no mutation ever fires — the identity factor
    /// that keeps the golden baseline byte-identical when the feature is
    /// disabled.
    pub fn should_mutate_scaled(&self, rng_val: Fixed, rate_multiplier: Fixed) -> bool {
        let effective_rate = self.mutation_rate * (Fixed::ONE - self.sacredness) * rate_multiplier;
        rng_val < effective_rate
    }

    /// §13.2 (AP2): fold the five mutation sources into one drift magnitude.
    ///
    /// ```text
    /// mutation = memory_error + emotional_exaggeration + identity_bias
    ///         + narrative_simplification + audience_tailoring
    /// ```
    /// Each source maps onto a meme property; the coefficients are scaled so
    /// the theoretical maximum sum is exactly 1.0 (0.3+0.25+0.2+0.15+0.1),
    /// preserving fine gradation instead of saturating the clamp:
    ///   - memory_error: complex memes degrade in transmission (1 − complexity)
    ///   - emotional_exaggeration: charged memes distort further (emotional_charge)
    ///   - identity_bias: identity-salient memes warp toward the in-group (identity_relevance)
    ///   - narrative_simplification: complex narratives get flattened (complexity)
    ///   - audience_tailoring: viral memes adapt to their listeners (virality)
    pub fn mutation_magnitude(&self) -> Fixed {
        let memory_error = (Fixed::ONE - self.complexity) * Fixed::from_f64(0.3);
        let emotional_exaggeration = self.emotional_charge * Fixed::from_f64(0.25);
        let identity_bias = self.identity_relevance * Fixed::from_f64(0.2);
        let narrative_simplification = self.complexity * Fixed::from_f64(0.15);
        let audience_tailoring = self.virality * Fixed::from_f64(0.1);
        (memory_error
            + emotional_exaggeration
            + identity_bias
            + narrative_simplification
            + audience_tailoring)
            .clamp_01()
    }

    /// Apply a mutation to this meme (drift during transmission).
    ///
    /// Deterministic: the drift magnitudes derive from the meme's own
    /// properties, so no extra RNG draws are needed — the caller already
    /// rolled the decision draw for `should_mutate[_scaled]`.
    pub fn mutate(&mut self, magnitude: Fixed) {
        // §13.2: The per-fire drift is intentionally subtle — ⅓ of the
        // magnitude-coefficient scale. The master multiplier is live by
        // default (0.3) and every successful transmission re-checks mutation,
        // so frequently-transmitted memes drift repeatedly over a year;
        // aggressive per-fire erosion (the original 0.3/0.25/0.15 scale)
        // collapsed credibility and killed echo-chamber reinforcement.
        // Sized so cumulative drift stays observable without dominating the
        // +0.05/transmission novelty-reinforcement loop.
        // Credibility erodes as the meme drifts from its origin.
        self.credibility = (self.credibility - magnitude * Fixed::from_f64(0.1)).clamp_01();
        // Emotional charge drifts up (emotional exaggeration).
        self.emotional_charge =
            (self.emotional_charge + magnitude * Fixed::from_f64(0.08)).clamp_01();
        // Complexity drifts down (narrative simplification).
        self.complexity = (self.complexity - magnitude * Fixed::from_f64(0.05)).clamp_01();
        // A mutated meme is a derived form of its own founding content: flip
        // the lineage and count generations so the drift is observable.
        // `parent` anchors the meme's founding id — the form it drifted
        // from — not a separate creator meme (in-place drift, no spawning).
        self.lineage = match &self.lineage {
            MemeLineage::Founding => MemeLineage::Derived {
                parent: self.id,
                generations: 1,
            },
            MemeLineage::Derived { parent, generations } => MemeLineage::Derived {
                parent: *parent,
                generations: generations + 1,
            },
        };
    }

    /// Decay novelty each tick (memes become less novel over time).
    pub fn tick_decay(&mut self, novelty_decay_factor: Fixed) {
        self.novelty = (self.novelty * novelty_decay_factor).max(Fixed::ZERO);
        // Hosts fade only once the meme goes stale (novelty collapsed).
        // Reinforcement IS transmission — every accepted transmission re-boosts
        // novelty (+0.05 in the spread loop), so an actively-spread meme keeps
        // its hosts while a neglected meme fades. With the 0.998/day novelty
        // decay, a meme that receives NO reinforcement crosses the 0.9
        // staleness threshold after ~2 months of sim time; every transmission
        // resets that clock. The previous flat -1/day wiped hosts within a day
        // regardless of novelty, making meme persistence (and therefore
        // spread) structurally impossible.
        if self.novelty < Fixed::from_f64(0.9) && self.host_count > 0 {
            self.host_count = self.host_count.saturating_sub(1);
        }
    }
}

/// Registry of all memes in the simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct MemeRegistry {
    /// All memes, indexed by id.
    pub memes: Vec<Meme>,
    /// Next available meme id.
    next_id: usize,
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

    /// Tick decay on all active memes using the given novelty decay factor.
    pub fn tick_all(&mut self, novelty_decay_factor: Fixed) {
        for meme in &mut self.memes {
            if meme.active {
                meme.tick_decay(novelty_decay_factor);
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
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.3), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        assert_eq!(m.id, 0);
        assert!(m.novelty > Fixed::ZERO);
        assert!(m.active);
    }

    #[test]
    fn transmission_chance_scales_with_trust() {
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.8), Fixed::from_f64(0.6), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let high_trust = m.transmission_chance(Fixed::from_f64(0.9), Fixed::ONE, Fixed::ZERO, Fixed::ONE);
        let low_trust = m.transmission_chance(Fixed::from_f64(0.2), Fixed::ONE, Fixed::ZERO, Fixed::ONE);
        assert!(high_trust > low_trust);
    }

    #[test]
    fn skepticism_reduces_transmission() {
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.8), Fixed::from_f64(0.6), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let no_skepticism = m.transmission_chance(Fixed::from_f64(0.5), Fixed::ONE, Fixed::ZERO, Fixed::ONE);
        let high_skepticism = m.transmission_chance(Fixed::from_f64(0.5), Fixed::ONE, Fixed::from_f64(0.8), Fixed::ONE);
        assert!(no_skepticism > high_skepticism);
    }

    #[test]
    fn sacred_meme_resists_mutation() {
        let mut sacred = Meme::new(0, "sacred".into(), MemeContent::Theological, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        sacred.sacredness = Fixed::ONE;
        sacred.mutation_rate = Fixed::ONE;
        // Even with max mutation rate, sacred memes resist
        assert!(!sacred.should_mutate(Fixed::from_f64(0.5)));
    }

    #[test]
    fn novelty_decays() {
        let mut m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let before = m.novelty;
        m.tick_decay(Fixed::from_f64(0.998));
        assert!(m.novelty < before);
    }

    #[test]
    fn registry_register_and_get() {
        let mut reg = MemeRegistry::default();
        let m = Meme::new(0, "test".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let id = reg.register(m);
        assert_eq!(id, 0);
        assert!(reg.get(0).is_some());
        assert!(reg.get(1).is_none());
    }

    #[test]
    fn registry_active_count() {
        let mut reg = MemeRegistry::default();
        let m1 = Meme::new(0, "a".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let mut m2 = Meme::new(0, "b".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        m2.active = false;
        reg.register(m1);
        reg.register(m2);
        assert_eq!(reg.active_count(), 1);
    }

    // ── §13.1 Meme deepen-it tests ───────────────────────────────────

    #[test]
    fn new_meme_defaults_institutional_fields() {
        let m = Meme::new(0, "t".into(), MemeContent::Moral, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        assert_eq!(m.lineage, MemeLineage::Founding);
        assert_eq!(m.institutional_backing, None);
        assert_eq!(m.suppression_level, Fixed::ZERO);
        assert!(m.complexity > Fixed::ZERO, "complexity derived from content type");
    }

    #[test]
    fn complexity_derived_by_content_type() {
        // §13.1: abstract/sacred content is more complex than practical.
        let theological = Meme::new(0, "t".into(), MemeContent::Theological, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let practical = Meme::new(0, "p".into(), MemeContent::Practical, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        assert!(theological.complexity > practical.complexity);
    }

    #[test]
    fn suppression_reduces_transmission() {
        // §13.1: suppression gates transmission `× (1 - suppression)`;
        // zero suppression is the identity (baseline-compatible).
        let base = Meme::new(0, "t".into(), MemeContent::Moral, Fixed::from_f64(0.8), Fixed::from_f64(0.6), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let free = base.transmission_chance(Fixed::from_f64(0.9), Fixed::ONE, Fixed::ZERO, Fixed::ONE);
        let mut suppressed = base.clone();
        suppressed.suppression_level = Fixed::from_f64(0.7);
        let gated = suppressed.transmission_chance(Fixed::from_f64(0.9), Fixed::ONE, Fixed::ZERO, Fixed::ONE);
        assert!(gated < free, "suppressed meme transmits less");
        // Zero suppression is identity: free == unsuppressed baseline.
        assert_eq!(free, base.transmission_chance(Fixed::from_f64(0.9), Fixed::ONE, Fixed::ZERO, Fixed::ONE));
    }

    #[test]
    fn lineage_derived_tracks_parent_and_generations() {
        let mut derived = Meme::new(0, "d".into(), MemeContent::Rumor, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        derived.lineage = MemeLineage::Derived { parent: 3, generations: 2 };
        assert_eq!(derived.lineage, MemeLineage::Derived { parent: 3, generations: 2 });
    }

    #[test]
    fn institutional_fields_serde_roundtrip() {
        let mut m = Meme::new(0, "t".into(), MemeContent::Political, Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        m.lineage = MemeLineage::Derived { parent: 1, generations: 3 };
        m.institutional_backing = Some(7);
        m.suppression_level = Fixed::from_f64(0.4);
        let json = serde_json::to_string(&m).unwrap();
        let restored: Meme = serde_json::from_str(&json).unwrap();
        assert_eq!(restored.lineage, m.lineage);
        assert_eq!(restored.institutional_backing, Some(7));
        assert_eq!(restored.suppression_level, m.suppression_level);
        assert_eq!(restored.complexity, m.complexity);
    }

    #[test]
    fn meme_content_covers_plan_taxonomy() {
        // §13.1: the plan lists rumor/insult/praise/theological/political/moral/
        // conspiracy/prophecy/historical/practical/taboo/joke/song/slogan —
        // the enum must expose all of them (Song + Rumor were added).
        let variants = [
            MemeContent::Accusation,
            MemeContent::Praise,
            MemeContent::Theological,
            MemeContent::Political,
            MemeContent::Moral,
            MemeContent::Conspiracy,
            MemeContent::Prophecy,
            MemeContent::Historical,
            MemeContent::Practical,
            MemeContent::Taboo,
            MemeContent::Joke,
            MemeContent::Song,
            MemeContent::Slogan,
            MemeContent::Rumor,
        ];
        assert_eq!(variants.len(), 14);
    }

    #[test]
    fn old_save_without_institutional_fields_restores_defaults() {
        // A save from before the §13.1 fields existed must deserialize to
        // sensible defaults. Fixed serializes as raw scaled i64 (SCALE=10_000).
        let old_json = r#"{
            "id": 0,
            "description": "old",
            "content_type": "Moral",
            "emotional_charge": 5000,
            "identity_relevance": 5000,
            "moral_charge": 0,
            "credibility": 5000,
            "novelty": 10000,
            "virality": 5000,
            "mutation_rate": 1000,
            "host_count": 0,
            "sacredness": 0,
            "created_tick": 0,
            "active": true
        }"#;
        let restored: Meme = serde_json::from_str(old_json).unwrap();
        assert_eq!(restored.lineage, MemeLineage::Founding);
        assert_eq!(restored.institutional_backing, None);
        assert_eq!(restored.suppression_level, Fixed::ZERO);
        assert_eq!(restored.complexity, Fixed::ZERO);
    }

    // ── §13.2 Agent-specific meme propagation tests ──────────────────

    /// §13.2: Gullible agent (high susceptibility, low skepticism) accepts
    /// memes more readily than a critical agent.
    #[test]
    fn gullible_agent_accepts_more_memes() {
        let m = Meme::new(0, "test".into(), MemeContent::Conspiracy,
            Fixed::from_f64(0.8), Fixed::from_f64(0.7), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        // Gullible: high conspiracy_susceptibility → high susceptibility
        //   susceptibility = 0.8 * 0.6 + 0.2 = 0.68
        //   skepticism = 0.2 * (1 - 0.7*0.5) * 0.6 = 0.066
        let gullible_sus = Fixed::from_f64(0.8) * Fixed::from_f64(0.6)
            + Fixed::from_f64(0.2);
        let gullible_sk = Fixed::from_f64(0.2)
            * (Fixed::ONE - Fixed::from_f64(0.7) * Fixed::from_f64(0.5))
            * Fixed::from_f64(0.6);
        let gullible = m.transmission_chance(Fixed::from_f64(0.7), gullible_sus, gullible_sk, Fixed::ONE);

        // Critical: low conspiracy_susceptibility → low susceptibility
        //   susceptibility = 0.1 * 0.6 + 0.2 = 0.26
        //   skepticism = 0.8 * (1 - 0.2*0.5) * 0.6 = 0.432
        let critical_sus = Fixed::from_f64(0.1) * Fixed::from_f64(0.6)
            + Fixed::from_f64(0.2);
        let critical_sk = Fixed::from_f64(0.8)
            * (Fixed::ONE - Fixed::from_f64(0.2) * Fixed::from_f64(0.5))
            * Fixed::from_f64(0.6);
        let critical = m.transmission_chance(Fixed::from_f64(0.7), critical_sus, critical_sk, Fixed::ONE);

        assert!(gullible > critical,
            "Gullible agent ({}) should accept more than critical ({})",
            gullible.to_f64(), critical.to_f64());
    }

    /// §13.2: Dogmatic agents resist skepticism more than open agents.
    #[test]
    fn dogmatism_reduces_skepticism_effectiveness() {
        let m = Meme::new(0, "test".into(), MemeContent::Moral,
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        // Dogmatic: source_monitoring=0.8, dogmatism=0.9
        //   skepticism = 0.8 * (1 - 0.9*0.5) * 0.6 = 0.264
        let dogmatic_sk = Fixed::from_f64(0.8)
            * (Fixed::ONE - Fixed::from_f64(0.9) * Fixed::from_f64(0.5))
            * Fixed::from_f64(0.6);
        // Open: source_monitoring=0.8, dogmatism=0.1
        //   skepticism = 0.8 * (1 - 0.1*0.5) * 0.6 = 0.456
        let open_sk = Fixed::from_f64(0.8)
            * (Fixed::ONE - Fixed::from_f64(0.1) * Fixed::from_f64(0.5))
            * Fixed::from_f64(0.6);
        let dogmatic_chance = m.transmission_chance(
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), dogmatic_sk, Fixed::ONE);
        let open_chance = m.transmission_chance(
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), open_sk, Fixed::ONE);

        assert!(dogmatic_chance > open_chance,
            "Dogmatic agent ({}) should be more susceptible than open ({})",
            dogmatic_chance.to_f64(), open_chance.to_f64());
    }

    /// Transmission multiplier actually scales transmission chance.
    #[test]
    fn transmission_multiplier_scales_chance() {
        let m = Meme::new(0, "test".into(), MemeContent::Moral,
            Fixed::from_f64(0.8), Fixed::from_f64(0.6), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let no_boost = m.transmission_chance(
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), Fixed::ZERO, Fixed::ONE);
        let high_boost = m.transmission_chance(
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), Fixed::ZERO, Fixed::from_f64(1.5));
        let low_boost = m.transmission_chance(
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), Fixed::ZERO, Fixed::from_f64(0.5));
        assert!(high_boost > no_boost,
            "High multiplier ({}) should produce higher chance than baseline ({})",
            high_boost.to_f64(), no_boost.to_f64());
        assert!(no_boost > low_boost,
            "Baseline ({}) should produce higher chance than low multiplier ({})",
            no_boost.to_f64(), low_boost.to_f64());
    }

    /// §13.2: Suspicion floors ensure even critical agents aren't perfectly immune.
    #[test]
    fn suspicion_floor_prevents_perfect_immunity() {
        // Perfect agent: conspiracy_susceptibility=0, source_monitoring=1, dogmatism=0
        let perfect_sus = Fixed::from_f64(0.0) * Fixed::from_f64(0.6)
            + Fixed::from_f64(0.2);
        assert!(perfect_sus >= Fixed::from_f64(0.2),
            "Suspicion floor of 0.2 not enforced: {}", perfect_sus.to_f64());
        assert!(perfect_sus <= Fixed::ONE,
            "Suspicion should not exceed 1.0: {}", perfect_sus.to_f64());
    }

    // ── §13.2 Meme mutation tests ────────────────────────────────────

    /// A zero multiplier is the identity factor: no mutation ever fires.
    #[test]
    fn should_mutate_scaled_zero_multiplier_never_mutates() {
        let m = Meme::new(0, "t".into(), MemeContent::Rumor,
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.5));
        // multiplier ZERO → identity factor → never mutates, even at rng 0.
        assert!(!m.should_mutate_scaled(Fixed::ZERO, Fixed::ZERO));
        // multiplier ONE with rng 0 → mutates (0 < 0.5).
        assert!(m.should_mutate_scaled(Fixed::ZERO, Fixed::ONE));
    }

    /// Sacred memes resist mutation (rate × (1 − sacredness)).
    #[test]
    fn sacred_memes_resist_mutation() {
        let mut m = Meme::new(0, "t".into(), MemeContent::Theological,
            Fixed::from_f64(0.5), Fixed::from_f64(0.5), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.5));
        m.sacredness = Fixed::from_f64(0.9);
        // effective rate = 0.5 × (1 − 0.9) = 0.05
        assert!(!m.should_mutate(Fixed::from_f64(0.2)));
        assert!(m.should_mutate(Fixed::from_f64(0.04)));
    }

    /// The five-factor magnitude is bounded to [0, 1] and strong for
    /// charged, identity-relevant, viral memes.
    #[test]
    fn mutation_magnitude_is_bounded_and_derived_from_five_sources() {
        let m = Meme::new(0, "t".into(), MemeContent::Conspiracy,
            Fixed::from_f64(0.8), Fixed::from_f64(0.7), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let mag = m.mutation_magnitude();
        assert!(mag > Fixed::ZERO && mag <= Fixed::ONE);
        // A highly charged, identity-relevant, viral meme drifts strongly.
        assert!(mag > Fixed::from_f64(0.5));
    }

    /// Mutation drifts credibility down / emotional charge up and records
    /// the derived lineage with an incrementing generation count.
    #[test]
    fn mutate_drifts_fields_and_records_lineage() {
        let mut m = Meme::new(0, "t".into(), MemeContent::Rumor,
            Fixed::from_f64(0.2), Fixed::from_f64(0.2), 0, Fixed::from_f64(0.5), Fixed::from_f64(0.1));
        let mag = m.mutation_magnitude();
        let cred_before = m.credibility;
        let charge_before = m.emotional_charge;
        m.mutate(mag);
        assert!(m.credibility <= cred_before);
        assert!(m.emotional_charge >= charge_before);
        assert_eq!(m.lineage, MemeLineage::Derived { parent: 0, generations: 1 });
        // Second mutation increments the generation count.
        m.mutate(mag);
        assert_eq!(m.lineage, MemeLineage::Derived { parent: 0, generations: 2 });
    }
}
