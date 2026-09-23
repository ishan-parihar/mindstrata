# AGENTS.md — mindstrata Development Doctrine

**Read this file first. It encodes how we develop, verify, and commit. Follow it without being reminded.**

---

## 1. What This Project Is

`mindstrata` is a deterministic agent-based village simulation (Rust workspace) where emergent
social phenomena — factions, moral panics, marriages, revolutions, epidemics — arise from
coupled biological, psychological, and social subsystems. It is an R&D instrument, not a game:
every mechanism must be **live** (measurably influences behavior), every calibration must be
**probe-evidenced**, and every iteration must leave the suite **greener than it found it**.

**Where to look (the authority set — see `docs/DOCUMENTATION.md` for the full map).**
Current engine truth: **`docs/ENGINE_STATUS.md`** (authoritative for behaviour, realism and
scale). Work queue: **`docs/PLAN_DC3_DEVELOPMENT.md`**. Trajectory: `docs/ROADMAP.md`.
Specs of record (implemented): `docs/architecture/AP2.md` (village substrate),
`docs/architecture/AP3-afa/` (attractor-field Eras I–V). **Historical only — never use for
current state:** `docs/architecture/archive/AP1-implemented.md`,
`docs/PLAN_BIO_PSYCH_DEEPENING.md` (deepening program, closed i259),
`docs/AUDIT_2026-08-22_EMERGENT_REALISM.md` (realism findings, mostly closed).

## 2. The Iterative R&D Loop

Work proceeds in **iterations**, each one a complete arc ending in a commit + push:

1. **Pick one root cause.** Never batch unrelated fixes; each iteration owns one system.
2. **Probe before touching.** Write a tiny `crates/mindstrata-benches/examples/<iter>_*.rs`
   probe that measures the actual behavior at the actual horizon/seed. Evidence first.
3. **Fix the root, not the symptom.** If a test fails because a *producer* went dead,
   revive the producer — do not re-pin the assertion to accept zero.
4. **Re-run the probe → compare → full suite → fmt/clippy → commit → push.**
5. **Re-audit.** Each fix shifts downstream equilibria; re-run the simulation and hunt for
   new gaps before declaring anything done.

Terse continuation prompts ("continue", "proceed") mean: pick up from the last actionable
state, no re-summarizing, no clarifying questions.

## 3. Verification Discipline (non-negotiable)

- **The gate is `scripts/gate` (quick) / `scripts/gate --full` (before push).** It runs
  fmt, clippy, the bench-naming law (IC-4), the documentation authority map
  (`scripts/doc_index.py`), the warn-only perf budgets, the seed-family sweep, and the
  golden baselines (quick) or the whole release suite (full). The pre-commit hook covers
  only fmt+clippy; the release suite + goldens are enforced at push time. Behavioral
  iterations run `--full`. No exceptions, no reminders.
- **Known gate gap (recorded debt, i349/i351 ledger):** the clippy step does not pass
  `--all-targets`, so test-cfg code in crates other than the one being built can hide
  warnings. Closing it requires clearing the ~14 existing `mindstrata-tests` test-cfg
  warnings first — its own tooling iteration, not a drive-by.
- **Test-population policy (charter, i368): the perf budget is a CEILING, not a test mandate.**
  The DC-3 P0 budget (122/1088/5961 µs/tick @ N=12/48/96) bounds *cost*; it does not
  dictate which N a test runs at. Every test and probe picks **the smallest N that
  exhibits the phenomenon under test** (a gossip pin needs N=12; a settlement/trade pin
  needs N≥144). The default matrix is **{12, 48, 144}** — three canonical tiers (village /
  large village / town edge) — with N=256 reserved as the **town-scale stress tier**
  (envelope headroom measured at i359). Do not raise a test's N for "more realism" when a
  smaller N demonstrates the same contract; do not lower it below the phenomenon's floor
  (that is the unreachable-gate hazard, §5). Perf gates keep the N=12/48/96 envelope.
- Full suite is release-mode (`--release`); debug-mode runs of long-horizon tests take 10–20×.
- Snapshot drift is reviewed via `cargo insta test -p mindstrata-tests --release`, then
  `cargo insta accept --all` **only with documented evidence** of why the shift is expected.
- The canonical success signal is **observable output** (probes, rendered artifacts, test
  runs), never source diffs or intent.
- Stale binaries produce fix-less reproductions: rebuild before re-auditing.
- **Documentation alignment is gated.** `scripts/gate` runs `scripts/doc_index.py`: every
  non-exempt doc must be classified in `docs/DOCUMENTATION.md` §6, and AUTHORITY/ACTIVE
  docs must carry a resolvable `reconciled_commit`. A behaviour change that invalidates an
  assertion in `docs/ENGINE_STATUS.md` (the single source of engine truth) or a ledger row
  is **not done** until that doc moves in the same commit. If a doc contradicts
  `ENGINE_STATUS.md`, the doc is stale — fix it or mark it `SUPERSEDED`; never leave the
  contradiction standing.

## 4. Calibration Honesty Rules

These rules exist because we repeatedly paid for violating them:

1. **No lucky-seed re-pins.** If a test only passes on one seed after a sweep, the *system*
   is broken — fix the hazard/gate design, not the seed. (Iter-236 shipped 24 red and
   created a backlog that took six iterations to clear.)
