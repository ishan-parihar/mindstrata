//! Causal Provenance Layer — §34 of the architecture spec.
//!
//! Every important event should have:
//! - event ID
//! - tick
//! - causal parents (what events caused this one)
//! - responsible agent/institution
//! - decision trace (what factors influenced the decision)
//! - outcome trace (what resulted from the decision)
//!
//! This enables debugging emergent behavior by tracing causal chains.
//! Instead of modifying every SimEvent variant, we store provenance
//! data in a separate map keyed by tick + agent.

use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use serde::{Deserialize, Serialize};

/// Why an agent selected a particular action.
/// §34: "decision trace — what factors influenced the decision"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionTrace {
    /// Which agent made this decision.
    pub agent: AgentId,
    /// Tick when the decision was made.
    pub tick: u64,
    /// The action chosen.
    pub action_name: String,
    /// Key factors that influenced the decision.
    pub factors: Vec<DecisionFactor>,
    /// Whether the agent followed a routine instead of utility selection.
    pub from_routine: bool,
    /// Whether the action was interrupted by critical needs.
    pub interrupted_by_critical_needs: bool,
    /// Whether the agent abandoned an intention to make this decision.
    pub intention_abandoned: bool,
}

/// A single factor that influenced a decision.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DecisionFactor {
    /// What kind of factor (e.g., "need_pressure", "norm_compliance", "identity", "routine").
    pub kind: String,
    /// How much this factor influenced the decision (signed).
    pub magnitude: Fixed,
    /// Optional description of the factor.
    pub description: String,
}

/// The provenance of an event — what caused it.
/// §34: "causal parents — what events caused this one"
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventProvenance {
    /// Tick when the event occurred.
    pub tick: u64,
    /// Agent primarily responsible for this event.
    pub responsible_agent: AgentId,
    /// What kind of event this is.
    pub event_kind: String,
    /// Causal parent event descriptions (what led to this event).
    pub causal_parents: Vec<String>,
    /// Outcome description (what resulted from this event).
    pub outcome: String,
}

/// The Causal Provenance Store — tracks causality across the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalProvenance {
    /// Decision traces indexed by (tick, agent).
    decisions: Vec<DecisionTrace>,
    /// Event provenance indexed by tick.
    events: Vec<EventProvenance>,
}

impl CausalProvenance {
    /// Create a new empty provenance store.
    pub fn new() -> Self {
        Self {
            decisions: Vec::new(),
            events: Vec::new(),
        }
    }

    /// Record a decision trace.
    pub fn record_decision(&mut self, trace: DecisionTrace) {
        self.decisions.push(trace);
    }

    /// Record event provenance.
    pub fn record_event(&mut self, provenance: EventProvenance) {
        self.events.push(provenance);
    }

    /// Get all decisions made by a specific agent.
    pub fn decisions_for_agent(&self, agent: AgentId) -> Vec<&DecisionTrace> {
        self.decisions.iter().filter(|d| d.agent == agent).collect()
    }

    /// Get all decisions made in a specific tick range.
    pub fn decisions_in_range(&self, start_tick: u64, end_tick: u64) -> Vec<&DecisionTrace> {
        self.decisions.iter().filter(|d| d.tick >= start_tick && d.tick < end_tick).collect()
    }

    /// Get all events in a specific tick range.
    pub fn events_in_range(&self, start_tick: u64, end_tick: u64) -> Vec<&EventProvenance> {
        self.events.iter().filter(|e| e.tick >= start_tick && e.tick < end_tick).collect()
    }

    /// Get the last N decisions.
    pub fn recent_decisions(&self, n: usize) -> &[DecisionTrace] {
        let start = self.decisions.len().saturating_sub(n);
        &self.decisions[start..]
    }

    /// Get the last N event provenances.
    pub fn recent_events(&self, n: usize) -> &[EventProvenance] {
        let start = self.events.len().saturating_sub(n);
        &self.events[start..]
    }

    /// Total number of decision traces.
    pub fn decision_count(&self) -> usize {
        self.decisions.len()
    }

    /// Total number of event provenances.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn provenance_store_records_decisions() {
        let mut store = CausalProvenance::new();
        let trace = DecisionTrace {
            agent: AgentId::new(0),
            tick: 100,
            action_name: "Work".into(),
            factors: vec![],
            from_routine: true,
            interrupted_by_critical_needs: false,
            intention_abandoned: false,
        };
        store.record_decision(trace);
        assert_eq!(store.decision_count(), 1);
    }

    #[test]
    fn provenance_store_records_events() {
        let mut store = CausalProvenance::new();
        let prov = EventProvenance {
            tick: 100,
            responsible_agent: AgentId::new(0),
            event_kind: "NormViolated(No Theft)".into(),
            causal_parents: vec!["Low trust".into()],
            outcome: "Shame increased".into(),
        };
        store.record_event(prov);
        assert_eq!(store.event_count(), 1);
    }

    #[test]
    fn decisions_for_agent_filter() {
        let mut store = CausalProvenance::new();
        store.record_decision(DecisionTrace {
            agent: AgentId::new(0),
            tick: 100,
            action_name: "Work".into(),
            factors: vec![],
            from_routine: false,
            interrupted_by_critical_needs: false,
            intention_abandoned: false,
        });
        store.record_decision(DecisionTrace {
            agent: AgentId::new(1),
            tick: 100,
            action_name: "Eat".into(),
            factors: vec![],
            from_routine: false,
            interrupted_by_critical_needs: false,
            intention_abandoned: false,
        });

        let agent0 = store.decisions_for_agent(AgentId::new(0));
        assert_eq!(agent0.len(), 1);
        assert_eq!(agent0[0].action_name, "Work");
    }

    #[test]
    fn recent_decisions() {
        let mut store = CausalProvenance::new();
        for i in 0..10 {
            store.record_decision(DecisionTrace {
                agent: AgentId::new(0),
                tick: i,
                action_name: format!("Action{}", i),
                factors: vec![],
                from_routine: false,
                interrupted_by_critical_needs: false,
                intention_abandoned: false,
            });
        }
        let recent = store.recent_decisions(3);
        assert_eq!(recent.len(), 3);
        assert_eq!(recent[0].action_name, "Action7");
    }


}
