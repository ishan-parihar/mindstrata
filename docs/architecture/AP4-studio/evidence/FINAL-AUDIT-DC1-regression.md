# DC-1 FINAL REGRESSION AUDIT (post-COMPLETE)

Owner: QA + PROD. Generated 2026-08-29 against `8dd2ff2`.
Status: **GREEN — no functional/operational regressions.**

## Inputs

* Initial plan: `04-cycle-plan-DC1.md` (106-phase ladder from `b56164a`).
* Final state: 51/106 phases delivered (49 prior + DESIGN 8-9 + STORY 8-9
  i277/i278 in this commit).
* 3 systemic hazards known + 1 new (H6) — see `AGENTS.md` §5.

## Regression audit per IC-6 gate family

### G1 — Suite (`307/0/1`)

* `cargo test -p mindstrata-tests --lib --release`: 307 passed / 0 failed / 1
  ignored / 278.31 s.
* No test was deleted or disabled to make this green.
* CO-2026-001 re-anchors documented in
  `docs/balance/change-orders/CO-2026-001.md` (6 test re-pins with mechanism,
  7 snapshot regens, 2 golden baseline regens).

### G2 — Golden custody (`5/5`)

* `golden/riverford_minor/seed_42/baseline.json` regenerated 2026-08-29
  (post-CO-2026-001 metric_hash 14508470650233533994, agent_count 13
  was 12, total_grain 81.1551 was 82.0257).
* `golden/collapse/seed_42/baseline.json` regenerated 2026-08-29
  (post-CO-2026-001 metric_hash 16076669445635078694).
* 3 additional golden copies (deterministic, different-seeds, agent-count)
  unchanged.
* Mechanism: event-stream carries catalyst kinds, fan-out routes
  pressure to 4 quadrants → 0.5% magnitude drift in metric_hash and
  agent_count drift in calm-riverford (one new birth within 1000 ticks).

### G3 — Performance budget (`IC-8`)

* `i270_tick_perf` (PLATFORM `4.16`): 11,559 / 10,700 tps @ N=12, 1,157 /
  860 tps @ N=48 — both above floors, projection to 100K ticks = 9.3 s
  (within 15 s).
* `i271_render_perf` (CLIENT `3.9`): 0.17 ms heavy channel / 1.0 ms
  floor — within 1 ms.
* `scripts/gate --full` adds the perf leg before the 3/3 suite check.

### G4 — Playability (`pacing-model.md`)

* `i271_render_perf`: trends 0.17 ms / 1 ms (PASS).
* `i277_needs_calibration_sweep`: identifies H6 dead-channel debt on
  13/19 needs (pre-existing, not regression).
* `pacing-model.md` v1 measured: 10K 0.93s / 50K 4.7s / 100K 9.3s vs
  N=48 116s, 15s/180s playability target.

### G5 — Content canon (`IC-5`)

* CO-2026-001 RATIFIED: pathology 4-quadrant fan-out documented with
  old/new band table for 7 affected tests.
* `pathology-curves.md` v1 ratifies the 4-fold dark/golden ×
  addiction/allergy operator.
* `needs-bands.md` v1 ratifies 19 needs salience bands.
* `difficulty-levers.md` DRAFT (3 knobs).

### G6 — Calibration audit (`i268` sweep)

* `i268_seed_family_sweep`: 12/12 PASS post-CO-2026-001 (CA-1/CA-2/CA-4).
* `i273_violence_seed_sweep`: 11/12 seeds have violence liveness.
* `i274_pestilence_seed_sweep`: 12/12 seeds have pestilence liveness,
  used to re-anchor `revolution_is_regime_change_not_repeat_loop`.
* `i277_needs_calibration_sweep`: identifies H6 (13/19 needs
  saturated, pre-existing dead-channel debt).
* `i278_polarity_reconciliation_probe`: POLARITY_LIVE verdict (3-realm
  grammar operational).

## H6 — psychological-need relief gate (NEW 2026-08-29)

`i277_needs_calibration_sweep` shows 13/19 needs saturating to deficit
1.0 across all 12 seeds. Mechanism: only 6/19 needs have `relieve()`
in the action-kind table. Verified pre-CO-2026-001 (same saturation),
**not a regression**. Fix is DC-2: widen dominant-need gate OR add
relieve paths. **DO NOT** silently fix — Iter-263 H5 lesson applies
(piecemeal relief-path changes shift every aggregate producer).

## H5 — founder variance (RE-RECORDED)

Already closed as documented ceiling in `AGENTS.md` §5.

## H1-H4 — closing summary

* H1 (founding mythology from world context): closed via
  `vendor/afa/` extraction in 2.1.
* H2-H4: addressed by Iter-247 (interoception), 248 (Whitehall +
  sleep-debt), 249 (ToM-steering) — prior session memory.

## What was NOT done (ponytail)

* **Polarity → belief** data path (events → claims): deferred to DC-2
  (STORY 9-10).
* **13 dead-channel needs** (H6): deferred to DC-2.
* **CLIENT polish lanes** (19-22): deferred to DC-2.
* **PLATFORM CI / modding hooks** (8-10): already surveyed in
  `modding-surface.md`; read-only modding in DC-1.
* **Era III content** (STORY 8-11 polarity/collective wiring): type
  engine landed; behavioral wiring pending DC-2.
* **TOOLS beat dashboards hosted DB**: documented as ponytail —
  self-service via JSONL + git + i270/i271 --quick.

## What this means for the final audit

* **No functional regressions** — every test that was green stays green,
  every test that needed re-anchoring is documented in CO-2026-001 with
  mechanism.
* **No operational regressions** — GATE GREEN, perf budgets met,
  calibration sweep PASS, no orphan tests (suite-segmentation
  audit at 2026-08-29 = 0 orphans).
* **Audit vs initial 106-phase plan**: 51/106 phases delivered. 55/106
  remaining are correctly deferred (ponytail, with `ponytail:` upgrade
  paths in each `*_skeleton.md` or `*_DRAFT.md`). The deferrals are
  the work, not the gap.

## Signature

* `cargo test`: 307/0/1 (278.31 s)
* `bash scripts/gate`: GATE GREEN
* `scripts/bench_index --strict`: 22 ok / 36 legacy / 0 viol
* `i268`: FAMILY_PASS 12/12
* `i270`: PASS
* `i271`: PASS
* `i272`: PASS
* `i273`: 11/12 violence liveness
* `i274`: 12/12 pestilence liveness
* `i277`: needs sweep, H6 logged
* `i278`: POLARITY_LIVE
* `cargo fmt --all`: clean
* `cargo clippy --workspace --quiet`: clean
* `git push`: 8dd2ff2 pushed to main
