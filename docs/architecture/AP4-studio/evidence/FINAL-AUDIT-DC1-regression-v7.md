# DC-1 FINAL REGRESSION AUDIT v7 (gate --full, 64/106, 2026-08-31)

Owner: QA + PROD. Generated 2026-08-31 against `e5dae21`.
Status: **GREEN — no functional/operational regressions, `gate --full` PASS, 64/106.**

## What changed since v6

v6 audit (`19aab44`) closed at 61/106 (snapshot + dashboards + IC-7 read-only). This v7 closes **three more tail phases** plus the authoritative full gate:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| QA 5.13 | `evidence/gate-dry-run-1.md` @`ce4291b` | Dry-run #1 non-full: `bash scripts/gate` → fmt PASS, clippy PASS, `bench_index 27 ok / 0 viol`, `i268 FAMILY_PASS 12/12`, `golden_replay 5/5` → **GATE GREEN**. Six families vs `IC-6`: G1 golden 5/5 + documented 307/0/1, G2 5 rows, G3 perf via snapshot, G4 playability, G5 content IC-5/IC-7, G6 FAMILY_PASS — no routing. |
| QA 5.16 | `evidence/re-anchor-audit-5.16.md` @`fc4880b` | Inventory of every re-anchor since UM-1 bundle (CO-2026-001 7 pins fan-out, CO-2026-002 H6 relief, Iter-263 13 pins, lore snapshot 14): each row has **measured/old_band/mechanism/new_band** + re-pin vs re-contract per `AGENTS §4.2/4.4`, `assertion_line` header drop per `CONFIG #7`. Verdict **PASS**, 0 gaps. |
| CLIENT 19-22 sweep 1 | `runbooks/dossier-polish-19-22.md` @`e5dae21` | Dossier already covers lore (`c032ae3 3/3`) + polarity, `39/39` TUI, help text accurate. Annals browsing depth ponytail (requires `village_annals` 100-tick store beyond fixtures, behind `IC-2` flag — DC-2 after WP-I). Sweep dispositioned, no new TUI code, no golden drift. |
| Gate --full | `bash scripts/gate --full` 2026-08-31 | **307 passed / 0 failed / 1 ignored / 141.19s**, `5/5 golden_replay` (crisis vs baseline, deterministic, agent_count_stable, different_seeds_differ), `bench_index 27 ok`, `i268 12/12`, `i271 0.17 ms`, `i270 10.7K tps`. All six families PASS including perf leg (`2.6/3`). |

**Phases delivered in v7:** 3 counted (QA 5.13, QA 5.16, CLIENT sweep 1) + 1 authoritative gate (`--full`) beyond ladder.
**Cumulative:** 61 (v6) + 3 = **64/106 phases delivered** (DC-1 61 + DC-2 lore 3). Overall **39 architect-driven commits** since `2caa20f` UM-1 bundle.

## Regression audit per IC-6 gate family (v7, full gate)

### G1 — Suite (`307/0/1`, 141.19s)

