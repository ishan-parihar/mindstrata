---
name: plan-rust-craft
status: HISTORICAL
description: "Rust-craft audit (2026-09-24) against the rust-best-practices handbook, plus its fix ladder (C1-C5). Owns the measured clippy --all-targets inventory, the broken i286 census probe, the lint-inheritance hole in mindstrata-development, and the gate-closure iteration. Owns no engine behaviour (that is PLAN_DC5); every item here is craft, tooling, or citation integrity."
type: Plan
reconciled_commit: d557f2c
created: 2026-09-24
---

# Mindstrata — Rust craft audit & fix ladder (2026-09-24)

> **What this is.** A full-workspace audit against the rust-best-practices handbook
> (9 chapters), every finding measured, turned into a doctrine-compliant fix ladder.
> **What it is not.** Not an engine-behaviour plan (`PLAN_DC5_DEVELOPMENT.md` owns that)
> and not a licence to churn working code — AGENTS.md §6 governs: "fix opportunistically,
> never churn a file just for this."

---

## 0. Method & evidence base

- **Instruments.** `cargo clippy --workspace --all-targets` (default pass, then a
  `--keep-going` pass for the full inventory) with clippy 0.1.98 / rustc 1.98.0; a
  workspace-lint simulation for `mindstrata-development`; pattern censuses
  (unwrap/expect/allow/panic/wildcard/shim) with `#[cfg(test)]`-boundary classification
  so in-file test modules are not counted as production; duplicate/complexity metrics
  from a non-LSP analyzer (pointers only, never verdicts).
- **Tree state at measurement.** `6d34c91` **plus 8 in-flight files** of the i393
  speech-act migration (`core/src/parameters.rs`, `sim/src/sim/{memory_ops,pass_social,
  social_cluster}.rs`, `sim/src/sim/tests/culture.rs`, `sim/src/systems/cognitive.rs`,
  `social/src/social/{interaction,speech_act}.rs`). The one lib-target warning
  attributed to `social` below was a transient mid-edit state; the current content lints
  clean (verified exit 0). Exact counts in the test-cfg tail may shift by a few —
  **C1 step 0 is a re-measure after that migration lands.** Every other finding is a
  committed-state fact (its files are not in the dirty set).
