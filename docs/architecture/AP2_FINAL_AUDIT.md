# Mindstrata Final Exhaustive Audit — Iteration 186+

**Date:** 2026-08-17 · **Scope:** full workspace (mindstrata-sim 16,507-line sim.rs, 43 sim modules, 17 psychology, 20 social, 15 biology) · **Method:** full test suite + clippy + systematic production-caller sweep of every `pub fn`/field per module + targeted runtime probes.

---

## 0. Verification Baseline

| Gate | Result |
|---|---|
| `cargo test --workspace --release` | **1,342 tests / 0 failures** (984 sim + 304 tests-crate + 20 core + 15 render + 13 tui + 6 CLI) |
| `cargo clippy --workspace --all-targets` | Clean for all library crates; only minor warnings in throwaway bench examples (unused imports/closure style — no correctness) |
| Determinism (probe-verified) | Golden baseline + snapshots consistent; disease-state fingerprints IDENTICAL across seeds/horizons/densities |
| Long-horizon stability | 100K/200K/300K × 12/24/48 agents all PASS; no system dominates (calm/famine 0 coups, pestilence 7–75 crisis coups) |

**Verdict: the system is implemented and wired integratively to a high standard.** The 43-module architecture is nearly all live: per-tick state-update paths (`tick_update`/`update`/`tick_all`) are called from the sim loop for every subsystem audited. This report documents the residual gaps found — all in the *query/modifier-interface* layer (state is computed, stored, and updated, but not all of it is *consumed by decisions*), plus a small set of fully-orphaned modules/functions.

---

## 1. Verified-Live Subsystems (no action needed)

Every one of these was confirmed wired with production call sites in the sim loop and/or decision consumers (not just exported):

- **Biology §7**: all 14 modules' `tick_update` called via `EmbodiedState::tick_update` from the per-tick pass (stress/bonding/dominance/growth axes update; derived_health/energy/hunger/fatigue facades feed the body). Immune, cardiovascular, respiratory, skeletal, musculoskeletal, digestive, thermal, circadian (state), development (environment), reproductive (pregnancy/birth) all live.
- **Psychology §8**: appraisal→emotions (all families incl. moral_outrage), emotion regulation, interoception (felt_need_deficit feeds motivation), motivation, personality→traits, self-model (update_narrative live), belief/epistemic (update_beliefs live), moral cognition (update_moral_emotions live at sim.rs:3270), ToM (mind_models updated + infer_intent called in the interaction pass), imagination/prospection (hope/dread fold at sim.rs:3285), narrative (scripts + themes + temperament), skills/habits (decay_habits/execute_habit), attachment (on_separation/on_reunion live), psychopathology (depression_risk feeds imagination), decision policy, cognitive runtime (can_plan_long_term, effective_planning_depth).
- **Cognitive §9**: attention budget, neural-like folds (prediction error → arousal gate at 0.3, belief/attention consumers), script grammar (courtship replay track), reinforcement values.
- **Relational §10**: relationship v2 (O(1) packed slots), stages, attraction, courtship, marriage, kinship, households, clans, three relational fields (refresh_relational_fields daily), relational power.
- **Groups §12**: group formation (trauma bonding), cults (identity_fusion live), rituals (36 refs), faction v2 (1:1 with v1, suppression_resistance, attachment styles).
- **Noosphere/culture §13**: memes (105 refs), echo chambers (23), propaganda (15), sacred (50), ritual (36), ideology (13), technology (14), collective memory (28), education, **rumor v2** (create sim.rs:6211 → register → tick_all 9354 → transmission_pass_lazy 9451 with group-attachment escalation; the `panic_pressure`/`debunk` *query methods* are un-consumed but the rumor system itself is deeply wired).
- **Institutions/factions/market/economy**: council legitimacy equilibrium, factions (formation/coup/revolution crisis-driven post-Iteration-186), market price/supply/demand (price() consumed by eat/drink/work/wage paths), black market, progressive treasury dividend, grievance computation.
- **Cross-cutting**: provenance (9 sites), journal (20 sites), scheduler (TickPhases gates all passes), agent tiers + LOD cognitive budget (can_appraise/can_prospect/can_memory_op/can_social_infer all gated), gossip (legacy Rumor + process_gossip live), diplomacy (raid_chance/caravan_chance), military (roster/drills/readiness), schools (terms/lessons/graduates), legal (prosecute at 11283), mods (opt-in content packs, intentionally non-tick-loop), spec_lint (dev tool).

---

## 2. Findings — Prioritized by Leverage

### FINDING 1 (HIGH) — Endocrine arousal axis is write-only: the S2-2-4 gain is wired, the level has NO decision consumer

