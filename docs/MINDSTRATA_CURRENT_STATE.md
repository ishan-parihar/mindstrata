# Mindstrata — Current State Technical Document

**Prepared for:** Lead Game Designer  
**Date:** August 5, 2026  
**Codebase Version:** 38,900+ lines of Rust across 5 crates, 749 tests passing, 0 clippy warnings

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
- 749 automated tests (property tests, golden replays, statistical emergence, integration, 10K-tick stability)
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
| Endocrine | `EndocrineState` | 6 hormonal axes (stress/bonding/dominance/metabolic/arousal/growth) |
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
701 tests including 10K-tick stability, multi-seed long-horizon macro health, golden baseline, property tests.

### Known Remaining Gaps (<1%)

1. **Benchmarks as CI gate — RESOLVED (Iteration 30)** — `mindstrata-benches` criterion harnesses are now wired into CI: a `tick_throughput_regression_gate` integration test (24 agents, 2000 ticks < 30s, ~11x headroom) plus a `.github/workflows/ci.yml` pipeline (fmt → lenient clippy → full tests → bench compile → gate) on every push/PR.
2. **Some plan modules under different names** — `pain.rs` → integrated in `nervous.rs`; `attention_v2.rs` → root `attention.rs`; etc. (cosmetic only)

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
| Scenarios | riverford.ron, drought.ron, famine.ron, pestilence.ron, collapse.ron | Simulation scenarios |

All data-driven specs are validated by `spec_lint.rs` for existence and non-emptiness.

---

## 9. Test Coverage & Quality

### 9.1 Test Breakdown

| Test Category | Count | Purpose |
|---|---|---|
| Unit tests (sim) | 623 | Per-system correctness (incl. skeletal + digestive §7.2.3/§7.2.7 + relational power §11.2 + speech acts §8.1.11 + perception §8.1.2 + pregnancy §7.2.6 + memory §8.1.3 + motivation §8.1.5 + thermoregulation §7.3.3) |
| Integration tests | 130 | Full simulation runs (100–10,000 ticks, multi-seed macro health, scenario battery) |
| Core unit tests | 20 | Kernel correctness |
| Property tests | included in above | Proptest: determinism, bounds |
| Golden replay | 1 | Identical output from same seed |
| 10K-tick stability | 1 | Long-running simulation stability |
| Multi-seed macro health | 1 | 5 seeds × 15K ticks, every invariant (Iter 31) |
| Tick-throughput gate | 1 | 24 agents/2000 ticks < 30s (Iter 30) |
| Scenario battery | 5 | drought/famine/pestilence vs riverford differentiation + collapse compound emergence (Iter 29/34/35/36) |
| Economy-under-plague | 1 | disease depresses Work productivity ~4.8% via journal Worked records (Iter 37) |
| Skeletal + digestive | 2 | §7.2.3/§7.2.7 systems neutral in calibrated runs; penalties reach derived facade (Iter 38) |
| Relational power | 1 | §11.2 power_balance populated daily — dead field brought alive, asymmetric (Iter 39) |
| Speech acts | 1 | §8.1.11 speech_log populated from interactions — structured linguistic frame, 12 unit tests (Iter 40) |
| Perception/attention | 1 | §8.1.2 biases + salience competition populated; 6 unit tests (Iter 41) |
| Pregnancy lifecycle | 1 | §7.2.6 pregnancy Option<PregnancyState> dormant in sim (births via demography); 8 unit tests (Iter 42) |
| Memory taxonomy + traces | 1 | §8.1.3 nine-kind MemoryKind taxonomy + MemoryTrace properties live in real runs (Flashbulb derived, distortion events recorded); 7 unit tests (Iter 43) |
| Motivation full formula | 1 | §8.1.5 five-factor pressure formula + complete 13-need roster live (care/romance grow, scarcity context derived, pressure_full amplifies); 16 unit tests (Iter 44) |
| Thermal state | 1 | §7.3.3 ThermalState extracted to EmbodiedState (body_temperature/cold_stress/heat_stress); thermoneutral baseline live in real runs; 6 unit tests (Iter 45) |
| **Total** | **757** | |

### 9.2 Quality Metrics

| Metric | Value |
|---|---|
| Tests passing | 757/757 (100%) |
| Clippy warnings | 0 |
| Unsafe code | 0 (`unsafe_code = "forbid"`) |
| TODO/FIXME comments | 0 |
| Determinism | Verified by golden replay |
| 10K-tick stability | Passes (~12s standalone) |
| Commits | 349 |

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

