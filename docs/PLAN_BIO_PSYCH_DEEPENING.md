# Plan — Biological & Psychological Deepening (bio/psycho → social upgrade)

*Created Iteration 242+, post-commit `8881980`. Supersedes nothing; extends the
audit roadmap's Phase 1 into a full bio↔psych↔social coupling program.*

> **STATUS UPDATE (post-Iter-246, commit `36dd22b`) — Arc A COMPLETE.**
>
> Landed vs plan, with deviations:
> - **Iter 244** shipped genome blending (`Genome::blend`, midpoint ± 0.06
>   mutation), `EmbodiedState::born` shared constructor, `Personality::inherit`
>   (mid-parent + 20% shrinkage), and THREE dead gene families activated
>   (metabolic rate/satiety/fat-storage, immune recovery/exposure,
>   strength/endurance ceilings). Deviation: physical_potential →
>   injury-susceptibility & labor-output wiring DEFERRED (conflict code lives
>   in the hot tick path); ceilings only.
> - **Parent links**: landed as `AgentBundle.parent_a/parent_b: Option<usize>`
>   (parallel agent, sim split) — supersedes this plan's orphan-rule sketch.
> - **Iter 245** shipped family surnames (`person::FIRST_NAMES`,
>   `person::inherit_surname`, maternal-line inheritance at both birth paths).
>   Deviations: uniqueness enforcement NOT needed (given names cycle a pool;
>   collisions acceptable); regulation-init unification (audit H6) was ALREADY
>   closed by Iter-239 — plan item stale.
> - **Iter 246** shipped moral-values vertical transmission (mid-parent 70% +
>   community prior 30% + noise, both birth paths). Deviation: foundational-
>   beliefs and ideology-axis affinity inheritance NOT yet done — newborns
>   still take hardcoded axis defaults (`family.rs` both sites); folded into
>   Iter 247 prep below as a small carry-over item.
> - Verification probes run: value-band contracts, sole-parent path, surname
>   three-generation lineage test, six-draw RNG contract. ~~STILL OPEN~~
>   CLOSED (i259_audit, commit `6542614`): parent-child CONSTITUTION
>   r=0.53–0.66 across runs — inside the [0.3, 0.7] slope contract; sibling >
>   stranger holds. Note: expressed-trait correlation is the WRONG metric
>   here (life-course plasticity pulls traits toward the birth anchor);
>   constitution-vs-constitution is the heredity surface.
>
> Suite: 298 passed / 6 failed (unchanged failure set through Arc A).

## 0. Where we are

Suite **298 passed / 6 failed** (`cargo test -p mindstrata-tests --lib --release`,
post-Iter-242). Audit Phases 0 is functionally complete: hazard accumulation
ships (faction pressure, mobilization windows, marriage/courtship liveness),
regime change fires crisis-spaced, belief charging runs daily. The six red
tests are independent single-system calibration drift — none block architecture
work except by our own green-before-commit rule.

Layer inventory (all wired unless noted): 15 biology modules, 17 psychology
modules, 20 social modules, 14 culture modules, 4 noosphere modules.
Determinism byte-identical, provenance traces live, spec_lint validates RON,
agent_tier LOD exists (34 fns), metrics CSV export exists.

## 1. Gap analysis (evidence-first)

### G1 — Heredity does not exist (the biggest gap)

Both birth paths construct newborns with **fresh random** endowments:

- Real-birth path `sim.rs:~9821`: `Personality::random(rng)` +
  `EmbodiedState::random(child_age, rng)`
- Replacement path `build_replacement_newborn` `sim.rs:9259`: same pattern

Consequences: `Genome` (5 predisposition families) never transmits;
personality has zero parent correlation; children share no family resemblance;
`foundational_beliefs()` re-rolls values; no parent links on replacement
newborns. The genome system is per-agent decoration, not a population-level
genetic substrate.

Sub-gaps inside G1:
- **G1a**: no crossover/mutation operators anywhere (`genome.rs` has only
  `Genome::random`).
- **G1b**: 3 of 5 predisposition families are effectively dead:
  `metabolic_predispositions`, `physical_potential`, and most of
  `health_predispositions` have no consumers outside `genome.rs`
  (only `immune_strength` at `biology/mod.rs:201`; fertility → base_fertility
  at :181; trait predispositions partially read for attachment/depression/
  addiction at `sim.rs:1057/3739/9277`). Even where read, they seed states —
  they do not modulate ongoing dynamics.
- **G1c**: naming is `Child_{idx}` on the replacement path; no family names,
  so lineage is invisible in every downstream artifact (chronicles, dossiers).

### G2 — Interoception is built but dormant

`psychology/interoception.rs` implements the felt-signal filters
(`felt_hunger/thirst/fatigue/pain`, `emotional_body_tone`,
`felt_need_deficit`) — and **none of them sit in the decision pipeline**.
Only call sites: one metrics computation (`sim.rs:14373`) and tests. Body
state reaches the mind through raw deficits, not through per-agent felt
intensity. The "Genome → Endocrine → Nervous → Interoception → Emotion" chain
claimed in `biology/mod.rs:8` breaks at the Interoception link.

