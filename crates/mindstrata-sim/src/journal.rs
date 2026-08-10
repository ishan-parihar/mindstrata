//! Event journal — append-only causal provenance log.
//!
//! Every important event is recorded with tick, agent, kind, and optional cause.
//! This enables debugging emergent behavior by tracing causal chains.

use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Kinds of events worth recording in the journal.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum JournalEntryKind {
    Consumed { resource: String, amount: f64 },
    Worked { productivity: f64 },
    Worshiped,
    Rested,
    TheftDetected { resource: String, amount: f64, fine: f64 },
    CommittedViolence { target: u64, injury: f64 },
    /// §19.5.F: Wealth inherited from a deceased agent.
    Inheritance { heir_count: u64, amount: f64 },
    /// §31: An agent died (generational replacement fills the slot).
    Died { age: f64, cause: String },
    /// §19.5.I: Knowledge discovered through work or exploration.
    KnowledgeDiscovered { knowledge_id: u64, name: String },
    /// §19.5.F: Knowledge learned from parent through childhood socialization.
    KnowledgeSocialized { knowledge_id: u64 },
    /// §5 (Iteration 149): A court returned a verdict on a prosecuted
    /// violation — the supplemental court fine on a Guilty verdict.
    LegalVerdict { case_id: u64, guilty: bool, sentence: f64 },
    /// §5 (Iteration 150): A hostile neighboring settlement raided the
    /// village granary.
    TradeRaid { settlement: String, grain_lost: f64 },
    /// §5 (Iteration 150): A caravan arrived from a neighboring settlement.
    TradeCaravan { settlement: String, grain_gained: f64 },
    /// §5 (Iteration 151): A formal school term convened — the teacher,
    /// cohort size, and graduates of the term.
    SchoolTerm { teacher: u64, cohort: u64, graduates: u64 },
    /// §5 (Iteration 152): A religion's yearly conversion pass recorded a
    /// number of new converts.
    TheologyConversion { converts: u64 },
    /// §5 (Iteration 152): A mid-year religious festival was held, with the
    /// attending believer count.
    TheologyFestival { attenders: u64 },
    /// §5 (Iteration 153): A military drill pass — the militia trained,
    /// building collective readiness.
    MilitaryDrill { attenders: u64, readiness: f64 },
}

/// A single journal entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JournalEntry {
    pub tick: u64,
    pub agent: AgentId,
    pub kind: JournalEntryKind,
}

/// Append-only event journal.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventJournal {
    entries: Vec<JournalEntry>,
}

impl EventJournal {
    /// Create a new empty journal.
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// Record an event.
    pub fn record(&mut self, tick: u64, agent: AgentId, kind: JournalEntryKind) {
        self.entries.push(JournalEntry { tick, agent, kind });
    }

    /// Get all entries for a specific agent.
    pub fn entries_for_agent(&self, agent: AgentId) -> Vec<&JournalEntry> {
        self.entries.iter().filter(|e| e.agent == agent).collect()
    }

    /// Get entries in a tick range.
    pub fn entries_in_range(&self, start: u64, end: u64) -> Vec<&JournalEntry> {
        self.entries.iter().filter(|e| e.tick >= start && e.tick < end).collect()
    }

    /// Get the last N entries.
    pub fn recent(&self, n: usize) -> &[JournalEntry] {
        let start = self.entries.len().saturating_sub(n);
        &self.entries[start..]
    }

    /// Total number of entries.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if the journal is empty.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn journal_records_and_retrieves() {
        let mut journal = EventJournal::new();
        let agent = AgentId::new(0);

        journal.record(100, agent, JournalEntryKind::Worshiped);
        journal.record(101, agent, JournalEntryKind::Consumed { resource: "grain".into(), amount: 0.5 });

        assert_eq!(journal.len(), 2);
        assert!(!journal.is_empty());

        let agent_entries = journal.entries_for_agent(agent);
        assert_eq!(agent_entries.len(), 2);
    }

    #[test]
    fn journal_range_filter() {
        let mut journal = EventJournal::new();
        let a0 = AgentId::new(0);
        let a1 = AgentId::new(1);

        journal.record(10, a0, JournalEntryKind::Worshiped);
        journal.record(20, a1, JournalEntryKind::Rested);
        journal.record(30, a0, JournalEntryKind::Worked { productivity: 0.5 });

        let range = journal.entries_in_range(15, 25);
        assert_eq!(range.len(), 1);
        assert_eq!(range[0].tick, 20);
    }

    #[test]
    fn journal_recent() {
        let mut journal = EventJournal::new();
        let agent = AgentId::new(0);

        for i in 0..10 {
            journal.record(i, agent, JournalEntryKind::Rested);
        }

        let recent = journal.recent(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].tick, 7);
        assert_eq!(recent[2].tick, 9);
    }
}
