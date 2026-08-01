# **Mindstrata**
## A Deterministic, Emergent Human-Society Simulation

The core ambition:

> A deterministic, data-oriented, emergent social-psychological-systemic simulation, rendered initially as a bare ASCII/TUI debug interface, built primarily in Rust.

The game should feel like a hybrid of:

- **The Sims**: individual psychology, needs, relationships, daily behavior
- **Cities: Skylines**: settlements, economies, infrastructure, institutions
- **Age of Empires**: historical scale, factions, conflict, technological/cultural change
- **Dwarf Fortress**: deep simulation, emergent history, agent-level detail

But its differentiator is not "building tunnels" or "city zoning."  
Its differentiator is the **ontology of simulation**:

> Every human-scale node has psychological, behavioral, relational, and systemic substrates, and history emerges from their interaction rather than from scripted events.

---

# 1. Core Design Philosophy

## 1.1 Simulation First, GUI Later

The TUI is not the game.  
The TUI is a **debug instrument**.

The simulation should be able to run headlessly:

```bash
cargo run -- --seed 42 --ticks 10000
```

The GUI/TUI only observes simulation state. It does not drive it.

This gives you:

- deterministic replay
- easier debugging
- faster iteration
- cleaner architecture
- future modding support
- possible server/client separation later

---

## 1.2 Emergence Through First Principles

Do not script high-level events directly.

Instead of:

```rust
if unrest > 0.8 {
    spawn_revolt();
}
```

Simulate the underlying causes:

- need pressure
- emotional appraisal
- trust erosion
- resource scarcity
- institutional legitimacy
- social influence
- collective identity formation
- norm internalization
- rumor propagation

Then revolt becomes an **emergent possibility**, not a hardcoded event.

---

## 1.3 Agents Are Not Omniscient

Agents should not know the global world state.

An agent knows:

- its body
- its needs
- its emotions
- its memories
- its relationships
- its local perception
- its institutional memberships
- rumors it has heard
- messages it has received
- its beliefs, which may be false

This is crucial for believable emergence.

Misinformation, gossip, panic, prejudice, factional myths, and institutional failure all require agents to have **partial, distorted, local knowledge**.

---

## 1.4 Determinism Is Non-Negotiable

The simulation should be reproducible from:

- initial world seed
- scenario definition
- input command log

This is essential for debugging emergent behavior.

If the same seed produces different outcomes, you will not be able to understand why your simulation behaves the way it does.

---

# 2. Language Choice

## 2.1 Rust Is Recommended

Rust is a very strong fit because you need:

- performance
- memory safety
- strong typing
- data-oriented design
- headless simulation capability
- good tooling
- long-term maintainability
- safe parallelism later
- excellent testing ecosystem

The main risks with Rust are:

- compile times
- overengineering abstractions
- fighting the borrow checker if you design too much like inheritance-based OOP
- slower early prototyping than Python/C#

But for this project, Rust is probably the best long-term choice.

---

## 2.2 Recommended Rust Stack

Use Rust, but avoid building a full game engine.

### Core

| Purpose | Crate |
|---|---|
| ECS | `bevy_ecs` or `shipyard` |
| Serialization | `serde` |
| Data files | `ron` |
| Randomness | `rand`, `rand_chacha` |
| Logging/tracing | `tracing`, `tracing-subscriber` |
| CLI | `clap` |
| Error handling | `thiserror`, `anyhow` |
| Graph structures | `petgraph` |
| Profiling | `tracing`, `criterion`, optionally `samply` |
| TUI | `ratatui`, `crossterm` |

### Optional

| Purpose | Crate |
|---|---|
| Persistence | `postcard`, `bincode`, `redb`, `sqlite` |
| Parallelism | `rayon`, `bevy_tasks` |
| Testing | `proptest`, `insta` |

I recommend starting with **Bevy ECS as a library**, not full Bevy.

You can use its ECS, schedules, commands, and resources without rendering.

A custom ECS is possible, but not recommended for the first version unless you have a strong reason.

---

# 3. High-Level Architecture

The architecture should be layered.

```text
+---------------------------------------------------------------+
|                    TUI / Debug Instrument                     |
|   map, agent inspector, event log, decision trace, metrics    |
+---------------------------------------------------------------+
|                    Scenario / Experiment Layer                |
|      seeds, shocks, scenarios, assertions, mod definitions    |
+---------------------------------------------------------------+
|                    Simulation Orchestrator                    |
|        fixed tick loop, schedules, replay, snapshots          |
+---------------------------------------------------------------+
|                    Causal Provenance Layer                    |
|        event journal, decision traces, causal chains          |
+---------------------------------------------------------------+
|                    Systemic Layer                             |
|  institutions, law, economy, ecology, governance, conflict    |
+---------------------------------------------------------------+
|                    Social Layer                               |
| relationships, groups, factions, gossip, reputation, norms    |
+---------------------------------------------------------------+
|                    Epistemic Layer                            |
|     perception, attention, beliefs, rumors, information       |
+---------------------------------------------------------------+
|                    Psychological Layer                        |
| needs, traits, emotion, memory, identity, goals, appraisal    |
+---------------------------------------------------------------+
|                    Behavioral Layer                           |
| affordances, utility AI, planning, habits, action execution   |
+---------------------------------------------------------------+
|                    World Layer                                |
| space, regions, resources, sites, objects, time, ecology      |
+---------------------------------------------------------------+
|                    Core Kernel                                |
| ECS, RNG, fixed-point math, commands, persistence, events     |
+---------------------------------------------------------------+
```

The key principle:

> The GUI observes the simulation. It does not drive it.

---

# 4. The Five Substrates (Refined Ontology)

You said each human-scale node should have:

- psychological
- behavioral
- relational
- systemic

This is the heart of the architecture.

A "human-scale node" can be:

- a person
- a household
- a firm
- a faction
- a temple
- a guild
- a council
- a settlement
- an institution
- an informal movement

But these are not all minds in the same way.

A **person** has a full psychological substrate.

A **household**, **firm**, **faction**, or **institution** has a **derived collective psychological substrate**: morale, cohesion, legitimacy, grievance, trust, ideological rigidity, etc.

So the substrates apply differently depending on entity type.

---

## 4.1 Psychological Substrate

This is the internal state of a mind.

For persons, it includes:

### Biological / Need State

Examples:

- hunger
- thirst
- fatigue
- comfort
- safety
- health
- arousal
- stress

### Psychological Needs

Examples:

- belonging
- esteem
- autonomy
- mastery
- meaning
- stimulation
- predictability
- fairness

### Personality

Use a small trait model. Do not overdo it initially.

Possible traits:

- openness
- conscientiousness
- extraversion
- agreeableness
- neuroticism
- risk tolerance
- conformity
- ambition
- altruism
- traditionalism
- dominance
- impulsivity

### Emotional State

Use both:

#### Dimensional emotion

- valence
- arousal
- potency/control

#### Discrete emotions

- fear
- anger
- joy
- sadness
- disgust
- shame
- pride
- guilt
- trust
- contempt
- gratitude
- envy

Emotions should arise from appraisal, not be directly assigned.

Example:

```text
event: lost food
appraisal:
  goal_relevance = high
  coping_potential = low
  fairness = violated
  agency = other
=> anger + sadness
```

### Beliefs

Agents need a belief model.

Examples:

- "The market is fair."
- "The ruler is legitimate."
- "My neighbor can be trusted."
- "Foreigners are dangerous."
- "Hard work leads to wealth."
- "The harvest will fail."

Each belief has:

- proposition
- confidence
- emotional charge
- source
- last reinforced
- resistance to change
- identity linkage
- social reinforcement count

### Memory

Memory should be selective, not a perfect log.

Types:

- episodic memory
- semantic memory
- procedural memory
- social memory
- emotional memory

Each memory has:

- salience
- age
- emotional intensity
- rehearsal count
- distortion level
- associated entities
- tags

Important: agents should forget, distort, and ruminate.

### Goals

Goals emerge from:

- needs
- beliefs
- emotions
- relationships
- roles
- institutional obligations

Examples:

- get food
- protect family
- gain status
- obey law
- punish betrayer
- accumulate wealth
- spread faith
- reform institution

### Identity

Identity is crucial for systemic emergence.

Components:

- personal identity
- group identities
- role identities
- moral identity
- narrative identity

Example identities:

- farmer
- parent
- citizen
- believer
- worker
- rebel
- merchant
- outsider

Identity affects utility functions.

An agent who identifies strongly as a devout believer may accept actions that a purely self-interested agent would reject.

---

## 4.2 Behavioral Substrate

This is how psychology becomes action.

The behavioral layer should not be a giant state machine.

It should combine:

- utility-based action selection
- hierarchical task decomposition
- affordance-based interaction
- habit and routine
- interruption by emotion/urgency
- bounded rationality

### Action Primitives

Examples:

```text
move
wait
rest
eat
drink
work
produce
acquire
consume
exchange
talk
persuade
threaten
help
steal
flee
attack
worship
vote
organize
migrate
hoard
share
```

### Affordances

Objects and entities expose possible actions.

Example:

```text
Well:
  affordances:
    - drink
    - fetch_water
    - gossip_near
    - contaminate

Market:
  affordances:
    - buy
    - sell
    - loiter
    - protest
    - advertise

Person:
  affordances:
    - talk
    - trade
    - threaten
    - bond
    - accuse
    - follow
```

This is important for emergence.

The world should not be a collection of hardcoded interactions.  
It should be a field of possible actions.

### Utility Function

A simplified model:

```text
utility(action) =
    need_pressure_weight * need_relief_estimate
  + emotional_weight * emotional_relief_estimate
  + social_weight * relationship_effect
  + systemic_weight * institutional_alignment
  + identity_weight * identity_congruence
  + novelty_weight * stimulation_value
  - cost_weight * expected_cost
  - risk_weight * expected_risk
  + noise
```

The noise term is important. Humans are not perfectly rational.

### Bounded Rationality

Agents should not optimize perfectly.

Limit them by:

- perception radius
- memory capacity
- attention
- misinformation
- emotional distortion
- social trust
- cognitive fatigue
- routine inertia

This will make emergence more believable.

---

## 4.3 Relational Substrate

This models relationships between agents.

Relationships should be first-class entities.

Example relationship types:

- kin
- friend
- rival
- enemy
- spouse
- parent-child
- employer-employee
- ruler-subject
- creditor-debtor
- ally
- co-religionist
- faction member
- neighbor
- stranger

Each relationship has:

- trust
- affection
- respect
- fear
- obligation
- intimacy
- power asymmetry
- shared history
- emotional valence
- reciprocity balance
- last interaction
- relationship labels

Example:

```rust
struct Relationship {
    from: Entity,
    to: Entity,
    kind: RelationshipKind,
    trust: f32,
    affection: f32,
    respect: f32,
    fear: f32,
    obligation: f32,
    power_balance: f32,
    shared_identity: f32,
    last_interaction_tick: u64,
}
```

### Social Graph

You need a social graph, but do not make it a single monolithic graph.

Use multiple graphs:

- kinship graph
- friendship graph
- economic exchange graph
- political allegiance graph
- religious affiliation graph
- rumor propagation graph
- organizational membership graph

This avoids a common mistake: treating "society" as one relationship type.

### Social Processes

Simulate:

- bonding
- betrayal
- gossip
- reputation
- forgiveness
- grudges
- reciprocity
- conformity
- persuasion
- status competition
- coalition formation
- polarization
- in-group/out-group formation

These are essential for emergent history.

---

## 4.4 Systemic Substrate

This is the most ambitious part.

You want nested systems within systems within systems.

Examples:

```text
person
  -> household
    -> neighborhood
      -> settlement
        -> province
          -> polity
            -> civilization
              -> ecological system
                -> climate system
```

But also overlapping systems:

```text
person
  -> firm
    -> market
      -> trade network
        -> economic sphere

person
  -> temple
    -> religion
      -> ideological sphere

person
  -> gang
    -> faction
      -> political conflict system
```

So do not model systems as only a tree.

Model them as:

- hierarchies
- networks
- markets
- ecologies
- institutions
- normative orders

### System Entities

Systems themselves should be entities.

Examples:

- Household
- Firm
- Guild
- Temple
- Army
- Faction
- Market
- LegalSystem
- Settlement
- Region
- Ecosystem
- Ideology
- Economy

Each system can have:

- members
- roles
- resources
- treasury
- legitimacy
- cohesion
- norms
- policies
- goals
- memory
- symbols
- territory
- boundaries
- internal communication capacity

Example:

```rust
struct Institution {
    kind: InstitutionKind,
    legitimacy: f32,
    cohesion: f32,
    resources: ResourceLedger,
    norms: Vec<NormId>,
    roles: Vec<RoleId>,
    parent: Option<Entity>,
    territory: Option<Entity>,
}
```

### Collective Psychology

A household, faction, or institution can have a derived psychological substrate.

Not a literal mind, but a collective state:

- morale
- identity strength
- trust
- grievance
- ideological rigidity
- legitimacy
- fear
- ambition
- cohesion
- collective memory

This should be derived from members and institutional history.

Example:

```text
faction.morale =
    average(member.morale)
  + leadership_bonus
  + recent_victory_bonus
  - internal_conflict_penalty
  - resource_scarcity_penalty
```

### Nested System Rules

Systems affect agents through:

- roles
- obligations
- permissions
- prohibitions
- taxes
- wages
- rituals
- laws
- norms
- sanctions
- information channels

