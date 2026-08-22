# Plan — Scaling Foundation: From 100K to 200K+ LOC

*Created post-commit `e5cb5a9` (module-segregation refactor). Companion doctrine:
`AGENTS.md` §7. This plan owns the crate-extraction program; behavioral arcs
(Audit Phases 2–6) proceed in parallel and are never mixed into these commits.*

> **Baseline (post `e5cb5a9`, pre-S1):** 101,124 LOC total.
> `mindstrata-sim` = 69,655 LOC (69% of all source). Largest file
> `actions.rs` 2,398 (~880 production). 102 files still carry `use super::*`.
> Gates green: fmt clean · clippy workspace zero warnings · 1,023 sim units ·
> 304/0 tests-crate release · golden replay 8/8 vs stored baseline.

---

## 0. Problem statement

Doubling inside one crate puts ~140K LOC in a single compile unit:

- **Build-time coupling** — any edit recompiles the whole dependency cone; incremental
  wins evaporate as the crate grows.
- **No enforced dependency direction** — modules can reach into each other's internals;
  only convention (and grep) prevents `psychology` from poking `economy` privates.
- **Agent contention** — parallel sessions collide on shared files; merge-order risk
  grows with file fan-in.

File-level splitting (done) made the structure navigable. It does not fix any of the
three above. Crate boundaries do — cargo turns architecture into compile errors.

## 1. Non-negotiable principles (inherited from §3/§4 discipline)

1. **Golden replay is the referee.** Every extraction is a pure structural move proven
   byte-identical (`golden_replay_vs_baseline` 8/8) before commit. Never mix a
   behavioral change into an extraction commit.
2. **Shim transitions.** Old paths keep working via `pub use` re-exports during and
   after extraction (pattern: `person/mod.rs`). Zero call-site churn per step; churn is
   a separate mechanical commit if wanted at all.
3. **Coupling survey before surgery.** Wildcards must be settled BEFORE choosing seams
   (S1 → S2 ordering is not optional). Explicit imports are the evidence base.
4. **One system per iteration.** One extraction (or one wildcard sweep) per arc, ending
   in commit + push. No batching unrelated moves.
5. **Determinism pipeline stays explicit.** Cross-crate calls remain plain pipeline
   steps in fixed order inside `sim/core.rs`. RNG-stream discipline (§5) is untouched:
   draw order is a cross-crate contract once crates exist.

## 2. S1 — Wildcard settlement round 2 (the coupling survey)

**Scope:** 102 files under `biology/ psychology/ culture/ social/ noosphere/` plus root
modules still using `use super::*`. Tests keep idiomatic globs (`sim/tests.rs`,
integration suites).

**Procedure:** same scripted approach as round 1 (commit `e5cb5a9`):

1. Snapshot tree to `/tmp/opencode/s1_snapshot_<time>/` before touching anything.
2. Scripted rewrite per file: parse used identifiers (strip comments/doc-tests);
   lowercase module bindings → `use crate::<mod>;`; CamelCase items → `use super::{…}`
   or defining-module path; add missing trait imports (`rand::{Rng, SeedableRng}`,
   `itertools::*`) from compiler errors; iterate `cargo check` until zero.
   Known script traps (round-1 lessons): self-truncating write-backs, set-algebra that
   selects nothing, param renames colliding with method calls, `self` leaking into
   generated import lists, child-module names misrouted past `super::`.
3. `cargo fmt && cargo clippy --workspace` → expect ≤ current warning count.
4. Golden replay + full suite gates.
5. **Deliverable beyond green gates:** emit `docs/scaling/coupling_map.md` — for each
   subsystem directory, the sorted list of external symbols/modules it consumes. This
   map IS the seam proposal for S2/S3.

**Exit criteria:** zero `use super::*` in non-test sources; gates green; coupling map
committed.

## 3. S2 — First crate extraction: `mindstrata-person`

Why this one first: pure data + local dynamics (state types, constructors, inheritance
blends, plasticity math); depends only on `mindstrata-core`; no tick-pipeline calls of
its own; smallest blast radius while proving the recipe end-to-end.

**Contents:** `src/person/**` + `src/biology/**` (~10–12K LOC estimated).

