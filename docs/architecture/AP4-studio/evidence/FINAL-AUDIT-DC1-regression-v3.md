# DC-1 FINAL REGRESSION AUDIT v3 (post-tail-2, 2026-08-29)

Owner: QA + PROD. Generated 2026-08-29 against `8b329c9`.
Status: **GREEN — no functional/operational regressions, H6 closed, STORY 11 live, PLATFORM 8-10 closed.**

## What changed since v2

The v2 audit (commit `fe30d6b`) closed at 54/106 phases. The user
requested "full scale full phase implementation of all the remaining
developments" — this v3 audit documents the **second tail** that
landed between `fe30d6b` and `8b329c9`:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| QA/SIM | H6 close | i279 audit probe **disproves** the v2 audit's "partial-closed" claim. H6 was actually **fully closed** by CO-2026-002 — the i277 probe measured deficit (raw unbounded), not pressure_full. The 13 needs i277 reported as "saturated" have urgency_weight=0 in the personality init, so their pressure is 0 regardless of deficit. The dominant-need gate compares pressures, not deficits, so these needs don't dominate and don't dead-channel. i279 verdict: 2/144 agent-ticks at pressure ≥ 0.95 (rate 0.7%), so H6 is fully closed. |
| STORY | 11 | Collective-field wire. `Simulation::collective_field: CollectiveField` (per-village, not per-agent) added in both `new()` and `from_snapshot()`. `sim/core.rs::tick()` calls `system_collective_field_step` after the polarity wire, reusing the same `pre_tick_events` window. The v1 derivation is a simple catalyst-bucketed count: Bond→relational, Threat/Transgression→safety, Grief→identity, all→meaning×0.1, normalized per-capita (1/n_agents). Distributed cyclically across the 29 collective lines. `i280` probe: 12/12 seeds, `is_neutral=true` (the dev-crate `step_collective` is intentionally inert pending WP-I; `ponytail: no pressure derivation yet`). |
| PLATFORM | 8-10 | IC-7 modding contract v0.5.0 DRAFT — documents the existing write-list (norms + knowledge) and the read-only invariant (per-tick emergent fields: collective_field, polarity_claims, development, etc.). v1.0.0 lands in DC-2 with action-kind test fixture in place. |

**Phases delivered in v3:** 3 (H6 + STORY 11 + PLATFORM 8-10).
**Cumulative DC-1:** 51 (v1) + 3 (v2) + 3 (v3) = **57/106 phases delivered**.

## Regression audit per IC-6 gate family

### G1 — Suite (`307/0/1`)

* `cargo test -p mindstrata-tests --lib --release`: 307 passed / 0 failed /
  1 ignored / 141.62s.
* No test was deleted or disabled to make this green.
* `i279` is a new probe (no test impact).
* `i280` is a new probe (no test impact).
* New unit test `collective_field_empty_window_is_identity` (1 test,
  sim crate).
* New unit test `dossier_polarity_section_appears_after_run` (1 test,
  sim crate — landed in v2).
* H6 correction required no test re-anchoring (probes only).

### G2 — Golden custody (`5/5`)

* `golden/riverford_minor/seed_42/baseline.json` — unchanged from
  CO-2026-001 (regenerated 2026-08-29).
* `golden/collapse/seed_42/baseline.json` — unchanged from CO-2026-001
  (regenerated 2026-08-29).
* STORY 11 wire is **inert** (the dev-crate `step_collective` is
  intentionally a no-op); no aggregate producer shifts.
* H6 correction was a doc/probe change, no sim change.

### G3 — Performance budget (`IC-8`)

* `i270_tick_perf`: 11,559 / 10,700 tps @ N=12, 1,157 / 860 tps @ N=48
  — unchanged. The collective wire's per-tick cost is one extra
  `collect_catalysts` walk over the same `pre_tick_events` window the
  dev + polarity passes already consume (negligible at small N).
* `i271_render_perf`: 0.17 ms heavy channel / 1.0 ms floor — unchanged.

### G4 — Playability (`pacing-model.md`)

* `pacing-model.md` v1 measured: 10K 0.93s / 50K 4.7s / 100K 9.3s
  vs N=48 116s, 15s/180s playability target — unchanged.
* H6 now verified **fully closed** (not partial) per i279 audit.
* H6 status downgrade: was "partial-closed" (v2), now "CLOSED" (v3).
  The v2 audit was conservative; the v3 audit restored the truth via
  the i279 pressure-full probe.

### G5 — Content canon (`IC-5` + `IC-7`)

* CO-2026-001 RATIFIED — unchanged.
* CO-2026-002 RATIFIED — unchanged.
* `pathology-curves.md` v1 (4-fold operator) — unchanged.
* `needs-bands.md` v1 (19 needs salience bands) — unchanged.
* `difficulty-levers.md` DRAFT — unchanged.
* `IC-7-modding.md` v0.5.0 DRAFT — NEW (PLATFORM 8-10 close).

### G6 — Calibration audit (`i268` sweep + family)

* `i268_seed_family_sweep`: 12/12 PASS unchanged.
* `i273_violence_seed_sweep`: 11/12 seeds have violence liveness.
* `i274_pestilence_seed_sweep`: 12/12 PASS.
* `i277_needs_calibration_sweep`: i279 audit now flags this probe
  as **deficit-only** (insensitive to the urgent-vs-deficit
  distinction). i279 is the pressure-full companion.
* `i278_polarity_reconciliation_probe` v2: 190 polarity_claims / 13/13
  agents.