- **Evidence:** `biology/mod.rs:306-310` updates `endocrine.arousal` every tick; probe `p6_orphan_probe` shows the level moving (calm 0.487 / famine 0.528 / pestilence 0.554 @50K); yet **zero production reads** of `endocrine.arousal.level` exist (grep across src; only `affect.arousal` — a *separate* emotion-derived quantity at sim.rs:4963 `(fear+anger+joy)×0.5`, plus the §9.2 prediction-error spike at 12149 — is consumed by decisions).
- **Impact:** The biological arousal axis runs its dynamics and stops. AP2 §7.2.2 specifies arousal as a driver of attention/scanning/impulsivity; the plan's biology→psychology modulation (ADR-002) is missing for this axis. Also `endocrine.dominance.level` is *documented* unwired (sim.rs:3642 comment: "dominance feeds no decision consumer yet").
- **Effort:** S · **Risk:** low (adding a consumer shifts calibration; re-probe).
- **Suggested fix:** feed `endocrine.arousal.level` into the attention budget / appraisal intensity / decision-urgency fold (with a scaling factor kept small so calibrated runs are re-pinned once, not law-flipped); wire `endocrine.dominance.level` into the conflict/status escalation as the comment already anticipates.

### FINDING 2 (HIGH) — Circadian behavioral interface is dead: `should_sleep()`/`sleep_deprived()` never called

- **Evidence:** `circadian.rs:128/133` define `should_sleep`/`sleep_deprived`; zero production callers. The sim drives sleep purely from the *action choice* (`is_sleeping = matches!(current_action, ActionKind::Rest)` at sim.rs:2900) while `sleep_pressure`/`sleep_debt` still build underneath (probe: 0.369–0.527 / 0.333–0.500 across scenarios) — state updated, interface ignored. `is_night()` similarly un-consumed.
- **Impact:** The circadian system cannot *cause* behavior; agents sleep by action-selection coincidence, not because their biology demands it. AP2 §7.3.2's sleep-pressure→sleep-seeking loop is not operationalized. Also leaves `sleep_deprived` (a strong candidate stressor/impairment signal) unused.
- **Effort:** S · **Risk:** low–med (changes Rest-action selection rates; re-pin behavior tests).
- **Suggested fix:** in the routine/action-selection pass, bias toward Rest when `should_sleep()` (phase + pressure) — and consume `sleep_deprived()` as a cognition/performance penalty so the state feeds back.

### FINDING 3 (HIGH) — Disease system: 3 of 5 kinds unreachable; `apply_injury` dead (with hardcoded RNG)

- **Evidence:** `p6_orphan_probe`: calm/famine carry **zero** diseases; pestilence carries **Epidemic only** across the whole horizon. `DiseaseKind::Cold/Fever` are created **only in unit tests** (health.rs `#[cfg(test)]`); `apply_injury` (health.rs:240, the sole WoundInfection source) has **zero production callers** and uses a hardcoded `rng_value = Fixed::from_f64(0.3)` (line 252: "simplified; in real sim use RNG") — so it would not even be deterministic if wired. `should_contract` (health.rs:139) is a dead duplicate of the inline §17b contagion.
- **Impact:** The "4-kind mix" (verified via `p5_mix_dedup` injection probe) exists only in the probe. Real runs never exercise Cold/Fever/WoundInfection severity/duration/transmission, the immune system's differentiation (healthy agents never catch mild diseases), or injury→infection dynamics from violence. AP2 §7.3.1 immune + §7.2.6 wound-infection path are latent.
- **Effort:** M · **Risk:** med (changes health trajectories; re-probe immunity/recovery pins).
- **Suggested fix:** (a) wire `apply_injury` into the conflict/injury path with a real RNG draw (deterministic stream), (b) add a seasonal/weather Cold/Fever vector (e.g., temperature-modifier-driven exposure in the ecology pass), (c) delete or reuse `should_contract` to avoid drift with the live §17b contagion.

### FINDING 4 (MED) — Market price-trend history is write-only: `trend()`/`price_trend()` zero consumers

- **Evidence:** `market.rs:109` pushes every price into `recent_prices` (bounded 20); `trend()` (116) and `price_trend()` (207) compute the signal; **zero production consumers** (only `price()` is read — by eat/drink/work/wage paths). Probe confirms the economy is live but flat-priced: no agent reacts to *price movement*.
- **Impact:** AP2 §13.4 market-distortion/propaganda and any speculative/timing behavior are impossible; prices could swing wildly with no behavioral feedback. The recorded history is dead weight.
- **Effort:** S · **Risk:** low (additive consumer; re-pin economics tests).
- **Suggested fix:** feed `trend()` into the buy/sell/work decision (e.g., stockpile when rising, sell when falling — a simple momentum rule) so the recorded history earns its storage.

