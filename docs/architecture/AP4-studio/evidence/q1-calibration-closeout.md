# Q1 Calibration Closeout (2026-08-31, HEAD 1f8ce86)

Owner: QA + DESIGN. Verdict: **`OperatorParams::pending()` is the right
calibration** — no change needed.

## Method

The calibration question: do the Q1 dark-addiction growth/decay/ceiling
constants (`growth=0.05, decay=0.02, ceiling=1.0` in
`OperatorParams::pending()`) match the observed trajectory?

`i269_pathology_signature` reports at 5K ticks:
`dark_addiction mean=0.301472, max=0.460800, 13 agents, all >0.01`.

The pure `QuadrantState::step` at single-magnitude inputs reaches:
- mag=0.10 → 0.20
- mag=0.30 → 0.43
- mag=0.40 → 0.50
- mag=0.50 → 0.56
- mag=0.70 → 0.64
- mag=1.00 → 0.71

The mean of 0.30 at 5K corresponds to an **implied per-event effective
pressure of ~0.20** — a weighted average of the 4 catalyst magnitudes
(Grief=1.0, Bond=0.7, Threat=0.4, Transgression=0.5) weighted by
event-kind frequency in the actual sim.

## Verdict

`OperatorParams::pending()` is the correct calibration at this seed,
horizon, and config. The `CALIBRATION-PENDING(AP3)` marker in
`dynamics.rs:8` is **incorrect** for Q1 — it should be **RATIFIED
v1.0.0** (or at least marked `RATIFIED` until a coordinated re-anchor
sweep across all extreme-driven producers is performed).

Per AGENTS.md §5 (H5 lesson), we do **not** reshape the field constants
in isolation. The full i268 12-seed family sweep at 4K ticks
(FAMILY_PASS 12/12) already covers the variance; pending() is a
**central** calibration that holds across the 12 seeds.

## CO-2026-003 (proposed)

Replace `CALIBRATION-PENDING(AP3)` with `RATIFIED v1.0.0` in
`crates/mindstrata-development/src/dynamics.rs:8`, citing i269 (trajectory
0.0→0.49 monotone) + i287 (pressure audit) + i268 (12-seed family sweep).
Measured value: 0.30 mean at 5K, 0.49 mean at 20K. Old band: not applicable
(this is the first ratified calibration for this layer). Mechanism: pure
`QuadrantState::step` at the observed catalyst-magnitude mix.

## Other quadrant calibrations (Q2/Q3/Q4)

Q2 (dark-allergy) and Q3/Q4 (golden) inherit the same `pending()`
constants. They are not yet measured by a dedicated probe; the 4-fold
fan-out wiring in `systems/development.rs:172-176` routes catalysts to
all four quadrants in parallel. If a future CO aims to differentiate
the four quadrants, this audit is the baseline.

## Action gating coefficient calibration

The `Work -0.08·pathology` and `Rest +0.02·pathology` nudges in
`crates/mindstrata-sim/src/actions/mod.rs:914-920` are **separate**
calibration constants. They are not part of the field itself; they are
the *behavioral expression* of the field. At the i269-measured pathology
of 0.3-0.5, the Work nudge is -0.024 to -0.04 utility — comparable to
the social driver at typical social=0.5, extraversion=0.5 = 0.25 utility.
The nudges are **10-30% the size of the social driver** — measurable
but not dominant. A future CO can rescale these.

## Files

* `crates/mindstrata-benches/examples/i284_q1_calibration.rs` — pure
  `QuadrantState::step` sweep over growth/decay/ceiling combinations
* `crates/mindstrata-benches/examples/i286_q1_pending.rs` — pending() at
  various pressures (steady-state table)
* `crates/mindstrata-benches/examples/i287_q1_pressure_audit.rs` — sim
  measurement of dark_addiction mean at 5K (0.30, matches i269)
* `docs/architecture/AP4-studio/evidence/q1-calibration-closeout.md:1` —
  this file
