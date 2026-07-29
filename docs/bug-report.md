# Mindstrata — Implementation Gap Bug Report

**Date:** July 29, 2026
**Baseline:** architecture.md (v1) vs. `main` branch (commit HEAD)
**Scope:** Every architecture section systematically compared against implemented code

---

## Executive Summary

The architecture document specifies ~40 major subsystems. Of these, **~22 are implemented and functional**, **~8 are partially implemented (stubs or disconnected)**, and **~10 are entirely missing**. The codebase is ~14,000 lines of Rust with 231 passing tests. The core simulation loop works and produces deterministic output. However, several critical emergence targets (factions, trade, migration, movement) are structurally absent from the tick loop despite having module-level code.

---

## Severity Definitions

| Severity | Definition |
|---|---|
| **P0 — Critical** | Architecture explicitly requires this; its absence breaks core simulation guarantees or emergence targets |
| **P1 — High** | Architecture specifies this; module exists but is not wired into the tick loop |
| **P2 — Medium** | Architecture specifies this; partial implementation exists but is incomplete |
| **P3 — Low** | Architecture mentions this as optional/future; not blocking current functionality |

---

## Section-by-Section Audit

### §1. Core Design Philosophy

| Spec | Status | Notes |
|---|---|---|
| Deterministic replay from seed | ✅ Implemented | `RngStreams`, seeded `ChaCha8Rng`, `Clock` all work |
| Headless simulation | ✅ Implemented | CLI runs `sim` and `scenario` commands |
| Emergence from first principles | ⚠️ Partial | Primitives exist, but key emergence (factions, trade, revolt) doesn't trigger |
| Agents not omniscient | ✅ Implemented | Agents only see local state; beliefs can be false |

### §4. Five Substrates

#### §4.1 Psychological Substrate

| Spec | Status | Notes |
|---|---|---|
| Body state (health, energy, hunger, thirst, fatigue) | ✅ Implemented | `BodyState` in `person.rs` |
| Need state (10 needs) | ✅ Implemented | `NeedState` with all 10 deficits |
| Personality (12 traits) | ✅ Implemented | `Personality` with random generation |
| Dimensional affect (valence, arousal, potency) | ✅ Implemented | `Affect` struct |
| Discrete emotions (12) | ✅ Implemented | `DiscreteEmotions` with all 12 |
| Appraisal-driven emotions | ✅ Implemented | `appraisal.rs` with goal relevance, coping, fairness, agency |
| **BUG: Emotion decay too aggressive** | 🔴 P1 | 0.01/tick decay wipes emotions before accumulation; most agents show `Joy: 0.00, Fear: 0.00` in short runs |
| Beliefs with confidence, resistance, identity linkage | ✅ Implemented | `Belief` struct in `person.rs` |
| **BUG: Proposition display shows `prop_{}`** | 🔴 P1 | Belief inspector renders `prop_{}` instead of human-readable proposition names; proposition registry not wired to display |
| Memory with encoding, decay, rehearsal | ✅ Implemented | `MemoryStore` with `MemoryKind`, `MemoryTag` |
| Attention system | ✅ Implemented | `AttentionState` with salience, habituation, budget |
| Cognitive state (bounded rationality) | ✅ Implemented | `CognitiveState` with stress → heuristic bias |
| Derived mental states | ✅ Implemented | `DerivedMentalState` with trauma, depression, ambition |
| Moral values (6 foundations) | ✅ Implemented | `MoralValues` with care, fairness, loyalty, authority, purity, liberty |
| **Memory reconsolidation** | ❌ P2 | Architecture §22.4 specifies "remembering an event can change the memory" — not implemented |
| **Social memory type** | ⚠️ P2 | `MemoryKind::Social` exists but social memory (reputation snapshots) not distinguished from general memory |

#### §4.2 Behavioral Substrate

