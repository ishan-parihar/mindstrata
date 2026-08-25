# Needs Bands — Fulfillment Thresholds (skeleton)

Owner: DESIGN → consumed by SIM gating (WP-G1, FR-027). Companion to canon-inventory
row 1. Theory grounding: `docs/architecture/AP3-afa/03-substrate.md` §drives.

## Evidence

- Existing mechanics: need decay rates live in `SimParameters`
  (`system_need_decay_with_params`, crates/mindstrata-sim/src/systems/mod.rs:57–74);
  goal generation gates on fixed threshold 0.3/0.5/0.6/0.7 constants inline
  (`system_goal_generation`, systems/mod.rs:130–197). These are empirical, NOT
  theory-cited — they become CALIBRATION-PENDING(AP3) entries at canon.rs scaffold time.
- Motivation-context amplifications couple fear/anger/joy/sadness into drive pressures
  (Iteration-124 fix; see docs/MINDSTRATA_CURRENT_STATE.md historical note).

## Targets (to be probe-measured before values land)

| Drive | Threshold constant today | Theory question | Probe plan |
|---|---|---|---|
| hunger/thirst | 0.9 damage / 0.5 goal | survival-band floor from drive theory | i<iter>_needs_gate_delta |
| safety | 0.7 social-goal analog | scarcity vs threat distinction | same sweep |
| social | 0.3 retain / 0.7 generate | belonging pacing | same sweep |
| meaning/worship | 0.4 traditional gate | meaning deficit accumulation | same sweep |
| esteem/autonomy | derived at 2/3 meaning rate | band shape justification | sensitivity scan |

## Open questions

1. Should thresholds be per-personality modulated (ambition/extraversion already shift
   goal generation)?
2. Zero-at-zero invariant: all bands must be identity-neutral until a development field
   moves (FR-027).
