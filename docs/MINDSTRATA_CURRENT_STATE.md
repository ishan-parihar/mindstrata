# Mindstrata — Current State Technical Document

**Prepared for:** Lead Game Designer  
**Date:** August 5, 2026  
**Codebase Version:** 37,797 lines of Rust across 5 crates, 613+ tests passing, 0 clippy warnings

---

## Table of Contents

1. [Executive Summary](#1-executive-summary)
2. [Core Design Philosophy](#2-core-design-philosophy)
3. [Architecture Overview](#3-architecture-overview)
4. [Directory Structure & File Map](#4-directory-structure--file-map)
5. [The 10 Substrates (Layer-by-Layer)](#5-the-10-substrates-layer-by-layer)
6. [What Fully Works (Complete Systems)](#6-what-fully-works-complete-systems)
7. [Architecture Plan 2 Implementation Status](#7-architecture-plan-2-implementation-status)
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
- 37,797 lines of Rust source code
- 613+ automated tests (property tests, golden replays, statistical emergence, integration, 10K-tick stability)
- 60+ simulation modules covering biology, psychology, social dynamics, economics, ecology, demography, health, conflict, culture, institutions, and noospheric fields
- Deterministic replay from any seed
- CLI with agent psychology inspector, CSV metrics export, scenario runner
- Architecture Plan 2 (AP2.md) ~99% complete (dead-system sweep Iter 22–28 finished)

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

### 3.1 Layer Cake (12 Substrates)

```
┌──────────────────────────────────────────────────────┐
│ Debug / TUI / Inspector / Replay                     │
├──────────────────────────────────────────────────────┤
│ Scenario / Experiment / Shock Layer                  │
├──────────────────────────────────────────────────────┤
│ Simulation Orchestrator                              │
├──────────────────────────────────────────────────────┤
│ Provenance / Explanation / Metrics                   │
├──────────────────────────────────────────────────────┤
│ Noospheric Layer                                     │
│ memes, rumors, propaganda, collective memory,        │
│ narrative frames, legitimacy, moral panic            │
├──────────────────────────────────────────────────────┤
│ Cultural Layer                                       │
│ practices, taboos, rituals, knowledge, education     │
├──────────────────────────────────────────────────────┤
│ Institutional Layer                                  │
│ council, temple, market, guild, household, clan      │
├──────────────────────────────────────────────────────┤
│ Social Layer                                         │
│ relationships, attachment, status, kinship, factions │
├──────────────────────────────────────────────────────┤
│ Behavioral Layer                                     │
│ utility AI, intentions, habits, skills, speech acts  │
├──────────────────────────────────────────────────────┤
│ Psychological Layer                                  │
│ self-model, theory of mind, identity, morality,      │
│ emotion regulation, narrative, imagination           │
├──────────────────────────────────────────────────────┤
│ Cognitive Layer                                      │
│ attention, memory, belief, inference, learning       │
├──────────────────────────────────────────────────────┤
│ Biological Layer                                     │
│ genome, hormones, organs, nervous system, metabolism │
├──────────────────────────────────────────────────────┤
│ World Layer                                          │
│ space, terrain, sites, resources, time, ecology      │
├──────────────────────────────────────────────────────┤
│ Core Kernel                                          │
│ ECS-like stores, RNG, fixed-point, IDs, events       │
└──────────────────────────────────────────────────────┘
```

### 3.2 Multi-Timescale Scheduler (§6)

The simulation uses a formalized 10-phase scheduler:

| Phase | Interval | Systems |
|---|---|---|
| Fast | 1 tick | Perception, action, autonomic, emotion |
| Hourly | 6 ticks | Hormonal fast modulation |
| Deca | 10 ticks | Body state, demography, conflict |
| Duodeca | 12 ticks | Ritual execution |
| Centum | 100 ticks | Institutional decisions, policy |
| Daily | 144 ticks | Memory consolidation, decay, culture |
| Quincent | 500 ticks | Feud decay, hierarchy |
| Weekly | 1008 ticks | Status recalibration, faction dynamics |
| Seasonal | 4320 ticks | Developmental, demographic shifts |
| Yearly | 51840 ticks | Climate, culture drift, technology |

1 tick = 10 simulated minutes.

### 3.3 Tick Loop Execution Order

Each tick executes systems in this precise order (deterministic causal chain):

```
 1. Scenario shocks
 2. Time advance / circadian / season
 3. Biological update (EmbodiedState.tick_update)
 4. Trust network sync
 5. Cognitive state update (stress → heuristic bias)
 6. Executive function update
 7. Attachment system daily decay
 8. Motivation update
 9. Emotion regulation strategy selection
10. Moral cognition update
11. Prospection (mental simulation)
12. Narrative identity interpretation
13. Developmental psychology
14. Psychopathology update
15. Cultural cognition + decision policy
16. Skill/habit update
17. Status dimensions update
18. Relationship decay (level-of-detail cached)
19. Need decay (nonlinear pressure)
20. Body state update
21. Goal generation
22. Action execution (utility AI → effects)
23. Social interactions (proximity-based)
24. Appraisal (events → emotions via Lazarus model)
25. Emotion decay
26. Belief update
27. Gossip propagation
28. Norm evaluation
29. Memory encoding
30. Marriage formation
31. Birth mechanics
32. Kinship/household daily pass
33. Causal provenance recording
34. Metric snapshot
```

---

## 4. Directory Structure & File Map

### 4.1 Workspace Layout
```
mindstrata/
├── Cargo.toml                    # Workspace root
├── Cargo.lock
├── docs/
│   ├── architecture/
│   │   ├── architecture-part-2.md  # Human-Scale Deepening Plan
│   │   └── AP2.md                  # Architecture Plan 2 (4,060+ lines)
│   └── MINDSTRATA_CURRENT_STATE.md # This document
├── specs/                        # Data-driven game definitions (RON)
│   ├── ontology.ron
│   ├── components.ron
│   ├── systems.ron
│   ├── actions.ron
│   ├── norms.ron
│   ├── propositions.ron
│   ├── biology/                  # §15.1 biological data specs
│   │   ├── genome.ron
│   │   ├── hormones.ron
│   │   ├── organs.ron
│   │   ├── diseases_v2.ron
│   │   ├── life_stages.ron
│   │   └── reproduction.ron
│   ├── psychology/               # §15.1 psychological data specs
│   │   ├── cognitive_systems.ron
│   │   ├── emotions_v2.ron
│   │   ├── regulation_strategies.ron
│   │   ├── identity_frames.ron
│   │   └── moral_foundations.ron
│   ├── social/                   # §15.1 social data specs
│   │   ├── courtship.ron
│   │   ├── marriage.ron
│   │   ├── kinship.ron
│   │   └── status_roles.ron
│   └── culture/                  # §15.1 cultural data specs
│       ├── rituals.ron
│       ├── propaganda.ron
│       ├── taboos_v2.ron
│       ├── sacred_symbols.ron
│       └── education.ron
├── golden/                       # Golden replay test data
│   └── riverford_minor/seed_42/baseline.json
└── crates/
    ├── mindstrata-core/          # Core kernel
    ├── mindstrata-sim/           # Simulation engine (30,000+ lines)
    │   └── src/
    │       ├── biology/          # §7 Biological systems (2,557 lines)
    │       ├── psychology/       # §8 Psychological systems (4,182 lines)
    │       ├── social/           # §10 Social systems (17 files)
    │       ├── culture/          # §13 Cultural systems (3,677 lines)
    │       ├── noosphere/        # §13 Noospheric systems (858 lines)
    │       └── ...               # Core simulation modules
    ├── mindstrata-cli/           # Command-line interface
    ├── mindstrata-tui/           # Terminal UI (debug instrument)
    └── mindstrata-tests/         # Integration & property tests
```

### 4.2 Code Size Summary

| Area | Lines | Modules |
|---|---|---|
| Biology (§7) | 2,557 | 11 modules + mod.rs |
| Psychology (§8) | 4,182 | 16 modules + mod.rs |
| Social (§10) | ~8,000 | 17 files |
| Culture (§13) | 3,677 | 14 modules + mod.rs |
| Noosphere (§13) | 858 | 5 files |
| Core simulation | ~18,500 | sim.rs, person.rs, world.rs, etc. |
| **Total** | **37,797** | **60+ modules** |

---

## 5. The 10 Substrates (Layer-by-Layer)

### 5.1 World Layer
2D grid world with terrain, resource stocks, and named sites. ✅ **Fully working.**

### 5.2 Biological Layer (§7)
Rich embodied substrate with 11 biological systems:

| System | Struct | Purpose |
|---|---|---|
| Genome | `Genome` | Heritable predispositions, trait inheritance |
| Endocrine | `EndocrineState` | 7 hormonal axes (stress/bonding/dominance/fertility/metabolic/arousal/growth) |
| Metabolism | `MetabolicState` | Energy conversion, nutrient processing |
| Cardiovascular | `CardiovascularState` | Heart, circulation, fitness |
| Respiratory | `RespiratoryState` | Lungs, oxygenation, endurance |
| Musculoskeletal | `MuscularState` | Strength, endurance, fatigue |
| Nervous | `NervousSystemState` | Autonomic arousal, pain, trauma |
| Immune | `ImmuneState` | Disease resistance, inflammation |
| Reproductive | `ReproductiveState` | Fertility, pregnancy, pair-bonding |
| Circadian | `CircadianState` | Sleep pressure, rhythm |
| Development | Developmental stages | Aging, maturation |

Integrated via `EmbodiedState` with compatibility facade (`derived_health()`, `derived_energy()`, etc.). ✅ **Fully working.**

### 5.3 Psychological Layer (§8)
16 psychological systems forming a structured mind:

| System | Struct | Purpose |
|---|---|---|
| Interoception | `InteroceptiveState` | Body → felt experience |
| Cognitive Runtime | `CognitiveRuntime` | Executive function, working memory, inhibition |
| Emotion Regulation | `EmotionRegulationState` | 12 regulation strategies |
| Self Model | `SelfModel` | Identity claims, roles, values, self-esteem |
| Theory of Mind | Theory of mind | Other-model inference |
| Moral Cognition | `MoralCognition` | 6 moral foundations, moral emotions |
| Imagination | `ProspectionState` | Mental simulation, hope/dread |
| Narrative | `NarrativeIdentity` | Life themes, redemption/contamination scripts |
| Attachment | `AttachmentSystem` | Secure/anxious/avoidant/disorganized styles |
| Developmental | `DevelopmentalPsychState` | Life stages, developmental processes |
| Psychopathology | `PsychopathologyState` | Depression, anxiety, PTSD, paranoia risk dynamics |
| Skill | `SkillState` | Skills, habits, automaticity |
| Motivation | Motivation system | Biological + psychological needs |
| Cultural Cognition | `RitualScript` | Cultural categories, taboos, honor codes |
| Decision Policy | `DecisionPolicy` | Integrates utility/morality/identity/habit |

✅ **Fully working.**

### 5.4 Social Layer (§10)
17 files covering relationships, status, groups:

| System | Key Structs | Purpose |
|---|---|---|
| Relationship V2 | `RelationshipV2` | 20+ relational dimensions, stages, history |
| Attraction | `AttractionModel` | 10-factor attraction model |
| Courtship | `Courtship` | Courtship pipeline |
| Marriage | `Marriage`, `PairBond` | Marriage + pair-bonding |
| Kinship | `KinshipGraph` | Biological/marriage/adoption/ritual links |
| Household | `Household` | Members, pooled resources, cohesion |
| Clan | `Clan`, `ClanRegistry` | Clan dynamics |
| Patronage | `PatronageRelation` | Patron-client relations |
| Hierarchy | Hierarchy formation | Power dynamics |
| Group Formation | `PeerGroup`, `GroupRegistry` | Group emergence |
| Cult | `CultDynamics` | Cult formation dynamics |
| Faction V2 | `FactionV2` | Faction dynamics |
| Status | `StatusDimensions` | Dominance/prestige/authority/legitimacy/wealth/honor |
| Epistemic | `EpistemicState`, `TrustNetwork` | Information processing |

✅ **Fully working.**

### 5.5 Cultural & Noospheric Layer (§13)
14 cultural + 5 noospheric modules:

| System | Key Structs | Purpose |
|---|---|---|
| Meme | `Meme`, `MemeRegistry` | 14 content types, virality, mutation |
| Meme Aggregator | `MemeAggregator` | Performance optimization |
| Rumor V2 | `RumorV2`, `RumorRegistry` | Accusation severity, evidence quality |
| Propaganda | `PropagandaCampaign` | Institutional narrative shaping |
| Ritual | `Ritual`, `RitualRegistry` | Cohesion technology |
| Collective Memory | `CollectiveMemory` | Shared history |
| Echo Chamber | `EchoChamberState` | Polarization tracking |
| Sacred | `SacredValue` | Sacred boundary protection |
| Education | Education system | Knowledge transfer |
| Narrative Frame | Narrative interpretation | Event meaning-making |
| Ideology | Ideology system | Belief systems |
| Knowledge | Knowledge diffusion | Category-based learning |
| Noospheric Field | `NoosphericField` | Spreading activation |
| Belief Ecology | `BeliefEcologyNoosphere` | Narrative clusters |
| Legitimacy | `LegitimacyField` | Institutional legitimacy |
| Moral Panic | `MoralPanic` | Panic dynamics |

✅ **Fully working.**

### 5.6 Level-of-Detail Cognition (§17)
- `AgentTier` system: focal → secondary → background → dormant
- `CognitiveBudgetTracker`: per-tick appraisal/prospection budget limits
- Background agents use simplified heuristics
- Dormant relationships skip per-tick decay

### 5.7 Causal Provenance (§16)
- 13 `ProvenanceCategory` variants
- `SystemTrace` records cross-system causal influences
- `DecisionTrace` captures action selection factors

---

## 6. What Fully Works (Complete Systems)

### ✅ Full Cognitive Pipeline
Perception → Attention → Appraisal → Emotion → Belief Update → Goal Formation → Intention → Action Selection → Execution → Feedback → Learning → Self-Model Update

### ✅ Embodied Biological Substrate
11 biological systems feeding into psychology via endocrine/nervous signals

### ✅ 12-Trait Personality System
Random generation, modulates all behavior

### ✅ 22-Emotion Appraisal System
Lazarus model with expanded dimensions (moral violation, identity threat, sacredness violation, etc.)

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
Weighted scoring with noise, routine bias, feud pursuit, identity congruence

### ✅ 8-Type Social Interactions
Talk, Help, Trade, Gossip, Comfort, Threaten, Insult, Teach

### ✅ Relationship V2 with Stages
20+ relational dimensions, romantic/social/authority/kin stages

### ✅ Courtship, Marriage, Kinship
Courtship pipeline → marriage → household → clan formation

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

### ✅ Meme Propagation
14 content types, virality, mutation, institutional backing

### ✅ Propaganda Campaigns
Institutional narrative shaping through trusted channels

### ✅ Rituals
Cohesion technology with synchrony, sacredness, identity relevance

### ✅ Collective Memory
Shared history maintained by ritual, storytelling, institutional repetition

### ✅ Echo Chambers & Polarization
Network-level belief structure tracking

### ✅ Noospheric Field
Spreading activation, concept vectors, symbolic nodes

### ✅ Moral Panic Registry
Detects and tracks moral panic episodes

### ✅ Deterministic Replay
Seed → identical simulation, snapshot save/load

### ✅ Causal Provenance
Full decision trace audit trail

### ✅ Spec Linting
Validates RON specs against code, validates 19 data-driven spec files

---

## 7. Architecture Plan 2 Implementation Status

The AP2.md plan is **~99% complete**. All major systems are implemented and integrated into the tick loop, and the final dead-system sweep (Iterations 22–28) brought every flagged unused subsystem alive: SelfModel, Interoception, emotional body tone, narrative frames, sacred values + violation outrage, LegitimacyField, and the desacralization lifecycle.

### §6 Multi-Timescale Scheduler ✅
10 formalized phases with `TickPhases::compute()`.

### §7 Biological Systems (2,557 lines) ✅
All 11 biological modules implemented and wired into `EmbodiedState.tick_update()`.

### §8 Psychological Systems (4,182 lines) ✅
All 16 psychological modules implemented with level-of-detail gating.

### §9 Cognitive Runtime ✅
`CognitiveRuntime` and `DecisionPolicy` integrate all psychology into action.

### §10 Relational Systems (17 files) ✅
`RelationshipV2`, courtship, marriage, kinship, household, clan, patronage, group formation, cult dynamics.

### §11 Power/Status ✅
`StatusDimensions` with dominance/prestige/authority/legitimacy/wealth/honor.

### §12 Group Formation ✅
`PeerGroup`, `GroupRegistry`, `CultDynamics`, `FactionV2`.

### §13 Noospheric/Cultural ✅
Memes, rumors, propaganda, rituals, collective memory, echo chambers, sacred values, education, ideology, noospheric field, belief ecology, legitimacy, moral panic.

### §14 Integration ✅
All systems wired into deterministic tick loop with proper ordering.

### §15 RON Specs (32 files) ✅
32 RON spec files including 19 new data-driven specs with `// Maps to:` comments and spec_lint validation.

### §16 Provenance ✅
13 `ProvenanceCategory` variants, `SystemTrace`, `DecisionTrace`.

### §17 Level-of-Detail ✅
`AgentTier`, `CognitiveBudgetTracker`, tier-gated psychology.

### §18 Testing ✅
613+ tests including 10K-tick stability, golden baseline, property tests.

### Known Remaining Gaps (<1%)

1. **Benchmarks not yet run as CI gate** — `mindstrata-benches` exists with 5 criterion benchmarks (single_tick, 100_tick_burst, 1000_tick_run, metrics_snapshot, agent_summaries), but no automated regression gate wires them into CI.
2. **Some plan modules under different names** — `pain.rs` → integrated in `nervous.rs`; `attention_v2.rs` → root `attention.rs`; etc.

---

## 8. Data-Driven Specifications (RON Files)

32 RON spec files in `specs/`:

| Category | Files | Purpose |
|---|---|---|
| Core | ontology, components, systems, actions, norms, propositions | Game rules |
| Biology | genome, hormones, organs, diseases_v2, life_stages, reproduction | Biological data |
| Psychology | cognitive_systems, emotions_v2, regulation_strategies, identity_frames, moral_foundations | Psychological data |
| Social | courtship, marriage, kinship, status_roles | Social data |
| Culture | rituals, propaganda, taboos_v2, sacred_symbols, education | Cultural data |
| Scenarios | riverford.ron, drought.ron | Simulation scenarios |

All data-driven specs are validated by `spec_lint.rs` for existence and non-emptiness.

---

## 9. Test Coverage & Quality

### 9.1 Test Breakdown

| Test Category | Count | Purpose |
|---|---|---|
| Unit tests (sim) | 554 | Per-system correctness |
| Integration tests | 114 | Full simulation runs (100–10,000 ticks) |
| Core unit tests | 20 | Kernel correctness |
| Property tests | included in above | Proptest: determinism, bounds |
| Golden replay | 1 | Identical output from same seed |
| 10K-tick stability | 1 | Long-running simulation stability |
| **Total** | **688** | |

### 9.2 Quality Metrics

| Metric | Value |
|---|---|
| Tests passing | 688/688 (100%) |
| Clippy warnings | 0 |
| Unsafe code | 0 (`unsafe_code = "forbid"`) |
| TODO/FIXME comments | 0 |
| Determinism | Verified by golden replay |
| 10K-tick stability | Passes (~12s standalone) |
| Commits | 344 |

---

## 10. Technology Stack & Dependencies

### 10.1 Core Stack
| Purpose | Crate |
|---|---|
| Language | Rust (2024 edition) |
| Math | Custom `Fixed` type (i64, 4 decimals) |
| RNG | `rand` + `rand_chacha` (ChaCha8) |
| Serialization | `serde` + `ron` + `postcard` + `serde_json` |
| Logging | `tracing` + `tracing-subscriber` |
| CLI | `clap` (derive) |
| Errors | `thiserror` |
| Graphs | `petgraph` |
| TUI | `ratatui` + `crossterm` |
| Testing | `proptest` + `insta` |

### 10.2 Lint Configuration
```toml
[workspace.lints.clippy]
pedantic = { level = "warn", priority = -1 }

[workspace.lints.rust]
unsafe_code = "forbid"
```

---

## 11. How to Run

```bash
# Basic simulation (12 agents, 500 ticks)
cargo run --release -p mindstrata-cli -- sim --seed 42 --ticks 500

# Drought scenario
cargo run --release -p mindstrata-cli -- scenario specs/scenarios/drought.ron --ticks 2000

# Run all tests
cargo test --workspace

# Run integration tests specifically
cargo test -p mindstrata-tests

# Run 10K-tick stability test
cargo test -p mindstrata-tests ten_thousand_tick_stability

# Run golden baseline test
cargo test -p mindstrata-tests golden_replay_vs_baseline

# Check for warnings
cargo clippy --workspace

# Lint RON specs against code
cargo test -p mindstrata-sim spec_lint
```

---

## 12. Recommended Future Development Priorities

### Phase 1: Performance & Quality
1. **Add criterion benchmarks** for the tick loop to catch performance regressions
2. **Add targeted integration tests** for new biological/psychological/social systems
3. **Snapshot tests** for key outputs using `cargo insta`

### Phase 2: Deepen Existing Systems
4. **Multi-settlement** — neighboring villages with trade routes, diplomacy, conflict
5. **Technology tree** — knowledge prerequisites, innovation chains
6. **Weather system** — temperature, rainfall, droughts, floods
7. **Resource spoilage** — food rots in inventory

### Phase 3: Add Depth
8. **Legal system** — courts, trials, property rights
9. **Education system** — schools, apprenticeships
10. **Religious mechanics** — rituals, theological beliefs, calendar
11. **Military system** — organized warfare, conscription

### Phase 4: Make It Playable
12. **Interactive TUI** — player can select agents, issue commands
13. **Visual rendering** — 2D map with agent sprites
14. **Save/load UI** — snapshot management
15. **Modding API** — formal interface for custom content

---

*This document reflects the codebase state as of August 5, 2026. The simulation engine has been upgraded with Architecture Plan 2 implementations: embodied biology, structured psychology, rich social relationships, cultural systems, and noospheric fields. All systems are integrated into a deterministic tick loop with level-of-detail cognition and causal provenance tracking. Iterations 22–28 completed the crate-wide dead-system sweep, and Iteration 29 fixed the scenario system (declared tick horizons honored; drought drains water proportionally so scenario magnitudes genuinely differentiate).*