| Spec | Status | Notes |
|---|---|---|
| Utility-based action selection | ✅ Implemented | `actions::select_action` with 11 action types |
| Action primitives (20+ listed) | ⚠️ P2 | Only 11 actions implemented: Eat, Drink, Rest, Work, Socialize, Worship, Trade, Help, Threaten, Steal, Idle. Missing: Flee, Attack, Migrate, Hoard, Share, Persuade, Organize, Vote |
| **Affordance system** | ❌ P1 | Architecture §24.2 specifies objects/sites expose affordances — not implemented; actions are hardcoded per site kind |
| Daily routines | ✅ Implemented | `DailyRoutine` with village schedule |
| Intention commitment | ✅ Implemented | `Intention` struct with abandonment logic |
| **Hierarchical task network** | ❌ P2 | Architecture §24.2 specifies HTN for longer goals — not implemented; only immediate utility AI |
| **Goal source tracking** | ⚠️ P2 | `Goal` struct has `kind` and `priority` but `source` field missing (architecture specifies `GoalSource`: needs, beliefs, emotions, identity, roles, obligations) |
| **Goal rejection/completion tracking** | ⚠️ P2 | Architecture §22.4 distinguishes desires → goals → intentions with `rejected` and `completed` lists — only `active` goals exist |
| Bounded rationality (perception radius, memory, attention) | ✅ Implemented | Attention system + cognitive state |
| **Noise in utility function** | ⚠️ P3 | Architecture §24.3 specifies `+ noise` term — not visible in `select_action` |

#### §4.3 Relational Substrate

| Spec | Status | Notes |
|---|---|---|
| Relationships as first-class entities | ✅ Implemented | `Relationship` struct with trust, affection, respect, fear, obligation |
| Social interactions (7 types) | ✅ Implemented | Help, Trade, Gossip, Threaten, Insult, Comfort, Bond |
| **Missing interaction types** | ⚠️ P2 | Architecture lists: Accuse, Follow, Betray, Comfort — Comfort exists but Accuse, Follow, Betray missing |
| **Social graphs (multiple)** | ❌ P2 | Architecture §11 specifies multiple graphs (kinship, friendship, economic, political, religious) — only one flat relationship list exists |
| **Status competition** | ❌ P2 | Architecture §11.1 lists status competition as essential social process — not implemented |
| **Coalition formation** | ❌ P2 | Not implemented (related to factions but distinct) |
| **Forgiveness/grudges** | ❌ P3 | Not implemented as explicit mechanisms |

#### §4.4 Systemic Substrate

| Spec | Status | Notes |
|---|---|---|
| Institutions as entities | ✅ Implemented | `Institution` with legitimacy, cohesion, roles, treasury |
| 3 default institutions (Council, Temple, Market) | ✅ Implemented | Created in `default_institutions()` |
| **BUG: Faction formation never triggers** | 🔴 P0 | `factions.rs` has full implementation (grievance computation, leader selection, faction creation) but **no code in `sim.rs` tick loop calls `create_faction` or checks formation conditions**. Factions always show `Factions: 0` |
| Norms with enforcement | ✅ Implemented | `NormRegistry` with pressure computation, violation recording |
| Tax collection | ✅ Implemented | `Institution::collect_taxes` |
| Wage payment | ✅ Implemented | `Institution::pay_wages` |
| Policy issuance | ✅ Implemented | `Institution::propose_policy` with delay |
| Institutional records | ✅ Implemented | `InstitutionalRecord` with tick, action, affected agents |
| **BUG: `derive_collective_psychology` never called in tick loop** | 🔴 P1 | The method exists on `Institution` but `sim.rs` never calls it. Collective psychology stays at default values |
| **Corruption dynamics** | ❌ P2 | `corruption` field exists but no system increases/decreases it based on behavior |
| **Institutional decision procedures** | ⚠️ P2 | `propose_policy` exists but no decision procedure (voting, decree) triggers it |
| **Office/term mechanics** | ❌ P3 | Architecture §12.3 specifies offices with term lengths — roles exist but no term limits or elections |

#### §4.5 Epistemic Layer

