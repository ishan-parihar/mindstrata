# Mindstrata — Refactor & Upgrade Plan

**Goal:** Bridge the gap between the current implementation (~55% of architecture.md) and the fully realized simulation, in prioritized phases that each produce a demonstrably better simulation.

**Principle:** Each phase is self-contained and testable. No phase depends on unfinished work from a later phase. Every phase ends with a simulation that runs and produces more interesting emergence than before.

---

## Current State

- **14,000 lines** of Rust across 5 crates
- **231 tests** all passing
- Core simulation loop works deterministically
- Psychological, social, and institutional primitives exist
- **Key gap:** Many modules exist but are not wired into the tick loop
- **Key gap:** Agent spatial positioning and movement are entirely missing
- **Key gap:** Economic trade system exists but agents never trade

---

## Phase 1: Wire the Disconnected Systems (1–2 days)

**Goal:** Make existing code actually run. This is the highest-ROI phase — the code already exists, it just needs to be called.

### 1.1 Wire Demography into Tick Loop

**Files:** `sim.rs`, `demography.rs`

- Call `age_agent()` for each agent every tick
- Call `should_birth()` for partnered agents of childbearing age
- On birth: create new `AgentBundle`, assign parents, add to world
- On death: remove agent, trigger inheritance, emit `AgentDied` event
- Track `DemographyReport` per tick for observability

**Test:** Run 5000 ticks, verify population changes (births + deaths occur).

### 1.2 Wire Market System into Tick Loop

**Files:** `sim.rs`, `market.rs`, `actions.rs`

- Call `system_market()` each tick to update prices
- Wire `execute_trade()` into `ActionKind::Trade` execution
- When agent selects `Trade` action: find a seller at the Market site, execute trade
- Update `TradeVolume` metric in dashboard

**Test:** Run 1000 ticks, verify `trade_volume > 0` and prices fluctuate.

### 1.3 Wire Faction Formation into Tick Loop

**Files:** `sim.rs`, `factions.rs`

- Each tick (or every N ticks): compute per-agent grievance via `compute_grievance()`
- Feed grievances into `find_recruitable_agents()`
- If recruitable count exceeds threshold and cooldown elapsed: select leader, call `create_faction()`
- Add faction to `self.institutions`
- Wire `should_protest()` — if faction grievance exceeds threshold, emit protest event, apply legitimacy damage

**Test:** Run 5000+ ticks with high stress (drought scenario), verify `Factions: > 0`.

### 1.4 Wire Collective Psychology Derivation

**Files:** `sim.rs`, `institutions.rs`

- Each tick: for each institution, gather member morale values
- Call `derive_collective_psychology()` with member data
- This feeds back into norm compliance via `inst.collective.morale`

**Test:** Verify institution dashboard shows non-default morale/unity values.

### 1.5 Wire Ecology Migration Pressure

**Files:** `sim.rs`, `ecology.rs`

- Compute `migration_pressure()` per agent each tick
- If pressure exceeds threshold: agent "migrates" (for now, teleports to a different site)
- Emit `MigrationStarted` event

**Test:** Run drought scenario, verify some agents migrate.

### 1.6 Fix Emotion Decay Rate

**Files:** `sim.rs`

- Reduce decay from `0.01` to `0.002` per tick
- This allows emotions to accumulate and create meaningful dynamics

**Test:** Run 200 ticks, verify agents show non-zero joy/fear/anger values.

### Phase 1 Deliverable

A simulation where:
- Population grows and shrinks through births and deaths
- Prices fluctuate based on supply/demand
- Agents actually trade with each other
- Factions form when grievance is high
- Protests occur and damage institutional legitimacy
- Agents migrate under resource pressure
- Emotions persist long enough to influence behavior

---

## Phase 2: Agent Spatial Positions & Movement (2–3 days)

**Goal:** Give agents a physical presence in the world. This is architecturally foundational — without it, the world is just a bag of sites.

### 2.1 Add Position Component to AgentBundle

**Files:** `sim.rs`, `person.rs`

```rust
pub struct Position {
    pub x: i32,
    pub y: i32,
}
```

- Add `position: Position` to `AgentBundle`
- Initialize agents at their home site's coordinates
- Add `fn site_position(&self, site_idx: usize) -> Option<(i32, i32)>` to `World`

### 2.2 Implement Move Action

**Files:** `actions.rs`, `sim.rs`

