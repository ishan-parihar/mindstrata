# Iteration 315 — pathology ceiling band (difficulty-levers row 3, final axis)

**Status:** LANDED · **Root cause owned:** `docs/balance/difficulty-levers.md`
row 3's third axis — the catalog lists `CEILING 0.65–1.0 per 4 quadrants`, but
i304 promoted growth/decay only and explicitly deferred the ceiling ("a
per-quadrant ceiling band is a separate hypothesis, recorded as residual rather
than guessed"). This is that iteration: the hypothesis is probed before it is
promoted.

## Why the ceiling governs

Growth is `headroom = ceiling − intensity`
(`mindstrata-development/src/dynamics.rs:104`), so the ceiling sets each
quadrant's equilibrium. i304 measured Q2 (dark-allergy) 0.628 standard / 0.734
brittle against ceiling 0.80 and Q4 0.500/0.634 against 0.75 — both approach
their caps — while Q1 0.341/0.480 and Q3 0.106/0.185 sit far below theirs.

## What landed

1. **`SimParameters.pathology_ceiling_scale`** (Fixed multiplier, serde default
   `Fixed::ONE` via `scale_identity`, the i304 pattern — a bare
   `#[serde(default)]` would have loaded ZERO and silently switched the lever
   off for every existing payload).
2. **`with_difficulty`** writes it with the row-3 band: Lenient (Resilient)
   `0.85`, Standard `1.0`, Harsh (Brittle) `1.15`.
3. **`pathology_params`** scales `ceiling` and clamps to `[0, 1]`.

### Bands in the catalog's `0.65–1.0` range

| band | Q1 | Q2 | Q3 | Q4 |
|---|---|---|---|---|
| resilient (0.85×) | 0.6800 | 0.6800 | 0.7225 | 0.6375 |
| standard (1.0×) | 0.8000 | 0.8000 | 0.8500 | 0.7500 |
| brittle (1.15×) | 0.9200 | 0.9200 | 0.9775 | 0.8625 |

## Probe verdict — `PATHOLOGY_CEILING_BAND_LIVE`

`i315_pathology_ceiling`, 20K ticks × 12 seeds, **growth and decay held at canon
so only the ceiling varies** (isolation leg):

| ceiling | Q1 | Q2 | Q3 | Q4 | alive |
|---|---|---|---|---|---|
| 0.85× | 0.2789 | 0.5188 | 0.0899 | 0.4117 | 12/12 |
| 1.00× | 0.3351 | 0.6148 | 0.1054 | 0.4895 | 12/12 |
| 1.15× | 0.3850 | 0.7175 | 0.1225 | 0.5685 | 12/12 |

Per-seed direction on Q2: **high > canon 12/12**, **canon > low 12/12** (Q4 the
same). The finding worth recording: **every** quadrant is ceiling-sensitive, not
just Q2/Q4 — lowering the cap lowers headroom for the low quadrants too, so the
axis is a genuine whole-field lever rather than a Q2-only clamp.

**Standard ≡ canon**: `pathology_params(Standard)` ceilings are bit-for-bit the
`PROD_QUADRANT_PARAMS` consts (`x * 1.0 == x` in IEEE-754), and the core
Standard-identity serde test includes the new field. **Zero golden/snapshot
re-anchor** — the same zero-blast contract rows 2 and 3's earlier halves shipped
under.

## §4.4 re-contract

`pathology_params_scale_growth_and_decay_without_touching_ceilings` asserted the
ceilings did **not** move — the correct pin while the ceiling was not a knob. It
is now renamed and re-contracted to
`pathology_params_scale_growth_decay_and_ceiling_with_the_band`, guarding the
scaled ceilings (0.85×/1.15×), their resilient-lower / brittle-higher direction,
and their containment in `[0, 1]`.

## Pins

- core: `pathology_ceiling_band_lowers_resilient_and_raises_brittle` (raw
  8 500/10 000/11 500), plus the Standard-identity and legacy-serde-default tests
  extended to the new field.
- sim: the re-contracted `pathology_params` pin above.

## Verification

`cargo fmt --all` clean · clippy **0** · core **35/35** · sim **280/280** ·
tests **309/0/1** · `scripts/gate --full` **GATE GREEN**, golden byte-identical.