- **In-flight caveat settled + C1 step-0 re-measure (2026-09-24).** Two corrections,
  both measured: (a) `git diff HEAD` on the migration's `interaction.rs` shows `params`
  was used at HEAD (`params.social_reciprocal_factor` is deleted *by* the in-flight
  edit), so the one transient lib warning was introduced mid-flight — `6d34c91` itself
  is clean; there is no committed-state warm-cache vacuity, and F4's exit-code form is
  robustness, not the repair of a proven false green. (b) The re-measure on the
  in-flight tree reads **339 warning sites across 147 files** (+F1's error), of which
  **4 are owned by the in-flight migration and stay with it**: `core/src/parameters.rs`
  ×1 and `sim/src/sim/tests/mod.rs` ×2 (its own files), plus `sim/src/sim/core.rs`'s
  `rel_pos` dead-code — at HEAD its last production callers were
  `social_cluster.rs:614,617,894`, removed by the migration's working tree. C1 clears
  the remaining ~335 and verifies its exit criterion on the **committed** state (clean
  `git archive` export of the C1 commit), never by touching the in-flight files.
- **Reproduction.** `cargo clippy --workspace --all-targets --keep-going`; per-finding
  commands are named inline. Raw counts below are this audit's premises in the
  DC-5 style: an iteration that moves one must move this file.

---

## 1. Per-chapter verdicts

| Handbook chapter | Verdict | Measured basis |
|---|---|---|
| 1 — idioms (borrow/Copy/Option) | mostly clean; mechanical lint tail | inventory is ch1-class: `explicit_iter_loop` 64, `cloned_instead_of_copied` 37, `map_unwrap_or` 17, `ptr_arg` 6, `clone_on_copy` 4 — all in test-cfg or probe code |
| 2 — clippy & linting | **two holes (F2, F3)** | gate step omits `--all-targets`; `mindstrata-development` does not inherit `[lints]` |
| 3 — performance | clean | `redundant_clone`/`needless_collect` enabled workspace-wide and lib-clean; per-tick clone ban and perf budgets are already doctrine |
| 4 — error handling | essentially clean | 6 true production `unwrap`/`expect` sites, 4 already conformant invariant-`expect`s; 0 `panic!`/`todo!` in production; `thiserror` in core + sim |
| 5 — testing | strong | descriptive names doctrine-mandated; `cargo insta` adopted; doc tests present; multi-assert pins are the calibration idiom (deliberate DAMP divergence, no action) |
| 6 — dispatch | exemplary | 0 `dyn Trait` in production code |
| 7 — type-state | policy held | incremental-on-touched-code rule already in AGENTS.md §6; no retrofits |
| 8 — comments & docs | mostly clean | 0 TODO/FIXME in production src (tracked in plan docs per doctrine); `deny(missing_docs)` in 6 leaves; 2 module carve-outs (F5); `doc_lazy_continuation` ×5 in examples |
| 9 — pointers & threads | exemplary | 0 `Rc`/`RefCell`/`Cell` in production; `unsafe_code = "forbid"` workspace-wide; single-threaded deterministic |

---

## 2. Findings (measured, 2026-09-24)

**F1 — a retired probe trips the clippy gate (deny-by-default lint); its citation survives.**
`crates/mindstrata-benches/examples/i286_norm_proposal_census.rs:44`:
`holders.len().max(claims.min(0)) + holders.len() * 0 + holders.len()` trips
`clippy::erasing_op` (`* 0`) — a deny-by-default **clippy** lint, so `cargo clippy`
cannot compile it. Correction at C1 step 0: plain rustc accepts the expression —
`cargo build --example i286_norm_proposal_census` exits 0 (verified 2026-09-24) — so
the census **did run** in i286 and `evidence/i286_norm_proposals.md:33`'s numbers are
honestly attributed; the audit's "ghost citation" concern is retracted. What remains:
single-commit history (`5ee3c1c`), never edited since, verdict superseded (vivo + i288 +
i292), sibling `i286_diag` already "deleted after use" — dead code that blocks the C1
gate flip, so it goes the way its sibling went, and the evidence doc's instrument list
is annotated in the same commit (§4.14 citation discipline).

**F2 — the gate's clippy step omits `--all-targets` (recorded gap, now measured).**
AGENTS.md §3 records "~14 existing `mindstrata-tests` test-cfg warnings". Measured
today: **1 hard error (F1) + 340 warnings across 122 targets**, none of which the gate
can see:
- 273 warnings in 113 benches-example targets (probes);
- 66 in the `(lib test)` targets of 8 crates: sim 21, tests 18, social 8 (mid-edit
  file), tui 8, person 5, development 4, core 1, render 1;
- 1 transient production-lib warning (the in-flight `params` unused variable —
  already gone in current content).
Top lints: `explicit_iter_loop` 64, `cloned_instead_of_copied` 37,
`uninlined_format_args` 18, `map_unwrap_or` 17, `redundant_closure_for_method_calls` 16,
`unnecessary_cast` 16, `field_reassign_with_default` 11, `many_single_char_names` 11 —
a mechanical, mostly machine-applicable pile (`cargo clippy --fix` suggestions exist
for the majority). The "~14" number in AGENTS.md §3 is stale and moves with C1.
Re-measured at C1 step 0 (§0): 339 sites / 147 files on the in-flight tree — the delta
vs the audit count is the migration's own new in-flight code, which lands with its
warnings, not with C1.

**C1 landing (2026-09-24):** cleared to zero. `cargo clippy --workspace --all-targets
--locked -- -D warnings` exits 0 — **0 warnings, 0 errors across every target** (warm
run 39 s; the C1 commit is additionally verified cold on a clean `git archive`
export, §5). The 339 sites resolved as: 105 machine fixes (`cargo clippy --fix`),
the rest hand-cleared in a six-slice sweep plus mop-up; the 4 in-flight sites named
in §0 were resolved inside the migration's own working tree — its two still-dirty
files (`core/src/parameters.rs`, `sim/src/sim/tests/mod.rs`) are excluded from the
C1 commit and land with their owner. Two instrument facts recorded for the next
sweep: `erasing_op` is clippy-only (F1's correction — `cargo run` never runs lints),
and clippy 1.98's `items_after_test_module` suggestion is machine-applicable
(it moved the `collective_memory` items itself during the --fix pass).
Load-bound floor note: the archive export's step-6 `i270 --quick` read tps 2921–6396
against the 8000 floor while a concurrent session built on the same host; the
controlled A/B — 6d34c91 (pre-C1) tps 6365 vs post-C1 6396, Δ < 1 %, both under the
same load — proves the shortfall is machine contention, not C1. Every other step was
green on the committed state: cold `--all-targets -D warnings` exit 0 (52 s), fmt OK,
bench law 0 violations, warn-only budgets 117.5 µs/tick @ N=12 (budget 150), seed
sweep FAMILY_PASS 12/12, full release suite **314/0/1** in 201 s.

**F3 — `mindstrata-development` is the only crate not inheriting workspace lints.**
12 of 13 crates carry `[lints] workspace = true`; development does not, so `pedantic`,
`redundant_clone`, `needless_collect`, `large_enum_variant` never run on it. Measured
impact of adding the line: **0 new warnings** (simulated with the workspace-equivalent
lint set). A zero-cost standards hole.

**F4 — the gate's clippy step is grep-on-output instead of `-D warnings`.**
`scripts/gate` pipes `cargo clippy --workspace --quiet` into `grep -E "^warning|^error"`.
It works, but it depends on cargo's output shape rather than the compiler's exit code.
`-- -D warnings` is the robust form: a unit that fails is never cached, so the verdict
re-surfaces on every run regardless of cache warmth, and errors and warnings are
uniform. Fold into the same gate edit as F2. The versioned pre-commit hook
(`scripts/githooks/pre-commit`, via `core.hooksPath`) carries the same grep form
**plus a staging bug**: its `case "$STAGED" in *.rs)` matches the whole
newline-joined string, so a mixed `.rs`+`.md` staging whose *last* file is not `.rs`
skips the checks entirely. C1 rewrites both.

**F5 — suppression hygiene: 12 bare `#[allow]`, 2 module doc carve-outs.**
- `core/src/fixed.rs:24` — `#[allow(cast_sign_loss, float_cmp,
  cast_possible_truncation, cast_precision_loss)]` on `Fixed`: all four lints are
  already allowed at `[workspace.lints.clippy]` level → the attribute is dead weight;
  **delete**.
- `psych/psychology/development_field.rs:64`, `sim/sim/core.rs:323` — `#[allow(...,
  reason = "...")]` with genuine why-comments; **convert to `#[expect]`** so they
  self-clean (the `reason` carries over).
- `tui/src/render.rs:1163` — `#[allow(unreachable_patterns)]` on an exhaustive
  identity match; **convert to `#[expect]`**: it fires today, and stops firing (loudly)
  the day a new `IdentityKind` variant makes the wildcard reachable — exactly the alert
  wanted.
- `tests/src/comparison.rs` — 6× `#[allow(dead_code)]` on `pub` items; likely vestigial
  (pub items in the lib target are not dead) — verify, then delete the dead ones or the
  whole module if nothing consumes it.
- `benches/examples/{i261_gini,i287_institution_shift}.rs` — `#[allow(dead_code)]` on
  appended debug helpers; call them or delete them.
- 14 existing `#[expect]`s are in active use — the pattern is established.
- `social/src/gossip.rs:1` and `social/src/culture/knowledge.rs:9` carry module-level
  `#![allow(missing_docs)]` **inside a `deny(missing_docs)` crate**: document those two
  modules (preferred) or keep the allow with a reason comment.

**F6 — error handling: 6 true production `unwrap`/`expect` sites; 4 already conformant.**
Census with cfg(test) boundaries: 208 `unwrap()` workspace-wide, of which **6** are in
production code:
- `sim/src/sim/institutions_impl.rs:415,433` — `sort_by(|a,b|
  a.partial_cmp(b).unwrap())` on `Fixed::to_f64()` values: infallible (fixed-point has
  no NaN) but the idiom is wrong; **`total_cmp`** is the correct tool (also faster).
- `sim/src/sim/population.rs:457,461` — `.expect("seeded site present")`: real
  invariant, message present — **conformant, keep**.
- `development/src/polarity.rs:391,425` — `LineId::new("cognitive").expect("cognitive
  line is in registry")`: registry invariant with message and fallback comment —
  **conformant, keep**.
Also: 0 `todo!`/`unimplemented!` anywhere, 0 production `panic!` (all five `panic!`
sites are test/probe match arms), `thiserror` types in core (`error.rs`) and sim.

**F7 — dispatch/pointers: nothing to fix.** 0 `dyn Trait`, 0 `Rc`/`RefCell`/`Cell` in
production code; `unsafe_code = "forbid"` workspace-wide.

**F8 — duplication: two Rule-of-Three candidates; the rest is deliberate.**
4492 duplicated lines (2.9% of 154K) across 131 groups. The largest groups are
benches-example scaffolding (e.g. a 98-line harness ×5 probes) — **deliberately not
queued**: probes are self-contained evidence artifacts (IC-4; handbook §1.8 test-code
DAMP rule — extraction couples frozen evidence to a changing harness). Below the
Rule-of-Three (in-file ×2 repeats in household.rs, education.rs, world.rs,
institutions_impl.rs, cli main.rs): leave (§1.8 — duplication is cheaper than the wrong
abstraction). Two groups cross the threshold and share one decision:
- `social/src/conflict.rs:151-168` == `social/src/gossip.rs:494-511` ==
  `social/src/social/epistemic.rs:391-408` (71 lines ×3 files, same crate);
- `sim/src/snapshot.rs:577-589` == `social/src/social/group_formation.rs` ×5 blocks
  (54 lines, cross-crate).
Extract **only if** reading confirms it is one shared decision, as a pure-refactor
iteration with **byte-identical golden replay** as the proof (§7 discipline).

**F9 — complexity: 193 functions ≥ 10 cyclomatic complexity; worst 130.**
`sim/src/sim/household.rs:106 tick_kinship_household_daily` (cc 130) is the named top
target. This is not a new program: AGENTS.md §7 already tracks the `*_impl` glue
detangle as infrastructure work in its own iterations — fold these numbers into that
queue as its measured justification.

**F10 — recorded debt re-quantified (policy unchanged, counts current).**
157 `use super::*` wildcards (transitional; settle on touched files per §7); 32 legacy
re-export shim modules in `sim/src/lib.rs` (§7: keep until call-site churn, "or
nothing"); `#![allow(missing_docs)]` at core and sim crate roots (doctrine §6:
deliberate until public surfaces settle); no `rustfmt.toml` (stable `cargo fmt` only —
deliberate; nightly `group_imports` would reformat 151K lines for zero behaviour).

**F11 — dead-code pointers, not verdicts.** The analyzer flags 198 items, of which 113
are test-only-usage (legitimate pub test helpers). True dead-field auditing is owned by
the i391 instrument (`scripts/field_census.py`, cross-module-read rule) — run it on any
suspicion raised here; never mass-delete off this list.

---

## 3. The fix ladder (C1–C5)

Each iteration is one root cause, ends in `scripts/gate --full` green, and cites its
re-measured numbers here. Structural-only changes must produce **byte-identical
goldens**.

**C1 — gate closure (the AGENTS.md §3 queued tooling iteration). — LANDED 2026-09-24.**
1. *Step 0:* re-measured `cargo clippy --workspace --all-targets` (339 sites / 147 files,
   4 owned by the in-flight i401 migration — see §0); F2's counts refreshed there.
2. F1 resolved by **deletion**: `i286_norm_proposal_census.rs` removed (verdict
   superseded by vivo + i288 + i292; sibling `i286_diag` already went the same way),
   with `evidence/i286_norm_proposals.md`'s instrument list annotated (§4.14). Note the
   correction recorded in §0: plain rustc accepted the `erasing_op` expression, so the
   census *did* run and its numbers were honestly attributed — the deletion is on
   dead-code grounds, not citation-integrity grounds.
3. `[lints] workspace = true` added to `mindstrata-development` (F3; measured 0 new
   warnings post-migration, as simulated).
4. The `(lib test)` warnings cleared — cfg(test)-only surface, zero production
   behaviour.
5. The example warnings cleared — mechanical (`explicit_iter_loop`,
   `cloned_instead_of_copied`, `uninlined_format_args`, `field_reassign_with_default`,
   `doc_lazy_continuation`, …), probe measurement behaviour unchanged. One
   machine-applied `--fix` needed hand-repair: `i387_utility_decomposition` had its
   `*winner` deref left behind after the fix hoisted the value into a local (E0614) —
   exactly the hazard of landing an auto-fix without an exit-code gate, and the reason
   step 6 is the point of the iteration.
6. Gate step + versioned pre-commit hook flipped to `cargo clippy --workspace
   --all-targets --locked -- -D warnings` (exit code, not grep-on-output), and the
   AGENTS.md §3 known-gap paragraph amended to the closed form.
*Exit:* measured — `cargo clippy --workspace --all-targets --locked -- -D warnings`
**exits 0** on the C1 tree (verified cold and warm); gate green; goldens byte-identical
(structural-only: no production-code behaviour changed, every fixed site is cfg(test),
a probe example, or a doc comment).

**C2 — suppression hygiene (F5).** Delete the redundant `Fixed` allow; convert the two
reason-carrying allows and the tui `unreachable_patterns` to `#[expect]`; resolve the
tests/benches `dead_code` allows by use-or-delete; document or de-carve the two social
modules. *Exit:* `grep '#\[allow(' crates` returns only criterion-justified harness
allows (e.g. the criterion `missing_docs` ones, which carry reasons).

**C2 landing (2026-09-24):** done, with three corrections the execution measured:
(a) the `sim/sim/core.rs` `needless_range_loop` allow was a DEAD suppression — the
lint does not fire on that loop, so the attribute is simply deleted (the `#[expect]`
conversion errored as unfulfilled, which is the mechanism doing its job);
(b) `i261_gini.rs`'s `council_debug` is CALLED — first statement of `main` — so the
audit's "dead helper" label was wrong; only the vestigial allow is deleted, the
function stays; (c) the two social `#![allow(missing_docs)]` carve-outs were STALE —
zero missing-docs diagnostics surface without them; the modules were already
documented and the attributes are removed. The `comparison.rs` module (§19.5.J
harness) is consumed by nothing (`#[cfg(test)] mod` with no importers) and is deleted
whole — F5's own prescription; its 3 self-tests leave the suite (314 → 311), a
dead-code deletion, not a re-anchor. *Exit state:* the remaining `#[allow]`s are
exactly the criterion harness pair (`benches/{subsystems,tick_loop}.rs`,
reason-commented) and the two doctrine-recorded crate-root `#![allow(missing_docs)]`
in core + sim (F10 — deliberate until public surfaces settle); the strict exit
sentence above is amended by this record.

**C3 — error-handling polish (F6).** The two `institutions_impl` sort sites move to
`total_cmp` (identical ordering for fixed-point values, no Option, faster); the four
conformant `expect`s stay. *Exit:* zero `unwrap()` in production sort paths; gate green.
**C3 landing (2026-09-24):** landed — `:415` → `sort_by(f64::total_cmp)` (canonical path
form) and the tuple sort at `:433` → `total_cmp` closure. Proof: full tests-crate
release suite **311/0/1** with goldens byte-identical; `clippy --all-targets -D
warnings` exit 0. Production `sort_by(partial_cmp().unwrap())` count: zero.

**C4 — Rule-of-Three extractions (F8), pure refactor only.** Confirm the two candidate
groups are one decision each, extract, prove **byte-identical golden replay**. If
reading shows they are coincidental duplication, record the rejection here instead
(§1.8 — a recorded rejection is a landing).
**C4 landing (2026-09-24): both groups REJECTED as coincidental — recorded, per the
§1.8 rule this plan cites.** Reading both: (1) the 71-line trio is `make_personality()`
— three `#[cfg(test)]` fixtures each constructing an all-0.5 `Personality` in
`conflict.rs`/`gossip.rs`/`epistemic.rs`'s own test modules; (2) the 54-line group is
`make_snapshot_with_active_group()` — a `GroupCandidate` test fixture in `snapshot.rs`'s
test module vs the same literal ×5 in `group_formation.rs`'s tests, **cross-crate**.
Both are test-fixture DAMP duplication (the class F8 already exempts for probes):
extraction would couple independent test modules — across crates, for group 2 — to one
fixture, and the shared fixture forks the moment one domain's tests want a different
profile: the wrong abstraction. No code changed; the analyzer pointers are retired.

**C5 — complexity debt joins the existing queue (F9).** `household.rs` splits under the
§7 procedure when its iteration comes; this plan adds only the measured numbers.
**C5 landing (2026-09-24):** the measured numbers live in F9 (193 fns ≥ 10 cc; worst
`tick_kinship_household_daily` cc 130) and the detangle queue is owned by AGENTS.md §7
+ PLAN_DC5_DEVELOPMENT.md; the PLAN_DC5 row addition itself is deferred to the next doc
reconciliation (PLAN_DC5 is mid-flight in the concurrent session's staging). **The
ladder C1–C5 is complete; this plan is closed.**

---

## 4. Explicit non-goals (each doctrine-anchored)

- **No mass unwrap churn** — six sites exist; four are already the handbook's preferred
  form.
- **No nightly rustfmt / import regrouping** — 151K-line reformat, zero behaviour.
- **No probe-scaffold deduplication** — DAMP over DRY for evidence artifacts (F8).
- **No `missing_docs` enable in core/sim** — doctrine §6 defers until surfaces settle.
- **No dead-code mass deletion** — i391's `field_census.py` instrument owns verdicts.
- **Shims and wildcards stay** — §7 policy; counts recorded in F10.

## 5. Verification law

Every C-iteration: probe first where behaviour is touched (C1's probe census, C4's
golden replay), `scripts/gate --full` green, byte-identical goldens for structural
changes, and this file's numbers updated in the landing commit (§4 reconciliation
rule). C1 additionally owns the doctrine edit that closes the recorded gate gap.