Example norm:

```text
Norm:
  id: no_theft
  scope: settlement
  strength: 0.8
  punishment: fine
  legitimacy: 0.7
```

Agents internalize norms to varying degrees based on:

- conformity
- identity
- fear of punishment
- belief in legitimacy
- peer behavior

This allows emergence of:

- compliance
- corruption
- rebellion
- moral panic
- institutional decay
- reform movements

---

## 4.5 Epistemic / Informational Substrate (Cross-Cutting Layer)

This is not necessarily a fifth substrate in the philosophical sense, but architecturally it is essential.

You need a strict separation between:

1. **What is true in the world** (Objective World State)
2. **What an agent believes is true** (Subjective Epistemic State)
3. **What a group believes is true** (Social Facts)
4. **What an institution officially records or claims** (Institutional Facts)
5. **What actually caused what** (Causal Provenance)

Without this, you cannot get believable emergence for:

- rumors
- misinformation
- panic
- prejudice
- factional myths
- corruption
- institutional failure
- legitimacy collapse
- moral panics
- false accusations
- political radicalization

So the refined ontology should be:

```text
Objective World State
  - entities
  - resources
  - positions
  - events
  - ecological facts
  - economic facts

Subjective Epistemic State
  - perceptions
  - memories
  - beliefs
  - rumors
  - expectations
  - misbeliefs

Social Facts
  - relationships
  - obligations
  - debts
  - promises
  - offenses
  - reputations
  - status claims

Institutional Facts
  - roles
  - offices
  - laws
  - policies
  - norms
  - legitimacy
  - official records
  - enforcement capacity

Causal Provenance
  - event chains
  - decision traces
  - belief-update traces
  - institutional decision traces
  - replayable causality
```

This is the real differentiator of your project.

Not "ASCII Dwarf Fortress without tunnels."

Instead:

> A deterministic simulation of human societies where history emerges from locally bounded minds, social relationships, institutional processes, and material constraints.

---

# 5. Entity Model

Use ECS. Avoid deep object inheritance.

## 5.1 Core Entities

### Person

```rust
struct Person {
    id: Entity,
    name: String,
    age: f32,
    sex: Sex,
    culture: CultureId,
    birth_tick: u64,
}
```

### Psychological Components

```rust
struct BodyState {
    health: Fixed,
    energy: Fixed,
    hunger: Fixed,
    thirst: Fixed,
    fatigue: Fixed,
    sickness: Fixed,
    injury: Fixed,
    age: Fixed,
    fertility: Option<Fixed>,
}

struct NeedState {
    hunger_deficit: Fixed,
    thirst_deficit: Fixed,
    fatigue_deficit: Fixed,
    safety_deficit: Fixed,
    social_deficit: Fixed,
    esteem_deficit: Fixed,
    autonomy_deficit: Fixed,
    meaning_deficit: Fixed,
    fairness_deficit: Fixed,
    predictability_deficit: Fixed,
}

struct Personality {
    openness: Fixed,
    conscientiousness: Fixed,
    extraversion: Fixed,
    agreeableness: Fixed,
    neuroticism: Fixed,
    risk_tolerance: Fixed,
    conformity: Fixed,
    ambition: Fixed,
    altruism: Fixed,
    traditionalism: Fixed,
    dominance: Fixed,
    impulsivity: Fixed,
}

struct Affect {
    valence: Fixed,
    arousal: Fixed,
    control: Fixed,
}

struct DiscreteEmotions {
    fear: Fixed,
    anger: Fixed,
    joy: Fixed,
    sadness: Fixed,
    disgust: Fixed,
    shame: Fixed,
    guilt: Fixed,
    pride: Fixed,
    trust: Fixed,
    contempt: Fixed,
    gratitude: Fixed,
    envy: Fixed,
}

struct Appraisal {
    goal_relevance: Fixed,
    goal_congruence: Fixed,
    coping_potential: Fixed,
    expectedness: Fixed,
    fairness: Fixed,
    agency: AgencyAttribution,
    social_visibility: Fixed,
    identity_relevance: Fixed,
    uncertainty: Fixed,
}

struct AttentionState {
    focus: Option<Entity>,
    attention_budget: Fixed,
    habituation: HashMap<StimulusKind, Fixed>,
    salience_bias: Fixed,
}

struct Memory {
    episodic: Vec<EpisodicMemory>,
    semantic: Vec<SemanticMemory>,
    procedural: Vec<ProceduralMemory>,
    social: Vec<SocialMemory>,
    emotional: Vec<EmotionalMemory>,
    capacity: usize,
}

struct Beliefs {
    entries: Vec<Belief>,
}

struct Belief {
    proposition: PropositionId,
    confidence: Fixed,
    emotional_charge: Fixed,
    identity_linkage: Fixed,
    resistance: Fixed,
    source_tags: Vec<SourceId>,
    last_reinforced_tick: u64,
    social_reinforcement: Fixed,
    contradiction_set: Vec<PropositionId>,
}

struct Goals {
    active: Vec<Goal>,
    rejected: Vec<Goal>,
    completed: Vec<Goal>,
}

struct Goal {
    kind: GoalKind,
    source: GoalSource,
    priority: Fixed,
    commitment: Fixed,
    horizon: TimeHorizon,
    success_criteria: SuccessCriteria,
    created_tick: u64,
}

struct IdentityState {
    personal_identities: Vec<IdentityId>,
    group_identities: Vec<Entity>,
    role_identities: Vec<RoleId>,
    moral_identities: Vec<MoralIdentityId>,
    narrative_tags: Vec<NarrativeTagId>,
    identity_strength: HashMap<IdentityId, Fixed>,
}

struct MoralValues {
    care: Fixed,
    fairness: Fixed,
    loyalty: Fixed,
    authority: Fixed,
    purity: Fixed,
    liberty: Fixed,
}

struct CognitiveState {
    attention_capacity: Fixed,
    executive_capacity: Fixed,
    fatigue: Fixed,
    stress: Fixed,
    rumination: Fixed,
    planning_horizon: Fixed,
    heuristic_bias: Fixed,
}
```

### Behavioral Components

```rust
struct ActionState {
    current_action: Option<ActionId>,
    progress: f32,
    interrupted: bool,
}

struct Skills {
    values: HashMap<SkillId, f32>,
}

struct Habits {
    routines: Vec<Habit>,
}

struct Intention {
    goal: GoalId,
    plan: PlanId,
    current_step: usize,
    commitment: Fixed,
    started_tick: u64,
}
```

### Relational Components

```rust
struct SocialIdentity {
    groups: Vec<Entity>,
    roles: Vec<RoleId>,
    status: f32,
    reputation: f32,
}

struct Relationship {
    from: Entity,
    to: Entity,
    kind: RelationshipKind,
    trust: Fixed,
    affection: Fixed,
    respect: Fixed,
    fear: Fixed,
    obligation: Fixed,
    intimacy: Fixed,
    grievance: Fixed,
    gratitude: Fixed,
    power_balance: Fixed,
    shared_identity: Fixed,
    reciprocity_balance: Fixed,
    last_interaction_tick: u64,
}

struct SocialFact {
    id: SocialFactId,
    kind: SocialFactKind,
    from: Entity,
    to: Entity,
    created_tick: u64,
    strength: Fixed,
    fulfilled: bool,
    witnesses: Vec<Entity>,
    associated_events: Vec<EventId>,
}
```

### Systemic Components

```rust
struct Membership {
    member: Entity,
    organization: Entity,
    role: RoleId,
    rank: f32,
    joined_tick: u64,
}

struct Institution {
    kind: InstitutionKind,
    legitimacy: Fixed,
    cohesion: Fixed,
    treasury: ResourceLedger,
    norms: Vec<NormId>,
    roles: Vec<RoleId>,
    parent: Option<Entity>,
    territory: Option<Entity>,
    offices: Vec<OfficeId>,
    policies: Vec<PolicyId>,
    laws: Vec<LawId>,
    corruption: Fixed,
    enforcement_capacity: Fixed,
    communication_capacity: Fixed,
    information_quality: Fixed,
    decision_procedure: DecisionProcedure,
    official_records: Vec<OfficialRecord>,
}

struct Office {
    id: OfficeId,
    institution: Entity,
    title: String,
    holder: Option<Entity>,
    authority: Vec<AuthorityKind>,
    obligations: Vec<ObligationId>,
    term_length: Option<u64>,
    legitimacy: Fixed,
}

struct Settlement {
    name: String,
    population: u32,
    region: Entity,
    economy: Entity,
    government: Option<Entity>,
}

struct Faction {
    collective_identity: IdentityId,
    leadership: Vec<Entity>,
    cohesion: Fixed,
    grievance: Fixed,
    mobilization_capacity: Fixed,
    communication_network: Vec<Entity>,
}
```

---

# 6. World Model

You do not need tunnel building, but you do need space.

## 6.1 Spatial Representation

Start with a simple 2D grid.

```rust
struct Tile {
    terrain: Terrain,
    fertility: Fixed,
    moisture: Fixed,
    temperature: Fixed,
    resource_stock: Option<ResourceStock>,
    depletion: Fixed,
    disease_pressure: Fixed,
    owner: Option<Entity>,
    site: Option<Entity>,
}
```

Later you can add:

- regions
- biomes
- watersheds
- climate cells
- travel networks
- administrative boundaries

Do not overbuild geography initially.

---

## 6.2 Sites

Sites are meaningful locations.

Examples:

- house
- farm
- well
- market
- temple
- barracks
- workshop
- square
- prison
- school
- shrine
- mine
- port

Sites provide affordances.

```rust
struct Site {
    kind: SiteKind,
    owner: Option<Entity>,
    controller: Option<Entity>,
    capacity: u32,
    affordances: Vec<Affordance>,
}
```

---

# 7. Simulation Loop

Use a fixed timestep.

Example:

```text
1 tick = 15 minutes
1 day = 96 ticks
1 year = 35,040 ticks
```

But you may want multiple schedules:

```text
micro tick:
  perception, immediate action

standard tick:
  needs, movement, interaction

daily tick:
  routines, household accounting

weekly tick:
  markets, institutions

monthly tick:
  demographics, ecology, policy

yearly tick:
  climate, culture drift, technology
```

---

## 7.1 Loop Structure

```rust
loop {
    advance_clock();
    process_input_commands();

    run_perception_systems();
    run_psychological_update_systems();
    run_goal_selection_systems();
    run_behavior_planning_systems();
    run_action_execution_systems();
    run_interaction_systems();

    run_social_update_systems();
    run_economic_systems();
    run_institutional_systems();
    run_ecological_systems();

    run_event_generation_systems();

    flush_commands();
    emit_snapshot_if_needed();
    render_tui_if_enabled();
}
```

Important:

> Keep system order explicit.

Emergence does not mean random execution order.  
Emergence requires stable causal order.

---

# 8. Event Architecture

Events are crucial.

The simulation should produce events such as:

```rust
enum SimEvent {
    AgentAte {
        agent: Entity,
        food: Entity,
    },
    AgentFailedAction {
        agent: Entity,
        action: ActionId,
    },
    RelationshipChanged {
        from: Entity,
        to: Entity,
        delta: RelationshipDelta,
    },
    TradeOccurred {
        buyer: Entity,
        seller: Entity,
        good: ResourceId,
        price: f32,
    },
    NormViolated {
        agent: Entity,
        norm: NormId,
        witnesses: Vec<Entity>,
    },
    InstitutionChangedPolicy {
        institution: Entity,
        policy: PolicyId,
    },
    FactionFormed {
        faction: Entity,
        founders: Vec<Entity>,
    },
    RumorSpread {
        source: Entity,
        target: Entity,
        rumor: RumorId,
    },
    MigrationStarted {
        agent: Entity,
        from: Entity,
        to: Entity,
    },
    EmotionalShift {
        agent: Entity,
        from: EmotionVector,
        to: EmotionVector,
    },
}
```

Events should be used for:

- memory formation
- emotional appraisal
- gossip
- institutional response
- narrative logging
- analytics
- debugging
- replay

Do not use events only as UI notifications.

Events are part of the simulation's causal memory.

---

# 9. Psychological Model in More Detail

Use a layered cognitive architecture.

```text
Perception
  -> Attention
    -> Appraisal
      -> Emotion
        -> Need Pressure
          -> Goal Formation
            -> Action Selection
              -> Execution
                -> Feedback
```

---

## 9.1 Needs

Use continuous values.

Example:

```rust
struct NeedState {
    deficit: Fixed,
    pressure: Fixed,
    satiation_rate: Fixed,
    decay_rate: Fixed,
}
```

Need pressure should not equal deficit directly.

Use:

```text
pressure = deficit^exponent * trait_modifier * context_modifier
```

For example:

```text
hunger_pressure =
    hunger_deficit^1.5
  * personality.impulsivity
  * food_visibility
  * social_context
```

---

## 9.2 Appraisal

Events are appraised along dimensions:

- goal relevance
- goal congruence
- coping potential
- expectedness
- fairness
- agency
- social visibility
- identity relevance

Example:

```rust
struct Appraisal {
    goal_relevance: Fixed,
    goal_congruence: Fixed,
    coping_potential: Fixed,
    expectedness: Fixed,
    fairness: Fixed,
    agency: AgencyAttribution,
    social_visibility: Fixed,
    identity_relevance: Fixed,
}
```

