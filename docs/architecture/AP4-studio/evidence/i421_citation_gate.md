# i421 — Citation check in the gate (doc_index check 4)

**Status:** DONE (2026-09-24). `scripts/doc_index.py --check` now fails when a
`*.md|rs|py|sh` token cited in an AUTHORITY or ACTIVE doc does not resolve. Wired
into `scripts/gate` step 2.6 (no gate-script change needed — the check rides the
existing `doc_index.py --check` invocation).

## Why

i386 swept the governing docs by hand and found five citation-drift classes:
ghost filenames (`IC-2-observability.md`, `IC-7-ui-telemetry.md` — never
existed), moved files (`sim/pass_health.rs`, `psychology/lore.rs`), a
crate-moved symbol, two never-shipped artifacts cited as shipped (`scripts/
golden_replay.sh`, `keybind_cheatsheet.md`), and balance docs reporting live
levers as unlanded. `doc_index.py` passed green throughout — it checked
*structure* (classified, no ghost index rows, reconciled markers), and citations
were invisible to the gate. This check closes that class permanently: a doc
that cites a file which does not exist now fails the gate, not a by-hand sweep.

## Design

**Scope — the must-be-true set.** Only AUTHORITY and ACTIVE docs per the §6
index (8 docs today, root docs included). HISTORICAL/REFERENCE docs are frozen
records of their era: `docs/MINDSTRATA_CURRENT_STATE.md` citing `sim.rs` is
history, not drift. Same scope principle as the freshness (marker) rule.

**Resolution ladder** (the project's own conventions, tried in order):
repo-relative (`crates/…`, `scripts/…`) → docs-relative → doc-parent-relative →
AP4-studio shorthands (`evidence/X`, `charters/X`, `contracts/X`, `runbooks/X`)
→ `archive/X` and any `AP…/…` token under `docs/architecture/` →
crate-relative (`sim/core.rs` → `crates/mindstrata-sim/src/sim/core.rs`,
`core/src/X.rs` → `crates/mindstrata-core/src/X.rs`, and first-component-as-
crate-stem: `development/lore.rs` → `crates/mindstrata-development/src/lore.rs`)
→ bare filename resolved against `git ls-files` basenames. URLs, absolute/`~`
paths and globs are skipped; fenced code blocks are skipped.

**`HISTORICAL_CITES` — the curated escape.** A must-be-true doc may
legitimately name a nonexistent path *as history*: AGENTS §4.14 cites i386's
ghosts and moved files as the drift class it fixed; the PLAN_DC3 i386 row quotes
what it re-pointed; `realms.md`/`rays.md` are vendor-blocked external vault
artifacts; ENGINE_STATUS §1 records rust-craft C2's `tests/comparison.rs`
deletion. Each such citation is curated in-script as a **(doc, token,
line-anchor)** triple — a regex matched against the citing LINE, so the
exemption is **line-scoped**: a future occurrence of the same token elsewhere
in the doc still fails the gate (a doc-wide key would have exempted it
forever). 25 anchored entries today, each with its reason in the comment;
anything not anchored must resolve *now*. (Count note: the repair commit
message said 24; the correct count is 25 — the file is the truth.) The i421
ledger row itself was
caught by its own gate while landing (it quoted the tokens it curates as bare
text) — reworded to describe rather than re-cite, which is the correct fix,
not a new exemption.

## Findings on the live tree

- **669 raw citation tokens across 39 governed docs**; the ladder resolves all
  but the must-be-true residue. The gate now watches ~130 tokens across the
  8 must-be-true docs.
- **2 real drift sites fixed in this commit** (both AGENTS.md §7, both
  pre-split prescription examples that landed under other names):
  `sim/family.rs` (marriage/birth/kinship) — the split actually produced
  `sim/household.rs` / `sim/marriage.rs`; and `sim/tests.rs` — the tests live at
  `sim/tests/mod.rs` with per-domain modules.
- **1 extractor false positive class fixed**: `cardiovascular.shock_risk`
  matched as `cardiovascular.sh` until the extension was required to end at a
  word boundary (`\b`).

## Trip proof (the plan row's exit criterion)

Planted in `docs/ENGINE_STATUS.md` (reverted immediately):

```
CITATION: docs/ENGINE_STATUS.md cites `evidence/i999_planted_ghost.md` — no such file …
CITATION: docs/ENGINE_STATUS.md cites `scripts/definitely_missing.sh` — no such file …
→ exit 1
```

After revert: `doc_index: OK — 72 governed docs all classified … 0 ghosts`,
exit 0. Both halves proven live — **and made repeatable**: `doc_index.py
--selftest` re-runs the ghost trip, all four resolution conventions, the
word-boundary extractor rule, and the line-scoped exemption (same token on a
different line must fail) as pure-function checks, wired into `scripts/gate`
next to `--check`.

## Note on concurrent work

The gate first failed at segment 1 (fmt) on the *other* session's untracked
in-flight probe draft (`i402_kind_schedule.rs`, two wrapping diffs). That
session had been silent ~50 min with a dead run; the tree was formatted with
`cargo fmt` (mechanical, semantics-preserving) so any session's gate can pass —
noted here per AGENTS §7 concurrent-etiquette.
