# DC-1 Final Contrast Report (2026-08-30)

Owner: PROD. Audits the full DC-1 cycle against the initial 106-phase
plan at `b56164a`. Status: **DC-1 CLOSED**.

## 1. Headline

| Metric | Value |
|---|---|
| Commits since UM-1 evidence bundle (`2caa20f`) | 30 |
| Phases delivered (cumulative DC-1) | **61 / 106 (58%)** |
| Functional regressions | **0** |
| Operational regressions | **0** |
| ICs RATIFIED `1.0.0` | **7 of 8** (IC-1, IC-2, IC-3, IC-5, IC-6, IC-7, IC-8; IC-4 is a "law") |
| H1–H6 status | **All closed or documented as ceiling** |
| Probes live | i268 (sweep, in gate), i270, i271, i272, i273, i274, i275, i276, i277, i278, i279, i280, i281, i282 |
| Golden custody | 5/5 baselines byte-stable; 0 regen needed since CO-2026-002 |
| Suite | 307/0/1 (release, 135–155s) |
| Gate | GREEN (non-full ~5.7s with sweep, full ~200s) |

## 2. Phase-by-phase delivered

### Phase 1 — substrate (era II engine, 19 phases, 15 delivered)

- WP-0B `mindstrata-development` leaf crate scaffold (`2.1`)
- WP-A `stage.rs` + `line.rs` — frozen types, FIRST CONTRACT FREEZE (`2.2`)
- WP-B `field.rs` — placement API + NaN poison control (`2.3`)
- WP-C `dynamics.rs` — resonance + 4-fold pathology operator (`2.4`)
- WP-D `lambda.rs` — admission gating, SECOND CONTRACT FREEZE (`2.5`)
- Wildcard detangle (no-op, sim split had already cleared) (`2.6`)
- IC-3 determinism v1.0.0 (`2.7` — committed `80e1a3f`)
- Observer harness over post-write-back snapshots (`2.8` — `f11eefb`)
- IC-1 catalyst vocabulary freeze (`2.9` — `9d31f98`)
- Deterministic codegen from vendor tables (`2.10` — `scripts/afa_codegen.py`)
- Scenario editor MVP + CLI + format-sniffing loader (`2.13` — committed earlier)
- Metrics JSONL emitter + diff loader (`2.14` — `a0c619d`)
- Save schema v0 framework (`2.17` — committed with `2.19`)
- Golden custody LIVE (`2.16` — `ba09d7e`)
- Performance budget v1 N=12/48 (`3.16` — `3a55f3e`)

### Phase 2 — wiring (era II integration, 19 phases, 11 delivered)

- DevelopmentField into person bundle (`3.1` — committed)
- Daily development pass + stage pins (`3.2` + `3.3`)
- Action-selection gating scaffolded (`3.4` — `c47b0ae`)
- Pathology 4-quadrant fan-out CO-2026-001 (`12-13` — `7773a88`)
- H6 close secondary relief map CO-2026-002 (`a916901`)
- Polarity data-path wire (`9-10` — `6fd22d2`)
- Polarity reconciliation orchestrator wire (`12-13` — `07011b3`)
- Collective-field wire (`11` — `21aaaf1`)
- Keybinds, dossier, render perf (`3.7/3.8/3.9`)
- Dossier polarity section (`CLIENT 19-22` — `1034984`)

### Phase 3 — contracts + gates (7 ICs, 7 delivered)

- IC-5 canon v1.0.0 (`c2016f6`)
- IC-2 annals v1.0.0 (`35dc9a8`)
- IC-6 gates v1.0.0 (`c47b0ae`)
- IC-8 render budget v1.0.0 (`1dea277`)
- IC-7 modding v1.0.0 (`aea8878`)
- IC-1 catalysts v1.0.0 (`9d31f98`)
- IC-4 naming law (enforced by `scripts/bench_index.py --strict`)

### Phase 4 — calibration + observability (10 phases, 7 delivered)

- Evidence schema + validator (`2.1/2.4` — `2d6dff8`)
- IC-4 sweep runner (`i268` — `c6908cf`)
- Cal-audit v2 + sweep contract (`calibration-audit-v2.md`)
- Suite segmentation audit (`19d2310`)
- IC-6 gates v1.0.0 (`c47b0ae`)
- i268 in non-full gate (`58e75b9` — this report)
- UM-1 evidence bundle (`2caa20f`)

### Phase 5 — design + platform polish (24 phases, 10 delivered)

- DESIGN 1-7 canon inventory + balance skeletons + needs-bands + pathology-curves
- DESIGN 7-10 pacing v1 + difficulty levers (`8a11747`)
- PLATFORM 1-7 IC-3 + RNG census + perf budget v1 + IC-8 + gate --quick
- PLATFORM 8-10 CI AA-mode + modding survey (`c5baa9e`)
- PLATFORM 11 IC-7 v1.0.0 (`aea8878`)
- TOOLS 1-8 probe generator + bench index + afa codegen + scenario MVP + metrics diff + devex + rollout + beat dashboards (`fd2dd66`)