Then emotions are derived:

```text
if goal_relevance high:
  if goal_congruence positive:
    joy/pride/relief
  else:
    if agency == self:
      guilt/shame
    if agency == other:
      anger/contempt
    if agency == circumstance:
      sadness/fear
```

This gives you psychologically grounded emergence.

---

## 9.3 Belief Updating

Use a simple Bayesian-ish model.

```text
new_confidence =
    old_confidence * resistance
  + evidence_strength * source_trust
  + emotional_reinforcement
  + social_reinforcement
```

Normalize/clamp to `[0, 1]`.

Important: beliefs should resist change when:

- identity-linked
- emotionally charged
- socially shared
- repeatedly reinforced
- tied to high-status sources

This allows ideology, rumor, and polarization to emerge.

---

# 10. Behavioral Model in More Detail

Avoid hardcoded AI trees.

Use a hybrid.

---

## 10.1 Utility AI

For immediate action selection.

```text
candidate_actions = perceive_affordances(agent)
scored_actions = score(agent, candidate_actions)
chosen_action = select(scored_actions, personality, emotion, noise)
```

---

## 10.2 Hierarchical Task Network

For longer goals.

Example:

```text
Goal: feed_self
  -> acquire_food
    -> go_to_market
    -> buy_food
  -> prepare_food
  -> eat_food
```

If a subtask fails, replan.

---

## 10.3 Routines

Agents should have daily routines.

Example:

```text
06:00 wake
07:00 eat
08:00 work
12:00 socialize/eat
13:00 work
18:00 return home
20:00 socialize/worship/rest
22:00 sleep
```

Routines create stability.

Emergence happens when routines are disrupted by:

- hunger
- fear
- opportunity
- conflict
- institutional demands
- relationship events

---

# 11. Relational Model in More Detail

Relationships should update through interaction.

Example interaction:

```text
A helps B
```

Effects:

```text
B.trust(A) += help_value * B.trust_openness
B.obligation(A) += help_value
A.relationship_value(B) += B.response
```

If B does not reciprocate later:

```text
A.feelings(B) -= resentment
```

---

## 11.1 Reputation

Reputation is not global. It is local and networked.

```text
reputation(agent, observer) =
    direct_experience
  + trusted_gossip
  - rumor_distortion
  + institutional_labeling
```

---

## 11.2 Gossip

Gossip should mutate information.

```text
rumor_accuracy =
    source_memory_accuracy
  * source_trust
  * transmission_fidelity
  * emotional_salience_bias
  * identity_bias
```

This can produce:

- moral panics
- false accusations
- prestige
- scapegoating
- factional myths

---

# 12. Systemic Model in More Detail

This is where your game can become truly distinctive.

---

## 12.1 Systems as Entities

Do not treat "economy" as a global calculator.

Instead:

- markets are entities
- firms are entities
- households are entities
- governments are entities
- religions are entities
- ecosystems are entities

They interact through explicit transactions.

---

## 12.2 Nested Systems

Use multiple nested graphs.

### Political Hierarchy

```text
household
neighborhood
village
town
province
kingdom
empire
```

### Economic Network

```text
producer
market
merchant
guild
trade_route
regional_economy
world_market
```

### Cultural Network

```text
person
community
tradition
religion
ideology
civilizational_sphere
```

### Ecological System

```text
field
watershed
biome
climate_zone
```

An agent can belong to many systems simultaneously.

---

## 12.3 Institutional Norms

Norms should be data.

Example:

```text
Norm(
    id: "private_property",
    scope: Settlement,
    strength: 0.8,
    internalization_bias: 0.6,
    violation_cost: Fine(100),
    enforcement: Guards,
    legitimacy_dependency: true,
)
```

Agents evaluate norm compliance:

```text
norm_pressure =
    norm.strength
  * agent.conformity
  * agent.identity_alignment
  * fear_of_punishment
  * peer_compliance_rate
  * institutional_legitimacy
```

This allows corruption and rebellion to emerge naturally.

---

## 12.4 Legitimacy

Legitimacy is a core systemic variable.

```text
legitimacy =
    procedural_fairness
  + outcome_satisfaction
  + traditional_authority
  + charismatic_authority
  + institutional_memory
  - corruption
  - repression
  - unmet_expectations
```

When legitimacy collapses, agents may:

- evade taxes
- ignore laws
- form factions
- create alternative institutions
- migrate
- revolt

---

# 13. Emergence Targets

You should design for specific emergent phenomena.

Do not just say "emergence." Define what kinds of emergence you want.

---

## 13.1 Individual Emergence

- habits
- addiction
- trauma
- ambition
- depression
- radicalization
- skill development
- moral change

---

## 13.2 Social Emergence

- friendships
- rivalries
- families
- cliques
- gossip networks
- status hierarchies
- marriage markets
- feuds
- patron-client relations

---

## 13.3 Economic Emergence

- price formation
- scarcity hoarding
- inequality
- specialization
- migration due to wages
- black markets
- debt spirals
- firm formation
- trade routes

---

## 13.4 Political Emergence

- factions
- protests
- coups
- corruption
- reform movements
- legitimacy crises
- state formation
- institutional decay

---

## 13.5 Cultural Emergence

- rumors
- fashions
- taboos
- religious movements
- ideological polarization
- collective memory
- stigma
- prestige norms

---

## 13.6 Ecological Emergence

- overfarming
- resource depletion
- famine
- migration due to climate
- disease spread
- settlement collapse

If your primitives are correct, these should not require dedicated "story events."

---

# 14. Determinism and Debugging

This is non-negotiable for a simulation like this.

---

## 14.1 Deterministic Requirements

You need:

- fixed timestep
- seeded RNG
- stable iteration order
- deterministic floating point behavior
- replay log
- snapshot system
- debug tracing
- entity inspector
- event timeline

---

## 14.2 RNG Discipline

Do not use one global RNG everywhere.

Use separate RNG streams:

```rust
struct RngStreams {
    world: ChaCha8Rng,
    psychology: ChaCha8Rng,
    behavior: ChaCha8Rng,
    social: ChaCha8Rng,
    economy: ChaCha8Rng,
    ecology: ChaCha8Rng,
    narrative: ChaCha8Rng,
}
```

This makes debugging easier.

---

## 14.3 Avoid Nondeterminism

Be careful with:

- `HashMap` iteration order
- parallel reductions
- floating point accumulation order
- OS time
- filesystem ordering
- multithreaded work stealing

If you use parallelism, make the final aggregation deterministic.

For critical simulation values, consider using fixed-point integers instead of floats if cross-platform deterministic replay becomes difficult.

Example:

```text
0.0..=1.0 represented as 0..=10_000
```

This can make many psychological and economic quantities more deterministic and easier to debug.

---

# 15. Performance Strategy

Rust will help, but architecture matters more.

---

## 15.1 Use Data-Oriented Design

Prefer:

- components as plain data
- systems operating on queries
- structure-of-arrays
- batch processing
- cache-friendly iteration

Avoid:

- deep trait-object hierarchies in hot paths
- per-entity dynamic dispatch
- excessive `Rc<RefCell<T>>`
- global mutable state

---

## 15.2 Level of Detail

Not every agent needs full simulation every tick.

Use LOD:

```text
focus agents:
  full psychology, full behavior, full perception

nearby agents:
  simplified psychology, routine behavior

distant agents:
  aggregate statistics, probabilistic behavior
```

Example:

```text
village population = 500
fully simulated = 50
routine-simulated = 450

kingdom population = 100,000
aggregate cohorts = 200
```

This is how you scale.

---

## 15.3 Spatial Partitioning

Use chunks/regions.

```text
world
  region
    chunk
      tile
```

Agents update based on local chunk.

---

## 15.4 Social Graph Sharding

For relationships, shard by:

- household
- settlement
- faction
- workplace

Do not query the entire social graph every tick.

---

# 16. Persistence

You need two things.

---

## 16.1 Snapshots

Full world state at a tick.

Use:

- `serde`
- `postcard` or `bincode`

Example:

```text
/saves/world_seed_123_tick_10000.snapshot
```

---

## 16.2 Event Journal

Append-only log of important events.

Example:

```text
tick 102: Agent(34) ate Food(98)
tick 104: Agent(12) insulted Agent(34)
tick 105: Relationship(12->34).trust -= 0.08
tick 120: Agent(34) spread Rumor(7) to Agent(22)
```

This is invaluable for debugging emergence.

---

# 17. TUI Design

Keep it minimal.

The TUI should be a simulation debugger.

---

## 17.1 Main Views

### ASCII World Map

```text
. . F F . . W W
. H H F . . W W
. H H . M . . .
. . . . M . T .
. . R R . . T .
```

Legend:

```text
H = house
F = farm
W = water
M = market
T = temple
R = road
. = terrain
```

---

### Agent List

```text
ID   Name    Age  Mood   Needs       Goal
12   Anna    34   +0.2   H- T+ S-    work
13   Bran    41   -0.4   H+ T- S+    find_food
14   Cara    22   +0.1   H= T+ S=    socialize
```

---

### Agent Inspector

```text
Agent: Anna
Age: 34
Role: farmer
Household: 7
Faction: none

Needs:
  hunger: 0.32
  fatigue: 0.51
  safety: 0.12
  social: 0.68

Emotions:
  joy: 0.21
  fear: 0.05
  anger: 0.11

Goals:
  1. finish_work
  2. eat
  3. visit_friend

Beliefs:
  market_is_fair: 0.62
  neighbor_trustworthy: 0.77
  ruler_legitimate: 0.41
```

---

### Relationship View

```text
Anna -> Bran
  trust: 0.71
  affection: 0.62
  respect: 0.48
  obligation: 0.22

Bran -> Anna
  trust: 0.55
  affection: 0.49
  respect: 0.61
  obligation: 0.35
```

---

### Event Log

```text
[1024] Anna bought grain from Bran for 12 coins
[1025] Bran felt respected
[1026] Cara heard rumor: market_is_unfair
[1027] Cara trust in market -0.04
```

---

### System Dashboard

```text
Settlement: Riverford
Population: 214
Food: 812
Morale: 0.58
Legitimacy: 0.49
Inequality: 0.37
Unrest: 0.22
Market price grain: 13.2
```

The TUI should not be the game. It should be an instrument panel.

---

# 18. Recommended Repository Structure

Use a Cargo workspace.

```text
project/
  Cargo.toml
  crates/
    mindstrata-core/
    mindstrata-sim/
    mindstrata-cli/
    mindstrata-tui/
    mindstrata-tests/
```

---

# 19. What to Keep, Expand, Contract, Remove, and Add

## 19.1 Keep

| Existing idea | Verdict | Reason |
|---|---|---|
| Simulation first, GUI later | Keep | Correct. The TUI should remain a debugger. |
| Deterministic replay | Keep | Non-negotiable for debugging emergence. |
| Rust | Keep | Strong fit for performance, safety, and long-term maintainability. |
| ECS | Keep | Correct for data-oriented simulation. |
| Agents are not omniscient | Keep | Essential. |
| Emergence from first principles | Keep | Correct design philosophy. |
| Four substrates | Keep | Good conceptual decomposition. |
| Institutions as entities | Keep | Very important. |
| Relationships as first-class entities | Keep | Correct. |
| Events as causal memory | Keep | Essential. |
| Data-driven definitions | Keep | Necessary for modding and iteration. |
| Bare ASCII/TUI | Keep | Correct priority. |

---

## 19.2 Expand

| Existing segment | What to expand |
|---|---|
| Psychological substrate | Turn it into a full cognitive pipeline: perception → attention → appraisal → emotion → belief update → identity/norm pressure → goal formation → intention. |
| Beliefs | Add provenance, source trust, identity linkage, emotional charge, contradiction, resistance, and social reinforcement. |
| Memory | Add forgetting, distortion, rehearsal, reconsolidation, emotional bias, and memory capacity. |
| Behavioral layer | Add action grammar, bounded planning, habits, routines, interruptions, and intention commitment. |
| Relational layer | Add obligations, debts, promises, offenses, contracts, kinship, status, and reputation as observer-relative belief. |
| Systemic layer | Add institutional decision-making, offices, bureaucracy, enforcement, law, property, legitimacy dynamics, and principal-agent problems. |
| Determinism | Add fixed-point math, stable iteration order, deterministic RNG streams, and replay hashing. |
| Debugging | Add decision traces, causal chains, belief-update traces, and entity timelines. |
| Testing | Add golden replays, statistical emergence tests, and scenario assertions. |

---

## 19.3 Contract

| Existing segment | What to contract |
|---|---|
| Crate structure | Do not start with 16 crates. Start with fewer crates and split later. |
| Personality traits | Start with a small trait set, not a full psychology textbook. |
| Emotions | Start with dimensional affect plus a small set of discrete emotions. |
| Social graphs | Start with 3 or 4 graphs, not every possible network. |
| Institutions | Start with household, market, council, temple, farm. Add guilds/armies/legal courts later. |
| Ecology | Start with simple fertility, resources, and seasons. Do not build full climate simulation early. |
| Economy | Start with grain, water, tools, coin, labor. Do not build full macroeconomics early. |
| TUI | Keep ugly. Inspector, log, map, and trace views only. |
| Parallelism | Defer until deterministic single-threaded core is stable. |
| GUI ambition | Defer indefinitely until simulation is deep and debuggable. |

---

## 19.4 Remove or Defer

