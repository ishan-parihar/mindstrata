//! Collective memory — group-level shared memories (§13.5).
//!
//! Collective memory is maintained by ritual, storytelling, monuments,
//! education, anniversary events, and institutional repetition.
//!
//! ```text
//! Collective memory components:
//!   - founding myths (identity-giving narratives)
//!   - shared traumas (collective suffering events)
//!   - heroes (agents who embody group values)
//!   - villains (agents who betrayed or harmed the group)
//!   - sacred events (events that must be remembered)
//!
//! Memory maintenance:
//!   - ritual repetition strengthens shared memories
//!   - education transmits to new members
//!   - anniversaries reinforce salience
//!   - unused memories decay over generations
//! ```

use mindstrata_core::fixed::Fixed;
use serde::{Deserialize, Serialize};

/// A shared memory event held by a group.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedMemory {
    /// Unique memory identifier.
    pub id: usize,
    /// Human-readable description of the event.
    pub description: String,
    /// Category of the shared memory.
    pub kind: SharedMemoryKind,
    /// Tick when the original event occurred.
    pub event_tick: u64,
    /// Salience of this memory to the group (0 = forgotten, 1 = vivid).
    pub salience: Fixed,
    /// Emotional charge of the memory (positive for triumphs, negative for traumas).
    pub emotional_charge: Fixed,
    /// How many times this memory has been rehearsed through ritual or storytelling.
    pub rehearsal_count: u32,
    /// Agent IDs of key participants in the memory.
    pub key_agents: Vec<usize>,
    /// Degree of distortion from the original event (0 = accurate, 1 = mythologized).
    pub distortion_level: Fixed,
    /// Whether this memory is a founding myth (core to group identity).
    pub is_founding_myth: bool,
}

/// Category of shared memory.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SharedMemoryKind {
    /// Founding or origin story.
    Founding,
    /// Collective trauma (disaster, massacre, famine).
    Trauma,
    /// Triumph or victory.
    Triumph,
    /// Sacred event (revelation, miracle, covenant).
    Sacred,
    /// Moral lesson (betrayal, justice, sacrifice).
    Moral,
    /// Heroic deed.
    Heroic,
}

impl SharedMemory {
    /// Create a new shared memory.
    pub fn new(
        id: usize,
        description: String,
        kind: SharedMemoryKind,
        event_tick: u64,
        emotional_charge: Fixed,
    ) -> Self {
        Self {
            id,
            description,
            kind,
            event_tick,
            salience: Fixed::ONE,
            emotional_charge,
            rehearsal_count: 0,
            key_agents: Vec::new(),
            distortion_level: Fixed::ZERO,
            is_founding_myth: false,
        }
    }

    /// Strengthen this memory through rehearsal (ritual, storytelling).
    pub fn rehearse(&mut self) {
        self.rehearsal_count += 1;
        // Each rehearsal strengthens salience slightly, but also increases distortion
        self.salience = (self.salience + Fixed::from_f64(0.05)).clamp_01();
        self.distortion_level =
            (self.distortion_level + Fixed::from_f64(0.01)).clamp_01();
    }

    /// Decay salience over time (memories fade without rehearsal).
    pub fn decay(&mut self, ticks_elapsed: u64) {
        let decay_amount = Fixed::from_f64(0.001) * Fixed::from_int(ticks_elapsed as i64);
        self.salience = (self.salience - decay_amount).max(Fixed::ZERO);
    }
}

/// Collective memory for a group (faction, household, settlement, institution).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveMemory {
    /// Identifier for the group that holds this memory.
    pub group_id: usize,
    /// All shared memories.
    pub memories: Vec<SharedMemory>,
    /// Agent IDs considered heroes by this group.
    pub heroes: Vec<usize>,
    /// Agent IDs considered villains by this group.
    pub villains: Vec<usize>,
    /// Tick of last rehearsal event.
    pub last_rehearsal_tick: u64,
    /// Overall group cohesion derived from shared memory (0 = fragmented, 1 = unified).
    pub cohesion: Fixed,
    /// Next memory id.
    next_id: usize,
}

impl CollectiveMemory {
    /// Create a new collective memory for a group.
    pub fn new(group_id: usize) -> Self {
        Self {
            group_id,
            memories: Vec::new(),
            heroes: Vec::new(),
            villains: Vec::new(),
            last_rehearsal_tick: 0,
            cohesion: Fixed::from_f64(0.5),
            next_id: 0,
        }
    }

    /// Add a new shared memory to this group.
    pub fn add_memory(
        &mut self,
        description: String,
        kind: SharedMemoryKind,
        event_tick: u64,
        emotional_charge: Fixed,
    ) -> usize {
        let id = self.next_id;
        self.next_id += 1;
        let mut mem = SharedMemory::new(id, description, kind, event_tick, emotional_charge);
        if kind == SharedMemoryKind::Founding {
            mem.is_founding_myth = true;
        }
        self.memories.push(mem);
        // New shared events strengthen cohesion
        self.cohesion =
            (self.cohesion + emotional_charge * Fixed::from_f64(0.05)).clamp_01();
        id
    }

    /// Record a hero (agent who embodied group values).
    pub fn add_hero(&mut self, agent_id: usize) {
        if !self.heroes.contains(&agent_id) {
            self.heroes.push(agent_id);
        }
    }

    /// Record a villain (agent who betrayed the group).
    pub fn add_villain(&mut self, agent_id: usize) {
        if !self.villains.contains(&agent_id) {
            self.villains.push(agent_id);
        }
    }

    /// Rehearse all memories (called during ritual or anniversary).
    pub fn rehearse_all(&mut self, tick: u64) {
        self.last_rehearsal_tick = tick;
        for mem in &mut self.memories {
            mem.rehearse();
        }
        // Rehearsal strengthens group cohesion
        self.cohesion = (self.cohesion + Fixed::from_f64(0.02)).clamp_01();
    }

