# Execution Runbook — Iterations 266 onward

*Created post-Iteration-265 (commit `3705dc9`), tree green (307 passed / 0 failed,
clippy 0). Governing arc: **AP3 — Attractor-Field Architecture**
(`docs/architecture/AP3-afa/PLAN.md`). This runbook is subordinate to that package:
`AP3-afa/04-waves.md` remains the authoritative dispatch plan and file-ownership
ledger; this document adds the iteration-by-iteration schedule, per-iteration
verification contract, and coordination protocol.*

---

## 0. Where things stand

- Bio/psych deepening program: CLOSED (Iter-259). Audit roadmap Phases 2–6: LANDED.
  Infrastructure queue: DRAINED. Handoff polish tier: DONE (Iter-264 TUI dossier,
  Iter-265 annals provenance).
- Propaganda emergence landed (Iter-263): institution-owned runtime producer;
  long-horizon refill verified.
- H5 founder-distribution shaping: CLOSED AS DEBT (AGENTS.md §5). Do not retry
  piecemeal — needs larger founding population + coordinated re-anchor sweep.
- Parallel session shares this clone: it authored the AP3 package (untracked at
  this writing, landing imminently) and owns the Phase-5 jitter / ritual-emergence
  lanes. Check `git log --oneline -5 && git status --short` before EVERY session.

## 1. Era I — Foundation (iterations 266–272)

Theme: make the KosmOS ontology machine-consumable in-repo and stand up the inert
`mindstrata-development` substrate crate. **Zero behavioral blast**: golden replay
byte-identical after every one of these iterations; no sim consumers exist yet.

| Iter | WP | Scope | Exit criteria |
|---|---|---|---|
| 266 | [WP-0A](architecture/AP3-afa/waves/WP-0A-vendor-codegen.md) | Vendor ontology extracts (`vendor/ladder.csv`, `lines.csv`, `couplings.csv`, `cells/**`, `PROVENANCE.md`) + `scripts/afa_codegen.py` emitting deterministic Rust tables + `COVERAGE.md`. Docs/scripts only — `crates/**` forbidden by ledger. | Codegen idempotent (byte-identical rerun); coverage report committed; suite untouched-green |
| 267 | [WP-0B](architecture/AP3-afa/waves/WP-0B-crate-scaffold.md) | Scaffold `crates/mindstrata-development` (leaf on core, `deny(missing_docs)` from birth); wire generated canon tables; hand-written `canon.rs` constants marked `// CALIBRATION-PENDING(AP3)` | Clippy clean w/ full docs; crate unreferenced by sim so golden byte-identical; unit pins (17 slug round-trips, registry lookups, resonance shape) |
| 268 | [WP-A](architecture/AP3-afa/waves/WP-A-stage-line-types.md) | `stage.rs` (`StageCoord`, band predicates) + `line.rs` (`LineId`, `Scope`, `StageLinesMap`). **FIRST CONTRACT FREEZE** — record hashes in the WP brief changelog | Band boundaries match vendor ladder exactly; unattested defaults visible not silent; golden untouched; freeze hashes recorded |
| 269 | WP-B | Field engine part 1 (`field.rs`): attractor-field placement API over frozen stage/line types | Golden untouched; unit pins per brief; no sim consumers |
| 270 | WP-C | Field engine part 2 (`dynamics.rs`, `pathology.rs`): resonance + 4-fold pathology operator, Agape/Eros metabolism, pure functions only | Golden untouched; pathology identity-at-neutral pins; determinism pins |
| 271 | WP-D | Field engine part 3 (`lambda.rs`): Λ-gating; **SECOND FREEZE** — field-engine APIs locked before any person-side wiring | Golden untouched; freeze hashes recorded; engine fully unit-pinned |
| 272 | Era-I gate audit | Re-audit: coverage gaps → backlog notes; probe-harness scaffold for Era II (`i2XX_*.rs` patterns); doctrine §3 calibration-prep checklist; slack for spillover | All six WP briefs' acceptance boxes ticked; ledger status column updated; suite + goldens green |

Era-I exit gate (from 04-waves §1): golden byte-identical · unit pins green ·
zero consumers.

## 2. Era II — Person wiring (sketch, iterations 273–280)

Budget, not promise. Sequence per dependency graph: **E** (psyche placement +
bundle field — owns `psyche.rs`, integration windows on `mod.rs`/`population.rs`
draws) → **F** (`pass_development.rs`, ⓦ core.rs window) → **G1** → **H1**.
Freeze points: after E (person-side placement), before any behavioral fold.

Exit: first measurable developmental differentiation across agents; every new
consumer passes a zero-at-zero gate (identity at neutral inputs → byte-identical
until the pass actually moves something).

## 3. Eras III–V

See 04-waves §1 table (content emergence 281–292, collective holon 293–305,
harvest 306–320 then open-ended loop recipe). Plan each era's iterations only
when the previous era's exit gate holds — estimates there will slip and that is
by design ("budgets not promises").

## 4. Per-iteration verification contract (every iteration, no exceptions)

1. Probe BEFORE touching: `crates/mindstrata-benches/examples/i<iter>_<name>.rs`
   (unique names prevent collision). Observable output is the only success signal.
2. Gates: `cargo fmt --all && cargo clippy --workspace --quiet && cargo test
   -p mindstrata-tests --lib --release`. Behavioral iterations additionally
   `scripts/gate --full` BEFORE push (the Iter-263 retroactive-gate lesson).
3. Structural changes are proven by byte-identical golden replay; behavioral
   changes own re-anchor evidence per AGENTS.md §4 (measured value, old band,
   mechanism).
4. Snapshot drift: insta CLI is absent on this host — hand-promote `.snap.new`
   body into `.snap` dropping the `assertion_line` header field.
5. Environment drift check: if the suite shifts with NO code change, bisect via
   `git archive <green-sha> | tar -x -C /tmp/x` and build there — a pristine-
   archive failure proves toolchain drift (the 896ddb6 precedent), not code.
6. Standing hazards (AGENTS.md §5): Fixed-4 truncation disease (f64 shadows for
   sub-resolution rates) · mem::take write-back trap · midpoint neutrality for
   genome/state multipliers · RNG stream discipline (field-order draws,
   count-alignment ≠ byte-alignment) · founder variance IS the budget at N=12.

## 5. Parallel-session coordination

- The ledger (04-waves §3) is authoritative. Whole-file scopes only; ⓦ
  integration windows dispatch alone.
- This session's historical lanes (TUI render/session, chronicle/annals, sim
  `household.rs`) overlap Era-V WPs K/L — before ANY AP3 work touching those
  paths, confirm the other session is idle (`git log` freshness) and re-plan
  ownership in the wave ledger if hot.
- Land small, land often: checkpoint-commit verified edits immediately; HEAD
  has moved mid-flight twice before.
- Merge conflict = planning bug: fix the ledger, record in wave status, not
  just the conflict.

## 6. Fallback queue (only if AP3 is blocked)

Ordered, each an independent commit point:

1. Ritual post-formation emergence (Phase-4 item 3 residual) — **coordinate
   first**: flagged open by the parallel session; claim via ledger before
   starting.
2. `sensory_acuity` genome field micro-debt: zero consumers (documented
   Iter-260 deferral) — either wire or formally retire the field.
3. `physical_potential` injury-susceptibility & labor-output wiring (documented
   Iter-244 deviation; hot-path risk — probe-first).

Explicitly YAGNI'd (do not build without new evidence): O(n) contact grid for
contagion (Iteration-261 commit message).
