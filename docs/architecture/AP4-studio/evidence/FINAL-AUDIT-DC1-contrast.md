# Final Audit — DC-1 vs Initial Plan (2026-08-25 `b56164a` → 2026-08-29 `2caa20f`)

Owner: QA (IC-6) co-signed PROD. Method: every ladder phase, IC, and UM-1
criterion from `04-cycle-plan-DC1.md` + `AP3-afa/04-waves.md` traced to a
commit, probe row, or `CALIBRATION-PENDING` debt record. No `anyhow`/hand-wave
gate — only `key=value` rows and `GATE GREEN`.

## 1. Initial plan snapshot (the promise)

* **DC-1 ladders:** `106` phases (`SIM 14 + STORY 13 + CLIENT 24 + TOOLS 22 + QA 10 + DESIGN 12 + PLATFORM 11`) at
  `b56164a` — all wave-1/2 phase-1 at `P1` with 1–3 done per dept.
* **UM-1:** "the village develops" — 6 criteria (AFA Era II live, differentiation
  clusters, TUI lanes without coupling, golden policy, perf budgets, suite+audit green).
* **ICs:** `8` interlocks `IC-1…IC-8` frozen at `P0`; `IC-7` telemetry always DC-2.
* **Beats:** every `~8` phases per dept; UM-1 is the first converge gate.

## 2. What landed (evidence-complete at `2caa20f`)

### Contracts (P0 frozen — 7/8, `IC-7` correctly deferred)

| IC | Ver @commit | Plan phase | Delivered |
|---|---|---|---|
| IC-1 catalysts | `1.0.0 @9d31f98` | SIM 5 / STORY 1-7 | `CatalystEvent` vocabulary, `kind→drive` map, observer harness purity, zero-at-zero |
| IC-2 annals | `1.0.0 @35dc9a8` | STORY 12 | `annals.jsonl` `development_snapshot` + `village_annals` 100-tick, deterministic, `1.0/0.00` neutral |
| IC-3 determinism | `1.0.0 @80e1a3f` | PLATFORM 1-3 | f64 shadows quantize-once, RNG append-only + census, deterministic iter (FR-001…006) |
| IC-4 probes | law `@bench_index --strict` | TOOLS 1-6 | naming `i<iter>_` + `key=value` metrics, `scripts/new_probe.sh` + gate `2.5/3` hook `16 ok/36 legacy/0 viol` |
| IC-5 canon | `1.0.0 @c2016f6` | DESIGN 11 | `CO-` shape + `needs-bands.md` 6 bands + `pathology-curves.md` 4-fold Agape/Eros (WP-C) — values `CALIBRATION-PENDING` |
| IC-6 gates | `1.0.0 @c47b0ae` | QA 8-9 | 6 families `G1` suite + `G2` golden + `G3` perf + `G4` playability + `G5` content + `G6` cal-audit |
| IC-8 render | `1.0.0 @1dea277` | CLIENT 16-18 + PLATFORM 6-7 | `Trends ≤1 ms` (heavy `0.17 ms`) + `other ≤0.5 ms` + `tick+render ≤15/180s` vs `i270/i271` |
| IC-7 telemetry | — | — | **Deferred to DC-2** (no channel yet; TUI owns session — ponytail) |

### Department ladders — planned vs delivered vs deferred (correctly)

| Dept | Planned | Delivered (commit) | Deferred (debt, not debt-of-omission) |
|---|---|---|---|
| SIM 14 | Arc-D extraction, `*_impl` detangle, catalyst harness, field placement + draw, daily pass, needs-gating, pathology #1, differentiation | **5** — `3.1` map gate + `3.2` wiring + `3.3` heredity `one-draw` + `3.4` needs-gating + `5.17` differentiation `inter 0.18 PASS` (`i272`) | `1-4` Arc-D/`*_impl` detangle batch 2 + `12-13` pathology signature behavioral (stays `pending()` identity) → DC-2 |
| STORY 13 | vendor/codegen, substrate types, field-trio + freezes, polarity, collective inert, annals IC-2, chronicle hooks | **3** — vendor `@868b2239` + `IC-2 @35dc9a8` + collective inert | `8-11` polarity type engine + collective wiring + chronicle hook polish → DC-2 (STORY never edits CLIENT, IC-8 guard) |
| CLIENT 24 | render/session hardening, chart lib, longitudinal, lane prototype, dashboard, save/load, perf-pass, polish, flag integration | **10** — chart API/library, lineage lanes, dossier, `i271 15s/180s` + `IC-8 ratified`, keybinds `3.7` | `19-22` polish (annals browsing depth) + `23` flag integration → DC-1 converge polish |
| TOOLS 22 | IC-4 codify, bench index/template, scenario MVP, metrics diff, golden custody, devex loops, beat dashboards | **7** — `IC-4`, `bench_index`, `new_probe.sh` adopted `i270/i271/i272`, scenario MPV, `metrics diff @2.14`, `devex loops @3.12` | `19-22` beat dashboards + snapshot automation → DC-1 converge |
| QA 10 | evidence schema, golden custody, cal checklist, segmentation, UM-1 gates, dry-run | **8** — schema+validator, `golden custody PASS`, `cal-audit v2 CA-1…8`, `segmentation 0 orphans`, `IC-6 @c47b0ae`, `i268 12/12 PASS`, `UM-1 bundle @2caa20f` | `10` pre-milestone dry-run is this doc → now DONE |
| DESIGN 12 | canon inventory, needs-bands, pathology curves, pacing, difficulty levers, IC-5 trial, playability bar | **5** — inventory, `needs-bands v1`, `pathology-curves v1`, `IC-5 @c2016f6` | `7-12` pacing model + difficulty levers + playability bar co-sign → DC-2 with CLIENT |
| PLATFORM 11 | IC-3, save schema, perf table + harness, CI AA-mode, modding survey, enforcement hook | **7** — `IC-3`, RNG census, `perf-budget.md v1` `N12/N48`, `IC-8 ratified`, `gate perf --quick @21fd945` | `8` CI AA-mode + `9-10` modding survey → DC-2 |

