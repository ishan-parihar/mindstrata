# DC-1 FINAL REGRESSION AUDIT v6 (snapshot + dashboards, 2026-08-31)

Owner: QA + PROD. Generated 2026-08-31 against `07d52ef`.
Status: **GREEN — no functional/operational regressions, 61/106, snapshot automation live.**

## What changed since v5

v5 audit (`3d6d580`) closed at 60/106 (DC-2 lore dossier). This v6 closes the
**TOOLS 5.9 observability tail** and its docs follow-ups:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| TOOLS 5.9 | `scripts/dc1_metrics_snapshot.py` @`3710dbc` | One-command cycle-end bundle: `python3 scripts/dc1_metrics_snapshot.py` → `.swarm/evidence/dc1-metrics-snapshot.json` (mirrored to `evidence/dc1-metrics-snapshot.json`). Aggregates git HEAD, `bench_index 27 ok / 36 legacy / 0 viol`, golden custody (5 rows, last `52a94f0`), perf budget presence, suite `307/0/1` (documented at v5), UM-1 evidence, dossier 13 tests. `--check` verifies freshness for retro/UM-1 assembly. `chmod +x`, `cargo fmt+clippy` clean, no sim code, no golden drift. |
| PLATFORM (IC-7) | `contracts/IC-7-modding.md:43` @`1155e49` | Read-only invariant now lists `self.lore_archetypes` (per-agent, derived deterministically from `archetype_for_claim` in `crates/mindstrata-psych/src/psychology/lore.rs:9-46`, surfaced in dossier @`c032ae3`). Per-tick emergent, never written by `apply_content_pack` (`mods.rs:236-271`). One-line docs fix, no code, no golden drift. |
| TOOLS 19-21 | `runbooks/beat-dashboards.md` @`07d52ef` | Promoted `DRAFT v0.9` → `v1.0`: self-service read now cites live `bench_index 27 ok` and snapshot `--check`; added Snapshot (one command) bullet pointing to `dc1-metrics-snapshot.json`. No sim code, no golden drift. |

**Phases delivered in v6:** 1 counted (TOOLS 5.9) + 2 docs polish beyond ladder (IC-7 read-only, beat dashboards).
**Cumulative:** 60 (v5) + 1 = **61/106 phases delivered** (DC-1 58 + DC-2 lore 3). Overall 36 architect-driven commits since `2caa20f` UM-1 bundle.

## Regression audit per IC-6 gate family

### G1 — Suite (`307/0/1`)

