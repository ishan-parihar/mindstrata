//! Relationship V2 — enriched relational model with developmental stages.
//!
//! Replaces simple relationship edges with rich relational systems including:
//! - Core affect (trust, affection, respect, fear, obligation)
//! - Deep relational dimensions (intimacy, passion, commitment, solidarity, resentment)
//! - Identity and meaning (shared identity, role expectations)
//! - History (stage progression, betrayal/reconciliation history)
//! - Dynamics (decay, volatility, last interaction)

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Relationship stage classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub enum RelationshipStage {
    Unnoticed,
    Noticed,
    Acquaintance,
    Familiar,
    Neighbor,
    Friend,
    CloseFriend,
    Confidant,
    Ally,
    Disliked,
    Rival,
    Enemy,
    Nemesis,
    Kin,
    ParentChild,
    Sibling,
    Cousin,
    InLaw,
}

impl RelationshipStage {
    pub fn next_positive(&self) -> Option<Self> {
        match self {
            Self::Unnoticed => Some(Self::Noticed),
            Self::Noticed => Some(Self::Acquaintance),
            Self::Acquaintance => Some(Self::Familiar),
            Self::Familiar => Some(Self::Neighbor),
            Self::Neighbor => Some(Self::Friend),
            Self::Friend => Some(Self::CloseFriend),
            Self::CloseFriend => Some(Self::Confidant),
            Self::Confidant => Some(Self::Ally),
            _ => None,
        }
    }

    pub fn next_negative(&self) -> Option<Self> {
        match self {
            Self::Neighbor => Some(Self::Disliked),
            Self::Disliked => Some(Self::Rival),
            Self::Rival => Some(Self::Enemy),
            Self::Enemy => Some(Self::Nemesis),
            _ => None,
        }
    }

    pub fn base_trust(&self) -> Fixed {
        match self {
            Self::Unnoticed => Fixed::ZERO,
            Self::Noticed => Fixed::from_f64(0.2),
            Self::Acquaintance => Fixed::from_f64(0.35),
            Self::Familiar => Fixed::from_f64(0.5),
            Self::Neighbor => Fixed::from_f64(0.55),
            Self::Friend => Fixed::from_f64(0.65),
            Self::CloseFriend => Fixed::from_f64(0.8),
            Self::Confidant => Fixed::from_f64(0.9),
            Self::Ally => Fixed::from_f64(0.85),
            Self::Disliked => Fixed::from_f64(0.2),
            Self::Rival => Fixed::from_f64(0.1),
            Self::Enemy => Fixed::from_f64(0.05),
            Self::Nemesis => Fixed::ZERO,
            Self::Kin => Fixed::from_f64(0.6),
            Self::ParentChild => Fixed::from_f64(0.8),
            Self::Sibling => Fixed::from_f64(0.7),
            Self::Cousin => Fixed::from_f64(0.5),
            Self::InLaw => Fixed::from_f64(0.45),
        }
    }
}

/// Threshold for progressing from one stage to the next.
pub fn stage_progression_threshold(stage: RelationshipStage) -> Fixed {
    match stage {
        RelationshipStage::Unnoticed => Fixed::from_f64(0.3),
        RelationshipStage::Noticed => Fixed::from_f64(0.4),
        RelationshipStage::Acquaintance => Fixed::from_f64(0.5),
        RelationshipStage::Familiar => Fixed::from_f64(0.6),
        RelationshipStage::Neighbor => Fixed::from_f64(0.7),
        RelationshipStage::Friend => Fixed::from_f64(0.8),
        RelationshipStage::CloseFriend => Fixed::from_f64(0.85),
        RelationshipStage::Confidant => Fixed::from_f64(0.9),
        _ => Fixed::ONE,
    }
}

