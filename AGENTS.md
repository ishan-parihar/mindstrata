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
   and Gini band unchanged).

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
    sim/{population,api}.rs #   constructors/seeding; command channel
    sim/*_impl.rs + {household,economy,births_deaths,marriage,clans,
      cults_noosphere,memory_ops,norms_impl,social_cluster,...}.rs
                            #   impl-Simulation domain glue (detangle target)
    actions/{mod,tests}.rs  #   action-selection engine (sits above domains)
    {routines,scheduler,snapshot,scenario,spec_lint,agent_tier,
     provenance,population_cap,mods,assets}.rs  # infra
    legacy shims in lib.rs preserve pre-extraction crate:: paths
  mindstrata-tests/         # integration_tests/{biology,psychology,social,culture,
                            #   governance,economy,legal,infra}/ + snapshots/golden
  mindstrata-tui/           # {lib,render,session,assets_view,scene}.rs
  mindstrata-cli/ mindstrata-render/ mindstrata-benches/  # entry points, 186 probes
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
- **`docs/PLAN_DC3_DEVELOPMENT.md`** — the live work queue (§3.1 calibration-debt ledger;
  §7/§9 execution ledgers).
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

Live queue, in order (evidence link per item):

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
   a +0.06–0.08 offset; and the panic channel's firing density halved (3/5 → 3/10 swept
   seeds) as the trust field de-saturated — its residual dependence on the wide trust
   range is its own iteration, not a pin to move.
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