**Steps (each gated):**

1. Create crate `crates/mindstrata-person` with `pub use` shim surface mirroring
   today's `crate::person::` / `crate::biology::` paths.
2. Move modules verbatim. Fix intra-crate imports only.
3. In `mindstrata-sim`: replace moved code with `use mindstrata_person as person;`-
style aliases so ALL existing paths compile unchanged (shim principle).
4. Gates: fmt/clippy/sim-units/tests-crate-release/golden.
5. Optional follow-up mechanical commit: migrate call sites to canonical crate paths,
   drop shims. Only after the identity proof lands.

**Serde note:** snapshot compatibility is part of golden (agent_hash). Renamed crate =
no change to serialized shape; do NOT touch `#[serde]` attributes or field order while
moving.

**Exit criteria:** `mindstrata-person` builds standalone (`cargo check -p`); golden
8/8; suite green; sim LOC drops accordingly.

## 4. S3+ — Cluster extractions (order set by the S1 map)

Indicative order (re-order freely if the coupling map disagrees):

| Crate | Contents | Why this cluster |
|---|---|---|
| `mindstrata-psych` | `psychology/**` (+ appraisal? map decides) | consumed by actions/appraisal, consumes person |
| `mindstrata-social` | `social/**`, `culture/**`, `noosphere/**` | large, mutually cohesive, downstream of psych |
| `mindstrata-institutions` | institutions/legal/diplomacy/military/theology/schools/norms/factions | governance cluster, mostly event-driven |
| `mindstrata-world` | world/world_gen/ecology/market/logistics/demography | environment layer |

**End state:** `mindstrata-sim` retains ONLY orchestration — tick pipeline, SystemContext
borrowing, RNG stream manager, command channel, snapshot I/O, metrics — target <15K LOC.
Dependency DAG enforced by cargo: `core ← person ← {psych} ← {social, institutions,
world} ← sim ← {tui, cli, render}` (exact edges from the map).

**Rules during S3:**

- One crate per iteration; shim-then-migrate two-step each time.
- If an extraction stalls on a genuine cyclic dependency, STOP — that cycle is a real
  architectural finding. Record it in the plan doc, resolve behaviorally (split the
  type or invert via trait) in its own iteration, then resume. Never break the DAG with
  a god-crate "for convenience".
- Trait objects stay out of hot paths (§6 static dispatch rule holds across crates).

## 5. Coordination protocol with behavioral arcs (B/C/D)

Behavioral work concentrates in `psychology/ social/ actions.rs` right now
(Iter-247 landed interoception→somatic_marker; Arc C targets ToM/social).

- Before starting an extraction touching a hot subtree: check `git log --oneline -3`
  and file mtimes (§7 etiquette). Sequence extractions INTO gaps between behavioral
  iterations; announce intent in the plan doc status blockquote.
- If a behavioral iteration lands mid-extraction: finish the extraction commit FIRST
  (it's provably neutral), rebase-free — the other session rebases nothing; both land
  linearly. Re-run gates after their landing regardless.
- `actions.rs` test-module extraction (production/tests split) waits until Arc B/C move
  off that file.

## 6. Continuous policies already active (do not regress)

- >2,500-line / >15-mixed-symbol trigger for module splits.
- Workspace lints warn-level; `deny(missing_docs)` per stabilized NEW crate from day
  one (cheap when the crate is born, expensive to retrofit).
- Type-state adoption on touched lifecycles only.
- Integration tests stay namespaced by domain; new subsystems get a matching suite dir.
- Every calibration re-anchor keeps naming measured-value/old-band/mechanism.

## 7. What we are NOT doing (anti-scope guard)

- No async runtime, no ECS framework adoption, no macro-generated registries — the
  fixed-tick explicit pipeline is the determinism guarantee; frameworks obscure it.
- No interface-with-one-implementation abstractions at crate seams; crates communicate
  through plain types/functions unless polymorphism is demonstrated need.
- No big-bang monorepo reshuffle; each phase leaves main greener than it found it.
- No moving `tests/` or benches until crates stabilize (they pin behavior across the
  whole workspace — that is their job).

---

## Status log

- Post `e5cb5a9`: baseline recorded (see top blockquote). S1 not started.