/// A rich directed relationship between two agents.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipV2 {
    pub from: AgentId,
    pub to: AgentId,
    pub trust: Fixed,
    pub affection: Fixed,
    pub respect: Fixed,
    pub fear: Fixed,
    pub obligation: Fixed,
    pub intimacy: Fixed,
    pub passion: Fixed,
    pub commitment: Fixed,
    pub attachment_security: Fixed,
    pub dependence: Fixed,
    pub power_balance: Fixed,
    pub solidarity: Fixed,
    pub resentment: Fixed,
    pub admiration: Fixed,
    pub jealousy: Fixed,
    pub gratitude: Fixed,
    pub guilt_toward: Fixed,
    pub moral_debt: Fixed,
    pub shared_identity: Fixed,
    pub stage: RelationshipStage,
    pub stage_progress: Fixed,
    pub interaction_count: u32,
    pub last_positive_tick: u64,
    pub last_negative_tick: u64,
    pub decay_rate: Fixed,
    pub volatility: Fixed,

    // §17.3: Relationship caching — skip full updates for dormant relationships.
    // Updated on every interaction; daily decay pass updates all stale ones.
    pub last_update_tick: u64,
    /// Whether this relationship has pending changes that need processing.
    pub dirty: bool,
}

impl RelationshipV2 {
    pub fn new(from: AgentId, to: AgentId) -> Self {
        Self {
            from, to,
            trust: Fixed::from_f64(0.4),
            affection: Fixed::from_f64(0.3),
            respect: Fixed::from_f64(0.3),
            fear: Fixed::ZERO,
            obligation: Fixed::ZERO,
            intimacy: Fixed::ZERO,
            passion: Fixed::ZERO,
            commitment: Fixed::ZERO,
            attachment_security: Fixed::from_f64(0.3),
            dependence: Fixed::ZERO,
            power_balance: Fixed::ZERO,
            solidarity: Fixed::ZERO,
            resentment: Fixed::ZERO,
            admiration: Fixed::ZERO,
            jealousy: Fixed::ZERO,
            gratitude: Fixed::ZERO,
            guilt_toward: Fixed::ZERO,
            moral_debt: Fixed::ZERO,
            shared_identity: Fixed::ZERO,
            stage: RelationshipStage::Unnoticed,
            stage_progress: Fixed::ZERO,
            interaction_count: 0,
            last_positive_tick: 0,
            last_negative_tick: 0,
            decay_rate: Fixed::from_f64(0.0005),
            volatility: Fixed::from_f64(0.5),
            last_update_tick: 0,
            dirty: false,
        }
    }

    pub fn quality(&self) -> Fixed {
        (self.trust * Fixed::from_f64(0.3)
            + self.affection * Fixed::from_f64(0.25)
            + self.respect * Fixed::from_f64(0.2)
            + self.intimacy * Fixed::from_f64(0.15)
            + self.commitment * Fixed::from_f64(0.1))
            .clamp_01()
    }

    pub fn record_positive(&mut self, tick: u64, magnitude: Fixed) {
        let vol = self.volatility;
        self.trust = (self.trust + magnitude * vol * Fixed::from_f64(0.02)).clamp_01();
        self.affection = (self.affection + magnitude * vol * Fixed::from_f64(0.015)).clamp_01();
        self.gratitude = (self.gratitude + magnitude * Fixed::from_f64(0.01)).clamp_01();
        self.resentment = (self.resentment - magnitude * Fixed::from_f64(0.005)).max(Fixed::ZERO);
        self.last_positive_tick = tick;
        self.last_update_tick = tick;
        self.dirty = true;
        self.interaction_count += 1;
    }

    pub fn record_negative(&mut self, tick: u64, magnitude: Fixed) {
        let vol = self.volatility;
        self.trust = (self.trust - magnitude * vol * Fixed::from_f64(0.03)).max(Fixed::ZERO);
        self.affection = (self.affection - magnitude * vol * Fixed::from_f64(0.02)).max(Fixed::ZERO);
        self.resentment = (self.resentment + magnitude * Fixed::from_f64(0.02)).clamp_01();
        self.fear = (self.fear + magnitude * Fixed::from_f64(0.01)).clamp_01();
        self.last_negative_tick = tick;
        self.last_update_tick = tick;
        self.dirty = true;
        self.interaction_count += 1;
    }

    pub fn decay(&mut self, ticks_elapsed: u64) {
        let decay = self.decay_rate * Fixed::from_int(ticks_elapsed as i64);
        self.trust = (self.trust - decay).max(Fixed::ZERO);
        self.affection = (self.affection - decay * Fixed::from_f64(0.5)).max(Fixed::ZERO);
        self.intimacy = (self.intimacy - decay * Fixed::from_f64(0.3)).max(Fixed::ZERO);
        // NOTE: dirty is NOT set here — decay is passive, not an interaction.
        // dirty is only set by record_positive/record_negative (active interactions).
    }