    /// Decay memories that haven't been rehearsed recently.
    pub fn tick_decay(&mut self, tick: u64) {
        let ticks_since_rehearsal = tick.saturating_sub(self.last_rehearsal_tick);
        if ticks_since_rehearsal > 0 {
            for mem in &mut self.memories {
                mem.decay(ticks_since_rehearsal);
            }
        }
        // Cohesion slowly decays without reinforcement
        self.cohesion = (self.cohesion - Fixed::from_f64(0.001)).max(Fixed::ZERO);
    }

    /// Compute the group's identity strength from shared myths.
    pub fn identity_strength(&self) -> Fixed {
        let myth_count = self.memories.iter().filter(|m| m.is_founding_myth).count();
        let hero_count = self.heroes.len();
        let base = Fixed::from_int(myth_count as i64) * Fixed::from_f64(0.15)
            + Fixed::from_int(hero_count as i64) * Fixed::from_f64(0.05);
        (base + self.cohesion * Fixed::from_f64(0.3)).clamp_01()
    }

    /// Compute the group's collective trauma load.
    pub fn trauma_load(&self) -> Fixed {
        self.memories
            .iter()
            .filter(|m| m.kind == SharedMemoryKind::Trauma)
            .map(|m| m.emotional_charge * m.salience)
            .fold(Fixed::ZERO, |acc, t| (acc + t).clamp_01())
    }
}

/// Registry of collective memories for all groups.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectiveMemoryRegistry {
    /// All collective memories, indexed by group_id.
    pub entries: Vec<CollectiveMemory>,
}

impl Default for CollectiveMemoryRegistry {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
        }
    }
}

impl CollectiveMemoryRegistry {
    /// Get or create collective memory for a group.
    pub fn get_or_create(&mut self, group_id: usize) -> &mut CollectiveMemory {
        if !self.entries.iter().any(|e| e.group_id == group_id) {
            self.entries.push(CollectiveMemory::new(group_id));
        }
        self.entries.iter_mut().find(|e| e.group_id == group_id)
            .expect("just inserted")
    }

    /// Get collective memory for a group (read-only).
    pub fn get(&self, group_id: usize) -> Option<&CollectiveMemory> {
        self.entries.iter().find(|e| e.group_id == group_id)
    }

    /// Tick decay on all group memories.
    pub fn tick_all(&mut self, tick: u64) {
        for entry in &mut self.entries {
            entry.tick_decay(tick);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_collective_memory_has_sane_defaults() {
        let cm = CollectiveMemory::new(42);
        assert_eq!(cm.group_id, 42);
        assert!(cm.memories.is_empty());
        assert!(cm.heroes.is_empty());
        assert!(cm.villains.is_empty());
    }

    #[test]
    fn add_memory_increments_id() {
        let mut cm = CollectiveMemory::new(0);
        let id1 = cm.add_memory("founding".into(), SharedMemoryKind::Founding, 0, Fixed::ONE);
        let id2 = cm.add_memory("victory".into(), SharedMemoryKind::Triumph, 100, Fixed::from_f64(0.8));
        assert_eq!(id1, 0);
        assert_eq!(id2, 1);
        assert_eq!(cm.memories.len(), 2);
    }

    #[test]
    fn founding_myth_flag_set_automatically() {
        let mut cm = CollectiveMemory::new(0);
        let id = cm.add_memory("origin".into(), SharedMemoryKind::Founding, 0, Fixed::ONE);
        assert!(cm.memories[id].is_founding_myth);
    }

    #[test]
    fn rehearsal_strengthens_salience() {
        let mut cm = CollectiveMemory::new(0);
        let id = cm.add_memory("event".into(), SharedMemoryKind::Triumph, 0, Fixed::from_f64(0.5));
        // Decay salience first so rehearsal can increase it
        cm.memories[id].salience = Fixed::from_f64(0.5);
        let before = cm.memories[id].salience;
        cm.memories[id].rehearse();
        assert!(cm.memories[id].salience > before);
        assert!(cm.memories[id].distortion_level > Fixed::ZERO);
    }

    #[test]
    fn add_hero_and_villain() {
        let mut cm = CollectiveMemory::new(0);
        cm.add_hero(5);
        cm.add_hero(5); // duplicate ignored
        cm.add_villain(3);
        assert_eq!(cm.heroes.len(), 1);
        assert_eq!(cm.villains.len(), 1);
    }

    #[test]
    fn identity_strength_scales_with_myths_and_heroes() {
        let mut cm = CollectiveMemory::new(0);
        let empty_identity = cm.identity_strength();
        cm.add_memory("founding".into(), SharedMemoryKind::Founding, 0, Fixed::from_f64(0.8));
        cm.add_hero(1);
        let full_identity = cm.identity_strength();
        assert!(full_identity > empty_identity);
    }

    #[test]
    fn trauma_load_from_traumas() {
        let mut cm = CollectiveMemory::new(0);
        cm.add_memory("disaster".into(), SharedMemoryKind::Trauma, 0, Fixed::from_f64(0.9));
        cm.add_memory("victory".into(), SharedMemoryKind::Triumph, 0, Fixed::from_f64(0.8));
        let load = cm.trauma_load();
        assert!(load > Fixed::ZERO);
    }

    #[test]
    fn registry_get_or_create() {
        let mut reg = CollectiveMemoryRegistry::default();
        reg.get_or_create(10);
        reg.get_or_create(10); // should not duplicate
        assert_eq!(reg.entries.len(), 1);
        assert!(reg.get(10).is_some());
        assert!(reg.get(99).is_none());
    }
}