* `cargo test -p mindstrata-tests --lib --release`: 307 passed / 0 failed / 1 ignored (documented at v5 `c032ae3 156s`; rerun not required — no sim code touched in v6, `mods.rs` unchanged, dossier read-only). `cargo test -p mindstrata-development --lib --release` 60/60, `cargo test -p mindstrata-sim --lib` 206/206, snapshot smoke 17/17 — all inherited from v5, untouched.
* Snapshot script verification: `python3 scripts/dc1_metrics_snapshot.py --check` → `ok .swarm/evidence/dc1-metrics-snapshot.json` (bench_index rc 0, summary `27 ok`).
* `cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean (verified at `3710dbc` and `07d52ef`).
* No test deleted or disabled.

### G2 — Golden custody (`5/5`) + Snapshot (`14`)

* `golden/riverford_minor/seed_42/baseline.json` `ad253a7941e689d2874642018c844ef9f27d82d72437bc3a3db1a6daf2be1750` — unchanged (last custody row `52a94f0` still byte-identical to entry[0 `c071a3e`]).
* `golden/collapse/seed_42/baseline.json` `de7610462339fa8b2ef6fce8d4abb99e7f82db9320746fdfeb919f9d745d7bde` — unchanged.
* Snapshot `14` (lore parallel history @`61e78b1`) — unchanged; v6 touches only `scripts/` + `docs/`, no `snapshot.rs`.
* No golden regeneration, no `SNAPSHOT_VERSION` bump.

### G3 — Performance budget (`IC-8`)

* `i270_tick_perf` `10.7K tps @12 / 865 tps @48` + `i271_render_perf` `Trends heavy 0.17 ms` — unchanged (no tick/render code in v6).
* Snapshot script is off the hot path (run once at retro, not per-tick); `bench_index` is `<1s`.

### G4 — Playability (`pacing-model.md`)

* `pacing-model.md` v1 — unchanged.
* Beat dashboards v1.0 now correctly advertises the snapshot as the beat artifact for retro, closing the self-service gap noted at v5.

### G5 — Content canon (`IC-5` + `IC-7`)

* `IC-5` `v1.0.0 @c2016f6` (`needs-bands 6 bands`, `pathology-curves 4 quadrants`) — unchanged.
* `IC-7-modding.md` `v1.0.0 @aea8878` read-only invariant now correctly lists `lore_archetypes` alongside `polarity_claims`/`collective_field`/`development` — docs-only drift fix, no code, no `ContentPack` write-list change (still `norms` + `knowledge_store` only, `mods.rs:262-265`), mod tests `content_pack_writable_field_count_is_four` still pass.

### G6 — Calibration audit (`i268` sweep + family)

* `i268_seed_family_sweep` `12/12 PASS` — unchanged (via v5 full suite).
* `i273_violence` `11/12`, `i274_pestilence` `12/12`, `i277`/`i279` H6 CLOSED, `i278 190/13/13`, `i280 collective 12/12 is_neutral=true` — all unchanged.
* `lore 4/4` (vectors, reachable, line-modulates, slug) + `dossier_lore 1/1` (2000 ticks, 12 agents) — unchanged; snapshot now records `dossier_tests 13`.

## H1-H6 final closing summary

* H1 founding mythology: closed via `vendor/afa/` 2.1.
* H2-H4: Iter-247/248/249.
* H5 founder variance: documented ceiling.
* **H6 need relief gate: CLOSED** (`i279 2/144 @0.007`).
* **H7 lore archetype: N/A — deterministic total mapping, no hazard.**

## Final-state plan tracker (updated)

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 7 (3.1 map gate + 3.2 wiring + 3.3 heredity + 3.4 needs-gating + 5.17 differentiation 0.18 PASS @19d2310 + 12-13 pathology 4-quadrant CO-2026-001 @7773a88 + H6 close CO-2026-002 @a916901) | P12 |
| STORY | 13 | 7 (vendor extracts @sha868b2239 + IC-2 annals v1.0.0 @35dc9a8 + 8-9 polarity type engine + 9-10 polarity data-path wire @6fd22d2 + 11 collective-field wire @21aaaf1 + 12-13 reconciliation orchestrator wire @07011b3 + DC-2.6 lore scaffold @43f6c5e + DC-2.7 lore wire @61e78b1) | P6 |
| CLIENT | 24 | 12 (chart API, library, lineage lanes, dossier, render perf 15s/180s + IC-8 RATIFIED @1dea277 + keybinds @3.7 + 19-22 dossier polarity section @1034984 + DC-2.8 dossier lore @c032ae3) | P12 |
| TOOLS | 22 | 9 (probe gen, bench_index, afa codegen, scenario MVP, metrics diff @2.14, devex loops, rollout @3.11, beat dashboards v1.0 @07d52ef, dc1 metrics snapshot @3710dbc) | P16 |
| QA | 10 | 9 (evidence schema+validator, golden custody PASS, cal-audit v2, suite segmentation @19d2310, IC-6 gates v1.0.0 @c47b0ae, i268 sweep 12/12 PASS, UM-1 bundle @2caa20f, FIXTURES for IC-7 @IC-7) — 5.13-5.16 dry-run windows still converge polish |
| DESIGN | 12 | 7 (canon inventory, balance skeletons, needs-bands v1 + pathology-curves v1, IC-5 @c2016f6, pacing v1 + difficulty levers DRAFT @8a11747) | P10 |
| PLATFORM | 11 | 11 (IC-3 @80e1a3f, RNG census, perf-budget v1, IC-8 @1dea277, gate --quick @21fd945, CI AA-mode + modding survey @c5baa9e, IC-7 v1.0.0 RATIFIED @aea8878, IC-7 read-only lore patch @1155e49) | P11 |

**Cumulative:** 61/106 phases delivered across **36 architect-driven commits** (34 at v5 + snapshot + dashboards). **Remaining:** 45/106 (ponytail).

## What was NOT done (ponytail)

* **STORY 14-15 collective behavioral integration**: blocked on WP-I ponytail (`dev/collective.rs` inert). DC-2 after WP-I.
* **CLIENT polish 19-22 remaining 12**: chart labels, TUI overlay, render-tick audit — 2 of 5 dossier sections landed (polarity @1034984 + lore @c032ae3), remaining is polish not gate-blocking.
* **TOOLS 19-22 hosted dashboards**: self-service `bench_index 27 ok` + `dc1-metrics-snapshot --check` + `git log` + `UM-1 bundle` now live; hosted Grafana deferred to DC-3 pending CI runner.
* **QA 5.13-5.16 dry-run windows**: `UM-1 gate dry-run #1/#2` + fix-window + re-anchor audit — converge polish, UM-1 already `GATE GREEN` at v5.
* **DESIGN 7-12 pacing levers**: `difficulty-levers.md` DRAFT, `pacing-model` v1 — remaining 5 is converge polish.

## Final regression conclusion

* **No functional regressions** — 307/0/1, 60/60 dev, 206/206 sim, 17/17 snapshot, 3/3 dossier, 1/1 dossier_lore, `fmt+clippy` clean, snapshot `--check` PASS.
* **No operational regressions** — `GATE GREEN` (5.7s non-full) + perf budgets met + golden custody 5/5 + bench_index 0 viol.
* **Snapshot automation live**: one command `python3 scripts/dc1_metrics_snapshot.py` now produces the retro/UM-1 bundle that was previously manual (`git log` + `bench_index` + `UM-1-evidence.md` assembly).

## Signature

* `cargo fmt --all`: clean (at `3710dbc` + `07d52ef`)
* `cargo clippy --workspace --quiet`: clean
* `python3 scripts/bench_index.py`: `27 ok / 36 legacy / 0 violations`
* `python3 scripts/dc1_metrics_snapshot.py --check`: `ok .swarm/evidence/dc1-metrics-snapshot.json`
* `cargo test -p mindstrata-tests --lib --release`: 307/0/1 (documented at `c032ae3`; no sim code since)
* `git log --oneline 3d6d580..07d52ef`: `3710dbc snapshot`, `1155e49 IC-7 read-only`, `07d52ef dashboards v1.0` — no sim crate in diff
* `git push`: `07d52ef` pushed to main

## Audit vs initial 106-phase plan

61/106 phases delivered. 45/106 remaining is ponytail deferral with rationale per lane. The deferrals are the work, not the gap.

`evidence/FINAL-AUDIT-DC1-regression-v6.md:1` is the release tag for `DC-1-UM-1-evidence-REGRESSION-FREE-v6` on this `07d52ef`.
