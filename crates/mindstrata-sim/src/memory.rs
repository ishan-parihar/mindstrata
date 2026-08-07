//! Memory system — multiple memory systems (Architecture-plan-2 §8.1.3).
//!
//! Agents do not have perfect recall.  Memories are:
//! - **selective**: only salient events are encoded
//! - **lossy**: memories decay over time
//! - **emotionally biased**: emotionally intense events are remembered longer
//! - **rehearsal-reinforced**: recalled memories become stronger
//! - **capacity-limited**: old memories are evicted when capacity is reached
//!
//! §8.1.3 (AP2): "Expand current memory into multiple memory systems."
//! The taxonomy now carries the plan's nine [`MemoryKind`]s, and each trace
//! is a [`MemoryTrace`] with the plan's properties — `sensory_richness`,
//! `accuracy`, `valence`, `identity_relevance`, `social_sharedness`,
//! `last_rehearsed`, and a `distortion_history` of [`DistortionEvent`]s.
//!
//! This module is **write-only observational state**: nothing in the sim
//! reads memory for decisions, and memory is excluded from snapshot
//! projections and the golden baseline, so enriching the derivation is
//! drift-free by construction.
//!
//! Kind producers this iteration:
//! - `Somatic` — bodily events (eating, drinking, resting).
//! - `Emotional` — positive emotional interactions (help, comfort).
//! - `Traumatic` — threat/insult events (fear-laden).
//! - `Social` — neutral interactions with other agents.
//! - `Flashbulb` — a *derivation*: any trace whose encoding salience reaches
//!   0.4 AND emotional charge reaches 0.6 is upgraded to Flashbulb (the
//!   "traumatic events flash back" emergent effect). The bar is calibrated to
//!   the live salience envelope: `compute_salience` is a product of four
//!   ≤ 1.0 factors, so it tops out around 0.45 in real runs — the ≥ 0.4
//!   tier is the vivid tail of the percept distribution.
//!
//! `Episodic`, `Semantic`, `Procedural`, and `Cultural` are the plan's
//! remaining taxonomy slots — their producers are the future consumer
//! systems (narrative → Episodic, education → Semantic, skills →
//! Procedural, ritual → Cultural) in separately-calibrated iterations.
//!
//! Distortion mechanics (§8.1.3): emotion distorts (`reconsolidate`
//! erodes `accuracy` and records an `EmotionalReconsolidation` event),
//! retrieval reconstructs (rehearsal erodes `accuracy` slightly and
//! records a `Rehearsal` event), and identity protects (baseline
//! `identity_relevance` is reserved for the identity-protection wiring).

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Identifier for a memory trace (plan: `pub id: MemoryId`).
pub type MemoryId = u64;

/// Maximum distortion events kept per trace — growth guard (see
/// [`MemoryTrace::record_distortion`]).
pub const MAX_DISTORTION_EVENTS: usize = 32;

/// Types of memory — the plan's nine-kind taxonomy (§8.1.3).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum MemoryKind {
    /// A specific past event, personally experienced (producer: narrative).
    Episodic,
    /// General knowledge detached from any single event (producer: education).
    Semantic,
    /// How to do things — skills and habits (producer: skill practice).
    Procedural,
    /// A social interaction with another agent.
    Social,
    /// A strongly valenced emotional event (help, comfort, joy).
    Emotional,
    /// An unusually vivid, high-salience moment (derived at encoding).
    Flashbulb,
    /// A fear-laden threat or insult event (anger/reconsolidation target).
    Traumatic,
    /// A culturally shared memory (producer: ritual, collective memory).
    Cultural,
    /// A bodily/physiological event (eating, drinking, resting).
    Somatic,
}

/// Why a memory's accuracy changed — the plan's `DistortionEvent` payload.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DistortionCause {
    /// Current emotions biased the trace during reconsolidation.
    EmotionalReconsolidation,
    /// Retrieval reconstructed the trace (introducing error).
    Rehearsal,
}

/// A single recorded distortion applied to a memory trace.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct DistortionEvent {
    /// Tick when the distortion occurred.
    pub tick: u64,
    /// What caused the distortion.
    pub cause: DistortionCause,
    /// Signed magnitude of the change in `accuracy`.
    pub delta: Fixed,
}