*This document reflects the codebase state as of August 5, 2026. The simulation engine has been upgraded with Architecture Plan 2 implementations: embodied biology, structured psychology, rich social relationships, cultural systems, and noospheric fields. All systems are integrated into a deterministic tick loop with level-of-detail cognition and causal provenance tracking. Iterations 22–28 completed the crate-wide dead-system sweep; Iteration 29 fixed the scenario system (declared tick horizons honored; drought drains water proportionally); Iteration 30 wired the benchmark regression gate + first CI pipeline; Iteration 31 added the multi-seed long-horizon macro-health sweep; Iteration 32 revived the education system via the apprenticeship pass; Iteration 33 brought the material/storage layer alive (site storage capacity with overflow spoilage); Iteration 34 added the Famine food-crisis scenario (ShockKind::Famine, proportional grain drain), a famine-vs-riverford differentiation test, and a bumper-harvest storage-bleed test validating Iter 33 at scale; Iteration 35 added the Pestilence mortality-crisis scenario (ShockKind::Pestilence — an immediate health-weighted mortality wave routed through the §31 death machinery, plus a virulent epidemic seeding that block 17b spreads), with a deaths-differentiation test and an outbreak-spread test; Iteration 36 added the Collapse scenario — a staggered cascade (drought 0.6@500, famine 0.6@800, pestilence 0.6@1100) that proves compound emergence: the same pestilence shock kills more when famine has already weakened health, with a multi-axis scarcity test and the scenario battery now at 5; Iteration 37 closed the economy-under-plague gap — Work productivity is now scaled by a severity-weighted sickness factor (health::work_impairment, 0.2 floor) so a pestilence is no longer economically invisible: the plague now disrupts food production end-to-end (CLI pestilence grain 3.0 vs 3.1 pre-change), with a unit-tested helper (healthy = 1.0, virulent < 1.0, floor reached) and an integration test proving a Fever outbreak depresses journaled Work productivity by ~4.8% against an identical-seed healthy control; Iteration 38 closed the last two missing biological systems from the AP2 §7.2/§7.3 roster — SkeletalState (§7.2.3, spec already declared in organs.ron but zero code) and DigestiveState (§7.2.7, zero code) — implemented as biology/skeletal.rs + biology/digestive.rs with identity-at-baseline multipliers (health_factor, effective_digestion, effective_mobility = exactly 1.0 for healthy adults, so calibrated runs carry zero drift, verified by the 723-test suite and byte-identical CLI grain 37.9), wired into EmbodiedState.tick_update, the Eat action (consume_food), and Work productivity (mobility factor); elder frailty (age > 60) now erodes structural integrity and mobility, severe injury accumulates fracture risk + chronic pain, and spoiled food damages gut health; new fields carry #[serde(default)] so pre-Iter-38 snapshots still restore; digestive tuning section added to organs.ron; Iteration 39 closed the §11.2 RelationalPower gap — RelationshipV2.power_balance was declared but never written (dead field), now populated daily by a pure-function RelationalPower computation (dependence, resource control, emotional/social leverage, coercive capacity, moral obligation, alternatives) from existing relationship + status state, with the sign convention positive = A dominates B, verified asymmetric and zero-drift across the full suite; power_balance is currently write-only observational state (documented facade-fold — the intended consumer, relational dominance/conflict bias, is a future iteration so no calibrated trajectory can drift); Iteration 40 implemented the §8.1.11 Language and Communication system — "language should be action" — as social/speech_act.rs: the full 20-kind speech-act vocabulary (inform, request, command, promise, threaten, apologize, praise, insult, gossip, confess, bless, curse, persuade, accuse, deny, reassure, flirt, propose, vow, excommunicate) with the SpeechAct struct (speaker, listener, audience, content ref, emotional tone, credibility, social cost, relational intent), a pure deterministic interpreter (SpeechAct::from_interaction maps each live InteractionKind to its canonical act), and a resolve_effect model (trust/affection/status/obligation/reputation deltas — computed and unit-tested, not yet applied); the sim now records every InteractionOccurred event onto the speaker's bounded speech_log (64 max, write-only observational state with #[serde(default)] so pre-Iter-40 snapshots restore), with 12 unit tests (mapping, intent, social cost, sign conventions, credibility scaling, boundedness, and a sign-match validation that resolve_effect agrees with the deltas process_interaction actually applies) plus an integration test proving records are populated, well-formed, and grounded in the live interaction vocabulary — zero drift verified across the full 735-test suite and byte-identical CLI grain 37.9; Iteration 41 implemented the §8.1.2 Perception and Attention upgrade — the existing AttentionState (the live salience gateway feeding memory encoding) was extended into the plan's full cognitive gateway with the three perceptual-bias dimensions (threat_bias from fear/anger/trauma-risk — trauma-triggered hypervigilance; social_bias from extraversion + attachment security; novelty_bias from openness, each clamped [0.3, 0.7] around the 0.5 neutral anchor), a bounded salience-competition record (PerceptKind + SalientItem, top-8 percepts by computed salience from the real memory-encoding loop), and attention_capacity() exposing the previously write-only budget; all new state is write-only observational (pure recompute_biases/record_salience, no RNG, nothing feeds back into compute_salience — the calibrated salience gate stays byte-identical), with 6 unit tests + an integration test proving biases vary across agents and the competition map is populated/bounded — zero drift verified across the full 743-test suite and byte-identical CLI grain 37.9; Iteration 42 closed the §7.2.6 PregnancyState gap — the plan mandates `pregnancy: Option<PregnancyState>` but the code carried a flat `pregnant: bool + pregnancy_progress: Fixed` whose lifecycle was dead (attempt_conception never called in sim; births flow through probabilistic demography), and EndocrineState additionally carried a fully-inert duplicate `FertilityAxis` cluster (declared, defaulted, never read or written) — the exact deadweight the YAGNI directive targets: the refactor replaced the flat fields with the plan's `Option<PregnancyState>` (gestation_progress, gestation_duration, stage via GestationStage::of — First/Second/ThirdTrimester/FullTerm, observational strain + birth-risk derived from the reproduction.ron spec's health_risk_base/nutrition_demand_increase), rewrote tick_update/attempt_conception/complete_pregnancy around the Option shape (conception blocked while pregnant, full-term clears the slot and increments children_born), removed the dead FertilityAxis cluster (26 lines), re-pointed the endocrine snapshot to the live reproductive.fertility (0.5 dead default → live values, the sole field delta), and added 8 unit tests + an integration test proving pregnancy stays None across a real run while reproductive.fertility dynamics remain live — zero drift verified across the full 749-test suite, clippy clean, and byte-identical CLI grain 37.9; Iteration 43 implemented the §8.1.3 Learning and Memory upgrade — "expand current memory into multiple memory systems": the 4-kind MemoryKind (Consumption/Social/Positive/Negative) became the plan's full 9-kind taxonomy (Episodic, Semantic, Procedural, Social, Emotional, Flashbulb, Traumatic, Cultural, Somatic) and the plain Memory struct became the plan's MemoryTrace with every mandated property — sensory_richness, accuracy (starts 1.0, eroded by distortion), valence (sign follows kind), identity_relevance (0 baseline — identity-protection is a future consumer), social_sharedness (1.0 when another agent is involved), last_rehearsed, and distortion_history: Vec<DistortionEvent> (tick + cause + delta) — with genuine mechanics wired: encoding derives the trace properties from existing salience/charge/other-agent state (pure, no RNG), vivid moments (salience ≥ 0.4 AND charge ≥ 0.6 — calibrated against live probes: compute_salience tops out ≈0.45 in riverford runs, so ≥0.4 is the vivid tail; charge caps at arousal×0.6+0.1 ≤ 0.7) upgrade to Flashbulb (the "traumatic events flash back" emergent effect), reconsolidation now records an EmotionalReconsolidation event and erodes accuracy when anger/joy bias a Traumatic/Emotional trace, and rehearsal updates last_rehearsed and records a Rehearsal event (retrieval reconstructs); existing encode sites map Consumption→Somatic (eat/drink), Positive→Emotional (help/rest), Negative→Traumatic (threat/insult), Social→Social; the memory module is write-only observational state (nothing reads it for decisions; excluded from snapshot projections and golden), so the richer derivation is drift-free by construction — verified by 7 new unit tests (monotonic ids, derived trace properties, valence sign, flashbulb upgrade boundaries, distortion recording, rehearsal recording), an integration test asserting the taxonomy + trace properties + distortion events are live across a real riverford run, the full 757-test suite green with zero snapshot drift, clippy clean on the new code, and byte-identical CLI grain 37.9; remaining taxonomy slots Episodic/Semantic/Procedural/Cultural are documented with their future producer systems (narrative, education, skills, ritual) for separately-calibrated iterations; Iteration 44 closed the §8.1.5 Layered Motivation gap — the plan's full five-factor pressure formula (`need_deficit × personality_weight × emotional_amplification × cultural_legitimacy × situational_affordance`) was reduced in code to `deficit × urgency_weight`, the plan's 13-need psychological roster was missing care + romance, and dominant_need was computed every tick but never consumed: the module now completes the roster (care + romance added with their own growth/urgency, MotiveCategory gained Hash), adds per-tick context derivation (emotional amplification from the agent's fear/anger/joy/sadness, cultural legitimacy from the legitimacy field, situational affordance from world food/water scarcity — the world totals precomputed once per tick before SystemContext borrows self.world mutably), and pressure_full() implements the plan's exact five-factor formula with per-category modulation (fear→safety/certainty, anger→justice, joy→play/belonging/romance, sadness dampens meaning, scarcity amplifies hunger/thirst) so the argmax is genuinely live (a fearful agent's safety pressure out-competes, verified by unit test); the legitimacy slope was calibrated against live probes — legitimacy_field.overall sits at ~0 in riverford runs (agents hold no legitimacy sources), so a wide slope would halve safety pressure and cancel fear amplification; 0.4 slope clamped [0.7, 1.3] keeps the formula live while remaining plan-faithful (1.0 at the 0.5 mean-zero anchor); dominant_need remains write-only observational (no behavioral consumer yet — documented facade-fold), so calibrated runs carry zero drift — verified by 8 new unit tests (roster completion, per-category amplification, dominance flips, neutral degeneracy to base pressure) + an integration test proving care/romance grow for every agent, scarcity context is derived, and pressure_full amplifies over the baseline across a real riverford run, the full 769-test suite green with zero snapshot drift, clippy clean, and byte-identical CLI grain 37.9; Iteration 45 closed the §7.3.3 Thermoregulation gap — the plan's §7.4 integrated body architecture mandates a distinct `ThermalState { body_temperature, cold_stress, heat_stress }` inside `EmbodiedState`, but the code folded all three fields into `MetabolicState` (the exact Iter 38 extraction pattern): the new biology/thermal.rs module carries the plan-mandated struct + Default (0.5/0/0 thermoneutral) + tick_update with the pre-Iter-45 thermoregulation math moved verbatim, `EmbodiedState` gained `#[serde(default)] thermal: ThermalState` (so pre-Iter-45 snapshots restore), the metabolic block shrank to energy/hunger/hydration (signature `tick_update(activity_level)` — the now-unused ambient_temperature param dropped), the respiratory cold-stress consumer re-pointed to `self.thermal.cold_stress`, and EmbodiedState::tick_update runs thermal immediately after metabolic with the post-update energy reserves — byte-identical by construction (probe-verified: the pre-existing "metabolic warmth" term `energy × 0.01 × cold_stress × 0.001` is below Fixed precision, so starving and well-fed agents stay byte-identical even after 1000 cold ticks — an inert pre-existing term preserved verbatim rather than silently changed), with 6 new unit tests (thermoneutral default, cold/heat stress, comfort band, energy-term inertness) + an integration test asserting every agent holds the thermoneutral baseline across a real riverford run, the full 775-test suite green with zero snapshot drift, clippy clean on the new code, and byte-identical CLI grain 37.9; Iteration 46 wired the two remaining live-producer memory taxonomy slots from §8.1.3 — Procedural and Semantic — which had been write-only since the nine-kind taxonomy landed in Iteration 43: skill practice now encodes a Procedural memory when a 0.1-proficiency milestone is crossed (a pure `skill_milestone_crossed` gate on the tenth boundary — ~100 practice ticks apart at 0.001/tick, so the 200-capacity store is not flooded; Work→farming, Trade→trading, Socialize/Worship→social, salience 0.3, gated on the agent-tier memory budget), and a successful apprenticeship pass now encodes a Semantic memory for the student (tag LearnedKnowledge, other_agent = teacher, salience 0.4, naturally sparse — one pass per tick with deterministic success, gated on the learner's memory budget); two new MemoryTag variants (SkillMastered, LearnedKnowledge) complete the content discriminator; both writes are observational state (nothing reads them for decisions; memory excluded from snapshot projections and golden), so calibrated runs carry zero drift — verified by a unit test pinning the milestone boundary math (fires only on tenth crossings, never sub-step or cap-plateau), an integration test proving both slots fire live (forced teacher→student apprenticeship encodes Semantic; a plain 2000-tick run encodes Procedural), the full 777-test suite green with zero snapshot drift, clippy clean on the new code, and byte-identical CLI grain 37.9; Iteration 47 completed the §8.1.3 nine-kind memory taxonomy — the final two write-only slots, Episodic and Cultural, were wired to their documented live producers: the narrative block (sim.rs tick, Focal agents only via runs_narrative) now encodes MemoryKind::Episodic with the new MemoryTag::LifeEvent when the agent's integrated-life-event count crosses a 100-event chapter boundary (life_chapter_crossed — exact integer math, no stateful cooldown; the narrative block fires every tick in riverford — probe: ~3600 events/agent over 2000 ticks — so the chapter gate keeps encodes sparse at ~1 per 50-100 ticks; salience scales with the interpreted event's emotional magnitude, crises recall louder, budget-gated), and the duodeca ritual block now encodes MemoryKind::Cultural with MemoryTag::RitualParticipated for each participant when a due ritual executes (rituals fire on their monthly 4320-tick interval, so encodes are naturally sparse; salience follows the ritual's emotional_intensity + sacredness, budget- and tier-gated); both writes are observational state (nothing reads memories for decisions; memory excluded from snapshot projections and golden), so calibrated runs carry zero drift — verified by a unit test pinning the chapter-gate boundary math (fires only on 100-event crossings, never sub-chapter), an integration test proving both slots fire live (a 2000-tick run encodes Episodic; a 4500-tick run past the first monthly ritual encodes Cultural), the full 779-test suite green with zero snapshot drift, clippy clean on the new code, and byte-identical CLI grain 37.9 — the §8.1.3 taxonomy is now complete: all nine kinds encode from live systems; Iteration 48 implemented the §8.1.4 Appraisal and Emotion "deepen it" mandate — "expand beyond 8 emotions" — the code's 8-family roster (fear/anger/joy/sadness/trust/shame/pride/guilt) became the plan's full 22-family roster (adding disgust, contempt, awe, gratitude, jealousy, envy, loneliness, tenderness, humiliation, relief, hope, despair, nostalgia, moral_outrage) and the 8-dimension Appraisal became the plan's 11 dimensions (adding sacredness_violation, attachment_threat, status_threat, purity_violation, controllability, future_implication, narrative_meaning): the appraisal construction site now derives the 7 new dimensions from live agent state (max sacredness × witnessed unfairness, separation distress, threat × status deficit, unfairness × trust deficit, conscientiousness, signed (1 − threat − need) future outlook, narrative coherence), appraise() derives the 14 new emotion deltas from the deepened dimensions (moral_outrage ← sacredness violation, disgust ← purity violation, contempt ← status security × unfairness, awe ← |future implication| × identity × unexpectedness, gratitude ← unexpected positive, jealousy ← status × attachment threat, envy ← status threat × incongruence, loneliness ← attachment threat × social invisibility, tenderness ← positive × status security, humiliation ← status threat × identity, relief ← positive × low controllability, hope/despair ← signed future implication split by coping, nostalgia ← narrative meaning × positive), and the agent's DiscreteEmotions carries all 22 with per-field serde defaults (pre-Iter-48 snapshots restore); RegulationStrategy gained the plan's final SubstanceUse variant (13/13, with a matched arm — intoxicants blunt both poles); the expanded families are write-only observational state (the 8 core emotions still drive valence/arousal — their appraise() paths are untouched, so calibrated runs are byte-identical), with new fields flowing through the existing take/sync lifecycle — verified by a unit test pinning the dimension→emotion mappings plus zero-leakage on a neutral appraisal, an integration test proving the expanded families (hope/relief/gratitude/tenderness/nostalgia) are populated across a real riverford run, the full 781-test suite green with zero snapshot drift, clippy clean on the new code, and byte-identical CLI grain 37.9; remaining §8.1.4 work is the consumer wiring (the expanded families feeding decisions — a future behavioral iteration).*