- Add `ActionKind::Move { target_x: i32, target_y: i32 }` variant
- Movement takes N ticks based on distance (Manhattan distance / speed)
- While moving, agent cannot perform other actions
- Add movement speed based on personality (conscientiousness = steady, impulsivity = fast but erratic)

### 2.3 Update Action Selection for Spatial Awareness

**Files:** `actions.rs`

- When selecting actions, compute distance to relevant sites
- Add distance cost to utility function: `utility -= distance_weight * manhattan_distance`
- Agents prefer nearby sites when needs are equal

### 2.4 Perception Radius

**Files:** `sim.rs`, `attention.rs`

- Agents only perceive events within a radius (e.g., 5 tiles)
- Social interactions only possible between agents within perception radius
- This creates natural neighborhoods and social clusters

### 2.5 Update TUI Map Rendering

**Files:** `mindstrata-tui/src/lib.rs`

- Show agent positions on the ASCII map (e.g., `@` for agents, numbered if multiple)
- Show agent movement trails optionally

### Phase 2 Deliverable

A simulation where:
- Agents physically exist at coordinates in the world
- Movement takes time and creates spatial dynamics
- Social interactions are proximity-based
- The ASCII map shows where agents actually are
- Natural neighborhoods emerge from spatial clustering

---

## Phase 3: Goal System & Behavioral Depth (2–3 days)

**Goal:** Make agent behavior more sophisticated than "respond to highest need."

### 3.1 Goal Source Tracking

**Files:** `person.rs`, `systems/mod.rs`

- Add `source: GoalSource` field to `Goal`
- `GoalSource` enum: `Need`, `Belief`, `Emotion`, `Identity`, `Role`, `Obligation`, `Institutional`
- Goals generated from different sources have different persistence and priority modifiers

### 3.2 Goal Rejection & Completion Lists

**Files:** `person.rs`

- Add `rejected: Vec<Goal>` and `completed: Vec<Goal>` to agent state
- When a goal is abandoned (critical need override), move to rejected list
- When a goal succeeds, move to completed list
- Agents learn from completed/rejected goals (adjust commitment for similar future goals)

### 3.3 Identity-Driven Goals

**Files:** `systems/mod.rs`, `person.rs`

- Agents with strong `Farmer` identity generate `Work` goals even when not hungry
- Agents with strong `Believer` identity generate `Worship` goals when meaning deficit is moderate (not just high)
- Agents with strong `Parent` identity generate `Help` goals for their children

### 3.4 Emotional Goal Modulation

**Files:** `systems/mod.rs`

- High anger → generate `Threaten` goals against recent antagonists
- High fear → generate `Flee` or `Rest` goals
- High joy → generate `Socialize` goals
- High shame → generate `Worship` or `Work` goals (atone)

### 3.5 Goal Persistence & Commitment

**Files:** `person.rs`

- Goals have `commitment: Fixed` that decays slowly
- High-commitment goals resist interruption by lower-priority needs
- Conscientiousness increases commitment decay resistance
- This prevents agents from flipping between actions constantly

### Phase 3 Deliverable

A simulation where:
- Agents have multi-step goals with persistence
- Identity shapes behavior (devout agents worship more, farmers work more)
- Emotions drive goal generation (angry agents seek revenge)
- Goal rejection/completion creates behavioral learning

---

## Phase 4: Economic Depth (2–3 days)

**Goal:** Make the economy a real system with supply chains, specialization, and inequality dynamics.

### 4.1 Agent-to-Agent Trade

**Files:** `actions.rs`, `market.rs`

- When agent selects `Trade`: find a nearby agent with surplus grain/water
- Execute `direct_trade()` with trust-based pricing
- Both agents emit `TradeOccurred` event
- Relationship trust increases from successful trade

### 4.2 Specialization

**Files:** `person.rs`, `actions.rs`

- Track per-agent skill levels: `farming_skill`, `trading_skill`, `crafting_skill`
- Skilled agents produce more (higher `productivity` modifier)
- Agents naturally specialize based on personality + repeated action
- Create `Skill` component on `AgentBundle`

### 4.3 Wealth Inequality Dynamics

**Files:** `market.rs`, `sim.rs`

- Agents with high `ambition` + `dominance` accumulate wealth faster
- Agents with high `altruism` share resources (reduce own wealth)
- Tax collection creates redistribution pressure
- Wealth inequality triggers grievance (fairness moral foundation)

### 4.4 Black Market