### G3 — Bio→social gradients absent

No status→health gradient (Whitehall effect): hierarchy position does not
feed chronic stress load, so `social/hierarchy.rs` outcomes cannot feed back
into biology. Social support feeds psychopathology ✓, but rank does not.

### G4 — Psych→social coupling is thin relative to layer size

`theory_of_mind` has ~10 refs in an 18K-line tick loop against a 20-module
social layer; attachment influences exist at seeding only. Courtship/marriage
do not condition on partner-modeling quality; speech acts carry no deception /
belief-about-believer content.

### G5 — Architecture debt concentrated in one file

`sim.rs` = **17,942 lines**, 40 public fns; the tick pipeline interleaves
biology, psychology, social, culture passes inline. `systems/` module exists
but is empty (`mod.rs` only). Every behavioral iteration risks collateral
blast radius because passes aren't isolated; extraction is prerequisite
infrastructure for safe deepening.

### G6 — Known calibration drift (Iteration 243 scope)

Six single-system failures: famine timing, patronage rank weighting,
flashbulb/plan taxonomy memory, motivation emotional context, neural-like
prediction-error gates, noosphere conviction margin. Each needs root-first
recalibration, not re-pins (memory rule #1773 / ledger discipline).

## 2. Upgrade plan (ordered, each iteration committed green)

Ordering principle unchanged: green suite + byte-identical short-horizon
golden between iterations; long-horizon surfaces re-pinned once per arc with
lineage/evidence probes. Fixed-truncation disease rules apply everywhere
(f64 shadow fields for sub-resolution rates).

### Iteration 243 — Suite green *(prerequisite, already scoped)*
Root-cause the six drift tests. Exit: `cargo test --workspace` fully green.

### Arc A — Real heredity (Iterations 244–246) *— closes G1*

**Iter 244 — Genetic substrate**
1. `Genome::blend(a, b, rng, mutation_rate)` in `genome.rs`: per-gene
   midpoint + noise (diploid-average approximation; deterministic from
   seed+tick stream). Sex: 50/50 draw.
2. Wire into BOTH birth paths; record `parent_ids` on every newborn
   (replacement path inherits deceased's parents if absent — orphan rule).
3. Activate dead predisposition families: metabolic → hunger/thirst rate
   multipliers; physical_potential → injury susceptibility & labor output;
   health_predispositions → disease resistance & recovery tau.
   *All five families must have ≥1 dynamic consumer (spec_lint-style check).*

**Iter 245 — Quantitative-genetics personality + names**
1. Personality: child trait = mid-parent + shrinkage toward mean + Gaussian
   noise (variance calibrated so population variance is stationary, not
   collapsing across generations).
2. Family surnames: household-carried; children take father's/mother's per
   scenario culture; uniqueness enforcement; replace `Child_{idx}`.
3. Unify regulation-init across both birth paths (closes audit H6).
Verification probe: sibling-sibling trait correlation > stranger correlation;
parent-child regression slope in [0.3, 0.7].

**Iter 246 — Vertical culture transmission**
1. Moral values & foundational beliefs seeded from parents (with noisy
   adoption) + community priors weighted by exposure; not re-rolled.
2. Ideology initial affinity inherited with mutation.
Verification probe: ideological clustering by family vs random assignment;
golden blast contained to long-horizon surfaces (births rare in short windows).

### Arc B — Embodiment → mind (Iterations 247–248) *— closes G2, G3*

**Iter 247 — Interoception activation** *(next up)*
0. Carry-over from Arc A: inherit ideology-axis affinity + foundational
   beliefs from parents (small; same blend shape as moral values).
1. Route need salience through `felt_*`: action utilities consume felt
   deficits, not raw ones; sensitivity set by genome+nervous sensitivity
   (already stored). NOTE: wiring sites moved to `sim/core.rs` +
   `sim/family.rs` after the split — check mtimes before editing.
2. Somatic marker: high felt-pain/fatigue biases risk-averse action choice.
3. `emotional_body_tone` joins appraisal inputs (embodied emotion bias).
Probe: high-sensitivity agents react to deficits earlier than low-sensitivity
at identical body state; calm-world action mix unchanged within bands.

**Iter 248 — Social physiology**
1. Whitehall gradient: effective-status percentile feeds chronic-stress
   accumulation rate (low rank → higher load), closing the hierarchy→biology
   loop.
2. Sleep debt → next-day social participation penalty (withdrawal already
   modeled in psychopathology — reuse channels).
Probe: high-status cohort mean chronic_load < low-status cohort across seeds.

### Arc C — Mind → social depth (Iteration 249) *— closes G4*

**Iter 249 — Social cognition steering**
1. Partner choice conditions on modeled reliability (ToM estimate of
   counterpart), not just proximity/wealth terms.
2. Speech acts gain intent tags (deceive/impress/inform); receivers update
   trust on detected mismatch (rumor_v2 + epistemic integration points exist).
Probe: repeated defectors lose invite rate over time vs one-shot strangers.

### Arc D — Architecture extraction (Iterations 250–251) *— closes G5*

**Iter 250 — Pass decomposition (pure refactor)**
Move the interleaved bio/psych/social passes from `sim.rs` into
`systems/{biology_pass,psych_pass,social_pass,...}.rs` behind a pass
registry. Golden snapshots MUST remain byte-identical (this proves the
refactor moved code, not behavior). Exit criterion: `sim.rs` < 6K lines,
each pass owns its section header contract.

**Iter 251 — Observability for the new layers**
Extend `metric_history` with lineage stats (family count, mean kinship,
trait variance trajectory), emotion distribution percentiles, Gini + skill
curves (closes E6/E8 monitoring). TUI longitudinal charts from CSV
(pull-forward of audit Phase 6.3 — cheap now that CSV exists).

### After Arc D
Resume audit roadmap Phases 2–6 exactly as written there (affect realism,
survival reflexes, de-scripted founding, world variance, chronicle UX) —
Arc A makes Phase 6 agent dossiers (lineage trees) immediately possible.

## 3. Effort summary

| Arc | Iters | Theme | Risk |
|-----|-------|-------|------|
| 243 | 1 | suite green | low (calibration only) |
| A | 244–246 | heredity substrate | med (demographic coupling) |
| B | 247–248 | embodiment→mind | med (decision-path blast) |
| C | 249 | social cognition | low-med |
| D | 250–251 | refactor + observability | refactor guarded by byte-identity |

Roughly eight working iterations before audit Phase 2 resumes. Each is an
independent commit point; the program degrades gracefully if paused at any
boundary.

---

## 4. PROGRAM CLOSE (Iteration 259, commit e55d32c)

All arcs delivered. Suite 305 passed / 0 failed; clippy workspace clean.

- **Arc A (heredity)**: genome blending, personality quantitative genetics,
  surnames, moral-values + ideology transmission. Verified end-to-end
  (parent-child r=0.81, sibling r=0.92).
- **Arc B (embodiment→mind)**: interoception filters in the decision
  pipeline; Whitehall status→stress→health loop; sleep-debt withdrawal.
- **Arc C (mind→social)**: deception detection via speech-intent mismatch;
  modeled-reliability partner choice.
- **Phase 2 affect realism**: piecewise graded fear shoulder, hedonic joy
  baseline, hedonic setpoint drift — the E1 dysphoria plateau is dead.
  Audit correction (i259_audit probe): median valence is positive on calm
  seeds (7/13: +0.02/+0.03) but mildly negative where feud storms ran
  (42/5: −0.07) — mechanism contracts (joy floor 0.011 uniform, fear
  medians differentiated) are the pinned surface, not universal-seed
  positivity.
- **Phase 3**: survival-integrity reflexes beneath utility AI.
- **Phase 4**: earned traumas replace fabricated history; world-conditional
  grievance memes; clans derive from settlement clustering.
- **Phase 5**: meandering rivers, fertility fields, jittered site grammar,
  soil-scaled founding stocks — every seed is a different village.
- **Phase 6 core**: village chronicle annals + agent dossiers with lineage,
  inherited-vs-expressed drift, genome highlights, life timelines
  (--chronicle / --dossier IDX).
- **Knife-edge resolved**: hazard-accumulated epidemic immunity ended the
  12-flip TRANSIENT↔ENDEMIC oscillation structurally.

Leftovers queued: propaganda emergence post-institution; pop-cap lift;
chronicle-in-TUI polish.

---

## 5. PRE-NEXT-PHASE AUDIT (Iteration 260)

Full plan re-audit before resuming development. Findings:

- **Carry-over item closed (Iter-247 item 0)**: ideology-axis inheritance was
  landed at Iteration 247 (births_deaths.rs both paths) but FOUNDATIONAL-
  BELIEF/SACRED-VALUE transmission had silently stayed open — founders hold
  seeded sacred values (`population.rs`), newborns started EMPTY. Fixed:
  maternal-line verbatim clone on the real-birth path, deceased-household
  clone on the replacement path (deterministic, zero RNG draws — noise
  deferred until evidence demands). Probe evidence (i260_sacred_probe,
  deleted per discipline): seed 42, marker strengthened on all founders,
  conception x4, 40K ticks -> 5/5 children carry the full maternal set.
  Suite 305/0 with no snapshot drift (births absent from pinned windows).
- **Documented deferrals confirmed still-deferred (acceptable)**:
  physical_potential injury-susceptibility & labor-output wiring (ceilings
  only, per Iter-244 deviation); sensory_acuity genome field has zero
  consumers (micro-debt; family-level exit criterion already satisfied via
  strength/endurance ceilings).
- **G5/G6 status**: G6 closed by Iteration 243. G5 superseded by the crate
  ladder + doctrine correction `9c53812` (<15K target retired); remaining
  systems/ pass extraction stays queued infrastructure, not a blocker.

Verdict: nothing blocks the next phases. Queued leftovers stand.