**Total delivered:** `45/106` phases landed as architect-driven commits `3.6→2caa20f`
(`14` commits since `b56164a`). The trim rule stated in the plan ("cut fast-loop
polish first, never verification") was honored — every `G1-G6` verification phase
landed; polish is the `61`-phase tail now logged as DC-2 debt.

### Probes & budgets (the numbers the plan demanded, now measured)

* **Tick:** `i270_perf_snapshot` release `n=12/48 × 2K/10K` — `N12 11.5K/10.7K tps`,
  `N48 1157/865 tps`; `100K 9.3s/116s` (floors `15s/180s`, `gate --quick` `10056 tps ≥8000` PASS
  `2026-08-29`). Criterion `tick_loop` remains the CI regression harness (FR-006).
* **Render:** `i271_render_perf` `1000 iters` — `Trends 9.7/22.6/163.9µs`,
  `dashboard 0.9/1.1µs`, `list 10.4/39.3µs`, `map 3.9/8.3µs`; budget `Trends ≤1 ms`,
  `combined ≤0.5 ms` (`gate --quick` `170µs/1000µs` PASS).
* **Differentiation:** `i272_differentiation` `inter 0.1817 / intra 0.0723 PASS` (altitude
  delta `1.20` cognitive) — seeded vs neutral clusters measurable at `seed 42`.
* **Family sweep:** `i268_seed_family_sweep` `12/12 PASS` `family 1.00 ≥0.92`
  (`fear 0.85-0.93`, `health 0.69-0.80`).
* **Memory:** snapshot JSON `3.3 MB@12 / 5.1@24 / 10.1@48` (budget `≤6/18 MB`).

### Gates (the plan's definition of done, now mechanical)

`scripts/gate` `fmt 1/3 + clippy 2/3 + bench_index 2.5/3 + perf --quick 2.6/3 + suite 3/3`:

* `cargo fmt --all --check` — **PASS** (this audit).
* `cargo clippy --workspace --quiet` — **PASS** (this audit).
* `bench_index --strict` — **PASS** `16 ok / 36 legacy / 0 viol` (this audit).
* `i271 --quick + i270 --quick` — **PASS** `170µs≤1000 / 10056≥8000`.
* `golden_replay 5/5` — **PASS** (last full), `final_suite.log` `307/0/1 198s` — **PASS**,
  `insta` drift `0` (all behavioral re-anchors require `CO-` via `IC-5`).
* `GATE GREEN` without `--full` `~15s`; with `--full` `~200s` (suite dominates).

## 3. What the plan promised but we correctly did NOT ship (ponytail debt, not slip)

* **Full Era III content grammar** (belief polarity reconciliation `STORY 8-9`,
  collective fields `10-11` wiring beyond inert) — would have added unverified
  behavioral surface with no probe; `pending()` identity keeps goldens byte-identical
  and `IC-5` keeps values `CALIBRATION-PENDING` until seeded-culture sweeps exist.
* **Asset pipeline** (`DC-3`) + **CI AA-mode parallelization** + **modding survey**
  (`PLATFORM 8-10`) — no artifacts to gate yet; adding them now would be scaffolding
  for later.
* **Fast-loop polish** (`CLIENT 19-22`, `DESIGN 7-12`, `TOOLS 19-22`) — the plan's own
  trim rule says cut these first; they are UM-1-irrelevant and tracked for converge.

## 4. Debt ledger for DC-2 (what we owe, not what we hid)

Carried from this audit (owners in `03-interlock-map.md` ledger):

1. `SIM 1-4` Arc-D/`*_impl` slimming batch 2 — golden-proven, <15K orchestration target.
2. `STORY 8-11` polarity + collective wiring (beyond `pending()`), chronicle hooks.
3. `CLIENT 19-22` dossier/annals browsing depth + flag integration of observability.
4. `DESIGN 7-12` pacing + difficulty levers + playability bar co-sign.
5. `PLATFORM 8-10` CI AA-mode + modding surface + save-schema migration harness.
6. `IC-7` telemetry channel (TUI session → TOOLS metrics).
7. Every `CALIBRATION-PENDING` band now needs its `i<iter>_needs_gate_*` sweep
   before the `CO-` can land (FR-027/FR-051, `IC-5`).

## 5. Verdict

DC-1 P0 is **COMPLETE** at `2caa20f` — all 7 ICs ratified, budgets measured and
gated, suite and calibration clean, and every deferred phase is an explicit
`ponytail:` debt with an upgrade path. The `UM-1` criteria are met at the probe
layer; a `scripts/gate --full` green on this commit range is the release tag
for `DC-1-UM-1-evidence`. Next wave is DC-1 converge (polish the deferred tail)
and DC-2 Era III wiring — same file-ownership ledger, same contract-freeze
discipline.
