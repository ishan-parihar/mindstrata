# Re-anchor Audit — AGENTS §4.2 Completeness (QA 5.16)

**Commit:** `ce4291b` (62/106)
**Scope:** Every calibration re-anchor since `2caa20f` UM-1 bundle (covers Iter-263 trapezoid through CO-2026-002).

## Rule

AGENTS.md §4.2: every re-anchor comment must carry **measured value, old band, mechanism** ("health-sync restoration lifted contempt to 0.073"), never "widen until green." Re-contract vs re-pin distinction (AGENTS §4.4) must be explicit.

## Inventory

| Change-order | File | Old band | Measured | Mechanism | New band | Type |
|---|---|---|---|---|---|---|
| CO-2026-001 @7773a88 | `crates/mindstrata-tests/...` kinship_penalty | `0.3330` pin | `0.2807` (12-seed i274 family) | 4-quadrant fan-out dilutes dark_addiction 100%→25% → family-formation suppression down, dark_allergy absorbs 25% | `0.25–0.30` | re-contract (deliberate fan-out, 7 pins) |
| CO-2026-001 | `crates/mindstrata-tests/...` 6 other pins | various | ±0.5% magnitudes | same fan-out dilution of Work suppression cascade | documented per pin | re-contract |
| CO-2026-002 @a916901 | `crates/mindstrata-sim/...` H6 secondary relief | 0.0 | `2/144 @0.007` via i279 12-seed sweep | secondary relief gate CLOSED (needs 2.19) | `≥1/144` | re-pin (measured, mechanism: relief gate) |
| Iter-263 trapezoid | 13 pins @16238f5 | various | probed via `probe_fixes.rs` | founder variance IS behavioral budget at N=12 | re-anchored bands | re-pin (H5 debt) |
| Lore wire @61e78b1 | snapshot `14` | v13 | deterministic `archetype_for_claim` total mapping | new parallel history (snapshot 14) | `v14` | new (not re-anchor) |
| Dossier lore @c032ae3 | `chronicle.rs:312` | none | `lore archetypes: N across M` | read-only total | new test | new (no re-anchor) |

## Completeness check

* Every row above has **measured, old_band, mechanism, new_band** in its commit message and associated probe file (`i273`/`i274`/`i279`, `probe_fixes.rs`, `i272`).
* No "widen until green" — each band shift cites a mechanism (fan-out, relief gate, H5 variance, deterministic mapping).
* Re-contract vs re-pin is explicit: CO-2026-001 is re-contract (old assertion tested heredity/pacing that fan-out legitimately invalidates, guarding liveness instead), others are re-pins.
* Insta snapshots (`7 snapshots @CO-2026-001`) regenerated via `.snap.new → .snap` drop of `assertion_line` header (per `CONFIG_VALUES #7` — no `cargo insta` on host).

## Verdict

**PASS** — all re-anchors carry required fields; 0 gaps. Open question: `difficulty-levers.md` DRAFT thresholds still `CALIBRATION-PENDING(AP3)` until probe lands — filed to DC-2 (not a re-anchor failure).

## Gate

This audit satisfies QA 5.16 `IC-6 §4` and unblocks DC-1 close. Next: `gate --full` dry-run #2 (5.15) as final UM-1 bundle.
