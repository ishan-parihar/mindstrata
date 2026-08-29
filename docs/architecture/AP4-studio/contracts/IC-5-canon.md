# IC-5 — Canon & Change-Orders

```yaml
provider: DESIGN (canon schema + values)
consumer: SIM, PSYCH, SOCIAL, WORLD, QA (gate FR-051)
frozen_at: commit 19d2310 (spec-source FR-051 + AGENTS.md §4 + calibration-audit-v2.md CA-1)
version: 1.0.0
status: RATIFIED-P0 (DESIGN owns VALUES only; structure owned by builder crates)
change_orders: []
```

## Purpose

The ONLY path by which a calibration constant changes: a DESIGN-authored
change-order citing probe evidence (measured value, old band, mechanism) in
`AGENTS.md` §4.2 form. Prevents "widen until green" re-pins and lucky-seed
re-anchors. Structure (types, fields) belongs to the builder crate; DESIGN
owns the numbers inside frozen structures.

## Surface (FROZEN v1.0.0 — process, not code)

### Canon location

* `docs/balance/**` — source of truth for tuned values (needs-band thresholds,
  pathology intensity curves, pacing targets). Code mirrors values via
  `..Default` + named constants; no hidden literals in passes.
* `crates/mindstrata-core/src/parameters.rs` and leaf `parameters`-like modules
  are the runtime projection — DESIGN change-orders update both canon doc and
  code constant together.

### Change-order record (required fields)

```yaml
id: CO-YYYY-NNN        # e.g. CO-2026-001
date: 2026-08-29
author: DESIGN
affected_canon: docs/balance/needs-bands.md#fulfillment-thresholds
code_site: crates/mindstrata-psych/src/motivation.rs:42
old_band: "[0.40, 0.55] at seed 42, 4K ticks (liveness pin)"
measured_value: "0.33 at seed 42, 4K ticks (probe i271_… raw 0.33)"
mechanism: "health-sync restoration lifted contempt to 0.073 (Iter-247 class)"
new_band: "[0.30, 0.38]  — re-pin, same contract"  # or re-contract with guard
probe_evidence: crates/mindstrata-benches/examples/i271_….rs
golden_impact: "none (structure unchanged) | behavioral — hashes X→Y via custody"
approved_by: QA + SIM owner
```

`mechanism` must name the system that moved the value (not "widen until green").
`golden_impact` is custody-aware (`runbooks/golden-replay-custody.md`).

### Re-pin vs re-contract (AGENTS.md §4.4)

* **Re-pin:** same contract, magnitude drifted — new band encloses probe value,
  old assertion still valid.
* **Re-contract:** old assertion tested something heredity/pacing legitimately
  invalidates — say so explicitly and guard the real invariant (liveness,
  positivity, decoupling bounds) instead.

### Needs-band / pathology specs (DESIGN 4.22/4.23, attached)

* `docs/balance/needs-bands.md` — fulfillment thresholds per `MotiveCategory`
  (19), cited to theory cells, with probe plans. Thresholds activate only after
  zero-at-zero verification (FR-027).
* `docs/balance/pathology-curves.md` — intensity curves for the 4-fold
  dark/golden × addiction/allergy model (Agape/Eros metabolism, FR-024).

Both docs are DESIGN-owned and versioned via this IC's `change_orders`.

## Guarantees

* No constant changes land without a `CO-` record quoting probe evidence.
* Lucky-seed re-pins are rejected: sweeps via `i268_seed_family_sweep`
  (`runbooks/calibration-audit-v2.md` CA-5, `suite-segmentation.md`) must show
  family pass ≥11/12 before a band can be claimed stable.
* Dead producers remain bugs even when tests pass on saturated states
  (equilibrium probes accompany liveness assertions — `calibration-audit-v2.md` CA-2).

## Obligations

* SIM/PSYCH/SOCIAL/WORLD must not retune a constant without routing through DESIGN
  via this IC; direct `parameters.rs` edits without a `CO-` are rejected at review.
* QA enforces FR-051 at gate: every diff touching a `CALIBRATION-PENDING` marker
  or a `parameters` constant must reference a `CO-` id in the commit message.

## Tests guarding this contract

* `calibration-audit-v2.md` CA-5 sweep (`i268_seed_family_sweep 12/12 PASS 2026-08-29`);
* `suite-segmentation.md` liveness pins (fear_p90_unsat, stress_unsat, health_alive);
* `golden-replay-custody.md` — behavioral goldens re-anchor only via QA custody.

## Changelog

| ver | change | approved |
|---|---|---|
| 1.0.0 | FROZEN — process + record shape from FR-051/AGENTS §4; needs-bands + pathology docs as attached canon | DESIGN+QA sign-off 2026-08-29 |
