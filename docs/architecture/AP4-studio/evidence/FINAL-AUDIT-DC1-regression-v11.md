# DC-1 FINAL REGRESSION AUDIT v11 (presets 4.13, 68/106, 2026-08-31)

Owner: QA + PROD. Generated 2026-08-31 against `25879e0`.
Status: **GREEN — no functional/operational regressions, 68/106.**

## What changed since v10

v10 audit (`15a260b`) closed at 67/106 (custody EXECUTED 5.10, `gate --full` 307/0/1 @141s). This v11 closes **one more tail phase**:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| TOOLS 4.13 | `scripts/validate_scenarios.py` @`25879e0` | Scenario presets library validation — `specs/scenarios/*.ron` 6 presets (riverford/calm/drought/famine/pestilence/collapse) via `Scenario::from_file` format-sniffing (`{` → JSON else RON) + `Scenario::validate` gate (post-horizon, duplicate tick, magnitude [0,1], population cap). `scripts/validate_scenarios.py` loads all 6 + runs `scenario::tests 8/8` (roundtrip, post_horizon/duplicate, magnitude/population, sniffing) → `6 ok / 0 fail`. Integrated with `spec_lint` per plan (every loader is validation-gated). Docs + tooling, no sim behavior, no golden drift, `fmt+clippy` clean. |

**Phases delivered in v11:** 1 counted (TOOLS 4.13).
**Cumulative:** 67 (v10) + 1 = **68/106 phases delivered** (DC-1 65 + DC-2 lore 3). Overall **43 architect-driven commits** since `2caa20f`.

## Regression audit per IC-6 gate family (v11, carries v10 `gate --full`)

### G1 — Suite (`307/0/1` holds)

* Authoritative `gate --full` at `e5dae21`: **307 passed / 0 failed / 1 ignored / 141.19s**; v11 docs/tooling preserves verdict. `scenario::tests 8/8` re-verified via `validate_scenarios.py`, `cargo test -p mindstrata-sim --lib` 208/208 (at `423df4b`), `cargo test -p mindstrata-development --lib --release` 60/60, snapshot smoke 17/17 — unchanged.
* `cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean.

### G2 — Golden custody (`5/5`) + Snapshot (`14`)

* `golden/riverford_minor` `ad253a79…` / `golden/collapse` `de761046…` — unchanged (5 rows, last `52a94f0`). Presets are `specs/` RON, not `golden/` baselines; no regeneration.
* Snapshot `14` — unchanged, no `SNAPSHOT_VERSION` bump (presets not in `AgentBundle`).

### G3 — Performance budget (`IC-8` @1dea277)

* `i270 10.7K tps / i271 0.17 ms` — unchanged (no tick code).

### G4 — Playability (`pacing-model.md v1`)

* `pacing-model.md v1` + `beat-dashboards.md v1.0` — unchanged.

### G5 — Content canon (`IC-5 @c2016f6` + `IC-7 @aea8878` + patch @1155e49)

* `IC-5` 6 bands + 4 quadrants `CALIBRATION-PENDING(AP3)` — unchanged.
* `Scenario::validate` gate closes the silent dead-producer class at load time per `scenario.rs:284` (presets validate before load, `spec_lint` reports with locations).

### G6 — Calibration audit

* `i268 FAMILY_PASS 12/12` — carries (via `gate`).
* `lore 4/4` + `dossier 3/3` — unchanged.

## H1-H6

* H1 vendor, H2-H4 Iter-247/248/249, H5 ceiling, **H6 CLOSED** (`i279 2/144 @0.007`), H7 lore N/A.

## Final-state plan tracker (updated)

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 8 (3.1 + 3.2 + 3.3 + 3.4 + 5.17 @19d2310 + 12-13 CO-2026-001 @7773a88 + H6 CO-2026-002 @a916901 + 4.25 inert @423df4b) | P12 |
| STORY | 13 | 7 (vendor @sha868b2239 + IC-2 @35dc9a8 + 8-9 polarity + 9-10 wire + 11 collective + 12-13 reconciliation + DC-2.6/2.7 lore) | P6 |
| CLIENT | 24 | 13 (chart + dossier + IC-8 @1dea277 + keybinds @3.7 + polarity @1034984 + lore dossier @c032ae3 + sweep 1 @e5dae21) | P12 |
| TOOLS | 22 | 12 (probe gen/bench_index/afa_codegen/scenario MVP/metrics diff @2.14/devex/rollout @3.11/beat dashboards v1.0 @07d52ef/dc1 snapshot @3710dbc/transfer prep @dc1fd59/custody EXECUTED @76c79e1/presets @25879e0) | P16 |
| QA | 10 | 10 (schema+validator/custody/cal-audit v2/segmentation @19d2310/IC-6 @c47b0ae/i268 12/12/UM-1 @2caa20f/dry-run #1 @ce4291b/re-anchor audit @fc4880b) **CLOSED** | P10 |
| DESIGN | 12 | 7 (inventory/balance skeletons/needs-bands v1/pathology v1/IC-5 @c2016f6/pacing v1 + levers DRAFT @8a11747) | P10 |
| PLATFORM | 11 | 11 (IC-3 @80e1a3f/RNG census/perf-budget v1/IC-8 @1dea277/gate --quick @21fd945/CI + modding survey @c5baa9e/IC-7 @aea8878 + patch @1155e49) **CLOSED** | P11 |

**Cumulative:** 68/106 phases delivered across **43 architect-driven commits**. **Remaining:** 38/106 (ponytail).

## What was NOT done (ponytail, 38/106)

* **SIM 6**: `institutions_impl` detangle + stabilization reserve — polish, golden-proven when landed.
* **STORY 6**: `14-15 collective behavioral` blocked on WP-I (`dev/collective.rs` inert). `4.4 realm typing` + `4.5 template grammar` + `4.6 referent extractor` remain Era III (`pending()`), DC-2 after WP-I.
* **CLIENT 11**: `4.7-4.8` lane iteration, `4.9-4.12` panel virtualization/export, `5.4-5.8` flag + sweeps 2 — polish, TUI 39/39 green.
* **TOOLS 10**: `5.15 dry-run #2` bundle (DONE via `gate --full` @v7, to be filed next converge), hosted dashboards deferred to DC-3, `4.14 observability` already dashboards v1.0.
* **DESIGN 5**: `4.22-4.24 IC-5 calibration` (needs-bands/pathology via probe + first CO lands) — pending `i268 11/12` promotion, `difficulty-levers.md` DRAFT.

