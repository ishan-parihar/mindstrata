# Iteration 270 — ponytail batch: lambda threshold + catalyst magnitudes

**Status:** LANDED · **Probe:** `crates/mindstrata-benches/examples/i270_lambda_catalyst_calibration.rs`

## Probe evidence (5 seeds × 2000 ticks, N=12)

### Catalyst magnitude census (live event stream)

| Kind | n | mean | min | max |
|---|---|---|---|---|
| Bond | 56 | 0.796 | 0.700 | 0.800 |
| Threat | 500 | 0.397 | 0.311 | 0.661 |

- No Grief or Transgression events fired in any 2000-tick window (Grief needs
  deaths — first deaths land past 2000 at N=12; Transgression needs
  NormViolated, which is event-rate-limited like Q3 was).
- Threat dominates volume 9:1 over Bond; Conflict magnitudes spread
  0.311–0.661 (injury/fear-coupled, genuinely differentiated).
- Bond is bimodal-by-construction: marriage 0.8, birth 0.7.

### Threshold sweep (admission rate over observed magnitudes)

| t | admitted |
|---|---|
| 0.02–0.30 | 556/556 = **1.000** |
| 0.35 | 334/556 = **0.601** |

## Decision: ratify t = 0.05 (no pin move)

Per §4.4 this is a **ratification**, not a re-anchor:

- The dead-zone is **empty by design margin**: minimum live magnitude is 0.311
  (Conflict base 0.3 + smallest nonzero injury), 6.2× the threshold. Every
  threshold in [0.02, 0.30] admits 100% of live pressure — the observed
  dynamics are threshold-invariant across a 15× band.
- t = 0.05 stays: it sits far below any live magnitude (zero behavior change,
  golden-invariant) while meaningfully gating sub-noise pressure (anything
  under 0.05 admits exactly zero — the Fixed-quantum phantom-delta guard in
  `admit_quantized` stays load-bearing).
- t = 0.35 would gate 40% of Threat pressure — a *behavioral* change with no
  probe evidence of need. Recorded as the knob for future pacing work, not
  pulled now.

**Mechanism named:** threshold sits below the minimum catalyst magnitude by
design margin; the minimum magnitude is set by the Conflict base (0.3), not by
the threshold.

## Catalyst magnitudes: ratified as-designed

- Threat spread (0.311–0.661) is injury/fear-coupled — genuinely
  differentiated, not a lump.
- Bond's two-point structure (0.7/0.8) matches the two bond-forming events.
- Missing kinds (Grief, Transgression) are event-rate-limited producers, not
  calibration failures — same §4.3 class as Q3 was; their magnitude entries
  (1.0 death, 0.5 violation) remain spec-midpoint pins until a probe can
  observe them at horizon. Grief observation requires a mortality-horizon
  probe (deaths land past tick 2000); recorded in needs-bands.md follow-ups.

## Liveness verification

Live-pass replay (seed 42, full event window through `system_development`):
**12/12 agents' Q3 field moved** — the pass consumes the live stream, gate
open throughout.

## Verification

- Full suite 307/0/1 (`gate --full` GREEN)
- clippy --workspace: 0 warnings; fmt clean
- Doc-tests: 3/3 (referent.rs doc-test rot fixed this iteration —
  `GrossReferent::Family` phantom variant in doc example corrected to the
  real `Group`/`World` mapping; module docs now match the impl)
- Bench law: 0 violations (i270 registered)