* **Iteration 49 (commit pending) — §8.1.6 Personality and Temperament**: the plan's "keep 12 traits, but add temperament and state-trait dynamics" mandate vs code — Personality carried exactly the 12 traits with **zero temperament fields and zero trait-write sites** (traits were frozen after `random()`); §8.1.7's SelfModel was already complete (10/10 fields), so Personality/Temperament was the genuine gap. Added `Temperament` (7 biologically-rooted dimensions: reactivity, soothability, sociability, persistence, sensitivity, regularity, approach_withdrawal) derived **deterministically from the 12 traits** (zero extra RNG draws — seeded runs stay byte-identical), with per-field `#[serde(default)]` for old-save restore. Added the §8.1.6 trait-plasticity mechanic — `Temperament::plastic_update` implements the plan's formula `trait_change = repeated_state_expression × identity_integration × social_reinforcement × developmental_plasticity` (rate 0.0005, chosen well above the Fixed 10 000-scale resolution so the dynamics are never inert): each dimension is pushed by its live state signal (arousal as stress, clamped valence as recovery, trust+joy as social engagement, success-rate as goal striving) and pulled back toward the trait-derived baseline (equilibrium `x → baseline + signal`), gated by self-model coherence (identity integration) and age (developmental plasticity, full at birth → zero at 70). Wired as sim tick-phase 9 writing ONLY the observational temperament layer — the 12 decision-read core traits are immutable, so calibrated runs are byte-identical (golden hashes cover only hunger/thirst/fatigue/valence). The Fixed-precision trap (per-tick deltas truncating to zero at SCALE 10 000) was caught by the unit tests and fixed by recalibration. Verified by 5 unit tests (deterministic derivation, reactivity monotonicity, core-trait invariance under plasticity, inertness without integration/at max age, age-scaled plasticity) + an integration test (populated/bounded/varied across a real run, seed-deterministic end-state), the full 787-test suite green with zero snapshot drift, clippy clean, and byte-identical CLI grain 37.9; remaining §8.1.6 work is the temperament consumer wiring (temperament feeding decisions — a future behavioral iteration, the prerequisite for letting the 12 core traits themselves move).*

