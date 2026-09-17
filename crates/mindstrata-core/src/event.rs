//! Simulation events — the causal memory of the world.
//!
//! Events are **not** UI notifications.  They are first-class simulation
//! objects that drive memory formation, emotional appraisal, gossip,
//! institutional response, narrative logging, and replay.
//!
//! Every important state change should be captured as an event so that
//! causal provenance can be traced.

use crate::clock::Tick;
use crate::conflict::ConflictKind;
use crate::fixed::Fixed;
use crate::id::{AgentId, EntityId, ResourceId};
use serde::{Deserialize, Serialize};

/// Unique identifier for an event instance.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct EventId(u64);

impl EventId {
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    pub const fn as_u64(self) -> u64 {
        self.0
    }
}

impl std::fmt::Debug for EventId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Ev({})", self.0)
    }
}

/// A simulation event.
///
/// Each variant captures the minimum data needed for downstream systems
/// (appraisal, gossip, memory, provenance) to react.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SimEvent {
    // ── Agent lifecycle ──────────────────────────────────────────────
    AgentSpawned {
        agent: AgentId,
        tick: Tick,
    },
    AgentDied {
        agent: AgentId,
        cause: DeathCause,
        tick: Tick,
    },
    /// Iteration-272 (§4.3 dead-producer fix): a surviving agent suffers the
    /// loss of a close tie (spouse/co-resident kin). Emitted by the deaths
    /// pass in the same tick it captured the target — BEFORE the in-place
    /// generational replacement clears partner/parent references — so the
    /// development pass's catalyst mapping routes Grief to a *surviving*
    /// subject instead of the deceased's replacement newborn (the old
    /// `AgentDied → Grief(subject)` mapping landed on the replaced slot).
    /// `deceased` is recorded for provenance/appraisal context.
    GriefStruck {
        mourner: AgentId,
        deceased: AgentId,
        tick: Tick,
    },
    /// Iteration-274 (§4.3 dead-producer fix): a communal ritual was executed.
    /// Rituals previously bonded endocrine axes and memories without ever
    /// emitting an event, so the collective field's Relational bucket — whose
    /// only other feed is the once-per-pair MarriageFormed/ChildBorn — was a
    /// dead producer (i274 probe: Relational max_stage pinned at 1.000 even at
    /// N=96). Emitted per executed ritual from the ritual pass; the catalyst
    /// mapping expands it to one Bond catalyst per participant (the substrate's
    /// "ritual/festival participation → culture-line collective catalysts"
    /// channel). `bonding` is the ritual's bonding_effect for magnitude
    /// scaling; `sponsor` and `kind` are provenance context.
    RitualPerformed {
        participants: Vec<AgentId>,
        sponsor: usize,
        ritual_id: u64,
        bonding: Fixed,
        tick: Tick,
    },

    // ── Biological / needs ───────────────────────────────────────────
    AgentAte {
        agent: AgentId,
        food: EntityId,
        tick: Tick,
    },
    AgentDrank {
        agent: AgentId,
        source: EntityId,
        tick: Tick,
    },
    AgentRested {
        agent: AgentId,
        tick: Tick,
    },

    // ── Social ───────────────────────────────────────────────────────
    RelationshipChanged {
        from: AgentId,
        to: AgentId,
        trust_delta: Fixed,
        affection_delta: Fixed,
        tick: Tick,
    },
    InteractionOccurred {
        from: AgentId,
        to: AgentId,
        kind: InteractionKind,
        tick: Tick,
    },

    // ── Economic ─────────────────────────────────────────────────────
    TradeOccurred {
        buyer: AgentId,
        seller: AgentId,
        good: ResourceId,
        quantity: Fixed,
        price: Fixed,
        tick: Tick,
    },

    // ── Institutional ───────────────────────────────────────────────
    NormViolated {
        agent: AgentId,
        norm_id: u64,
        witnesses: Vec<AgentId>,
        tick: Tick,
    },
    InstitutionChangedPolicy {
        institution: EntityId,
        policy_id: u64,
        tick: Tick,
    },

    // ── Informational ────────────────────────────────────────────────
    RumorSpread {
        source: AgentId,
        target: AgentId,
        content_hash: u64,
        distortion: Fixed,
        tick: Tick,
    },
    KnowledgeTransferred {
        source: AgentId,
        target: AgentId,
        knowledge_id: u64,
        tick: Tick,
    },

    // ── Conflict ─────────────────────────────────────────────────────
    ConflictOccurred {
        aggressor: AgentId,
        target: AgentId,
        kind: ConflictKind,
        injury: Fixed,
        fear_induced: Fixed,
        tick: Tick,
    },
    FeudFormed {
        party_a: AgentId,
        party_b: AgentId,
        tick: Tick,
    },

    // ── Demographic ─────────────────────────────────────────────────
    MarriageFormed {
        spouse_a: AgentId,
        spouse_b: AgentId,
        tick: Tick,
    },
    ChildBorn {
        child: AgentId,
        parent_a: AgentId,
        parent_b: AgentId,
        tick: Tick,
    },

    // ── Movement ─────────────────────────────────────────────────────
    AgentMoved {
        agent: AgentId,
        from_site: EntityId,
        to_site: EntityId,
        tick: Tick,
    },
}

/// How an agent died.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DeathCause {
    Starvation,
    Dehydration,
    Disease,
    Violence,
    OldAge,
    Unknown,
}

/// A categorised social interaction.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionKind {
    Talk,
    Help,
    Threaten,
    Trade,
    Gossip,
    Comfort,
    Insult,
    Teach,
}

/// A lightweight event record stored in the event journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventRecord {
    pub id: EventId,
    pub event: SimEvent,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn event_roundtrip() {
        let ev = SimEvent::AgentAte {
            agent: AgentId::new(1),
            food: EntityId::new(99),
            tick: Tick::new(42),
        };
        let json = serde_json::to_string(&ev).unwrap();
        let back: SimEvent = serde_json::from_str(&json).unwrap();
        assert_eq!(format!("{ev:?}"), format!("{back:?}"));
    }
}
