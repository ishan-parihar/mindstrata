# Mindstrata — AP2 Full-Depth Re-Audit Plan

**Scope:** Verify that *everything* mandated by `docs/architecture/AP2.md` (the Human-Scale Deepening Architectural Plan) is implemented, **and** — critically — that it *operationalizes*: every system produces state that moves, is consumed by decisions, and produces realistic, differentiated, non-saturated behavior.

**Method:** Every audited system is examined on two axes simultaneously:

1. **Static validity** — read the code: does the plan-mandated struct/mechanic exist, is it wired into the tick loop, is it deterministic, are its consumers real (not write-only)?
2. **Dynamic operationalization** — run it: probe live state means/spreads across seeds and horizons, exercise parameter knobs, run differential (consumer on/off) harnesses, and confirm the emergent behavior the plan promises actually emerges.

**Baseline certified at plan creation:** commit `d8af39c` (Iteration 181), tree clean, **1,314 workspace tests listed**, golden replay byte-identical, CLI + noosphere inspector verified live.

---

## 0. Audit Method, Verdicts, and Evidence

### 0.1 The "Write-Only Detector" (the project's central failure pattern)

The dominant residual category in this codebase (see `docs/REMAINING_WORK_REPORT.md` §1) is **observational state that is produced but never read back into decisions**. Every system audit MUST mechanically answer:

- Does any code path **write** this state? (tick loop, daily pass, event handler)
- Does any code path **read** this state into a *decision* (utility, escalation, belief update, interaction, courtship, pricing, legitimacy, propagation)? A read by an inspector/TUI/`derive_*` does **not** count as a decisional consumer.
- Is the parameter that tunes it actually referenced outside `parameters.rs`?

Method: `rg` for every field of the audited struct; classify each reference as *producer*, *decisional consumer*, *observational consumer*, or *none*.

### 0.2 Verdict taxonomy

| Verdict | Meaning |
|---|---|
| `FULL` | Implemented, wired, decisional consumers live, calibrated, tested |
| `WIRED-LIVE` | Implemented and wired; consumer present but inert in calibrated windows (documented) |
| `WRITE-ONLY` | Produced, zero decisional consumers (the §1 RWR failure pattern) |
| `PARTIAL` | Plan mandates X, only X′ (subset/abstraction) exists |
| `MISSING` | Plan mandate with no implementation |
| `SUPERSEDED` | Replaced by a better design (must be documented in RWR reserved taxonomy) |
| `DEAD-PARAM` | Tuning knob with zero references outside parameters.rs |
| `UNVERIFIED` | Exists but not yet probed — do not claim liveness without a probe |

### 0.3 Evidence standard

Every finding must be **probe-pinned**: a concrete number (mean/max/spread/count/tick) over a named seed × horizon × configuration, produced by a runnable probe or test. Claims like "live and differentiated" require `probe: mean 0.42–0.58 across 4 seeds @5000`, not assertion.

### 0.4 Severity scale

- **S1** — Plan mandate entirely absent or behaviorally dead (write-only with no consumer anywhere).
- **S2** — Implemented but a documented plan effect does not occur in any probed window (inert consumer, saturated state, dead knob).
- **S3** — Implemented and live but under-tested / under-probed (no differential proof of the consumer).
- **S4** — Cosmetic/spec drift (plan vs code naming, RON spec constants unconsumed, reserved taxonomy).

---

## 1. The Audit Toolchain (verified working commands)

Every phase below draws on these primitives. All verified live during plan creation.

### 1.1 CLI inspectors (`cargo run -q -p mindstrata-cli -- sim ...`)

| Flag | Surface | AP2 § |
|---|---|---|
| `--inspect-agent N` | Per-agent state summary | §17.1 |
| `--psychology N` | Full psychology pipeline | §8 |
| `--beliefs N` | Belief state | §8.1.8 |
| `--decisions N` | Decision traces (provenance) | §16.2 |
| `--timeline N` | Event timeline | §6.3 |
| `--noosphere` | Legitimacy/memes/panics/propaganda/echo/rumors/field | §13 |
| `--market` / `--factions` / `--clans` / `--patronage` / `--records` | Institution dashboards | §10–13 |
| `--show-relationships from,to` | RelationshipV2 edge | §10.2 |
| `--export-metrics PATH` | CSV metric history | §6.5 |
| `--render-map` / `--render-replay` | Visual verification | §5 (Iter 157/171) |
| `--save-snapshot` / `--load-snapshot` | Snapshot round-trip | §16.1 |
| `scenario <name>` | drought/famine/pestilence/collapse/calm/riverford | §20 |

### 1.2 Probe examples (`cargo run -p mindstrata-benches --example <name> --release`)

| Probe | Purpose |
|---|---|
| `tick_probe` | Wall-clock tick cost calibration |
| `narr_probe` | Narrative script bounds + resilience factor liveness |
| `iter181_diag` | Revolution + birth-pipeline diagnostics |
| `altruism_sweep` | Parameter-rate sweep evidence (altruism→help) |
| `regression_gate` / `subsystem_gate` | Release-mode perf floors (CI) |

**Probe creation pattern** (used throughout this plan): a new `benches/examples/<system>_probe.rs` that builds a `Simulation` across seeds/horizons and prints mean/min/max/spread of the audited state, plus consumer inputs. New probes are the primary audit instrument where the existing suite lacks coverage.

### 1.3 Test suites (`cargo test -p mindstrata-tests <filter>`)

golden_replay · snapshot_tests (insta) · property_tests (proptest) · statistical_emergence · integration_tests · long_horizon_tests · scale_tests · comparison · behavioral_delta · agent_lifetime_trace · test_helpers (harness: `run_scenario_with_params`, `scenario_behavioral_delta`).