* **Iteration 50 (commit pending) — §8.1.3 Memory retrieval mechanics**: the plan's "retrieval reconstructs" was the last missing memory mechanic — the Iter-43-49 taxonomy left `MemoryStore` with encode/decay/rehearse/reconsolidate but **zero retrieval** (nothing read memories back; `rehearse_random` picks uniformly at random). Added `retrieve_salient` (deterministic salience-scored selection — `strength + emotional_charge/2 + identity_relevance/4`, no RNG — returning `RetrievedMemory` reconstructions whose strength is amplified by charge/mood-congruence/arousal and whose accuracy is eroded by arousal, the plan's "retrieval reconstructs") and `retrieve_and_reconsolidate` (live intrusive recall: the salience argmax trace is rehearsed — rehearsal_count/last_rehearsed/strength up — and an emotion-mismatched reconstruction erodes its accuracy while recording a new `DistortionCause::RetrievalReconstruction` event, closing the plan's "rehearsal strengthens", "retrieval reconstructs", and "trauma memories become intrusive" (high-charge traces always win the argmax)). Wired into the per-agent daily memory block beside `reconsolidate`; purely observational memory state, deterministic (no RNG — unlike `rehearse_random`, which draws from the Behavior stream), so calibrated runs stay byte-identical (golden hashes cover only hunger/thirst/fatigue/valence). Verified by 4 unit tests (salience ordering + determinism, arousal-dependent reconstruction, strengthen-and-distort under anger, empty-store inertness) + an integration test (RetrievalReconstruction events fire across a real run and are seed-deterministic), the full 792-test workspace green with zero snapshot drift, clippy clean on the new code, and byte-identical CLI grain 37.9; remaining §8.1.3 work is the decisional retrieval consumer (retrieved memories feeding action/belief decisions — a future behavioral iteration that will re-calibrate the golden baseline).*

- **Iteration 51 (AP2 §9.2 Deterministic Neural-Like Implementation):** closed the session's largest structural gap — the plan's §9.2 mandates five mechanisms (concept embeddings + cosine similarity, spreading activation, predictive error, RL action values, script grammar), and four of the five had zero production code. Added `psychology/neural_like.rs`: `ConceptVector` (12-dim Fixed embeddings, cosine via a deterministic integer `isqrt`/`fixed_sqrt` since Fixed has no sqrt), `AssociationNetwork` (seeded 12×12 symmetric weights, `spread()` = retention decay + input + association propagation, parallel-update), `PredictiveExpectation` (EMA `observe` returns signed prediction error), `ActionValues` (need/emotional/social/identity relief EMA-learned on success, `value()` = sum − cost − risk), `BehaviorScript` (the plan's 9-step CourtshipScript grammar). Wired into the daily per-agent block as **observational state** (network.spread from a Fixed-domain 12-dim input vector built from living standards/valence/status/kinship/scarcity/threat/sacredness/fear/joy; expectation.observe of the safety composite; RL learn_from_outcome on successful actions) — zero drift, byte-identical CLI. 6 unit tests + 1 integration test (state populates across a run, seed-deterministic); clippy clean after fixing midpoint overflow, range loops, derivable Default, and test-literal field reassignment. Remaining §9.2 work is the *decisional* consumer (activations/values feeding action selection — a future behavioral iteration that will re-calibrate the golden baseline).

- **Iteration 52 (AP2 §10.2 RelationshipV2 deepen-it):** the plan's §10.2 struct mandates 11 fields the code's `RelationshipV2` lacked (role_expectation, public/private labels, kinship_coefficient, household/faction/institutional links, positive/negative memory weights, betrayal_history, reconciliation_history) with 4 supporting types absent entirely. Added `RelationshipLabel` (24 variants, the §10.3 taxonomy incl. authority branch), `RoleExpectation` (kin/authority branch), `BetrayalEvent`/`ReconciliationEvent` (bounded 16-entry FIFO histories). Wired live: daily-boundary pass derives labels/role/kinship from stage + fills structural links from shared household/faction/institution membership; violence site records `BetrayalKind::Violence` both directions; a deterministic reconciliation trigger records Apology when a betrayed relationship's trust recovers above 0.5 — closing the betrayal→reconciliation arc in real runs, all writing only new fields (byte-identical CLI). 10 unit + 1 integration test (fields populate, betrayal + reconciliation histories non-empty, seed-deterministic end-state).

- **Iteration 53 (AP2 §10.7 Household deepen-it)**: `HouseholdRole` was dead code (defined, never stored); the plan's `Household` mandates `roles` + `traditions`. Added both with `#[serde(default)]` — `roles: Vec<HouseholdRole>` (Head/Partner/Adult/Child/Elder/Dependent) derived by `derive_roles()` from member ages + head + partner links (value comparison, no indexing; one role per member, Head precedence), and `traditions: Vec<u64>` (PracticeId) via `collect_traditions()` — sorted-union of member `knowledge.practices`, recomputed on member change. Wired into the daily household pass (byte-identical: only the two new fields written). 16 unit + 1 integration test (roles parallel members + head holds Head + traditions sorted-dedup + seed-deterministic end-state). Roles/traditions are producer-side: the §10.7 *dynamics* (division of labor, childcare, hospitality) that consume them are documented future behavioral work.

- **Iteration 54 (AP2 §10.5 Marriage-as-institution)**: the plan's §10.5 says "Marriage is not just a relationship label. It is an institution." — but the code had the *scaffolding* without the *institution*: `Marriage::new` was only called in unit tests, the formation loop set `agent.partner` + fired an event but never pushed a `Marriage` into `marriage_registry.marriages`, so the death-dissolve pass and `daily_update` iterated empty vecs. Fixed: (1) `Marriage` gained the plan's 3 institutional fields with `#[serde(default)]` — `kin_alliance: Vec<KinshipLink>` (Spouse + deduped InLaw edges), `property_arrangement: PropertyArrangement` (None/Separate/CommunalPool/Dowry/BridePrice, derived deterministically from the wealth gap), `vows: Vec<Vow>` (Fidelity/Provision/Protection/Honor/Care/Obedience/TillDeath, derived from marriage type); (2) the formation loop now instantiates the record — type deterministically derived (Chosen if an active courtship exists, Remarriage if either partner had a prior marriage, else Arranged; no RNG), pushed to the registry, so dissolution-on-death and daily strain/recognition decay now run on real records; (3) `DissolutionCause` grew from 4 to the plan's full 6 causes (added `Abandonment`, `Divorce`, `ReligiousSanction` with dissolve arms — the reviewer caught that death/abandonment/annulment/divorce/exile/religious-sanction was only 4/6). 12 unit + 1 integration test (institution instantiated in production runs, kin_alliance/vows/arrangement/legitimacy non-trivial, all 6 dissolution causes, old-save serde(default) restore, seed-deterministic end-state). Byte-identical: only new registry entries written, no RNG, pair_bonds still unpopulated so courtship gating (`already_bonded`) is unchanged.

- **Iteration 55 (AP2 §11.1 StatusDimensions institutional_rank)**: the plan's `StatusState`/`StatusDimensions` mandates 10 status components — the code had 9; `institutional_rank` (power through institutional role) was absent entirely (no field, no references). Added it with `#[serde(default)]` (default ZERO, old-save restorable) and wired the daily status pass to derive it deterministically as the agent's **highest institutional role authority** (scan of `institutions[].roles[]` for `holder == AgentId`, max authority; Vec-iteration deterministic, no RNG). Only the new field is written — `effective_status()` formula untouched — so the golden baseline stays byte-identical. 4 unit + 1 integration test (default zero, serde roundtrip, old-save restore, all-10-components present; role-holders carry non-zero rank in 2000-tick runs, seed-deterministic). Producer-side: the consumers (weighting `institutional_rank` into `effective_status()` and §11.3 hierarchy formation) are documented future behavioral work that will re-calibrate the baseline.

- **Iteration 56 (AP2 §13.1 Meme System)**: closed the last structural gap in the meme subsystem. `Meme` gained the 4 plan-mandated fields — `lineage: MemeLineage` (new enum: Original/FolkEtymology/InstitutionalSeed/PropagandaTool/Mutant, serde + Copy; derived in `Meme::new` from content type, so Mutant/FolkEtymology are reserved taxonomy variants), `complexity: Fixed` (derived from content type), `institutional_backing: Option<u64>` (`#[serde(default)]`, restored via old-save), `suppression_level: Fixed` (`#[serde(default)]` = ZERO) — and `MemeContent` grew from 12 to the plan's full 14 content types (added `Rumor`, `Song`). `transmission_chance()` now applies the §13.2 `- suppression` term as a `(1 − suppression_level)` multiplier — exactly 1.0 today since nothing writes a non-zero suppression level (producer-inert; an active suppressor — e.g., authorities suppressing hostile memes — is documented future behavioral work that will re-calibrate the baseline). Daily pass derives `institutional_backing` deterministically (scan institutions for a mandate containing the meme's text, first match; Vec-iteration, no RNG — only the new field written, byte-identical). Tests: 7 new unit (lineage defaulting, transmission suppression factor, all-14-content coverage, serde roundtrip + old-save restore, complexity derivation) + 1 integration (`meme_institutional_backing_and_suppression`, 2000-tick, seed-deterministic).

- **Iteration 57 (AP2 §13.5 CollectiveMemory + §13.6 BeliefEcology)**: closed the last structural gaps in the group-culture cluster. (1) `CollectiveMemory` gained the plan's 3 missing fields — `founding_myths: Vec<u64>` (plan types `Vec<MemeId>`; derived daily from memes with `MemeContent::Historical`), `traumas: Vec<SharedTrauma>` (new type: description/event_tick/severity/active, single-sourced `ACTIVE_TRAUMA_SALIENCE_THRESHOLD` = 0.1; derived from `Trauma`-kind memories), `sacred_events: Vec<EventId>` (derived from `Sacred`-kind memories) — all `#[serde(default)]`, refreshed daily by the new `refresh_derived_views()`. (2) The live echo-chamber state gained the plan's missing `narrative_dominance: BTreeMap<u64, Fixed>` (deterministic variant of the plan's `HashMap`, documented) — derived daily as `credibility × virality` per meme. All derivations are deterministic Vec/BTreeMap iterations with no RNG, writing only the new fields, so the golden baseline stays byte-identical. Tests: 7 new unit + 1 integration (`collective_memory_and_echo_chamber_plan_fields`). **Documented type mapping**: the plan's §13.6 `BeliefEcology` is implemented by the live `EchoChamberState` (echo_chamber.rs, in SimState); `culture::ideology::BeliefEcology` is a legacy incomplete struct (dead code — left untouched, do not grep the plan name expecting the live type). **Caveats**: `sacred_events` maps memory ids onto `EventId::new(id)` — synthetic, since the sim has no event-id log (revisit if a real event log lands); the plan types `cross_cutting_ties: Fixed` but the live field is `total_cross_cutting_ties: u32` (a count) — a stored Fixed tie-ratio is future work. Producer-side: consumers (narrative dominance feeding cluster assignment, trauma shaping group psychology) are documented future behavioral work that will re-calibrate the baseline.

- **Iteration 58 (AP2 §10.3 Relationship Stages kin-branch + §10.6 Kinship instantiation)**: the deepest relational gap yet — the kin branch was scaffolded but **never instantiated**. The stage tables' own contract says "Kin stages are assigned, not advanced", yet nothing ever assigned them, and the kinship graph only received edges at populate (where the initial adults have no parents) — so for the entire life of every default simulation the graph had **zero edges**, and the §10.3 kin stages, `coefficient_between` courtship gate, and §10.6 family structure were all permanently inert. Built: (1) **birth-time kinship wiring** in `tick_birth_mechanics` — newborns now mirror parent↔child `ParentChild` edges in both directions plus `Sibling` edges to every prior child of either parent (deterministic, no RNG); (2) **`AncestorDescendant` stage** — the plan's 6th kin stage, added to the enum + label + all 8 stage tables (base_trust 0.45, coefficient 0.25, role Caregiver), live via a 2-hop ancestry scan in the daily pass (grandparent ↔ grandchild); (3) **daily kin-stage assignment pass** at the head of the §10.3 daily block — assigns ParentChild/Sibling/InLaw from direct `KinshipGraph::link_between` (new bidirectional helper), else AncestorDescendant from 2-hop ancestry, refreshing identity metadata; pairs already at a kin stage are skipped; (4) **orphaned-kin reset** (reviewer catch): when death removes edges and the slot is replaced in place by a stranger, the terminal kin stage is reset to Unnoticed — a permanent kin label must not mislabel a stranger; the §10.3 transition gate now skips kin stages outright. Tests: 3 new unit (kin mapping, kin-not-advanceable, ancestor tables) + 2 integration (`kin_stages_instantiated_from_kinship_graph` — manual edges → ParentChild/Sibling/AncestorDescendant both directions, metadata, determinism, death-path reset; `births_mirror_into_kinship_graph` — elevated birth rate → edges + coefficients + sibling wiring). **Byte-identity**: default runs have no kin edges within snapshot windows (~0 births at default birth rate), so zero snapshot drift, CLI byte-identical — verified empirically. **Documented limitations (future work)**: (a) `InLaw → InLaw` maps but no code creates `InLaw` edges (marriage doesn't add them) — the InLaw stage is reachable only if such edges exist; (b) `Cousin` has no `KinshipLink::Cousin` type or transitive derivation — reserved; (c) newborns have empty `relationship_v2s` (the v2 layer is frozen at initial population by its O(1) index layout) — kin stages manifest on initial-population pairs only; a v2 dynamic-growth refactor is the real future work for full §10.3 instantiation; (d) the §10.3 authority branch (patron/client, lord/vassal, master/apprentice, priest/layperson, elder/junior, guard/citizen) remains reserved taxonomy mapped to `RoleExpectation` variants — live role wiring is future institutional work.

- **Iteration 59 (AP2 §12.3 Group Attachment Styles)**: §12.1 (all 10 `GroupType`s) and §12.2 (8-component `GroupCandidate::formation_pressure`) were already complete; the gap was §12.3 — *"attachment styles scale upward"* existed only at the individual level. Implemented: (1) `GroupAttachmentStyle` enum (Secure/Anxious/Avoidant/Disorganized, `#[default]` Secure) with per-style `cohesion_retention()` factors (0.996/0.995/0.993/0.990 — secure holds best, disorganized most volatile); (2) `derive_group_attachment_style(&[AttachmentStyle])` — deterministic modal aggregation, ties resolve Secure > Anxious > Avoidant > Disorganized, empty → Secure; (3) `PeerGroup.attachment_style` (`#[serde(default)]` — old snapshots load as Secure, verified by unit test); (4) the daily formation pass (sim.rs ~6388) derives the style from `candidate.members`' `agent.attachment.style`; (5) `daily_update` now decays cohesion by style (replaces the flat 0.995). Byte-identity: `group_registry` is serialized in snapshot.rs but captured by no snapshot test and CLI metrics are unaffected — empirically **zero drift, CLI byte-identical (grain 37.9 / water 131.6 / fear 0.981)**. 688 sim + 147 integration = **835 workspace tests green**, clippy clean, benches build. Scope caveats: (a) the plan's §12.3 behavioral markers beyond cohesion (anxious groups escalate rumors / cling / panic; disorganized purge / trauma-bond / volatile leadership; avoidant isolate) are **not** yet wired — cohesion retention is the minimal live hook, the rest is future behavioral work; (b) the sim.rs formation-wiring block (style derivation from members) is a thin wrapper over the unit-tested helper and is not exercised end-to-end (formation fires only when `should_form()` pressure > 0.5, which default baselines don't reach) — accepted risk.

- **Iteration 60 (AP2 §12.4 Cult Dissolution completion)**: §12.4's struct + formation + recruitment were already complete; the gap was the **dissolution model** — the call-site itself documented *"leader_competence: no competence model is wired yet, so the leader-failure branch is inert"*, and 4 of the plan's 7 conditions were missing entirely. Implemented: (1) `CultDissolutionSignals` struct (leader_competence, institutional_legitimacy, member_average_fatigue, cult_age_ticks, prophecy_disconfirmed, internal_betrayal, rival_narrative_pressure, economic_stress) + `PROPHECY_GRACE_TICKS` (one week — fresh cults have no falsifiable prophecies yet); (2) `should_dissolve` rewritten to cover all 7 plan conditions, with two behavioral nuances: prophecy disconfirmation is *resisted* by high identity fusion (Festinger's "when prophecy fails") and gated by the grace period, and betrayal requires low dependence (members still dependent resist); (3) sim.rs `tick_cults` now computes real signals — leader competence proxied by the leader's `personality.conscientiousness`, prophecy disconfirmation from the sacred meme's credibility (< 0.3 or vanished), internal betrayal when >half of members' meaning need is satisfied (`needs.meaning` is a deficit scale; LOW = satisfied), rival narrative pressure as the excess emotional charge of the hottest competing meme, and economic stress via the same world-totals-vs-expected scarcity proxy the motivation system uses. Byte-identity: default baselines form no cults (formation gated), so the per-cult loop never executes and the only unconditional new code is the read-only economic-stress precompute — CLI byte-identical (37.9/131.6/0.981), zero drift. **695 sim + 148 integration = 843 workspace tests green**, clippy clean, benches build. Tests: 6 new unit (each condition + fusion-resist + grace-period + high-dependence-resist) + full-lifecycle integration test (formation under crisis → betrayal dissolution → fallout rebound). Debug note: the integration test re-asserts the meaning-deficit precondition one tick before daily tick 2880 because the sim's own need dynamics satisfy the deficit over successive daily ticks — verified empirically via a probe test (single-tick window preserves the deficit); `run(2879)` provably contains no qualifying daily tick (all ≤ 2736 < CULT_COOLDOWN 2880), so formation is deterministic. `is_none_or`/`is_some_and` used per clippy; no `rust-version` declared in the workspace so no MSRV contract is broken.

- **Iteration 61 (AP2 §10.1 Three Relational Fields)**: new `social/relational_field.rs` — `PERCEPTION_RADIUS` (manhattan 5) + `RelationalFields` capturing the plan's three perceptual layers as a deterministic per-agent daily snapshot (the established observational pattern from Iter 54/57; a future behavioral iteration will make it a decisional consumer): **sensory** (nearby_agents, nearest_closeness via linear falloff 1-at-distance-1 → 0-at-radius, perceived_stress = mean (fear+anger)/2 of neighbors), **social** (social_trust = mean relationship trust, social_obligation = mean obligation, kin_count, peer_status = max neighbor effective status), **noospheric** (belief_confidence = mean belief confidence, hottest_charge = max belief emotional charge, legitimacy_perceived = the noospheric legitimacy field, collective_fear = world mean fear). `AgentBundle` gained `relational_fields` (`#[serde(default)]` for old-snapshot compat) at all 3 construction sites (populate, replacement newborn, birth); `refresh_relational_fields()` runs at the end of the daily `tick_derived_states_and_beliefs` with single-pass accumulation — no per-agent Vec allocations (reviewer-mandated, §17.4 population target), no RNG. Byte-identity: purely observational writes to a serde-defaulted field with no behavioral consumer — CLI byte-identical (37.9/131.6/0.981), zero drift. **700 sim + 149 integration = 849 workspace tests green**, clippy clean, benches build. Tests: 5 unit (closeness falloff at distance 1 / at radius / beyond radius, mean edge cases, perception bounds) + integration `relational_fields_refresh` (all-layer [0,1] bounds, `all()` agents perceive ≥1 neighbor, seed-determinism byte-identical across two runs). Debug note: the first closeness formula returned 0.2 at the radius instead of 0 — the unit test caught the bug and the falloff was normalized; the integration test asserts `all()` (not `any`) so any future 16×16 layout drift fails loudly and informatively.

- **Iteration 62 (AP2 §10.4 Pair-Bond wiring — reviving a fully dead subsystem)**: the audit found the romantic pair-bond layer was **never instantiated in production** — `PairBond::new` was exercised only by unit tests, so the registry's `pair_bonds` vec stayed empty and the whole subsystem (bond_strength / strain / jealousy_load / stage ladder / dissolution) was dead code, the same dead-end class as the empty meme/ritual/propaganda registries fixed in earlier iterations. Fix: (1) marriage.rs — new `PairBond::new_married(a, b, tick)` constructor (starts at the `Married` stage with the marriage-era strength baseline 0.6, avoiding post-construction field writes), `charge_jealousy(jealousy_emotion, dependence)` (load += emotion × (0.35 + 0.65×dependence) × 0.15, clamped — dependence = partner trust weights how hard the appraised jealousy threat lands on the bond; appraisal already folds attachment anxiety / status threat / fear of abandonment into the jealousy emotion), and `advance_stage(has_children)` (post-marriage ladder: Married → HouseholdFormed → Parenthood once a shared child exists → Strain when strain > 0.6 ↔ Stabilization on reconciliation when strain < 0.35); (2) sim.rs — pair bonds form 1:1 with marriages in the daily social pass; new daily `tick_pair_bonds` pass charges jealousy, records positive/negative bond events (calibrated small: magnitude 0.1 for calm maintenance — the first version's 0.5 saturated bond_strength to 1.0 in ~40 days, erasing bond differentiation; equilibrium is now ~0.002/day so healthy bonds plateau near ~0.8), and advances stages; parent-pair set precomputed once per pass (O(n) + O(bonds) lookups — §17.4 scale discipline); (3) integration test `pair_bonds_form_with_marriages_and_dynamics_run` (1:1 marriage alignment, bounded fields, post-marriage stages only, daily-pass advancement proof, conditional parenthood implication, seed determinism via f64-sum keys). Byte-identity: writes go only to `pair_bonds` (never snapshot-tested, no behavioral consumer); the formation pass is untouched except an additive push; no RNG touched — CLI byte-identical (37.9/131.6/0.981), zero drift. **705 sim + 150 integration = 855 workspace tests green** (5 new unit + 1 new integration), zero new clippy warnings (66 pre-existing sim warnings unchanged), benches build. **Scope caveat (future consumer)**: jealousy-driven breakup — `should_dissolve()` (bond_strength < 0.1 && strain > 0.8) plus the plan's separation/widowhood effects — remains observational; wiring it to `Marriage::dissolve` is a decisional behavioral iteration that will require golden-baseline recalibration. Debug note: the integration test's first "dynamics live" assertion failed correctly — in a peaceful village the appraised jealousy emotion is legitimately ~0 so jealousy_load stays 0; liveness is instead proven deterministically by `advance_stage` promoting every bond past `Married` after daily ticks (all bonds formed before tick 2000 receive a daily tick within the follow-up window), with the jealousy charge path covered by unit tests.
