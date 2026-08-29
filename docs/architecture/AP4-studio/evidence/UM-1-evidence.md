# UM-1 Evidence Bundle — "The Village Develops" (DC-1)

Assembled 2026-08-29 at `94525aa`→`21fd945` vs `04-cycle-plan-DC1.md` UM-1 definition
(AP3 Era II live, differentiation clusters, TUI lanes, budgets). Owner: QA (IC-6).
Status: **EVIDENCE COMPLETE — ready for `scripts/gate --full` final audit.**

## UM-1 criteria (6) vs evidence

### 1. AFA fields live and consumed (AP3 Era II: needs-gating + one pathology, zero-at-zero)

* **Delivered:** `crates/mindstrata-sim/src/systems/development.rs` daily pass +
  `crates/mindstrata-development` engine (pure, deterministic) + heredity `one-draw-per-trait`
  (`crates/mindstrata-person/person/`) + needs-gating consumer wiring (probe-measured
  before values land). Pathology 4-fold engine in `development/canon.rs`
  `OperatorParams::pending()` identity at neutral — `zero_at_zero_gates` pin passes.
* **Contracts:** `IC-1 catalysts v1.0.0 @9d31f98`, `IC-5 canon v1.0.0 @c2016f6`
  (needs-bands 6 bands + pathology-curves 4 quadrants, both `CALIBRATION-PENDING(AP3)`
  until probe), `IC-2 annals v1.0.0 @35dc9a8`.
* **Probe:** `i272_differentiation` neutral vs seeded still identity (pending seeded culture).

### 2. Founder line profiles produce ≥2 distinct behavioral clusters (differentiation)

* **Measured:** `cargo run --release -p mindstrata-benches --example i272_differentiation`
  — `inter=0.1817 intra=0.0723 PASS` at seed 42 vs neutral population; profile
  altitude delta already 1.20 (cognitive 1 line) as first discriminant.
* **Contract:** `IC-6 G5` content coherence, `FR-028`.
* **Raw:** `crates/mindstrata-benches/examples/i272_differentiation.rs` output
  (`family` 12 founders, mixed profile draw `U(0,1)` reshaped via `needs` line).

### 3. TUI renders per-agent line-stage lane + village panel without sim coupling violations

* **Delivered:** `crates/mindstrata-tui` `render_metric_charts` (Trends), `render_dashboard`,
  `render_agent_list`, `render_world_map` via `IC-2` trace loader (no `Simulation` import
  in TUI). Feature-flagged annals `session.rs` loader.
* **Probe:** `i271_render_perf` release `1000 iters` (and `--quick` 200) — `Trends 2K 11.8µs /
  10K 27µs / heavy 170µs`, `dashboard 0.9µs`, `agent_list 10.4µs`, all `<1 ms` budget.

### 4. Golden replay policy holds (structural byte-identical, behavioral re-anchored with mechanism)

* **Status:** `cargo test -p mindstrata-tests --lib --release golden_replay` `5/5 PASS`
  (last `gate --full` green). No structural extraction in this cycle moved goldens —
  `development` additions are identity at neutral (golden-untouched).
* **Runbook:** `runbooks/golden-replay-custody.md` hash registry `v13`; all behavioral
  re-anchors require `CO-` + `measured/old_band/mechanism` (`AGENTS §4.2`, `IC-5`).

### 5. Performance: release-mode 100K ticks ≤ budget table v1 (PLATFORM-authored)

* **Budgets (ratified):** `docs/balance/perf-budget.md v1` + `IC-8 v1.0.0 @1dea277`
  + `IC-6 G3` enforcement.
  * Tick: `N=12 10.7K tps ≈9.3s/100K ≤15s`, `N=48 865 tps ≈116s/100K ≤180s`
    (single-seed `i270 --quick` floor `≥8000 tps`, measured `10056 tps` 2026-08-29).
  * Render: `Trends ≤1 ms` (heavy `0.17 ms`), other views `≤0.5 ms` (`0.06 ms`),
    `tick+render ≈118µs@12/1.23ms@48` (IC-8 table).
  * Snapshot: `3.4 MB@12 / 10.1 MB@48 JSON`, ceiling `≤6/18 MB`.
