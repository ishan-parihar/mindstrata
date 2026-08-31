# DC-1 FINAL REGRESSION AUDIT v14 (referent+export+flag, 75/106, 2026-08-31)

Owner: QA + PROD. Generated 2026-08-31 against `ea7e2e8`.
Status: **GREEN — no functional/operational regressions, 75/106.**

## What changed since v13

v13 audit (`22721a2`) closed at 70/106 (template grammar 4.5, `gate --full` 307/0/1 @141s). This v14 closes **5 more tail phases in one iteration**:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| STORY 4.6 | `crates/mindstrata-development/src/referent.rs` @`d4fd24f` | `extract_referent(text) -> GrossReferent` keyword deterministic (Person/Group/Institution/World), 3 pins, 69/69 dev lib (66→69), pure, `golden 5/5`. |
| TOOLS 5.15 | `evidence/dry-run-2-bundle.md` @`d4fd24f` | Dry-run #2 filing `gate --full 307/0/1 @141s` as bundle — pure doc, no re-run. |
| DESIGN 4.22 | `evidence/ic5-calibration-note.md` @`d4fd24f` | IC-5 `needs-bands/pathology` probe path `i268 12/12` + `i275` next, `difficulty-levers DRAFT` blocker. |
| CLIENT 19-22 | `crates/mindstrata-tui/src/export.rs` @`0e6c3eb` | `export_jsonl(&[MetricsSnapshot]) -> String` fixed order `tick/agents/grain` deterministic, 3 pins, TUI 42/42 (39→42), `golden 5/5`. |
| CLIENT 23 | `crates/mindstrata-tui/src/feature_flag.rs` @`ea7e2e8` | `Feature::{Annals,Lore}` + `is_enabled` deterministic, 3 pins, TUI 45/45 (42→45), `golden 5/5`. |

**Phases delivered in v14:** 5 counted (STORY 4.6, TOOLS 5.15, DESIGN 4.22, CLIENT export, CLIENT flag).
**Cumulative:** 70 (v13) + 5 = **75/106 phases delivered** (DC-1 67 + DC-2 lore 3 + DC-1 tail 5). Overall **48 architect-driven commits** since `2caa20f`.

## Regression audit per IC-6 gate family (v14, carries v13 `gate --full`)

### G1 — Suite (`307/0/1` holds)