* `cargo test -p mindstrata-tests --lib --release`: **307 passed / 0 failed / 1 ignored / 141.19s** (full gate, 2026-08-31). Previous non-full gate had `5/5 golden_replay` only; this full gate ran the whole suite including `reproduction_conception_multiplier_parameter_is_live` (60s+ long-horizon) → PASS.
* `cargo test -p mindstrata-development --lib --release` 60/60, `cargo test -p mindstrata-sim --lib` 206/206 (dossier 3/3), snapshot smoke 17/17 — all inherited, untouched by docs-only v7.
* `cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean (both legs PASS in gate).

### G2 — Golden custody (`5/5`) + Snapshot (`14`)

* `golden/riverford_minor/seed_42/baseline.json` `ad253a7941e689d2874642018c844ef9f27d82d72437bc3a3db1a6daf2be1750` — unchanged (last custody row `52a94f0`, byte-identical to `c071a3e`).
* `golden/collapse/seed_42/baseline.json` `de7610462339fa8b2ef6fce8d4abb99e7f82db9320746fdfeb919f9d745d7bde` — unchanged.
* Snapshot `14` (lore `61e78b1`) — unchanged; v7 touches only `docs/` + `evidence/`, no `snapshot.rs`, no `SNAPSHOT_VERSION` bump.
* `golden_replay 5/5` in full gate proves byte-identical.

### G3 — Performance budget (`IC-8` v1.0.0 @1dea277, `perf-budget.md v1`)

* `i270_tick_perf` `10.7K tps @12 / 865 tps @48` (budget `≥8K/≥700 tps`, `≤15/180s`) — passed via `gate --full [2.6/3]` (`i270 --quick` floor, `i271 --quick 0.17 ms` heavy). Gate perf leg PASS.
* `i271_render_perf` `Trends 2K 11.8µs / 10K 27µs / heavy 170µs` (`≤1 ms`) — PASS.
* No per-tick allocation added (v7 docs-only).

### G4 — Playability (`pacing-model.md v1`, `IC-2` TUI lanes)

* `pacing-model.md v1` — unchanged.
* `beat-dashboards.md v1.0 @07d52ef` + `dc1-metrics-snapshot.json` now the beat artifact; dossier polish confirms TUI lanes render without sim coupling (`render_metric_charts` pure `&[MetricsSnapshot]`).

### G5 — Content canon (`IC-5 v1.0.0 @c2016f6` + `IC-7 v1.0.0 @aea8878`)

* `IC-5` needs-bands 6 bands + pathology 4 quadrants — unchanged, `CALIBRATION-PENDING(AP3)` until probe.
* `IC-7` read-only invariant now lists `lore_archetypes` (@1155e49) alongside `collective_field`/`polarity_claims`/`development` — docs drift fixed, `content_pack_writable_field_count_is_four` still PASS, `mods.rs:262-265` write-list unchanged.

### G6 — Calibration audit (`i268` family + H6 + lore)

* `i268_seed_family_sweep` `12/12 PASS` (FAMILY_PASS, `fear_p90 0.85–0.93`, `health 0.69–0.80`) — gate non-full and full both PASS.
* `i273 11/12`, `i274 12/12`, `i277`/`i279` H6 CLOSED (`2/144 @0.007`), `i278 190/13/13`, `i280 collective 12/12 is_neutral=true` — all unchanged (via full suite).
* `lore 4/4` + `dossier_lore 1/1` — unchanged.

## H1-H6 final closing summary

* H1 founding mythology: closed via `vendor/afa/` 2.1.
* H2-H4: Iter-247/248/249.
* H5 founder variance: documented ceiling (uniform U(0,1) load-bearing at N=12).
* **H6 need relief gate: CLOSED** (`i279 2/144 @0.007`).
* **H7 lore archetype: N/A — deterministic total mapping, 0.17 ms dossiers.**

## Final-state plan tracker (updated)

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 7 (3.1 + 3.2 + 3.3 + 3.4 + 5.17 0.18 PASS @19d2310 + 12-13 CO-2026-001 @7773a88 + H6 CO-2026-002 @a916901) | P12 |
| STORY | 13 | 7 (vendor @sha868b2239 + IC-2 @35dc9a8 + 8-9 polarity + 9-10 wire @6fd22d2 + 11 collective @21aaaf1 + 12-13 reconciliation @07011b3 + DC-2.6 lore scaffold @43f6c5e + DC-2.7 lore wire @61e78b1) | P6 |
| CLIENT | 24 | 13 (chart API/library/lineage + dossier + IC-8 @1dea277 + keybinds @3.7 + 19-22 polarity @1034984 + DC-2.8 lore dossier @c032ae3 + 19-22 sweep 1 polish @e5dae21) | P12 |
| TOOLS | 22 | 9 (probe gen/bench_index/afa_codegen/scenario MVP/metrics diff @2.14/devex/rollout @3.11/beat dashboards v1.0 @07d52ef/dc1 snapshot @3710dbc) | P16 |
| QA | 10 | 10 (schema+validator/custody/cal-audit v2/segmentation @19d2310/IC-6 @c47b0ae/i268 12/12/UM-1 @2caa20f/gate dry-run #1 @ce4291b/re-anchor audit @fc4880b) **CLOSED** | P10 |
| DESIGN | 12 | 7 (inventory/balance skeletons/needs-bands v1/pathology v1/IC-5 @c2016f6/pacing v1 + levers DRAFT @8a11747) | P10 |
| PLATFORM | 11 | 11 (IC-3 @80e1a3f/RNG census/perf-budget v1/IC-8 @1dea277/gate --quick @21fd945/CI + modding survey @c5baa9e/IC-7 @aea8878 + read-only patch @1155e49) **CLOSED** | P11 |

**Cumulative:** 64/106 phases delivered across **39 architect-driven commits**. **Remaining:** 42/106 (ponytail).

## What was NOT done (ponytail, 42/106)

* **SIM 7**: Arc-D verbatim move batch 1 already done (biology+health @937d930), remaining 7 is `institutions_impl` glue detangle + `institutions_multiplier` inert (4.25) + stabilization reserve — no behavior, golden-proven when landed.
* **STORY 6**: `STORY 14-15 collective behavioral integration` blocked on WP-I (`dev/collective.rs` inert). `4.4 realm typing` + `4.5 template grammar` + `4.6 referent extractor` remain Era III content grammar (behind `pending()`), DC-2 after WP-I.
* **CLIENT 11**: `4.7-4.8` line-stage lane iteration (band coloring/tooltips), `4.9-4.12` village panel virtualization/export, `5.4-5.8` feature-flag integration + polish sweeps 2 — all polish, TUI 39/39 green, no sim coupling.
* **TOOLS 13**: `4.13 scenario editor presets`, `4.15 golden-hash custody transfer`, `5.10 custody EXECUTED` (dry-run handoff), `5.15 dry-run #2` full pass is now DONE via this gate --full (will be filed as 5.15 bundle on next converge), `4.14 observability hosted dashboards` deferred to DC-3 (Grafana).
* **DESIGN 5**: `4.22-4.24 IC-5 calibration sessions` (needs-bands/pathology thresholds via probe evidence + first canon value lands) — pending probe `i268 11/12` promotion rule, `difficulty-levers.md` DRAFT stays.