2. **Test re-anchors require probe evidence in the comment**: the measured value, the old
   band, and the *mechanism* that moved it ("health-sync restoration lifted contempt to
   0.073"), never "widen until green."
3. **A dead producer is a bug even when tests pass.** Tests can pass on saturated states
   (fear pinned at 0.99 everywhere). Probe equilibrium values, not just assertions.
4. **Distinguish re-pins from re-contracts.** Re-pin = measured magnitude drifted, same
   contract. Re-contract = the old assertion tested something heredity/pacing legitimately
   invalidates (e.g., magnitude-similarity bands); say so explicitly and guard the real
   invariant instead (liveness, positivity, decoupling bounds).
5. **Knife-edge flags are debt.** When a pin sits on an unstable equilibrium (epidemic R0≈1
   flipping TRANSIENT↔ENDEMIC nine times), record it as systemic debt in the ledger rather
   than flip-flopping pins forever.
   **Corollary (i388): a MAX over one seed's stochastic path is a knife-edge by
   construction.** Three pins broke this way in one iteration — a pain maximum, a Help
   count, and a revolution count — each of which had been re-anchored by seed-shopping
   before. Preferred forms, in order: (a) the mechanism invariant over a small family
   (`warm > cold` on ≥2 of 3 seeds, aggregate ≥5%), (b) reachability over a family
   ("at least one seed reaches the band" + per-seed liveness), (c) a per-seed route pin
   instead of a seed family. When a family member dies, **re-discover the family by sweep**
   — never widen the bar to keep the old members.
6. **Organic vs realism-preserving constants (i373 audit).** A constant is
   *realism-preserving* when the hard number IS the modelled natural law (physiology,
   mutation noise, the cognitive ontology, solver damping) or a performance envelope —
   endogenizing it is REFUSED by default. A constant is an *organic candidate* when a
   state variable in the sim already measures what the constant guesses at — promote it
   to a state-derived law, behaviourally, with its own sweep. The full classification
   lives in `evidence/i373_hardcodedness_audit.md`; the queue is led by the
   legitimacy-coupled dividend share (**LANDED, i374** — `s = s0 + k·(1−legit)`,
   midpoint-neutral through the old constant at equilibrium legitimacy 0.6; the
   treasury now feeds back through legitimacy), endogenous tax policy,
   geography-derived marriage distance, and status-scaled patronage capacity
   (**LANDED, i375** — `cap = 3 + floor(status×4)`; redistribution only, volume
   and Gini band unchanged), and the legacy v1 trust row (**LANDED, i376** — it
   converges onto the dyadic `RelationshipV2` store instead of mean-reverting toward a
   hardcoded 0.5; the purest instance of the test, because the state variable that
   already measured the right thing was *the other store of the same quantity*).
7. **Probe a store's equilibrium, not only a `const` census (i376).** The highest-yield
   audit method is to ask whether a store's *equilibrium is doing work*: i376 found a
   14×-mistuned counter-force, a whole saturated reader family, and two divergent per-act
   gains for one quantity — none of which a `const` grep surfaces, because they are
   numbers inside call sites. Alternate both methods each refresh.
8. **Test scaffold is not simulator configuration.** Seed families that probes and pins
   measure on (panic seeds, golden-window seeds, liveness seeds) are not hardcoded sim
   values. Re-anchoring one is legitimate **only** with a runnable sweep (i376: 10-seed
   panic sweep `{7,11,46}`→`{7,1,23}`; 70-seed golden-window sweep back to canonical seed
   42) and the mechanism recorded. Never re-anchor a family from a single failing run.
   When a family has been re-anchored three times, the family is the fault, not the
   seeds: hold a fixed swept family and DISCOVER which members fire (i378), so a pacing
   shift moves which member carries the downstream legs instead of breaking the pin.
9. **Measure a trigger's HEADROOM before diagnosing it (i378) — and always sample BOTH
   SIDES (i381).** "The pin family moved" and "the producer is starving" look identical
   from a firing count and want opposite fixes. Sample the trigger's own inputs every
   tick and report the best score it ever reached: a clearing margin of a few per-cent
   means the threshold is an **absolute threshold on a moving distribution** (i378:
   firing legs cleared by 2.5–9.4%, one swept seed missing by 0.5%) — the fault is the
   absolute form, whose organic redesign is a **relative/anomaly** trigger against the
   population's own baseline. A large negative margin means the producer is genuinely
   under-driven — revive it. **i381's correction:** a corpus of *firing* seeds alone
   cannot size the redesign. i378 concluded "`panic_ratio` is the clean discriminator"
   from crisis seeds only; the calm worlds are in fact *warm* (85% of ticks above that
   leg) and always had been. Size a level bar from the gap between the highest
   **must-not-fire** plateau and the lowest **must-fire** spike (i381: 0.4188 vs 0.5475
   — a 1.31× gap whose geometric midpoint is the shipped floor), and state the
   achievable margin honestly: the relative form changes *what the bar depends on*, it
   does not manufacture headroom the signal does not contain.
10. **Census GATE SELECTIVITY, not just constants (i379).** A threshold is only a defect
   when it stops discriminating, and the first two audit methods cannot see a gate whose
   *reachable band* sits entirely on one side of its bar. Measure the share of samples
   that OPEN each gate, over a **distribution type × context** grid — and always read a
   `0.0%` open-rate against the world that *should* open it (`hunger > 0.85` is dark in a
   fed village by design). The four rulings: **quantile gates over the fixed founder draw
   stay hardcoded** (their robustness IS the uniform draw; reshaping it silently moves
   every one — §5 H5); **crisis gates on bounded need scales stay hardcoded**;
   **gain/scale mismatches** split in two (i380): **3a** — the term is small but the channel
   is alive, so rescale and pin the outcome (i376's v1 trust gains, where rescaling
   worked); **3b** — the candidate loses the argmax on the terms it does NOT have, so
   rescaling is a decoy (`Socialize`: 0/1 utility selections across 112 669 arbitrations,
   and neither a ×6 gain nor a ×20 need accrual revives it). **Before rescaling a term
   that "rounds to nothing", compare it with the measured WINNER's utility** (0.692 mean
   here) — if no plausible value reaches it, the term is not the defect. **absolute
   thresholds on self-driven aggregates** (panic charge, the faction legitimacy arm) get
   the relative/anomaly form. Full audit: `evidence/i379_gate_selectivity_audit.md`.
   Audit method ladder: `const` census → store-equilibrium probe (i376) → gate
   selectivity (i379) → trigger headroom (i378). Use all four; each is blind to what the
   next one finds.
11. **A gate census is a snapshot of ONE seed — and verifying a DEFECT does not verify
   the REPAIR (i382).** i379 read the faction legitimacy arm (`legit < 0.5`) at 0.0% open
   in four worlds and queued it as unreachable; the i382 probe (5 scenarios × 3 seeds)
   measured **0.00% in 8 of 15 worlds and 0.07–4.26% in the other 7**, against a 0.54–0.58
   mandate equilibrium: the defect (an absolute bar a hair below its own distribution) was
   right, the *shape* (dead) was wrong, and the armed set was decided by the seed. Two
   rules follow. **(a)** An audit verdict needs the seed dimension before "unreachable"
   becomes a fact; one seed's 0.0% is a sample, not a property. **(b)** The same corpus
   that proves a defect must be used to choose the repair: i382's plan opened with
   *deleting* the arm (the i372 precedent for a dead predicate), and the corpus refuted it
   — 5 of 8 formations were cliff-only, all in quiet worlds the accumulator cannot
   reproduce (it integrates; the instant arm is the step response). Count what a change
   would REMOVE across the corpus, not only what it adds. Also: **check every SITE of a
   reference** — the same absolute `0.5` fed both the arm and the i240 accumulator's
   deficit, so both were dead together, and fixing one would have left the other inert.
12. **Match the metric's SCOPE to the predicate's scope (i383).** A per-agent gate measured
   per *tick* ("did any agent clear it") reads 100% whenever one agent is above it and 0%
   whenever none is — figures that have nothing to do with the gate's discrimination. i383's
   first probe reported `fear > 0.5` at **100.00% in all 10 worlds** and nearly shipped the
   verdict "the shock gate is unconditional decoration"; at the predicate's own scope the arm
   opens a healthy **15.2–38.8%** of agent-ticks and needed no change. State the scope in the
   probe's column headings, and re-read every alarming rate at the other scope before acting.
   Corollary (the same iteration): the audit's class can be **split across arms of one
   predicate** — `anger > 0.5` was dead while `fear > 0.5` beside it was live, so fix the arm
   that is dark and leave the one that discriminates (§4.10).
