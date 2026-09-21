# AGENTS.md — mindstrata Development Doctrine

**Read this file first. It encodes how we develop, verify, and commit. Follow it without being reminded.**

---

## 1. What This Project Is

`mindstrata` is a deterministic agent-based village simulation (Rust workspace) where emergent
social phenomena — factions, moral panics, marriages, revolutions, epidemics — arise from
coupled biological, psychological, and social subsystems. It is an R&D instrument, not a game:
every mechanism must be **live** (measurably influences behavior), every calibration must be
**probe-evidenced**, and every iteration must leave the suite **greener than it found it**.

Architecture docs: `docs/architecture/archive/AP1-implemented.md` (what exists),
`docs/architecture/AP2.md` (the spec). Deepening program: `docs/PLAN_BIO_PSYCH_DEEPENING.md`.
Audit methodology: `docs/AUDIT_2026-08-22_EMERGENT_REALISM.md`.

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

- `cargo fmt --all && cargo clippy --workspace --quiet && cargo test -p mindstrata-tests --lib --release`
  must run before EVERY commit. No exceptions, no reminders. Behavioral iterations
  additionally run `scripts/gate --full` before push — the pre-commit hook covers only
  fmt+clippy; the release suite + golden baselines are enforced at push time.
- Full suite is release-mode (`--release`); debug-mode runs of long-horizon tests take 10–20×.
- Snapshot drift is reviewed via `cargo insta test -p mindstrata-tests --release`, then
  `cargo insta accept --all` **only with documented evidence** of why the shift is expected.
- The canonical success signal is **observable output** (probes, rendered artifacts, test
  runs), never source diffs or intent.
- Stale binaries produce fix-less reproductions: rebuild before re-auditing.

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
- **RNG stream discipline**: birth-path constructors consume draws in field order;
  `Personality::inherit` consumes exactly one draw per trait to preserve alignment.
  Different range widths consume different byte counts — count-alignment ≠ byte-alignment.
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
- `#![deny(missing_docs)]` is aspirational for `mindstrata-core`; don't enable it
  workspace-wide while public surfaces still churn.

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

### Current layout (post crate-ladder, `f66b988`+`3ad212b`; refresh me when structure moves)

```
crates/
  mindstrata-core/          # Fixed, ids, clock, events, rng, parameters, propositions
  mindstrata-person/        # person/ aggregate + biology/ + health        [leaf]
  mindstrata-psych/         # psychology/ + appraisal + memory/attention/
                            # belief_update/journal                        [leaf]
  mindstrata-institutions/  # institutions/legal/diplomacy/military/theology/
                            # schools/norms/factions types (pure domain)   [leaf]
  mindstrata-social/        # social/ culture/ noosphere/ + gossip/conflict
  mindstrata-world/         # world/world_gen/ecology/market/logistics/
                            # demography/black_market
  mindstrata-sim/           # ORCHESTRATION ONLY (target <15K):
    sim/mod.rs              #   Simulation struct, AgentBundle, wiring
    sim/core.rs             #   tick() pipeline order
    sim/pass_*.rs           #   six verbatim tick passes
    sim/{population,api}.rs #   constructors/seeding; command channel
    sim/*_impl.rs + {household,economy,births_deaths,...}.rs
                            #   impl-Simulation glue (Arc-D detangle target)
    actions/{mod,tests}.rs  #   action-selection engine (sits above domains)
    {routines,scheduler,snapshot,scenario,spec_lint,agent_tier,
     provenance,population_cap,mods}.rs  # infra
    legacy shims in lib.rs preserve pre-extraction crate:: paths
crates/mindstrata-tests/src/integration_tests/{biology,psychology,social,
  culture,governance,economy,legal,infra}.rs
crates/mindstrata-tui/src/{lib,render,session}.rs
```

DAG (cargo-enforced): core ← person ← psych ← social; person ← institutions;
person/institutions ← world; all five ← sim ← tui/cli/render/tests/benches.

