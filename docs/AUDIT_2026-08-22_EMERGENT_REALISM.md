# Emergent-Realism Audit — Iteration 239

**Date:** 2026-08-22
**Method:** Static sweep of every seeding/birth/world-gen/emotion site + empirical probes on the release binary (5-seed × 20K metric exports, 100K-tick single-seed trajectory, agent psychology inspectors, targeted example probes) + full-suite verification runs.
**Scope:** anything that hardcodes outcomes, injects false history, breaks heredity, or pins dynamics in ways that defeat the "emergent from first principles" design contract.

---

## 0. Operational state discovered by this audit

| Item | Finding |
|---|---|
| HEAD (`a5c8175`, Iter 236) | **24 integration tests failing** — last committed batch shipped without a full-suite run. |
| Uncommitted tree | Held partial fixes for those failures (legitimate near-zero re-pins after Iter-235 made jealousy/envy/contempt/despair producers live; drought-window mechanics) **plus** an Iter-237 genome-derived biology change that killed the calibrated birth pipeline outright (0 conceptions on seed 5 @220K). Recovered: re-pins + drought fix kept; biology reverted to HEAD baselines. |
| Suite after recovery + thermal fix | See §4 for the remaining red set and its root causes. |

**Standing rule violated repeatedly:** commits landed without `cargo test --workspace`. The iteration log itself documents the consequence — calibration windows re-anchored seed-by-seed (185→187→…) because every behavioral iteration re-paces shared RNG streams and flips knife-edge outcomes 0↔N. That treadmill is a structural finding (§3.7), not bad luck.

---

## 1. Confirmed code bugs (fixed this session)

### 1.1 Thermal runaway ratchet (Iteration 239 fix)
`biology/thermal.rs::tick_update`

Three stacked defects, all invisible until probed end-to-end:

1. **Ratchet (introduced Iteration 222).** The restoring term `(ambient − body)` was replaced by `(ambient − set_point)` — a quantity that never vanishes when body == ambient. Body temperature performed a biased random walk and pinned at the 1.0 clamp instead of tracking seasons. Probe: body **0.769** vs ambient **0.535** after 4320 Spring ticks.
2. **Quantization stall.** The 0.001 convergence rate times any sub-0.05 delta rounds to zero in 4-decimal Fixed — convergence stalled ~0.05 short of ambient, and the genome-derived set-point offset (±0.025) was a complete **no-op** in dynamics. Iter-222's headline feature did nothing.
3. **Inverted insulation sign (inherited from the pre-Iter-45 inline block).** `- metabolic_warmth × cold_stress` cooled *well-fed* agents faster while two generations of comments claimed the opposite ("slowed by metabolic regulation", "well-fed resists cold"). Dead below quantum for its entire life; the f64 core exposed it.

**Fix:** f64 integrator shadow field (lazy-seeded, snapshot-safe via `#[serde(default)]`) accumulating sub-quantum rates at full precision; convergence restored on one shared `(felt_ambient − body)` delta; set-point implemented as an experienced-temperature shift (±0.025 equilibrium variation across genomes, stress still responsive within one tick); metabolic term promoted to live insulation with corrected sign. New regression pin `body_converges_to_ambient_not_clamp`; metabolic-inertness pin replaced by the intended live contract.

