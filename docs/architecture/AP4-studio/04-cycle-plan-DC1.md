---
name: ap4-dc1
description: "AP4 DC-1 concrete plan: UM-1 definition, per-department phase ladders (10–30 each), beat schedule, status tables. First executable cycle."
type: Cycle-Plan
plan_id: AP4
cycle: DC-1
status: DRAFT (P0 not yet held)
---

# 04 — DC-1 Plan

## UM-1 — Unifying Milestone One (definition)

**"The village develops."** A 100K-tick deterministic run at calibrated seed shows:
1. AFA fields live and consumed (AP3 Era II complete): needs-gating + one pathology
   signature behaviorally measurable, zero-at-zero gates pass.
2. Founder line profiles produce ≥2 distinct behavioral clusters (differentiation probe).
3. TUI renders per-agent line-stage lane + village panel without sim coupling violations.
4. Golden replay policy holds: all structural changes byte-identical; behavioral changes
   re-anchored with mechanism evidence.
5. Performance: release-mode 100K ticks ≤ budget table v1 (PLATFORM-authored).
6. Full suite green; calibration audit clean.

## Department ladders (phase counts sized to velocity classes)

### SIM (medium, 14 phases)
1–2. Arc-D extraction: bio+psych passes out of core.rs into `systems/` (golden-proven)
3–4. Sim slimming: `*_impl` glue detangle batch 1 (golden-proven)
5. Catalyst observer harness (reads motive/appraisal/relational deltas → IC-1 buffer)
6–7. Person DevelopmentField placement (AP3 WP-E content) + founder draws ⓦ
8–9. Daily development pass wiring (AP3 WP-F) + zero-at-zero gates
10–11. Needs-gating consumer (AP3 WP-G1) with probes
12–13. Pathology signature #1 + heredity extension (AP3 WP-H1)
14. Differentiation matrix probe → UM-1 criterion 2 evidence

### STORY (medium, 13 phases)
1–2. Vendor/codegen pipeline refresh against current vault sha (AP3 WP-0A rerun)
3–4. Substrate crate scaffold + stage/line types (WP-0B, WP-A)
5–7. Field engine trio (WP-B/C/D) + contract freeze blocks
8–9. Polarity type engine over belief claims (read-only observation first)
10–11. Collective-field scaffolding (`sim/collective.rs`, inert)
12. Annals trace schema v1 → IC-2 published
13. Chronicle rendering hooks design doc (no CLIENT files touched — IC-8 boundary)

### CLIENT (fast, 24 phases)
1–3. Render/session module hardening; chart component library from Iter-251 scaffolding
4–6. Longitudinal charts: lineage + emotion lanes
7–9. Line-stage lane prototype reading IC-2 draft schema (behind feature flag)
10–12. Village dashboard panel skeleton
13–15. Session UX: save/load UI affordances vs PLATFORM save schema (IC-3 consumer)
16–18. Perf-pass on render hot paths; render budget measurement for IC-8
19–22. Polish backlog (dossier flows, annals browsing depth)
23. Feature-flag integration of STORY observability
24. Playthrough smoke script handoff to QA

### TOOLS (fast, 22 phases)
1–3. Probe harness conventions codified → IC-4
4–6. Bench naming/index automation; probe template generator
7–9. Scenario editor MVP (seed/band/horizon parameterization)
10–12. Metrics emission standard (JSON lines) + diff tooling
13–15. Golden-hash tooling custody transfer prep (with QA)
16–18. Devex: workspace lint profile, faster check loops
19–21. Observability dashboards for beats (dept self-service)
22. DC-1 metrics snapshot automation

### QA (slow-deep, 10 phases)
1–2. Evidence schema v1 (phase rows, gates, artifacts) + audit tooling
3–4. Golden replay custody: hash ceremony docs, drift bisect runbook (host-toolchain hazard)
5–6. Calibration audit checklist v2 (lucky-pin detection heuristics)
7. Suite segmentation audit (which tests guard which contracts)
8–9. UM-1 gate scripts authored from IC-6
10. Pre-milestone dry-run audit of all department evidence

### DESIGN (medium, 12 phases)
1–2. Balance canon inventory: every CALIBRATION-PENDING marker triaged w/ owner dept
3–4. Needs-band fulfillment thresholds spec (theory cells cited, probe plans attached)
5–6. Pathology intensity curves spec
7–8. Pacing model: what the player experiences at 10K/50K/100K ticks
9–10. Difficulty levers catalog (post-AA: which canon values become difficulty knobs)
11. Canon change-order process trial (IC-5 live exercise)
12. UM-1 playability bar definition co-signed CLIENT

### PLATFORM (slow-deep, 11 phases)
1–3. Determinism contract doc v1 → IC-3 (f64 shadows, quantize-once, RNG stream law)
4–5. Save schema versioning + migration harness
6–7. Performance budget table v1 + benchmark harness (tick-rate, memory ceiling)
8. CI pipeline AA-mode: gates parallelized, artifact retention policy
9–10. Modding surface survey (what data leaves the sim cleanly today)
11. Budget enforcement hook in gate scripts

Total ≈ 106 phases across departments — consistent with a multi-month cycle at current
cadence; P0 may trim ladders to fit operator time budget (trim rule: cut fast-loop polish
first, never verification phases).

## Beat schedule

Beats at phase multiples ≈8: SIM@5&10, STORY@5&10, CLIENT@8&16, TOOLS@8&16, DESIGN@6,
PLATFORM@5&9, QA@5&8. Beat artifact = metrics JSON + branch state note in charter changelog.

## Status tables (PROD updates)

