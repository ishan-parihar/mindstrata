# Iteration 304 — pathology growth/decay bands (difficulty-levers row 3)

**Status:** LANDED · **Root cause owned:** `docs/balance/difficulty-levers.md`
row 3 (pathology growth/decay/ceiling) was a DRAFT hypothesis row whose
"knobs" were module constants in `crates/mindstrata-sim/src/systems/development.rs`
— the pass took no parameter input, so no band could reach it and the catalog's
Resilient/Standard/Brittle bands had no executable surface. i303 promoted row 2
(need decay) and named this row as the next promotion candidate; this is that
iteration.

## What landed

1. **`SimParameters::{pathology_growth_scale, pathology_decay_scale}`**
   (`mindstrata-core/src/parameters.rs`) — Fixed multipliers, serde default =
   `Fixed::ONE` via an explicit helper (a bare `#[serde(default)]` would have
   loaded ZERO, silently disabling the operator for every pre-i304 payload).
   `with_difficulty` now writes the row-3 band alongside row 2:
   Lenient → Resilient `0.5×` growth / `1.2×` decay; Standard → `1.0/1.0`;
   Harsh → Brittle `1.8×` growth / `0.7×` decay.
2. **`pathology_params(&SimParameters) -> [OperatorParams; 4]`**
   (`sim/systems/development.rs`) — the single resolve point, called ONCE per
   tick. `PROD_QUADRANT_PARAMS` becomes the canon (Standard-band) array the
   resolver scales; ceilings are deliberately not scaled (the catalog's row-3
   bands name growth and decay only).
3. **`system_development_with_params(agents, events, params)`** — the tick
   pipeline now calls this (`sim/core.rs` passes `&self.params`); the old
   two-arg `system_development` remains as a wrapper at the canon band, so
   ~14 pre-i304 call sites (probes, unit tests) are untouched.
4. **Snapshot v16** already persists `SimParameters` (i303); the two new
   fields ride along and are asserted in the round-trip + resume pins.
5. **Probe** `i304_pathology_bands` (12-seed family × 3 bands × 20K ticks) +
   **3 sim unit pins** + **1 core pin** + **2 snapshot pins**.

## The §5 quantization check (why this row is cleaner than row 2)

Row 2's 0.6×/1.4× mapping quantized non-uniformly at Fixed-4 (three documented
collapses). Row 3 does not, and the probe measures it rather than asserting
it: `0.5 / 1.2 / 1.8 / 0.7` are all exactly representable (raw 5000 / 12000 /
18000 / 7000), and the operator itself is f64-native (`OperatorParams`), so
the scaled per-tick fractions are exact — no sub-resolution class exists here.
The multipliers are still materialized once at construction (never re-derived
per tick), keeping the §5 discipline uniform across both promoted rows.

## Exit evidence — probe, 20K ticks/seed, family of 12

### Leg 1 — configuration surface

| Check | Before i304 (measured) | After i304 |
|---|---|---|
| `SimParameters` pathology-band keys (lenient↔standard) | 0 | 2 |
| `SimParameters` pathology-band keys (standard↔harsh) | 0 | 2 |
| resolved tuples pairwise distinct across bands | false | true |

Resolved per-quadrant `growth/decay/ceiling` (canon → bands):

| band | Q1 dark-add | Q2 dark-all | Q3 golden-add | Q4 golden-all |
|---|---|---|---|---|
| resilient | 0.0300/0.0180/0.800 | 0.0225/0.0264/0.800 | 0.0350/0.0180/0.850 | 0.0150/0.0300/0.750 |
| standard (canon) | 0.0600/0.0150/0.800 | 0.0450/0.0220/0.800 | 0.0700/0.0150/0.850 | 0.0300/0.0250/0.750 |
| brittle | 0.1080/0.0105/0.800 | 0.0810/0.0154/0.800 | 0.1260/0.0105/0.850 | 0.0540/0.0175/0.750 |

### Leg 2 — Standard ≡ canon (the zero-blast contract)