13. **Check the MANIPULATION is live before re-pinning the RESPONSE (i384).** A pin that has
   been re-anchored over and over is usually not measuring a fragile producer — it is
   measuring a dead experiment. `neural_like_prediction_error_folds_are_live_and_directional`
   had been re-seeded five times (42 → 99 → 20 → 11 → 99 → 22, then re-contracted to a 6-seed
   majority) because its "differential" flipped sign every era. The real cause: grain was
   zeroed once at t=0, production/foraging refilled it, and by the 5000-tick sample the two
   arms were the same world — **starvation raised mean hunger on 1 of 12 seeds**, so the
   measured deltas (1/6 wins, best +0.0220; widened family 4/12 = chance) were trajectory
   noise. Holding the set point every tick (reviving the producer, §2.3) made the contract
   land with real margin (hunger 6/6, wins 5/6, best +0.0995). Rule: for any pin over a
   trial/control pair, first report the **manipulation check** — the share of the family in
   which the independent variable actually moved — and treat a majority-miss as a test bug,
   not a calibration result. Corollary: a gain-mismatch flag can be a **migration** flag
   rather than a rescale (i373 filed the v1 interaction gains as Class 3a; i384's honest
   fix at the economy site was deleting the v1 write, since rescaling a writer queued for
   deletion buys nothing) — read the consumers before choosing.
14. **A governing doc's citations are part of its contract (i386).** When an extraction or
   migration moves a file, every doc that named it gets re-pointed **in the same commit** —
   `FROZEN` freezes the *rule*, never the path. Corollary: before trusting a ledger row's
   claim about **where** a value landed, open the file it names. The i386 sweep found three
   drift classes across the governing set while `scripts/doc_index.py` — which checks
   *structure*, not citations — reported OK: ghost citations (`IC-2-observability.md`,
   `IC-7-ui-telemetry.md` never existed under those names), moved files
   (`sim/pass_health.rs` → `systems/health.rs`, `psychology/lore.rs` →
   `development/lore.rs`), and the expensive one — **a doc asserting a value was NOT landed
   while the code had shipped it as a difficulty band under another name**
   (`PATHOLOGY_GROWTH_*` was never created; `PROD_QUADRANT_PARAMS` +
   `SimParameters::pathology_*_scale` is the real surface). A stale "not landed" is worse
   than a stale line number: it invites rebuilding live behavior, and it hides a shipped
   lever from anyone planning the next sweep.
15. **Measure a consumer's RESPONSE ELASTICITY before choosing it for a heritable trait
   (i392).** Midpoint neutrality (§4.6) is necessary and **not sufficient**: `novelty_seeking`
   → the exploration driver was wired exactly midpoint-neutrally (the gene-pinned-0.5 control
   reproduced the stored golden metric_hash **byte for byte**, so §4.6 was *proven*, not
   asserted), the channel was live and monotone under an A/B with one number pinned, and the
   population-mean multiplier stayed within ±5% of 1.0 — and it was still the wrong place to
   put the trait. The driver's response to its own coefficient is **elasticity ≈ 4**
   (coefficient 1.92 → 2.16 buys 1 853 → 3 256 Wander wins; the gene's full range is a 4.1×
   spread), so *aggregate* neutrality hid *per-agent* redistribution: the top carrier's
   coefficient rose to 2.29 while the rest fell, and the world's **revolution liveness family
   collapsed to 1 of 3 seeds** plus 2 goldens and 5 snapshots. **Rule:** before wiring a
   heritable trait onto a knob, probe the knob's response curve across the trait's range and
   require it to be *proportional*; an elastic surface turns a trait difference into a
   population-level lottery over the founder draw, and a rejection recorded with those numbers
   is a landing (i380/i392 precedent), not a failure. Corollary: re-anchoring is not a licence
   — a **liveness** producer going dark (§2.3) is never re-pinned to accept zero, whatever the
   golden diff says.

## 5. Known Systemic Hazards

- **Fixed-4 truncation disease**: `Fixed::mul` truncates; any per-tick increment below
  5e-5 quantizes to zero. Killed thermal convergence (239), faction pressure (240),
  gestation advance (242), epidemic exposure (244). **Rule:** sub-resolution rates compute
  in f64 and quantize once (`Fixed::from_f64(x)`), or accumulate in f64 shadow fields.
- **mem::take write-back trap** (Iteration-218 refactor): writes to `self.agents[i].body`
  INSIDE the biology pass are discarded unless synced during/after the write-back loop.
  Health-sync was silently dead for 24 iterations because of this.
- **Midpoint neutrality**: when coupling a genome/state multiplier into existing math, shape
  it so the multiplier = 1.0 at the population midpoint (gene 0.5), or golden/snapshots
  shift for everyone, not just carriers.
- **Calibrated-window dormancy is the golden gate's real condition (i392, all five rows).**
  Midpoint neutrality is about the *population*; the golden gate is about the *window*. Every
  coupling in this engine that passes `gate --full` byte-identically does so because it is
  **dormant throughout the calibrated horizon** — the repeated `identity at zero` device:
  norm resistance is zero before the first monthly ritual (tick 4320), obligation is 1.0 at
  the 0.5 anchor, humiliation/contempt/despair are never produced in calm worlds, the puberty
  ramp opens at ~385K ticks, severe injury never occurs in any corpus. A trait **drawn at
  founder time** cannot use that device — every agent deviates at tick 0 — which is exactly
  why `puberty_age` (row 1) and `chronic_pain_risk` (row 3) landed while `novelty_seeking`
  (row 2), `sensory_acuity` (row 4) and `aggression_threshold` (row 5) were all rejected:
  row 5's locus was *proven* live and *proven* smooth (pool share 41 → 37 → 31 → 11 → 6,
  the gradient rows 2/4 lacked), and it still broke four independent families (revolution
  1/3, fear-contagion 10/12, prediction-error 3/6, founding kinship 0.5 ≠ 0) plus both
  goldens. **Rule:** before wiring, name the state that makes the coupling zero at tick 0
  and verify it is zero for the whole calibrated horizon; if the only such state does not
  exist, the coupling needs one (route the expression through a tick-0-zero state such as
  `norm_resistance > 0`) or the row is a rejection with a record — re-engineering the
  goldens is not the fallback.
- **RNG stream discipline**: birth-path constructors consume draws in field order;
  `Personality::inherit` consumes exactly one draw per trait to preserve alignment.
  Different range widths consume different byte counts — count-alignment ≠ byte-alignment.
- **The unreachable-gate / dead-producer class (i306, i307, i346–i351).** A threshold or
  gate calibrated against a channel that was never actually live is a silent bug: `Wander`
  lost all 107 085 arbitrations by 25–30× the jitter, the feud gate sat past p99.9 of its
  channel, the `Background` entry gate below its reachable floor, and the witness channel
  had no locality test at all (village trust ratcheted to 1.000). **Rules:** probe the
  channel's *reachable distribution* before trusting any gate on it; and remember a 0.00%
  share cannot distinguish "the branch never runs" from "it runs and loses" — instrument
  the decision (`sim::decision_census`) before choosing a fix. Tests passing on a saturated
  or dead channel is the failure mode this class exists to catch.
- **Founder variance IS the behavioral budget at small N (Iteration 263, audit
  H5)**: reshaping founder trait draws away from U(0,1) starves every
  extreme-driven producer. A triangular/bell draw killed the stress axis (max 0),
  plasticity deltas (0.000), fear contagion, courtships, and faction formation;
  even a variance-preserving trapezoid (tails <0.05/>0.95 clipped 8%→1%) broke 13
  liveness pins (fear contagion 0.33→0.24, motivation context, violence window,
  prediction-error seeds). **Rule:** uniform founder draws are load-bearing at
  N=12; realistic distribution shaping requires a larger founding population AND
  a coordinated re-anchor sweep across all extreme-driven producers. Recorded as
  systemic debt — do not attempt piecemeal.

