# IC-6 — Milestone Gates (UM-1)

```yaml
provider: QA (gate definitions) + PLATFORM/CLIENT/DESIGN (gate co-sign)
consumer: PROD (UM-1 unify), all departments (must pass to ship)
frozen_at: commit 35dc9a8 (IC-2 ratified; FR-054, 04-cycle-plan-DC1.md §UM-1)
version: 1.0.0
status: RATIFIED-P0 (6 families mechanical, change mid-cycle is IC change-order)
change_orders: []
```

## Purpose

The checkable definition of done for UM-1 "the village develops" — six gate
families, each with a script, owner, and artifact. Nothing merges to UM-1
without all six green; QA holds rejection power without product code.

## Surface — 6 gate families (FROZEN v1.0.0)

| # | Family | Script / artifact | Owner | Pass rule |
|---|---|---|---|---|
| G1 | Suite green | `cargo test -p mindstrata-tests --lib --release` (release only) + `cargo insta` drift review + `scripts/gate --full` | QA | 307/0 (current `final_suite.log 198s`) + snapshots accepted with `CO-` evidence; no lucky-seed re-pin without sweep |
| G2 | Golden policy | `scripts/golden_replay.sh` + `runbooks/golden-replay-custody.md` hash registry | QA+PLATFORM | Structural changes byte-identical (`agent_hash` + file hashes); behavioral changes re-anchor only via QA custody with measured `old→new` + mechanism |
| G3 | Perf budget | `scripts/bench_index.py --strict` + `docs/balance/perf-budget.md` + `IC-8` frame budget | PLATFORM+TOOLS | `tick 19K+/s`, `100K ≤15s`, `render_trends ≤1 ms`, `golden 5 PASS`; violations fail gate even if suite green |
| G4 | Playability bar | `docs/balance/pacing-model.md` + playthrough smoke (`scripts/playthrough_smoke.sh` co-signed CLIENT) | DESIGN+CLIENT | 10K/50K/100K pacing probes show distinguishable player experience; `keybind_cheatsheet.md` + `dossier_flow.md` UX smoke pass |
| G5 | Content coherence | `docs/balance/needs-bands.md` + `docs/balance/pathology-curves.md` + `IC-5` CO- registry | DESIGN+STORY | No `CALIBRATION-PENDING` value consumed without `CO-` + probe + theory citation; pathology operator identity at neutral (zero-at-zero) |
| G6 | Calibration audit | `runbooks/calibration-audit-v2.md` + `i268_seed_family_sweep` + `suite-segmentation.md` | QA | CA-5 family `≥11/12` (`i268` current `12/12 PASS`), CA-2 saturation `fear_p90 <0.95`, no orphan tests, dead-producer probes accompany every liveness pin |

Gate scripts live in `scripts/` and are co-signed by TOOLS for custody
transfer; changing a threshold mid-cycle is a `CO-` against this IC.

## Guarantees

* Every gate is mechanically checkable by `scripts/gate` or a named runner —
  no human-judgment gate.
* Golden custody is append-only; `cargo insta` accept requires `CO-` + mechanism.

## Obligations

* Departments must produce beat artifacts (`metrics.jsonl` per IC-4 + branch note)
  for QA to evaluate G3/G5/G6 before converge windows.
* PLATFORM enforces G3 budget hook in gate; CLIENT enforces G4 bar.

## Tests guarding this contract

* The gates themselves — `scripts/gate --full` runs G1+G2+G3; G4–G6 are
  checklist-driven with artifact paths cited in the UM-1 evidence bundle
  (`docs/architecture/AP4-studio/templates/PHASE.md`).

## Changelog

| ver | change | approved |
|---|---|---|
| 1.0.0 | FROZEN — 6 families from FR-054/UM-1, wired to IC-3/IC-5/IC-8, custody, and runbooks | QA+PROD sign-off 2026-08-29 |