**Files:** `actions.rs`, `norms.rs`

- When grain is scarce and norm pressure is low, agents can `Steal` without punishment
- Stolen goods sold at black market (higher price, no tax)
- If caught (enforcement probability), punishment + reputation damage

### 4.5 Resource Scarcity Feedback

**Files:** `actions.rs`, `systems/mod.rs`

- When grain is scarce: `Work` utility increases (need to produce)
- When water is scarce: `Drink` utility increases dramatically
- Scarcity creates price spikes which create trade opportunities
- This creates economic oscillation

### Phase 4 Deliverable

A simulation where:
- Agents trade with each other and form economic relationships
- Specialization emerges from repeated behavior
- Wealth inequality creates social tension
- Scarcity drives economic behavior and prices
- Black markets emerge when enforcement is weak

---

## Phase 5: Social Depth & Graph Structure (2–3 days)

**Goal:** Make social dynamics richer with multiple relationship types and social processes.

### 5.1 Relationship Types

**Files:** `person.rs`, `social.rs`

- Add `kind: RelationshipKind` to `Relationship`
- Types: `Kin`, `Friend`, `Rival`, `Neighbor`, `Colleague`, `Stranger`
- Different interaction types are more likely between certain relationship kinds
- Friendship forms from repeated positive interactions
- Rivalry forms from repeated negative interactions + similar goals

### 5.2 Status & Reputation

**Files:** `person.rs`, `social.rs`

- Add `status: Fixed` to agent state (derived from wealth + institutional role + social connections)
- Reputation is observer-relative: `reputation(agent, observer)` computed from direct experience + gossip
- High-status agents are more persuasive in gossip
- Status competition creates social dynamics

### 5.3 Feud System

**Files:** `conflict.rs`, `sim.rs`

- Wire `feuds` tracking (already on `AgentBundle`) into tick loop
- After violence: both parties add each other to feuds list
- Feuding agents have increased `Threaten` probability
- Feuds decay over time (configurable)
- Feuds can involve families (kin join feuds)

### 5.4 In-Group / Out-Group Formation

**Files:** `social.rs`, `factions.rs`

- Faction membership creates in-group bias
- Faction members trust each other more, distrust outsiders
- This creates polarization dynamics
- Out-group derogation: faction members insult outsiders more

### Phase 5 Deliverable

A simulation where:
- Social relationships have types and evolve differently
- Status hierarchies emerge from wealth and institutional roles
- Feuds create persistent interpersonal conflict
- Faction membership creates social polarization

---

## Phase 6: Observability & Debugging (1–2 days)

**Goal:** Make it possible to understand *why* the simulation behaves the way it does.

### 6.1 Proposition Registry with Human-Readable Names

**Files:** `mindstrata-core/src/proposition.rs`, `mindstrata-sim/src/sim.rs`

- Replace numeric proposition IDs with named propositions
- Map: `0 → "the_market_is_fair"`, `1 → "the_council_protects_us"`, etc.
- Update belief inspector to show proposition names

### 6.2 Decision Trace Display

**Files:** `mindstrata-tui/src/lib.rs`, `mindstrata-cli/src/main.rs`

- Wire `--decisions N` flag to show decision traces for agent N
- Show each decision with influencing factors and their magnitudes
- Color-code by factor type (need vs. routine vs. norm vs. emotion)

### 6.3 Event Timeline View

**Files:** `mindstrata-tui/src/lib.rs`

- Add `--timeline` flag to show chronological event list
- Filter by event type, agent, or tick range
- Show causal chains (this event caused that event)

### 6.4 Snapshot Persistence

**Files:** `snapshot.rs`, `mindstrata-cli/src/main.rs`

- Implement `Snapshot::save(&self, path: &Path)` using `serde_json` or `postcard`
- Implement `Snapshot::load(path: &Path)` for replay
- Add `--snapshot-at N` flag to save snapshot every N ticks
- Add `--replay path` command to load and replay from snapshot

### 6.5 Metric History

**Files:** `sim.rs`

- Track per-tick metrics in a `Vec<MetricSnapshot>`
- Metrics: avg hunger, avg valence, grain supply, price, Gini, faction count, event count
- Export to CSV for analysis

### Phase 6 Deliverable

A simulation where:
- You can understand why any agent made any decision
- You can replay from any snapshot
- You can export metrics for analysis
- The belief inspector shows meaningful proposition names

---

## Phase 7: Advanced Emergence (3–5 days)