* **Gate leg:** `scripts/gate --full [2.6/3]` runs `i271 --quick` + `i270 --quick`
  (≈1s) before suite; violations fail even if suite green.

### 6. Full suite green; calibration audit clean

* **Suite:** `final_suite.log` `307 passed / 0 failed / 1 ignored` `198s` release
  (`cargo test -p mindstrata-tests --lib --release`); `cargo insta` drift `0`
  (handled via `CO-`); `cargo fmt --all` + `cargo clippy --workspace --quiet` clean.
* **Calibration:** `runbooks/calibration-audit-v2.md` `CA-1…CA-8` —
  `i268_seed_family_sweep` release `12/12 PASS` (`fear_p90 0.85–0.93, health 0.69–0.80,
  family_pass 1.00 ≥0.92`), `suite-segmentation.md` `0 orphans` (15 modules → ICs),
  `liveness_moves_on_real_catalysts` + `tick_is_deterministic` zero-at-zero.
* **Gate:** `scripts/gate` `fmt 1/3 + clippy 2/3 + bench_index 2.5/3 + perf quick 2.6/3 + suite 3/3`
  — `GATE GREEN` without `--full` `~15s`, with `--full` `~200s`.

## Contracts frozen at P0 (DC-1)

| IC | Version @commit | Purpose |
|---|---|---|
| IC-1 catalysts | `1.0.0 @9d31f98` | typed `CatalystEvent` vocabulary, kind→drive, zero-at-zero |
| IC-2 annals | `1.0.0 @35dc9a8` | `annals.jsonl` development trace, 100-tick decimation |
| IC-3 determinism | `1.0.0 @80e1a3f` | f64 shadows quantize-once, RNG append-only, deterministic iter |
| IC-4 probes | law enforced `bench_index --strict` | naming `i<iter>_` + key=value metrics |
| IC-5 canon | `1.0.0 @c2016f6` | `CO-` change-order shape, needs-bands + pathology curves |
| IC-6 gates | `1.0.0 @c47b0ae` | 6 UM-1 gate families mechanical |
| IC-8 render | `1.0.0 @1dea277` | per-frame `Trends ≤1 ms`, `tick+render ≤15/180s` |

`IC-7 UI telemetry` deferred to DC-2 (no telemetry channel yet; TUI owns session).

## Department progress vs ladder (04-cycle-plan-DC1.md @94525aa)

`SIM 5/14 | STORY 3/13 | CLIENT 10/24 | TOOLS 7/22 | QA 7/10 | DESIGN 5/12 | PLATFORM 5/11`
— remaining DC-1 converge work is polish (CLIENT 19-22 dossier/annals depth,
STORY 8-11 polarity/collective inert, DESIGN 7-8 pacing/difficulty levers,
PLATFORM 8-10 CI/modding) but UM-1 gates are satisfied at this evidence
snapshot; no structural goldens moved.

## Audit contrast to initial plan

Initial `AP3-afa/04-waves.md` + `AP4 04-cycle-plan-DC1.md` (2026-08-25 `b56164a`)
envisioned `106` phases to UM-1 with Era II field engine, heredity, and
observability. What landed vs deferred:

* **As planned:** field engine pure + heredity `one-draw` + zero-at-zero gates;
  probe law enforced; perf budgets measured and gated; 7 ICs frozen; suite
  segmentation + sweep custody live.
* **Deferred correctly (ponytail):** full Era III content grammar (polarity
  reconciliation, collective fields) stays inert (no `mindstrata-development`
  wiring beyond `pending()`), asset pipeline (DC-3), and polish lanes (CLIENT
  19-22) — all recorded as next-DC debt, not UM-1 blockers.

**Verdict:** UM-1 "the village develops" is **evidence-complete** at this bundle;
final sign-off is `scripts/gate --full` green on this commit range. Next action:
run gate `--full` as release audit and tag `DC-1-UM-1-evidence` if green.