### 1.4 Interactive TUI (`cargo run -p mindstrata-tui`)

Keys: `space` auto-run, `n` step, `t`/`Tab` cycle views, `j`/`k` select agent, `w/e/d/r/s/p` issue behavior commands, `x` cancel. For observing per-tick dynamics live (spot-checks only; deterministic verification always via probes/tests).

### 1.5 Sanity gates

```bash
cargo test --workspace                     # all tests
cargo clippy --workspace -- -D warnings    # strict lint gate
cargo test -p mindstrata-sim spec_lint     # RON spec validation
cargo test -p mindstrata-tests golden_replay_vs_baseline
```

---

## 2. Master Audit Map (AP2 § → module → wiring)

| AP2 § | System | Module(s) | Tick-loop wiring | Key tests |
|---|---|---|---|---|
| §6 | Multi-timescale scheduler | `scheduler.rs` | `TickPhases::compute` once/tick | scheduler units, 10K stability |
| §7.2.1 | Genome | `biology/genome.rs` | `EmbodiedState::tick_update` | genome units, snapshot endocrine |
| §7.2.2 | Endocrine | `biology/endocrine.rs` | hourly + daily hormonal passes | `endocrine_states_500_ticks` snap |
| §7.2.3 | Skeletal | `biology/skeletal.rs` | body update | skeletal units |
| §7.2.4 | Muscular | `biology/musculoskeletal.rs` | body update | musculoskeletal units |
| §7.2.5 | Nervous/pain/trauma | `biology/nervous.rs` | body update + daily trauma decay | nervous units, Iter-176 trauma |
| §7.2.6 | Reproductive | `biology/reproductive.rs` | pregnancy pass + birth pass | birth-pipeline integration |
| §7.2.7 | Digestive | `biology/digestive.rs` | body update | digestive units |
| §7.2.8 | Respiratory | `biology/respiratory.rs` | body update | respiratory units |
| §7.2.9 | Cardiovascular | `biology/cardiovascular.rs` | body update | cardiovascular units |
| §7.3.1 | Immune | `biology/immune.rs` | body update + disease pass | immune units, pestilence scenario |
| §7.3.2 | Circadian | `biology/circadian.rs` | daily | circadian units |
| §7.3.3 | Thermal | `biology/thermal.rs` | daily | thermal units, winter hardship |
| §7.4 | EmbodiedState facade | `biology/mod.rs` | `derived_*` in needs/body blocks | derived-health units |
| §8.1.1 | Interoception | `psychology/interoception.rs` | cognitive block | interoception units |
| §8.1.2 | Attention | `attention.rs` (root) | perception block | attention units |
| §8.1.3 | Memory | `memory.rs` (root) + narrative | memory encoding/consolidation | memory units, Iter-181 |
| §8.1.4 | Appraisal/22 emotions | `appraisal.rs` (root) | appraisal + emotion blocks | 12-emotion consumer tests |
| §8.1.5 | Motivation | `psychology/motivation.rs` | motivation block | motivation units |
| §8.1.6 | Personality/temperament | `person.rs`, `psychology/developmental.rs` | yearly trait pass | Iter-179/180 tests |
| §8.1.7 | Self-model | `psychology/self_model.rs` | self-model block | self-model units |
| §8.1.8 | Belief/epistemic | `belief_update.rs`, `social/epistemic.rs` | belief block | belief units, Iter-168 |
| §8.1.9 | Theory of mind | `psychology/theory_of_mind.rs` | daily ToM | ToM units |
| §8.1.10 | Moral cognition | `psychology/moral_cognition.rs` | moral block + escalation gates | moral units, Iter-90/91/93 |
| §8.1.11 | Speech acts | `social/speech_act.rs` | interaction block | speech-act units, Iter-95 |
| §8.1.12 | Executive function | `psychology/cognitive_runtime.rs` | executive block | executive units |
| §8.1.13 | Developmental | `psychology/developmental.rs`, `biology/development.rs` | daily/seasonal | development units |
| §8.1.14 | Attachment | `psychology/attachment.rs` | daily attachment block | attachment snap, Iter-173 |
| §8.1.15 | Psychopathology | `psychology/psychopathology.rs` | psychopathology block | psych units |
| §8.1.16 | Prospection | `psychology/imagination.rs` | prospection block | Iter-103/117 |
| §8.1.17 | Narrative | `psychology/narrative.rs` | narrative block + stress factor | Iter-181 tests |
| §8.1.18 | Cultural cognition | `psychology/cultural_cognition.rs` | daily + taboo gates | Iter-165…169 |
| §8.1.19 | Skill/habit | `psychology/skill.rs` | skill block | skill units |
| §8.1.20 | Decision policy | `psychology/decision_policy.rs` | decision block | decision units |
| §9.1 | Cognitive runtime | `psychology/cognitive_runtime.rs` | executive/decision | runtime units |
| §9.2 | Neural-like (embeddings/RL) | `psychology/neural_like.rs` | learned_delta in utility | Iter-94 |
| §10.1 | Three fields | `social/relational_field.rs` | field sync | Iter-107…114 |
| §10.2/10.3 | RelationshipV2 + stages | `social/relationship_v2.rs`, `relationship_stages.rs` | interaction + daily decay | stage snapshots |
| §10.4 | Courtship/attraction | `social/attraction.rs`, `courtship.rs` | daily courtship | Iter-101/118/165 |
| §10.5 | Marriage/pair-bond | `social/marriage.rs` | marriage pass | marriage units, Iter-175 |
| §10.6 | Kinship | `social/kinship.rs` | kinship daily | kinship units |
| §10.7 | Household | `social/household.rs` | household daily + pooling | Iter-119 |
| §10.8 | Clan | `social/clan.rs` | clan passes | clan units, clans CLI |
| §11.1 | Status dims | `social/status_dims.rs` | status pass | status units, Iter-106 |
| §11.2 | Relational power | `social/relational_power.rs` | power in escalation | Iter-102 |
| §11.3 | Hierarchy | `social/hierarchy.rs` | hierarchy pass | hierarchy units |
| §12.2/12.3 | Group formation | `social/group_formation.rs` | group pass | Iter-121 |
| §12.4 | Cult | `social/cult.rs` | cult check | cult units |
| §12.5 | Ritual | `culture/ritual.rs` | duodeca ritual pass | ritual units |
| §13.1/13.2 | Meme | `culture/meme.rs`, `meme_aggregator.rs` | gossip + daily aggregate | Iter-174 |
| §13.3 | Rumor v2 | `culture/rumor_v2.rs` | gossip/rumor pass | rumor hops tests |
| §13.4 | Propaganda | `culture/propaganda.rs` | campaign pass | Iter-177 |
| §13.5 | Collective memory | `culture/collective_memory.rs` | daily memory | Iter-129 |
| §13.6 | Echo chambers | `culture/echo_chamber.rs` | daily ecology | Iter-120 |
| §13+ | Noosphere field | `noosphere/field.rs`, `legitimacy.rs`, `moral_panic.rs` | field passes | Iter-112/115 |
| §15 | RON specs | `specs/**` (32 files) | `spec_lint.rs` | spec_lint test |
| §16 | Provenance | `provenance.rs` | trace recording | Iter-170 |
| §17 | LOD/tiers/budget | `agent_tier.rs`, `attention.rs`, `meme_aggregator.rs` | tier reclassify | Iter-144/158/159 |