| Spec | Status | Notes |
|---|---|---|
| Propositions with confidence | ✅ Implemented | `Belief` with proposition_id, confidence |
| **BUG: Proposition registry not human-readable** | 🔴 P1 | Propositions are numeric IDs (0, 1) with no name mapping; belief inspector shows `prop_{}` |
| Belief updating (Bayesian-ish) | ✅ Implemented | `belief_update.rs` with resistance, evidence, social reinforcement |
| Gossip with mutation | ✅ Implemented | `gossip.rs` with emotional distortion, identity bias |
| Knowledge diffusion | ✅ Implemented | Cultural knowledge transfers through social interactions |
| **Rumor propagation graph** | ⚠️ P2 | Gossip flows through relationship edges but no dedicated rumor propagation graph |
| **Institutional facts (official records)** | ⚠️ P2 | `InstitutionalRecord` exists but agents don't read/query them to form beliefs |

---

## §6. World Model

| Spec | Status | Notes |
|---|---|---|
| 2D grid world | ✅ Implemented | `World` with `Vec<Tile>` |
| Terrain types (7) | ✅ Implemented | Grassland, Forest, Hill, Mountain, Water, Desert, Swamp |
| Sites (10 kinds) | ✅ Implemented | House, Farm, Well, Market, Temple, Barracks, Workshop, Square, Prison, School |
| **BUG: No agent spatial positioning** | 🔴 P0 | Architecture §6 requires agents to have positions in the world. **Agents have no `(x, y)` coordinates.** `nearest_site_of_kind` ignores the `_from_site` parameter. Agents teleport to sites instantly. |
| **BUG: No agent movement** | 🔴 P0 | No `move` action, no pathfinding, no travel time between sites. Architecture §24.1 lists `move` as an action primitive. |
| Resource stocks per site | ✅ Implemented | `ResourceStock` with quantity, quality, access rights |
| Access rights (3 types) | ✅ Implemented | Public, OwnerOnly, InstitutionMembers |
| **BUG: `InstitutionMembers` access always returns true** | 🔴 P1 | `can_access_resource` for `InstitutionMembers` variant hardcodes `true` with a comment "Currently no sites use this variant" |
| Resource spoilage | ✅ Implemented | Per-tick spoilage with seasonal modifier |
| Regions | ✅ Implemented | `Region` struct, single "Riverford" region |
| **BUG: `resource_stock` field in `Tile` never used** | 🟡 P2 | Architecture §6.1 specifies `resource_stock: Option<ResourceStock>` on tiles — `Tile` has no such field; resources live only on sites |

---

## §7. Simulation Loop

| Spec | Status | Notes |
|---|---|---|
| Fixed timestep | ✅ Implemented | `Clock` with tick advancement |
| Multiple schedules (micro, standard, daily, weekly, monthly, yearly) | ⚠️ P2 | Only one tick rate exists. Architecture specifies daily/weekly/monthly/yearly ticks — all systems run every tick |
| Explicit system ordering | ✅ Implemented | Systems run in documented order in `tick()` |
| **Missing systems from tick loop** | 🔴 P1 | See "Disconnected Systems" below |

---

## §8. Event Architecture

| Spec | Status | Notes |
|---|---|---|
| Event enum with typed variants | ✅ Implemented | `SimEvent` in `event.rs` with 10+ variants |
| Events used for memory, gossip, provenance | ✅ Implemented | Memory encoding from events, gossip from events, provenance traces |
| **Missing event types** | ⚠️ P2 | Architecture lists: `FactionFormed`, `MigrationStarted`, `EmotionalShift` — none implemented as `SimEvent` variants |

---

## §13. Emergence Targets

| Target | Status | Notes |
|---|---|---|
| **Individual: habits, addiction, trauma, ambition** | ⚠️ Partial | Trauma ✅, ambition ✅ (derived), habits via routines ✅, addiction ❌ |
| **Social: friendships, rivalries, families** | ⚠️ Partial | Relationships ✅, families via demography ✅, rivalries ❌, feuds ❌ |
| **Economic: price formation** | ⚠️ Partial | Price tracker ✅, but **trade volume is 0.0** — agents don't actually trade with each other |
| **Economic: inequality** | ✅ Implemented | Gini coefficient computed |
| **Economic: black markets** | ❌ P3 | Not implemented |
| **Political: factions** | 🔴 P0 | Module exists but never triggers |
| **Political: protests** | ❌ P0 | `should_protest` exists but never called in tick loop |
| **Political: corruption** | ❌ P2 | Field exists, no dynamics |
| **Cultural: rumors, fashions, taboos** | ⚠️ Partial | Rumors via gossip ✅, taboos via norms ✅, fashions ❌ |
| **Ecological: overfarming** | ✅ Implemented | Work ticks decay fertility |
| **Ecological: famine** | ⚠️ Partial | Grain depletes but no famine event system |
| **Ecological: migration** | ❌ P0 | `migration_pressure` computed in ecology but **never used** to actually move agents |

