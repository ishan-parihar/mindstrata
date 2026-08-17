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

/// §19.5.B: Record of how a belief was updated — source, evidence, outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefUpdateTrace {
    /// Which agent's belief was updated.
    pub agent: AgentId,
    /// Tick when the update occurred.
    pub tick: u64,
    /// Which proposition was affected.
    pub proposition_id: u64,
    /// Confidence before the update.
    pub old_confidence: Fixed,
    /// Confidence after the update.
    pub new_confidence: Fixed,
    /// What caused the update (e.g., "gossip", "direct_experience", "institutional_record").
    pub cause: String,
    /// How much the belief changed (delta).
    pub delta: Fixed,
}

/// §19.5.B: Record of an institutional decision — policy, enforcement, tax collection.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstitutionalTrace {
    /// Which institution made this decision.
    pub institution_name: String,
    /// Tick when the decision was made.
    pub tick: u64,
    /// What kind of decision (e.g., "tax_collection", "wage_payment", "policy_enacted").
    pub decision_kind: String,
    /// Description of what happened.
    pub description: String,
    /// Affected agents (if any).
    pub affected: Vec<AgentId>,
    /// Outcome success.
    pub success: bool,
}

/// §19.5.J: Record of a relationship change — trust, affection, fear changes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipTrace {
    /// Source agent.
    pub from: AgentId,
    /// Target agent.
    pub to: AgentId,
    /// Tick when the change occurred.
    pub tick: u64,
    /// What caused the change (e.g., "social_interaction", "violence", "norm_violation", "gossip").
    pub cause: String,
    /// Trust before the change.
    pub old_trust: Fixed,
    /// Trust after the change.
    pub new_trust: Fixed,
    /// Affection before the change.
    pub old_affection: Fixed,
    /// Affection after the change.
    pub new_affection: Fixed,
    /// Brief description.
    pub description: String,
}

/// §16.1: Cross-system provenance category — tracks which subsystems influenced an event.
///
/// Every new system must produce debug traces. This enum classifies the
/// causal origin of events across the full substrate stack.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProvenanceCategory {
    /// Biological cause — genome, metabolism, immune, cardiovascular.
    Biological,
    /// Hormonal modulation — endocrine axes (cortisol, oxytocin, testosterone, etc.).
    Hormonal,
    /// Attachment trigger — attachment style influenced behavior.
    Attachment,
    /// Identity threat — self-model or group identity was challenged.
    IdentityThreat,
    /// Meme exposure — agent was exposed to a cultural meme.
    Meme,
    /// Propaganda exposure — institutional narrative campaign.
    Propaganda,
    /// Relationship transition — trust/affection stage changed.
    Relationship,
    /// Status change — wealth, role, or social status shifted.
    Status,
    /// Group formation — faction, household, or kinship group created.
    Group,
    /// Ritual effect — ritual participation altered state.
    Ritual,
    /// Belief mutation — belief confidence changed significantly.
    Belief,
    /// Trauma trigger — traumatic event occurred.
    Trauma,
    /// Reproductive event — conception, birth, or pregnancy milestone.
    Reproductive,
}

impl std::fmt::Display for ProvenanceCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Biological => write!(f, "biological"),
            Self::Hormonal => write!(f, "hormonal"),
            Self::Attachment => write!(f, "attachment"),
            Self::IdentityThreat => write!(f, "identity_threat"),
            Self::Meme => write!(f, "meme"),
            Self::Propaganda => write!(f, "propaganda"),
            Self::Relationship => write!(f, "relationship"),
            Self::Status => write!(f, "status"),
            Self::Group => write!(f, "group"),
            Self::Ritual => write!(f, "ritual"),
            Self::Belief => write!(f, "belief"),
            Self::Trauma => write!(f, "trauma"),
            Self::Reproductive => write!(f, "reproductive"),
        }
    }
}

/// §16.1: A cross-system provenance trace — records which subsystems influenced an event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemTrace {
    /// Agent this trace applies to.
    pub agent: AgentId,
    /// Tick when the trace was recorded.
    pub tick: u64,
    /// Which subsystem caused this event.
    pub category: ProvenanceCategory,
    /// Human-readable description of what happened.
    pub description: String,
    /// Quantitative magnitude of the effect (0-1 scale).
    pub magnitude: Fixed,
    /// Optional causal parent description.
    pub cause: String,
}