*Generic labels in the Key tests column (e.g. "genome units", "skeletal units", "cardiovascular units") mean "the unit tests inside that module's `#[cfg(test)]` block" — the auditor's job is to verify those tests exist and assert the module's contract, not to find a single named test. Named references (Iter-NNN) pin specific shipped tests/probes.*

---

## 3. Audit Phases

Each phase ends with a **findings log entry** (see §14) and a verdict per system. Phases are independent enough to run in parallel batches; 1–2 and 8–10 have no interdependencies.

---

### Phase 1 — Baseline Integrity & Determinism Core

**Goal:** Certify that the substrate every other audit relies on is sound: determinism, RNG streams, scheduler phases, snapshot round-trip, and the full green baseline.

**AP2 §:** 4.1 (determinism non-negotiable), 6 (multi-timescale model), 14.1 (compatibility).

**Static checks:**
- [ ] `scheduler.rs`: confirm all 10 phases (Fast 1, Hourly 6, Deca 10, Duodeca 12, Centum 100, Daily 144, Quincent 500, Weekly 1008, Seasonal 4320, Yearly 51840) match plan §6.2 and the state-doc table. Verify `TickPhases::compute` is called exactly once per tick and no system bypasses phase gating.
- [ ] RNG streams: enumerate `RngStream` variants; verify each stream is virgin (only its owning subsystem draws) by `rg` on all `get_mut(RngStream::…)` sites; flag any subsystem drawing from another's stream (plan §4.1).
- [ ] `from_snapshot`: every restore path (`#[serde(default)]` for weather/tech/court/diplomacy, re-seed guards) present; no double-seed or missed-reseed (compare against the list in `sim.rs` §16.1 restore block).
- [ ] `Fixed` math: no `f64` in hot paths unless documented (`should_birth` 1/35040 precedent); `clamp_01` misuse audit on the two known near-miss patterns (Iter-112 clamp erased amplification; Iter-172 tone floor).

**Dynamic tests:**
- [ ] `cargo test -p mindstrata-tests golden_replay_vs_baseline` → byte-identical.
- [ ] `cargo test -p mindstrata-tests long_horizon_tests` → 50K determinism (metrics + per-agent projection) on seeds 42/7/99.
- [ ] `cargo test -p mindstrata-tests property_tests` → all invariant sweeps (bounds, no negative age, belief confidence ∈ 0..1, no resource duplication).
- [ ] Snapshot round-trip: run `--save-snapshot` at tick T, resume via `--load-snapshot` for N ticks, and diff against a continuous run (the `save_load_round_trip.rs` CLI test covers this; also run manually at 10K).
- [ ] `cargo run -p mindstrata-benches --example tick_probe --release` → record ticks/sec for the 12/24-agent grids (feeds Phase 10 gate context).
- [ ] TUI spot-check: launch, step 5 ticks, confirm `n`/`space`/`j`/`k`/`q` behave; issue one command (`w`) and cancel (`x`).

**Acceptance:** golden byte-identical; 50K determinism holds; phase map complete; zero cross-stream RNG draws; snapshot resume byte-identical.

---

### Phase 2 — Biological Substrate (§7)

**Goal:** Verify every biological system exists, is wired into `EmbodiedState::tick_update`, and *modulates psychology/behavior* — not just holds numbers. Special attention to the Iter-172/176 lesson: **saturation** (values pinned at 0 or 1 with no per-agent spread) and **dead knobs** (params with zero references).

**Systems:** genome, endocrine (7 axes), skeletal, muscular, nervous (arousal/pain/trauma), reproductive (puberty/pregnancy/birth), digestive, respiratory, cardiovascular, immune, circadian, thermal, development, `EmbodiedState` facade.