- **Dead fields are found by census, not by hunt (i391)**: `python3
  scripts/field_census.py --file <defs> --sites` counts writes / constructor literals /
  reads per field and treats a read inside the field's OWN file as plumbing, so only a
  **cross-module** read counts as a consumer. Run it before adding a trait/behaviour gene
  (five genes shipped dead this way: `aggression_threshold`, `novelty_seeking`,
  `chronic_pain_risk`, `sensory_acuity`, `puberty_age`), and before claiming a genome-wide
  distribution result — trait variance over inert draws is a false affordance. Suspect rows
  are pointers, not verdicts: an aggregate whose consumer lives in its own file is the
  documented false-positive class.

## 6. Rust Craft Standards (rust-best-practices handbook)

Based on Apollo GraphQL's [Rust Best Practices Handbook]
(https://github.com/ApolloGraphQL/rust-best-practices) — full chapters live at
`~/.agents/skills/rust-best-practices/references/`. Adapted to this repo's realities
below; where this section conflicts with a determinism rule from §5, determinism wins
and the deviation gets a `// ponytail:` comment naming the ceiling and upgrade path.

### Borrowing, cloning, ownership

- Prefer `&T` / `&str` / `&[T]` parameters; clone only at ownership-transfer points
  (birth-path value snapshots are legitimate: the parent keeps living).
- Small `Copy` types (`Fixed`, indices, bools) pass **by value** — do not take `&Fixed`
  in new APIs.
- No `.clone()` inside per-tick loops; hoist or borrow. The tick pipeline runs 100K+
  iterations in probes — redundant clones there are real regressions.
- Use `.iter()` over `&Vec` collections; `.copied()`/`.cloned()` at the END of an
  iterator chain, not per-element maps.

### Error handling

- Fallible operations return `Result<T, E>`; no `unwrap()`/`expect()` outside tests.
  (Existing sim code predates this rule; fix opportunistically, never churn a file
  just for this.)
- Library crates use concrete error types (`thiserror` when introduced); `anyhow` is
  for binaries only.
- Prefer `?` and combinator methods (`map_or`, `ok_or_else`) over match chains.

### Performance mindset

- Benchmark with `--release` only; debug numbers are meaningless here.
- Watch clippy's perf lints: `redundant_clone`, `needless_collect`,
  `large_enum_variant`, `clone_on_copy`.
- Hot-path math stays in `Fixed`/f64 primitives — no allocation in per-tick passes;
  iterators over index loops where it reads better, not for its own sake.

### Linting discipline

- The gate command (§3) already covers the workspace; treat NEW warnings as errors.
- Fix warnings, don't silence them. When suppression is truly justified use
  `#[expect(clippy::lint)]` (auto-fails when stale) with a why-comment — never bare
  `#[allow(...)]`.
- Transitional `use super::*` wildcards from the sim split are tracked debt (§7
  queue item), not a license for new ones.

### Testing craft

- Descriptive names stating behavior: `values_transmit_vertically`, not `test_moral`.
  One concept per test; probe evidence lives in the comment.
- Doc tests (`/// ```rust`) for pure public helpers — see `person::inherit_surname`
  for the pattern.
- Snapshot drift via `cargo insta` per §3 rules.

### Generics & dispatch

- Static dispatch by default; `dyn Trait` only for genuinely heterogeneous collections.
  Box at API boundaries, not inside hot loops.

### Type-state pattern

- Encode invalid states in types when a lifecycle exists (e.g., pregnancy phases,
  institution lifecycles). Adopt incrementally on touched code; do not retrofit working
  state machines wholesale.

### Comments vs documentation

- `//` explains WHY (calibration evidence, hazard workarounds, the mechanism that moved
  a test); `///` explains WHAT/HOW for public API consumers.
- Every calibration re-anchor comment names: measured value, old band, mechanism
  (§4.2). A TODO without a follow-up path doesn't belong — record it in the plan doc
  instead.
- `#![deny(missing_docs)]` is **enforced** in the leaf crates born or migrated under the
  crate ladder (`person`, `psych`, `social`, `institutions`, `world`, `development`). It is
  deliberately **not** yet enabled in `mindstrata-core` or `mindstrata-sim`, whose public
  surfaces still churn — do not enable it workspace-wide until those settle.

## 7. Hierarchical Module Splitting (the scaling foundation)

As systems grow, files MUST split hierarchically — a god-file cannot be developed safely
by parallel agents or reviewed precisely. The pattern established by the sim.rs split
(17,942 lines → `sim/` directory):

### When to split

- A file exceeds ~2,500 lines or ~15 top-level symbols of mixed concern.
- Two agents need to work the same file concurrently (split FIRST, then parallelize).
- A pass/system has its own data structures and lifecycle inside a shared impl.

### How to split (proven procedure)

1. **One domain per module**: `sim/family.rs` (marriage/birth/kinship), `sim/economy.rs`,
   `sim/norms_impl.rs`… name by domain, not by layer.
2. **The struct stays in `mod.rs`** (`Simulation`, `AgentBundle`): state definitions live
   at the root; behavior lives in domain modules as `impl Simulation` blocks using
   `pub(super)` visibility for internal steps.
3. **Move code verbatim** — a split is a pure refactor proven by **byte-identical golden**
   runs and an unchanged suite. Never mix behavioral changes into a split commit.
4. **Tests follow their subject**: `sim/tests.rs` for unit-level, per-domain test modules
   in `integration_tests/{economy,governance,psychology,social,…}/`. Namespaced paths make
   failure triage instant.
5. **Wildcards are transitional**: `use super::*` in fresh impl modules is tolerated
   during migration but is cleanup debt — settle to explicit imports once the module
   stabilizes.

### Current layout (post crate-ladder; refresh me when structure moves)

> **The `<15K` orchestration target is RETIRED** (doctrine correction `9c53812`). `sim` is
> ~36K LOC and is deliberately *not* shrunk below the point where the domains it glues can
> be read and reviewed; file/crate boundaries, not LOC, are the scaling currency. The
> remaining `sim`-internal tidy-up (Arc-D pass extraction into `systems/`, `*_impl` glue
> detangling) continues as infrastructure work in its own iterations.

```
crates/
  mindstrata-core/          # Fixed, ids, clock, events, rng, parameters, propositions
  mindstrata-person/        # person/ aggregate + biology/ (11 systems) + health  [leaf, deny(missing_docs)]
  mindstrata-psych/         # psychology/ (16 systems) + appraisal + memory/attention/
                            # belief_update/journal                          [leaf, deny(missing_docs)]
  mindstrata-institutions/  # institutions/legal/diplomacy/military/theology/
                            # schools/norms/factions (pure domain)           [leaf, deny(missing_docs)]
  mindstrata-social/        # social/ culture/ noosphere/ + gossip/conflict    [leaf, deny(missing_docs)]
  mindstrata-world/         # world/world_gen/ecology/market/logistics/
                            # demography/black_market                        [leaf, deny(missing_docs)]
  mindstrata-development/   # AFA developmental lines + field engine          [leaf, deny(missing_docs)]
  mindstrata-sim/           # ORCHESTRATION (~36K; see the retired-target note above)
    sim/mod.rs              #   Simulation struct, AgentBundle, wiring
    sim/core.rs             #   tick() pipeline order + per-pass profiler
    sim/pass_*.rs           #   the five remaining verbatim passes
                            #     (action, ecology, scenario, social, weather)
    systems/                #   Arc-D-extracted passes: biology, cognitive,
                            #     appraisal, decay, health, development, genesis,
                            #     trade_diffusion, institutions_multiplier (+INVENTORY.md)
    sim/decision_census.rs  #   action-selection instrument (opt-in, inert by default)
    sim/{population,api}.rs #   constructors/seeding; the public read API
    sim/{household,economy,births_deaths,marriage,clans,education,
      cults_noosphere,memory_ops,norms_impl,social_cluster,chronicle,
      assets,catalyst_observers}.rs + {factions,institutions,diplomacy,
      legal}_impl.rs + snapshot_metrics.rs
                            #   impl-Simulation domain glue (~26 files; detangle target)
    actions/{mod,tests}.rs  #   action-selection engine (sits above domains)
  # crate-root infrastructure (not under sim/):
    {routines,scheduler,snapshot,scenario,spec_lint,agent_tier,
     provenance,population_cap,mods}.rs
    legacy shims in lib.rs preserve pre-extraction crate:: paths
  mindstrata-tests/         # integration_tests/{biology,psychology,social,culture,
                            #   governance,economy,legal,infra}/ + snapshots/golden
  mindstrata-tui/           # {lib,render,session,assets_view,scene}.rs
  mindstrata-cli/ mindstrata-render/ mindstrata-benches/  # entry points, 217 probes
```

DAG (cargo-enforced): `core ← person ← psych ← {social, institutions, world}`;
`development` below `psych`; all leaf crates ← `sim` ← `tui/cli/render/tests/benches`.

### The module → crate ladder — **DONE** (S1–S3, commits `e702a77`, `f66b988`)

File splits fix navigability; only **crate boundaries** fix build-time coupling and
dependency direction. The ladder landed one rung per iteration:

1. **S1 — DONE (closed as a no-op).** Round 1 had already cleared production wildcards,
   so the rung shipped the **coupling map** (`docs/scaling/coupling_map.md`) instead.
2. **S2 — DONE.** `mindstrata-person` (+ `biology/`) extracted as the first leaf crate.
3. **S3 — DONE.** `mindstrata-psych` (+appraisal), `-institutions`, `-social`, `-world`
   extracted in coupling-map order. **Every extraction is golden-proven byte-identical**,
   and the DAG above is cargo-enforced. New crates were born with `deny(missing_docs)`.

**Adding a new crate** follows the same discipline in `docs/PLAN_SCALING_FOUNDATION.md`.

Crate-extraction discipline (the procedure that produced the ladder):

- **Golden replay is the referee** — every extraction byte-identical, behavioral
  changes never mixed into structural commits.
- **Shim transitions** — old paths keep compiling via `pub use`; call-site churn is a
  separate later commit or nothing.
- **Coupling survey before surgery** — never pick seams without the S1 map; if an
  extraction hits a genuine cycle, STOP and record it as an architectural finding.
  Cycles are resolved behaviorally in their own iteration; never papered over with a
  god-crate.
- **Serde shape is frozen while moving** — no attribute/field-order edits inside an
  extraction (snapshot compatibility rides golden's agent_hash).
- **New crates get `deny(missing_docs)` at birth** — cheap when born, expensive later.

### Splitting etiquette for parallel agents

- Check `git log --oneline -3` and `git status` BEFORE editing; another agent may have
  landed mid-flight changes (this happened: a struct edit survived only because the
  splitter migrated uncommitted content).
- Never edit files another agent is actively migrating (watch mtimes); poll compile state
  instead of force-fixing a half-written file.
- Do not create branches/worktrees unless explicitly asked; work lands linearly on main.
- Commit boundaries belong to the iteration owner; if your edit rides inside someone's
  uncommitted file, note it honestly in the commit message.

## 8. Where Things Stand / Next Work

**This section is a pointer, not a ledger.** The authoritative set is:

- **`docs/ENGINE_STATUS.md`** — current engine truth (behaviour, realism, measured scale).
- **`docs/PLAN_DC5_DEVELOPMENT.md`** — **the live plan** (DC-5): the measured gap taxonomy
  G1–G8, the iteration ladder (i387+) with a probe and an exit criterion per item, its
  dependencies, and the operator decisions it depends on.
- **`docs/PLAN_DC3_DEVELOPMENT.md`** — the DC-3 ledger (closed) + the DC-4 execution ledger
  and the §3 calibration-debt history.
- **`docs/DOCUMENTATION.md`** — which doc owns which scope, and the status of every doc.
- **`docs/architecture/AP4-studio/evidence/`** — per-iteration probes and measured verdicts.

Keep those current and treat this section as a summary. It has gone stale twice — once by
holding the Iteration-247 queue while DC-3 closed and DC-4 opened, and once by naming the
sparse relationship store "the only remaining lever" after i338/i344/i350 had measured and
demoted it. `scripts/doc_index.py` (in the gate) now catches the *structural* drift; the
judgement calls remain ours.

Landed since this section was last written (see `git log` for the trail): the full
module-segregation refactor, the **crate ladder** (S1–S3: `core ← person ← psych ←
{social, institutions, world} ← sim`, DAG cargo-enforced, every extraction golden-proven
byte-identical, new crates born with `deny(missing_docs)`), **DC-3 complete at 4/4 legs**
(Era IV holon, multi-village UM-3, N≥48 perf budget + `scripts/gate` perf envelopes, asset
pipeline v0), and **DC-4** (difficulty-levers rows 2–3 live, pathology/goal-gate bands,
meaning/physiological reflex repairs, injury channel, pain-veto revival, TUI asset/scene
viewers, and the scale program).

The **dead-producer campaign** (i346–i351) then anatomized and revived the action layer
with the `sim::decision_census`: `Move` (feud gate above reach + shadowed by routine,
i347), the `Background` tier (entry gate below floor + a social clause starving factions,
i348), the witness channel (village trust ratcheted to 1.000 — no locality test, i349),
the per-agent mean-folds (stranger-diluted, i350), and `Wander` (zero relief — plus a
`Fixed::from_raw` unit bug the census caught, i351). i352 closed the last per-tick dense-
matrix scan (the marriage pass was O(N³) every tick).

**Uncommitted-once-verified — i392 + A9:** `i392` row 2 landed as a **measured rejection**
(`evidence/i392_novelty_driver.md` — the `novelty_seeking` → exploration-driver wiring was
live, monotone and midpoint-neutral *at machine precision*, and still reverted because the
surface is elastic (≈4) and the revolution liveness family fell to 1 of 3 seeds; the
rejection exposed **i392b**, the driver's stale 5.44%/3.53%-vs-0.5–3% band with no test behind
it). **A9 is DECIDED and landed** (`evidence/a9_density_policy.md`): the constant-density
world law is the **interim** area policy, **single-sourced** in the world generator after
seven stale copies, anchored so the calibrated corpora cannot move, and recorded with the
**first-principles path that retires it** (carrying capacity → causal space → encounter-driven
contact → fission) in `PLAN_DC5_DEVELOPMENT.md` §4. Two corrections this pair of iterations
insists on: a consumer's **response elasticity** is part of its suitability (§4.15), and the
`max co-location 19 → 4` fix was **A11's (i345)**, not A9's.

**i392b LANDED** (`evidence/i392b_wander_band.md`) — the exploration driver's stale band is
**measured, re-contracted and pinned by a test for the first time**. i351's "0.5–3% of
decisions at 20K" had stopped being true (12-seed family: **1.67–8.81%, mean 4.30%**) and the
breach was already sitting in `ENGINE_STATUS` §5's own i384 table, with nothing asserting it
because the instrument was a probe. The re-contract was decided by the band's own stated
rationale: **no Work displacement** (Work +1.18 pt at N=12, −0.76 pt at N=48 at the recorded
config), so re-calibrating would have been a magnitude knob pulled for nothing (§4.4). The
contract is now **liveness + gate exclusivity (≥99% of wins quiet-window; measured 100%) +
boundedness (≤12%, sized between the calibrated 9.20% and the over-drive coefficient 3.0's
13.30%)**, and both halves were **proven to trip** (coefficient 0 → liveness; 5.0 → bound).
Behaviour is unchanged — goldens byte-identical. Recorded debt: **3–76% of Wander's wins are
noise-decided** (mean 32%), an action-layer margin question. **i392 rows 3–5 then ran and the
act is CLOSED**: row 3 landed, rows 4 and 5 rejected with records — see the W0 entry below.

Live queue, in order (evidence link per item):

0. **DC-5 W0 — the named dead/inert surfaces (live now).** **i387 LANDED** (the utility is
   a labelled bucket vector + the census records winner/runner-up/`Socialize` vectors per
   arbitration — the measurement that decides the action layer's fate); **i388 LANDED**
   (`evidence/i388_relational_outlet.md`: the relational urgency family + the
   population-relative social band, which took utility-selected `Socialize` 0 → 109/20 121
   and live goals 0 → 9/51, with both goldens, 7 snapshots and three pins re-anchored by
   measurement — the pain veto band re-contracted to a family reachability invariant, the
   revolution family rediscovered as `{5, 42, 12345}`, the tenderness Help pin moved to a
   family mechanism at ≥5%). **i389 LANDED** (`evidence/i389_command_channel.md`: the
   council now speaks — `systems/decree.rs`, a bounded legitimacy-scaled ask shipped as
   `GoalSource::Decree`, a directive that DECAYS where the operator's `Command` is durable;
   calm 0.00% with the calm golden byte-identical, collapse 0.05%, collapse golden
   re-anchored at agent_count 12 → 12). **i390 LANDED** (`evidence/i390_office_succession.md`:
   the lifecycle gap i389 exposed — with the seat filled the pestilence authority channel was
   live, so the vacancy *was* the root cause. `systems/succession.rs` fills vacant seats from
   the living membership on a 50-tick cadence by the institution's own criterion
   (`status_v2.effective_status()`, ties on `AgentId`, no RNG stream). Measured with the pass
   disabled: Elder seat **100.00% vacant with a candidate pool present 100% of the time**,
   directive 0.0000%; after: **0.00% vacancy**, directive **0.0654%**, **calm 0.0000%**
   (authority still silent when nothing is wrong) — and **both goldens byte-identical**,
   since neither golden window contains an office-holder death. **i391 LANDED**
   (`evidence/i391_dead_field_sweep.md`, measurement): `scripts/field_census.py` sweeps every
   field for writes/literals/reads and counts only a **cross-module** read as a consumer, so
   plumbing (constructor/`Default`/`random`/`inherit`/blend) no longer masks the class. Six
   suspects, **five confirmed dead genes** — `aggression_threshold`, `novelty_seeking`,
   `chronic_pain_risk`, `sensory_acuity`, `puberty_age` are drawn, defaulted and blended, and
   read by nothing outside `genome.rs` (the 6th, `EmbodiedState.metabolic`, is the
   instrument's documented false-positive class: a consumer in the field's own file). One row
   is a live affordance the engine already measures — `reproductive.rs:168` read a hardcoded
   `13.0` while `puberty_age` draws 11.0–15.0. **i392 row 1 LANDED**
   (`evidence/i392_puberty_clock.md`): the clock now reads
   `ReproductiveUpdateParams.puberty_age`, fed from the genome, with the retired constant as
   the default so the wiring is midpoint-neutral by construction. Probe: gene spread
   3.3–3.9 yr; **0 agent-ticks in age [10,16) over 240K + 987K agent-ticks** (founders ≥18,
   the ramp opens at ~385 000 ticks) so the fix is invisible in every corpus — and both
   goldens are byte-identical, 0 re-anchors. Live at the clock: a 12-year-old reads
   `Early`/0.1428 on gene 11.0, `Prepubescent` on 15.0, and identical to pre-fix on 13.0.
   **i392 row 2 MEASURED AND REJECTED** (`evidence/i392_novelty_driver.md`): the plan's
   named consumer for `novelty_seeking` was the i351 `Wander` driver, and the wiring was
   built, measured and reverted — live and monotone under a pinned-gene A/B (132 → 2 464 →
   3 927 Wander decisions), **§4.6 proven at machine precision** (the gene-pinned-0.5 control
   reproduces the stored golden `metric_hash` `0xb2a1be3b18fbb46d` byte for byte, and the
   natural-gene divergence starts at tick 657 for agent 0 alone, gene 0.8381), and still
   wrong: the surface is **elastic (≈4)**, so aggregate neutrality (±5% mean multiplier) hid
   per-agent redistribution and the **revolution liveness family fell to 1 of 3 seeds** — a
   dead producer, which §2.3 forbids re-pinning, so the candidate was reverted rather than
   re-anchored. Row 2 also produced the act's **precondition (i392b)**: the driver's own
   acceptance band is stale — `i351_wander_bands` reads **5.44% / 3.53%** of decisions at 20K
   against i351's documented **0.5–3%** target, and *nothing pins it* (the instrument is a
   probe, not a test), so no gate ever re-measured the claim the coefficient was chosen for.
   **i392b then re-contracted that band** (measured 1.67–8.81%, mean 4.30% over the 12-seed
   family at i351's own config, against a documented 0.5–3% that the doc's own i384 table had
   already breached; no displacement — Work +1.18/−0.76 pt — so magnitude recorded and
   liveness + gate exclusivity + ≤12% pinned by a test that was proven to trip).
   **Rows 3–5 then ran on that unblocked ground, and act i392 is CLOSED:** row 3
   (`chronic_pain_risk`) LANDED (`evidence/i393_chronic_pain_gene.md` — band closed in all
   five calibrated corpora over 2.29 M agent-ticks *by construction*, response exactly
   linear, anchored at the draw midpoint 0.25 so no +8% cohort drift); row 4
   (`sensory_acuity`) MEASURED AND REJECTED (`evidence/i394_sensory_acuity.md` — encode
   threshold inside the draw range, 13× occupancy spread, revolution 1/3); row 5
   (`aggression_threshold`) MEASURED AND REJECTED (`evidence/i395_aggression_threshold.md` —
   the locus was *confirmed* live and smooth, 41 → 37 → 31 → 11 → 6 across the gene range,
   and it still broke four independent families plus both goldens); and that fifth row
   produced the missing half of the doctrine as **§5 calibrated-window dormancy** — the
   golden gate is a *window* condition, so a founder-time gene on a tick-0-live gate cannot
   pass, its recorded upgrade path being a tick-0-zero state to route the expression through.
   **Five inert genes in, no inert gene left unwired-by-accident: two wired, three
   deliberately inert with records.** Then **i393** the speech-act/kind migration. Carried forward from i388 as
   measured-but-unexplained: action duration rose 4.31 → 5.40 ticks and the 2K relationship
   stages shifted shallower (`Unnoticed` 20 → 26, `Friend` 8 → 5) — W2 must decide whether
   contact falls when the drive is served *before* the sparse-store question is re-opened.

1. **The v1→v2 relationship migration (charter DECIDED: finish it subsystem by
   subsystem)** — **i369 landed the first reader migration** (comfort/soothing reads the
   dyadic v2 store; three-way runnable pin; collapse golden one birth flipped 13→12,
   in-contract). **i376 closed the divergence SOURCE**: the queued cognitive writer
   (`systems/cognitive.rs`) was not merely a divergence source but a producer failing at
   its own purpose — v1 trust saturated (mean 0.887–0.920, 79–85% of pairs ≥0.95 at 50K)
   because its 0.001/day mean reversion lost to the interaction gains by ~14×, while the
   dyadic store held 0.588–0.628, so the two diverged by 0.27–0.31 mean across 55–63% of
   pairs and every v1 trust gate read a dead signal. The legacy row now **converges onto
   the dyadic row** (`v1.trust += (v2.trust − v1.trust) × relationship_dormant_decay`
   daily, default 1.0 = full sync; baseline is the sim's own dyadic estimate, not a magic
   0.5). Remaining readers: economy, norms_impl, household, births_deaths — each
   behavioural with its own probe. Marriage pass REFUTED (i353). **Queued next from i376's
   evidence:** v1's interaction gains remain ~10× v2's (`record_positive` ×0.02), leaving
   a +0.06–0.08 offset — **re-measured as i384's open item**: at 50K/N=48 the offset is
   v1.mean 0.681–0.708 vs v2.mean 0.630–0.656 with 17–25% of pairs >0.05 apart, because the
   legacy row both gains ~50× faster per act and never decays within the day (v2 decays
   0.0002/tick). It is a **subsystem migration, not a coefficient**: the speech-act effect
   model documents its base deltas as the EXACT magnitudes `process_interaction` applies
   (guarded by `model_sign_matches_applied_deltas`), and the v1 `RelationshipKind` ladder
   rides on the same movement — both must move to the dyadic store in the same commit.
   **The first landing of the WRITER side is in (i384, `evidence/i384_economy_trade_store.md`):**
   the §13.3 trade price's trust read and the §19.5.J trade-trust write both moved to the
   dyadic store — the last self-contained v1 read+write pair. The probe sized the stake
   first: the two stores sat **+0.09…+0.13 apart on exactly the pairs that trade** (6–40×
   the population-wide offset) because this site was itself a v1 writer at a flat +0.02/act,
   **2× the dyadic +0.0100**, so the buyer paid 2.8–4.0% under the price its own trust
   warranted. After: read-source delta **2.84–4.03% → 0.34–0.52%**, traded-pair v2 trust
   rose to 0.93–0.98 (the write is live), trades +1.4…2.8%, Gini +0.02…+0.05. Both goldens
   re-anchored (agent_count 12 → 12), 4 snapshots accepted, the faction census went 12 → 14
   formations, three pins re-anchored on their own sweeps, and the new
   `trade_price_reads_the_dyadic_store_and_writes_it` pin guards the migration. **Remaining
   readers: norms_impl, household, births_deaths** (each behavioural, each with its own
   probe); the speech-act/v1-kind-ladder move is a separate commit. Doctrine gained §4.13
   (check the manipulation, not just the response).

   **i385 then reconciled the documentation surface itself, and the reconciliation paid for
   its own iteration.** `ENGINE_STATUS.md` §1 was 15K LOC / 30 probes / two suite counts
   stale and §5 still carried pre-i347 decision numbers; both are re-measured (see
   `ENGINE_STATUS.md` §1/§5) and `reconciled_commit` advanced. The substantive find: the
   **deliberative surface is 41.1% of decisions, not the documented 32.8%**, because
   `Wander`/`Idle`/`Move` — each dead or near-dead before i347/i351/i356 — now win
   arbitrations. And a probe was lying about its own engine: the i346 census feud leg
   measured the **retired** `anger > 0.4` bar (reporting 0.0000%) while the same run recorded
   `Move` firing 218/901 times; it now measures the shipped gate (`0.02` + needs guard) over
   the full window and agrees with the census. §5 also states two caveats that were only
   implied before: `Socialize` is **routine-only** (0 utility selections in 112 669
   arbitrations, i380 — the open root cause is now i386's winner-decomposition census) and
   the `Command` channel is structurally unreachable (no shipped generator — a deliberate
   no-op, not a dead wire).

   **i386 then swept the governing document set for citation drift** — the reconciliation
   that i385's own §1/§5 re-measurement implied. Five REFERENCE docs and the balance canon
   were carrying it: two ghost contract filenames in the interlock map
   (`IC-2-observability.md`, `IC-7-ui-telemetry.md`), the two pass files still named by
   their pre-extraction paths in the determinism law (`pass_health.rs` →
   `systems/health.rs`; `pass_biology.rs` → `systems/biology.rs`), `lore.rs` still credited
   to the psych crate after the ladder moved it to `development` (and its symbol to :100),
   and two balance docs asserting the `mindstrata-development` crate, its `canon.rs` and its
   markers **did not exist** while the crate, the constants and 27 code-side marker sites
   have been shipping since WP-0B. The pathology entry was the expensive one: the row-3
   lever is **fully live** (i304 growth/decay, i315 ceiling, measured 12-seed families) but
   its spec said "draft, values not landed" and named `PATHOLOGY_*` constants that were
   never created — the real surface is `PROD_QUADRANT_PARAMS` +
   `SimParameters::pathology_{growth,decay,ceiling}_scale`. Doctrine gained §4.14 (**a
   governing doc's citations are part of its contract**), and `canon-inventory.md` is now a
   status ledger with per-row landing verdicts instead of a forward-guess.

   **The panic-channel item is CLOSED as diagnosed (i378):** the
   firing-density move was not a dead producer — the trigger is an ABSOLUTE threshold
   sitting inside its own input distribution (firing legs clear by 2.5–9.4%, one swept
   seed missing by 0.5%), i.e. §4.5 knife-edge debt. Both panic tests now hold a fixed
   10-seed family and discover its firing members, so no further family renames. The
   queued follow-up is the organic redesign: a **relative/anomaly** trigger against the
   population's own charge baseline (sized by `i378_panic_threshold_headroom`).
   **That redesign LANDED (i381):** the trigger is now
   `avg_charge ≥ max(baseline × 1.25, 0.47) AND panic_ratio ≥ 0.30`, with a
   per-proposition EWMA baseline (τ = 10 × the 300-tick panic cadence) that adopts its
   first observation on a cold start. The floor 0.47 is the geometric midpoint of the
   measured decision gap (`crisis/42` 0.4188 must-not-fire vs `crisis/11` 0.5475
   must-fire); i378's knife-edge seed now clears by 16% instead of missing by 0.5%.
   Two contracts re-anchored with attribution (attachment distress re-contracted to a
   2/3 supermajority; the revolution family discovered as `{42,7,23}` by a 10-seed
   sweep); goldens and snapshots byte-identical. **Also corrected by i381:** i378's
   "`panic_ratio` is the clean discriminator" was an artefact of sampling crisis seeds
   only — calm worlds are warm (85% of their ticks meet that leg).
   **The same Class-4 ruling then closed the faction legitimacy reference (i382):**
   the absolute `0.5` appeared at BOTH sites (the instant cliff arm and the i240
   accumulator's deficit) and was dead at both in 8 of 15 worlds — the i382 probe
   measured it opening 0.00% of ticks there while the mandate equilibrium sat at
   0.54–0.58, and 0.07–4.26% elsewhere. Both sites now read a baseline the council's
   own history established (`LEGITIMACY_DEBT_TAU_TICKS = 100`, derived from the
   0.01/tick convergence rate; `LEGITIMACY_COLLAPSE_MARGIN = 0.05`, the offset the
   deleted bar measured empirically). Deficit contribution went 0.0000 (8/15 worlds)
   → 0.021–0.233 (15/15), formations 8 → 12 with all five collapse-only ones
   preserved; **zero calibrated blast radius** (goldens byte-identical, no pin moved),
   with the new `faction_trigger_reads_the_councils_own_mandate` pin as the runnable
   check. **Still queued from i379's rulings:** the `needs.social` utility term
   (Class 3b — needs the winner-decomposition census first). **The anger arms then
   landed (i383):** `emotions.anger > 0.50` was measured opening for **0.00–0.30% of
   agent-ticks** at BOTH of its sites — the abandonment shock (ORed with a live fear arm,
   hence a false affordance) and the anger→Work "aggressive productivity" emitter (anger
   the only gate, hence a dead producer) — while `fear > 0.5` beside it opened 15.2–38.8%
   and was left alone. Both anger arms now read `1.25 × the population's own mean anger`
   (`EMOTION_SHOCK_RATIO`, the i381 anomaly multiple); the emitter's priority is the
   excess over that bar. Live: anger-sourced Work goals 0 in 7 of 10 worlds →
   **0.020–0.184 per agent-tick in all 10**. One collapse golden re-anchored with mortality
   unchanged, calm golden byte-identical, no snapshot drift.
2. **A9 — the envelope at constant density** — re-scoped by i344: a **fidelity** policy
   (max co-location 19 → 4, contacted share halved), **not** throughput (+9.1%/−2.3%/+5.6%
   at N=96/144/192). A charter decision; do not sell it as speed.
3. **Clustered world generator — LANDED (i360).** Sim capacity already reached **N=256**
   (i359, `ENVELOPE_EXPANDED_2_7X`) but the i345 Vogel spiral collapsed the density world
   to **one settlement**; i360 places houses around `cluster_count_for = clamp(houses/16,
   1, 4)` centres (`world_gen.rs`), so N=192 → **3** and N=256 → **4** settlements at the
   natural gap, each with its own live polity holon (genesis memes/polity [2,2,1] / [6,4,2,1]).
   Calibrated range (<25 houses) is byte-identical (the ring/i345 path is untouched);
   golden 5/5, sim **299/299**. **The multi-settlement stack is now end-to-end LIVE
   (i362):** at 20K, **100% of genesis memes leak across a village boundary** (12/12 at
   N=192, 20/20 at N=256; 1018/2806 foreign host-links) via cross-polity trade, so
   geography → partition (i298) → per-polity holon (i297) → trade diffusion (i299) all
   fire at town scale. The **cap-raise open item is closed as refuted (i366):** the cap
   never binds in the reachable range (`(houses/16).clamp(1,4)` needs N ≥ 320, above
   `MAX_POPULATION = 256`), so the real knob is the **divisor** (a sweep to ~10–12 for
   5–6 villages is queued as an experiment, trading village size for count).
4. **Wealth tail — RESOLVED end-to-end (i363 + i370).** i363's council surplus dividend
   bent the Gini below baseline (`WEALTH_TAIL_BENT`, structural across six seeds per
   i365) but left a "residual hoard" anomaly (seed 23: 25 274). **i370 root-caused the
   anomaly and the old 31-vs-3 membership debt as ONE bug:** the revolution path cloned
   the whole faction roster into `council.members` — the taxed class — so the dividend
   equilibrium T* = reserve + inflow/share scaled with the roster (probe: seed 7, 1
   coup → 34 members, treasury 28 697 @50K). A coup now installs OFFICES (faction leader
   → Elder, top dominance/conscientiousness → Guard Captain + second seat; 3-office
   `default_institutions` shape), the rest return to villagers; post-fix seed 7:
   3/3/3 members, treasury 182.0. The i365 treasury-ceiling item is **closed as
   superseded** (the organic dividend suffices once the tax base is bounded);
   `revolution_is_regime_change_not_repeat_loop` re-contracted to pin the office
   hand-over (the old ≥5-membership pin asserted the bug itself).
5. **Closed:** `Idle` (i356 — the `Play` recreation driver made the last dead action
   live, 0.01%–3.67% of decisions) and `A8` `Wander` (i351). **Refuted/closed as
   premises:** locomotion pace (i357 — `Move` already steps one tile per tick), the
   dual-store *migration* (i353/i356 — redundancy, not a liveness fault), and the
   "N≥192 capacity" framing (i359 — capacity is fine; world structure is the blocker).
6. **§17 tier-gate residual — CLOSED (i372, charter decision: DELETE).** The dead
   `runs_full_biology()` / `runs_action_selection()` predicates are removed (zero call
   sites, i316; whole-gate payoff ≈5%, i328; Background dark in calm towns, i367). The
   *cognitive* rungs stay wired and pinned. A future biology LOD rung gets a real gate,
   not a false affordance.
7. **The sparse relationship store is DEMOTED, not queued** — i338 (ω(N²) is a housing
   artifact), i344 (world area is fidelity, not throughput), and i350 (the contacted graph
   re-saturates 13.3%→73.7% by 40K) all measured it as a ≤2× constant decaying toward 1.
   Do **not** build it; re-open only against new horizon evidence.
8. **Deferred with triggers** (do not pull early): `VecDeque<SimEvent>` — **i354 refuted the
   horizon trigger** (>250K-tick runs already work; i327 bounded the buffer), so its only
   remaining trigger is *jitter-free ticks*; recent-claims index (>250K ticks); H5
   founder-variance shaping (needs larger N **and** a coordinated re-anchor sweep);
   vendor-blocked era items (realms.md, resonance attestation, cult-liveliness).

## 9. Tone & Conduct

- No over-explaining infrastructure the operator already has. Concise relevance assessments.
- Code first; explanations only what was asked. Mark deliberate simplifications
  (`// ponytail:` comments naming ceiling + upgrade path).
- Every non-trivial logic change leaves at least one runnable check.
- Verification gates run automatically, unprompted, before every commit.