| Item | Recommendation |
|---|---|
| Full game engine | Remove. Use Bevy ECS as a library, not full Bevy. |
| Fancy rendering | Defer. |
| Scripted high-level events | Remove as primary design mechanism. Use only exogenous scenario shocks. |
| Global omniscient "unrest" variable | Avoid using it as a causal driver. It may be a dashboard metric, but revolt must emerge from agents. |
| Deep trait-object hierarchies | Avoid in hot paths. |
| Inheritance-style OOP | Avoid. |
| Full technology tree | Defer. |
| Full world-market simulation | Defer. |
| Full climate model | Defer. |
| Full legal code | Defer, but design for it early. |
| Full modding API | Defer, but use data files from day one. |

---

## 19.5 Add

These are under-specified or missing in the current document.

### A. Epistemic Layer

Add a formal model of:

- propositions
- beliefs
- evidence
- sources
- rumors
- official records
- misinformation
- trust in information sources

### B. Causal Provenance Layer

Every important event should have:

- event ID
- tick
- causal parents
- responsible agent/institution
- decision trace
- outcome trace

This is essential for debugging emergence.

### C. Institutional Decision Layer

Institutions should not just have variables like `legitimacy` or `cohesion`.

They need:

- decision procedures
- offices
- officials
- information channels
- delays
- corruption
- enforcement capacity
- policy issuance
- record keeping

### D. Legal / Normative Layer

Add explicit modeling of:

- property rights
- contracts
- crimes
- punishments
- adjudication
- enforcement
- obligations
- permissions
- prohibitions

### E. Material Logistics Layer

Resources should not be abstract global numbers.

Add:

- storage
- spoilage
- transport cost
- carrying capacity
- access rights
- ownership
- local scarcity
- site inventory

### F. Demographic / Life-Course Layer

Add:

- aging
- births
- deaths
- marriage
- fertility
- childhood socialization
- inheritance
- household formation

### G. Health and Disease Layer

Add:

- health
- immunity
- injury
- malnutrition
- disease transmission
- sanitation
- epidemics

### H. Conflict / Coercion Layer

Add:

- threats
- intimidation
- violence
- combat
- casualties
- trauma
- occupation
- repression
- rebellion
- feuds

### I. Cultural / Knowledge Layer

Add:

- practices
- rituals
- symbols
- taboos
- education
- apprenticeship
- innovation
- knowledge diffusion
- ideological traditions

### J. Observability Layer

Add:

- decision traces
- belief traces
- relationship traces
- institutional traces
- metric dashboards
- scenario comparison
- replay diffing

---

# 20. Refined High-Level Architecture

Your existing layered diagram is good, but I would revise it like this:

```text
+---------------------------------------------------------------+
|                    TUI / Debug Instrument                     |
|   map, agent inspector, event log, decision trace, metrics    |
+---------------------------------------------------------------+
|                    Scenario / Experiment Layer                |
|      seeds, shocks, scenarios, assertions, mod definitions    |
+---------------------------------------------------------------+
|                    Simulation Orchestrator                    |
|        fixed tick loop, schedules, replay, snapshots          |
+---------------------------------------------------------------+
|                    Causal Provenance Layer                    |
|        event journal, decision traces, causal chains          |
+---------------------------------------------------------------+
|                    Systemic Layer                             |
|  institutions, law, economy, ecology, governance, conflict    |
+---------------------------------------------------------------+
|                    Social Layer                               |
| relationships, groups, factions, gossip, reputation, norms    |
+---------------------------------------------------------------+
|                    Epistemic Layer                            |
|     perception, attention, beliefs, rumors, information       |
+---------------------------------------------------------------+
|                    Psychological Layer                        |
| needs, traits, emotion, memory, identity, goals, appraisal    |
+---------------------------------------------------------------+
|                    Behavioral Layer                           |
| affordances, utility AI, planning, habits, action execution   |
+---------------------------------------------------------------+
|                    World Layer                                |
| space, regions, resources, sites, objects, time, ecology      |
+---------------------------------------------------------------+
|                    Core Kernel                                |
| ECS, RNG, fixed-point math, commands, persistence, events     |
+---------------------------------------------------------------+
```

The important change is that the **Epistemic Layer** and **Causal Provenance Layer** are now explicit.

---

# 21. Refined Human-Scale Node Model

You said each human-scale node has all substrates. That is correct, but different node types should implement those substrates differently.

## 21.1 Node Types

A human-scale node can be:

- person
- household
- firm
- temple
- guild
- faction
- council
- settlement
- institution
- informal movement

But these are not minds in the same way.

### Person

A person has:

- full psychological substrate
- full behavioral substrate
- full relational substrate
- partial systemic substrate through memberships

### Household

A household has:

- shared resources
- roles
- internal relationships
- collective routines
- derived morale/cohesion
- decision procedure, often informal

### Institution

An institution has:

- offices
- roles
- rules
- resources
- legitimacy
- enforcement capacity
- official records
- decision procedure

### Faction

A faction has:

- collective identity
- leadership
- cohesion
- grievance
- mobilization capacity
- informal communication network

### Settlement

A settlement has:

- population
- sites
- economy
- institutions
- ecology
- infrastructure
- aggregate metrics

---

# 22. Psychological Substrate: Major Refinement

Your current psychological substrate is good, but it should be reorganized as a cognitive pipeline rather than a list of variables.

The psychological substrate should not be "a bag of meters."

It should be:

```text
Body
  -> Needs
    -> Perception
      -> Attention
        -> Appraisal
          -> Emotion
            -> Memory Encoding
              -> Belief Update
                -> Identity / Norm Pressure
                  -> Goal Formation
                    -> Intention
                      -> Behavioral Selection
                        -> Feedback
```

This is the heart of the simulation.

---

## 22.1 Psychological Components

### BodyState

This is the biological foundation.

```rust
struct BodyState {
    health: Fixed,
    energy: Fixed,
    hunger: Fixed,
    thirst: Fixed,
    fatigue: Fixed,
    sickness: Fixed,
    injury: Fixed,
    age: Fixed,
    fertility: Option<Fixed>,
}
```

Body state affects:

- need pressure
- emotional stability
- labor capacity
- disease risk
- mortality risk
- planning horizon

---

### NeedState

Keep needs, but distinguish between:

- deficit
- pressure
- satiation rate
- decay rate

Example:

```rust
struct NeedState {
    hunger_deficit: Fixed,
    thirst_deficit: Fixed,
    fatigue_deficit: Fixed,
    safety_deficit: Fixed,
    social_deficit: Fixed,
    esteem_deficit: Fixed,
    autonomy_deficit: Fixed,
    meaning_deficit: Fixed,
    fairness_deficit: Fixed,
    predictability_deficit: Fixed,
}
```

Important:

> Need pressure should not equal need deficit.

Use nonlinear pressure:

```text
pressure =
    deficit^exponent
  * personality_modifier
  * emotional_modifier
  * context_modifier
```

Example:

```text
hunger_pressure =
    hunger_deficit^1.5
  * impulsivity_modifier
  * food_visibility_modifier
  * social_context_modifier
```

This prevents agents from behaving like perfect optimization machines.

---

### Personality

Contract the trait list initially.

A good starting set:

```rust
struct Personality {
    openness: Fixed,
    conscientiousness: Fixed,
    extraversion: Fixed,
    agreeableness: Fixed,
    neuroticism: Fixed,

    risk_tolerance: Fixed,
    conformity: Fixed,
    ambition: Fixed,
    altruism: Fixed,
    traditionalism: Fixed,
    dominance: Fixed,
    impulsivity: Fixed,
}
```

You can later add:

- vengeance
- greed
- piety
- paranoia
- empathy
- asceticism
- hedonism
- fatalism

But do not start with too many traits.

---

### Affect / Emotional State

Use both dimensional and discrete emotion.

Dimensional:

```rust
struct Affect {
    valence: Fixed,
    arousal: Fixed,
    control: Fixed,
}
```

Discrete:

```rust
struct DiscreteEmotions {
    fear: Fixed,
    anger: Fixed,
    joy: Fixed,
    sadness: Fixed,
    disgust: Fixed,
    shame: Fixed,
    guilt: Fixed,
    pride: Fixed,
    trust: Fixed,
    contempt: Fixed,
    gratitude: Fixed,
    envy: Fixed,
}
```

But initially, use a smaller set:

- fear
- anger
- joy
- sadness
- trust
- shame
- pride
- guilt

Add others later.

---

### Appraisal

This is crucial.

Emotions should arise from appraisal, not be directly assigned.

```rust
struct Appraisal {
    goal_relevance: Fixed,
    goal_congruence: Fixed,
    coping_potential: Fixed,
    expectedness: Fixed,
    fairness: Fixed,
    agency: AgencyAttribution,
    social_visibility: Fixed,
    identity_relevance: Fixed,
    uncertainty: Fixed,
}
```

Example mapping:

```text
If goal_relevance is high:
  If goal_congruence is positive:
    joy, relief, pride
  Else:
    If agency == self:
      guilt, shame
    If agency == other:
      anger, contempt
    If agency == circumstance:
      fear, sadness
    If coping_potential is low:
      fear intensifies
    If fairness is violated:
      anger intensifies
    If identity_relevance is high:
      shame/pride intensify
```

This gives you psychologically grounded emergence.

---

### Attention and Salience

This is missing from the current document and is very important.

Agents cannot appraise everything.

Add:

```rust
struct AttentionState {
    focus: Option<Entity>,
    attention_budget: Fixed,
    habituation: HashMap<StimulusKind, Fixed>,
    salience_bias: Fixed,
}
```

Salience should depend on:

- intensity
- novelty
- emotional relevance
- identity relevance
- survival relevance
- social relevance
- unexpectedness

Example:

```text
salience =
    intensity
  + novelty
  + survival_relevance
  + social_relevance
  + identity_relevance
  - habituation
```

This allows:

- panic
- fixation
- ignorance
- rumor spread
- moral panics
- missed warning signs
- elite blindness
- mass delusion

---

### Memory

Your memory model is good, but expand it.

Memory should be:

- selective
- lossy
- emotionally biased
- socially reinforced
- rehearsed
- distorted over time
- identity-protective

```rust
struct MemoryTrace {
    id: MemoryId,
    kind: MemoryKind,
    tick: u64,
    salience: Fixed,
    emotional_intensity: Fixed,
    rehearsal_count: u32,
    age: u64,
    distortion: Fixed,
    associated_entities: Vec<Entity>,
    tags: Vec<TagId>,
    source: MemorySource,
}
```

Memory types:

- episodic
- semantic
- procedural
- social
- emotional

Add memory processes:

1. Encoding
2. Decay
3. Rehearsal
4. Distortion
5. Reconsolidation
6. Rumination
7. Social reinforcement

Important rule:

> Remembering an event can change the memory.

This is powerful for historical distortion, grievance formation, and factional myth-making.

---

### Beliefs

Your belief model needs to become one of the most important systems in the game.

A belief should not just be a string and a confidence value.

It should have:

```rust
struct Belief {
    proposition: PropositionId,
    confidence: Fixed,
    emotional_charge: Fixed,
    identity_linkage: Fixed,
    resistance: Fixed,
    source_tags: Vec<SourceId>,
    last_reinforced_tick: u64,
    social_reinforcement: Fixed,
    contradiction_set: Vec<PropositionId>,
}
```

Examples of propositions:

```text
the_market_is_fair
the_ruler_is_legitimate
my_neighbor_can_be_trusted
foreigners_are_dangerous
hard_work_leads_to_wealth
the_harvest_will_fail
the_temple_is_corrupt
the_council_protects_us
grain_prices_are_too_high
the_guards_are_unjust
```

Belief update should be Bayesian-ish but psychologically biased.

```text
new_confidence =
    old_confidence * resistance
  + evidence_strength * source_trust
  + emotional_reinforcement
  + social_reinforcement
  + identity_protection_bias
```

Clamp to `[0, 1]`.

Beliefs should resist change when they are:

- identity-linked
- emotionally charged
- socially shared
- repeatedly reinforced
- tied to high-status sources
- tied to institutional authority
- tied to moral foundations

This allows:

- ideology
- propaganda
- rumor
- polarization
- prejudice
- religious movements
- political radicalization
- institutional denial

---

### Proposition Registry

Add a formal proposition system.

Examples:

```rust
enum Proposition {
    EntityHasTrait { entity: Entity, trait_id: TraitId },
    EntityIsTrustworthy { entity: Entity },
    InstitutionIsLegitimate { institution: Entity },
    ResourceIsScarce { resource: ResourceId, location: Entity },
    GroupIsThreatening { group: Entity },
    NormIsJust { norm: NormId },
    FutureEventLikely { event_kind: EventKind, probability: Fixed },
}
```

This lets beliefs be machine-processable.

Without this, beliefs are just flavor text.

---

### Goals

Goals should emerge from:

- needs
- emotions
- beliefs
- identity
- relationships
- roles
- obligations
- institutional duties

```rust
struct Goal {
    kind: GoalKind,
    source: GoalSource,
    priority: Fixed,
    commitment: Fixed,
    horizon: TimeHorizon,
    success_criteria: SuccessCriteria,
    created_tick: u64,
}
```

Distinguish between:

- desires
- goals
- intentions

This is important.

A desire is something an agent wants.

A goal is a selected desired outcome.

An intention is a committed plan.

Without this distinction, agents will flip-flop constantly.

---

### Identity

Expand identity.

