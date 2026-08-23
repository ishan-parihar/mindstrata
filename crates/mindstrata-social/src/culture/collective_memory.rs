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

use mindstrata_core::event::EventId;
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

/// §13.5 (AP2): A collective trauma — an event of shared suffering that
/// binds a group (disaster, famine, massacre). Derived by the sim's daily
/// pass from `SharedMemory` records of kind `Trauma`.
/// Salience above which a derived trauma is considered actively referenced.
/// Single-sourced so `refresh_derived_views` and consumers agree.
pub const ACTIVE_TRAUMA_SALIENCE_THRESHOLD: f64 = 0.1;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
/// Struct. (doc added at S3 extraction)
pub struct SharedTrauma {
    /// Description of the traumatic event.
    pub description: String,
    /// Tick when the event occurred.
    pub event_tick: u64,
    /// Severity of the trauma (0 = minor hardship, 1 = existential catastrophe).
    pub severity: Fixed,
    /// Whether the trauma is still actively referenced by the group.
    pub active: bool,
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
        self.distortion_level = (self.distortion_level + Fixed::from_f64(0.01)).clamp_01();
    }

    /// Decay salience over time (memories fade without rehearsal).
    pub fn decay(&mut self, ticks_elapsed: u64) {
        self.decay_preserved(ticks_elapsed, Fixed::ONE);
    }

    /// Decay salience over time, scaled by a preservation factor
    /// (§8.1.4, Iteration 129): a nostalgic population's mean nostalgia
    /// slows the daily fade (`decay × preservation`, preservation ∈
    /// [0.7, 1.0] for the shipped constants — never fully frozen, per the
    /// Iter-12 linear-decay design). Deterministic, no RNG.
    pub fn decay_preserved(&mut self, ticks_elapsed: u64, preservation: Fixed) {
        let decay_amount =
            Fixed::from_f64(0.001) * Fixed::from_int(ticks_elapsed as i64) * preservation;
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
    /// §13.5 (AP2): Founding myths — meme ids carrying the group's origin
    /// narratives. Derived by the sim's daily meme pass from memes with
    /// `MemeContent::Historical` (the plan types this `Vec<MemeId>`).
    #[serde(default)]
    pub founding_myths: Vec<u64>,
    /// §13.5 (AP2): Shared traumas — collective suffering events.
    #[serde(default)]
    pub traumas: Vec<SharedTrauma>,
    /// §13.5 (AP2): Sacred events that must be remembered (revelation,
    /// miracle, covenant). Derived from `Sacred`-kind memories.
    #[serde(default)]
    pub sacred_events: Vec<EventId>,
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
            founding_myths: Vec::new(),
            traumas: Vec::new(),
            sacred_events: Vec::new(),
            heroes: Vec::new(),
            villains: Vec::new(),
            last_rehearsal_tick: 0,
            cohesion: Fixed::from_f64(0.5),
            next_id: 0,
        }
    }

    /// §13.5 (AP2): Recompute the derived plan fields (`traumas`,
    /// `sacred_events`) from the shared-memory log. `founding_myths` is
    /// derived by the sim's daily meme pass (it references the meme
    /// registry). Deterministic: reads only `memories`, writes only the new
    /// fields, so the golden baseline stays byte-identical.
    pub fn refresh_derived_views(&mut self) {
        self.traumas = self
            .memories
            .iter()
            .filter(|m| m.kind == SharedMemoryKind::Trauma)
            .map(|m| SharedTrauma {
                description: m.description.clone(),
                event_tick: m.event_tick,
                severity: m.salience,
                active: m.salience > Fixed::from_f64(ACTIVE_TRAUMA_SALIENCE_THRESHOLD),
            })
            .collect();
        self.sacred_events = self
            .memories
            .iter()
            .filter(|m| m.kind == SharedMemoryKind::Sacred)
            .map(|m| EventId::new(m.id as u64))
            .collect();
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
        self.cohesion = (self.cohesion + emotional_charge * Fixed::from_f64(0.05)).clamp_01();
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
        self.tick_decay_preserved(tick, Fixed::ONE);
    }

    /// Decay memories that haven't been rehearsed recently, scaled by a
    /// preservation factor (§8.1.4, Iteration 129 — the nostalgia consumer;
    /// see `decay_preserved`).
    pub fn tick_decay_preserved(&mut self, _tick: u64, preservation: Fixed) {
        // The sim calls `tick_all` once per day (daily phase), so decay by a
        // fixed one-day amount here. Decaying by the FULL ticks-since-rehearsal
        // on every daily call accumulated quadratically (0.001·t, 0.001·2t, …),
        // wiping every memory to salience 0 within ~2 weeks — no ritual could
        // ever sustain them (Iteration 12 fix). At 0.001/day a memory fades
        // over ~2.7 years without rehearsal, and the monthly ritual rehearse
        // (+0.05) comfortably outpaces it.
        for mem in &mut self.memories {
            mem.decay_preserved(1, preservation);
        }
        // Cohesion slowly decays without reinforcement.
        let cohesion_decay = Fixed::from_f64(0.001) * preservation;
        self.cohesion = (self.cohesion - cohesion_decay).max(Fixed::ZERO);
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
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct CollectiveMemoryRegistry {
    /// All collective memories, indexed by group_id.
    pub entries: Vec<CollectiveMemory>,
}

impl CollectiveMemoryRegistry {
    /// Get or create collective memory for a group.
    pub fn get_or_create(&mut self, group_id: usize) -> &mut CollectiveMemory {
        let idx = if let Some(i) = self.entries.iter().position(|e| e.group_id == group_id) {
            i
        } else {
            self.entries.push(CollectiveMemory::new(group_id));
            self.entries.len() - 1
        };
        &mut self.entries[idx]
    }

    /// Get collective memory for a group (read-only).
    pub fn get(&self, group_id: usize) -> Option<&CollectiveMemory> {
        self.entries.iter().find(|e| e.group_id == group_id)
    }

    /// Tick decay on all group memories.
    pub fn tick_all(&mut self, tick: u64) {
        self.tick_all_preserved(tick, Fixed::ONE);
    }

    /// Tick decay on all group memories, scaled by a preservation factor
    /// (§8.1.4, Iteration 129 — the nostalgia consumer; see
    /// `decay_preserved`).
    pub fn tick_all_preserved(&mut self, tick: u64, preservation: Fixed) {
        for entry in &mut self.entries {
            entry.tick_decay_preserved(tick, preservation);
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
        let id2 = cm.add_memory(
            "victory".into(),
            SharedMemoryKind::Triumph,
            100,
            Fixed::from_f64(0.8),
        );
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
        let id = cm.add_memory(
            "event".into(),
            SharedMemoryKind::Triumph,
            0,
            Fixed::from_f64(0.5),
        );
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
        cm.add_memory(
            "founding".into(),
            SharedMemoryKind::Founding,
            0,
            Fixed::from_f64(0.8),
        );
        cm.add_hero(1);
        let full_identity = cm.identity_strength();
        assert!(full_identity > empty_identity);
    }

    #[test]
    fn trauma_load_from_traumas() {
        let mut cm = CollectiveMemory::new(0);
        cm.add_memory(
            "disaster".into(),
            SharedMemoryKind::Trauma,
            0,
            Fixed::from_f64(0.9),
        );
        cm.add_memory(
            "victory".into(),
            SharedMemoryKind::Triumph,
            0,
            Fixed::from_f64(0.8),
        );
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

    #[test]
    fn tick_decay_is_linear_not_quadratic() {
        let mut cm = CollectiveMemory::new(0);
        cm.add_memory("founding".into(), SharedMemoryKind::Founding, 0, Fixed::ONE);
        cm.memories[0].salience = Fixed::from_f64(0.5);
        // 10 daily calls (once per day) must decay by 10 × 0.001, not by the
        // quadratic sum 0.001·(1+2+…+10) the old full-elapsed decay produced.
        for t in 1..=10u64 {
            cm.tick_decay(t);
        }
        let actual = cm.memories[0].salience;
        let expected = Fixed::from_f64(0.49);
        assert!(
            (actual - expected).to_f64().abs() < 0.001,
            "linear daily decay expected {expected:?}, got {actual:?}"
        );
    }

    #[test]
    fn decay_preserved_scales_the_linear_daily_fade() {
        // §8.1.4 (Iteration 129): `decay_preserved` scales the same linear
        // daily fade by the preservation factor — at the shipped floor 0.7
        // a memory loses 0.0007/day instead of 0.001/day; at identity 1.0
        // it is byte-identical to the un-scaled `decay`. Deterministic,
        // no RNG.
        let mut id = CollectiveMemory::new(0);
        id.add_memory("founding".into(), SharedMemoryKind::Founding, 0, Fixed::ONE);
        id.memories[0].salience = Fixed::from_f64(0.5);
        let mut preserved = id.clone();
        id.tick_decay(1);
        preserved.tick_decay_preserved(1, Fixed::from_f64(0.7));
        let id_salience = id.memories[0].salience.to_f64();
        let preserved_salience = preserved.memories[0].salience.to_f64();
        assert!(
            (id_salience - 0.499).abs() < 0.00001,
            "identity preservation must match the un-scaled decay (got {id_salience})"
        );
        assert!(
            (preserved_salience - 0.4993).abs() < 0.00001,
            "floor-0.7 preservation must scale the fade to 0.0007/day (got {preserved_salience})"
        );
        assert!(
            preserved_salience > id_salience,
            "preservation must strictly retain more salience than the un-scaled fade"
        );
    }

    #[test]
    fn refresh_derived_views_maps_trauma_memories_to_shared_traumas() {
        let mut cm = CollectiveMemory::new(0);
        cm.add_memory(
            "drought".into(),
            SharedMemoryKind::Trauma,
            500,
            Fixed::from_f64(0.9),
        );
        cm.add_memory(
            "victory".into(),
            SharedMemoryKind::Triumph,
            600,
            Fixed::from_f64(0.8),
        );
        cm.memories[0].salience = Fixed::from_f64(0.7);
        cm.refresh_derived_views();
        assert_eq!(cm.traumas.len(), 1);
        assert_eq!(cm.traumas[0].description, "drought");
        assert_eq!(cm.traumas[0].event_tick, 500);
        assert_eq!(cm.traumas[0].severity, Fixed::from_f64(0.7));
        assert!(cm.traumas[0].active);
    }

    #[test]
    fn refresh_derived_views_maps_sacred_memories_to_event_ids() {
        let mut cm = CollectiveMemory::new(0);
        let id = cm.add_memory("covenant".into(), SharedMemoryKind::Sacred, 900, Fixed::ONE);
        cm.add_memory(
            "routine".into(),
            SharedMemoryKind::Moral,
            950,
            Fixed::from_f64(0.3),
        );
        cm.refresh_derived_views();
        assert_eq!(cm.sacred_events.len(), 1);
        assert_eq!(cm.sacred_events[0], EventId::new(id as u64));
    }

    #[test]
    fn inactive_trauma_when_salience_fades() {
        let mut cm = CollectiveMemory::new(0);
        cm.add_memory(
            "famine".into(),
            SharedMemoryKind::Trauma,
            100,
            Fixed::from_f64(0.9),
        );
        cm.memories[0].salience = Fixed::from_f64(0.05);
        cm.refresh_derived_views();
        assert_eq!(cm.traumas.len(), 1);
        assert!(!cm.traumas[0].active);
    }

    #[test]
    fn plan_fields_default_empty_and_old_save_restores() {
        let cm = CollectiveMemory::new(7);
        assert!(cm.founding_myths.is_empty());
        assert!(cm.traumas.is_empty());
        assert!(cm.sacred_events.is_empty());
        // Old save without the three new fields restores via serde(default).
        // `Fixed` serializes as its raw scaled i64 (SCALE = 10_000), so
        // cohesion 0.5 is the literal 5000.
        let old = r#"{"group_id":7,"memories":[],"heroes":[],"villains":[],"last_rehearsal_tick":0,"cohesion":5000,"next_id":0}"#;
        let restored: CollectiveMemory = serde_json::from_str(old).unwrap();
        assert!(restored.founding_myths.is_empty());
        assert!(restored.traumas.is_empty());
        assert!(restored.sacred_events.is_empty());
    }
}

/// §13.5 crisis-memory policy (moved verbatim from sim/memory_ops during
/// the slimming program): episode-guarded scarcity trauma — one crisis
/// yields one memory; a live trauma (< ~300 days old) blocks re-recording.
/// Seeded origin memories (event_tick == 0) never block. Averages are
/// computed by the caller (it owns the agent slice).
pub fn maybe_record_famine(
    registry: &mut CollectiveMemoryRegistry,
    avg_hunger: Fixed,
    avg_thirst: Fixed,
    tick: u64,
) {
    let recent = registry.get(0).is_some_and(|cm| {
        cm.memories.iter().any(|m| {
            m.kind == SharedMemoryKind::Trauma
                && m.event_tick > 0
                && tick.saturating_sub(m.event_tick) < 43200 // ~300 days
        })
    });
    if recent {
        return;
    }
    let (desc, charge) = if avg_thirst > Fixed::from_f64(0.6) {
        ("Thirst gripped the village when the wells ran low", 0.8)
    } else if avg_hunger > Fixed::from_f64(0.6) {
        ("The village went hungry when the harvest failed", 0.8)
    } else {
        return;
    };
    registry.get_or_create(0).add_memory(
        desc.into(),
        SharedMemoryKind::Trauma,
        tick,
        Fixed::from_f64(charge),
    );
}

/// §13.5: faction founding myth — group 1 + faction id keeps faction
/// memory separate from the village's (group 0).
pub fn record_faction_founding(
    registry: &mut CollectiveMemoryRegistry,
    faction_id: usize,
    tick: u64,
) {
    registry.get_or_create(1 + faction_id).add_memory(
        "The faction was born of shared grievance against the council".into(),
        SharedMemoryKind::Founding,
        tick,
        Fixed::from_f64(0.7),
    );
}