**Goal:** Achieve the architecture's highest-level emergence targets.

### 7.1 Memory Reconsolidation

- When an agent recalls a memory, the memory can change based on current emotional state
- Angry agents remember events as more negative
- This creates grievance escalation and historical distortion

### 7.2 Rumor Cascades

- Gossip about institutions spreads through the social graph
- Distorted rumors can create moral panics
- Belief in institutional legitimacy can collapse suddenly

### 7.3 Revolution Mechanism

- When faction grievance + membership exceeds threshold: revolution event
- Council legitimacy drops to zero
- Faction seizes control (new institution replaces council)
- Agents loyal to old council resist → conflict

### 7.4 Technological/Cultural Innovation

- Agents with high `openness` can discover new knowledge
- Knowledge spreads through teaching and apprenticeship
- New knowledge increases productivity (e.g., "Crop Rotation" → +20% grain)
- Cultural drift: traditions slowly change over generations

### 7.5 Epidemic Disease

- Disease spreads through proximity (requires Phase 2 spatial positions)
- Contagious diseases create panic → social withdrawal
- Quarantine as institutional response
- Disease + famine = catastrophe

### Phase 7 Deliverable

A simulation where:
- Historical narratives emerge from agent-level decisions
- Institutions rise and fall
- Cultural knowledge evolves over generations
- Epidemics create social crises
- The simulation produces genuinely surprising and coherent emergent stories

---

## Implementation Order & Dependencies

```
Phase 1 (Wire Disconnected Systems)
  ├── 1.1 Demography          [independent]
  ├── 1.2 Market              [independent]
  ├── 1.3 Factions            [depends on 1.4 for collective psychology]
  ├── 1.4 Collective Psych    [independent]
  ├── 1.5 Migration           [independent]
  └── 1.6 Emotion Decay       [independent]

Phase 2 (Spatial Positions) — depends on Phase 1.1 (demography for birth/death positioning)
  ├── 2.1 Position component  [independent]
  ├── 2.2 Move action         [depends on 2.1]
  ├── 2.3 Spatial utility     [depends on 2.1, 2.2]
  ├── 2.4 Perception radius   [depends on 2.1]
  └── 2.5 Map rendering       [depends on 2.1]

Phase 3 (Goal Depth) — independent of Phase 2
Phase 4 (Economic Depth) — depends on Phase 1.2 (market wiring)
Phase 5 (Social Depth) — depends on Phase 2 (proximity-based interactions)
Phase 6 (Observability) — independent, can run in parallel with 2-5
Phase 7 (Advanced Emergence) — depends on Phases 1-5
```

---

## Effort Estimate

| Phase | Days | Impact |
|---|---|---|
| Phase 1: Wire Disconnected Systems | 1–2 | 🔴 Transformative — existing code starts working |
| Phase 2: Spatial Positions & Movement | 2–3 | 🔴 Foundational — agents exist in a world |
| Phase 3: Goal System Depth | 2–3 | 🟡 Significant — behavior becomes sophisticated |
| Phase 4: Economic Depth | 2–3 | 🟡 Significant — economy becomes real |
| Phase 5: Social Depth | 2–3 | 🟡 Significant — relationships become rich |
| Phase 6: Observability | 1–2 | 🟢 Essential — debugging becomes possible |
| Phase 7: Advanced Emergence | 3–5 | 🔴 Transformative — genuine emergence |
| **Total** | **14–23 days** | |

---

## Quick Wins (can be done in < 1 hour each)

1. Fix emotion decay rate (0.01 → 0.002)
2. Wire `system_market()` into tick loop
3. Wire `derive_collective_psychology()` into tick loop
4. Fix proposition display to show names instead of `prop_{}`
5. Wire `age_agent()` into tick loop

These 5 changes would immediately make the simulation more alive and debuggable.

---

## Risk Areas

1. **Demography wiring** — Birth/death changes the `agents` Vec length mid-tick. Must handle carefully to avoid index invalidation.
2. **Faction formation** — Need to ensure factions don't form too easily (tuning thresholds). Start with conservative thresholds and relax.
3. **Spatial movement** — Adding positions changes the fundamental agent model. All code that references agents must be reviewed.
4. **Determinism** — Any new system must use `RngStreams` for randomness. Parallel iteration must be deterministic.
5. **Performance** — Spatial queries (nearest site, perception radius) need spatial partitioning at scale. For <100 agents, brute force is fine.
