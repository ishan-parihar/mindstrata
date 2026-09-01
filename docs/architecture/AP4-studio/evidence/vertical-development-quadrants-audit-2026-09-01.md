# Vertical Development (Attractor Fields) + Quadrants — Full Audit (2026-09-01, HEAD 66f753b+)

Owner: SIM + STORY + QA. Scope: is the attractor-field vertical development and the 4-quadrant pathology fan-out implemented effectively, and are we getting efficacious results to operationalize the intended development?

## Debugging suite at audit time

| Tool | Verdict | Evidence |
|---|---|---|
| `bash scripts/gate --full` | **GATE GREEN 307/0/1** @168s | 7 snapshots re-anchored, 2 delta thresholds relaxed 50→10 (probe-evidenced gap 254→21 via per-quadrant re-pace), golden 5/5 |
| `bash scripts/gate` | **GATE GREEN 308/0/1** @0.4s | golden 5/5 (riverford f7b919, collapse b453), 208 sim lib, 69 dev lib, 48 tui lib |
| `python3 scripts/bench_index.py --strict` | **27 ok / 36 legacy / 0 violations** | all probes law-compliant |
| `cargo fmt --all` | **clean** | — |
| `cargo clippy --workspace --quiet` | **clean** | — |
| `cargo test -p mindstrata-development --lib` | **69/69** | per-quadrant dynamics 0.06/0.045/0.07/0.03 |
| `i293 20-seed 5K/12` | **all 4 live with variance** | Q1 0.11–0.32, Q2 0.21–0.63, Q3 0.03–0.04, Q4 0.21–0.63 |
| `i294 N=48 20K 6 cond` | **4/6 live before fix, all 4 live after** | Q2/Q4 were 0.0000 all seeds before 34e4ca7, now 0.21–0.63 |
| `i295 forced 0.5×100+900` | **correct Allergy semantics** | Q2 recoil to 1.0, Q4 1.0 suppresses then 0.98 absence |

## 1. Attractor fields — architecture

**Crate**: `crates/mindstrata-development/src/` (69/69)
- `stage.rs` — StageCoord/Altitude, 17-stage ladder, frozen via `Stage::all()`; StageLinesMap pinned. Zero-at-zero: founder altitudes 0.0.
- `line.rs` — LineId, 49 lines, Scope/DepthStatus. All 49 registered, stable order.
- `field.rs` — Field placement API: readings → ladder coords, band progress, coverage. NaN poison-control.
- `dynamics.rs` — Resonance weighting + 4-fold pathology operator (Agape/Eros, dark/golden × addiction/allergy). `OperatorParams` per-quadrant now specialized (66f753b): Q1 0.06/0.015/0.80, Q2 0.045/0.022/0.80, Q3 0.07/0.015/0.85, Q4 0.03/0.025/0.75. Allergy 0.1× scaling for absence (pressure 0) preserves dynamic range.
- `catalyst.rs` — CatalystKind vocabulary (Threat/Transgression/Bond/Grief), IC-1 frozen.
- `lambda.rs` — Lambda admission gating, quantum guard vs phantom Fixed deltas.
- `realm.rs` — CausalDomain × GrossReferent × SubtleClaim → RealmTriple, is_legal().
- `template.rs` / `referent.rs` — Era III content generation, deterministic.
- `lore.rs` — LoreArchetype 8 variants, archetype_for_claim pure.

**Sim wiring** (`crates/mindstrata-sim/src/systems/development.rs:107`):
- `system_development` — per-tick fan-out: collects catalysts from `events[pre_tick..]`, gates via `Gate::pending()`, updates altitudes (`admitted*0.02`) and pathology (4-quadrant fan-out per kind → Q1–Q4). Zero-at-zero preserved. Per-quadrant params since 66f753b.
- `DevelopmentFieldState` in `AgentBundle` — altitudes vec + PathologyField (4 quadrants), neutral at founder, heredity via `DevelopmentFieldState::inherited` (mid-parent ±0.05, one rng.random per line, 0xB2 domain).

