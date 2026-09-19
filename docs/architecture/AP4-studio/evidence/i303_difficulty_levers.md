# Iteration 303 — difficulty-lever surface (DC-4 entry "b")

**Status:** LANDED · **Root cause owned:** the ratified difficulty-levers
catalog (`docs/balance/difficulty-levers.md`, DESIGN 9–10) was a DRAFT
hypothesis list — the canon values it proposes to expose as Low/Medium/High
bands existed only as scattered constants with no executable surface. DC-4
names this as its first core-sim iteration ("difficulty levers (needs-bands
canon, CO-2026-003 ratified)").

## What landed

1. **`DifficultyProfile`** (`mindstrata-core/src/parameters.rs`):
   `Lenient | Standard | Harsh`, `Default = Standard`, `decay_multiplier()`
   = 0.6 / 1.0 / 1.4 (the catalog's candidate bands), `FromStr`/`Display`
   for the operator surface.
2. **`SimParameters::with_difficulty(profile)`** — the pure-data mapping the
   levers doc prescribes (`match difficulty { Low => 0.85, … }` shape):
   rescales the six need-decay rates ONCE at construction (§5 quantize-once;
   no per-tick re-derivation). `Standard` returns the canon defaults
   untouched — identity by construction, not by arithmetic.
3. **`SimParameters.difficulty`** field (serde default = Standard) — the
   band's provenance record, so a snapshot states which band it carries.
4. **Snapshot v15 → v16**: `Snapshot.params` (serde default) — restore now
   replays the captured `SimParameters` instead of rebuilding
   `SimParameters::default()`. The pre-i303 restore silently discarded EVERY
   run-level tuning override across save/load; once band selection is a run
   property that becomes a replay hazard, so the v16 bump is the fix.
5. **CLI**: `--difficulty lenient|standard|harsh` on the `sim` command
   (applied before `populate()`; ignored with a note when resuming a
   snapshot, whose captured params take precedence).
6. **Probe** `i303_difficulty_bands` + **4 core unit pins** + **2 integration
   pins**.

## Exit evidence — probe (12-seed family × 3 bands × 2000 ticks)

### Leg 1 — effective Fixed-4 raw bands (1 raw = 1e-4/tick), quantize-once

| band | hunger | thirst | fatigue | safety | social | meaning |
|---|---|---|---|---|---|---|
| lenient (×0.6) | 6 | 12 | 3 | 2 | 1 | 1 |
| standard (×1.0) | 10 | 20 | 5 | 3 | 2 | 1 |
| harsh (×1.4) | 14 | 28 | 7 | 4 | 3 | 2 |

Documented quantization, not hidden: social Lenient 0.00012 rounds DOWN to
raw 1 (half canon, not 0.6×); meaning Lenient 0.00009 rounds back onto canon
raw 1 — the Lenient meaning band is **sub-resolution at Fixed-4** and
collapses onto Standard. Recorded per §4.4 (semantics narrowed by
construction, not magnitude); the upgrade path, if a 3-way distinct meaning
band is ever required, is the standing §5 f64-shadow accumulator — a
behavioral iteration of its own.

### Leg 2 — Standard ≡ canon (the zero-blast contract)

| Check | Result |
|---|---|
| params serde identical to default | true |
| end-state digest identical (seed 42, 2000 ticks) | true |
| integration pin `standard_difficulty_is_byte_identical_to_canon` | PASS |

### Leg 3 — family differential (aggregate is the pre-registered contract)

| band | hunger | thirst | fatigue | social | meaning | aggregate | Worship share |
|---|---|---|---|---|---|---|---|
| lenient | 0.0160 | 0.1112 | 0.3597 | 0.0052 | 0.2614 | **0.1507** | 0.0530 |
| standard | 0.0168 | 0.1215 | 0.3589 | 0.0087 | 0.2608 | **0.1534** | 0.0443 |
| harsh | 0.0159 | 0.1491 | 0.3659 | 0.0095 | 0.4170 | **0.1915** | 0.0429 |

| Leg | Verdict |
|---|---|
| Standard identity | PASS |
| Family stability (12/12 alive × 3 bands) | PASS |
| Aggregate need pressure monotone | PASS (0.1507 < 0.1534 < 0.1915, +27% lenient→harsh) |
| Catalog's named felt effect (Worship share) monotone | PASS (0.0530 > 0.0443 > 0.0429, −19%) |

Per-channel monotonicity measured 2/5 and is reported as the finding, not
smoothed: **thirst +34%** and **social +83%** respond cleanly; hunger and
fatigue end-deficits are **relief-saturated** at this horizon (the feeding
and rest systems absorb a 2.3× decay-band change — measured 0.0160 vs
0.0159 hunger); meaning is quantization-collapsed at Lenient (leg 1) and
+60% at Harsh. Because the channels couple through behavior, strict
per-channel ordering is not a contract the mechanism can promise — the
aggregate is, and the micro contract (exact raw band per channel) is pinned
in `parameters::tests`.

**Verdict: `DIFFICULTY_BANDS_LIVE` — levers row 2 (need decay) promoted from
DRAFT to a measured runtime surface; Standard is byte-identical to the
ratified canon, so no golden/snapshot re-anchor accompanies the promotion.**

## What this opens / carries forward

- **Levers row 1 (founding variance)** stays Medium-only by standing
  prohibition (AGENTS §5 H5 founder-variance debt: never piecewise — needs
  larger N + a coordinated re-anchor sweep).
- **Levers row 3 (pathology growth/decay/ceiling)** is the natural next
  iteration: `PROD_QUADRANT_PARAMS` (sim `systems/development.rs`) is a const
  tuple today — promoting it needs the same param-threading treatment
  (`system_development` currently takes no params), then a Resilient/
  Standard/Brittle band mapping + probe.
- **Fulfillment-threshold half of row 2**: the decay half landed here; the
  goal-generation gates (0.3/0.5/0.6/0.7, inline in `system_goal_generation`)
  are not yet parameters — recorded as the row-2 residual, not silently
  claimed.
- CLIENT/CLI: the band is now a first-class run property (CLI flag + snapshot
  field), so a client difficulty selector is a read/echo of a published
  surface (IC-8-clean), not a sim coupling.

## Verification

- core lib **31/31** (4 new pins); sim lib snapshot tests 12/12 (2 new pins,
  incl. the v16 wire + resume pins); tests-crate `difficulty` filter 2/2
  (new integration pins).
- fmt clean; clippy 0; bench law 0 violations; full release suite + golden
  baselines green with **zero re-anchors** (Standard identity by
  construction).