### The module → crate ladder (scaling to 200K+ LOC)

File splits fix navigability; only **crate boundaries** fix build-time coupling and
dependency direction. The ladder, one rung per iteration:

1. **S1 — settle remaining wildcards** (~102 files in `biology/ psychology/ culture/
   social/ noosphere/` + roots). Explicit imports double as the coupling survey.
2. **S2 — extract `mindstrata-person`** (+ `biology/`) as the first leaf crate;
   proves the recipe on the smallest blast radius.
3. **S3+ — cluster extractions** (`mindstrata-psych`, `mindstrata-social`,
   `mindstrata-institutions`, `mindstrata-world`) ordered by the S1 coupling map.
   End state: `sim` = pure orchestration (<15K LOC), dependency DAG enforced by cargo.

Crate-extraction discipline (full procedure in `docs/PLAN_SCALING_FOUNDATION.md`):

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

**The live queue is `docs/PLAN_DC3_DEVELOPMENT.md`** (its §3.1 is the calibration-debt
ledger, its execution ledgers are the iteration trail; per-iteration probes and measured
verdicts live in `docs/architecture/AP4-studio/evidence/`). Keep that file current and
treat this section as a pointer, not a second ledger — it went stale once by holding a
queue from Iteration 247 while DC-3 and DC-4 landed.

Landed since this section was last written (see `git log` for the trail): the full
module-segregation refactor, the **crate ladder** (S1–S3: `core ← person ← psych ←
{social, institutions, world} ← sim`, DAG cargo-enforced, every extraction golden-proven
byte-identical, new crates born with `deny(missing_docs)`), **DC-3 complete at 4/4 legs**
(Era IV holon, multi-village UM-3, N≥48 perf budget + `scripts/gate` perf envelopes, asset
pipeline v0), and **DC-4 opening** (difficulty-levers rows 2–3 live, pathology/goal-gate
bands, meaning reflex and physiological-reflex repairs, TUI asset/scene viewers, and the
scale program: i330–i345 removed every accidental relationship scan, re-ranked the scale
levers on measurement, and fixed the housing governor).

Live queue, in order (evidence link per item):

1. **The sparse relationship store with stable identity** — i341/i344 confirmed it is the
   *only* remaining lever on the tick's dominant cost class (~58% of the tick walks N(N−1)
   rows while 8–11% carry state; contact volume is flat and world size does not help).
   Constraints already measured: i337 disqualified a positional index (positions are reused
   by births/deaths), and it carries a §4.4 re-contract of
   `relationship_store_is_complete_at_populate` plus a full re-anchor sweep. Probe first.
2. **A9 — the envelope at constant density** — re-scoped by i344: a **fidelity** policy
   (max co-location 19 → 4, contacted share halved), **not** throughput (+9.1%/−2.3%/+5.6%
   at N=96/144/192). A charter decision; do not sell it as speed.
3. **A8 — locomotion is a dead producer** (`Wander`/`Move` 0.00% of agent-ticks; positions
   frozen). §4.3 class: decide the semantics, revive the producer, expect a behavioural
   sweep. Read `i338`/`i340` first — space is currently decorative.
4. **§17 tier gate reads the same non-varying `relationship_count`** that made the i342
   social-count proxy a dead channel; i328 measured the tier-gate payoff at ≈5%, so this is
   a correctness item, not a perf one.
5. **Deferred with triggers** (do not pull early): `VecDeque<SimEvent>` (>250K-tick
   horizons), recent-claims index (same), H5 founder-variance shaping (needs larger N **and**
   a coordinated re-anchor sweep), vendor-blocked era items (realms.md, resonance
   attestation, cult-liveliness).

## 9. Tone & Conduct

- No over-explaining infrastructure the operator already has. Concise relevance assessments.
- Code first; explanations only what was asked. Mark deliberate simplifications
  (`// ponytail:` comments naming ceiling + upgrade path).
- Every non-trivial logic change leaves at least one runnable check.
- Verification gates run automatically, unprompted, before every commit.
