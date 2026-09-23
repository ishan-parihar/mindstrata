# i386 — the governing document set re-pointed at the tree (citation drift, five classes)

**Status:** LANDED (documentation + one probe-free code comment; no simulator behaviour
changed) · **Root cause owned:** i385 reconciled `ENGINE_STATUS.md`'s *numbers* and
`reconciled_commit`, but the reconciliation stopped at the two files it was reading. The
same class of drift — **a doc citing something that moved or never existed** — was sitting
in the governing set that agents actually plan from, and `scripts/doc_index.py` cannot see
it (it validates structure: classified, no ghosts, marker resolves — all green).

## Method (so the sweep is repeatable, not a memory exercise)

1. **Ghost scan** over every governed doc (authority/active/reference, excluding the
   auto-classified historical directories): extract every `*.(md|rs|py|sh|ron|toml|json)`
   token, resolve it against the repo root **and** the doc's own directory, then against a
   basename index of the whole tree. Two buckets: *true ghosts* (no such basename anywhere)
   and *suspect paths* (basename exists elsewhere ⇒ a moved/re-pointed file).
2. **Ground-truth read** of every suspect that a department plans from: open the file the
   doc names, and the file the code actually uses.
3. Fix the doc **and**, where the code's own doc-comment asserted the same stale fact, fix
   that too.

## What the sweep found (five classes, all fixed)

| # | Class | Instance | Truth |
|---|---|---|---|
| 1 | Ghost citation | `03-interlock-map.md` named `contracts/IC-2-observability.md` and `contracts/IC-7-ui-telemetry.md` | the shipped files are `IC-2-annals.md` and `IC-7-modding.md` |
| 2 | Moved pass file | IC-3 determinism law cited `sim/pass_health.rs:125,174,182` and pass purity on `sim/pass_biology.rs:6–192` | Arc-D extraction moved them to `systems/health.rs` (draws :126/:175/:183) and `systems/biology.rs:10–197`; the **law** itself was re-verified unchanged |
| 3 | Crate-moved symbol | IC-7 credited `self.lore_archetypes` to `crates/mindstrata-psych/src/psychology/lore.rs:9-46` | the crate ladder moved it to `crates/mindstrata-development/src/lore.rs`, where `archetype_for_claim` is at **:100** |
| 4 | **Doc says "not landed" about a live lever** | `canon-inventory.md`: "zero markers exist in compiled code — the `mindstrata-development` crate does not exist yet"; `pathology-curves.md`: "spec frozen, **values not landed** … no sim-side pathology behavior is live" | the crate, `canon.rs` (3 constants, two of them consumed by `lambda.rs:56` / `collective.rs:191`) and **27 code-side `CALIBRATION-PENDING` marker sites across 7 files** have been shipping since WP-0B; the row-2/row-3 levers are **fully live** (i304/i315, 12-seed isolation + family legs) |
| 5 | Aspirational constant names | `pathology-curves.md` + `difficulty-levers.md` row 3 named `PATHOLOGY_GROWTH_* / DECAY_* / CEILING_*` as `canon.rs` entries | **no such constants were ever created.** The real surface is `PROD_QUADRANT_PARAMS` (`systems/development.rs`) reached through `SimParameters::{pathology_growth_scale, pathology_decay_scale, pathology_ceiling_scale}` — the difficulty-band pattern |

## Why class 4 is the expensive one

Classes 1–3 cost an agent a few minutes (open the file, find it renamed). Class 4 is a
**trap**: `canon-inventory.md`'s own stated purpose is to tell the next planner which canon
entries still need work, and it said the entire scaffold was unbuilt. Its seven-row "forward
registry" also predicted a landing mechanism (canon.rs constants) that four of five shipped
levers did not use. An agent obeying the doc would have re-scaffolded a crate that exists
and re-derived bands that are already measured. The corrected file is now a **status**
ledger: per-row verdict, the file the value actually lives in, and the evidence link.

## Doctrine and enforcement

- **AGENTS.md §4.14 (new):** a governing doc's citations are part of its contract — re-point
  in the commit that moves the file; `FROZEN` freezes the rule, never the path; and verify a
  ledger row's claim about *where* a value landed by opening the file it names.
- **DOCUMENTATION.md §4 rule 4 (new):** states explicitly what `doc_index.py` cannot check,
  so the gate's green light is not mistaken for citation correctness.
- **`ENGINE_STATUS.md` §10:** one bullet — `file:line` citations in governing docs are
  pointers, not contracts.

## Blast radius

Documentation only (`AGENTS.md`, `DOCUMENTATION.md`, `ENGINE_STATUS.md`, `README.md`,
`docs/ROADMAP.md`, `docs/PLAN_DC3_DEVELOPMENT.md`, `docs/balance/{canon-inventory,
pathology-curves,difficulty-levers,needs-bands,perf-budget}.md`,
`docs/architecture/AP4-studio/{PLAN.md,03-interlock-map.md,charters/DC3-P0-perf-budget.md,
contracts/IC-3,IC-5,IC-6,IC-7,runbooks/calibration-audit-v2.md}`, AP3 refs index) plus one
stale code doc-comment. **No probe, no gate, no test, no golden, no simulator behaviour
changed** — the full `scripts/gate --full` run is the verification, and it is green.

Re-measured facts quoted in this pass (all verified at `cd4ac0a`): 13 crates · 456 `.rs` ·
146 320 LOC · **1 683** `#[test]` fns (social 397, tests 313, sim 301, psych 223, person
121, development 83, institutions 68, tui 61, world 60, core 35, render 15, cli 6) · 217
probes · 166 evidence docs (+this one) · 36 RON specs · `MAX_POPULATION = 256` · 178
auto-classified docs · 70 governed docs, 7 carrying a reconciliation marker.