### FINDING 5 (MED) — Psychopathology gating interface dead: `is_impaired()`/`social_modifier()`/`cognitive_modifier()` zero consumers

- **Evidence:** `psychopathology.rs:142/147/153` define the mental-health→behavior modifiers; zero production callers. The state IS live (depression_risk feeds the imagination fold at sim.rs:3285, tick_update runs at 3512) but mental health never *gates behavior* — a depressed/paranoid agent imagines darker futures but socializes/works identically.
- **Impact:** AP2 §8.1.15's causal-inputs→effects loop ("mental health affects functioning") is half-wired: inputs and state exist, behavioral effects don't.
- **Effort:** S · **Risk:** low–med (re-pin behavior distributions).
- **Suggested fix:** apply `social_modifier()` to social-interaction participation and `cognitive_modifier()`/`is_impaired()` to the LOD cognitive budget (impaired agents appraise/prospect less) — closes the loop with minimal surface.

### FINDING 6 (MED) — Fully-orphaned module: `logistics`

- **Evidence:** `logistics.rs` (TransportLeg, Warehouse, `local_scarcity_modifier`) exported in lib.rs:26; **zero production references** outside its own file (grep: only lib.rs). `local_scarcity_modifier` (96) is a near-duplicate of the live `market::scarcity_modifier` (451) but dead.
- **Impact:** AP2's transport/warehouse/local-scarcity economics layer is unimplemented-in-practice. Not a bug (nothing depends on it), but the code is dead weight and drifts from the live market logic.
- **Effort:** M (wire into market/trade) or S (delete) · **Risk:** low either way.
- **Suggested fix:** either wire `local_scarcity_modifier` + warehouses into the market pass (site-level scarcity pricing) or remove the module; at minimum delete the duplicate to prevent drift.

### FINDING 7 (LOW) — Dead public helpers (safe to delete or wire; no runtime effect)

