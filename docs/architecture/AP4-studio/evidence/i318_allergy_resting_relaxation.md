# Iteration 318 — Allergy resting relaxation (difficulty-levers row 3 residual)

**Status:** LANDED · **Root cause owned:** `docs/balance/difficulty-levers.md` row 3
residual — *"the Q2 dark-allergy quadrant still sits at 0.6–0.7 against ceiling 0.80 in
the canon band (the i293/i294 always-step absence-growth law), so its dynamic range is
compressed."* This iteration sizes that claim, names the mechanism, and closes it.

## The mechanism

`QuadrantState::step` for Allergy was:

```
next = I + growth · 0.1 · headroom · (1 − pressure)      // absence growth
           − decay · pressure · I                        // pressure is the ONLY decay
```

Addiction carries a baseline leak (`− decay · I`); Allergy did not. So a tick with no
pressure was **pure monotone growth toward `ceiling`**, and the quadrant's long-horizon
equilibrium was set by the *horizon*, not by the agent's reconciliation (transgression /
grief) diet. Only pressure could bring it back down, and the natural transgression rate is
sparse — so agents with a quiet life asymptote onto the cap with zero headroom.

The always-step absence law itself is deliberate and stays (i293 20-seed / i294 N=48 —
without it Q2/Q4 were pinned at exactly 0.0000 forever). The missing piece was a
relaxation that does not require an external event.

## Probe verdict — the saturation is real and horizon-growing

`i318_allergy_dynamic_range`, 12-seed family (the i268 instrument), canon band.

**Before** (the old law, from the probe's leg 0 and the pre-fix run):

| horizon | Q2 mean | Q2 p50 | Q2 ≥95% of ceiling | Q4 ≥95% of ceiling |
|---|---|---|---|---|
| 20 000 | 0.6182 | 0.7157 | 0/160 (**0%**) | 0/160 (0%) |
| 50 000 | 0.6812 | 0.7695 | 93/185 (**50%**) | 86/185 (46%) |
| 100 000 | 0.6776 | 0.7902 | 152/237 (**64%**) | 119/237 (50%) |

So the compression is not present at 20K (a healthy 0.35–0.75 spread) — it is a
**lifecycle** effect that the lever's own catalog horizon ("100K lineage") walks straight
into. `sd` rises with horizon (0.152 → 0.182 → 0.210): differentiation does not collapse,
but by 100K two thirds of the population have arrived at the cap with no headroom left.

**Leg 0 — the law itself, stepped directly** (Q2 params `growth 0.045 / decay 0.022 /
ceiling 0.80`; the new relaxation is `0.05 × decay`):

| pressure | ticks | old | new |
|---|---|---|---|
| 0.00 | 1 000 | 0.7912 | 0.6405 |
| 0.00 | 20 000+ | **0.8000** (= ceiling) | **0.6429** |
| 0.05 | 20 000+ | 0.6363 | 0.5282 |

Old absence equilibrium is *exactly the ceiling*; the new one is `growth·SCALE·ceiling /
(growth·SCALE + decay·RELAX)` — strictly below it for every quadrant.

## What landed

1. `ALLERGY_RESTING_RELAXATION = 0.05` and `ALLERGY_ABSENCE_SCALE = 0.1` as named
   measured/old/mechanism constants in `mindstrata-development::dynamics`.
2. `QuadrantState::step` Allergy branch gains `− decay · ALLERGY_RESTING_RELAXATION · I`.
3. Scaled by `decay`, so the relaxation rides the row-3 **difficulty decay band** (a
   brittle village relaxes slower) without adding an `OperatorParams` field — no serde
   churn for the four probes that build `OperatorParams` by literal.

## After — `ALLERGY_CEILING_PILE_ELIMINATED`

| horizon | Q2 mean | Q2 p50 | Q2 max | ≥95% ceiling | Q4 ≥95% ceiling |
|---|---|---|---|---|---|
| 20 000 | 0.5371 | 0.5912 | 0.6234 | **0/156 (0%)** | 0/156 |
| 50 000 | 0.5643 | 0.6265 | 0.6425 | **0/186 (0%)** | 0/186 |
| 100 000 | 0.5815 | 0.6398 | 0.6429 | **0/244 (0%)** | 0/244 |

The asymptotic Q2 equilibrium now lands **inside the catalog's `0.6–0.7` band** (0.643)
instead of drifting onto the 0.80 cap; Q4 settles at 0.529 (catalog ~0.5). The quadrant is
now diet-determined at every horizon rather than horizon-determined.

## Blast radius and re-anchors (§4)

- **Lever verdicts still live, re-run at HEAD:** `PATHOLOGY_BANDS_LIVE` (Q1 direction
  brittle>standard>resilient **12/12** on all three comparisons) and
  `PATHOLOGY_CEILING_BAND_LIVE` (Q2/Q4 high>canon 12/12, canon>low 12/12).
- **Sim suite 280/280, dev suite 83/83** — no re-pins. The dev-level absences-law pin
  survives by construction (the relaxation term is zero at `I = 0`, so the exact
  documented increment is unchanged; the `>0.5 after 500` bound still holds ahead of the
  new 0.833 attractor).
- **One snapshot re-anchored:** `long_horizon_surface_10000_ticks`. Mechanism — lower
  lifetime allergy ⇒ less violence suppression (the `legal.rs` note: Allergy pressure
  *suppresses* the violence-escalation channel) ⇒ marginally more social throughput:
  `event_count 140 527 → 140 737` (+0.15%), `avg_stress 0.3716 → 0.3761`,
  `total_grain 1.2697 → 1.9641`, `total_memory_traces 884 → 948`, relationship-stage
  mix moves a few counts. **`agent_count 12` preserved**, `institution_count 3`,
  `faction_count 0`, `tick 10 000`. Both **goldens byte-identical** (their 4 320-tick
  horizon sits before the divergence grows).

## Pin

- `allergy_absence_attractor_stays_below_ceiling` — converges from neutral over 20 000
  absence ticks and asserts the intensity matches the leak-balance prediction (`±1e-3`)
  **and** leaves >0.1 of headroom below the ceiling. This is the invariant the old law
  violated (its absence attractor *was* the ceiling).

## Verification

`cargo fmt --all` clean · clippy **0** · dev **83/83** · sim **280/280** · tests
**309/0/1** · `scripts/gate --full` **GATE GREEN**.

**Probe:** `cargo run --release -p mindstrata-benches --example i318_allergy_dynamic_range`
(≈4 min wall; 12 seeds × 20K/50K/100K + the law leg).
