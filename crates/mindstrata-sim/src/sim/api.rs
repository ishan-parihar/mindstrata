//! External command channel and read-only accessors.

use super::{
    CausalProvenance, EventJournal, Fixed, Goal, GoalKind, GoalSource, NormRegistry, Relationship,
    RelationshipV2, SimEvent, Simulation, Tick, World,
};

impl Simulation {
    /// Access the current tick.
    /// §5 (Iteration 155): The interactive-TUI command channel — inject a
    /// high-priority directive goal into an agent's goal queue.
    ///
    /// The goal is exempt from goal-generation decay and need-dropping, so
    /// it persists until satisfied or replaced. While present, the tick's
    /// selection phase honors its aligned action over routine and internal
    /// drives (`command_goal_action`), yielding only to critical needs of
    /// other kinds (the sim's own interruption rule) — so commands are
    /// strong nudges, not mind control: a commanded agent still eats,
    /// drinks, and rests when those are truly pressing.
    ///
    /// Called only by the interactive TUI (and tests) *between* ticks —
    /// never from the tick loop — so calibrated windows are untouched.
    /// Directives serialize into snapshots (they are ordinary goals), so a
    /// commanded world saved to disk carries its directives on reload.
    /// Returns false if the agent index is out of range.
    pub fn command_agent(&mut self, agent_idx: usize, kind: GoalKind) -> bool {
        if agent_idx >= self.agents.len() {
            return false;
        }
        let tick = self.current_tick().as_u64();
        let agent = &mut self.agents[agent_idx];
        // Upsert: a repeat directive of the same kind replaces the old one
        // (prevents stale duplicate directives piling up in the queue).
        agent
            .goals
            .retain(|g| !(g.source == GoalSource::Command && g.kind == kind));
        agent.goals.push(Goal {
            kind,
            priority: Fixed::ONE,
            commitment: Fixed::ONE,
            created_tick: tick,
            source: GoalSource::Command,
        });
        true
    }

    /// §5 (Iteration 155): Cancel every outstanding directive on an agent,
    /// returning it to fully autonomous behavior. Returns false if the agent
    /// index is out of range.
    pub fn clear_commands(&mut self, agent_idx: usize) -> bool {
        if agent_idx >= self.agents.len() {
            return false;
        }
        self.agents[agent_idx]
            .goals
            .retain(|g| g.source != GoalSource::Command);
        true
    }

    pub fn current_tick(&self) -> Tick {
        self.clock.tick()
    }

    /// Number of agents.
    pub fn agent_count(&self) -> usize {
        self.agents.len()
    }

    /// Total events generated.
    pub fn event_count(&self) -> usize {
        self.events.len()
    }

    /// Get a reference to the world.
    pub fn world(&self) -> &World {
        &self.world
    }

    /// Get a reference to all relationships.
    pub fn relationships(&self) -> &[Relationship] {
        &self.relationships
    }

    /// §10.3 (Iteration 201): the V2 relationship edge from `from` to `to`,
    /// when it exists — exposes the enriched stage/stage_progress fields the
    /// legacy `Relationship` summary lacks, for zero-blast observability
    /// (the TUI relationship view).
    pub fn relationship_v2_between(&self, from: usize, to: usize) -> Option<&RelationshipV2> {
        if from >= self.agents.len() || to >= self.agents.len() || from == to {
            return None;
        }
        let idx = Self::relationship_v2_pos(from, to);
        self.agents[from].relationship_v2s.get(idx)
    }

    /// Get recent events (last n).
    pub fn recent_events(&self, n: usize) -> &[SimEvent] {
        let start = self.events.len().saturating_sub(n);
        &self.events[start..]
    }

    /// Get a reference to the event journal.
    pub fn journal(&self) -> &EventJournal {
        &self.journal
    }

    /// Get journal entry count.
    pub fn journal_len(&self) -> usize {
        self.journal.len()
    }

    /// Get a reference to the norm registry.
    pub fn norms(&self) -> &NormRegistry {
        &self.norms
    }

    /// §34: Get a reference to the causal provenance store.
    pub fn provenance(&self) -> &CausalProvenance {
        &self.provenance
    }
}