Identity is not just a tag.

It is a psychological force.

```rust
struct IdentityState {
    personal_identities: Vec<IdentityId>,
    group_identities: Vec<Entity>,
    role_identities: Vec<RoleId>,
    moral_identities: Vec<MoralIdentityId>,
    narrative_tags: Vec<NarrativeTagId>,
    identity_strength: HashMap<IdentityId, Fixed>,
}
```

Examples:

```text
farmer
parent
citizen
believer
worker
rebel
merchant
outsider
soldier
elder
heretic
loyalist
```

Identity affects:

- utility functions
- norm internalization
- emotional appraisal
- group solidarity
- willingness to sacrifice
- resistance to persuasion
- moral emotions

Example:

```text
utility(action) += identity_congruence * identity_strength
```

An agent who strongly identifies as "devout" may accept costs that a purely self-interested agent would reject.

---

### Moral Foundations / Values

This is missing and useful.

Add a small values model:

```rust
struct MoralValues {
    care: Fixed,
    fairness: Fixed,
    loyalty: Fixed,
    authority: Fixed,
    purity: Fixed,
    liberty: Fixed,
}
```

These affect:

- norm internalization
- emotional response to violation
- political alignment
- religious susceptibility
- factional attraction

For example:

- high fairness → anger at corruption
- high loyalty → factional solidarity
- high authority → obedience to institutions
- high purity → disgust at taboo violation
- high liberty → resistance to taxation/coercion

This helps generate ideological diversity.

---

### Cognitive State

Add bounded rationality explicitly.

```rust
struct CognitiveState {
    attention_capacity: Fixed,
    executive_capacity: Fixed,
    fatigue: Fixed,
    stress: Fixed,
    rumination: Fixed,
    planning_horizon: Fixed,
    heuristic_bias: Fixed,
}
```

Stress should reduce planning horizon.

```text
effective_planning_horizon =
    base_horizon
  * (1 - stress)
  * conscientiousness_modifier
```

When stressed, agents should:

- rely on habit
- imitate peers
- obey authority
- become impulsive
- become aggressive or fearful
- simplify beliefs

This is crucial for panic, mob behavior, and crisis response.

---

### Derived Mental States

Do not simulate everything as direct variables.

Some states should be derived:

- trauma
- depression
- anxiety
- addiction
- resentment
- radicalization
- despair
- resilience
- bitterness
- ambition

Example:

```text
trauma_risk =
    repeated_high_stress_events
  + low_coping_potential
  + low_social_support
  + high_neuroticism

depression_risk =
    chronic_need_deficit
  + low_meaning
  + low_autonomy
  + repeated_failure
  + social_isolation
```

These should influence traits over time.

Psychology should be plastic.

---

# 23. Collective Psychological Substrate

Institutions and groups should not have a full human mind, but they should have a derived collective psychology.

For collectives:

```rust
struct CollectivePsychology {
    morale: Fixed,
    cohesion: Fixed,
    grievance: Fixed,
    fear: Fixed,
    ambition: Fixed,
    trust_in_leadership: Fixed,
    ideological_rigidity: Fixed,
    collective_memory: Vec<CollectiveMemory>,
}
```

For institutions:

```rust
struct InstitutionalPsychology {
    legitimacy: Fixed,
    cohesion: Fixed,
    corruption: Fixed,
    confidence: Fixed,
    repression_willingness: Fixed,
    reform_pressure: Fixed,
    bureaucratic_inertia: Fixed,
}
```

These should be derived from:

- member states
- leadership
- resources
- recent victories/defeats
- internal conflict
- external threats
- legitimacy
- communication quality

Example:

```text
faction.morale =
    average(member.morale)
  + leadership_bonus
  + recent_victory_bonus
  - internal_conflict_penalty
  - resource_scarcity_penalty
  - repression_penalty
```

Important rule:

> Collective psychology should influence agents through roles, messages, sanctions, resources, and institutional decisions — not through magic global variables.

---

# 24. Behavioral Substrate: Refinement

Your behavioral layer should combine:

- utility AI
- hierarchical task decomposition
- affordances
- habits
- routines
- emotional interruption
- bounded rationality

This is correct.

Expand it in the following ways.

---

## 24.1 Action Grammar

Actions should be data.

Each action should have:

```rust
struct ActionDef {
    id: ActionId,
    category: ActionCategory,
    duration_ticks: u32,
    preconditions: Vec<Precondition>,
    affordance: AffordanceId,
    costs: Vec<ResourceCost>,
    risks: Vec<Risk>,
    effects: Vec<Effect>,
    social_meaning: Vec<SocialMeaning>,
    identity_congruence: Vec<IdentityAffinity>,
    norm_relations: Vec<NormRelation>,
}
```

Example:

```text
Action: buy_grain
  category: exchange
  duration: 4 ticks
  requirements:
    - at_site(market)
    - has_resource(coin, 5)
    - target_has_resource(grain, 1)
  effects:
    - transfer_resource(grain, target -> self, 1)
    - transfer_resource(coin, self -> target, 5)
    - modify_relationship(target, trust +0.01)
  social_meaning:
    - lawful_exchange
  norm_relations:
    - respects(private_property)
```

This allows actions to have social and moral meaning, not just mechanical effects.

---

## 24.2 Affordances

Keep affordances.

Objects, sites, and agents expose possible actions.

Example:

```text
Well:
  - drink
  - fetch_water
  - gossip_near
  - contaminate

Market:
  - buy
  - sell
  - loiter
  - protest
  - advertise
  - steal

Person:
  - talk
  - trade
  - threaten
  - bond
  - accuse
  - follow
  - help
  - betray
```

This is important for emergence.

The world should be a field of possible actions, not a set of hardcoded interactions.

---

## 24.3 Utility Function

Your utility model is good.

Refine it like this:

```text
utility(action) =
    need_relief_weight * expected_need_relief
  + emotional_relief_weight * expected_emotional_relief
  + social_weight * expected_relationship_effect
  + identity_weight * identity_congruence
  + normative_weight * norm_compliance
  + institutional_weight * institutional_alignment
  + novelty_weight * stimulation_value
  + habit_weight * habit_strength
  - cost_weight * expected_cost
  - risk_weight * expected_risk
  - cognitive_weight * planning_complexity
  + noise
```

The noise term is essential.

Humans are not perfectly rational.

---

## 24.4 Bounded Rationality

Agents should not search all possible actions.

Limit action search by:

- perception radius
- memory
- attention
- stress
- fatigue
- intelligence
- education
- social trust
- routine inertia

Use heuristics:

- imitate trusted peer
- obey authority
- follow habit
- choose first acceptable action
- copy high-status agent
- avoid feared object
- repeat previously successful action

This is more believable than perfect optimization.

---

## 24.5 Intentions

Add intention commitment.

```rust
struct Intention {
    goal: GoalId,
    plan: PlanId,
    current_step: usize,
    commitment: Fixed,
    started_tick: u64,
}
```

Agents should not abandon intentions too easily.

Abandonment should depend on:

- failure
- cost escalation
- emotional shock
- higher-priority interruption
- belief change
- social pressure

This gives behavioral stability.

---

## 24.6 Routines

Keep routines.

Routines are essential for social stability.

Example:

```text
06:00 wake
07:00 eat
08:00 work
12:00 socialize/eat
13:00 work
18:00 return home
20:00 socialize/worship/rest
22:00 sleep
```

Emergence happens when routines are disrupted by:

- hunger
- fear
- opportunity
- conflict
- institutional demands
- relationship events
- ecological shocks

---

# 25. Relational Substrate: Refinement

Your relational layer is mostly correct, but it needs more structure.

Relationships should be first-class entities.

```rust
struct Relationship {
    from: Entity,
    to: Entity,
    kind: RelationshipKind,
    trust: Fixed,
    affection: Fixed,
    respect: Fixed,
    fear: Fixed,
    obligation: Fixed,
    intimacy: Fixed,
    grievance: Fixed,
    gratitude: Fixed,
    power_balance: Fixed,
    shared_identity: Fixed,
    reciprocity_balance: Fixed,
    last_interaction_tick: u64,
}
```

But you also need **social facts**.

---

## 25.1 Social Facts

Social facts are not just feelings.

They are objective social entities.

Examples:

- promise
- debt
- favor
- offense
- insult
- contract
- marriage
- oath
- betrayal
- alliance
- feud
- patronage
- client relationship

```rust
struct SocialFact {
    id: SocialFactId,
    kind: SocialFactKind,
    from: Entity,
    to: Entity,
    created_tick: u64,
    strength: Fixed,
    fulfilled: bool,
    witnesses: Vec<Entity>,
    associated_events: Vec<EventId>,
}
```

Example:

```text
A promises grain to B
  -> creates obligation
  -> if broken
     -> B feels betrayed
     -> witnesses may update reputation
     -> relationship trust decreases
     -> norm violation may occur
```

This is essential for believable social emergence.

---

## 25.2 Multiple Social Graphs

Keep multiple graphs.

Start with:

1. kinship graph
2. friendship/trust graph
3. economic exchange graph
4. political allegiance graph
5. rumor propagation graph

Later add:

- religious affiliation
- organizational membership
- patron-client network
- status hierarchy
- conflict graph

Do not treat "society" as one relationship type.

---

## 25.3 Reputation

Reputation should be observer-relative.

It is not a global number.

```text
reputation(target, observer) =
    direct_experience
  + trusted_gossip
  - rumor_distortion
  + institutional_labeling
  + status_bias
```

An agent may be trusted by some and hated by others.

This allows:

- factional reputations
- scapegoating
- prestige
- moral panics
- false accusations
- local celebrity
- stigma

---

## 25.4 Gossip

Expand gossip.

Gossip should mutate information.

```text
rumor_accuracy =
    source_memory_accuracy
  * source_trust
  * transmission_fidelity
  * emotional_salience_bias
  * identity_bias
  * repetition_bonus
```

Gossip should be able to:

- simplify
- exaggerate
- distort
- moralize
- assign blame
- strengthen in-group identity
- create false beliefs

This is one of your strongest emergence vectors.

---

# 26. Systemic Substrate: Major Refinement

This is where your simulation can become truly distinctive.

Your current document says institutions should be entities. That is correct.

But institutions need more than variables.

They need decision processes.

---

## 26.1 Institutions as Cognitive Systems

An institution is not just:

```rust
struct Institution {
    legitimacy: f32,
    cohesion: f32,
    treasury: ResourceLedger,
}
```

It should also have:

```rust
struct Institution {
    kind: InstitutionKind,
    offices: Vec<OfficeId>,
    roles: Vec<RoleId>,
    members: Vec<Entity>,
    jurisdiction: Option<Entity>,
    territory: Option<Entity>,
    treasury: ResourceLedger,
    norms: Vec<NormId>,
    policies: Vec<PolicyId>,
    laws: Vec<LawId>,
    legitimacy: Fixed,
    cohesion: Fixed,
    corruption: Fixed,
    enforcement_capacity: Fixed,
    communication_capacity: Fixed,
    information_quality: Fixed,
    decision_procedure: DecisionProcedure,
    official_records: Vec<OfficialRecord>,
}
```

---

## 26.2 Offices

Separate person from office.

```rust
struct Office {
    id: OfficeId,
    institution: Entity,
    title: String,
    holder: Option<Entity>,
    authority: Vec<AuthorityKind>,
    obligations: Vec<ObligationId>,
    term_length: Option<u64>,
    legitimacy: Fixed,
}
```

Examples:

- elder
- mayor
- priest
- guildmaster
- captain
- judge
- tax collector
- market overseer

This prevents the common mistake where "the council" acts magically.

Institutions act through officeholders.

Officeholders have their own psychology.

This creates principal-agent problems:

- corruption
- embezzlement
- negligence
- factional capture
- reform conflict
- succession crises

---

## 26.3 Institutional Decision Procedures

Add explicit decision rules.

```rust
enum DecisionProcedure {
    Autocrat,
    CouncilVote,
    Consensus,
    BureaucraticRule,
    TraditionalPrecedent,
    CharismaticDecree,
    Lottery,
    Election,
}
```

Institutional action should emerge from:

1. information received
2. official reports
3. officeholder beliefs
4. factional pressure
5. legitimacy concerns
6. resource constraints
7. norms/laws
8. decision procedure

Example:

```text
Council receives reports:
  - grain shortage
  - unrest rising
  - merchant hoarding rumors

Officeholders appraise:
  - fear of revolt
  - loyalty to merchants
  - belief in fairness
  - legitimacy concern

Decision procedure:
  council vote

Outcome:
  impose grain price cap
```

This is much better than:

```rust
if unrest > 0.8 {
    spawn_revolt();
}
```

---

## 26.4 Institutional Information Is Not Omniscient

Institutions should not know the true world state.

They know:

- reports
- records
- rumors
- petitions
- tax data
- guard reports
- priest confessions
- merchant testimony
- census estimates

This information can be:

- delayed
- distorted
- corrupted
- incomplete
- ideologically filtered

Add:

```rust
struct InstitutionalInformation {
    reports: Vec<Report>,
    records: Vec<OfficialRecord>,
    information_quality: Fixed,
    delay_ticks: u32,
    corruption_bias: Fixed,
}
```

This allows institutional failure to emerge.

---

## 26.5 Legitimacy

Keep legitimacy as a core variable.

Refine the formula:

```text
legitimacy =
    procedural_fairness
  + outcome_satisfaction
  + traditional_authority
  + charismatic_authority
  + institutional_memory
  + perceived_competence
  - corruption
  - repression
  - unmet_expectations
  - elite_defection
```

Legitimacy affects:

- tax compliance
- law obedience
- willingness to report crimes
- willingness to serve
- protest threshold
- faction formation
- migration
- revolutionary sympathy

When legitimacy collapses, agents may:

- evade taxes
- ignore laws
- form factions
- create alternative institutions
- migrate
- revolt
- support rebels
- refuse conscription

---

## 26.6 Norms and Laws

Your norm model is good.

Expand it into a legal/normative system.

```rust
struct Norm {
    id: NormId,
    scope: NormScope,
    strength: Fixed,
    legitimacy: Fixed,
    internalization_bias: Fixed,
    violation_cost: Sanction,
    enforcement: EnforcementKind,
    emotional_response: Vec<EmotionKind>,
    exceptions: Vec<NormException>,
    visibility_required: bool,
}
```

Examples:

```text
do_not_steal
pay_taxes
honor_parents
obey_council
keep_oaths
respect_temple
do_not_betray_kin
do_not_hoard_grain
```

Agents evaluate norm pressure:

```text
norm_pressure =
    norm.strength
  * agent.conformity
  * agent.identity_alignment
  * agent.moral_internalization
  * fear_of_punishment
  * peer_compliance_rate
  * institutional_legitimacy
```

This allows emergence of:

- compliance
- corruption
- rebellion
- moral panic
- institutional decay
- reform movements
- hypocrisy
- norm erosion

---

# 27. Economy: Refinement

Do not make the economy a global calculator.

Make markets, firms, households, and trade routes entities.

Start simple.

## 27.1 Core Economic Entities

- person
- household
- firm
- market
- temple
- council
- farm
- workshop
- trade route

## 27.2 Core Resources

Start with:

- grain
- water
- wood
- tools
- cloth
- coin
- labor

Later add:

- meat
- ore
- luxury goods
- books
- weapons
- medicine

---

## 27.3 Resource Properties

Resources should have properties:

```rust
struct ResourceDef {
    id: ResourceId,
    category: ResourceCategory,
    perishable: bool,
    spoil_rate: Fixed,
    weight: Fixed,
    volume: Fixed,
    storability: Fixed,
    base_value: Fixed,
}
```

This matters for:

- famine
- hoarding
- trade
- logistics
- taxation
- theft
- spoilage
- inequality

---

## 27.4 Production

Production should require:

- labor
- skill
- tools
- land
- inputs
- time
- ecological conditions

Example:

```text
farm.produce_grain =
    fertility
  * moisture
  * labor_quality
  * tool_quality
  * season_modifier
  - depletion
  - pest_loss
```

---

## 27.5 Markets

Start with simple local price formation.

```text
price =
    base_price
  * scarcity_multiplier
  * demand_multiplier
  * trust_discount
  * transport_cost
  + rumor_panic_premium
```

Later add:

- order books
- arbitrage
- credit
- debt
- bankruptcy
- futures
- insurance

But do not start there.

---

# 28. Ecology: Refinement

Keep ecology simple at first.

Tiles should have:

```rust
struct Tile {
    terrain: Terrain,
    fertility: Fixed,
    moisture: Fixed,
    temperature: Fixed,
    resource_stock: Option<ResourceStock>,
    depletion: Fixed,
    disease_pressure: Fixed,
    owner: Option<Entity>,
    site: Option<Entity>,
}
```

Add:

- seasons
- regeneration
- overuse
- erosion
- disease pressure
- carrying capacity

This allows emergence of:

- famine
- migration
- resource wars
- settlement collapse
- ecological degradation

---

# 29. Conflict and Coercion: Add This

This is under-specified in the original document.

You need a conflict model.

## 29.1 Individual Conflict

Actions:

- threaten
- intimidate
- assault
- flee
- submit
- defend
- retaliate

Effects:

- fear
- injury
- trauma
- grievance
- status change
- reputation change
- relationship rupture

## 29.2 Collective Conflict

Factions and institutions need:

- mobilization capacity
- morale
- supplies
- leadership
- cohesion
- casualties
- legitimacy of violence

```rust
struct ConflictState {
    side_a: Entity,
    side_b: Entity,
    cause: ConflictCause,
    intensity: Fixed,
    morale_a: Fixed,
    morale_b: Fixed,
    casualties_a: Fixed,
    casualties_b: Fixed,
    legitimacy_a: Fixed,
    legitimacy_b: Fixed,
}
```

Conflict should emerge from:

- grievance
- scarcity
- legitimacy collapse
- identity polarization
- elite competition
- repression
- rumor
- honor culture
- factional mobilization

Do not script wars directly.

---

# 30. Culture and Knowledge: Add This

Culture should not be just a tag.

It should be a set of transmissible patterns.

## 30.1 Cultural Elements

- practices
- rituals
- symbols
- taboos
- stories
- moral values
- aesthetic preferences
- technological practices
- institutional traditions

```rust
struct CulturalTrait {
    id: CulturalTraitId,
    domain: CulturalDomain,
    transmission_mode: TransmissionMode,
    identity_linkage: Fixed,
    emotional_charge: Fixed,
    institutional_support: Fixed,
}
```

Examples:

```text
ancestor_veneration
market_fairness_norm
military_honor
egalitarian_custom
royal_divine_right
guild_secrecy
purification_ritual
```

## 30.2 Knowledge / Technology

Do not build a classic tech tree.

Instead model knowledge as practices known by agents and institutions.

```rust
struct Practice {
    id: PracticeId,
    domain: PracticeDomain,
    skill_requirement: Fixed,
    productivity_bonus: Fixed,
    diffusion_rate: Fixed,
}
```

Examples:

```text
crop_rotation
irrigation
metalworking
accounting
law_code
printing
military_drill
shipbuilding
```

Innovation should emerge from:

- need pressure
- skilled practitioners
- contact between cultures
- institutional patronage
- surplus resources
- education
- experimentation

---

# 31. Demography and Life Course: Add This

You need demographic simulation.

## 31.1 Life Stages

- infant
- child
- adolescent
- adult
- elder

Life stage affects:

- labor capacity
- fertility
- dependency
- social roles
- education
- authority
- mortality risk

## 31.2 Household Formation

Add:

- marriage
- divorce
- widowhood
- inheritance
- adoption
- household splitting
- migration of children

## 31.3 Socialization

Children acquire:

- language/culture
- norms
- skills
- identity
- religion
- class habits
- trauma
- wealth
- social networks

This is essential for long-term historical simulation.

---

# 32. Health and Disease: Add This

Health should not be just "hit points."

```rust
struct HealthState {
    overall_health: Fixed,
    immunity: Fixed,
    nutrition: Fixed,
    injury: Fixed,
    chronic_condition: Option<ConditionId>,
    infection: Option<DiseaseId>,
}
```

Disease emergence should depend on:

- density
- sanitation
- malnutrition
- migration
- trade routes
- climate
- public health institutions
- ritual purity practices

This allows:

- epidemics
- labor shortages
- religious explanation
- scapegoating
- migration
- institutional crisis

---

# 33. Epistemic Layer: The Most Important Addition

This deserves its own section.

You need a formal epistemic architecture.

---

## 33.1 Objective Facts

These are true in the world.

```text
Agent(12) has 5 grain.
Settlement food stock = 300.
Market price grain = 14.
Council legitimacy = 0.42.
Farm(3) fertility = 0.61.
```

## 33.2 Subjective Beliefs

Agents may believe false things.

```text
Agent(17) believes:
  market_is_unfair: 0.83
  merchant(9) is hoarding: 0.71
  council_is_corrupt: 0.64
```

## 33.3 Shared Beliefs

Groups may share beliefs.

```text
Faction(3) shared belief:
  council_is_illegitimate: 0.78
```

## 33.4 Official Records

Institutions may record things.

```text
Council record:
  tax_collected = 240
  reported_grain_stock = 800
```

But the record may be wrong or false.

## 33.5 Rumors

Rumors are propositions transmitted between agents.

```rust
struct Rumor {
    id: RumorId,
    proposition: PropositionId,
    origin: Option<Entity>,
    emotional_charge: Fixed,
    mutation_level: Fixed,
    spread_count: u32,
}
```

This gives you:

- misinformation
- panic
- scapegoating
- factional myths
- propaganda
- institutional distrust

---

# 34. Causal Provenance Layer: Essential for Debugging

You already mention causal explanation. Expand it into a first-class system.

Every important event should have:

```rust
struct EventRecord {
    id: EventId,
    tick: u64,
    kind: EventKind,
    actors: Vec<Entity>,
    targets: Vec<Entity>,
    causal_parents: Vec<EventId>,
    trace: Option<TraceId>,
}
```

Every important decision should have:

```rust
struct DecisionTrace {
    id: TraceId,
    agent: Entity,
    tick: u64,
    decision_kind: DecisionKind,
    inputs: Vec<TraceInput>,
    scored_options: Vec<ScoredOption>,
    chosen_option: OptionId,
    emotional_state: EmotionalSnapshot,
    belief_state: BeliefSnapshot,
}
```

Example:

```text
Agent: Anna
Tick: 1042
Decision: go_to_market

Reason:
  hunger_pressure = 0.71
  food_at_home = 0
  market_distance = 6
  safety = 0.82
  coin = 14
  trust_in_merchant = 0.66
  norm_compliance = 0.74

Utilities:
  go_to_market = 0.81
  steal_food = 0.39
  ask_neighbor = 0.52
  wait = 0.18

Chosen:
  go_to_market
```

Without this, emergence becomes unreadable.

---

# 35. Revised Simulation Loop

Your loop is good, but I would refine the order.

Use explicit phases.

```text
Phase 0: Input / Commands
  - process player commands

Phase 1: World Update
  - advance time
  - ecology (seasons, climate, disease)
  - resource regeneration/depletion
  - site decay/maintenance

Phase 2: Perception
  - agents perceive local world
  - institutions receive reports
  - rumors propagate

Phase 3: Attention
  - agents filter perceptions
  - salience computation
  - attention allocation

Phase 4: Appraisal
  - agents appraise attended stimuli
  - emotional response generated
  - institutional officeholders appraise reports

Phase 5: Psychological Update
  - need decay/satiation
  - belief update
  - memory encoding/decay
  - identity reinforcement
  - cognitive state update

Phase 6: Goal Formation
  - desires from needs/emotions/beliefs
  - goal selection
  - intention commitment

Phase 7: Planning
  - HTN decomposition
  - routine matching
  - affordance scanning

Phase 8: Action Selection
  - utility scoring
  - bounded search
  - noise injection
  - action chosen

Phase 9: Action Execution
  - movement
  - interaction
  - production
  - consumption
  - exchange
  - communication

Phase 10: Social Update
  - relationship updates
  - social fact creation/fulfillment
  - gossip transmission
  - reputation updates

Phase 11: Institutional Update
  - officeholder decisions
  - policy enforcement
  - tax collection
  - record keeping
  - legitimacy update

Phase 12: Economic Update
  - market price formation
  - production resolution
  - inventory updates
  - trade route updates

Phase 13: Conflict Resolution
  - threat assessment
  - combat resolution
  - coercion effects
  - trauma processing

Phase 14: Demographic Update
  - aging
  - births
  - deaths
  - marriage
  - household formation
  - migration

Phase 15: Cultural Update
  - practice diffusion
  - ritual performance
  - innovation
  - knowledge transmission

Phase 16: Event Generation
  - create event records
  - causal linking
  - trace recording

Phase 17: Output / Persistence
  - flush commands
  - emit snapshots
  - write event journal
  - render TUI
```

---

# 36. Core Abstractions

These should be first-class IDs:

```rust
EntityId
EventId
TraceId
PropositionId
BeliefId
MemoryId
NormId
LawId
PolicyId
ActionId
AffordanceId
GoalId
IntentionId
RelationshipId
SocialFactId
RumorId
InstitutionId
OfficeId
RoleId
ResourceId
SiteId
TileId
CulturalTraitId
PracticeId
DiseaseId
```

This gives you a clean ontology.

---

# 37. Phased Implementation Plan

## Phase 1: Deterministic Kernel

Goal:

- fixed tick loop
- seeded RNG
- deterministic output
- tracing logs
- fixed timestep

Command:

```bash
cargo run -p mindstrata-cli -- sim --seed 42 --ticks 1000
```

Acceptance:

- same seed produces identical output
- tick count matches
- no nondeterminism

---

## Phase 2: ECS and Basic Components

Use `bevy_ecs`.

Create:

```rust
Person
Position
BodyState
NeedState
Personality
EmotionalState
AttentionState
```

Systems:

```text
spawn_agents
advance_time
decay_needs
update_attention
print_summary
```

Acceptance:

- agents spawn
- needs decay
- attention updates
- deterministic output

---

## Phase 3: Psychological Pipeline

Goal:

- perception
- attention
- appraisal
- emotion
- belief update
- memory encoding

Acceptance:

- agents perceive local world
- emotions arise from appraisal
- beliefs update from evidence
- memories form

Emergent test:

- agents develop different beliefs about same event

---

## Phase 4: Behavioral AI

Goal:

- utility AI
- action definitions
- routines
- bounded planning
- intention commitment

Acceptance:

- agents seek food when hungry
- agents sleep when tired
- agents follow routines
- agents replan when interrupted

