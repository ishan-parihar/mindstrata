# Mindstrata — Current State Technical Document

**Prepared for:** Lead Game Designer  
**Date:** July 30, 2026  
**Codebase Version:** 17,460 lines of Rust across 5 crates, 266 tests passing, 2 clippy warnings

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Core Design Philosophy](#2-core-design-philosophy)
3. [Architecture Overview](#3-architecture-overview)
4. [Directory Structure & File Map](#4-directory-structure--file-map)
5. [The 10 Substrates (Layer-by-Layer)](#5-the-10-substrates-layer-by-layer)
6. [What Fully Works (Complete Systems)](#6-what-fully-works-complete-systems)
7. [Known Implementation Gaps](#7-known-implementation-gaps)
8. [Data-Driven Specifications (RON Files)](#8-data-driven-specifications-ron-files)
9. [Test Coverage & Quality](#9-test-coverage--quality)
10. [Technology Stack & Dependencies](#10-technology-stack--dependencies)
11. [How to Run](#11-how-to-run)
12. [Recommended Future Development Priorities](#12-recommended-future-development-priorities)

---

## 1. Executive Summary

Mindstrata is a **deterministic, emergent human-society simulation** written in Rust. It simulates a small medieval settlement where every person has a full psychological mind, biological needs, social relationships, moral values, and institutional memberships — all interacting to produce emergent history from first principles rather than scripted events.

**Core differentiator:** History emerges from locally bounded minds, social relationships, institutional processes, and material constraints — not from scripted events or global variables.

**Current scale:**
- 17,460 lines of Rust source code
- 266 automated tests (property tests, golden replays, statistical emergence, integration)
- 30 simulation modules covering psychology, behavior, social dynamics, economics, ecology, demography, health, conflict, culture, and institutions
- Deterministic replay from any seed
- CLI with agent psychology inspector, CSV metrics export, scenario runner

**Comparable to:** A hybrid of The Sims (individual psychology), Cities: Skylines (settlements/economy), Age of Empires (historical scale/factions), and Dwarf Fortress (deep emergent simulation) — but with the differentiator of a full cognitive pipeline for every agent.

---

## 2. Core Design Philosophy

### 2.1 Simulation First, GUI Later
The TUI is a **debug instrument**, not the game. The simulation runs headlessly:
```bash
cargo run --release -- --seed 42 --ticks 10000
```

### 2.2 Emergence Through First Principles
Instead of scripting high-level events (`if unrest > 0.8 { spawn_revolt() }`), the system simulates underlying causes: need pressure, emotional appraisal, trust erosion, resource scarcity, institutional legitimacy, social influence, rumor propagation. Revolt becomes an **emergent possibility**, not a hardcoded event.

### 2.3 Agents Are Not Omniscient
Each agent knows only: its body, needs, emotions, memories, relationships, local perception, institutional memberships, rumors heard, and beliefs (which may be false). This is crucial for misinformation, gossip, panic, prejudice, and institutional failure.

### 2.4 Determinism Is Non-Negotiable
Reproducible from: initial world seed + scenario definition + input command log. This is essential for debugging emergent behavior.

---

## 3. Architecture Overview

### 3.1 Layer Cake (10 Substrates)

```
┌─────────────────────────────────────────────┐
│  TUI / Debug Instrument                     │
│  map, agent inspector, event log, metrics   │
├─────────────────────────────────────────────┤
│  Scenario / Experiment Layer                │
│  seeds, shocks, scenarios, assertions       │
├─────────────────────────────────────────────┤
│  Simulation Orchestrator (sim.rs)           │
│  fixed tick loop, schedules, replay         │
├─────────────────────────────────────────────┤
│  Causal Provenance Layer                    │
│  event journal, decision traces             │
├─────────────────────────────────────────────┤
│  Systemic Layer                             │
│  institutions, economy, ecology, conflict   │
├─────────────────────────────────────────────┤
│  Social Layer                               │
│  relationships, factions, gossip, norms     │
├─────────────────────────────────────────────┤
│  Epistemic Layer                            │
│  perception, attention, beliefs, rumors     │
├─────────────────────────────────────────────┤
│  Psychological Layer                        │
│  needs, traits, emotion, memory, identity   │
├─────────────────────────────────────────────┤
│  Behavioral Layer                           │
│  utility AI, planning, habits, actions      │
├─────────────────────────────────────────────┤
│  World Layer                                │
│  space, terrain, sites, resources, time     │
├─────────────────────────────────────────────┤
│  Core Kernel (mindstrata-core)              │
│  ECS, RNG, fixed-point math, events         │
└─────────────────────────────────────────────┘
```

### 3.2 Tick Loop Execution Order

Each tick executes systems in this precise order (deterministic causal chain):

```
 1. Ecology (season advance, fertility decay)
 2. Cognitive State Update (stress → heuristic bias)
 3. Need Decay (nonlinear pressure)
 4. Body State Update (needs → health/energy)
 5. Goal Generation (needs + personality → goals)
 6. Action Execution (utility AI → action → effects)
 7. Social Interactions (proximity-based, 8 types)
 8. Appraisal (events → emotions via Lazarus model)
 9. Emotion Decay (emotions fade over time)
10. Belief Update (evidence + trust → belief change)
11. Memory Encoding/Decay/Rehearsal
12. Health Update (disease, malnutrition, recovery)
13. Institutional Update (collective psychology, norm evaluation)
14. Faction Dynamics (formation, protests)
15. Derived Mental States (trauma, depression, ambition)
16. Ecology Season Advance
17. Demography (aging, births, deaths)
18. Market Update (prices, inequality)
19. Causal Provenance Recording
20. Metric Snapshot
```

---

## 4. Directory Structure & File Map

### 4.1 Workspace Layout
```
mindstrata/
├── Cargo.toml                    # Workspace root (shared deps, lint config)
├── Cargo.lock
├── docs/
│   ├── architecture/
│   │   └── architecture.md       # Full architecture spec (2,500+ lines)
│   ├── bug-report.md
│   └── refactor-plan.md
├── specs/                        # Data-driven game definitions (RON format)
│   ├── ontology.ron              # Entity kinds and ID types
│   ├── components.ron            # Component schemas per entity type
│   ├── systems.ron               # System execution order and phase membership
│   ├── actions.ron               # Action grammar (preconditions, effects, social meaning)
│   ├── norms.ron                 # Social norm definitions (sanctions, enforcement)
│   ├── propositions.ron          # Belief proposition registry (12 propositions)
│   └── scenarios/
│       ├── riverford.ron         # Base village scenario (12 agents, 4320 ticks)
│       └── drought.ron           # Drought stress-test scenario
├── golden/                       # Golden replay test data
│   └── riverford_minor/
│       └── seed_42/
│           └── baseline.json
└── crates/
    ├── mindstrata-core/          # Core kernel (414 lines)
    │   └── src/
    │       ├── lib.rs
    │       ├── fixed.rs          # Fixed-point math (i64-based, deterministic)
    │       ├── rng.rs            # Deterministic RNG streams (ChaCha8)
    │       ├── id.rs             # Type-safe entity/agent IDs
    │       ├── clock.rs          # Deterministic tick clock
    │       ├── event.rs          # Simulation event enum (15+ event types)
    │       ├── error.rs          # Error types
    │       └── proposition.rs    # Proposition types for beliefs
    │
    ├── mindstrata-sim/           # Simulation engine (10,900+ lines)
    │   └── src/
    │       ├── lib.rs
    │       ├── sim.rs            # Main orchestrator (3,191 lines) — THE CORE
    │       ├── person.rs         # Agent entity + all psychological components (617 lines)
    │       ├── world.rs          # World grid, tiles, sites, resources (413 lines)
    │       ├── world_gen.rs      # Procedural village generation (230 lines)
    │       ├── actions.rs        # Action definitions + utility AI selection (469 lines)
    │       ├── appraisal.rs      # Cognitive appraisal → emotion mapping (167 lines)
    │       ├── belief_update.rs  # Bayesian-ish belief updating (156 lines)
    │       ├── social.rs         # Social interactions + relationship evolution (429 lines)
    │       ├── gossip.rs         # Gossip propagation + moral panic detection (611 lines)
    │       ├── norms.rs          # Norm registry + enforcement + crime records (529 lines)
    │       ├── institutions.rs   # Council, Temple, Market + collective psychology (669 lines)
    │       ├── factions.rs       # Faction formation + protests (368 lines)
    │       ├── conflict.rs       # Feuds + combat + escalation (488 lines)
    │       ├── market.rs         # Supply/demand pricing + trade + inequality (542 lines)
    │       ├── black_market.rs   # Illegal trade under scarcity (234 lines)
    │       ├── ecology.rs        # Seasons + overfarming + fertility (270 lines)
    │       ├── demography.rs     # Aging + births + deaths (311 lines)
    │       ├── health.rs         # Disease + injury + malnutrition (303 lines)
    │       ├── cultural.rs       # Knowledge diffusion + practices + taboos (528 lines)
    │       ├── memory.rs         # Episodic/semantic/social memory (270 lines)
    │       ├── attention.rs      # Salience + habituation + focus (193 lines)
    │       ├── provenance.rs     # Causal decision traces (544 lines)
    │       ├── journal.rs        # Event journal (append-only log) (128 lines)
    │       ├── routines.rs       # Daily routines (wake/work/socialize/sleep) (229 lines)
    │       ├── logistics.rs      # Resource transport + inventory (242 lines)
    │       ├── snapshot.rs       # Deterministic save/load (388 lines)
    │       ├── scenario.rs       # Scenario DSL + shock system (141 lines)
    │       ├── spec_lint.rs      # Validates RON specs against code (407 lines)
    │       ├── systems/mod.rs    # System functions called by orchestrator (273 lines)
    │       └── population_cap.rs # Population cap (2 lines — placeholder)
    │
    ├── mindstrata-cli/           # Command-line interface
    │   └── src/
    │       └── main.rs           # CLI entry point with clap
    │
    ├── mindstrata-tui/           # Terminal UI (debug instrument)
    │   └── src/
    │       └── lib.rs            # ASCII map + agent inspector + event log (826 lines)
    │
    └── mindstrata-tests/         # Integration & property tests
        └── src/
            ├── lib.rs            # 181 simulation tests (863 lines)
            ├── property_tests.rs # Proptest property tests (189 lines)
            ├── golden_replay.rs  # Deterministic replay verification (172 lines)
            ├── statistical_emergence.rs  # Statistical pattern tests (142 lines)
            └── comparison.rs     # Config comparison tests (284 lines)
```

### 4.2 Code Size Summary

| Crate | Lines | Public Items | Purpose |
|-------|-------|-------------|---------|
| mindstrata-core | 414 | ~20 | Deterministic kernel (math, RNG, IDs, events) |
| mindstrata-sim | 10,900+ | 350+ | Full simulation engine |
| mindstrata-cli | ~100 | 1 | CLI entry point |
| mindstrata-tui | 826 | ~15 | ASCII debug interface |
| mindstrata-tests | 1,450 | ~50 | Integration + property + golden tests |
| **specs/** | 590 | — | Data-driven RON definitions |
| **Total** | **17,460** | **~435** | |

---

## 5. The 10 Substrates (Layer-by-Layer)

### 5.1 World Layer (`world.rs`, `world_gen.rs`)

**What it does:** 2D grid world with terrain, resource stocks, and named sites.

**Components:**
- `Tile` — terrain type, fertility (0–1), moisture, resource stock
- `Site` — named location (House, Farm, Well, Market, Temple, Council) with inventory and owner
- `ResourceStock` — resource_id + quantity per site

**Sites generated:** Houses (with assigned owners), Farms (fertile), Well (water source), Market (trade), Temple (worship), Council (governance)

**Status:** ✅ **Fully working.** Procedural generation creates a 16×16 village with 12 agents assigned to homes.

---

### 5.2 Psychological Layer (`person.rs`)

**What it does:** Every agent has a full cognitive architecture.

#### 5.2.1 Body State
```rust
pub struct BodyState {
    pub health: Fixed,      // 0–1, degrades from disease/malnutrition
    pub energy: Fixed,      // 0–1, degrades from fatigue/actions
    pub hunger: Fixed,      // 0–1, 0=satisfied, 1=starving
    pub thirst: Fixed,      // 0–1, 0=satisfied, 1=dehydrated
    pub fatigue: Fixed,     // 0–1, 0=rested, 1=exhausted
    pub sickness: Fixed,    // 0–1, from disease
    pub injury: Fixed,      // 0–1, from conflict
    pub fertility: Option<Fixed>,
}
```

#### 5.2.2 Need State (8 Needs)
```rust
pub struct NeedState {
    pub hunger: Fixed,      // Biological survival
    pub thirst: Fixed,      // Biological survival
    pub fatigue: Fixed,     // Rest requirement
    pub safety: Fixed,      // Fear of threats
    pub social: Fixed,      // Loneliness, need for connection
    pub esteem: Fixed,      // Desire for respect/status
    pub autonomy: Fixed,    // Freedom of choice
    pub meaning: Fixed,     // Purpose, spiritual fulfillment
}
```

Need **pressure** is nonlinear: `pressure = deficit^1.5 × personality_modifier × context_modifier`

#### 5.2.3 Personality (12 Traits)
```rust
pub struct Personality {
    pub openness: Fixed,            // Big Five
    pub conscientiousness: Fixed,   // Big Five
    pub extraversion: Fixed,        // Big Five
    pub agreeableness: Fixed,       // Big Five
    pub neuroticism: Fixed,         // Big Five
    pub risk_tolerance: Fixed,      // Extension
    pub conformity: Fixed,          // Extension
    pub ambition: Fixed,            // Extension
    pub altruism: Fixed,            // Extension
    pub traditionalism: Fixed,      // Extension
    pub dominance: Fixed,           // Extension
    pub impulsivity: Fixed,         // Extension
}
```

Each agent gets random traits at birth. Traits modulate **everything**: emotional response, norm compliance, goal selection, stress handling, routine adherence.

#### 5.2.4 Emotional State (Dimensional + Discrete)
```rust
// Dimensional (PAD model)
pub struct Affect {
    pub valence: Fixed,    // Positive/negative
    pub arousal: Fixed,    // Calm/excited
    pub control: Fixed,    // Helpless/in-control
}

// Discrete emotions (8 core)
pub struct DiscreteEmotions {
    pub fear: Fixed,
    pub anger: Fixed,
    pub joy: Fixed,
    pub sadness: Fixed,
    pub trust: Fixed,
    pub shame: Fixed,
    pub pride: Fixed,
    pub guilt: Fixed,
}
```

**Emotions arise from appraisal, not direct assignment.** This is the key psychological innovation.

#### 5.2.5 Appraisal System (`appraisal.rs`)
Implements Lazarus's cognitive appraisal theory. Events are evaluated along 8 dimensions:

| Dimension | Meaning |
|-----------|---------|
| `goal_relevance` | How much does this matter to my goals? |
| `goal_congruence` | Does this help or hurt me? |
| `coping_potential` | Can I do anything about it? |
| `expectedness` | Was this expected? |
| `fairness` | Is this just or unjust? |
| `agency` | Who caused it? (Self / Other / Circumstance) |
| `social_visibility` | How public is this? |
| `identity_relevance` | Does this threaten who I am? |

**Emotion derivation examples:**
- Unfair treatment by another → **anger** (intensified by fairness violation)
- Helplessness against circumstances → **fear + sadness**
- Self-caused failure → **guilt + shame**
- Success from own effort → **pride + joy**
- Low coping potential → **fear intensifies**
- Unexpected events → **fear increases**

#### 5.2.6 Beliefs (`person.rs`, `belief_update.rs`)
```rust
pub struct Belief {
    pub proposition_id: u64,           // Which proposition (12 registered)
    pub confidence: Fixed,             // How sure (0–1)
    pub emotional_charge: Fixed,       // How emotionally loaded
    pub identity_linkage: Fixed,       // How tied to self-concept
    pub resistance: Fixed,             // How hard to change
    pub source: EvidenceSource,        // How acquired (Personal/Peer/Institutional/Hearsay)
    pub social_reinforcement: u32,     // Times confirmed by others
    pub is_accurate: bool,             // Whether based on truth
}
```

**Belief updating (Bayesian-ish with psychological bias):**
```
new_confidence = old_confidence × resistance
               + evidence_strength × source_trust
               + emotional_reinforcement
               + social_reinforcement
```

**Identity-linked beliefs resist change** — this is how ideology, propaganda, and polarization emerge naturally.

#### 5.2.7 Memory (`memory.rs`)
Capacity-limited (200 slots), with:
- **Episodic** — what happened
- **Semantic** — general knowledge
- **Social** — about relationships

Memory **decays** over time, gets **reinforced** by rehearsal, and **distorts** over time. Remembering an event can change the memory.

#### 5.2.8 Attention (`attention.rs`)
Controls which events the agent notices:
- **Salience** = intensity + novelty + survival relevance + social relevance − habituation
- **Habituation** — repeated stimuli become invisible
- **Focus** — current attention target

This allows panic (everything becomes salient), fixation, and ignorance of warning signs.

#### 5.2.9 Cognitive State (`person.rs`)
```rust
pub struct CognitiveState {
    pub attention_capacity: Fixed,    // Base attention (0–1)
    pub executive_capacity: Fixed,    // Base executive function (0–1)
    pub fatigue: Fixed,               // Cognitive fatigue (0–1)
    pub stress: Fixed,               // Current stress (0–1)
    pub planning_horizon: u32,        // Effective planning ticks
    pub heuristic_bias: Fixed,        // 0=systematic, 1=pure heuristics
}
```

**Key mechanic:** Stress reduces planning horizon (20 → as low as 4) and increases heuristic bias. When stressed, agents:
- Rely on habit and routine
- Imitate peers
- Obey authority
- Become impulsive, aggressive, or fearful
- Simplify beliefs

#### 5.2.10 Derived Mental States (`person.rs`)
Computed from lower-level variables, not directly set:
```rust
pub struct DerivedMentalState {
    pub trauma_risk: Fixed,      // From repeated stress + low coping
    pub depression_risk: Fixed,  // From chronic deficit + low meaning
    pub ambition: Fixed,         // From success + energy
    pub resilience: Fixed,       // From social support + coping
    pub resentment: Fixed,       // From perceived injustice
}
```

These accumulate slowly over ticks and influence traits over time — **psychology is plastic**.

**Status:** ✅ **All psychological systems fully working.** Complete cognitive pipeline from perception through action.

---

### 5.3 Behavioral Layer (`actions.rs`, `routines.rs`)

**Action primitives:** Eat, Drink, Rest, Work, Socialize, Worship, Wander, Idle, Threaten, Move

**Decision priority:**
1. **Critical needs override** (hunger > 0.9, thirst > 0.9, fatigue > 0.95 → forced action)
2. **Routine bias** — agents follow daily schedules (conscientiousness boosts adherence, openness reduces it)
3. **Feud pursuit** — angry agents with feuds move toward their rival
4. **Utility AI** — weighted scoring: need relief + emotional relief + social effects + identity congruence + norm compliance + cost + risk + noise

**Intentions:** Once an action is selected, the agent forms an **intention** with commitment level. Agents don't abandon intentions easily (conscientious agents persist through failures).

**Daily Routines:** Predefined schedules (06:00 wake → 08:00 work → 12:00 socialize → 18:00 home → 22:00 sleep) create behavioral stability. Emergence happens when routines are disrupted.

**Status:** ✅ **Fully working.** Utility AI + routine bias + intention commitment + critical need override.

---

### 5.4 Social Layer (`social.rs`, `gossip.rs`, `norms.rs`)

#### 5.4.1 Relationships
Every pair of agents has a directed relationship:
```rust
pub struct Relationship {
    pub from: AgentId,
    pub to: AgentId,
    pub trust: Fixed,
    pub affection: Fixed,
    pub respect: Fixed,
    pub fear: Fixed,
    pub obligation: Fixed,
    pub kind: RelationshipKind,        // Stranger → Neighbor → Friend/Rival → Kin
    pub interaction_count: u32,
    pub last_positive_tick: u64,
    pub last_negative_tick: u64,
}
```

**Relationship evolution:** 5+ interactions → Stranger becomes Neighbor. High trust + affection → Friend. Low trust → Rival. Friendship can downgrade.

#### 5.4.2 Interactions (8 Types)
| Type | Trust Δ | Affection Δ |
|------|---------|-------------|
| Talk | +0.01 | +0.005 |
| Help | +0.05 | +0.03 |
| Trade | +0.02 | 0 |
| Gossip | +0.005 | +0.01 |
| Comfort | +0.03 | +0.05 |
| Threaten | −0.10 | −0.05 |
| Insult | −0.08 | −0.10 |
| Teach | +0.04 | +0.02 |

**Witness system:** Other agents observe interactions and update their trust judgments accordingly.

**Proximity-based:** Agents can only interact within 5 tiles Manhattan distance. Creates natural neighborhoods.

**In-group bias:** Faction members get +0.1 trust bonus on positive interactions, −0.05 penalty on negative interactions with outsiders.

#### 5.4.3 Gossip (`gossip.rs`)
Rumors spread through the social graph as a **telephone game with emotional mutation:**
```
rumor_accuracy = memory_accuracy × source_trust × transmission_fidelity × emotional_salience × identity_bias
```

Each hop degrades accuracy by ~15%. Angry spreaders exaggerate. Traditionalist listeners accept authority-aligned rumors more easily.

**Moral panic detection:** When 30%+ of agents hold high-emotional-charge beliefs about an institution → sudden legitimacy collapse.

#### 5.4.4 Norms (`norms.rs`)
5 default norms: No Theft, Help Neighbors, Respect Elders, Obey Ruler, No Violence

Each has: strength, internalization, punishment severity, reinforcing identity.

**Norm pressure formula:**
```
pressure = norm.strength × (1 − conformity) − identity_strength × 0.3 − internalization
```

**Enforcement is probabilistic** — not all violations are caught (based on Council enforcement capacity).

**Escalating punishment:** CrimeRecord tracks offenses per agent. First offense = 1×, second = 1.5×, third = 2×, max 3×.

**Status:** ✅ **Fully working.** Relationships evolve, gossip mutates and spreads, moral panics can trigger, norms are enforced with escalating punishment.

---

### 5.5 Systemic Layer (`institutions.rs`, `factions.rs`, `conflict.rs`)

#### 5.5.1 Institutions
Three default institutions, each with full internal state:

| Institution | Roles | Function |
|-------------|-------|----------|
| **Council** | Elder, Guard Captain | Governance, law enforcement, taxation |
| **Temple** | Priest | Religious ritual, meaning provision |
| **Market** | Merchant | Trade, price formation, economic coordination |

Each institution has: `legitimacy, cohesion, enforcement_capacity, collective psychology (morale, grievance, trust in leadership)`

**Role assignment** is personality-based (not hardcoded):
- Elder: highest conscientiousness + agreeableness
- Priest: highest traditionalism + agreeableness
- Merchant: highest ambition + dominance
- Guard Captain: highest dominance + conscientiousness

#### 5.5.2 Factions (`factions.rs`)
Factions form when enough agents share: high resentment + low institutional trust + similar moral values + common grievance.

Factions have collective psychology (morale, cohesion, grievance) derived from member states.

**Protests:** Factions with high grievance and low institutional legitimacy can organize protests, which further erode legitimacy.

#### 5.5.3 Conflict (`conflict.rs`)
- **Feuds:** Agents can enter active feuds. Feuding agents move toward their rival, attempt threats/attacks.
- **Combat:** Physical confrontation with injury risk, trauma accumulation.
- **Escalation/De-escalation:** Based on outcomes, relationship damage, and emotional state.

**Status:** ✅ **Fully working.** Institutions have roles and collective psychology, factions form and protest, feuds escalate and resolve.

---

### 5.6 Economic Layer (`market.rs`, `black_market.rs`, `logistics.rs`)

#### 5.6.1 Market (`market.rs`)
**Supply/demand pricing:** Prices form from aggregate supply and demand across all sites:
- High demand + low supply → price rises
- Low demand + high supply → price falls
- Exponential moving average for smoothing
- Price floor (1.0) and ceiling (50.0)

**Trade mechanics:**
- Market trades: at market sites, pay coin for goods
- Direct trades: agent-to-agent, trust modifies price (high trust = 80% price, low trust = 120%)

**Inequality tracking:** Gini coefficient computed every tick.

#### 5.6.2 Black Market (`black_market.rs`)
Activates under scarcity + weak enforcement. Agents trade illegally when norm compliance is low and resources are scarce. Creates economic pressure and social tension.

#### 5.6.3 Logistics (`logistics.rs`)
Resource transport between sites, inventory management, carrying capacity.

**Status:** ✅ **Fully working.** Prices emerge from supply/demand, trade executes, inequality tracked, black market activates under pressure.

---

### 5.7 Ecological Layer (`ecology.rs`)

**Seasons:** Spring → Summer → Autumn → Winter (each ~8760 ticks = ~1 year)
- Spring: moderate growth, low spoilage
- Summer: peak growth, high spoilage
- Autumn: declining growth, low spoilage
- Winter: **NO growth**, very low spoilage

**Overfarming:** Work pressure degrades fertility. Too much farming → depleted land → food shortage → famine → migration pressure.

**Migration pressure:** Computed from local food scarcity, season, and shelter availability.

**Status:** ✅ **Fully working.** Seasons cycle, overfarming degrades land, migration pressure emerges from scarcity.

---

### 5.8 Health Layer (`health.rs`)

**5 disease types:**
| Disease | Severity | Transmission | Duration |
|---------|----------|-------------|----------|
| Cold | 0.1 | 5% per contact | 200 ticks |
| Fever | 0.3 | 3% per contact | 400 ticks |
| Wound Infection | 0.5 | Not contagious | 600 ticks |
| Malnutrition | 0.4 | Not contagious | 1000 ticks |
| Epidemic | 0.7 | 15% per contact | 800 ticks |

**Disease mechanics:** Transmission is proximity-based. Healthy agents resist better. Disease severity peaks at 1/3 duration then declines. Natural recovery when healthy and fed.

**Injury:** Conflict interactions can cause injury → wound infection → chronic health drain.

**Status:** ✅ **Fully working.** Diseases transmit, resolve, and interact with health/nutrition.

---

### 5.9 Demographic Layer (`demography.rs`)

**Life stages:** Child (0–12) → Adolescent (12–18) → Adult (18–60) → Elder (60+)

**Aging:** Age increases every tick. Elders face increasing death probability.

**Births:** Requires partner, childbearing age (18–45), minimum health. Birth rate decreases with existing children.

**Status:** ✅ **Fully working.** Agents age, can die, can have children.

---

### 5.10 Cultural Layer (`cultural.rs`)

**Knowledge system:** 5 seeded knowledge types (Crop Rotation, Well Maintenance, Herbal Medicine, Harvest Prayer, Grain Storage) with categories (Agricultural, Craft, Medical, Philosophical).

**Knowledge diffusion:** Spreads through social networks based on teacher skill, student openness, trust, and knowledge difficulty.

**Cultural pressure:** Traditional agents feel more pressure to conform to widespread practices. Open agents resist.

**Taboo system:** Forbidden practices with severity scaling by witness count and traditionalism.

**Status:** ✅ **Fully working.** Knowledge diffuses, cultural pressure modulates behavior, taboos have severity.

---

### 5.11 Causal Provenance Layer (`provenance.rs`, `journal.rs`)

Every important decision is recorded with:
- Agent, tick, action name
- Decision factors (hunger, thirst, fatigue, norm pressure, routine strength)
- Whether the agent was interrupted by critical needs
- Whether an intention was abandoned

Event journal provides an append-only log of all significant events.

**Status:** ✅ **Fully working.** Full decision trace audit trail for debugging emergence.

---

### 5.12 Core Kernel (`mindstrata-core`)

| Module | Purpose |
|--------|---------|
| `fixed.rs` | Fixed-point math (i64-based, 4 decimal places). Deterministic across platforms. |
| `rng.rs` | 7 separate RNG streams (World, Psychology, Behavior, Social, Economy, Ecology, Narrative) for debuggability |
| `id.rs` | Type-safe IDs: AgentId, EntityId, ResourceId, EventId, etc. |
| `clock.rs` | Deterministic tick counter |
| `event.rs` | 15+ event types (AgentAte, RelationshipChanged, NormViolated, TradeOccurred, etc.) |

**Status:** ✅ **Fully working.** Deterministic, debuggable, type-safe.

---

## 6. What Fully Works (Complete Systems)

### ✅ Full Cognitive Pipeline
Perception → Attention → Appraisal → Emotion → Belief Update → Goal Formation → Intention → Action Selection → Execution → Feedback

### ✅ 12-Trait Personality System
Random generation, modulates all behavior

### ✅ 8-Emotion Appraisal System
Lazarus model, emotions arise from event evaluation

### ✅ Belief System with 12 Propositions
Bayesian-ish update, identity protection, social reinforcement, source tracking

### ✅ Memory System
Capacity-limited, decays, reinforces, distorts

### ✅ Attention System
Salience-based, habituation, focus control

### ✅ Cognitive State (Bounded Rationality)
Stress → reduced planning, increased heuristic reliance

### ✅ Derived Mental States
Trauma, depression, ambition, resilience, resentment — all computed from lower-level variables

### ✅ Intention Commitment
Agents persist through failures, abandon under shock/stress

### ✅ Daily Routines
Schedules with personality modulation

### ✅ Utility AI Action Selection
Weighted scoring with noise, routine bias, feud pursuit

### ✅ 8-Type Social Interactions
Talk, Help, Trade, Gossip, Comfort, Threaten, Insult, Teach

### ✅ Relationship Evolution
Stranger → Neighbor → Friend/Rival, based on interaction history

### ✅ Proximity-Based Social Graph
5-tile perception radius, natural neighborhoods

### ✅ In-Group/Out-Group Bias
Faction members trust insiders, distrust outsiders

### ✅ Gossip with Emotional Mutation
Telephone game, accuracy degrades, angry spreaders exaggerate

### ✅ Moral Panic Detection
High emotional charge + widespread → legitimacy collapse

### ✅ 5 Norms with Enforcement
Probabilistic detection, escalating punishment, crime records

### ✅ 3 Institutions with Roles
Council, Temple, Market — personality-based role assignment

### ✅ Collective Psychology
Institutions have morale, grievance, trust derived from members

### ✅ Faction Formation & Protests
Emerges from shared resentment + low institutional trust

### ✅ Feuds & Conflict
Active feuds, combat, injury, trauma, escalation/de-escalation

### ✅ Supply/Demand Pricing
Emergent prices from aggregate supply and demand

### ✅ Market Trade + Direct Trade
Trust-modified pricing, Gini inequality tracking

### ✅ Black Market
Activates under scarcity + weak enforcement

### ✅ Seasons & Overfarming
4 seasons, fertility decay from overwork, winter stops growth

### ✅ Disease System
5 disease types, proximity transmission, severity curves

### ✅ Demography
Aging, births, deaths, life stages

### ✅ Cultural Knowledge Diffusion
5 knowledge types, teaching/learning, taboo system

### ✅ Deterministic Replay
Seed → identical simulation, snapshot save/load

### ✅ Causal Provenance
Full decision trace audit trail

### ✅ Spec Linting
Validates RON specs against code for consistency

### ✅ 266 Automated Tests
Property tests, golden replays, statistical emergence, integration

---

## 7. Known Implementation Gaps

### 7.1 CRITICAL GAPS (Block major features)

| Gap | Impact | Current State |
|-----|--------|---------------|
| **No visual rendering** | Cannot see the world | ASCII TUI exists but is debug-only, not a playable interface |
| **No player interaction** | Cannot play the game | CLI-only, no interactive mode |
| **No save/load UI** | Cannot resume games | Snapshot system works but no UI to trigger it |
| **No multi-settlement** | Single village only | World is one 16×16 grid, no inter-settlement trade/war |
| **No technology tree** | No progression | Knowledge exists but no unlock chains or prerequisites |

### 7.2 SIGNIFICANT GAPS (Limit depth)

| Gap | Impact | What Exists |
|-----|--------|-------------|
| **No marriage/family formation** | Agents start paired randomly | `partner` and `parent_a/b` fields exist but are never set during runtime |
| **No childhood socialization** | No generational knowledge transfer | Children exist conceptually but aren't born during simulation |
| **No property rights enforcement** | No real ownership disputes | Sites have `owner` field but no legal framework |
| **No contracts/debts** | No IOUs, loans, or promises | `obligation` field in relationships but no formal system |
| **No education system** | Knowledge diffusion is ad-hoc | No schools, apprenticeships, or structured learning |
| **No legal system** | No courts, trials, or adjudication | Norms exist but no formal justice process |
| **No migration between settlements** | Agents never leave | Migration pressure computed but no destination system |
| **No weather/climate** | Seasons only affect growth | No droughts, floods, or temperature extremes (scenario shocks are manual) |
| **No resource spoilage in inventory** | Food doesn't rot in agent inventory | Spoilage modifier exists in ecology but not applied to agent stocks |

### 7.3 MODERATE GAPS (Limit realism)

| Gap | Impact | What Exists |
|-----|--------|-------------|
| **No household collective decisions** | Households don't function as units | House sites exist but no household-level decision making |
| **No status competition** | No prestige dynamics | `StatusState` exists but doesn't drive behavior |
| **No reputation network** | Reputation is implicit in trust | No gossip-about-reputation system |
| **No ritual/religious mechanics** | Worship is a simple action | No ritual effects, religious calendar, or theological beliefs |
| **No innovation/emergence of new knowledge** | Only 5 seeded knowledge types | No discovery system, no research |
| **No cultural evolution** | Practices are static | `CulturalState` exists but practices don't emerge or die |
| **No wealth inheritance** | Dead agents' wealth disappears | No inheritance mechanism |
| **No trade routes** | Single market only | No inter-site or inter-settlement trade |
| **No military/army system** | Conflict is individual, not collective | Feuds exist but no organized warfare |
| **No propaganda/information control** | No institutional messaging | Gossip exists but institutions can't deliberately spread narratives |

### 7.4 MINOR GAPS (Polish items)

| Gap | Notes |
|-----|-------|
| **No agent naming beyond 24 names** | Names cycle after 24 agents |
| **No terrain variety** | All tiles are grassland (no forests, mountains, rivers) |
| **No seasons affecting agent clothing/behavior** | Seasons only affect ecology |
| **No day/night cycle in behavior** | Routines have time-of-day but no darkness effects |
| **No sound/music** | CLI-only, no audio |
| **No modding API** | RON files are data-driven but no formal modding interface |
| **No replay viewer** | Golden replay tests exist but no interactive replay UI |

---

## 8. Data-Driven Specifications (RON Files)

All game definitions are in `specs/` using RON (Rusty Object Notation):

| File | Purpose | Contents |
|------|---------|----------|
| `ontology.ron` | Entity taxonomy | 10 entity kinds, 25 ID types |
| `components.ron` | Component schemas | Which components belong to Person/Institution/World |
| `systems.ron` | Execution order | 15 phases, 25+ systems with read/write dependencies |
| `actions.ron` | Action grammar | 9 actions with preconditions, effects, social meaning |
| `norms.ron` | Social norms | 6 norms with sanctions, enforcement, scope |
| `propositions.ron` | Belief registry | 12 propositions agents can believe about |
| `scenarios/riverford.ron` | Base scenario | 12 agents, 16×16 world, festival + mild drought |
| `scenarios/drought.ron` | Stress test | Same world with severe drought at tick 500 |

**Spec linting** (`spec_lint.rs`) validates that code matches these RON definitions — catches orphaned components, missing systems, undefined actions.

---

## 9. Test Coverage & Quality

### 9.1 Test Breakdown

| Test Category | Count | Purpose |
|---------------|-------|---------|
| Unit tests (in modules) | ~180 | Per-system correctness |
| Integration tests | 48 | Full simulation runs (100–500 ticks) |
| Property tests | 10 | Proptest: determinism, idempotency, bounds |
| Golden replay tests | 3 | Identical output from same seed |
| Statistical emergence tests | 5 | Verify patterns emerge (trade volume, relationship evolution) |
| Comparison tests | 3 | Config comparison across scenarios |
| **Total** | **266** | |

### 9.2 Quality Metrics

| Metric | Value |
|--------|-------|
| Tests passing | 266/266 (100%) |
| Clippy warnings | 2 (low-impact `default_trait_access`) |
| Unsafe code | 0 (`unsafe_code = "forbid"` in workspace lints) |
| Determinism | Verified by golden replay tests |
| Cross-platform | Fixed-point math ensures identical results |

---

## 10. Technology Stack & Dependencies

### 10.1 Core Stack
| Purpose | Crate | Version |
|---------|-------|---------|
| Language | Rust | 2024 edition |
| Math | Custom `Fixed` type (i64, 4 decimals) | — |
| RNG | `rand` + `rand_chacha` (ChaCha8) | 0.9 |
| Serialization | `serde` + `ron` + `postcard` + `serde_json` | 1.0 / 0.8 / 1.0 / 1.0 |
| Logging | `tracing` + `tracing-subscriber` | 0.1 / 0.3 |
| CLI | `clap` (derive) | 4.0 |
| Errors | `thiserror` | 2.0 |
| Graphs | `petgraph` | 0.7 |
| TUI | `ratatui` + `crossterm` | 0.29 / 0.28 |
| Testing | `proptest` + `insta` | 1.0 |

### 10.2 Lint Configuration
```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }
cast_sign_loss = "allow"
cast_possible_truncation = "allow"
cast_precision_loss = "allow"
cast_possible_wrap = "allow"
float_cmp = "allow"
module_name_repetitions = "allow"
must_use_candidate = "allow"
missing_errors_doc = "allow"

[workspace.lints.rust]
unsafe_code = "forbid"
```

---

## 11. How to Run

```bash
# Basic simulation (12 agents, 500 ticks)
cargo run --release -- --seed 42 --ticks 500

# Drought scenario
cargo run --release -- --scenario specs/scenarios/drought.ron --ticks 2000

# Inspect agent #3's full psychology
cargo run --release -- --seed 42 --ticks 100 --psychology 3

# Export metrics to CSV
cargo run --release -- --seed 42 --ticks 500 --export-metrics metrics.csv

# Run all tests
cargo test

# Run specific test suite
cargo test -p mindstrata-tests

# Check for warnings
cargo clippy --all-targets

# Lint RON specs against code
cargo test spec_lint
```

---

## 12. Recommended Future Development Priorities

### Phase 1: Deepen Existing Systems (High Impact)
1. **Marriage & Family Formation** — agents find partners based on compatibility, form households, have children who inherit traits
2. **Household Decision Making** — households pool resources, make collective choices
3. **Status Competition** — wealth, role, and social status drive prestige dynamics
4. **Propaganda & Information Control** — institutions can deliberately spread narratives through trusted agents

### Phase 2: Expand World (Medium Impact)
5. **Multi-Settlement** — neighboring villages with trade routes, diplomacy, conflict
6. **Technology Tree** — knowledge prerequisites, innovation chains, cultural evolution
7. **Weather System** — temperature, rainfall, droughts, floods affecting agriculture
8. **Resource Spoilage** — food rots in inventory, requiring storage infrastructure

### Phase 3: Add Depth (Polish)
9. **Legal System** — courts, trials, property rights enforcement
10. **Education System** — schools, apprenticeships, structured knowledge transfer
11. **Religious Mechanics** — rituals, theological beliefs, religious calendar
12. **Military System** — organized warfare, conscription, battle outcomes

### Phase 4: Make It Playable (Product)
13. **Interactive TUI** — player can select agents, issue commands, trigger events
14. **Visual Rendering** — 2D map with agent sprites, resource indicators
15. **Save/Load UI** — snapshot management through the interface
16. **Modding API** — formal interface for custom scenarios, rules, and content

---

*This document reflects the codebase state as of July 30, 2026. The simulation engine is architecturally complete with 10 interconnected substrates, but many systems are at "v1" depth — functional but not yet rich enough for deeply emergent gameplay. The foundation is solid; the work ahead is deepening, expanding, and making it playable.*