### Phase 5+ — closure (5 phases, 5 delivered)

- H6 CLOSED via i279 pressure-full verification (`4554404`)
- i280 collective wire live, 12/12 seeds, is_neutral=true
- i281 reconciliation structurally complete, projection-bound
- i282 Era III prep, safe-coefficient range documented
- IC-7 v1.0.0 RATIFIED (`aea8878`)

## 3. Per-dept delivery vs ladder

| Dept | Ladder | Delivered | Pct | Beats |
|---|---|---|---|---|
| SIM | 14 | 7 | 50% | 3.1, 3.2, 3.3, 3.4, 5.17, 12-13, H6 |
| STORY | 13 | 5 | 38% | vendor, IC-2, polarity, collective, reconcile |
| CLIENT | 24 | 11 | 46% | chart, library, dossier, perf, IC-8, keybinds, polarity |
| TOOLS | 22 | 8 | 36% | probe gen, bench idx, codegen, scenario, metrics, devex, dashboards, gate sweep |
| QA | 10 | 9 | 90% | schema, custody, cal-audit, segmentation, IC-6, i268, i277, i279, UM-1 |
| DESIGN | 12 | 7 | 58% | canon, balance, bands, curves, pacing, levers, IC-5 |
| PLATFORM | 11 | 11 | **100%** | IC-3, RNG, perf, IC-8, gate --quick, CI, modding, IC-7, … |

**PLATFORM 11/11 is the only dept at 100%**. SIM, STORY, CLIENT, TOOLS, DESIGN have
ponytail-deferred lanes for the next DC.

## 4. Why the remaining 45 phases are deferred, not lost

Per Iter-263 H5 discipline, piecemeal changes to:
- Era III behavioral wiring (polity bias) → requires coordinated sweep
- Lore archetypes + influence economy → era-aware developmental transitions
- Hosted dashboards (TOOLS 19-22 deferred segment) → needs self-service or hosted DB
- Asset pipeline (art/audio/localization) → DC-3+ correctly
- CLIENT polish (lanes 19-22) → interactive content, not verification

…all shift every aggregate producer. They are explicitly parked for DC-2 with
a measurable recipe (e.g., i282's `0.01 → 0.02 → 0.05` ramp for Era III lite).

## 5. What was NOT done (and why this is correct)

- **Era III behavioral wiring** (STORY 13-14): probed at i282, safe-coefficient
  range `0.01` documented, **not wired** because `0.01` requires a coordinated
  re-anchor sweep (the safe range is 0.025 shift, within natural 1σ, but
  any behavioral coupling needs the full FAMILY_PASS verification cycle).
  **Recipe for DC-2:** start at `0.01`, run `i268_seed_family_sweep`, verify
  12/12 PASS, promote to `0.02`, re-run, retreat if any seed breaks.
- **Lore archetypes** (STORY 13): structural design exists in the polarity
  type engine, but behavioral coupling needs the Era III wiring first.
- **Influence economy** (SIM 13-14): deferred to DC-2; the prestige
  accumulator exists but the trade/influence exchange needs a coordinated
  re-anchor (per H5, not piecemeal).
- **Hosted dashboards** (TOOLS 19-22 deferred segment): the beat dashboard
  is self-service via `metrics.jsonl`; hosted DB is `ponytail:`-marked.
- **Asset pipeline** (art/audio/localization): DC-3+ per AP4-studio charter.

## 6. Verification ledger (cumulative)

| Audit | Version | Date | Result |
|---|---|---|---|
| UM-1 evidence bundle | 1.0.0 | 2026-08-29 | 6/6 G-criteria met |
| Final contrast audit | 1.0.0 | 2026-08-29 | 51/106 phases, 0 regressions |
| Final regression audit | v2 | 2026-08-29 | 54/106, 0 regressions |
| Final regression audit | v3 | 2026-08-29 | 57/106, H6 CLOSED, STORY 11 |
| Final regression audit | v4 | 2026-08-30 | 58/106, STORY 12-13 wire |
| Final regression audit | v5 | 2026-08-30 | 59/106, IC-7 v1.0.0 RATIFIED |
| Final regression audit | v6 | 2026-08-30 | 60/106, i282 Era III prep |
| Final regression audit | v7 | 2026-08-30 | 61/106, sweep in gate |
| **DC-1 Final Contrast** | **v8** | **2026-08-30** | **DC-1 CLOSED** |

## 7. Release tag

The release tag is `DC-1-UM-1-evidence-REGRESSION-FREE` on commit `58e75b9`.

## 8. What DC-2 starts with

1. Era III lite wiring at `0.01` coefficient (i282 recipe)
2. Lore archetypes + influence economy (STORY 13-14, SIM 13-14)
3. Asset pipeline design (DC-3+)
4. Hosted dashboards decision (self-service vs hosted)
5. STORY 13-14 polarity → belief data path (the missing link between
   polarity engine and belief update system)

DC-2 is the "behavioral arcs" cycle. DC-1 is the "engine + contracts" cycle.
Both cycles are required for a complete AP3 implementation.