Emergent test:

- food scarcity causes competition.

---

## Phase 5: Relationships and Gossip

Goal:

- dyadic relationships
- trust
- affection
- obligation
- interaction memory
- gossip

Acceptance:

- agents form friendships
- agents avoid disliked agents
- rumors spread
- reputation differs by observer

Emergent test:

- a theft creates a local feud.

---

## Phase 6: Households, Sites, and Economy

Goal:

- houses
- farms
- wells
- markets
- inventory
- production
- consumption
- household sharing

Acceptance:

- agents produce food
- agents trade food
- households consume shared resources
- prices respond to scarcity

Emergent test:

- famine causes migration and black markets.

---

## Phase 7: Norms and Institutions

Goal:

- norms
- roles
- households
- council
- temple
- legitimacy
- enforcement

Acceptance:

- agents pay taxes
- agents obey norms
- institutions punish violations
- legitimacy changes with fairness and outcomes

Emergent test:

- corrupt enforcement reduces legitimacy.

---

## Phase 8: Factions and Politics

Goal:

- factions
- collective identity
- recruitment
- protests
- leadership
- grievance

Acceptance:

- factions form around grievance
- agents recruit others
- institutions respond to unrest

Emergent test:

- legitimacy crisis produces rebellion or reform.

---

## Phase 9: Ecology, Demography, Health

Goal:

- seasons
- fertility
- resource depletion
- births
- deaths
- disease
- migration

Acceptance:

- harvest varies by season
- overfarming reduces fertility
- disease spreads in dense settlements
- population migrates under pressure

Emergent test:

- ecological decline produces famine and conflict.

---

## Phase 10: Debugger, Replay, and Metrics

Goal:

- ASCII map
- agent inspector
- event log
- decision trace
- relationship viewer
- metrics dashboard
- replay
- snapshot comparison

Acceptance:

- you can inspect any agent
- you can trace why an action happened
- same seed produces same history
- scenario comparisons are possible

---

# 38. Minimum Playable Vertical Slice

Your "Riverford" scenario is correct. I would make the first version even smaller.

## 38.1 First Slice: Riverford Minor

Entities:

- 30 persons
- 8 households
- 3 farms
- 1 market
- 1 well
- 1 temple
- 1 council
- 1 forest
- 1 river

Duration:

- 30 simulated days

Systems:

- needs
- food production
- labor
- household economy
- market exchange
- relationships
- gossip
- norms
- legitimacy
- simple ecology
- simple migration

Do not add yet:

- factions
- full law
- full tech
- full climate
- full warfare

Let factions emerge if possible.

---

## 38.2 Emergence Targets

You want to observe:

- who becomes poor
- who becomes trusted
- who becomes isolated
- whether rumors spread
- whether norms are violated
- whether market prices fluctuate
- whether people migrate
- whether the council loses legitimacy
- whether households form strategic marriages
- whether a local feud emerges

---

## 38.3 Scenario Shocks

Run the same settlement under different shocks:

### Scenario A: Normal Harvest

Baseline.

### Scenario B: Drought

Reduce farm fertility.

Expected emergent effects:

- food prices rise
- hunger increases
- theft increases
- migration increases
- legitimacy decreases

### Scenario C: Unfair Tax

Council increases tax.

Expected emergent effects:

- grievance increases
- tax evasion increases
- trust in council decreases
- faction formation becomes more likely

### Scenario D: Rumor of Hoarding

Inject rumor:

```text
merchant(9) is hoarding grain
```

Expected emergent effects:

- trust in merchant decreases
- market anxiety increases
- panic buying may occur
- accusation may occur
- reputation damage may persist even if rumor is false

These are not scripted outcomes. They are statistical expectations.

---

# 39. Testing Strategy for Emergent Systems

Your testing section is good. Expand it.

## 39.1 Unit Tests

Test formulas:

- need decay
- emotion appraisal
- utility scoring
- belief updating
- relationship decay
- price adjustment
- legitimacy change

## 39.2 Property Tests

Use `proptest`.

Examples:

- resources are never negative
- utility scores are finite
- belief confidence remains within `[0, 1]`
- relationships decay without interaction
- deterministic seed produces same output

## 39.3 Golden Replay Tests

Run scenario for 10,000 ticks.

Save:

- final metrics
- event hash
- agent summary
- relationship summary
- institutional summary

Compare against baseline.

If it changes, inspect whether intended.

## 39.4 Statistical Emergence Tests

Run many seeds.

Example:

```text
Scenario: drought
After 30 days:
  average_food < baseline_average_food
  migration_rate > baseline_migration_rate
  theft_rate > baseline_theft_rate
  legitimacy < baseline_legitimacy
```

These tests should not require exact outcomes.

They should test tendencies.

---

# 40. Major Architectural Risks and Mitigations

## Risk 1: Psychology Becomes a Bag of Meters

Mitigation:

Use the cognitive pipeline:

```text
perception -> attention -> appraisal -> emotion -> belief -> identity -> goal -> intention -> action
```

## Risk 2: Institutions Become Omniscient Gods

Mitigation:

Institutions only act through:

- offices
- officials
- reports
- records
- delays
- noise
- corruption
- enforcement capacity

## Risk 3: Emergence Becomes Unreadable

Mitigation:

Add decision traces and causal event chains from day one.

## Risk 4: Nondeterminism

Mitigation:

Use fixed-point math, stable iteration, seeded RNG streams, and replay hashes.

## Risk 5: Overbuilding Before Validation

Mitigation:

Build Riverford Minor first.

Do not build empire-scale simulation before agents can eat, sleep, trade, gossip, and obey norms.

## Risk 6: Too Many Abstractions

Mitigation:

Prefer plain data components in hot paths.

Use traits sparingly.

## Risk 7: GUI Creep

Mitigation:

Keep TUI ugly.

It is a debugger, not a game interface.

---

# 41. Suggested Immediate Next Steps

Do this exact sequence.

## Step 1: Create Workspace

```bash
cargo new mindstrata --workspace
cd mindstrata
mkdir crates specs tasks golden runs
```

Create:

```text
crates/mindstrata-core
crates/mindstrata-sim
crates/mindstrata-cli
crates/mindstrata-tui
crates/mindstrata-tests
```

---

## Step 2: Build Deterministic Tick Kernel

Requirements:

- seed
- tick count
- deterministic output
- tracing logs
- fixed timestep

Command:

```bash
cargo run -p mindstrata-cli -- sim --seed 42 --ticks 1000
```

---

## Step 3: Add ECS and Basic Components

Use `bevy_ecs`.

Create:

```rust
Person
Position
BodyState
NeedState
Personality
EmotionalState
AttentionState
```

Systems:

```text
spawn_agents
advance_time
decay_needs
update_attention
print_summary
```

---

## Step 4: Add Tiny TUI Debugger

Use `ratatui`.

Display:

- tick
- agent count
- average hunger
- average fatigue
- selected agent state

Do not make it pretty.

---

## Step 5: Implement First Emergent Loop

Agents must:

- get hungry
- perceive food
- attend to food
- appraise food availability
- form goal: get food
- choose action
- move to food
- eat
- fail if no food
- remember food location
- prefer safe food sources

This is your first real validation.

---

## Step 6: Add Belief and Memory

Agents should remember:

- where food was
- who helped them
- who harmed them
- whether places are safe
- whether institutions are fair

---

## Step 7: Add Relationships

Agents should form:

- trust
- affection
- fear
- obligation
- grievance

---

## Step 8: Add Gossip

Agents should transmit propositions:

```text
merchant_is_unfair
neighbor_is_trustworthy
well_is_contaminated
council_is_corrupt
```

---

## Step 9: Add Household and Market

Start with:

- grain
- water
- coin
- labor

Then add:

- farms
- wells
- markets
- households

---

## Step 10: Add Norms and Legitimacy

Start with:

```text
do_not_steal
pay_taxes
keep_promises
obey_council
```

Then observe:

- compliance
- evasion
- punishment
- resentment
- legitimacy change

---

# 42. Final Recommendation

Your architecture should be refined around this principle:

> The simulation is not primarily a world of objects.  
> It is a world of bounded minds, social facts, institutional processes, and causal chains.

So the most important architectural additions are:

1. **Epistemic layer**
   - beliefs, propositions, rumors, source trust, misinformation

2. **Causal provenance layer**
   - event chains, decision traces, replayable explanation

3. **Institutional decision layer**
   - offices, decision procedures, reports, enforcement, corruption

4. **Social facts layer**
   - obligations, debts, promises, offenses, contracts, reputations

5. **Material logistics layer**
   - storage, spoilage, transport, ownership, local scarcity

6. **Life-course layer**
   - aging, kinship, marriage, inheritance, socialization

7. **Conflict layer**
   - coercion, violence, trauma, repression, rebellion

8. **Cultural knowledge layer**
   - practices, rituals, norms, education, innovation

At the same time, contract the early scope:

- fewer crates
- fewer traits
- fewer emotions
- fewer institutions
- smaller map
- smaller population
- no fancy GUI
- no parallelism until stable
- no full climate/macro-economy/tech tree early

The correct first milestone is not "a world."

It is:

> A small settlement where agents can need, perceive, believe, remember, relate, exchange, obey, violate, gossip, and generate a readable causal history.

If you get that right, Mindstrata can generate history instead of merely containing it.
---

# 43. Crate Responsibilities (Refined)

The workspace is split into five crates:

```text
mindstrata-core/
mindstrata-sim/
mindstrata-cli/
mindstrata-tui/
mindstrata-tests/
```

## 43.1 `mindstrata-core`

Contains:

- fixed-point math
- RNG streams
- IDs
- event types
- error types
- deterministic collections
- serialization helpers
- tracing utilities

## 43.2 `mindstrata-sim`

Contains the simulation:

- world
- psychology
- behavior
- social
- systems
- economy
- ecology
- institutions
- epistemic layer
- causal provenance

Internally, use modules first:

```text
mindstrata-sim/
  src/
    world/
    psych/
    behavior/
    social/
    systems/
    epistemic/
    provenance/
    economy/
    ecology/
    demography/
    conflict/
    culture/
    schedule.rs
```

Split into more crates only when compile time or team/agent parallelism demands it.

## 43.3 `mindstrata-cli`

Primary automation interface.

Commands:

```bash
cargo run -p mindstrata-cli -- sim --seed 42 --ticks 10000
cargo run -p mindstrata-cli -- scenario riverford_minor --seed 42
cargo run -p mindstrata-cli -- trace --agent 12 --tick 1042
cargo run -p mindstrata-cli -- replay-diff run_a.snapshot run_b.snapshot
cargo run -p mindstrata-cli -- validate --gate phase-3
cargo run -p mindstrata-cli -- metrics --scenario drought --seeds 1..32
```

## 43.4 `mindstrata-tui`

Debugger only.

Views:

- map
- agent inspector
- event log
- decision trace
- relationship graph
- institution dashboard
- metrics

## 43.5 `mindstrata-tests`

Contains:

- golden replay tests
- statistical emergence tests
- scenario assertions
- invariant checks
- benchmark harnesses
- trace audits

---

# 44. Agentic Loop Architecture for Autonomous Development

The architecture must not only simulate agents; it must allow engineering agents to autonomously develop the simulation.

You need two loops.

## 44.1 In-game agent loop

```text
World State
  -> Perception
    -> Attention
      -> Appraisal
        -> Emotion
          -> Memory / Belief Update
            -> Identity / Norm Pressure
              -> Goal Formation
                -> Intention
                  -> Action Selection
                    -> Action Execution
                      -> World Effects
                        -> Causal Event Record
```

This loop produces emergent behavior.

## 44.2 Engineering agent loop

```text
Specification
  -> Task
    -> Implementation
      -> Deterministic Run
        -> Evidence
          -> Failure Analysis
            -> Patch
              -> Validation
                -> Baseline Update
                  -> Next Task
```

This loop produces reliable project progress.

The key is that every task must have:

- clear specification
- machine-readable acceptance criteria
- deterministic reproduction
- traceable failure modes
- automated validation
- small surface area
- measurable evidence

---

# 45. Specification-as-Data

AI agents need unambiguous contracts.

Use data files for stable ontology and scenario definitions.

## 45.1 `specs/ontology.ron`

Defines core IDs and entity categories:

```ron
Ontology(
  entity_kinds: [
    Person,
    Household,
    Farm,
    Market,
    Well,
    Temple,
    Council,
    Settlement,
    Faction,
    Institution,
  ],
  id_kinds: [
    EntityId,
    EventId,
    TraceId,
    PropositionId,
    BeliefId,
    MemoryId,
    NormId,
    LawId,
    PolicyId,
    ActionId,
    AffordanceId,
    GoalId,
    IntentionId,
    RelationshipId,
    SocialFactId,
    RumorId,
    InstitutionId,
    OfficeId,
    RoleId,
    ResourceId,
    SiteId,
    TileId,
    CulturalTraitId,
    PracticeId,
    DiseaseId,
  ],
)
```

## 45.2 `specs/components.ron`

Defines component schemas:

```ron
Components(
  person: [
    "Person",
    "BodyState",
    "NeedState",
    "Personality",
    "Affect",
    "DiscreteEmotions",
    "AttentionState",
    "Memory",
    "Beliefs",
    "Goals",
    "IdentityState",
    "MoralValues",
    "CognitiveState",
    "ActionState",
    "SocialIdentity",
  ],
  institution: [
    "Institution",
    "Offices",
    "NormRegistry",
    "PolicyRegistry",
    "ResourceLedger",
    "EnforcementCapacity",
    "InstitutionalInformation",
    "LegitimacyState",
  ],
)
```