/// A single memory trace with the plan's §8.1.3 properties.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryTrace {
    /// Unique trace id (monotonic within the store).
    #[serde(default)]
    pub id: MemoryId,
    /// What kind of memory this is.
    pub kind: MemoryKind,
    /// Tick when the event occurred.
    pub tick: u64,
    /// How salient the event was at encoding (0..1).
    pub salience: Fixed,
    /// Emotional charge at encoding (0..1) — plan's `emotional_charge`.
    #[serde(default)]
    pub emotional_charge: Fixed,
    /// How sensorially vivid the trace is (0..1).
    #[serde(default)]
    pub sensory_richness: Fixed,
    /// How faithful the trace is to the original event (0..1, starts at 1).
    #[serde(default)]
    pub accuracy: Fixed,
    /// Valence of the memory (−1..1: negative → positive).
    #[serde(default)]
    pub valence: Fixed,
    /// How much the memory touches the agent's identity (0..1).
    #[serde(default)]
    pub identity_relevance: Fixed,
    /// How much the memory is shared with others (0..1).
    #[serde(default)]
    pub social_sharedness: Fixed,
    /// Tick the trace was last rehearsed.
    #[serde(default)]
    pub last_rehearsed: u64,
    /// Record of every distortion applied to this trace.
    #[serde(default)]
    pub distortion_history: Vec<DistortionEvent>,
    /// Number of times this memory has been recalled.
    pub rehearsal_count: u32,
    /// Current strength/availability (decays over time).
    pub strength: Fixed,
    /// Optional: agent involved (for social memories).
    pub other_agent: Option<u32>,
    /// Brief content discriminator — the plan's `MemoryContent` slot.
    pub tag: MemoryTag,
}

/// Simple tags for memory classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryTag {
    AteFood,
    DrankWater,
    Rested,
    TalkedTo,
    HelpedBy,
    ThreatenedBy,
    InsultedBy,
    GossipedAbout,
    TradedWith,
    /// §8.1.3: Skill practice crossed a proficiency milestone (Procedural).
    SkillMastered,
    /// §8.1.3: Successfully acquired new knowledge (Semantic).
    LearnedKnowledge,
}

impl MemoryTrace {
    /// Record a distortion event, bounded to the most recent
    /// [`MAX_DISTORTION_EVENTS`] (observational-state growth guard — an
    /// angry agent reconsolidates every tick, so an unbounded history would
    /// balloon in long runs; matches the `speech_log` 64-cap pattern).
    fn record_distortion(&mut self, tick: u64, cause: DistortionCause, delta: Fixed) {
        self.distortion_history.push(DistortionEvent { tick, cause, delta });
        if self.distortion_history.len() > MAX_DISTORTION_EVENTS {
            self.distortion_history.remove(0);
        }
    }
}

/// The memory store for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    /// Episodic memories, ordered by tick (oldest first).
    pub episodes: Vec<MemoryTrace>,
    /// Maximum number of episodic memories.
    pub capacity: usize,
    /// Base decay rate per tick.
    pub decay_rate: Fixed,
    /// Threshold below which memories are evicted.
    pub eviction_threshold: Fixed,
    /// Monotonic counter for trace ids.
    #[serde(default)]
    next_id: MemoryId,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            episodes: Vec::new(),
            capacity: 200,
            decay_rate: Fixed::from_f64(0.0005),
            eviction_threshold: Fixed::from_f64(0.05),
            next_id: 0,
        }
    }
}

impl MemoryStore {
    /// Create a new memory store with given capacity.
    pub fn new(capacity: usize) -> Self {
        Self {
            episodes: Vec::with_capacity(capacity.min(500)),
            capacity,
            ..Default::default()
        }
    }

