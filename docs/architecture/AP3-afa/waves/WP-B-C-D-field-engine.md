---
name: wp-bcd
description: "AP3 WP-B/C/D: field state, dynamics gate, pathology operator, lambda gating. Era I parallel trio after WP-A freeze."
type: Work-Package-Brief
plan_id: AP3
wave: I
owned_paths:
  - "B: crates/mindstrata-development/src/{field,dynamics}.rs"
  - "C: crates/mindstrata-development/src/pathology.rs"
  - "D: crates/mindstrata-development/src/lambda.rs + content/polarity.rs graph states (types only)"
forbidden_paths: ["each other's files", "person/sim/social crates"]
depends_on: ["WP-A (frozen)"]
---

# WP-B — Field & Dynamics (`field.rs`, `dynamics.rs`)

`DevelopmentField` per 03-substrate §2. Dynamics = pure functions:

- `accumulate_press(field, catalysts, resonance) -> field` — catalyst uptake weighted by
  line attestation and resonance matrix; f64 only.
- `gate_and_advance(field, canon) -> field` — transcend-and-include: press→stage transition
  ONLY if fulfillment integral ≥ threshold AND no active Dark pathology on that line.
  Monotonic stage coordinate; press resets on advance.
- Resonance leakage: catalyst to line L leaks fraction w(L,M) into resonant line M.

Unit pins (synthetic inputs, no RNG): gradient monotonicity; NO-SKIP invariant; leakage
conservation bound; determinism (same input vector → bit-identical output).

# WP-C — Pathology Operator (`pathology.rs`)

Four kinds per 02-theory-map §3.3 table, each with theory citation in doc comment:

- `update_pathologies(field, fulfillment_history) -> field` — onset conditions per kind
  (Dark-Addiction: prolonged over-threshold clinging signal; Dark-Allergy: skipped-band
  detection via fulfillment gap; Golden-Addiction: high-line press while Red-band
  fulfillment < floor; Golden-Allergy: threshold met N consecutive windows without
  transition).
- `metabolize(field, agape_events, eros_events) -> field` — Agape decays Dark intensities;
  grounded Eros (call + foundation-check) resolves Golden; ungrounded Eros FEEDS
  Golden-Addiction (bypassing) — the polarity trap from pathologies.md.

Behavioral-signature pins are Era II (need living villagers); here pin the MATH: onset
truth tables on synthetic histories, metabolism monotonicity, bypass-trap directionality.

# WP-D — Lambda Gating (`lambda.rs`) (+ polarity type stubs)

- `frontier(lines) -> f64` (stage-weighted mode over attested lines).
- `gate_catalyst(catalyst, frontier, window) -> weight` — outside [Λ−1, Λ+1] → reduced
  weight (canon constant, starts 0.25); inside → full.
- Spiral property test: pathology-depressed Λ re-admits lower-window catalysis.

Polarity stubs for content/polarity.rs: `TensionState {Undiscovered, ActiveTension,
Reconciled}` + `refute()` transition (Reconciled→ActiveTension) — types only; engine is
WP-H2.

## Shared acceptance gates (all three)

- [ ] Pure-function discipline: no RNG, no clock, no global state
- [ ] clippy clean; unit pins green; golden byte-identical
- [ ] Post-merge contract hashes recorded in THIS file's freeze block

```
FROZEN(AP3/W-I): dynamics@<sha>, pathology@<sha>, lambda@<sha> — sign-off wave owner <date>
```

## Changelog

| wp | iter | note |
|---|---|---|