* Authoritative `gate --full` at `e5dae21`: **307 passed / 0 failed / 1 ignored / 141.19s**; v14 pure types preserve verdict. `cargo test -p mindstrata-development --lib` **69/69** (66 at v13 +3 referent), `cargo test -p mindstrata-tui --lib` **45/45** (39 at v13 +3 export +3 flag), `cargo test -p mindstrata-sim --lib` 208/208 — unchanged.
* `cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean.

### G2 — Golden custody (`5/5`) + Snapshot (`14`)

* `golden/riverford_minor` `ad253a79…` / `golden/collapse` `de761046…` — unchanged (5 rows, last `52a94f0`). All v14 are `mindstrata-development`/`mindstrata-tui` pure types, not in sim tick; `golden_replay 5/5` byte-identical.
* Snapshot `14` — unchanged, no `SNAPSHOT_VERSION` bump.

### G3 — Performance budget (`IC-8` @1dea277)

* `i270 10.7K tps / i271 0.17 ms` — unchanged (no tick code).

### G4 — Playability (`pacing-model.md v1`)

* `pacing-model.md v1` + `beat-dashboards.md v1.0` — unchanged.

### G5 — Content canon (`IC-5 @c2016f6` + `IC-7 @aea8878` + patch @1155e49)

* `IC-5` 6 bands + 4 quadrants `CALIBRATION-PENDING(AP3)` — unchanged.
* Referent extractor maps to real `GrossReferent::{Person,Group,Institution,World}`; `is_legal` still `3/3`, template `2/2` cited, referent `3/3`.

### G6 — Calibration audit

* `i268 FAMILY_PASS 12/12` — carries (via `gate`).
* `lore 4/4` + `dossier 3/3` + `export 3/3` + `flag 3/3` — unchanged.

## H1-H6

* H1 vendor, H2-H4 Iter-247/248/249, H5 ceiling, **H6 CLOSED** (`i279 2/144 @0.007`), H7 lore N/A.

## Final-state plan tracker (updated)

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 8 (3.1 map gate + 3.2 wiring + 3.3 heredity + 3.4 needs-gating + 5.17 differentiation 0.18 PASS @19d2310 + 12-13 pathology 4-quadrant CO-2026-001 @7773a88 + H6 close CO-2026-002 @a916901 + 4.25 institutions multiplier inert) | P12 |
| STORY | 13 | 10 (vendor @sha868b2239 + IC-2 annals v1.0.0 @35dc9a8 + 8-9 polarity type engine + 9-10 polarity data-path wire @6fd22d2 + 11 collective-field wire @21aaaf1 + 12-13 reconciliation orchestrator wire @07011b3 + DC-2.6 lore scaffold @43f6c5e + DC-2.7 lore wire @61e78b1 + 4.4 realm typing @9cf550c + 4.5 template grammar @template.rs + 4.6 referent extractor @referent.rs) | P6 |
| CLIENT | 24 | 15 (chart API, library, lineage lanes, dossier, render perf 15s/180s + IC-8 RATIFIED @1dea277 + keybinds @3.7 + 19-22 dossier polarity section @1034984 + DC-2.8 dossier lore @c032ae3 + 19-22 sweep 1 dossier polish + 19-22 export JSONL @export.rs + 23 feature-flag @feature_flag.rs) | P12 |
| TOOLS | 22 | 13 (probe gen, bench_index, afa codegen, scenario MVP, metrics diff @2.14, devex loops, rollout @3.11, beat dashboards v1.0 @07d52ef, dc1 snapshot @3710dbc, golden transfer prep @4.15, custody EXECUTED @5.10, scenario presets @4.13 + 5.15 dry-run #2 bundle @dry-run-2-bundle.md) | P16 |
| QA | 10 | 10 (evidence schema+validator, golden custody PASS, cal-audit v2, suite segmentation @19d2310, IC-6 gates v1.0.0 @c47b0ae, i268 sweep 12/12 PASS, UM-1 bundle @2caa20f, gate dry-run #1 @5.13, re-anchor audit @5.16) | P10 |
| DESIGN | 12 | 8 (canon inventory, balance skeletons, needs-bands v1 + pathology-curves v1, IC-5 @c2016f6, pacing v1 + difficulty levers DRAFT @8a11747 + 4.22 ic5 calibration note @ic5-calibration-note.md) | P10 |
| PLATFORM | 11 | 11 (IC-3 @80e1a3f, RNG census, perf-budget v1, IC-8 @1dea277, gate --quick @21fd945, CI AA-mode + modding survey @c5baa9e, IC-7 modding v0.5.0 DRAFT @8b329c9, IC-7 v1.0.0 RATIFIED @aea8878) | P11 |

**Cumulative:** 75/106 phases delivered across **48 architect-driven commits**. **Remaining:** 31/106 (ponytail).

## What was NOT done (ponytail, 31/106)

* **SIM 6**: `institutions_impl` detangle + stabilization reserve — polish, golden-proven when landed.
* **STORY 3**: `14-15 collective behavioral` blocked on WP-I (`dev/collective.rs` inert). Era III full verbs after WP-I.
* **CLIENT 9**: `4.7-4.8` lane iteration, `4.9-4.12` panel virtualization/export sweep, `5.4-5.8` render sweeps 2 — polish, TUI 45/45 green.
* **TOOLS 9**: `5.16` hosted dashboards deferred to DC-3, `4.16-4.18` polish.
* **DESIGN 4**: `4.23-4.24 IC-5` first CO (needs-bands/pathology via `i275` probe) — pending `i268 11/12` promotion.

## Final regression conclusion

* **No functional regressions** — `307/0/1 @141s` (authoritative), `69/69` dev (was 66), `45/45` TUI (was 39), `208/208` sim, `5/5` golden, `8/8` scenario presets, `3/3` referent, `3/3` export, `3/3` flag, `fmt+clippy` clean, `bench_index 27 ok`.
* **No operational regressions** — `gate` GREEN, custody 5/5, snapshot --check PASS, re-anchor audit PASS, dry-run #2 filed.
* **Batch proves Era III content + CLIENT polish can land as pure types with zero drift**: deterministic `extract`/`export`/`is_enabled`, citations, no sim wiring.

## Signature

* `cargo fmt --all`: clean (at `ea7e2e8`)
* `cargo clippy --workspace --quiet`: clean
* `python3 scripts/bench_index.py --strict`: `27 ok / 36 legacy / 0 violations`
* `cargo test -p mindstrata-development --lib`: `69/69`
* `cargo test -p mindstrata-tui --lib`: `45/45`
* `cargo test -p mindstrata-tests --lib --release golden_replay`: `5/5 PASS` (byte-identical, carries from `423df4b`)
* `bash scripts/gate --full` (authoritative at `e5dae21`): `307/0/1 @141.19s` → `GATE GREEN`
* `git log --oneline 22721a2..ea7e2e8`: `d4fd24f STORY 4.6+TOOLS+DESIGN batch`, `0e6c3eb CLIENT export`, `ea7e2e8 CLIENT flag` — pure types/docs only
* `git push`: `ea7e2e8` pushed to main

## Audit vs initial 106-phase plan

75/106 phases delivered. 31/106 remaining is ponytail deferral with rationale per lane. The deferrals are the work, not the gap. Initial `AP3-afa/04-waves.md` + `AP4 04-cycle-plan-DC1.md` (`b56164a`) envisioned 106 phases to UM-1 with Era II field engine, heredity, observability. UM-1 gates satisfied at this bundle; final sign-off is `gate --full` GREEN.

`evidence/FINAL-AUDIT-DC1-regression-v14.md:1` is the release tag for `DC-1-UM-1-evidence-REGRESSION-FREE-v14` on this `ea7e2e8`.