| Check | Result |
|---|---|
| params serde identical to default | true |
| end-state digest identical (seed 42, 20K ticks, f64 bit patterns) | true |
| `pathology_params` bit-identical to `PROD_QUADRANT_PARAMS` | PASS (`x * 1.0 == x` in IEEE-754) |

### Leg 3 — family differential (mean quadrant intensity)

| band | Q1 | Q2 | Q3 | Q4 | max Q1 | events | alive |
|---|---|---|---|---|---|---|---|
| resilient | 0.1970 | 0.4411 | 0.0541 | 0.3196 | 0.2963 | 258,395 | 12/12 |
| standard | 0.3405 | 0.6277 | 0.1059 | 0.4999 | 0.4757 | 264,263 | 12/12 |
| brittle | 0.4799 | 0.7342 | 0.1846 | 0.6337 | 0.6407 | 262,548 | 12/12 |

Per-seed dark-addiction direction: **brittle > standard 12/12**,
**standard > resilient 12/12**, **brittle > resilient 12/12** — the catalog's
predicted direction holds on every seed, not just on the aggregate (the
pre-registered contract, chosen because the Threat catalyst diet is sparse and
bursty per seed). Per-seed spread in the canon band runs 0.14 (seed 46) to
0.42 (seed 2) and the ordering holds inside each — the band is a multiplier on
the seed's own trajectory, not a level shift.

**Verdict: `PATHOLOGY_BANDS_LIVE` — levers row 3 promoted from DRAFT to a
measured runtime surface, with Standard byte-identical to the ratified canon
and therefore no golden/snapshot re-anchor.**

## Findings recorded, not smoothed

1. **Q2 (dark-allergy) runs hot in every band** — 0.44 (resilient) / 0.63
   (canon) / 0.73 (brittle) against ceiling 0.80. That is the i293/i294
   always-step absence-growth law (`growth × 0.1` per tick from neutral, which
   reaches ~0.4 within a few hundred ticks), not something this lever creates:
   the band scales a quadrant that was already near-saturated. The lever
   therefore has compressed dynamic range on Q2 and full range on Q1/Q3/Q4.
   Recorded as **systemic debt**: re-shaping the Allergy absence rate is a
   behavioral iteration of its own (it re-anchors the pathology equilibrium
   pins), exactly the class of change AGENTS §4.5 says to ledger rather than
   fold into a config-surface commit.
2. **Ceilings are not a band** — the catalog's row-3 text mentions a
   `0.65–1.0` ceiling range alongside growth/decay, but names multipliers only
   for growth and decay. Ceilings stay canon; a ceiling band is recorded as
   row-3 residual rather than guessed into the mapping.
3. **Event counts are near-identical across bands** (258K/264K/262K) while
   quadrant intensities differ 2.4×. The lever shapes how the village
   *metabolizes* what happens to it, not how much happens — the honest
   reading of "lineage diverges slowly vs brittly".

## Verification

- core lib **33/33** (2 new pins: band exactness + legacy-JSON default);
  sim lib development filter **36/36** (3 new pins: canon bit-identity,
  scale-without-ceiling, brittle-vs-resilient mechanism pin); sim lib snapshot
  **12/12** (2 pins extended with the row-3 scales); tests-crate release suite
  **309/0/1** (unchanged from i303 — no existing pin moved).
- fmt clean; clippy 0; full `scripts/gate --full` **GATE GREEN** with golden
  baselines byte-identical (**zero re-anchors**).

## Queue after this iteration

- **Row 3 residual:** ceiling band, plus the Q2 saturation debt above.
- **Row 2 residual:** fulfillment-threshold half (goal gates inline in
  `system_goal_generation`).
- **Row 1** stays prohibited (AGENTS §5 H5).
- DC-4 continues with the CLIENT asset-viewer and the graphical shell spike;
  the difficulty surface is now two rows deep on a single run property
  (CLI flag + snapshot field + probe), so a client difficulty selector is a
  read of a published surface (IC-8-clean).
