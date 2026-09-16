# Iteration 271 — Arc-D pass extraction complete: Batches 2 + 3

**Status:** LANDED · **Type:** pure structural refactor (AGENTS.md §7 ladder) — golden-referee proven byte-identical

## Scope

Completed the Arc-D bio/psych pass extraction per the frozen plan in
`crates/mindstrata-sim/src/systems/INVENTORY.md`. Batch 1 (biology + health →
`systems/{biology,health}.rs`) landed earlier at `937d930`; this iteration
executed the remaining two batches:

- **Batch 2**: `sim/pass_appraisal.rs` (801 lines) → `systems/appraisal.rs`,
  `sim/pass_decay.rs` (308 lines) → `systems/decay.rs`
- **Batch 3**: `sim/pass_cognitive.rs` (1,061 lines) → `systems/cognitive.rs`

All three moved with `git mv` (history-preserving), bodies **verbatim** —
zero behavioral edits, zero serde shape edits, RNG stream order untouched
(INVENTORY risk notes honored: write-back loop stayed in `core.rs`; no
draw-order changes).

## Visibility surface (shim-first pattern, minimal churn)

- Moved pass fns: `pub(super)` → `pub(crate)` (`tick_appraisal_pass`,
  `tick_decay_pass`, `tick_cognitive_pass`)
- `sim::ATTACHMENT_LOW_SUPPORT_THRESHOLD` → `pub(super)` (sole consumer moved)
- `sim::life_chapter_crossed` → `pub(super)` (sole consumer moved; test module
  keeps compiling via `use super::super::*` glob — pre-existing, tracked debt)
- `sim::snapshot_metrics` → `pub(super) mod`; `self_esteem_support` →
  `pub(crate)` (moved cognitive pass consumes it)
- `Simulation::relationship_v2_pos` → `pub(crate)` (same)
- `sim/mod.rs`: `use crate::appraisal::{Agency, Appraisal}` removed (moved
  with its consumer; canonical `crate::appraisal::` path used in place)

## Verification (byte-identity referee)

| Gate | Result |
|---|---|
| `cargo build -p mindstrata-sim` | 0 warnings, 0 errors |
| `cargo test -p mindstrata-sim --lib` | **216/216** |
| `cargo clippy --workspace --quiet` | 0 warnings |
| `cargo fmt --all --check` | clean |
| `scripts/gate --full` | **GATE GREEN — 307/0/1 @ 164 s, golden 5/5 byte-identical, zero baseline regen** |
| `bench_index.py --strict` | 0 violations |

**Only deltas are import paths + visibility qualifiers** — the same proof
signature as Batch 1.

## State of the Arc-D ladder

- `sim/core.rs`: 777 lines (orchestration + write-back only; the ~2,200 lines
  of moved pass bodies now live in domain modules under `systems/`)
- `systems/` now holds the full bio/psych pass set: `biology`, `health`,
  `cognitive`, `appraisal`, `decay` + the already-extracted `development`,
  `institutions_multiplier`
- Remaining sim slimming (per AGENTS.md §8.2): `sim/*_impl` glue detangling —
  a separate window; `actions.rs` waits until arcs move off it.