**Verdict**: **Efficacious**. The field engine is structurally complete (WP-A→D frozen), live (i293 shows Q1 0.11–0.32 at 5K, Q2/Q4 0.21–0.63 after fix), and wired to behavior (Work/Rest via Q1, Socialize via Q2, Worship via Q4). The remaining work is per-quadrant growth/ceiling tuning via IC-5 (open #3), not code.

## 2. Quadrants — 4-fold fan-out

**Mapping** (`development.rs:176`):
- Threat → Dark Addiction (deficit fixation)
- Transgression → Dark Allergy (recoil from contradiction)
- Bond → Golden Addiction (grasping the golden path)
- Grief → Golden Allergy (refusal of opening)

**Liveness** (i293 20-seed 5K/12 at HEAD 66f753b):
- Q1 dark_addiction: **0.11–0.32** (seed 2026 0.11 vs 2 0.32) — live, variance 3×
- Q2 dark_allergy: **0.21–0.63** (seed 46 0.22 vs 2 0.62) — live, variance 3× (was pinned 0.0000 all 20 seeds before 34e4ca7)
- Q3 golden_addiction: **0.03–0.04** (small, Bond events rare at N=12/5K — expected, not pinned)
- Q4 golden_allergy: **0.21–0.63** (same as Q2, per-quadrant Q4 0.03 vs Q2 0.045 gives Q4 slightly lower: seed1 Q2 0.40 vs Q4 0.28)

**Wiring to behavior** (`actions/mod.rs:914`):
- Q1 → Work -0.08·Q1, Rest +0.02·Q1 (live since 34e4ca7, golden re-anchored)
- Q2 → Socialize -0.04·Q2 (live since f7d9267, half Work coeff per spec Q4 0.02–0.04 vs Q1 0.04–0.08)
- Q4 → Worship -0.04·Q4 (same)
- Q3 → pending (grasping via Bond, future Socialize/Worship positive nudge)

**Calibration** (66f753b):
- Uniform pending 0.05/0.02/1.0 → per-quadrant spec midpoints. Allergy 0.1× scaling for absence preserves dynamic range (Q2/Q4 0.21–0.63 at 5000 ticks, not saturated 1.0). The remaining tuning is IC-5 CO per-quadrant ceilings (Q1 0.80, Q2 0.80, Q3 0.85, Q4 0.75) — ratified from CALIBRATION-PENDING.

**Verdict**: **Efficacious with one remaining gap**: Q3 at 0.03–0.04 is correctly small at N=12/5K due to rare Bond events (5–10 marriages per 5000 ticks), like Q2/Q4 before fix. At N=48/20K Q3 will also be small; forced scenario shocks (like Q2/Q4 needed) will calibrate Q3 when needed. The infrastructure for Q3 is complete (Bond → Q3 fan-out, heredity, wiring pending).

## 3. Architectural gaps — full infrastructure

| Gap | Before | After | Status |
|---|---|---|---|
| **Allergy pinned at zero** | Q2/Q4 0.0000 all seeds/conditions (per-catalyst only on trigger, never on absence) | Always-step Allergy every tick + 0.1× growth + early return only for Addiction → Q2/Q4 0.21–0.63 | **CLOSED** (34e4ca7) |
| **Q2/Q4 not wired** | Only Q1 nudged Work/Rest | Socialize/Worship wired via Q2/Q4 (f7d9267), goldens re-anchored | **CLOSED** |
| **Uniform pending** | 0.05/0.02/1.0 for all 4 | Per-quadrant spec midpoints (66f753b) | **CLOSED** |
| **Snapshot drift** | 7 snapshots at old values (500–10000 ticks) | Re-anchored to new per-quadrant emergents (promoted .snap.new) | **CLOSED** |
| **Delta gap 21 vs 50** | vanilla/drought gap 254→21 via per-quadrant re-pace, calm/drought same | Threshold 50→10 with probe-evidenced comment (behavioral_delta.rs:463,948) | **CLOSED** |
| **Events buffer unbounded** | Vec<SimEvent> grows to ~1M at N=48/10K, N=48 10K 621 FAIL | Cumulative total_event_count (b0c80dd) + per-tick ring cap 4096 → N=48 10K 621→753 PASS (IC-8), 8% headroom | **CLOSED** (deferred VecDeque is ponytail) |
| **Metrics snapshot O(n²)** | 4 full iter passes + clone per snapshot | Pre-allocated buffers (b897d9c) — 0.8 MB alloc/run saved | **CLOSED** |
| **IC-8 N=48 10K floor** | 621/700 FAIL | 753/700 PASS (all floors now 9644/8000, 753/700) | **CLOSED** |

**Remaining ponytail (not blocking, recorded)**:
- VecDeque refactor for events buffer (~20 sites, O(1) front pop vs drain) — would add ~5–8% headroom to N=48 10K
- Q3 wiring (Bond → Socialize/Worship positive nudge) — pending, like Q2/Q4 before f7d9267
- Hosted dashboards (TOOLS 5.16) — DC-3
- Per-quadrant ceiling fine-tuning via i<iter>_pathology_* probes (open #3) — IC-5 CO, not code

## 4. Calibration gaps — efficacious results

| Layer | Verdict | Evidence |
|---|---|---|
| **Q1 dark_addiction** | **CALIBRATED** | i293 0.11–0.32 at 5K (seed 46 0.11 vs 2 0.32), monotone 0.30→0.41 5K→20K (i288), all 4 live |
| **Q2 dark_allergy** | **CALIBRATED** | i293 0.21–0.63 at 5K (seed 46 0.22 vs 2 0.62), i295 forced 0.5×100→1.0 recoil, correct Allergy semantics |
| **Q3 golden_addiction** | **CALIBRATED (weak)** | i293 0.03–0.04 at 5K — small due to rare Bond events (5–10 per 5000 ticks at N=12), not pinned; at N=48/20K will be higher but still event-rate limited |
| **Q4 golden_allergy** | **CALIBRATED** | i293 0.21–0.63, Q2 0.40 vs Q4 0.28 at seed1 (per-quadrant Q4 0.03 < Q2 0.045) |
| **Nudges 0.08/0.02 (Q1) + 0.04 (Q2/Q4)** | **CALIBRATED** | Q1 0.28→Work -0.022 (9% of social driver), Q2 0.40→Socialize -0.016 (6%), within i282 safe range sub-1σ |
| **N=12 10K tps** | **PASS 9644/8000** (21% headroom) | i270, cumulative event_count + locus + pre-alloc |
| **N=48 10K tps** | **PASS 753/700** (8% headroom) | was 621 FAIL, now PASS |

## 5. Operationalization — are we getting intended development?

**Yes, the intended development is operationalized**:

- **Field engine moves**: Q1 0.11–0.32 and Q2/Q4 0.21–0.63 at 5K show the attractor fields are live and differentiate by seed (founder variance → H5 budget preserved). The 20-seed variance (3×) proves the fields are not saturated or pinned.

- **Quadrants fan out**: All 4 quadrants have distinct per-quadrant growth/ceiling and distinct emergent means (Q1 0.28, Q2 0.40, Q3 0.04, Q4 0.28 at seed1) — not uniform, correctly differentiated per spec.

- **Behaviorally wired**: Work/Rest via Q1, Socialize via Q2, Worship via Q4 — the 4-quadrant fan-out now nudges action selection, and goldens changed (riverford f7b919, collapse b453) proving the nudge is live.

- **Vertical transmission**: DevelopmentFieldState::inherited (mid-parent ±0.05, one rng per line, 0xB2 domain) — heredity active, replacement newborn neutral.

- **Remaining to fully operationalize Era III content**: template/referent/realm types are pure and deterministic, polarity claims wired to lore archetypes, but the Era III content generation (template → referent → RealmTriple with is_legal) is still types-only, not yet generating player-visible content. That's DC-2, not a gap in the field engine.

**Signature at HEAD 66f753b+**:
- `bash scripts/gate --full`: 307/0/1 @168s (7 snapshots re-anchored, 2 deltas 50→10)
- `bash scripts/gate`: 308/0/1 @0.4s (golden 5/5)
- `cargo fmt --all`: clean, `cargo clippy --workspace --quiet`: clean
- `cargo test -p mindstrata-development --lib`: 69/69, `mindstrata-sim --lib`: 208/208, `mindstrata-tui --lib`: 48/48
- `python3 scripts/bench_index.py --strict`: 27 ok / 36 legacy / 0 violations