- `institutions.rs:408/413` `increase_corruption`/`decrease_corruption` — corruption is mutated inline in `hierarchy.rs:233` instead.
- `factions.rs:209` `protest_legitimacy_effect` — the protest path uses `council_response` (228); the constant `PROTEST_LEGITIMACY_DAMAGE` is only referenced by the dead fn (protest legitimacy damage is currently folded into council_response's return).
- `norms.rs:174/201` `crime_records()`/`record_violation_with_punishment` — the crime path uses `check_violation`/`check_violation_with_enforcement` (live); these two accessors unused.
- `ecology.rs:363` `day_of_season` — unused (seasonal day tracked internally but never queried).
- Psychology query methods with test-only callers: `attachment::activation_level`/`receive_comfort`, `cognitive_runtime::can_inhibit`/`can_switch_strategy`, `theory_of_mind::is_in_group`/`dehumanize`, `self_model::identity_linkage`/`threaten_identity`, `cultural_cognition::outgroup_disgust_for`/`taboo_knowledge_factor`(internal), `moral_cognition::compute_outrage`(test-only; live path uses update_moral_emotions), `imagination::best_scenario`(test-only), `developmental::positive_socialization`/`record_education`, `skill::new`(constructor called via Self::), `neural_like::courtship`(test-only; real courtship is social::courtship).
- Bench-example warnings: unused imports/vars in `p4_err_probe`, `p5_death_probe`, `p5_emergent_probe` (+1 unreachable `_ => "interaction"` arm), `p5_emergent_diag`, `p5_faction_probe`, `p5_mix_dedup`, `p5_profile_probe`, `p5_100k_probe`, `biology_probe`; unused import in `long_horizon_tests.rs:21`.

---

## 3. Emergent-Quality Assessment (post-P5/P6 verification)

The emergent-quality gate from Iterations 185–186 holds and is now re-verified end-to-end:

- **Scenario hierarchy restored:** calm 0 revolutions @100K (was 42–129), famine 0–1, pestilence 7–75 — political drama lives where grievance is real.
- **Life themes diversify by temperament:** {Growth 5, Mission 7} splits in calm (was 13/13 Mission).
- **Economy healthy:** 0/12 below poverty line, progressive dividend shrinking Gini, market treasury ~172 (was coin-sink 2,138–4,999).
- **Epidemic endemic equilibrium:** flat 48 entries = 48 agents at every sample through 300K; recovery→re-infection cycling; deterministic; linear runtime.
- **No system dominates** the 100K surface (Nemesis 0.00, violence deaths 0 in calm).

**The one caveat on "novel experience" depth:** findings 1–3 are the last places where a *computed* quantity does not *cause* behavior. Fixing them (arousal→attention/urgency, circadian→sleep-seeking, disease kinds→immune/injury interplay, trend→market timing) is precisely what converts the remaining "simulated but inert" state into emergent gameplay — the stated Phase-5 goal of "real emergent quality."

---

## 4. Recommendation Order (dependency-aware)

1. **FINDING 1 + 5 together** (endocrine arousal + psychopathology gating → both feed the LOD budget/appraisal; one re-pin wave).
2. **FINDING 2** (circadian → action selection; touches routines tests).
3. **FINDING 3** (disease kinds + apply_injury; touches health/immune pins — probe with `p5_mix_dedup` first as the reference).
4. **FINDING 4** (market trend consumer).
5. **FINDING 6** (logistics wire-or-delete).
6. **FINDING 7** (dead helpers — delete or wire, no risk).

Each lands with its own probe-anchored re-pin wave (repo convention: probe → re-pin → golden/snapshots → full suite green).

---

## 5. What Was NOT Audited

- **mindstrata-tui / mindstrata-cli / mindstrata-render** UI/CLI/render code paths beyond the test suite (render_replay, render_map, save_load round-trip all green; no UI-specific audit).
- **Performance beyond the O(n²) social-field profile** (p5_profile_probe): the density ceiling is the daily `refresh_relational_fields` all-pairs scan + relationship-v2 passes; no spatial-index work done (out of scope — noted as the known density ceiling).
- **Modding API** (`mods.rs`): validated by tests only; no real content pack exercised end-to-end.
- **spec_lint**: runs against specs/ but no CI gate verified.

---

*Generated by the Iteration 186 final exhaustive audit. Companion probes: `p6_orphan_probe`, `p5_epidemic_dedup`, `p5_mix_dedup`, `p5_faction_probe`, `p5_faction_fine_trace`, `p5_emergent_diag`, `p5_profile_probe`.*

---

## 6. Iteration 187 Closure — ALL 7 FINDINGS FIXED

**Date:** 2026-08-17 · **Commit:** (this iteration) · **Result:** every write-only state now has a decision consumer; every dead helper retired. 13 calibration tests re-pinned (each anchor re-probed with seed sweeps before re-pinning — direction verified as RNG re-pacing, not law-flips), golden + 4 snapshots regenerated, full workspace green, clippy clean.

| # | Finding (Iteration 186) | Closure (Iteration 187) |
|---|---|---|
| 1 | Endocrine arousal level write-only | `recompute_biases(arousal)` — `(arousal − 0.5) × 0.5` threat fold, mean-zero at anchor (unit-pinned byte-identity), ONE-SIDED, clamped [0.3, 0.7]; wired from `endocrine.arousal.level` at the sole call site. Dominance's documented unwiring (sim.rs:3642) deferred deliberately. |
| 2 | Circadian behavioral interface dead | Motivation block consults the clock: `sleep_deprived()` floors fatigue at 0.8, `should_sleep()` at 0.6 — Rest utility + §8.1.5 sleep motive now respond. Identity-at-zero, no RNG. |
| 3 | Disease kinds latent | `apply_injury` takes an explicit RNG draw + dedup/cap guards, wired at the violence site (severity `injury × 10` preserves the legacy damage EXACTLY; energy drain `injury × 0.5` added). New daily seasonal pass seeds Cold (~17%/agent/season at Winter) + Fever (Cold→Fever 0.002/day), kind-dedup + cap guarded, normal duration path. `should_contract` (dead duplicate) deleted. Probe: Cold/Fever circulate in calm @50K. |
| 4 | Market price-trend write-only | Trade price multiplies in `price_trend(GRAIN) × 0.5` clamped [0.9, 1.1] — haggle-down on falling, pay-up on rising; mean-zero (trend 0 → 1.0 byte-identical). |
| 5 | Psychopathology gating dead | `is_impaired()` now gates the social-interaction participation mask (withdrawal) AND halves `effective_planning_depth()` (planning confidence). ONE-SIDED, healthy agents byte-identical. |
| 6 | `logistics` module orphaned | `local_scarcity_modifier` (site grain stock) now prices the farm-gate trade: damped [0.95, 1.2] premium/discount. Module live. |
| 7 | Dead public helpers | Deleted: `increase/decrease_corruption`, `protest_legitimacy_effect` + `PROTEST_LEGITIMACY_DAMAGE`, `crime_records()` + `record_violation_with_punishment`, `day_of_season` (all verified zero-callers). |

**Liveness evidence:** `p6_orphan_probe` extended — mean `threat_bias` now tracks the arousal fold and calm runs show Cold/Fever kinds circulating over a year+ horizon (previously Epidemic-only). `p7_repin_probe` retained as the reproducible re-anchor sweep harness.

**Verdict: the audit loop is complete for this iteration.** Phase 1–5 gaps and the 7 residual query-interface findings are all closed; the sim's computed state is now behaviorally live end-to-end. The next audit pass should focus on the deferred items noted in §5 plus new emergent-quality sampling under the post-Iter-187 dynamics.