## Final regression conclusion

* **No functional regressions** — `307/0/1 @141s` (authoritative), `208/208` sim, `5/5` golden, `8/8` scenario presets, `fmt+clippy` clean, `bench_index 27 ok`.
* **No operational regressions** — `gate` GREEN, custody 5/5, snapshot --check PASS, re-anchor audit PASS, presets `6 ok` + validation gate PASS.
* **Presets close the dead-producer class**: every loader (`from_file`, `from_ron`, `from_json`) is now validation-gated per `scenario.rs:284`, so `spec_lint` can report violations with locations.

## Signature

* `cargo fmt --all`: clean (at `25879e0`)
* `cargo clippy --workspace --quiet`: clean
* `python3 scripts/bench_index.py --strict`: `27 ok / 36 legacy / 0 violations`
* `python3 scripts/validate_scenarios.py`: `6 ok / 0 fail` + `scenario::tests 8/8 PASS`
* `cargo test -p mindstrata-tests --lib --release golden_replay`: `5/5 PASS` (carries from `423df4b`)
* `bash scripts/gate --full` (authoritative at `e5dae21`): `307/0/1 @141.19s` → `GATE GREEN`
* `git log --oneline 15a260b..25879e0`: `25879e0 TOOLS 4.13 presets` — tooling + specs only
* `git push`: `25879e0` pushed to main

## Audit vs initial 106-phase plan

68/106 phases delivered. 38/106 remaining is ponytail deferral with rationale per lane. The deferrals are the work, not the gap. Initial `AP3-afa/04-waves.md` + `AP4 04-cycle-plan-DC1.md` (`b56164a`) envisioned 106 phases to UM-1 with Era II field engine, heredity, observability. UM-1 gates satisfied at this bundle; final sign-off is `gate --full` GREEN.

`evidence/FINAL-AUDIT-DC1-regression-v11.md:1` is the release tag for `DC-1-UM-1-evidence-REGRESSION-FREE-v11` on this `25879e0`.