    /// §17.3: Check if this relationship needs decay this tick.
    ///
    /// Relationships are "dormant" when no interaction, rumor, or emotional
    /// event has touched them recently. Dormant relationships only receive
    /// the daily decay pass (every 144 ticks). Active (dirty) relationships
    /// decay every tick until cleared.
    pub fn is_active_this_tick(&self, current_tick: u64) -> bool {
        // Active if recently interacted with (dirty)
        if self.dirty {
            return true;
        }
        // Also active on daily boundary for the bulk decay pass
        current_tick > 0 && current_tick.is_multiple_of(144)
    }

    /// §17.3: Clear dirty flag after processing.
    pub fn clear_dirty(&mut self, tick: u64) {
        self.dirty = false;
        self.last_update_tick = tick;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn quality_computed() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.trust = Fixed::from_f64(0.8);
        r.affection = Fixed::from_f64(0.7);
        r.respect = Fixed::from_f64(0.6);
        r.intimacy = Fixed::from_f64(0.5);
        r.commitment = Fixed::from_f64(0.4);
        let q = r.quality();
        assert!(q > Fixed::from_f64(0.4));
    }

    #[test]
    fn positive_increases_trust() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        let initial = r.trust;
        r.record_positive(0, Fixed::from_f64(0.5));
        assert!(r.trust > initial);
    }

    #[test]
    fn negative_decreases_trust() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.trust = Fixed::from_f64(0.8);
        r.record_negative(0, Fixed::from_f64(0.5));
        assert!(r.trust < Fixed::from_f64(0.8));
    }

    #[test]
    fn stage_progression() {
        let next = RelationshipStage::Unnoticed.next_positive().unwrap();
        assert_eq!(next, RelationshipStage::Noticed);
    }

    // ── §17.3 Relationship Caching Tests ──────────────────────────────

    #[test]
    fn decay_does_not_set_dirty() {
        // §17.3: decay() is passive — it must NOT set the dirty flag.
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(!r.dirty);
        r.decay(1);
        assert!(!r.dirty, "decay() must not set dirty flag");
    }

    #[test]
    fn record_positive_sets_dirty() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(!r.dirty);
        r.record_positive(10, Fixed::from_f64(0.5));
        assert!(r.dirty, "record_positive must set dirty");
        assert_eq!(r.last_update_tick, 10);
    }

    #[test]
    fn record_negative_sets_dirty() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(!r.dirty);
        r.record_negative(20, Fixed::from_f64(0.5));
        assert!(r.dirty, "record_negative must set dirty");
        assert_eq!(r.last_update_tick, 20);
    }

    #[test]
    fn dormant_relationship_skips_decay() {
        // §17.3: A non-dirty relationship at a non-daily-boundary tick
        // should NOT be active (i.e., should skip decay).
        let r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        // Tick 50 is neither dirty nor a daily boundary (144)
        assert!(!r.is_active_this_tick(50),
            "Dormant relationship should not be active at non-daily tick");
    }

    #[test]
    fn daily_boundary_activates_all() {
        // §17.3: On daily boundary (tick % 144 == 0), all relationships
        // should be active for the bulk decay pass.
        let r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        assert!(r.is_active_this_tick(144),
            "Daily boundary should activate all relationships");
        assert!(r.is_active_this_tick(288),
            "Daily boundary should activate all relationships");
    }

    #[test]
    fn dirty_relationship_stays_active_until_cleared() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.record_positive(5, Fixed::from_f64(0.5));
        // At tick 100 (non-daily), still active because dirty
        assert!(r.is_active_this_tick(100),
            "Dirty relationship should be active at any tick");
        // Clear dirty
        r.clear_dirty(100);
        assert!(!r.is_active_this_tick(100),
            "After clear_dirty, relationship should be dormant");
    }

    #[test]
    fn clear_dirty_resets_flag() {
        let mut r = RelationshipV2::new(AgentId::new(0), AgentId::new(1));
        r.record_positive(5, Fixed::from_f64(0.5));
        assert!(r.dirty);
        r.clear_dirty(10);
        assert!(!r.dirty);
        assert_eq!(r.last_update_tick, 10);
    }
}
