# Mindstrata — Remaining Work Report

**Scope:** refinement, testing, validation, and optimization items still open.
**Baseline:** commit `f90e58a` (Iteration 89), tree clean, **958 workspace tests green** (772 sim + 186 integration), zero snapshot drift.
**Plan status:** AP2 is structurally ~99% complete — every plan-mandated *system* exists and is wired into the deterministic tick loop. The remaining work is overwhelmingly **decisional consumers** (state that is produced but never read back into decisions), **scale/perf hardening**, and **post-AP2 game features**.

---

## 1. Refinement — Decisional Consumers (write-only → behavioral)

The dominant category. A recurring pattern across the campaign: systems were built and wired as **observational state** (pure, no RNG, excluded from golden projections → zero drift), with their *behavioral consumer* documented as "future work". Wiring any of these **will re-calibrate the golden baseline** (each is a separate, carefully-tested iteration — the Iter-66 precedent).

| # | Plan § | Item | Producer (iteration) | Status |
|---|--------|------|----------------------|--------|
| 1 | §7.2.6 | **Conception→pregnancy→birth pipeline** — `attempt_conception` has **zero production callers**; births flow through probabilistic demography only; `Option<PregnancyState>` stays `None` in real runs | Iter 42 | **Largest inert biological channel** |
| 2 | §8.1.4 | Expanded 14 emotion families (disgust, contempt, awe, gratitude, jealousy, envy, loneliness, tenderness, humiliation, relief, hope, despair, nostalgia, moral_outrage) + deepened appraisal dimensions feeding **decisions** | Iter 48 | Observational |
| 3 | §9.2 | Neural-like activations / RL action values / prediction error feeding **action selection** | Iter 51 | Observational |
| 4 | §8.1.16 | Prospection scenarios feeding decisions; D2–D6 domains (threat/injustice/courtship/ambition) proven by unit test only — the scarce default world fires only D1 | Iter 71 | Observational |
| 5 | §8.1.5 | **WIRED (Iter 96)** — `dominant_need` urgency boost in `compute_utility`: the full-pressure argmax biases selection toward the dominant need's relief channel (exclusive boost; Safety/Esteem dominant = zero boost = fight-or-flight override) | Iter 44 | Wired |
| 6 | §8.1.2 | Attention biases feeding back into `compute_salience` (currently "nothing feeds back"; `attention_capacity()` exposed, unread) | Iter 41 | Observational |
| 7 | §8.1.11 | Speech-act `resolve_effect` model (trust/affection/status/obligation/reputation deltas) **applied** to relationships — currently computed + unit-tested but not applied | Iter 40 | Observational |
| 8 | §8.1.3 | Memory `identity_relevance` — identity-protection recall bias (baseline 0) | Iter 43 | Observational |
| 9 | §8.1.6 | **Trait plasticity** — the 12 core traits never move (prerequisite for developmental change) | Iter ~50 | Observational |
| 10 | §10.2 / §11.2 | Three relational fields (sensory/social/noospheric) and `power_balance` + private-label divergence → **relational dominance/conflict bias**; `institutional_rank` not yet weighted into `effective_status()` | Iter 39/81/55 | Observational |
| 11 | §10.4 | Attraction (familiarity/reciprocity/status/kinship-penalty/moral-disgust/social-cost, all live) → **behavioral courtship decisions**; `total_attraction` currently feeds only the observational D4 scenario gate; `taboo_penalty` unwired | Iter 75–79 | Observational |
| 12 | §10.4 | **"Seek proximity" dynamic** — courting pairs sit beyond perception radius; the ladder stalls at the pair's trust ceiling (documented Iter-76 limitation) | Iter 76 | Behavioral gap |
| 13 | §10.7 | Household roles/traditions consumers — division of labor, childcare, hospitality | Iter 53 | Producer-side only |
| 14 | §13.1–13.6 | `narrative_dominance` → cluster assignment; trauma → group psychology; `cross_cutting_ties` stored as u32 count vs plan's Fixed tie-ratio; `sacred_events` uses synthetic `EventId`s (revisit if a real event log lands) | Iter 57 | Producer-side only |
| 15 | §29.2 | Faction-v2 `threat_level`/`mobilization_capacity`/`legitimacy_of_violence` → revolt-vs-crackdown force-comparison consumer (suppression reads only `fighting_strength`) | Iter 74 | Observational |
| 16 | §8.1.10 | **Respect Elders + Obey Ruler norms** — internalized by ritual, zero behavioral consumers; `Institution.norm_ids` (temple's "Obey Ruler" declaration) has **zero consumers anywhere** | Iter 82–89 | Unwired |

**Reserved taxonomy (structural, no behavioral intent yet):** `LordVassal` + `GuardCitizen` stages (no live producer); `KinshipLink::Cousin` type reserved; `InLaw` edges never created by code (stage reachable only if edges exist); legacy `culture::ideology::BeliefEcology` dead struct (deliberately untouched); the §10.4 probabilistic daily ladder roll **superseded by design** (deterministic gates keep replay byte-identical — do not "fix" it).

---

## 2. Testing — Coverage Gaps

| Gap | Detail |
|-----|--------|
| **Behavioral proofs are uneven** | Many systems are proven by unit tests + liveness (state populates) rather than pre/post behavioral deltas. The strongest behavioral proofs were done once each: Iter-83 conflict probe (28,878→28,736 conflicts, 3→4 factions), Iter-89 coup exposure. A systematic behavioral-delta harness (same seed, consumer on/off) is missing. |
| **Population scale untested** | All tests ≤ 48 agents (max seen: 24). No validation at 100s/1000s agents — where the O(n²) daily trust matrix (rumors) and the §17.4 sparse-aggregation strategy become real. |
| **Long-horizon determinism** | Golden replay = 1 test; multi-seed macro health = 5 seeds × 15K (Iter 31). No long-horizon (50K+) determinism/emergence sweep beyond the 30K faction tests. |
| **Scenario battery** | 5 scenarios (riverford/drought/famine/pestilence/collapse). More are cheap to add from the spec files (specs/scenarios has riverford + drought; the ShockKind machinery is extensible). |
| **Statistical emergence** | `statistical_emergence.rs` exists but the documented §18.4 gossip-hops test was vacuous pre-Iter-73; similar audits are due for other §18.4 claims (courtship D4, faction counts, rumor saturation). |
| **Property tests** | Proptest limited to determinism/bounds — no invariant-style property tests (e.g., "trust never exceeds [0,1]", "enforcement_count monotonic"). |
| **Snapshot coverage** | 7 insta snapshots, all ≤ 2000-tick horizons (below the first ritual at 4320). A >4320-tick snapshot would capture the entire norm/memory/attraction surface — would require regeneration, deliberately avoided so far. |

---

## 3. Validation — Documentation & Process Debt

| Item | Detail |
|------|--------|
| **State-doc test tables are stale** | §9.1 claims 757 tests / 623 unit / 130 integration / 0 clippy warnings / 349 commits — actual: **958 tests (772 + 186)**, ~93 clippy warning lines, and the commit count has moved on. The "0 warnings" and "0 TODO" claims no longer hold. |
| **Clippy debt** | **93 warning lines** workspace-wide, dominated by: `field_reassign_with_default` (21), `assert!(==)` (13), `map(..).unwrap_or(..)` (9), `format!` inline vars (6), manual assign ops (6). CI runs **lenient** clippy. A `cargo clippy --all-targets -- -D warnings` sweep (the git history shows such sweeps were done periodically) would restore the gate. 2 of these are in recently-touched regions (sim.rs:8445 clone, integration_tests.rs:5050). |
| **State doc navigability** | The iteration history is one giant paragraph per iteration — hard to grep. A structured changelog (one bullet per iteration) would materially help. |
| **Golden recalibration queue** | Each decisional consumer in §1 will require a deliberate, documented golden recalibration (Iter-66 precedent: meme mutation). No backlog exists tracking which consumers are slated and in what order. |

---

## 4. Optimization — Performance & Scale

| Item | Detail |
|------|--------|
| **O(n²) daily passes** | Rumor transmission rebuilds the n×n trust matrix daily (fine at 12–48, documented as needing lazy/sparse treatment at scale, §17.4). |
| **Meme aggregation** | Plan's sparse/lazy aggregation strategy (§17.4) not implemented — linear scans are fine at village scale only. |
| **Benches** | Only `tick_loop.rs` criterion bench + the CI throughput gate (24 agents/2000 ticks < 30s). No per-system benches (e.g., memory encode hot path, rumor pass), no release-mode behavioral benchmark, no large-population scaling benchmark. |
| **Cognitive budget at scale** | `AgentTier`/`CognitiveBudgetTracker` exist but are exercised only implicitly; background-tier behavior at 100s of agents is unmeasured. |
| **Level-of-detail** | Focal-only narrative (`runs_narrative`) is the main tier gate; the §17.1 Focal/Secondary/Background calibration is not validated against a population mix. |

---

## 5. Beyond AP2 — New Game-Level Features (state-doc §12 priorities)

**Phase 2 — Deepen:**
- **Multi-settlement** — neighboring villages, trade routes, diplomacy, conflict
- **Technology tree** — knowledge prerequisites, innovation chains
- **Weather system** — temperature, rainfall, droughts, floods (only scripted shocks exist today)
- **Time-based resource spoilage** — Iter 33 handled storage-overflow spoilage; inventory rot over time remains

**Phase 3 — Add depth:**
- **Legal system** — courts, trials, property rights
- **Education system** — schools (apprenticeships are live, schools are not)
- **Religious mechanics** — theological beliefs, calendar (rituals are live)
- **Military system** — organized warfare, conscription

**Phase 4 — Playability:**
- **Interactive TUI** — the TUI is 999 lines; agent selection / issuing commands is the interactive gap
- **Visual rendering** — 2D map with agent sprites (no frontend exists)
- **Save/load UI** — snapshot management
- **Modding API** — formal interface for custom content

**Explicitly optional by plan:** §9.3 external LLM role (LLM-driven focal agents) — unimplemented, by design.

---

## 6. Recommended Prioritization

**Quick wins (low risk, no recalibration):**
1. Clippy `-D warnings` sweep (93 warnings — pure quality debt)
2. Refresh state-doc test tables + restructure the iteration log
3. Add the missing 2 pre-existing-warning fixes (sim.rs:8445, integration_tests.rs:5050)
4. Strengthen property tests (invariant-style) + a behavioral-delta harness (same seed, consumer on/off)

**Behavioral wiring (each recalibrates; highest game-fidelity value):**
5. §7.2.6 conception→pregnancy→birth pipeline (largest inert channel)
6. §8.1.10 Respect Elders / Obey Ruler + `Institution.norm_ids` (temple) — the natural continuation of Iterations 82–89
7. §8.1.4 expanded emotions → decisions; §9.2 neural-like → action selection
8. §10.4 "seek proximity" + attraction → courtship behavior (closes the documented stall)

**Scale & polish:**
9. Large-population validation (100+ agents) + sparse O(n²) passes
10. Multi-settlement + weather (biggest structural additions beyond AP2)

---

*Report generated from `docs/architecture/AP2.md`, `docs/MINDSTRATA_CURRENT_STATE.md`, and live codebase audits at commit `f90e58a`. All test counts and clippy numbers verified against the workspace at report time.*