---

## Disconnected Systems (P1 — Modules exist but not wired into tick loop)

These are the most critical bugs. The code exists but is never called:

1. **`factions::create_faction`** — Never called. No grievance computation happens per-tick.
2. **`factions::should_protest`** — Never called. No protests ever occur.
3. **`institutions::derive_collective_psychology`** — Never called. Collective psychology stays at defaults.
4. **`ecology::migration_pressure`** — Computed but never acted upon. No agents ever migrate.
5. **`demography::age_agent`** — Exists but **not called in `tick()`**. Agents don't age during simulation.
6. **`demography::should_birth`** — Exists but **not called in `tick()`**. No births during simulation (children appear only at initialization via naming convention `Child_N`).
7. **`market::system_market`** — Exists with price updating but **not called in `tick()`**. Prices never update.
8. **`market::execute_trade` / `direct_trade`** — Exist but **never called from action execution**. Trade volume is always 0.
9. **`health::system_health`** — Called ✅ (this one is wired)
10. **`logistics::carrying_cost`** — Exists but never applied to agents.
11. **`population_cap::MAX_POPULATION`** — Just a constant (`48`). Never checked.

---

## Missing Systems (Entirely absent from codebase)

| Architecture Section | Description | Priority |
|---|---|---|
| §6: Agent spatial positions | Agents have no `(x, y)` coordinates | P0 |
| §6: Agent movement/pathfinding | No movement action or travel time | P0 |
| §10.2: Hierarchical Task Network | Only flat utility AI, no multi-step planning | P2 |
| §11: Multiple social graphs | Only one flat relationship list | P2 |
| §12.3: Legal/normative layer | No property rights, contracts, crimes, adjudication | P2 |
| §15.3: Spatial partitioning | No chunks/regions for spatial queries | P3 |
| §15.4: Social graph sharding | No sharding by household/settlement/faction | P3 |
| §16.1: Snapshot persistence | `Snapshot` struct exists but no file I/O | P2 |
| §16.2: Event journal persistence | Journal in-memory only, no file output | P2 |
| §17: Interactive TUI | CLI-only, no live TUI mode | P2 |
| §22.4: Memory reconsolidation | "Remembering changes the memory" — not implemented | P3 |
| §24.2: Affordance system | Objects don't expose affordances; actions hardcoded | P1 |
| §31: Demographic tick integration | Aging/birth/death modules exist but not in tick loop | P0 |

---

## Summary Statistics

| Category | Count |
|---|---|
| Total architecture sections audited | 40+ |
| Fully implemented | 22 |
| Partially implemented | 8 |
| Entirely missing | 10+ |
| P0 bugs (critical gaps) | 5 |
| P1 bugs (disconnected systems) | 7 |
| P2 bugs (incomplete features) | 12 |
| P3 bugs (future work) | 6 |
| Total tests passing | 231 |
| Lines of Rust code | ~14,000 |

---

## Top 5 Priority Fixes

1. **Wire demography into tick loop** — `age_agent`, `should_birth`, death checks. Without this, the population is static after initialization.
2. **Wire faction formation into tick loop** — Compute per-agent grievance, check formation threshold, call `create_faction`. This is the #1 emergence target.
3. **Add agent spatial positions and movement** — Give agents `(x, y)` coordinates, implement `Move` action with travel time. Without this, the world is just a bag of sites.
4. **Wire market system into tick loop** — Call `system_market` for price updates, wire `execute_trade` into the `Trade` action.
5. **Fix emotion decay rate** — Reduce from 0.01/tick to ~0.002/tick so emotions accumulate and create meaningful dynamics.