**Static checks (per system):**
- [ ] Struct fields vs plan §7 structs (e.g. `EndocrineState` axes match §7.2.2; `NervousSystemState` includes pain + trauma; `ReproductiveState` matches §7.2.6).
- [ ] Producer sites: which block feeds each field? (`rg` the field name in sim.rs.)
- [ ] Decisional consumers: does any psychology/decision path read it? (e.g. stress axis → cognitive state; pain → attention; endocrine → appraisal.) Flag write-only fields.
- [ ] Each parameter in `parameters.rs` touching biology: referenced anywhere outside parameters.rs? (Phase 5 found 8 dead knobs this way.)
- [ ] Time-scale correctness: fast systems per-tick, hormones hourly/daily, pregnancy daily, development seasonal (§6.2 frequency table).

**Dynamic tests:**
- [ ] Run `cargo test -p mindstrata-tests snapshot_tests` — specifically `endocrine_states_500_ticks` and `agent_states_1000_ticks`: read the snap and check for *differentiation* (not all agents identical).
- [ ] New probe `biology_probe.rs`: seeds {42,1,7,99,46} × horizons {1000, 5000, 20000} × scenarios {calm, drought, famine, pestilence}: print per-axis mean/min/max and % of agents at 0.0 or 1.0. **Pass:** no axis saturates at 1.0 for >50% of agents in calm; famine/pestilence worlds show directional shifts (hunger/stress up, health down).
- [ ] Differential harness (`behavioral_delta`): stress-recovery rate 0.05 vs 0.15 → mean stress moves; trauma-decay 0.0005 vs 0.002 → mean trauma moves (both directions per Iter-176 evidence).
- [ ] Birth pipeline end-to-end: `cargo test -p mindstrata-tests integration_tests reproduction` filters; verify pregnancy → birth → parentage → marriage record → children_born chain on seed 46 @160K (Iter-180 anchor).
- [ ] `--psychology <id>` and `--inspect-agent <id>` on a famine run: confirm hunger/stress/arousal visually elevated vs calm run (operational realism spot-check).

**Acceptance:** every §7 system `FULL` or `WIRED-LIVE` with a documented inert-reason; zero S1/S2; snapshot differentiation verified.

---

### Phase 3 — Psychological Runtime (§8)

