# IC-5 first CO (DESIGN 4.24, 2026-08-31)

Owner: DESIGN. Generated 2026-08-31 against `75f5d0c` (77/106).
Status: **DRAFT — first CO shape filed, pending `i275` 11/12 promotion.**

## CO-2026-003 (draft)

* `id`: `CO-2026-003`, `date`: `2026-08-31`, `affected_canon`: `needs-bands.md` `work↔rest` band, `code_site`: `mindstrata-development/src/canon.rs:12`, `old_band`: `[0.40,0.60]`, `measured`: `0.43` (from `i268 FAMILY_PASS 12/12` `stress 0.16–0.38`), `probe`: `i268_seed_family_sweep`, `mechanism`: `pathology 4-quadrant fan-out dilutes dark_addiction Work suppression`, `new_band`: `[0.38,0.58]`, `golden`: `riverford_minor ad253a79…` re-anchored `0.3330→0.2807` via `CO-2026-001`.

## Why ponytail

First CO needs `i275_needs_band_calibration` `12/12` with `CV ≤0.35` per `calibration-audit-v2.md` — same blocker as `4.22`/`4.23`. Until `i275` promotes `11/12`, `IC-5` stays `v1.0.0` `CALIBRATION-PENDING(AP3)` and `difficulty-levers.md` stays `DRAFT`. Shape is filed so `i275` can land as patch, not design.

## Next

Land `CO-2026-003` for real when `i275` measures `needs decay σ` under `WP-I` — same batch as `STORY 14-15` collective behavioral.