/// The Causal Provenance Store — tracks causality across the simulation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CausalProvenance {
    /// Decision traces indexed by (tick, agent).
    decisions: Vec<DecisionTrace>,
    /// Event provenance indexed by tick.
    events: Vec<EventProvenance>,
    /// §19.5.B: Belief update traces for debugging information propagation.
    belief_updates: Vec<BeliefUpdateTrace>,
    /// §19.5.B: Institutional decision traces for debugging systemic causality.
    institutional: Vec<InstitutionalTrace>,
    /// §19.5.J: Relationship change traces for debugging social dynamics.
    relationships: Vec<RelationshipTrace>,
    /// §16.1: Cross-system provenance traces for explainability.
    system_traces: Vec<SystemTrace>,
}

impl CausalProvenance {
    /// Create a new empty provenance store.
    pub fn new() -> Self {
        Self {
            decisions: Vec::new(),
            events: Vec::new(),
            belief_updates: Vec::new(),
            institutional: Vec::new(),
            relationships: Vec::new(),
            system_traces: Vec::new(),
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

    /// §19.5.B: Record a belief update trace.
    pub fn record_belief_update(&mut self, trace: BeliefUpdateTrace) {
        self.belief_updates.push(trace);
    }

    /// §19.5.B: Record an institutional decision trace.
    pub fn record_institutional(&mut self, trace: InstitutionalTrace) {
        self.institutional.push(trace);
    }

    /// §19.5.J: Record a relationship change trace.
    pub fn record_relationship(&mut self, trace: RelationshipTrace) {
        self.relationships.push(trace);
    }

    /// Get all decisions made by a specific agent.
    pub fn decisions_for_agent(&self, agent: AgentId) -> Vec<&DecisionTrace> {
        self.decisions.iter().filter(|d| d.agent == agent).collect()
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

    /// §19.5.B: Get belief update traces for a specific agent.
    pub fn belief_updates_for_agent(&self, agent: AgentId) -> Vec<&BeliefUpdateTrace> {
        self.belief_updates
            .iter()
            .filter(|b| b.agent == agent)
            .collect()
    }

    /// §19.5.B: Get institutional traces for a specific institution.
    pub fn institutional_for(&self, name: &str) -> Vec<&InstitutionalTrace> {
        self.institutional
            .iter()
            .filter(|i| i.institution_name == name)
            .collect()
    }

    /// §19.5.B: Get the last N belief update traces.
    pub fn recent_belief_updates(&self, n: usize) -> &[BeliefUpdateTrace] {
        let start = self.belief_updates.len().saturating_sub(n);
        &self.belief_updates[start..]
    }

    /// §19.5.B: Get the last N institutional traces.
    pub fn recent_institutional(&self, n: usize) -> &[InstitutionalTrace] {
        let start = self.institutional.len().saturating_sub(n);
        &self.institutional[start..]
    }

    /// Total number of decision traces.
    pub fn decision_count(&self) -> usize {
        self.decisions.len()
    }

    /// Total number of event provenances.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// §19.5.B: Total number of belief update traces.
    pub fn belief_update_count(&self) -> usize {
        self.belief_updates.len()
    }

    /// §19.5.B: Total number of institutional traces.
    pub fn institutional_count(&self) -> usize {
        self.institutional.len()
    }

    /// §19.5.J: Get relationship traces for a specific agent (as source or target).
    pub fn relationships_for_agent(&self, agent: AgentId) -> Vec<&RelationshipTrace> {
        self.relationships
            .iter()
            .filter(|r| r.from == agent || r.to == agent)
            .collect()
    }

    /// §19.5.J: Get relationship traces between two specific agents.
    pub fn relationships_between(&self, from: AgentId, to: AgentId) -> Vec<&RelationshipTrace> {
        self.relationships
            .iter()
            .filter(|r| r.from == from && r.to == to)
            .collect()
    }

    /// §19.5.J: Get the last N relationship traces.
    pub fn recent_relationships(&self, n: usize) -> &[RelationshipTrace] {
        let start = self.relationships.len().saturating_sub(n);
        &self.relationships[start..]
    }

    /// §19.5.J: Total number of relationship traces.
    pub fn relationship_count(&self) -> usize {
        self.relationships.len()
    }

    // ── §16.1: Cross-system provenance ─────────────────────────────

    /// §16.1: Record a cross-system provenance trace.
    pub fn record_system(&mut self, trace: SystemTrace) {
        self.system_traces.push(trace);
    }

    /// §16.1: Get system traces for a specific agent.
    pub fn system_traces_for_agent(&self, agent: AgentId) -> Vec<&SystemTrace> {
        self.system_traces
            .iter()
            .filter(|t| t.agent == agent)
            .collect()
    }

    /// §16.1: Get system traces for a specific category.
    pub fn system_traces_by_category(&self, category: ProvenanceCategory) -> Vec<&SystemTrace> {
        self.system_traces
            .iter()
            .filter(|t| t.category == category)
            .collect()
    }

    /// §16.1: Get the last N system traces.
    pub fn recent_system_traces(&self, n: usize) -> &[SystemTrace] {
        let start = self.system_traces.len().saturating_sub(n);
        &self.system_traces[start..]
    }

    /// §16.1: Total number of system traces.
    pub fn system_trace_count(&self) -> usize {
        self.system_traces.len()
    }

    /// Trim all trace vectors to at most `max` entries each, keeping the most recent.
    pub fn trim(&mut self, max: usize) {
        if self.decisions.len() > max {
            self.decisions.drain(..self.decisions.len() - max);
        }
        if self.events.len() > max {
            self.events.drain(..self.events.len() - max);
        }
        if self.belief_updates.len() > max {
            self.belief_updates.drain(..self.belief_updates.len() - max);
        }
        if self.institutional.len() > max {
            self.institutional.drain(..self.institutional.len() - max);
        }
        if self.relationships.len() > max {
            self.relationships.drain(..self.relationships.len() - max);
        }
        if self.system_traces.len() > max {
            self.system_traces.drain(..self.system_traces.len() - max);
        }
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
                action_name: format!("Action{i}"),
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

    // ── §19.5.B: Belief Update Trace Tests ────────────────────────────

    #[test]
    fn belief_update_trace_recorded() {
        let mut store = CausalProvenance::new();
        store.record_belief_update(BeliefUpdateTrace {
            agent: AgentId::new(2),
            tick: 100,
            proposition_id: 0,
            old_confidence: Fixed::from_f64(0.3),
            new_confidence: Fixed::from_f64(0.7),
            cause: "gossip".into(),
            delta: Fixed::from_f64(0.4),
        });
        assert_eq!(store.belief_update_count(), 1);
        let updates = store.recent_belief_updates(10);
        assert_eq!(updates[0].cause, "gossip");
        assert_eq!(updates[0].agent, AgentId::new(2));
    }

    #[test]
    fn belief_updates_for_agent_filter() {
        let mut store = CausalProvenance::new();
        store.record_belief_update(BeliefUpdateTrace {
            agent: AgentId::new(0),
            tick: 100,
            proposition_id: 0,
            old_confidence: Fixed::from_f64(0.3),
            new_confidence: Fixed::from_f64(0.7),
            cause: "gossip".into(),
            delta: Fixed::from_f64(0.4),
        });
        store.record_belief_update(BeliefUpdateTrace {
            agent: AgentId::new(1),
            tick: 100,
            proposition_id: 1,
            old_confidence: Fixed::from_f64(0.6),
            new_confidence: Fixed::from_f64(0.2),
            cause: "direct_experience".into(),
            delta: Fixed::from_f64(-0.4),
        });
        let agent0 = store.belief_updates_for_agent(AgentId::new(0));
        assert_eq!(agent0.len(), 1);
        assert_eq!(agent0[0].cause, "gossip");
    }

    // ── §19.5.B: Institutional Trace Tests ────────────────────────────

    #[test]
    fn institutional_trace_recorded() {
        let mut store = CausalProvenance::new();
        store.record_institutional(InstitutionalTrace {
            institution_name: "Council".into(),
            tick: 100,
            decision_kind: "tax_collection".into(),
            description: "Collected 5.0 coins from 3 members".into(),
            affected: vec![AgentId::new(0), AgentId::new(1), AgentId::new(2)],
            success: true,
        });
        assert_eq!(store.institutional_count(), 1);
        let traces = store.recent_institutional(10);
        assert_eq!(traces[0].institution_name, "Council");
        assert_eq!(traces[0].affected.len(), 3);
    }

    #[test]
    fn institutional_for_filter() {
        let mut store = CausalProvenance::new();
        store.record_institutional(InstitutionalTrace {
            institution_name: "Council".into(),
            tick: 100,
            decision_kind: "tax_collection".into(),
            description: "Tax collected".into(),
            affected: vec![],
            success: true,
        });
        store.record_institutional(InstitutionalTrace {
            institution_name: "Temple".into(),
            tick: 100,
            decision_kind: "tithe_collection".into(),
            description: "Tithe collected".into(),
            affected: vec![],
            success: true,
        });
        let council = store.institutional_for("Council");
        assert_eq!(council.len(), 1);
        assert_eq!(council[0].decision_kind, "tax_collection");
    }

    // ── §19.5.J: Relationship Trace Tests ────────────────────────────

    #[test]
    fn relationship_trace_recorded() {
        let mut store = CausalProvenance::new();
        store.record_relationship(RelationshipTrace {
            from: AgentId::new(0),
            to: AgentId::new(1),
            tick: 100,
            cause: "social_interaction".into(),
            old_trust: Fixed::from_f64(0.5),
            new_trust: Fixed::from_f64(0.6),
            old_affection: Fixed::from_f64(0.3),
            new_affection: Fixed::from_f64(0.4),
            description: "Trust increased after friendly interaction".into(),
        });
        assert_eq!(store.relationship_count(), 1);
        let traces = store.recent_relationships(10);
        assert_eq!(traces[0].cause, "social_interaction");
        assert_eq!(traces[0].from, AgentId::new(0));
        assert_eq!(traces[0].to, AgentId::new(1));
    }

    #[test]
    fn relationships_for_agent_filter() {
        let mut store = CausalProvenance::new();
        store.record_relationship(RelationshipTrace {
            from: AgentId::new(0),
            to: AgentId::new(1),
            tick: 100,
            cause: "social_interaction".into(),
            old_trust: Fixed::from_f64(0.5),
            new_trust: Fixed::from_f64(0.6),
            old_affection: Fixed::from_f64(0.3),
            new_affection: Fixed::from_f64(0.4),
            description: "Trust increased".into(),
        });
        store.record_relationship(RelationshipTrace {
            from: AgentId::new(2),
            to: AgentId::new(3),
            tick: 100,
            cause: "violence".into(),
            old_trust: Fixed::from_f64(0.8),
            new_trust: Fixed::from_f64(0.2),
            old_affection: Fixed::from_f64(0.7),
            new_affection: Fixed::from_f64(0.1),
            description: "Trust destroyed by violence".into(),
        });
        let agent0 = store.relationships_for_agent(AgentId::new(0));
        assert_eq!(agent0.len(), 1);
        assert_eq!(agent0[0].cause, "social_interaction");
    }

    #[test]
    fn relationships_between_filter() {
        let mut store = CausalProvenance::new();
        store.record_relationship(RelationshipTrace {
            from: AgentId::new(0),
            to: AgentId::new(1),
            tick: 100,
            cause: "social_interaction".into(),
            old_trust: Fixed::from_f64(0.5),
            new_trust: Fixed::from_f64(0.6),
            old_affection: Fixed::from_f64(0.3),
            new_affection: Fixed::from_f64(0.4),
            description: "Trust increased".into(),
        });
        store.record_relationship(RelationshipTrace {
            from: AgentId::new(0),
            to: AgentId::new(1),
            tick: 105,
            cause: "violence".into(),
            old_trust: Fixed::from_f64(0.6),
            new_trust: Fixed::from_f64(0.1),
            old_affection: Fixed::from_f64(0.4),
            new_affection: Fixed::from_f64(0.0),
            description: "Trust destroyed".into(),
        });
        let traces = store.relationships_between(AgentId::new(0), AgentId::new(1));
        assert_eq!(traces.len(), 2);
        assert_eq!(traces[0].cause, "social_interaction");
        assert_eq!(traces[1].cause, "violence");
    }

    // ── §16.1: Cross-system provenance tests ───────────────────────

    #[test]
    fn system_trace_recorded() {
        let mut store = CausalProvenance::new();
        store.record_system(SystemTrace {
            agent: AgentId::new(0),
            tick: 500,
            category: ProvenanceCategory::Hormonal,
            description: "Cortisol spike from sleep deprivation".into(),
            magnitude: Fixed::from_f64(0.72),
            cause: "circadian_disruption".into(),
        });
        assert_eq!(store.system_trace_count(), 1);
        let traces = store.recent_system_traces(10);
        assert_eq!(traces[0].category, ProvenanceCategory::Hormonal);
    }

    #[test]
    fn system_traces_for_agent_filter() {
        let mut store = CausalProvenance::new();
        store.record_system(SystemTrace {
            agent: AgentId::new(0),
            tick: 100,
            category: ProvenanceCategory::Attachment,
            description: "Anxious attachment triggered clinginess".into(),
            magnitude: Fixed::from_f64(0.5),
            cause: "partner_absence".into(),
        });
        store.record_system(SystemTrace {
            agent: AgentId::new(1),
            tick: 100,
            category: ProvenanceCategory::Trauma,
            description: "Flashback from childhood abuse".into(),
            magnitude: Fixed::from_f64(0.9),
            cause: "loud_noise".into(),
        });
        let agent0 = store.system_traces_for_agent(AgentId::new(0));
        assert_eq!(agent0.len(), 1);
        assert_eq!(agent0[0].category, ProvenanceCategory::Attachment);
    }

    #[test]
    fn system_traces_by_category_filter() {
        let mut store = CausalProvenance::new();
        store.record_system(SystemTrace {
            agent: AgentId::new(0),
            tick: 100,
            category: ProvenanceCategory::Meme,
            description: "Heard rumor about neighbor".into(),
            magnitude: Fixed::from_f64(0.6),
            cause: "gossip".into(),
        });
        store.record_system(SystemTrace {
            agent: AgentId::new(1),
            tick: 100,
            category: ProvenanceCategory::Propaganda,
            description: "Council edict about taxation".into(),
            magnitude: Fixed::from_f64(0.8),
            cause: "institutional_broadcast".into(),
        });
        let meme_traces = store.system_traces_by_category(ProvenanceCategory::Meme);
        assert_eq!(meme_traces.len(), 1);
        assert_eq!(meme_traces[0].agent, AgentId::new(0));
    }

    #[test]
    fn trim_includes_system_traces() {
        let mut store = CausalProvenance::new();
        for i in 0..10 {
            store.record_system(SystemTrace {
                agent: AgentId::new(0),
                tick: i,
                category: ProvenanceCategory::Biological,
                description: format!("bio_event_{i}"),
                magnitude: Fixed::from_f64(0.5),
                cause: "test".into(),
            });
        }
        assert_eq!(store.system_trace_count(), 10);
        store.trim(3);
        assert_eq!(store.system_trace_count(), 3);
    }

    #[test]
    fn provenance_category_count_matches_spec() {
        // §16.1 defines exactly 13 cross-system provenance categories
        let cats = [
            ProvenanceCategory::Biological,
            ProvenanceCategory::Hormonal,
            ProvenanceCategory::Attachment,
            ProvenanceCategory::IdentityThreat,
            ProvenanceCategory::Meme,
            ProvenanceCategory::Propaganda,
            ProvenanceCategory::Relationship,
            ProvenanceCategory::Status,
            ProvenanceCategory::Group,
            ProvenanceCategory::Ritual,
            ProvenanceCategory::Belief,
            ProvenanceCategory::Trauma,
            ProvenanceCategory::Reproductive,
        ];
        assert_eq!(cats.len(), 13);
    }
}
