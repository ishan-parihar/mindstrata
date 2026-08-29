# IC-2 — Annals Trace Schema v1

```yaml
provider: SIM (trace emitter) + STORY (schema co-author)
consumer: CLIENT (TUI lineage/annals lanes), TOOLS (metrics diff), QA (UM-1 gates)
frozen_at: commit c2016f6 (IC-5 ratified; annals schema draft from AP3 WP-0B development field types)
version: 1.0.0
status: RATIFIED-P0 (annals as observability, not persistence; save schema remains IC-3)
change_orders: []
```

## Purpose

Stable observability record for per-agent development altitudes and village
annals — the ONLY channel by which CLIENT renders line-stage lanes and the
chronicle browser without coupling to sim internals. Trace is append-only,
deterministic, and feature-flagged until ratified; it never participates in
tick logic.

## Surface (FROZEN v1.0.0 — JSON lines, file `annals.jsonl` per run)

```json
{"tick": 1200, "agent": 3, "kind": "development_snapshot",
 "altitudes": {"cognitive": 1.2, "needs": 0.9, "health": 2.1},
 "pathology": {"dark_addiction": 0.00, "dark_allergy": 0.00, "golden_addiction": 0.00, "golden_allergy": 0.00},
 "catalysts_in_window": 2}
{"tick": 1200, "kind": "village_annals",
 "population": 12, "births": 0, "deaths": 0, "mean_altitude": 1.05}
```

Field notes:

* `altitudes` keys are `LineId` set from vendored `LineId` enum (11 individual
  lines in v1; 8 collective deferred to Era III). Values in stage units (f64),
  neutral `1.0` at founder init (FR-023 zero-at-zero).
* `pathology` quadrant intensities in `[0, ceiling]` per `OperatorParams`
  (all `0.00` until pathology wiring lands — identity at neutral).
* `catalysts_in_window` is the IC-1 event count in the 24-tick day window that
  produced this snapshot (observer harness accounting).
* Village record is one per 100-tick annals interval; agent records are per-agent
  per 100 ticks (decimated from daily development pass to keep trace ≤1 MB/100K).

File is written by `sim/snapshot.rs` annals emitter behind `--annals` flag;
CLIENT reads via `crates/mindstrata-tui/src/session.rs` trace loader — no direct
`Simulation` import in TUI (IC-8 boundary, STORY never edits CLIENT files).

## Guarantees

* Deterministic: same seed+horizon produces byte-identical `annals.jsonl`
  (f64 shadows quantized once at emission; line order is `LineId` enum order).
* Zero-at-zero: neutral founders with zero catalysts emit `1.0` altitudes and
  `0.00` pathology at every interval (existing `zero_at_zero_gates` pin).
* Append-only: new fields are optional with defaults; breaking shape change is
  a major version (IC change-order).

## Obligations

* CLIENT must tolerate missing optional fields (forward compat) and must not
  assume pathology non-zero before Era III.
* TOOLS metrics-diff treats annals as observability, not gate input until
  `IC-6` UM-1 gates promote it.

## Tests guarding this contract

* `zero_at_zero_gates` (SIM): neutral trace identity.
* `tick_is_deterministic` (SIM): annals byte-identical replay.
* `chronicle_render_smoke` (CLIENT, behind flag): TUI lanes render from fixture
  `annals.jsonl` without sim coupling.

## Changelog

| ver | change | approved |
|---|---|---|
| 1.0.0 | FROZEN — development_snapshot + village_annals JSON lines, 100-tick decimation, deterministic + zero-at-zero guarantees | STORY+SIM+CLIENT sign-off 2026-08-29 |
