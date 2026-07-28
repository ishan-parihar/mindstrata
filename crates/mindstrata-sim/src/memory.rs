//! Memory system — selective, lossy, emotionally biased memory.
//!
//! Agents do not have perfect recall.  Memories are:
//! - **selective**: only salient events are encoded
//! - **lossy**: memories decay over time
//! - **emotionally biased**: emotionally intense events are remembered longer
//! - **rehearsal-reinforced**: recalled memories become stronger
//! - **capacity-limited**: old memories are evicted when capacity is reached

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// Types of episodic memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MemoryKind {
    /// Something the agent ate or drank.
    Consumption,
    /// A social interaction with another agent.
    Social,
    /// A positive event (help received, achievement).
    Positive,
    /// A negative event (loss, insult, betrayal).
    Negative,
}

/// A single episodic memory trace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    /// What kind of event this memory records.
    pub kind: MemoryKind,
    /// Tick when the event occurred.
    pub tick: u64,
    /// How salient the event was at encoding (0..1).
    pub salience: Fixed,
    /// Emotional intensity at encoding (0..1).
    pub emotional_intensity: Fixed,
    /// Number of times this memory has been recalled.
    pub rehearsal_count: u32,
    /// Current strength/availability (decays over time).
    pub strength: Fixed,
    /// Optional: agent involved (for social memories).
    pub other_agent: Option<u32>,
    /// Brief description tag.
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
}

/// The memory store for an agent.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryStore {
    /// Episodic memories, ordered by tick (oldest first).
    pub episodes: Vec<Memory>,
    /// Maximum number of episodic memories.
    pub capacity: usize,
    /// Base decay rate per tick.
    pub decay_rate: Fixed,
    /// Threshold below which memories are evicted.
    pub eviction_threshold: Fixed,
}

impl Default for MemoryStore {
    fn default() -> Self {
        Self {
            episodes: Vec::new(),
            capacity: 200,
            decay_rate: Fixed::from_f64(0.0005),
            eviction_threshold: Fixed::from_f64(0.05),
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
    pub fn encode(
        &mut self,
        kind: MemoryKind,
        tick: u64,
        salience: Fixed,
        emotional_intensity: Fixed,
        other_agent: Option<u32>,
        tag: MemoryTag,
    ) {
        // Note: salience threshold is now handled by AttentionState in sim.rs
        // (threshold 0.2). This method trusts that the caller has already filtered.

        let memory = Memory {
            kind,
            tick,
            salience,
            emotional_intensity,
            rehearsal_count: 0,
            strength: Fixed::from_f64(0.8),
            other_agent,
            tag,
        };

        self.episodes.push(memory);

        // Evict weakest if over capacity
        if self.episodes.len() > self.capacity {
            self.evict_weakest();
        }
    }

    /// Decay all memories and remove those below the eviction threshold.
    pub fn decay(&mut self, current_tick: u64) {
        let age_weight = Fixed::from_f64(0.0003);

        for mem in self.episodes.iter_mut() {
            let age = current_tick.saturating_sub(mem.tick) as i64;
            let age_factor = Fixed::from_int(age) * age_weight;
            let rehearsal_bonus = Fixed::from_f64(mem.rehearsal_count as f64 * 0.02);
            let emotional_bonus = mem.emotional_intensity * Fixed::from_f64(0.1);

            let decay = self.decay_rate + age_factor - rehearsal_bonus - emotional_bonus;
            let decay = decay.max(Fixed::from_f64(0.0001));
            mem.strength = (mem.strength - decay).clamp_01();
        }

        // Remove memories below threshold
        self.episodes.retain(|m| m.strength >= self.eviction_threshold);
    }

    /// Recall a random memory for rehearsal (strengthens it).
    pub fn rehearse_random(&mut self, rng: &mut impl rand::Rng) -> bool {
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
        for mem in self.episodes.iter_mut() {
            cumulative += mem.strength.to_f64();
            if cumulative >= roll {
                mem.rehearsal_count += 1;
                mem.strength = (mem.strength + Fixed::from_f64(0.05)).clamp_01();
                return true;
            }
        }

        false
    }

    /// Count of current memories.
    pub fn count(&self) -> usize {
        self.episodes.len()
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
        store.encode(MemoryKind::Positive, 10, Fixed::from_f64(0.8), Fixed::from_f64(0.6), None, MemoryTag::HelpedBy);

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
        store.encode(MemoryKind::Positive, 0, Fixed::from_f64(0.9), Fixed::from_f64(0.8), None, MemoryTag::HelpedBy);
        let before = store.episodes[0].strength;
        let mut rng = rand::rngs::StdRng::seed_from_u64(42);
        store.rehearse_random(&mut rng);
        let after = store.episodes[0].strength;
        assert!(after > before);
    }
}