    /// Attempt to encode a new memory.
    ///
    /// The memory is only stored if its salience exceeds a threshold.
    /// If capacity is exceeded, the weakest memory is evicted.
    ///
    /// §8.1.3 derivations (all pure, no RNG):
    /// - `sensory_richness` — half salience, half emotional charge.
    /// - `accuracy` — starts at 1.0; distortion erodes it later.
    /// - `valence` — sign from the kind (negative for `Traumatic`,
    ///   positive for `Emotional`/`Flashbulb`, neutral otherwise).
    /// - `social_sharedness` — 1.0 when the trace involves another agent.
    /// - `identity_relevance` — 0 baseline (identity protection is a
    ///   future, separately-calibrated consumer).
    /// - `Flashbulb` upgrade — salience ≥ 0.4 AND charge ≥ 0.6. Calibrated
    ///   against live probes: salience (a product of four ≤ 1.0 factors)
    ///   tops out ≈ 0.45 in riverford runs, so the ≥ 0.4 tier is the vivid
    ///   tail; charge ≥ 0.6 is common (arousal ≥ 0.83). Both together mark
    ///   genuinely memorable moments.
    pub fn encode(
        &mut self,
        kind: MemoryKind,
        tick: u64,
        salience: Fixed,
        emotional_charge: Fixed,
        other_agent: Option<u32>,
        tag: MemoryTag,
    ) {
        // Note: salience threshold is now handled by AttentionState in sim.rs
        // (threshold 0.2). This method trusts that the caller has already filtered.

        // Valence comes from the BASE kind — a flashbulb derived from a
        // traumatic event stays negatively valenced (the plan's own emergent
        // effect is "traumatic events flash back", not "become pleasant").
        let valence = match kind {
            MemoryKind::Traumatic => Fixed::from_f64(-0.5),
            MemoryKind::Emotional => Fixed::from_f64(0.5),
            _ => Fixed::ZERO,
        };

        // §8.1.3: unusually vivid moments flash back — derive Flashbulb.
        // The vivid-tail threshold is riverford-calibrated (compute_salience
        // tops out ≈ 0.45 there); crisis scenarios shift the envelope, which
        // is desirable (more flashbacks under trauma) but re-checkable.
        let kind = if salience >= Fixed::from_f64(0.4) && emotional_charge >= Fixed::from_f64(0.6) {
            MemoryKind::Flashbulb
        } else {
            kind
        };

        let memory = MemoryTrace {
            id: self.next_id,
            kind,
            tick,
            salience,
            emotional_charge,
            sensory_richness: (salience + emotional_charge) * Fixed::from_f64(0.5),
            accuracy: Fixed::ONE,
            valence,
            identity_relevance: Fixed::ZERO,
            social_sharedness: if other_agent.is_some() { Fixed::ONE } else { Fixed::ZERO },
            last_rehearsed: tick,
            distortion_history: Vec::new(),
            rehearsal_count: 0,
            strength: Fixed::from_f64(0.8),
            other_agent,
            tag,
        };
        self.next_id += 1;

        self.episodes.push(memory);

        // Evict weakest if over capacity
        if self.episodes.len() > self.capacity {
            self.evict_weakest();
        }
    }

    /// Decay all memories and remove those below the eviction threshold.
    pub fn decay(&mut self, current_tick: u64) {
        let age_weight = Fixed::from_f64(0.0003);

        for mem in &mut self.episodes {
            let age = current_tick.saturating_sub(mem.tick) as i64;
            let age_factor = Fixed::from_int(age) * age_weight;
            let rehearsal_bonus = Fixed::from_f64(mem.rehearsal_count as f64 * 0.02);
            let emotional_bonus = mem.emotional_charge * Fixed::from_f64(0.1);

            let decay = self.decay_rate + age_factor - rehearsal_bonus - emotional_bonus;
            let decay = decay.max(Fixed::from_f64(0.0001));
            mem.strength = (mem.strength - decay).clamp_01();
        }

        // Remove memories below threshold
        self.episodes.retain(|m| m.strength >= self.eviction_threshold);
    }

