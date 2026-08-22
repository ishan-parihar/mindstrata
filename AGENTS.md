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
  must run before EVERY commit. No exceptions, no reminders.
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
     flipping TRANSIENT↔ENDemic nine times), record it as systemic debt in the ledger rather
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

## 6. Hierarchical Module Splitting (the scaling foundation)

As systems grow, files MUST split hierarchically — a god-file cannot be developed safely by
parallel agents or reviewed precisely. The pattern established by the sim.rs split (17,942
lines → `sim/` directory):

### When to split

- A file exceeds ~2,500 lines or ~15 top-level symbols of mixed concern.
- Two agents need to work the same file concurrently (split FIRST, then parallelize).
- A pass/system has its own data structures and lifecycle inside a shared impl.

### How to split (proven procedure)

1. **One domain per module**: `sim/family.rs` (marriage/birth/kinship), `sim/economy.rs`,
   `sim/norms_impl.rs`… name by domain, not by layer.
2. **The struct stays in `mod.rs`** (`Simulation`, `AgentBundle`): state definitions live at
   the root; behavior lives in domain modules as `impl Simulation` blocks using
   `pub(super)` visibility for internal steps.
3. **Move code verbatim** — a split is a pure refactor proven by **byte-identical golden**
   runs and an unchanged suite. Never mix behavioral changes into a split commit.
4. **Tests follow their subject**: `sim/tests.rs` for unit-level, per-domain test modules in
   `integration_tests/{economy,governance,psychology,social,…}/`. Namespaced paths make
   failure triage instant.
5. **Wildcards are transitional**: `use super::*` in fresh impl modules is tolerated during
   migration but is cleanup debt — settle to explicit imports once the module stabilizes.

### Current layout (post-split)

```
crates/mindstrata-sim/src/
  sim/mod.rs        # Simulation struct, AgentBundle, config, wiring
  sim/core.rs       # main tick() pipeline orchestration (largest; further splits expected)
  sim/{family,economy,norms_impl,factions_impl,institutions_impl,legal_impl,
       diplomacy_impl,education,cults_noosphere,memory_ops,social_cluster,
       clans,snapshot_metrics,tests}.rs
  biology/          # genome, metabolism, immune, thermal, nervous, … (EmbodiedState)
  psychology/       # emotions, theory_of_mind, moral_cognition, interoception, …
  systems/          # future home for extracted pass modules
crates/mindstrata-tests/src/integration_tests/
  {economy,governance,psychology,social,…}/   # namespaced integration suites
```

### Splitting etiquette for parallel agents

- Check `git log --oneline -3` and `git status` BEFORE editing; another agent may have
  landed mid-flight changes (this happened: a struct edit survived only because the
  splitter migrated uncommitted content).
- Never edit files another agent is actively migrating (watch mtimes); poll compile state
  instead of force-fixing a half-written file.
- Do not create branches/worktrees unless explicitly asked; work lands linearly on main.
- Commit boundaries belong to the iteration owner; if your edit rides inside someone's
  uncommitted file, note it honestly in the commit message.

## 7. Where Things Stand / Next Work

Landed through Iteration 246 (see `git log` for the full trail):
faction crisis-pressure lifecycle, lived-experience belief charging, health-sync revival +
fertility restoration, genome blending + dead-predisposition activation (Arc A part 1),
family surnames (part 2), vertical moral-values transmission (part 3).

Queue, in order:

1. **Iteration 243 — six single-system calibrations** (suite currently 298 passed / 6 failed):
   famine timing vs plague mortality, patronage rank→status margin, flashbulb memory gate,
   motivation fear context floor, neural prediction-error gate, noosphere conviction margin.
   Each is independent; probe → fix producer or re-anchor with evidence.
2. **Arc B — embodiment→mind**: interoception activation (`felt_*` filters exist in
   `psychology/interoception.rs` but have ZERO call sites in decision pipelines — the
   Genome→Endocrine→Nervous→Interoception→Emotion chain breaks at link 4); then Whitehall
   gradient (status→chronic stress→health) and sleep-debt withdrawal effects.
3. **Arc C — mind→social depth**: ToM-driven partner choice, speech-act intent/deception
   feeding trust.
4. **Arc D — infrastructure**: extract bio/psych passes from `sim/core.rs` into `systems/`
   (byte-identity-guarded), lineage/emotion/Gini metrics + TUI longitudinal charts.
5. Then resume audit Phases 2–6 in `docs/AUDIT_2026-08-22_EMERGENT_REALISM.md`.

## 8. Tone & Conduct

- No over-explaining infrastructure the operator already has. Concise relevance assessments.
- Code first; explanations only what was asked. Mark deliberate simplifications
  (`// ponytail:` comments naming ceiling + upgrade path).
- Every non-trivial logic change leaves at least one runnable check.
- Verification gates run automatically, unprompted, before every commit.