**Goal:** Verify all 20 §8.1 systems + their *decisional consumers*. This is the highest-density section and the historical home of write-only state (RWR §1 rows 2, 8, 9, and Iter-124's emptied-emotion-field bug).

**Static checks (per system):**
- [ ] Struct presence vs plan (SelfModel §8.1.7, EpistemicState §8.1.8, OtherModel §8.1.9, MoralCognition §8.1.10, ProspectionState §8.1.16, NarrativeIdentity §8.1.17, SkillState §8.1.19, DecisionPolicy §8.1.20).
- [ ] Tick-loop block presence for each (executive, motivation, regulation, moral, prospection, narrative, psychopathology, cultural cognition, skill, self-model).
- [ ] **LOD gating audit:** confirm each focal-only block is guarded by `AgentTier`/budget (§17) and that Background agents correctly skip (probe-pinned in Iter-181 for narrative; check the same for regulation/moral/prospection).
- [ ] **Emotion consumer ledger:** the RWR §1 row-2 list (12 emotions wired, Iter-98…130) — mechanically re-verify each consumer site exists and reads the *live* emotion field (the Iter-124/78 emptied-field pattern must not regress).
- [ ] Trait/temperament: `plastic_update_traits` yearly gate present; `TraitConstitution` anchor; altruism→help wiring (Iter-180).

**Dynamic tests:**
- [ ] `cargo test -p mindstrata-tests behavioral_delta` — every consumer differential (gratitude→help, awe→reverence, taboo family, altruism) still directional on the pinned seeds.
- [ ] New probe `psych_probe.rs`: seeds × {calm, famine, pestilence} × horizons {2000, 10000}: print for each psychology system: mean/spread, % at zero, and its consumer-input liveness. **Pass:** each of the 12 emotion consumers has a window where its emotion is non-zero OR the documented inert-reason holds (e.g. jealousy: inputs dormant in calm).
- [ ] `--psychology <id>` on a 10K calm run and a pestilence run: compare emotion/regulation/self-model surfaces for realism (stressed world should show elevated fear, more rumination/suppression, lower self-esteem).
- [ ] Prospection: verify D1/D2 dread→utility fold is live (Iter-103) via `--decisions <id>` traces; confirm scenario-kind discriminator D1–D6 exists (Iter-117).
- [ ] Narrative: `cargo run -p mindstrata-benches --example narr_probe --release` → scripts bounded (not saturated), resilience factor ≈1.0 ± spread, focal-only.
- [ ] `cargo test -p mindstrata-tests snapshot_tests` attachment + agent snapshots: verify per-agent differentiation.

**Acceptance:** every §8.1 system `FULL` or `WIRED-LIVE` with documented inert-reason; zero write-only systems; all 12 emotion consumers differential-proven; LOD gating correct.

---

### Phase 4 — Cognitive Runtime & Neural-Like Mind (§9)

**Goal:** Verify the "LLM-like" layer: `CognitiveRuntime`/`DecisionPolicy` integration, concept embeddings, spreading activation, predictive error, learned RL values, script grammar.

**Static checks:**
- [ ] `psychology/neural_like.rs`: concept-vector machinery exists; similarity/dot-product math is Fixed-point deterministic (§9.2).
- [ ] Spreading activation: node/edge activation update present and consumed (noospheric field is the network; verify the per-agent association side exists if plan-mandated).
- [ ] Predictive error: `ExpectationState.last_prediction_error` EXISTS in `neural_like.rs` (unit-tested: `prediction_error_tracks_surprise_and_learns`), but the plan §9.2 mandates a **fold into attention/belief/emotion intensity** — current reads are only in integration tests, so determine whether a decisional consumer exists or it is `WRITE-ONLY` (the open question; do not claim liveness without a probe).
- [ ] RL values: `learned_delta` → `compute_utility` fold (Iter-94) — verify zero-at-prior keeps tick-0 byte-identical.
- [ ] Script grammar: courtship script (§9.2) maps to the real courtship ladder (§10.4).

**Dynamic tests:**
- [ ] `cargo test -p mindstrata-sim neural_like` units (if none exist, flag S3 coverage gap and add a probe).
- [ ] New probe `learned_values_probe.rs`: seeds × horizons {1000, 10000, 50000}: print `learned_delta` distribution and utility influence; verify non-zero after learning, zero at tick 0.
- [ ] Differential: force a high-reward world (abundant grain) vs scarcity — learned values should diverge directionally.
- [ ] Confirm plan §9.3 (external LLM) is documented *optional/unimplemented by design* — no action beyond the RWR note.

**Acceptance:** RL learning loop proven live; each §9.2 mechanism verdict logged; predictive-error mechanism either proven live or explicitly flagged with severity (this is the most likely genuine `MISSING`/`PARTIAL` in the plan).

---

### Phase 5 — Relational Systems (§10)

**Goal:** Verify RelationshipV2 depth, stages, courtship/marriage/kinship/household/clan, the three relational fields, jealousy/betrayal/reconciliation — as *behavioral* systems.

**Static checks:**
- [ ] `RelationshipV2` field coverage vs plan §10.2 (deep dimensions: intimacy/passion/commitment/dependence/power_balance/jealousy/gratitude/moral_debt/betrayal_history/reconciliation_history).
- [ ] Stage machine: `relationship_stages.rs` — social/negative/authority/kin/romantic branches; transition gates; the reserved taxonomy (LordVassal/GuardCitizen, Cousin, InLaw) confirmed *reserved* not broken.
- [ ] Three fields (§10.1): sensory/social/noospheric — all six decisional consumers from Iter-107…114 still wired (fear contagion, kin buffer, belief rigidity, trust pacify, legitimacy deterrence, collective-fear panic amplifier, peer-status envy, obligation restraint).
- [ ] Courtship ladder: attraction-gated `try_advance`, `seek_step` proximity, taboo penalty (Iter-165), D4 reachability.
- [ ] Marriage: formation rate param (Iter-175), dissolution paths (death/abandonment/annulment/divorce/exile/religious).
- [ ] Household pooling (Iter-119): multi-member only, dependents-first rations.
- [ ] Clan: formation from marriages, feud enmities, myth adoption.

**Dynamic tests:**
- [ ] `cargo test -p mindstrata-tests statistical_emergence` — D4 reachability (10 seeds), faction variance (20 seeds).
- [ ] `cargo test -p mindstrata-tests snapshot_tests` — `relationship_stage_distribution_2000_ticks` snap: stages actually distributed (not all `Unnoticed`).
- [ ] New probe `relational_probe.rs`: seeds × horizons {5000, 20000}: relationship-dimension means/spreads (intimacy, jealousy, power_balance), stage histogram, marriage/courtship counts, household membership distribution, clan counts. **Pass:** >1 stage populated; some edges show non-trivial intimacy; marriages/courtships fire in at least one seed (per-seed variance is expected — this is emergence, not determinism).
- [ ] Jealousy chain: verify armed-but-inert status (Iter-126): emotion=0 probe in calm, and a forced-input unit test proving charge_jealousy → strain → should_dissolve wiring.
- [ ] `--clans`, `--patronage`, `--show-relationships` on a 20K run: visually confirm structures exist with real members.

**Acceptance:** §10 verdicts all `FULL`/`WIRED-LIVE`; stage distribution differentiated; courtship→marriage→household→clan chain reachable (D4 gate proven by statistical test); reserved taxonomy documented.

---

### Phase 6 — Power, Status & Hierarchy (§11)

**Static checks:**
- [ ] `StatusDimensions` component coverage vs §11.1 (dominance/prestige/authority/legitimacy/wealth/moral_reputation/network_centrality/institutional_rank/honor/shame).
- [ ] `RelationalPower` fields vs §11.2; `power_balance` → escalation fold (Iter-102) and `institutional_rank` → `effective_status` (Iter-106) present.
- [ ] Hierarchy formation/stabilization/destabilization paths (§11.3) — which are implemented vs planned? (Hierarchy emergence from competence/wealth/violence; legitimacy/ritual stabilization.)

**Dynamic tests:**
- [ ] `--factions` + `--inspect-agent` on a 20K run: status components visible and differentiated.
- [ ] New probe `status_probe.rs`: seeds × {calm, famine, collapse}: status mean/spread + rank-correlation with wealth; **pass:** famine/collapse worlds show changed status ordering (wealth collapse shifts rank).
- [ ] Differential: `power_balance` in escalation — the Iter-102 unit test + a behavioral-delta run (dominant vs subordinate dyad).

**Acceptance:** §11.1/11.2 `FULL`; §11.3 hierarchy verdict logged (may be `PARTIAL` if stabilization paths are observational — flag honestly).

---

### Phase 7 — Group Coherence & Communion (§12)

**Static checks:**
- [ ] Group types §12.1 — which of the 10 types exist as concrete registries? (Household, Clan, Faction, Cult, PeerGroup known; Congregation/Guild/Warband/Patronage-network need mapping.)
- [ ] Formation pressure formula §12.2 (shared_grievance + identity + synchrony + interaction + leadership + threat − cost − suppression) — map each term to a real input.
- [ ] Attachment scaling to groups §12.3 — do group dynamics differ by member attachment style? (If purely theoretical, flag `PARTIAL`/`SUPERSEDED`.)
- [ ] Cult model §12.4 — `CultDynamics` fields vs plan; formation/dissolution conditions wired.
- [ ] Ritual §12.5 — effects wired (bonding, anxiety reduction, norm reinforcement, legitimacy, collective effervescence).

**Dynamic tests:**
- [ ] `cargo test -p mindstrata-tests integration_tests` faction/cult filters + Iter-121 shared-trauma group-bonding fold.
- [ ] New probe `group_probe.rs`: seeds × {calm, famine, collapse} × {10000, 50000}: group counts (factions, peer groups, cults, households, clans), member distributions, group cohesion means. **Pass:** at least one group type forms in ≥1 seed; cohesion differentiated.
- [ ] Ritual: verify `executed_ritual_count > 0` at 4320/8640 (the 10K snapshot already pins this — confirm and extend with the probe).
- [ ] Cults: collapse/famine worlds — does cult formation ever fire? (Cooldown at §12.4; if never in any probed window, verdict `WIRED-LIVE` with the inert-reason documented.)

**Acceptance:** group-type map complete (each type either implemented or explicitly `SUPERSEDED`/reserved); ritual execution proven; cult formation honest verdict.

---

### Phase 8 — Cultural & Noospheric Field (§13)

**Goal:** Verify memes/rumors/propaganda/ritual/collective-memory/echo-chambers/sacred/noosphere-field as a propagating *field* with decisional consumers (Phase 5 tuning campaign Iter-174…178 closed the knobs; re-verify liveness holds).

**Static checks:**
- [ ] `Meme` struct vs §13.1 (lineage, emotional/identity/moral charge, virality, mutation, institutional backing, sacredness); 14 content types.
- [ ] Transmission formula §13.2 — which terms are real vs simplified (documented: code uses simplified product; spec weights unconsumed → S4).
- [ ] Rumor v2 §13.3 fields + hops-degradation.
- [ ] Propaganda §13.4: campaign fields, channels, effectiveness knob live (Iter-177).
- [ ] Collective memory §13.5 (ritual/storytelling maintenance; nostalgia preservation Iter-129).
- [ ] Echo chambers §13.6: `cross_cutting_ties` u32 vs plan Fixed tie-ratio (documented S4 delta); polarization index.
- [ ] Noosphere field: `field.rs` spreading activation, `legitimacy.rs`, `moral_panic.rs` lifecycle (Iter-115).

**Dynamic tests:**
- [ ] `--noosphere` on a 1500-tick run (verified live at plan creation): memes with hosts, campaigns active, polarization > 0, rumors present, nodes activated.
- [ ] `cargo test -p mindstrata-tests integration_tests` rumor filters (transmit through population, evidence degrades with hops).
- [ ] New probe `meme_probe.rs`: seeds × horizons {2000, 10000, 50000}: meme host counts (differentiated spread, no universal dominance), virality mean, campaign effectiveness mean, polarization trajectory, moral-panic triggers. **Pass:** the Iter-174 healthy envelope (all memes active, host spread 23/1/3/13/4 at 10K) reproduces; polarization rises over time.
- [ ] Behavioral-delta: propaganda effectiveness 1.0 vs 2.0 → campaign effectiveness strictly higher (Iter-177 test re-run); meme virality 0.3 vs 1.2 → host spread differs (Iter-174).
- [ ] Moral panic: verify trigger → escalate → drain lifecycle completes (Iter-115 window [11500,13500] anchor) via the panic integration tests.

**Acceptance:** §13 systems `FULL`; knobs live; the two documented S4 deltas (spec weights unconsumed, cross_cutting_ties type) re-confirmed as cosmetic.

---

### Phase 9 — Integration, Specs & Provenance (§14–§16)

**Static checks:**
- [ ] Tick loop order vs plan §14.2's 47 steps: map each planned step to a real block in `sim.rs` tick; the implementation is a consolidated 34-block loop — produce the mapping table and flag any planned step with no real block (e.g. "speech/language acts" lives inside interactions; "theory-of-mind update" inside daily pass — confirm equivalents).
- [ ] Compatibility facade §14.1: `derived_health/energy/hunger/thirst/fatigue/sickness/injury/fertility` all present and consumed by legacy needs/body blocks.
- [ ] RON specs §15: **36 files** on disk (verified at plan creation) — every file the plan §15.1 proposed is present (genome, hormones, organs, diseases_v2, life_stages, reproduction, cognitive_systems, emotions_v2, regulation_strategies, identity_frames, moral_foundations, attachment_styles, relationship_stages, courtship, marriage, kinship, status_roles, groups, memes, rituals, propaganda, taboos_v2, sacred_symbols, education) plus the core/scenario files; all carry `// Maps to:` comments; `spec_lint` validates.
- [ ] Provenance §16: 13 `ProvenanceCategory` variants; `SystemTrace`/`DecisionTrace` recorded; `--decisions` traces explain major events (plan §19 Phase-5 acceptance).

**Dynamic tests:**
- [ ] `cargo test -p mindstrata-sim spec_lint`.
- [ ] `cargo test -p mindstrata-tests golden_replay` + `comparison`.
- [ ] `--decisions <id>` on a 10K run: inspect a decision trace — confirm it shows the causal stack (need → emotion → utility → choice) per §16.2 example.
- [ ] `--timeline 50` + `--export-metrics`: event timeline and CSV row count sane.
- [ ] New probe `tick_order_probe.rs` (optional): instrument block entry order for 10 ticks and diff against the documented order (guards against silent reordering).

**Acceptance:** tick-order mapping complete with every §14.2 step accounted for (by name or documented consolidation); spec_lint green; provenance traces legible.

---

### Phase 10 — Performance, LOD & Scale (§17)

**Static checks:**
- [ ] `AgentTier` reclassify thresholds/maturity gate (Iter-159); tier-mix envelope 4n/5.
- [ ] `CognitiveBudgetTracker` — appraisal/prospection budget gates; Background zero-budget caps.
- [ ] Social-interaction gate excludes Background (Iter-158).
- [ ] Relationship caching: dormant decay path.
- [ ] Meme aggregation: `should_compute` gate (Iter-143).

**Dynamic tests:**
- [ ] `cargo test -p mindstrata-tests scale_tests` (full-capacity 48-agent).
- [ ] `cargo test -p mindstrata-tests long_horizon_tests` (10K/50K stability; ~72s).
- [ ] `cargo run -p mindstrata-benches --example subsystem_gate --release` and `regression_gate --release` → perf floors hold.
- [ ] `cargo bench -p mindstrata-benches` (criterion): record tick-loop + subsystem numbers as the perf baseline.
- [ ] New probe `tier_probe.rs`: population 6/12/24/48 × seeds: tier distribution (Focal/Secondary/Background counts) — verify the Iter-159 gradient materializes.

**Acceptance:** tier gradient present at all population sizes; perf gates green; cognitive budget has teeth (frozen Background memory proof exists).

---

### Phase 11 — Testing & Validation Strategy Audit (§18) — *audit the audit*

**Goal:** Verify the plan's own §18 testing mandates are all present and meaningful, and that they still pass.

**Static checks:**
- [ ] §18.1 unit examples → each has a real test (hormone bounds, genome inheritance ranges, pregnancy adult-only, attachment→distress, stage thresholds, meme lineage, propaganda→belief, sleep→heuristic, pain→attention, kinship→incest prevention).
- [ ] §18.2 property examples → real proptests (determinism across seeds, no negative age, no impossible fertility, no stage regression without event, belief ∈ 0..1, no resource dup, no action without local knowledge).
- [ ] §18.3 integration examples → real tests (courtship→marriage→household, inheritance of predispositions, childhood trauma→insecurity, chronic stress→aggression/depression, rumor degradation, propaganda shifts belief, ritual→cohesion, faction under grievance, cult under crisis).
- [ ] §18.4 statistical emergence → real multi-seed tests (proximity→friendship, compatibility→marriage, parental resemblance, gossip hops, propaganda↔legitimacy, stress↔conflict, ritual↔stability, inequality↔faction, attachment↔volatility). **Flag any row with no covering test as an S3 coverage gap** — the §18.4 table is the plan's own checklist.

**Dynamic tests:**
- [ ] `cargo test --workspace` full suite → 1,314+ green.
- [ ] `cargo clippy --workspace -- -D warnings` → 0.
- [ ] `cargo test -p mindstrata-tests` each of: statistical_emergence, property_tests, comparison, agent_lifetime_trace, behavioral_delta.
- [ ] New probe `emergence_probe.rs` for any §18.4 row lacking coverage (e.g. parental resemblance: correlate parent/child trait predispositions across all births in a 160K seed-46 run).

**Acceptance:** every §18.1–18.4 row maps to a real, passing test or an S3-flagged gap with a planned probe.

---

### Phase 12 — Roadmap Acceptance Criteria (§19)

**Goal:** Walk the plan's own Phase 0–5 acceptance criteria and verify each *operationally*.

| Plan acceptance | How to verify |
|---|---|
| P0: all 266 tests still pass / old BodyState API works / replay preserved | golden + full suite (trivially true at 1,314) |
| P1: hunger affects mood; stress hormones reduce planning; sleep debt impairs cognition; adults court; pregnancy possible; children inherit traits; elders decline | biology probe + `--psychology` + birth-chain integration tests |
| P2: agents ruminate; misremember under emotion; defend identity-linked beliefs; infer intentions; imagine futures; explainable decisions | psych probe + memory units + ToM units + `--decisions` traces |
| P3: friendships gradual; courtship emerges; marriage→household; kinship matters; attachment affects conflict; status struggles | relational probe + stage snapshots + Iter-121 group tests |
| P4: rumors mutate; institutions run campaigns; rituals increase cohesion; cults/factions form; legitimacy rises/falls; polarization measurable | meme probe + `--noosphere` + moral-panic tests |
| P5: 10K stability; emergent stories legible; no system dominates unnaturally; debug traces explain events | long-horizon tests + `--noosphere` + saturation checks in every probe + `--decisions` |

**Acceptance:** produce a row-by-row table with evidence (test/probe name + pinned numbers) for all 6 phases × 4–6 criteria.

---

### Phase 13 — Emergent Realism Deep-Dive (§20 example scenario)

**Goal:** Re-run the plan's §20 walkthrough scenario (drought over 2,200 ticks) and confirm the promised emergent chain: hunger/stress → rumors → faction formation → conflict/legitimacy erosion — actually unfolds.

**Dynamic tests:**
- [ ] `cargo run -q -p mindstrata-cli -- scenario drought --map` → walk the timeline: hunger rises, fear/stress rise, rumors fire, faction/conflict events appear.
- [ ] Extend to famine/collapse: compare emergent trajectories across all 6 scenarios with `--timeline 50` + `--export-metrics` (CSV diff).
- [ ] New probe `scenario_probe.rs`: for each scenario × 3 seeds: print final hunger/fear/legitimacy/conflict/grain means + event counts; confirm each scenario produces a *distinguishable, plausible* world state (drought ≠ famine ≠ collapse ≠ calm).
- [ ] Legibility: `--noosphere` at scenario end — polarization/memes/panics reflect the scenario (collapse → more panic).

**Acceptance:** §20 chain reproduced; scenario worlds statistically distinguishable; emergent stories legible end-to-end.

---

### Phase 14 — Cross-Cutting Sweeps (final)

**Goal:** The mechanical sweeps that catch the residual class of bugs independent of any single system.

- [ ] **Dead-parameter sweep:** for every field of `SimParameters`, `rg` for its name in `sim.rs` + modules. Any field with zero references outside parameters.rs → `DEAD-PARAM` finding (the exact class Phase 5 eradicated — verify zero remain; this is the highest-value sweep).
- [ ] **Write-only state sweep:** for each `pub` field of each audited struct, confirm a reader exists outside inspectors/derives. Produce the S1/S2 list.
- [ ] **Saturation sweep:** in every probe, flag any state with >80% of agents pinned at 0.0 or 1.0 in calm worlds (the Iter-164/172/176 signature).
- [ ] **RNG stream audit:** confirm no subsystem's behavior changed its draw count since the golden anchor (re-run golden after each probe; probes must be read-only).
- [ ] **Spec-vs-code drift:** compare each RON spec's constants/weights to the consuming code; log S4 drifts (known: propaganda weights).
- [ ] **TODO/FIXME/panic! sweep:** `rg "TODO|FIXME|unimplemented!|todo!|panic!"` across sim — gate at zero (state doc claims zero).
- [ ] **Determinism re-certification:** full golden + 50K determinism re-run at the end of the entire audit to prove probes/tests introduced no drift.
- [ ] **ADR compliance (§21):** ADR-001 deterministic core (P1), ADR-002 abstract biology (P2), ADR-003 neurosymbolic psychology (P4), ADR-004 developmental relationships (P5), ADR-005 culture as propagating field (P8), ADR-006 LOD agents (P10) — one-line confirmation each that the implementation still honors the decision (content is covered by the phases above; this row makes the mapping explicit).
- [ ] **§7.2.6 ethical compliance (the plan's hard rules):** verify age-gating in code — `rg` the adult/sexual-maturity gates on courtship initiation, attraction/romance targeting, and the birth/pregnancy path; confirm no romantic or reproductive interaction can route toward non-adult agents. Dynamic: probe asserts zero courtship/romance/birth events targeting agents below the adult gate across all seeds/horizons.
- [ ] **§23/§24 marker:** Immediate Next Steps / Final Summary are historical — superseded by the implemented codebase; deliberately no audit action (recorded here so the drop is explicit, not accidental).

---

## 4. Findings Log Format

Each phase appends to `docs/architecture/AP2_AUDIT_FINDINGS.md`:

```markdown
## Phase N — <name> (date)

| System | AP2 § | Verdict | Evidence (probe/test + numbers) | Severity | Action |
|---|---|---|---|---|---|
| Endocrine stress axis | §7.2.2 | FULL | biology_probe: mean 0.42–0.58, 0% saturated @4 seeds×5000 | — | — |
| Predictive error | §9.2 | PARTIAL | no consumer found; rg: prediction_error only in neural_like.rs | S1 | wire into attention/belief (Iter N+1) |
| … | | | | | |
```

Rules: every system in the Master Audit Map gets exactly one row per phase pass; verdicts must be evidence-backed; S1/S2 findings create RWR-style residuals tracked to an iteration.

---

## 5. Recommended Execution Order

1. **Phase 1** (baseline) first — everything depends on it.
2. **Phases 2, 5, 8** (biology, relational, noosphere) in parallel — three independent probe authors.
3. **Phase 3 + 4** (psychology + cognitive runtime) — sequential after 2 (they read biological state).
4. **Phases 6, 7, 9, 10, 11** in parallel (lightweight, mostly re-running existing suites).
5. **Phase 12** (roadmap acceptance) after 2–4, since it aggregates their evidence.
6. **Phase 13** after 8 (scenario legibility depends on noosphere probes).
7. **Phase 14** last, then full determinism re-certification.

Estimated effort: ~14 probe files + ~30 sweep findings; each phase is a self-contained deliverable (findings table) so partial completion is useful.

---

## 6. Known Pre-Existing Deltas to Re-Confirm (not to re-fix)

From `docs/REMAINING_WORK_REPORT.md` (reserved taxonomy + documented gaps). Each must be re-confirmed as *still intentional* and logged as S4 (or escalated if it has grown):

- `LordVassal`/`GuardCitizen` relationship stages — no live producer (reserved).
- `KinshipLink::Cousin` type reserved; `InLaw` edges never created by code.
- `culture::ideology::BeliefEcology` legacy dead struct (deliberately untouched).
- §10.4 probabilistic daily ladder roll superseded by deterministic gates (do not "fix").
- §9.3 external LLM role — unimplemented by design (optional).
- `specs/culture/propaganda.ron` weighted-formula constants unconsumed (simplified multiplicative product).
- `cross_cutting_ties` stored as u32 vs plan's Fixed tie-ratio; `sacred_events` uses synthetic `EventId`s.
- `pain.rs` → integrated in `nervous.rs`; `attention_v2.rs` → root `attention.rs`; `appraisal_v2.rs` → root `appraisal.rs` (naming consolidation, cosmetic).