    /// Recall a random memory for rehearsal (strengthens it).
    ///
    /// §8.1.3: retrieval reconstructs — rehearsal nudges `accuracy` down
    /// slightly and records a `Rehearsal` distortion event, while updating
    /// `last_rehearsed`.
    pub fn rehearse_random(&mut self, current_tick: u64, rng: &mut impl rand::Rng) -> bool {
        if self.episodes.is_empty() {
            return false;
        }

        // Weighted random selection: stronger memories are more likely to be recalled
        let total_strength: f64 = self.episodes.iter().map(|m| m.strength.to_f64()).sum();
        if total_strength <= 0.0 {
            return false;
        }

        let roll: f64 = rng.random_range(0.0..total_strength);
        let mut cumulative = 0.0;
        for mem in &mut self.episodes {
            cumulative += mem.strength.to_f64();
            if cumulative >= roll {
                mem.rehearsal_count += 1;
                mem.last_rehearsed = current_tick;
                mem.strength = (mem.strength + Fixed::from_f64(0.05)).clamp_01();
                // Retrieval reconstructs — tiny accuracy erosion.
                let delta = Fixed::from_f64(-0.002);
                mem.accuracy = (mem.accuracy + delta).clamp(Fixed::from_f64(0.01), Fixed::ONE);
                mem.record_distortion(current_tick, DistortionCause::Rehearsal, delta);
                return true;
            }
        }

        false
    }

    /// Count of current memories.
    pub fn count(&self) -> usize {
        self.episodes.len()
    }

    /// §22 / §8.1.3: Memory reconsolidation — recall biases memories toward
    /// current emotional state. Angry agents remember events as more
    /// negative; joyful agents remember more positively.
    ///
    /// §8.1.3: emotion distorts — each biased trace records an
    /// `EmotionalReconsolidation` distortion event and loses `accuracy`.
    ///
    /// Biases key on trace VALENCE, not kind, so a flashbulb derived from a
    /// traumatic event (still negatively valenced) is anger-amplified like
    /// its base kind — while neutral traces (Somatic, Social, Cultural,
    /// Episodic, Semantic, Procedural) are untouched.
    pub fn reconsolidate(&mut self, current_tick: u64, anger: Fixed, joy: Fixed) {
        let bias_strength = Fixed::from_f64(0.003); // subtle per-tick reconsolidation
        for mem in &mut self.episodes {
            let amplifier = match mem.valence {
                v if v < Fixed::ZERO => anger * bias_strength,
                v if v > Fixed::ZERO => joy * bias_strength,
                _ => Fixed::ZERO,
            };
            if amplifier > Fixed::ZERO {
                mem.emotional_charge = (mem.emotional_charge + amplifier).clamp_01();
                mem.strength = (mem.strength + amplifier * Fixed::from_f64(0.5)).clamp_01();
                mem.accuracy = (mem.accuracy - amplifier).clamp(Fixed::from_f64(0.01), Fixed::ONE);
                mem.record_distortion(current_tick, DistortionCause::EmotionalReconsolidation, -amplifier);
            }
        }
    }

    fn evict_weakest(&mut self) {
        if let Some(min_idx) = self
            .episodes
            .iter()
            .enumerate()
            .min_by(|a, b| a.1.strength.partial_cmp(&b.1.strength).unwrap_or(std::cmp::Ordering::Equal))
            .map(|(i, _)| i)
        {
            self.episodes.swap_remove(min_idx);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn encode_and_decay() {
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Emotional, 10, Fixed::from_f64(0.8), Fixed::from_f64(0.6), None, MemoryTag::HelpedBy);

        assert_eq!(store.count(), 1);

        // Decay a lot
        for _ in 0..5000 {
            store.decay(1000);
        }

        // Memory should eventually be evicted
        assert_eq!(store.count(), 0);
    }

    #[test]
    fn low_salience_stored_but_likely_evicted() {
        // After removing the internal threshold, encode() stores any salience.
        // Low-salience memories are stored but will be evicted quickly by decay.
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Social, 10, Fixed::from_f64(0.05), Fixed::ZERO, None, MemoryTag::TalkedTo);
        assert_eq!(store.count(), 1, "Low-salience memory should be stored (threshold moved to attention system)");
        // After heavy decay, it should be evicted
        for _ in 0..5000 {
            store.decay(1000);
        }
        assert_eq!(store.count(), 0, "Low-salience memory should be evicted after decay");
    }

