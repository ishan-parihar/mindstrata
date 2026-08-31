# IC-5 calibration note (DESIGN 4.22, 2026-08-31)

Owner: DESIGN. Generated 2026-08-31 against `22721a2` (70/106).
Status: **DRAFT — needs-bands/pathology via probe, pending 11/12 promotion.**

## Current IC-5 state

* `contracts/IC-5-canon.md` @`c2016f6` — 6 needs bands + 4 quadrants `CALIBRATION-PENDING(AP3)`, `CO-` shape `id/date/affected_canon/code_site/old_band/measured/probe/mechanism/new_band/golden`.
* `docs/balance/needs-bands.md` v1 + `pathology-curves.md` v1 — skeletons, `CALIBRATION-PENDING` constants in `mindstrata-development`.

## Probe path

* `i268_seed_family_sweep` `12/12 FAMILY_PASS` (11/12 promotion rule per `difficulty-levers.md` DRAFT @`8a11747`).
* Next: `i275_needs_band_calibration` (36-legacy probe promotion) will measure `needs decay σ` and `pathology growth` under `IC-5` CO-.

## What lands this note

* Records that `realm.rs`/`template.rs`/`referent.rs` are pure types with no tick impact — no `CO-` needed, `is_legal` law pinned `3/3`, template citations `2/2`, referent `3/3`.
* Behavioral fold (institutions/field engine) will be the first `CO-2026-003` when it moves a pin.

## Blocker to v1.0

`DESIGN 4.22-4.24` need a 12-seed sweep with `CV ≤0.35` + `pass-rate ≥11/12` (per `calibration-audit-v2.md`). Until then, `difficulty-levers.md` stays `DRAFT`.