## 45.3 `specs/systems.ron`

Defines system order and phase membership.

```ron
Systems(
  phases: [
    WorldUpdate,
    Perception,
    Attention,
    Appraisal,
    PsychologicalUpdate,
    GoalFormation,
    Planning,
    ActionSelection,
    ActionExecution,
    SocialUpdate,
    InstitutionalUpdate,
    EconomicUpdate,
    ConflictResolution,
    DemographicUpdate,
    CulturalUpdate,
    EventGeneration,
    Output,
  ],
)
```

## 45.4 `specs/actions.ron`

Defines action grammar.

```ron
Action(
  id: "buy_grain",
  category: Exchange,
  duration_ticks: 4,
  preconditions: [
    AtSite("market"),
    HasResource("coin", 5),
    TargetHasResource("grain", 1),
  ],
  effects: [
    TransferResource("grain", Target, Self, 1),
    TransferResource("coin", Self, Target, 5),
    ModifyRelationship(Target, Trust, +0.01),
  ],
  social_meaning: [
    "lawful_exchange",
  ],
  norm_relations: [
    Respects("private_property"),
  ],
)
```

## 45.5 `specs/norms.ron`

```ron
Norm(
  id: "do_not_steal",
  scope: Settlement,
  strength: 0.8,
  legitimacy: 0.7,
  internalization_bias: 0.6,
  violation_cost: Fine(100),
  enforcement: Guards,
  emotional_response: [Guilt, Shame, Anger],
  visibility_required: true,
)
```

## 45.6 `specs/propositions.ron`

```ron
Propositions(
  entries: [
    "market_is_fair",
    "council_is_legitimate",
    "council_is_corrupt",
    "merchant_9_is_hoarding",
    "well_is_contaminated",
    "neighbor_is_trustworthy",
    "foreigners_are_dangerous",
    "harvest_will_fail",
  ],
)
```

---

# 46. Scenario DSL

Scenarios should be data, not code.

This allows AI agents to run experiments autonomously.

Example:

```ron
Scenario(
  id: "riverford_minor",
  seed: 42,
  ticks: 4320,
  settlement: "Riverford",
  population: 30,
  households: 8,
  sites: [
    Farm(3),
    Market(1),
    Well(1),
    Temple(1),
    Council(1),
  ],
  resources: [
    "grain",
    "water",
    "coin",
    "tools",
    "labor",
  ],
  norms: [
    "do_not_steal",
    "pay_taxes",
    "keep_promises",
    "obey_council",
  ],
  shocks: [],
  assertions: [
    MetricGreaterThan("population_alive", 25),
    MetricExists("market_price_grain"),
    EventCountGreaterThan("AgentAte", 100),
  ],
)
```

Shock scenario:

```ron
Scenario(
  id: "drought",
  extends: "riverford_minor",
  shocks: [
    AtTick(
      tick: 500,
      effect: ModifyTiles(
        kind: Farm,
        fertility_multiplier: 0.4,
        duration_ticks: 3000,
      ),
    ),
  ],
  assertions: [
    Statistical(
      metric: "avg_food_per_household",
      direction: Decrease,
      confidence: 0.8,
    ),
    Statistical(
      metric: "theft_rate",
      direction: Increase,
      confidence: 0.7,
    ),
    Statistical(
      metric: "council_legitimacy",
      direction: Decrease,
      confidence: 0.6,
    ),
  ],
)
```

This allows emergence testing without scripting outcomes.

---

# 47. Verification Pyramid (Refined)

AI agents need a strong validation oracle.

Use five levels.

## 47.1 Unit tests

Test formulas:

- need decay
- emotion appraisal
- utility scoring
- belief updating
- relationship decay
- price adjustment
- legitimacy change
- norm pressure
- memory decay
- gossip mutation

Example:

```rust
#[test]
fn belief_update_strengthens_confidence_with_trusted_evidence() {
    // ...
}
```

## 47.2 Property tests

Use `proptest`.

Examples:

- resources are never negative
- utility scores are finite
- belief confidence remains within `[0, 1]`
- relationships decay without interaction
- deterministic seed produces same output
- fixed-point multiplication does not overflow silently
- event causal parents always precede children
- no agent acts on information it has not perceived

## 47.3 Golden replay tests

Run scenario for N ticks.

Save:

- final metrics
- event hash
- agent summary
- relationship summary
- institutional summary

Compare against baseline.

If it changes, inspect whether intended.

Example command:

```bash
cargo run -p mindstrata-cli -- golden --scenario riverford_minor --ticks 10000
```

Output:

```text
scenario: riverford_minor
seed: 42
ticks: 10000
event_hash: 8f3a...
metric_hash: c19b...
agent_hash: 44de...
status: match
```

## 47.4 Statistical emergence tests

Run many seeds.

Example:

```text
Scenario: drought
Seeds: 1..32
After 30 days:
  average_food < baseline_average_food
  migration_rate > baseline_migration_rate
  theft_rate > baseline_theft_rate
  legitimacy < baseline_legitimacy
```

These tests should not require exact outcomes.

They should test tendencies.

## 47.5 Trace audits

For any important event, the system must be able to answer:

```text
Why did this happen?
Which agent decided?
What did they believe?
What did they perceive?
What emotions were active?
Which norms applied?
Which institutional role authorized it?
Which prior events caused it?
```

Example command:

```bash
cargo run -p mindstrata-cli -- explain-event 18432
```

Output:

```text
Event: NormViolated
Agent: 17
Norm: do_not_steal
Tick: 8421

Causal chain:
  8120: drought reduced farm fertility
  8190: household food fell below threshold
  8301: agent hunger_pressure rose to 0.81
  8392: agent believed market_price too high
  8410: agent perceived grain at market
  8421: agent chose steal_grain

Decision trace:
  buy_grain = 0.44
  steal_grain = 0.61
  ask_neighbor = 0.38
  wait = 0.12

Contributing factors:
  high hunger pressure
  low coin
  low trust in market fairness
  low norm internalization
  low perceived punishment risk
```

This is what makes emergence debuggable.

---

# 48. Autonomous Engineering Agent Protocol

Define a strict protocol for AI agents.

## 48.1 Agent roles

You can implement these as separate agents or as modes of one agent.

### Architect Agent

Responsibilities:

- reads specifications
- proposes module boundaries
- ensures ontology consistency
- prevents overengineering
- approves new abstractions

### Implementer Agent

Responsibilities:

- implements one system at a time
- writes components and systems
- adds tracing
- writes unit tests
- keeps patches small

### Tester Agent

Responsibilities:

- writes property tests
- writes scenario assertions
- generates golden baselines
- checks invariants
- detects nondeterminism

### Debugger Agent

Responsibilities:

- reads failing traces
- isolates causal chain
- produces minimal reproduction
- proposes patch
- verifies fix

### Validator Agent

Responsibilities:

- runs acceptance gates
- compares metrics
- validates emergence hypotheses
- checks performance budgets
- produces evidence bundle

### Documentation Agent

Responsibilities:

- updates architecture docs
- records design decisions
- updates specs
- writes runbooks
- maintains task contracts

## 48.2 Autonomous task contract

Every task should have this structure:

```yaml
task_id: PSY-014
title: Implement belief update from trusted gossip
phase: PsychologicalUpdate
depends_on:
  - PSY-009
  - EPI-003
spec:
  - specs/propositions.ron
  - specs/components.ron
  - specs/systems.ron
implementation:
  - add BeliefUpdateTrace
  - update belief confidence from rumor source trust
  - apply identity resistance
  - apply social reinforcement
tests:
  unit:
    - trusted_source_increases_confidence
    - identity_linked_belief_resists_change
  property:
    - belief_confidence_within_bounds
  scenario:
    - rumor_hoarding
  trace:
    - belief_update_trace_exists
acceptance:
  - deterministic replay unchanged for unaffected systems
  - rumor scenario produces belief change in at least 30% of seeds
  - no belief confidence outside [0,1]
  - trace explains belief change
evidence:
  - test_output.json
  - metrics_delta.json
  - trace_sample.json
```

This makes autonomous progress possible.

## 48.3 Definition of Done

A system is done only when:

1. It matches the spec.
2. It has unit tests.
3. It has property tests.
4. It emits traces.
5. It affects at least one scenario metric.
6. It does not break determinism.
7. It does not break golden replays unless intended.
8. It has documentation.
9. It has an acceptance report.
10. It can be explained through the trace system.

This prevents "implemented but unverifiable" systems.

## 48.4 Autonomous development loop

Pseudo-code:

```python
for task in task_queue:
    load_spec(task)
    load_dependencies(task)

    implement_minimal_solution(task)

    run("cargo fmt")
    run("cargo clippy")
    run("cargo test")

    run("mindstrata-cli validate --task {task.id}")

    if failure:
        trace = collect_trace(failure)
        hypothesis = analyze_trace(trace)
        patch = generate_patch(hypothesis)
        apply_patch(patch)
        rerun_validation()

    if success:
        run("mindstrata-cli golden --update-if-justified")
        write_evidence_bundle(task)
        commit(task)
        advance_task_queue()
```

The important part is that failure is always reduced to:

```text
spec violation
test failure
trace anomaly
metric regression
nondeterminism
performance regression
```

Not vague "it feels wrong."

---

# 49. Observability for AI Agents

The simulation must be legible to machines.

## 49.1 Required CLI outputs

Every run should be able to emit:

```bash
--emit metrics
--emit events
--emit snapshots
--emit traces
--emit report
```

Example:

```bash
cargo run -p mindstrata-cli -- \
  scenario riverford_minor \
  --seed 42 \
  --ticks 10000 \
  --emit metrics,events,traces,report \
  --out runs/riverford_42
```

Output:

```text
runs/riverford_42/
  metrics.json
  events.jsonl
  snapshot.bin
  traces.jsonl
  report.md
  replay.hash
```

## 49.2 Metrics schema

Example:

```json
{
  "scenario": "riverford_minor",
  "seed": 42,
  "tick": 10000,
  "population_alive": 29,
  "avg_hunger": 0.37,
  "avg_fatigue": 0.44,
  "avg_legitimacy": 0.51,
  "food_stock": 618,
  "grain_price": 14.2,
  "theft_count": 3,
  "rumor_count": 11,
  "faction_count": 0,
  "migration_count": 1,
  "trust_density": 0.31,
  "grievance_index": 0.22
}
```

## 49.3 Trace schema

Example:

```json
{
  "trace_id": 842,
  "tick": 1042,
  "agent": 12,
  "decision_kind": "action_selection",
  "chosen": "go_to_market",
  "inputs": {
    "hunger_pressure": 0.71,
    "food_at_home": 0.0,
    "coin": 14.0,
    "safety": 0.82,
    "trust_in_merchant": 0.66
  },
  "scored_options": [
    { "option": "go_to_market", "utility": 0.81 },
    { "option": "steal_food", "utility": 0.39 },
    { "option": "ask_neighbor", "utility": 0.52 },
    { "option": "wait", "utility": 0.18 }
  ],
  "emotional_state": {
    "fear": 0.05,
    "anger": 0.11,
    "joy": 0.21
  },
  "belief_state": {
    "market_is_fair": 0.62,
    "neighbor_trustworthy": 0.77
  }
}
```

This allows AI agents to debug without guessing.

---

# 50. Additional Risks and Mitigations

## Risk 8: AI Agents Thrash Without Clear Validation

AI agents can enter loops when they lack a clear oracle.

**Mitigation:**

- Every task has explicit acceptance criteria
- Every acceptance criterion has a machine-checkable command
- The validation command either passes or fails with a clear error
- No task is accepted without evidence
- Evidence is stored in a standard format
- The agent can always ask "what specifically failed?"

---

# 51. Friction-Reducing Architecture for AI Agents

If you want maximum efficacy with autonomous agents, add these explicitly.

---

## 51.1 Every system must have observable effects

No system should exist unless it changes at least one of:

- metrics
- events
- traces
- agent behavior
- institutional outcome

This prevents invisible complexity.

---

## 51.2 Every task must be small

A good task usually touches:

- one phase
- one substrate
- one component family
- one scenario effect

Avoid large cross-cutting tasks unless they are integration milestones.

---

## 51.3 Every failure must produce a minimal trace

The system should automatically emit:

- failing tick
- failing entity
- failing system
- relevant inputs
- relevant beliefs
- relevant emotions
- relevant norms
- relevant causal parents

This makes debugging autonomous.

---

## 51.4 Every scenario must have assertions

No scenario should be only "run and see."

It should include:

- invariant assertions
- metric assertions
- statistical assertions
- trace assertions

---

## 51.5 Every baseline must be reviewable

Golden baselines should be stored as:

```text
golden/
  riverford_minor/
    seed_42/
      metrics.json
      replay.hash
      summary.md
```

If a baseline changes, the agent must produce a delta report.

---

## 51.6 Add a spec linter

This is highly recommended.

The linter should check:

- every action has preconditions and effects
- every norm has sanction and scope
- every proposition is registered
- every system declares reads/writes
- every event kind has trace metadata
- every scenario assertion refers to a known metric
- every institution has at least one decision procedure
- every office has authority and obligations

This prevents ontology drift.