### 1.2 Drought/flood recharge semantics (recovered from uncommitted Iter-238)
- Flood-mode recharge is suppressed inside the drought window (a drought desiccates the aquifer; floods cannot refill past the drain).
- Aquifer recharge applies to Wells only (a farm's water staying exactly stable absent production is the realism contract).
- `from_scenario` pre-sets `drought_until` so suppression holds from tick 0.
- Test threshold re-anchored to post-Iter-228 well capacity (2000).

### 1.3 Stale zero-blast assertions (recovered re-pins)
Iteration 235 deliberately made jealousy/envy/contempt/despair producers live in calm worlds (near-zero, not exactly-zero). Four tests still demanded exact zeros; re-pinned to `< 0.05` / `< 0.15` bands with updated rationale comments.

---

## 2. Hardcoded unrealistic implementations (open findings)

Ordered by damage to the emergent-realism contract.

### H1 — Zero heredity at birth *(deepest defect)*
Both birth paths (`build_replacement_newborn`, `tick_birth_mechanics`) construct children with:
- `Personality::random(...)` — fresh RNG, **zero parental inheritance** (the comment literally says "Inherit personality traits with noise"; no inheritance exists),
- `EmbodiedState::random(...)` — brand-new random genome; the Genome system's documented heritable predispositions are never transmitted → **evolution is structurally impossible**, family resemblance is zero,
- `MoralValues::random(...)`,
- ideology hardcoded at position 0.5 / conviction 0.5 on **both axes** (founding agents draw 0.2–0.8) — generational turnover replaces a differentiated population with centrist clones,
- replacement newborns additionally record `parent_a: None, parent_b: None` — the kinship graph does not know them.

Consequence: the sim's generational layer (the thing that would make long runs produce lineages, dynasties, selection pressure) is a façade. This is the largest gap between the design claim ("history emerges") and the machine.

### H2 — Tick-0 scripted culture (false history)
Every world starts with the same hand-written cultural DNA regardless of scenario or geography:
- 5 founding memes with fixed charges, including the political grievance *"The council is hoarding the well's water"* and doom prophecy *"A famine is coming"* — present even in worlds where nothing of the kind ever happened;
- collective memory pre-loaded with *"The last great drought nearly starved the village"* — **an invented shared trauma** in worlds without droughts;
- 2 propaganda campaigns (*"The council protects the well"*, *"Honor the spirits for rain"*) and 2 rituals, with congregations hard-split into pro/anti clusters by `traditionalism + agreeableness > 1.0`;
- the noosphere is seeded to mirror the fixed meme list.

Emergence cannot produce alternative histories when every run begins with the same mythology, grievances, and institutions' PR.

### H3 — Template world
`world_gen::generate_village` produces the identical Riverford every seed: straight river down the center column, 8 houses on a perfect circle of radius 4, farm west / well south / market east / temple north, forest ring border. Only the 5% random water-tile scatter varies. Combined with H2, **seed variance comes almost entirely from agent RNG draws** — geography and culture contribute nothing.

### H4 — Exactly 2 clans forever
`CLAN_COUNT = 2`, membership partitioned by home-site parity. Clan count can never respond to population size, geography, or feuds splitting groups.

### H5 — Population-shape unrealism
- All 12 personality traits ~ U(0,1): extreme personalities are massively oversampled vs real approximately-normal, mildly-correlated trait distributions.
- Wealth uniform 5..20 at founding; newborns flat 1.0 coin.
- MAX_POPULATION = 48 hard cap; population freezes at the cap with no overshoot/collapse dynamics.
- 24-name pool cycles above 24 agents (guaranteed duplicate names); children remain `Child_{idx}` for life.

### H6 — Real-birth path missing regulation init (small, mechanical)
The P3-4 personality-driven emotion-regulation initialization was applied to the initial population and `build_replacement_newborn`, but **not** to the real birth path in `tick_birth_mechanics` (still `EmotionRegulationState::default()` → permanent Reappraisal bias in death-heavy worlds — the exact failure mode P3-4 documents).

---

## 3. Empirical pathology (live binary probes)

### E1 — Chronic dysphoria plateau
Across seeds 1/42/99 @20K and seed 46 @100K (calm worlds): mean valence ≈ **−0.25…−0.30 permanently**, fear ≈ 0.28–0.36, stress rising 0.22→0.37. A peaceful village is stuck at a negative hedonic equilibrium for years.
Inspector snapshots show the mechanism's shape: valence/fear sit on **bimodal plateaus** (−0.96 / +0.94 / ~0.00) rather than graded distributions — the affect system has two attractors (neutral-calm and maxed-negative), not continuous graded states.

### E2 — Violence without stressors
5 failed-threat → violence escalations within 3000 ticks of a calm seed-42 world. Chronic assault absent famine/drought undermines the causal story violence is supposed to tell.

### E3 — Survival-needs inversion
Agents display thirst = 1.00, fatigue = 1.00, HP = 0.00 while executing Trade actions. At least for some agents the utility AI does not prioritize lethal-need resolution over economic activity — either the displayed state isn't what drives decisions or the utility weights are wrong at the extremes.

### E4 — Reproduction knife-edge
First conception lands at tick ~205K (~4 years) on the probe seed, and the calibrated windows break with every behavioral iteration (documented re-anchor chain 185→187→now). The pipeline works only inside razor-thin RNG corridors.

### E5 — Faction formation dead on anchored seeds
Pestilence seed 5 @30K: council legitimacy hovers ≈0.53 (above the < 0.5 formation gate) with **zero revolutions** — no crisis materializes, so no faction ever forms; `faction_counts_vary_across_worlds` reads {0} everywhere. Crisis scenarios no longer reliably produce crises.

### E6 — Monotonic wealth concentration
Gini climbs 0.15 → 0.67 over 20K ticks across all seeds — inequality accumulates mechanically (tax/fine drain vs trade flow) with no counter-acting forces (inheritance splits, charity, redistribution norms are absent or inert).

### E7 — Belief erosion without reinforcement loops
Foundational beliefs decay toward zero confidence (0.06–0.30) in ordinary runs — belief maintenance (community reinforcement, institutional repetition) doesn't keep pace with decay, so the epistemic layer empties out.

### E8 — Skill saturation
Farming pins at 1.00 population-wide while Trading sits at 0.00 — skill curves lack diminishing returns / decay pressure at the top and sufficient gradient at the bottom.

---

## 4. Remaining red suite (post-fix state, root-caused)

**Iteration 240 tally: 290 passed / 14 failed** (was 287/17 after Iter-239).
Healed this iteration: `factions_emerge_from_grievance`, `faction_v2_fighting_strength_links_to_protests`,
`faction_attachment_styles_scale_upward_and_dynamics_run`, `faction_counts_vary_across_worlds`
(crisis-pressure accumulator + recruitment wiring + entrenchment window; see git log).
Newly exposed (downstream re-pacing casualties of restored faction liveness):
`collective_fear_amplifies_panic_legitimacy_damage_end_to_end`, `taboo_shame_amplification_is_live_and_one_sided`,
`reproduction_conception_multiplier_parameter_is_live`.

All are consequences of the knife-edge architecture (§E4/E5) plus assertion staleness; each needs a root-first fix, not another lucky-seed re-pin.

| Cluster | Tests | Root cause |
|---|---|---|
| Moral panic | `moral_panic_lifecycle_registers_and_drains…`, `collective_fear_amplifies_panic…`, `long_horizon_50k…` (panic leg) | Panic trigger/lifetime drift — panics stopped firing or drain too fast in anchored windows. |
| Pairing pipeline | `marriage_forges_spouse_and_inlaw_kinship`, `kinship_penalty_rises_when_families_form`, `conception_pregnancy_birth_pipeline_runs_and_is_seed_deterministic`, `reproduction_conception_multiplier_parameter_is_live` | Formation rolls zeroed by downstream state shifts (feud-driven clan enmity between engineered pairs; water-economy shifts moving conception odds); windows are single-seed fragile. |
| Single-system drift | `collapse_famine_timing…`, `institutional_rank_weighted_into_effective_status`, `memory_system_produces_plan_taxonomy…`, `motivation_emotional_context_is_live`, `neural_like_prediction_error…`, `noospheric_belief_confidence_sustains_conviction`, `taboo_shame_amplification…` | Margins collapsed to ties or inverted under accumulated re-pacing (e.g., patronage 7 vs 7; flashbulb gate never fires; escalation-taboo comparison flipped). |

---

## 5. Verified healthy (audit controls)

Determinism byte-identical replays; **no mid-sim scripted events** (the §2.2 anti-pattern genuinely absent — no `if unrest > X spawn_revolt()` anywhere); multi-pathway mortality (Gompertz + disease + starvation + health-collapse + stress) live and directional; weather regimes (drought/flood emergence) functional; market trades actively (~15K trades / 100K ticks); spec_lint validates RON specs; provenance traces populate.

---

## 6. Iteration plan — realism first, then UX surface

Ordering principle: each phase must leave the workspace green and golden-determinism intact (long-horizon snapshots re-pinned with evidence, short-horizon windows untouched wherever possible). No phase starts until the previous one is committed green.

### Phase 0 — Suite-green restoration + kill the re-pin treadmill *(unblocks everything)*
1. Replace single-shot gates with **hazard accumulation**: sustained affinity/crisis builds a deterministic pressure term; the stochastic roll consumes accumulated pressure. Marriage, courtship, faction formation, cult emergence. This removes the 0↔N knife-edge that every behavioral iteration currently trips.
2. Faction gate: legitimacy threshold gains hysteresis + grievance-pressure coupling so crises reliably produce organization without a legitimacy cliff.
3. Re-anchor remaining tests onto statistical liveness contracts (≥3 seeds, direction + magnitude bands) instead of exact ticks/seeds.
4. Enforce full-suite green before every commit (local gate script; CI already exists).

### Phase 1 — Real heredity *(the deepest realism unlock)*
1. Genome blending at conception (parent crossover + small mutation).
2. Personality inheritance: mid-parent value + variance shrinkage + noise (quantitative-genetics shape).
3. Moral values and ideology transmitted from parents/community priors, not re-rolled or cloned at 0.5.
4. Replacement newborns record parent links; unify regulation-init across both birth paths (closes H6).
5. Naming: children inherit family names; expand name pools; uniqueness enforcement.
Blast control: births are rare inside calibrated horizons → golden stays byte-identical; long-horizon surfaces re-pinned once with lineage evidence.

### Phase 2 — Affect realism *(kills the dysphoria plateau)*
1. Root-cause the −0.96/+0.94 plateaus (decay floors/clamp asymmetry) and restore graded decay.
2. Rebalance appraisal baseline so calm-world equilibrium is mildly positive (hedonic adaptation: sustained positive conditions shift the set point up).
3. Fear tracks realized threat exposure, not ambient constant.
4. Verify calm-world violence approaches zero through the existing trust/norm channels (they exist — E2 says they're not being reached).

### Phase 3 — Survival-integrity reflexes
Physiological override layer beneath utility AI: thirst/hunger above critical thresholds force drink/eat before economic/social actions (the E3 inversion becomes impossible); health-critical agents restrict to rest/recovery. Small, testable, huge believability gain.

### Phase 4 — De-script the initial state
1. Founding culture generated from world/scenario context (water-stressed worlds grow water myths; stable ones don't start with grievance memes). Memes/rituals/memory drawn from RNG + world parameters, registered as data-driven RON content with spec_lint coverage.
2. Collective memory starts with only a founding myth derived from actual world gen; traumas must be earned by events.
3. Rituals/propaganda emerge after institution formation ticks, not pre-seeded.
4. Clan count derived from settlement clustering; variable per seed.

### Phase 5 — World variance
Parametric village generation: meandering river course, site-layout grammar with jittered placement, terrain-quality fields, resource-richness multipliers, village scale tied to population cap raise (48 is a ceiling that freezes demography; lift with LOD tiers already in place). Every seed becomes a different village worth reading about.

### Phase 6 — Observability UX *(make the emergence legible — the payoff layer)*
1. Village chronicle: auto-generated annals from collective memory + provenance traces ("Year 2: the harvest failed; the Council's legitimacy cracked; Ana's faction marched"). The emergent-history product made human-readable.
2. Agent dossiers with lineage trees (unlocked by Phase 1): inherited vs expressed traits, life-event timeline, relationship web.
3. TUI longitudinal charts from the metrics CSV that already exists (population, stress, wealth/Gini, belief ecology).
4. Event causality inspector: select any event → full decision trace (provenance system already records this).

Phases 1–3 are the exponential-believability lever; Phase 6 converts it into player-visible UX. Phases 4–5 multiply replay variety so chronicles differ run to run.