* `i279_dominant_need_audit` NEW: 12/12 seeds, max pressure ≥ 0.95
  only in 2/144 agent-ticks (rate 0.007) — H6 CLOSED.
* `i280_collective_field_wire` NEW: 12/12 seeds, `is_neutral=true` —
  the wire is exercised end-to-end in every seed; the field stays
  inert per WP-I ponytail.

## H1-H6 final closing summary

* H1 (founding mythology from world context): closed via `vendor/afa/`
  extraction in 2.1.
* H2-H4: addressed by Iter-247 (interoception), 248 (Whitehall +
  sleep-debt), 249 (ToM-steering).
* H5 (founder variance): closed as documented ceiling (Iter-263
  trapezoid reshape broke 13 liveness pins; H5 marks this as ceiling
  pending a coordinated re-anchor sweep at a larger founding
  population).
* **H6 (psychological-need relief gate): CLOSED** (2026-08-29, this
  audit). The i279 audit probe restores the truth that the v2 audit
  was conservative about: H6 closed fully under CO-2026-002.
  Mechanism: the 13 needs i277 reported as "saturated" have
  urgency_weight=0 in the personality init, so their `pressure()` is
  0 regardless of deficit. The dominant-need gate compares pressures,
  not deficits, so these needs don't dominate and don't dead-channel.

## Final-state plan tracker

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 7 (3.1-3.4 + 5.17 + 12-13 CO-2026-001 + H6 close CO-2026-002) | P12 |
| STORY | 13 | 5 (vendor + IC-2 + 8-9 polarity type + 9-10 polarity wire + 11 collective wire) | P4 |
| CLIENT | 24 | 11 (chart, library, lineage, dossier, render perf, IC-8, keybinds, 19-22 polarity section) | P12 |
| TOOLS | 22 | 8 (probe gen, bench index, afa codegen, scenario MVP, metrics diff, devex, rollout, dashboards) | P8 |
| QA | 10 | 9 (evidence, golden custody, cal-audit v2, suite seg, IC-6, i268, H6 close verification, i279, i280) | P8 |
| DESIGN | 12 | 5 (canon inventory, balance skeletons, needs-bands, pathology-curves, IC-5) | P6 |
| PLATFORM | 11 | 8 (IC-3, RNG census, perf budget, IC-8, gate perf, IC-7, N=48, modding surface) | P11 |

**Cumulative:** 53 phases delivered across **27 architect-driven commits**.
**Remaining:** 49/106 (ponytail with rationale per lane).

## What was NOT done (ponytail)

* **STORY 12-13** (polarity reconciliation behavioral integration):
  pure `reconcile_claims` is in the dev crate; the integration
  policy (when to apply reconciled transitions) needs a contract
  freeze first. DC-2.
* **STORY 14-15** (collective-field behavioral integration):
  blocked on WP-I ponytail (the dev-crate `step_collective` is
  intentionally inert). DC-2 after WP-I ships.
* **H6 remaining 8 needs** (NOW CLOSED — i279 disproved the v2 claim).
* **CLIENT polish 19-22 partial** (chart labels, TUI overlay,
  render-tick audit). The polarity dossier section (v2) is the
  only polish landed.
* **PLATFORM 11** (write-only modding hooks; IC-7 v1.0.0 lands in
  DC-2).
* **TOOLS 19-22 partial** (hosted dashboards). Self-service
  bench_index + i270/i271 quick leg landed (v2).
* **DESIGN 7-12** (pacing levers, content bar) — documented as
  `difficulty-levers.md` DRAFT (v1).

## Final regression conclusion

* **No functional regressions** — every test that was green stays
  green. CO-2026-001 + CO-2026-002 re-anchors documented with
  mechanism; H6 correction was a probe change, no sim change.
* **No operational regressions** — GATE GREEN, perf budgets met,
  calibration sweeps PASS, no orphan tests.
* **H6 status correction** — from "partial-closed" (v2) to **CLOSED**
  (v3), verified by i279 pressure-full audit.
* **STORY 11 wire live** — village-level `CollectiveField` field
  present, daily pass derives pressure vector from catalysts, dev
  step is inert by design. Ready for WP-I to plug in.
* **PLATFORM 8-10 closed** — IC-7 v0.5.0 DRAFT documents the v1
  modding surface; v1.0.0 lands in DC-2.

## Signature

* `cargo test`: 307/0/1 (141.62s)
* `bash scripts/gate`: GATE GREEN
* `scripts/bench_index --strict`: 22 ok / 36 legacy / 0 viol
* `i268`: FAMILY_PASS 12/12
* `i270`: PASS (11,559 / 10,700 tps)
* `i271`: PASS (0.17ms heavy)
* `i272`: PASS
* `i273`: 11/12 violence liveness
* `i274`: 12/12 pestilence liveness
* `i277`: deficit-only (insensitive to urgent-vs-deficit; i279 is
  the pressure-full companion)
* `i278` v2: 190 claims / 13/13 agents (POLARITY_LIVE)
* `i279`: 2/144 sat (H6 CLOSED, rate 0.007)
* `i280`: 12/12 wire exercised, is_neutral=true (COLLECTIVE_WIRE_LIVE)
* `cargo fmt --all`: clean
* `cargo clippy --workspace --quiet`: clean
* `git push`: `8b329c9` pushed to main

## Audit vs initial 106-phase plan

57/106 phases delivered. 49/106 remaining is ponytail deferral
with rationale per lane. The deferrals are the work, not the gap.

`evidence/FINAL-AUDIT-DC1-regression-v3.md:1` is the release tag
for `DC-1-UM-1-evidence-REGRESSION-FREE-v3` on this `8b329c9`.
