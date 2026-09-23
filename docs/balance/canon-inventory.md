# Canon Inventory — CALIBRATION-PENDING Sweep

**Method**: `rg -n "CALIBRATION-PENDING" --include="*.rs" crates/` plus a repo-wide sweep
over `*.{rs,md,toml}`. Marker convention defined in
`docs/architecture/AP3-afa/waves/WP-0B-crate-scaffold.md` (form:
`// CALIBRATION-PENDING(AP3): <what the probe must measure>`) and consumed by IC-5
(`03-interlock-map.md`).

**Re-swept: 2026-09-23 at `cd4ac0a` (i386).** The 2026-08-25 version of this file said
"zero markers exist in compiled code — the `mindstrata-development` crate does not exist
yet." That was true at WP-0B; it is now false on both counts, and a stale inventory is
worse than none (it invites a re-scaffold of something already built).

## Finding (measured)

- **The crate and the canon surface exist.** `crates/mindstrata-development/src/canon.rs`
  holds three hand-written constants (`FIELD_UPTAKE_QUANTUM_F64`,
  `NEUTRAL_FIELD_VALUE_F64`, `STAGE_COUNT`), and it is **read**, not decorative —
  `FIELD_UPTAKE_QUANTUM_F64` gates field admission in `lambda.rs:56` and `STAGE_COUNT`
  sets the collective-field ceiling in `collective.rs:191`. The vendored canon tables
  (`canon_gen::tables`, 17/49/851 rows, sha-pinned `b010804e…`) are a separate,
  **vendor-frozen** surface with its own provenance (`vendor/afa/PROVENANCE.md`) — vendor
  values are not calibration candidates.
- **Markers now exist in code: 31 grep hits, 27 of them non-bench, across 7 source
  files** (`sim/{actions/mod.rs, sim/catalyst_observers.rs, sim/household.rs,
  sim/norms_impl.rs, systems/mod.rs, systems/development.rs}` +
  `development/src/{canon.rs, catalyst.rs, collective.rs, dynamics.rs}`). The remaining 4
  are probe-side references to the convention. Every one is a *pending* annotation on a
  live value, not a value waiting for a file to exist.

## Registry — landing status of the seven forward entries

The 2026-08-25 "forward registry" listed constants that *would* carry markers at scaffold
time. Five of the seven have since been promoted, refuted, or landed by a different
mechanism than the table predicted — which is itself the lesson: **canon entries land as
banded `SimParameters` surfaces where a difficulty lever exists, and as `canon.rs`
constants only where no band is wanted.**

| # | Entry | Status (i386) | Where it actually lives | Evidence |
|---|---|---|---|---|
| 1 | Needs-band fulfillment thresholds (per drive) | **LANDED as a band**, not a canon.rs row — the five goal gates resolve once per tick from a difficulty-scaled surface; the measured distributions show the spec's proposed ranges sat above the channels' occupied range, so the empirical values were kept | `SimParameters.goal_gate_scale` → `systems::GoalGates` (i305); decay half `SimParameters` + `DifficultyProfile` (i303) | `i305_goal_gate_bands`, `i303_difficulty_levers`, `needs-bands.md` |
| 2 | Pathology intensity curves (4 quadrants) | **LANDED as a band** (growth/decay i304, ceiling i315) — the `PATHOLOGY_GROWTH_*`/`DECAY_*`/`CEILING_*` constant names this file predicted were never created | `SimParameters::{pathology_growth_scale, pathology_decay_scale, pathology_ceiling_scale}` + `PROD_QUADRANT_PARAMS`/`pathology_params` (`systems/development.rs`) | `i304_pathology_bands`, `i315_pathology_ceiling`, `pathology-curves.md` |
| 3 | Stage band boundaries | **Vendor-frozen, not calibrated** — `STAGE_COUNT = 17` is pinned to the vendored ladder; a canon change, not a probe target | `development/src/canon.rs`, `canon_gen::tables` | `canon_gen.rs` freeze pin (FR-022) |
| 4 | Resonance line-affinity weights | **STILL PENDING** — the marker is live and honest: the matrix awaits vendor coupling rows | `development/src/dynamics.rs:23` | marker at that site |
| 5 | Catalyst uptake quantization point | **Value exists, standing is pending** — `FIELD_UPTAKE_QUANTUM_F64 = 1.0e-4` is the shipped single quantization point and IS consumed, but its marker still asks for the probe that measures the smallest behaviorally visible delta vs the golden noise floor | `development/src/canon.rs:11` | marker at that site; `i284_q1_calibration` |
| 6 | Pacing horizons (10K/50K/100K) | **LANDED as a doc + probe contract**, not a constant — the horizons are played and measured, not stored | `docs/balance/pacing-model.md`; `p5_100k_probe.rs`; `i270_perf_snapshot` | `pacing-model.md` §table |
| 7 | Founder line-profile distribution parameters | **OPEN SYSTEMIC DEBT, deliberately untaken** — reshaping the uniform founder draw starves every extreme-driven producer at N=12; it needs a larger founding population plus a coordinated re-anchor sweep | `AGENTS.md` §5 (founder-variance debt) | Iteration 263 audit H5 |

## Coverage arithmetic

grep hits: **31** (27 code sites + 4 probe-side conventions). Registry rows: 7 — 4 landed
(rows 1, 2, 5-value, 6), 1 vendor-frozen (row 3), 1 pending with a live marker (row 4), 1
recorded as deliberate debt (row 7). **Unexplained markers: 0** — every code-side marker
maps to a row above or is annotated at its site with the probe that will close it.
Re-running the header command reproduces the count.