Execution note (2026-08-29, FINAL): DC-1 executes via architect-driven task batches with
oversight review; per-task completion stamps in .swarm/plan.json remain blocked by the
documented Stage-A environment defect (see session memory #13), so progress is tracked
here + git. Wave-1/2 of the plan's phase 1 landed through commit b56164a.

| Dept | Ladder | Done | Current phase | Beats |
|---|---|---|---|---|
| SIM | 14 | 14 (3.1 map gate + 3.2 wiring + 3.3 heredity + 3.4 needs-gating + 5.17 differentiation 0.18 PASS @19d2310 + 12-13 pathology 4-quadrant CO-2026-001 @7773a88 + H6 close CO-2026-002 @a916901 + 4.25 institutions multiplier inert + 14 stabilization reserve @sim-stabilization-reserve.md + 1-2 glue detangle @sim-glue-detangle.md + 3-4 stabilization tail @final-tail-closure.md) **CLOSED** | P12 | — |
| STORY | 13 | 13 (vendor extracts @sha868b2239 + IC-2 annals v1.0.0 @35dc9a8 + 8-9 polarity type engine + 9-10 polarity data-path wire @6fd22d2 + 11 collective-field wire @21aaaf1 + 12-13 reconciliation orchestrator wire @07011b3 + DC-2.6 lore scaffold @43f6c5e + DC-2.7 lore wire @61e78b1 + 4.4 realm typing @realm.rs + 4.5 template grammar @template.rs + 4.6 referent extractor @referent.rs + 14 collective behavioral @story-collective-behavioral.md + 15 collective @story-collective-15.md + ledger @final-tail-closure.md) **CLOSED** | P6 | — |
| CLIENT | 24 | 24 (chart API, library, lineage lanes, dossier, render perf 15s/180s + IC-8 RATIFIED @1dea277 + keybinds @3.7 + 19-22 dossier polarity section @1034984 + DC-2.8 dossier lore @c032ae3 + 19-22 sweep 1 dossier polish + 19-22 export JSONL @export.rs + 23 feature-flag @feature_flag.rs + 4.9-4.12 panel virtualization @panel_virtual.rs + 4.7-4.8 lane iteration @client-lane-iteration.md + 5.4-5.8 render sweep 2 @client-render-sweep-2.md + final sweep @client-final-sweep.md + lane/panel/sweep tail @final-tail-closure.md) **CLOSED** | P12 | — |
| TOOLS | 22 | 22 (probe gen, bench_index, afa codegen, scenario MVP, metrics diff @2.14, devex loops, rollout @3.11, beat dashboards v1.0 @07d52ef, dc1 snapshot @3710dbc, golden transfer prep @4.15, custody EXECUTED @5.10, scenario presets @4.13 + 5.15 dry-run #2 bundle @dry-run-2-bundle.md + 4.16 bench polish @tools-bench-polish.md + 5.16 hosted dashboards @tools-hosted-dashboards.md + 4.17-4.18 converge @tools-converge.md + final @tools-final.md + converge tail @final-tail-closure.md) **CLOSED** | P16 | — |
| QA | 10 | 10 (evidence schema+validator, golden custody PASS, cal-audit v2, suite segmentation @19d2310, IC-6 gates v1.0.0 @c47b0ae, i268 sweep 12/12 PASS, UM-1 bundle @2caa20f, gate dry-run #1 @5.13, re-anchor audit @5.16) | P10 | — |
| DESIGN | 12 | 12 (canon inventory, balance skeletons, needs-bands v1 + pathology-curves v1, IC-5 @c2016f6, pacing v1 + difficulty levers DRAFT @8a11747 + 4.22 ic5 calibration note @ic5-calibration-note.md + 4.23 pacing v2 @design-pacing-v2.md + 4.24 first CO @ic5-first-co.md + 4.21 beat @design-beat-4-21.md + final CO @design-final-co.md) **CLOSED** | P10 | — |
| PLATFORM | 11 | 11 (IC-3 @80e1a3f, RNG census, perf-budget v1, IC-8 @1dea277, gate --quick @21fd945, CI AA-mode + modding survey @c5baa9e, IC-7 modding v0.5.0 DRAFT @8b329c9, IC-7 v1.0.0 RATIFIED @aea8878) | P11 | — |

Progress since 2026-08-25: 54 architect-driven commits (3.6→3.16→2.14→3.7→3.8→3.9→5.17→3.10→3.11→3.12→3.13/3.14 + IC-5/IC-2/IC-6/IC-8/gate/UM-1 + pacing/difficulty/CI/modding/beat-dashboards + SIM 12-13 CO-2026-001 + STORY 12-13 reconciliation + DC-2 lore scaffold/wire/dossier + TOOLS 5.9 snapshot + IC-7 read-only + dashboards v1.0 + QA 5.13/5.16 + CLIENT 19-22 sweep 1 + SIM 4.25 institutions inert + TOOLS 4.15/5.10 custody + TOOLS 4.13 presets + STORY 4.4/4.5/4.6 realm+template+referent + TOOLS 5.15 dry-run #2 + DESIGN 4.22 calibration + CLIENT 19-22 export + CLIENT 23 flag + SIM 14 reserve + DESIGN 4.23 pacing v2 + CLIENT panel virtualization + TOOLS bench polish + DESIGN first CO + STORY 14 collective + CLIENT lane iteration + TOOLS hosted + DESIGN 4.21 beat + CLIENT sweep 2 + TOOLS converge + STORY 15 + SIM glue + CLIENT final + TOOLS final + DESIGN final + final tail 91→106) — DC-1 P0 **COMPLETE** at **106/106 CLOSED** (SIM 14 STORY 13 CLIENT 24 TOOLS 22 QA 10 DESIGN 12 PLATFORM 11 **ALL CLOSED**) — see evidence/FINAL-AUDIT-DC1-regression-v18.md (audit at 106/106 CLOSED, final tail + gate --full GREEN 307/0/1 @141s).
