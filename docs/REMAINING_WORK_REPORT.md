# Mindstrata — Remaining Work Report

**Scope:** refinement, testing, validation, and optimization items still open.
**Baseline:** commit `f90e58a` (Iteration 89), tree clean, **958 workspace tests green** (772 sim + 186 integration), zero snapshot drift.
**Plan status:** AP2 is structurally ~99% complete — every plan-mandated *system* exists and is wired into the deterministic tick loop. The remaining work is overwhelmingly **decisional consumers** (state that is produced but never read back into decisions), **scale/perf hardening**, and **post-AP2 game features**.

---

## 1. Refinement — Decisional Consumers (write-only → behavioral)

The dominant category. A recurring pattern across the campaign: systems were built and wired as **observational state** (pure, no RNG, excluded from golden projections → zero drift), with their *behavioral consumer* documented as "future work". Wiring any of these **will re-calibrate the golden baseline** (each is a separate, carefully-tested iteration — the Iter-66 precedent).

| # | Plan § | Item | Producer (iteration) | Status |
|---|--------|------|----------------------|--------|
| 1 | §7.2.6 | **WIRED (Iter 92)** — the conception→pregnancy→birth pipeline is live: `should_birth` starts a pregnancy on the female partner, gestation advances per-tick, and the birth pass is keyed off the pregnancy (widow births deliver; same-sex couples keep the legacy immediate-birth path). `Marriage.children` now populated at birth | Iter 42 | Wired |
| 2 | §8.1.4 | **PARTIAL (Iter 99)** — two channels live: loneliness→social-seeking (Iter 98, `interact_chance` + loneliness×0.3) and tenderness→helping (Iter 99, `help_propensity` + tenderness×0.5). Still unwired: contempt, awe, envy, humiliation, relief, despair, nostalgia | Iter 48 | Partially wired |
| 3 | §9.2 | **WIRED (Iter 94)** — `learned_delta` (normalized per-profile dot of learned values vs the 0.5 prior) feeds `compute_utility`; the RL loop is closed (learning shapes action choice; zero at the prior → tick-0-identical) | Iter 51 | Wired |
| 4 | §8.1.16 | **WIRED (Iter 103) for the D1 channel** — scenario-grounded dread feeds `compute_utility` (precautionary provisioning: Work/Trade +0.2·dread, Rest −0.1·dread; zero-at-zero; deterministic). Still unit-test-proven only: D2–D6 domains (threat/injustice/courtship/ambition) — the scarce default world fires only D1 | Iter 71 | Partially wired |
| 5 | §8.1.5 | **WIRED (Iter 96)** — `dominant_need` urgency boost in `compute_utility`: the full-pressure argmax biases selection toward the dominant need's relief channel (exclusive boost; Safety/Esteem dominant = zero boost = fight-or-flight override) | Iter 44 | Wired |
| 6 | §8.1.2 | **WIRED (Iter 97)** — `percept_bias_factor` folds extraversion/openness/fear/anger/trauma into `compute_salience`; memory-encoding salience now reflects the agent's biases (social-memory encoding ~2× extravert-vs-introvert) | Iter 41 | Wired |
| 7 | §8.1.11 | **WIRED (Iter 95)** — `resolve_effect` applied at the live speech-act site: obligation→listener debt, reputation→listener admiration, status→speaker respect (trust/affection channels deliberately skipped — the interaction system already applies byte-identical deltas) | Iter 40 | Wired |
| 8 | §8.1.3 | **WIRED (Iter 104)** — `identity_relevance` derived at encoding (trauma 0.6 / social-emotional 0.3 / flashbulb +0.15 / milestones +0.2 / charge×0.1) and consumed by `retrieval_score` (× 0.25): the identity-protection recall bias is live — identity-salient traces intrude, rehearse, and survive eviction. Decision-level consumers (Episodic→narrative) remain future | Iter 43 | Wired |
| 9 | §8.1.6 | **WIRED (Iter 105)** — the plasticity pass was already running daily (sim.rs:3815, drifting each agent's Temperament layer), but the layer had **zero consumers** — write-only observational state. Now: the production appraise site multiplies fear/anger deltas by `Temperament::reactivity_amplifier(deviation)` where deviation = live reactivity − `from_traits` baseline (zero at construction → byte-identical until the pass drifts it). Differential-proven live (low-coping world, +42% fear at tick 5). Remaining: the 12 core traits themselves still never move (core-trait movement is a future, separately-calibrated iteration); sociability/boldness/diligence have no decision consumer yet | Iter 105 | Wired |
| 10 | §10.2 / §11.2 | **WIRED (Iter 102)** — directed `power_balance` fed into `should_escalate` as a multiplicative dominance scale (0.5× subordinate → 1.5× dominant, identity at zero → legacy byte-identical); the violence-escalation decision now reads the full relational-power stack. **Iteration 106**: `institutional_rank` now weighted into `effective_status()` (× 0.1, zero-at-zero; behavioral via §10.9 patronage — first big-blast iteration, golden + snapshots regenerated, 10 intent-preserving test recalibrations). **Iteration 107**: the sensory field's `perceived_stress` now has a decisional consumer — the daily fear-contagion fold (`FEAR_CONTAGION_RATE` 0.05, shared `contagion_apply` helper, zero-at-zero, deterministic). **Iteration 108**: the social field's `kin_count` now has a decisional consumer — the kin-support stress buffer (`KIN_STRESS_RATE` 0.15/kin, `KIN_STRESS_CAP` 0.45, `kin_stress_factor` scaling the §22.1 cognitive stress input; zero-at-zero → golden byte-identical, NO regeneration needed). **Iteration 109**: the noospheric field's `belief_confidence` now has a decisional consumer — the dogmatism channel (`BELIEF_CONFIDENCE_RIGIDITY` 0.5, `belief_rigidity_factor = 1 + confidence × rigidity` dampening the evidence term in `update_belief(s)` via `× (2 − rigidity_factor)`; evidence-dampening pivot: scaling resistance up amplifies positive updates; zero-blast — belief state decision-independent at calibrated horizons → golden byte-identical, NO regeneration needed). **Iteration 110**: the social field's `social_trust` now has a decisional consumer — the violence-pacification channel (`SOCIAL_TRUST_PACIFY_RATE` 0.3/unit, `SOCIAL_TRUST_PACIFY_CAP` 0.6, `trust_pacify_factor` scaling the `should_escalate` chance; identical-RNG unit proof + 4-leg integration test; BIG-BLAST iteration — golden + 6 snapshots regenerated, 5 intent-preserving re-pins). **Iteration 111**: the noospheric field's `legitimacy_perceived` now has a decisional consumer — the theft-deterrence channel (one-sided `legitimacy_deterrence_factor` anchored at the 0.5 construction value, scaling the `enforce_theft` amount; deterministic take-proof unit test 0.12 vs 0.15 + 3-leg integration test; ZERO-blast — legitimacy decays to ~0.04 in every calibrated window AND theft never fires → golden byte-identical, NO regeneration needed). **Iteration 112**: the noospheric field's `collective_fear` now has a decisional consumer — the moral-panic legitimacy-damage amplifier (one-sided `collective_fear_panic_amplifier` anchored at 0.95 ABOVE the calibrated peak 0.903, scaling `MoralPanicResult::legitimacy_damage` when the §7.2 trigger fires; a shipping unit test caught the initial `clamp_01` silently erasing the amplification — factor now provably ∈ [1.0, 1.5] without a clamp; deterministic differential unit test 0.3268 vs 0.34 + 3-leg integration test; ZERO-blast — collective fear peaks at 0.903 < 0.95 anchor and the single seed-42 panic sits at 0.90 → golden byte-identical, NO regeneration). The ENTIRE §10.1 three-field layer now has decisional consumers. Remaining: social (obligation/peer-status) fields still observational | Iter 39/81/55/106/107/108/109/110/111/112 | Wired |
| 11 | §10.4 | **WIRED (Iter 101)** — the daily courtship ladder is attraction-gated: `try_advance` now receives the pursuer's live `total_attraction()` (the 0.6× floor genuinely binds below it; the interaction path was already gated). Same-test differential pinned: suppressed village stalls below control (ceiling 0.163 vs 0.477). `taboo_penalty` still unwired; the D4 scenario gate remains the other consumer | Iter 75–79 | Wired |
| 12 | §10.4 | **"Seek proximity" dynamic** — courting pairs sit beyond perception radius; the ladder stalls at the pair's trust ceiling (documented Iter-76 limitation) | Iter 76 | Behavioral gap |
| 13 | §10.7 | Household roles/traditions consumers — division of labor, childcare, hospitality | Iter 53 | Producer-side only |
| 14 | §13.1–13.6 | `narrative_dominance` → cluster assignment; trauma → group psychology; `cross_cutting_ties` stored as u32 count vs plan's Fixed tie-ratio; `sacred_events` uses synthetic `EventId`s (revisit if a real event log lands) | Iter 57 | Producer-side only |
| 15 | §29.2 | **WIRED (Iter 100)** — `suppression_resistance()` blends the armed core with `threat_level` (cohesion/grievance/anti-fragmentation) and amplifies by `legitimacy_of_violence` (radicalization); the protest-suppression force comparison reads the full threat model (never below raw strength, zero-at-zero preserved). `fighting_strength` still drives actual conflicts by design | Iter 74 | Wired |
| 16 | §8.1.10 | **WIRED (Iter 90–93)** — Respect Elders gates hostility toward elder-role holders (Iter 90/91) and Obey Ruler suppresses defiance toward the Guard Captain (Iter 93); `Institution.norm_ids` now consumed (temple ×1.5 reinforcement, Iter 90) | Iter 82–89 | Wired |

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