    #[test]
    fn capacity_eviction() {
        let mut store = MemoryStore::new(5);
        for i in 0..10 {
            store.encode(MemoryKind::Social, i, Fixed::from_f64(0.5), Fixed::ZERO, None, MemoryTag::TalkedTo);
        }
        assert!(store.count() <= 5);
    }

    #[test]
    fn rehearsal_strengthens_memory() {
        use rand::SeedableRng;
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Emotional, 0, Fixed::from_f64(0.9), Fixed::from_f64(0.8), None, MemoryTag::HelpedBy);
        let before = store.episodes[0].strength;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        store.rehearse_random(5, &mut rng);
        let after = store.episodes[0].strength;
        assert!(after > before);
    }

    // ── §8.1.3 taxonomy + trace-property tests ─────────────────────────

    #[test]
    fn trace_ids_are_monotonic() {
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Somatic, 0, Fixed::from_f64(0.5), Fixed::ZERO, None, MemoryTag::AteFood);
        store.encode(MemoryKind::Somatic, 1, Fixed::from_f64(0.5), Fixed::ZERO, None, MemoryTag::DrankWater);
        assert_eq!(store.episodes[0].id, 0);
        assert_eq!(store.episodes[1].id, 1);
    }

    #[test]
    fn encode_derives_trace_properties() {
        let mut store = MemoryStore::new(100);
        // Social trace with another agent involved.
        store.encode(MemoryKind::Social, 3, Fixed::from_f64(0.6), Fixed::from_f64(0.4), Some(7), MemoryTag::TalkedTo);
        let trace = &store.episodes[0];

        assert_eq!(trace.accuracy, Fixed::ONE, "accuracy starts at 1.0");
        assert_eq!(trace.valence, Fixed::ZERO, "social traces are valence-neutral");
        assert_eq!(trace.social_sharedness, Fixed::ONE, "shared when another agent is involved");
        assert_eq!(trace.identity_relevance, Fixed::ZERO, "identity relevance is a 0 baseline");
        assert_eq!(trace.last_rehearsed, 3, "freshly encoded traces were last rehearsed at encode tick");
        assert!(trace.sensory_richness > Fixed::ZERO, "salient+emotional traces are vivid");
        assert!(trace.distortion_history.is_empty(), "fresh traces have no distortion history");
        assert_eq!(trace.rehearsal_count, 0);
    }

    #[test]
    fn valence_sign_follows_kind() {
        let mut store = MemoryStore::new(100);
        // Charge 0.5 < 0.6 so the flashbulb upgrade does not hijack the kind.
        store.encode(MemoryKind::Traumatic, 0, Fixed::from_f64(0.9), Fixed::from_f64(0.5), None, MemoryTag::ThreatenedBy);
        assert!(store.episodes[0].valence < Fixed::ZERO, "traumatic traces are negatively valenced");
        store.encode(MemoryKind::Emotional, 1, Fixed::from_f64(0.9), Fixed::from_f64(0.5), None, MemoryTag::HelpedBy);
        assert!(store.episodes[1].valence > Fixed::ZERO, "emotional traces are positively valenced");
        // A flashbulb derived from a traumatic event KEEPS the base kind's
        // negative valence ("traumatic events flash back" — not "become
        // pleasant").
        store.encode(MemoryKind::Traumatic, 2, Fixed::from_f64(0.45), Fixed::from_f64(0.7), None, MemoryTag::ThreatenedBy);
        assert_eq!(store.episodes[2].kind, MemoryKind::Flashbulb);
        assert!(store.episodes[2].valence < Fixed::ZERO, "traumatic flashbulbs stay negatively valenced");
        // A flashbulb derived from an emotional event is positively valenced.
        store.encode(MemoryKind::Emotional, 3, Fixed::from_f64(0.45), Fixed::from_f64(0.7), None, MemoryTag::HelpedBy);
        assert_eq!(store.episodes[3].kind, MemoryKind::Flashbulb);
        assert!(store.episodes[3].valence > Fixed::ZERO, "emotional flashbulbs stay positively valenced");
    }

    #[test]
    fn flashbulb_upgrade_on_vivid_encoding() {
        let mut store = MemoryStore::new(100);
        // Salience ≥ 0.4 AND charge ≥ 0.6 → Flashbulb.
        store.encode(MemoryKind::Traumatic, 0, Fixed::from_f64(0.45), Fixed::from_f64(0.8), None, MemoryTag::ThreatenedBy);
        assert_eq!(store.episodes[0].kind, MemoryKind::Flashbulb);
        // Below the salience bar stays its base kind.
        store.encode(MemoryKind::Traumatic, 1, Fixed::from_f64(0.3), Fixed::from_f64(0.8), None, MemoryTag::ThreatenedBy);
        assert_eq!(store.episodes[1].kind, MemoryKind::Traumatic);
        // Below the charge bar stays its base kind.
        store.encode(MemoryKind::Traumatic, 2, Fixed::from_f64(0.45), Fixed::from_f64(0.5), None, MemoryTag::ThreatenedBy);
        assert_eq!(store.episodes[2].kind, MemoryKind::Traumatic);
    }

    #[test]
    fn reconsolidation_records_distortion_and_erodes_accuracy() {
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Traumatic, 0, Fixed::from_f64(0.9), Fixed::from_f64(0.5), None, MemoryTag::ThreatenedBy);
        store.reconsolidate(50, Fixed::from_f64(0.8), Fixed::ZERO);
        let trace = &store.episodes[0];
        assert_eq!(trace.distortion_history.len(), 1);
        assert_eq!(trace.distortion_history[0].tick, 50);
        assert_eq!(trace.distortion_history[0].cause, DistortionCause::EmotionalReconsolidation);
        assert!(trace.accuracy < Fixed::ONE, "distortion erodes accuracy");
        // The amplified memory got more negative.
        assert!(trace.valence <= Fixed::ZERO);
    }

    #[test]
    fn reconsolidation_biases_traumatic_flashbulb_via_valence() {
        // A flashbulb derived from a traumatic event keeps negative valence,
        // so anger reconsolidation must still amplify it (valence-keyed, not
        // kind-keyed).
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Traumatic, 0, Fixed::from_f64(0.45), Fixed::from_f64(0.7), None, MemoryTag::ThreatenedBy);
        assert_eq!(store.episodes[0].kind, MemoryKind::Flashbulb);
        let charge_before = store.episodes[0].emotional_charge;
        store.reconsolidate(10, Fixed::from_f64(0.9), Fixed::ZERO);
        assert!(
            store.episodes[0].emotional_charge > charge_before,
            "angry reconsolidation amplifies a negative flashbulb"
        );
        assert_eq!(store.episodes[0].distortion_history.len(), 1);
    }

    #[test]
    fn distortion_history_is_bounded() {
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Traumatic, 0, Fixed::from_f64(0.9), Fixed::from_f64(0.5), None, MemoryTag::ThreatenedBy);
        for tick in 0..100u64 {
            store.reconsolidate(tick, Fixed::ONE, Fixed::ZERO);
        }
        assert_eq!(
            store.episodes[0].distortion_history.len(),
            MAX_DISTORTION_EVENTS,
            "history capped to the most recent events"
        );
        assert_eq!(
            store.episodes[0].distortion_history[0].tick,
            100 - MAX_DISTORTION_EVENTS as u64,
            "oldest events evicted first"
        );
    }

    #[test]
    fn rehearsal_records_event_and_updates_last_rehearsed() {
        use rand::SeedableRng;
        let mut store = MemoryStore::new(100);
        store.encode(MemoryKind::Social, 0, Fixed::from_f64(0.9), Fixed::ZERO, None, MemoryTag::TalkedTo);
        let mut rng = rand::rngs::StdRng::seed_from_u64(7);
        store.rehearse_random(123, &mut rng);
        let trace = &store.episodes[0];
        assert_eq!(trace.last_rehearsed, 123, "rehearsal updates last_rehearsed");
        assert_eq!(trace.distortion_history.len(), 1);
        assert_eq!(trace.distortion_history[0].cause, DistortionCause::Rehearsal);
        assert!(trace.accuracy < Fixed::ONE, "retrieval reconstructs — slight accuracy erosion");
    }
}