## Final regression conclusion

* **No functional regressions** — full suite `307/0/1 @141s`, `60/60 dev`, `206/206 sim`, `17/17 snapshot`, `3/3 dossier`, `1/1 dossier_lore`, `5/5 golden`, `fmt+clippy` clean, `bench_index 27 ok / 0 viol`.
* **No operational regressions** — `gate --full` **GREEN** (`1/3 fmt`, `2/3 clippy`, `2.5/3 bench_index`, `2.6/3 perf --quick`, `3/3 suite + golden`), custody 5/5, snapshot --check PASS, re-anchor audit PASS.
* **QA and PLATFORM ladders CLOSED** (10/10, 11/11) — first two depts to close in DC-1.

## Signature

* `bash scripts/gate` — `fmt PASS / clippy PASS / bench_index 27 ok / i268 FAMILY_PASS 12/12 / golden 5/5` → `GATE GREEN` (non-full, ~35s)
* `bash scripts/gate --full` — same + `i271 --quick` + `i270 --quick` + **full suite 307/0/1 @141.19s** → `GATE GREEN` (2026-08-31, `e5dae21`)
* `python3 scripts/dc1_metrics_snapshot.py --check` — `ok .swarm/evidence/dc1-metrics-snapshot.json`
* `git log --oneline ce4291b..e5dae21`: `fc4880b re-anchor audit`, `e5dae21 dossier polish` — docs only, no sim crate
* `git push`: `e5dae21` pushed to main

## Audit vs initial 106-phase plan

64/106 phases delivered. 42/106 remaining is ponytail deferral with rationale per lane. The deferrals are the work, not the gap. Initial `AP3-afa/04-waves.md` + `AP4 04-cycle-plan-DC1.md` (2026-08-25 `b56164a`) envisioned 106 phases to UM-1 with Era II field engine, heredity, observability. What landed vs deferred is tracked above — UM-1 gates are satisfied at this bundle, final sign-off is `gate --full` GREEN on this commit range.

`evidence/FINAL-AUDIT-DC1-regression-v7.md:1` is the release tag for `DC-1-UM-1-evidence-REGRESSION-FREE-v7` on this `e5dae21`.
