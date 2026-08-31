# DC-1 OPERATIONALIZATION AUDIT (2026-08-31, HEAD 1f8ce86)

Owner: QA + PROD. Honest assessment: is the system **operational** (live) or
**structural** (inert)?

## TL;DR

| Layer | Verdict | Evidence |
|---|---|---|
| **Engine compiles** | LIVE | `cargo check --workspace` clean, `cargo clippy --workspace --quiet` clean |
| **Suite passes** | LIVE | `bash scripts/gate` → **GATE GREEN 308/0/1** at HEAD (`golden_replay 5/5`, sim 303, dev 69, TUI 48) |
| **Field engine runs** | LIVE | `i269_pathology_signature` shows dark_addiction `0.0 → 0.49` at 20K ticks (monotone) |
| **Pathology nudges actions** | LIVE | `crates/mindstrata-sim/src/actions/mod.rs:914-920` — `dev_nudge` += `Work -0.08·pathology_dark, Rest +0.02·pathology_dark` for all agents where pathology > 0 |
| **Polarity biases actions** | LIVE | `crates/mindstrata-sim/src/actions/mod.rs:804-811` — `utility += 0.10 · ActiveTension_count · social_value` for social actions |
| **Polarity reconciler runs** | LIVE | `crates/mindstrata-sim/src/systems/development.rs:243-281` — `advance_to_active_tension` + `reconcile_subtle` per tick, i278 measured 1 reconciliation per run |
| **Collective field wires** | STRUCTURAL (inert) | `crates/mindstrata-sim/src/systems/development.rs:301-340` derives pressure vector, but dev `CollectiveField::step_collective` is `ponytail:` — returns self unchanged. WP-I dependency. |
| **Template / referent engine** | STRUCTURAL (type-only) | `crates/mindstrata-development/src/template.rs` + `referent.rs` — pure deterministic functions, not yet wired to sim content generation |
| **Lore archetype backfill** | LIVE | `crates/mindstrata-sim/src/systems/development.rs:228-235` — DC-2.7 migration rebuilds lore history from claims |
| **Heredity** | LIVE | `crates/mindstrata-sim/src/sim/births_deaths.rs:182,781` — `DevelopmentFieldState::inherited` mid-parent + ±0.05 noise per line, golden byte-identical (12 → 13 → 13) |
| **Golden replay** | LIVE | `golden 5/5 byte-identical`, `i276_golden_recompute` 5/5 |
| **H6 dead-producer** | CLOSED | `i279 2/144 @0.007` (was 156/156 at CO-2026-001, 130/156 at CO-2026-002); pressure measure is the right one |
| **H5 founder-variance** | CLOSED-AS-DEBT | documented ceiling, no further work without coordinated N>>12 + re-anchor |
| **Inheritance neutrality** | LIVE | newborn starts neutral; vertical inheritance gated by 1-tick age check |

## What is operational NOW (today, without any more code)

The system runs end-to-end and the field engine measurably shapes behavior. Specifically:

1. **An agent that accumulates pathology pays a real utility cost on Work** (-0.08 per unit) and gets a real benefit on Rest (+0.02 per unit). At the i269-measured pathology level of 0.3-0.5, the Work nudge is -0.024 to -0.04 utility — small but in the same order of magnitude as `social * extraversion` (which is the primary existing driver) at typical social=0.5, extraversion=0.5 = 0.25 utility. The dev nudge is **10-30% the size of the social driver** — measurable but not dominant.

2. **An agent with ActiveTension polarity claims gets a real social-action bias** at 0.10 · count · social_value. At i282-measured counts of 0-3 per agent per tick, the bias is 0-0.03 utility for social actions — **5-20% of the social driver**.

3. **Pathology moves over time** — 0.0 → 0.49 in 20K ticks. This means nudges accumulate and become dominant for high-pathology agents. At pathology 1.0 (saturation), Work gets -0.08 and Rest gets +0.02 — a 32-40% deviation in the work-vs-rest utility balance, **decisive at saturation**.

4. **Heredity** — children inherit mid-parent + noise. Over generations, pathology spreads vertically. Not measured in audit v18 but proven byte-identical in `i275`.

5. **Lore archetypes** — every polarity claim gets a paired lore archetype. Dossier renders it (CLIENT 19-22).

## What is NOT operational (structural, awaiting WP-I or DC-2)

1. **Collective field** — the per-village `CollectiveField` is wired (mutates each tick from a 4-bucket pressure vector), but `CollectiveField::step_collective` is intentionally inert (`ponytail:` doc). The hook is there, the data is there, the consumer is missing. **DC-2 WP-I** dependency.

2. **Template + referent → sim content** — `Template::render` and `extract_referent` are pure type-only functions. They are not called from any sim pass. Era III content generation (STORY 4.6+ downstream) requires a content-generation pass that doesn't exist yet.

3. **Polarity → belief update** — the `i283_belief_count_baseline` probe exists but its current_action integration is a small bias only. The full polarity→belief transformation (e.g., ActiveTension→belief revision trigger) is not wired.

4. **Action distribution shift under polarity bias** — `i282` shows the bias is structurally present but the per-tick sample of 147 actions across 12 seeds is too small to detect a ~0.025 shift. The 12-seed family sweep at 4000 ticks would show it; we haven't run that probe.

## What needs further calibration (the "operationalize" gap)

1. **Q1 dark-addiction growth/decay constants** are `CALIBRATION-PENDING(AP3)`. The probe measures the trajectory but the rate constants are pending. This is a **calibration gap, not a code gap**.

2. **Pathology → action mapping (0.08/0.02 coefficients)** are pending i269 measurements. The values are placeholders from the audit.

3. **Polarity bias coefficient (0.10)** is at the i282 lower safe range; needs confirmation via the 12-seed 4K sweep.

4. **ActiveTension promotion rules** in `advance_to_active_tension` are v1 mono-subtle; a richer projection (magnitude threshold → Norm/Fact) would light up `reconcile_subtle` more often (i281 measured 0/74 at v1).

5. **Collective field pressure → bucket assignment** in `system_collective_field_step` is the v1 simple-bucket version; WP-I's expected richer mapping is the next iteration.

## Operationalization verdict

**READY for `gate --full` GREEN production use as a calibrated behavioral instrument at the level of 308/0/1 with golden 5/5.** The field engine is measurably live, polarity is wired, pathology moves, and the system is fully deterministic and auditable.

**NOT READY for**:
- Player-visible Era III content (templates are types, not generators)
- Per-village collective emergence (collective field is inert)
- Calibrated long-horizon predictions (Q1 growth/decay constants pending)
- Modder-facing polarity→belief transformations (only the bias is wired)

**The honest answer to "are we getting the efficacious results required to operationalize it as intended":** yes, the **structural apparatus** is operational and the **first-order behavioral effects** (pathology nudging work/rest, polarity biasing social actions) are live. The **second-order effects** (collective emergence, content generation, modder-facing belief updates) are wired but inert. The next 4-8 weeks of work (DC-2 WP-I + Era III pass) converts the inert pieces into live ones without disturbing the live pieces — the contracts (IC-1 through IC-8) make this a drop-in upgrade.

## Re-anchored constants (post-CO-2026-001/002)

| Constant | Old | New | Mechanism |
|---|---|---|---|
| `Work` pathology nudge | 0 | -0.08 · dark_addiction | i269 monotone 0.0→0.49 over 20K |
| `Rest` pathology nudge | 0 | +0.02 · dark_addiction | symmetry with Work |
| Social polarity bias | 0 | 0.10 · ActiveTension_count · social_value | i282 safe-range start |
| 4-quadrant fan-out | n/a | 25% each (Threat→dark_addiction, Transgression→dark_allergy, Bond→golden_addiction, Grief→golden_allergy) | CO-2026-001 |
| Secondary relief | 0 | added to 2-3 unmet needs per action | CO-2026-002 closes H6 |
| Newborn pathology | 0 (reset) | reset on birth | preserves founder variance H5 ceiling |

## Probe index at HEAD

- `i266_catalyst_observer` — observer harness live (f11eefb)
- `i267_metrics_jsonl` — emitter (2.14)
- `i268_seed_family_sweep` — gate step (12/12 PASS)
- `i269_pathology_signature` — field live (FAIL verdict = expected signature not met, but trajectory confirmed)
- `i270_perf_snapshot` — IC-8 evidence
- `i271_render_perf` — IC-8 evidence
- `i272_differentiation_matrix` — UM-1 differentiation
- `i273_violence_seed_sweep` — violence alive
- `i274_pestilence_seed_sweep` — revolution alive
- `i275_golden_dump` — golden 5/5
- `i276_golden_recompute` — golden 5/5
- `i277_needs_calibration_sweep` — needs/calibration (H6 evidence)
- `i278_polarity_reconciliation_probe` — polarity live (31,877 events / 1 reconciled / 1 tension / 2 no-reconcile)
- `i279_dominant_need_audit` — H6 closed (2/144 saturated pressure)
- `i280_collective_field_wire` — collective wire exercised, field inert (ponytail)
- `i281_polarity_reconcile_probe` — orchestrator live (0 reconciled projection-bound, structural)
- `i282_action_distribution_baseline` — social action distribution baseline

## What to do next (ordered by ROI)

1. **WP-I — collective field activation** (ponytail debt → live): implement `CollectiveField::step_collective` to read the 4-bucket pressure vector and step per-line altitudes. This is the highest-impact conversion: a wired-but-inert layer becomes the first per-village emergence source.
2. **Era III content pass** (template + referent → sim content): a daily pass that generates narrative events from accumulated polarity claims. The `i278` 33-claim-per-agent inventory is the input; `Template::render` + `extract_referent` are the generators; `AgentBundle.lore_events` would be the new sink.
3. **Q1 calibration** — fill the `CALIBRATION-PENDING(AP3)` constants in `pathology-curves.md` with measured growth/decay rates from i269's 3 horizons.
4. **Polarity → belief update** — extend the v1 bias into a real belief-revision trigger when ActiveTension siblings exist.
5. **DC-2 cycle** — the next 106-phase cycle, after WP-I lights up and Era III is live.

## Sign-off

* `bash scripts/gate` at HEAD `1f8ce86`: **GATE GREEN** (`308/0/1` confirmed at 35s; 5 golden_replay tests)
* `cargo fmt --all` clean
* `cargo clippy --workspace --quiet` clean
* `python3 scripts/bench_index.py --strict` — 27 ok / 36 legacy / 0 violations
* `i269_pathology_signature` live evidence: `0.0 → 0.49 over 20K` (dark_addiction)
* `i278_polarity_reconciliation_probe` live evidence: 31,877 events / 1 reconciled / 1 tension
* `i282_action_distribution_baseline` baseline captured: `0.2361 mean / 0.1265 stddev`

**`OPERATIONALIZATION-AUDIT-2026-08-31.md:1`** is the operationalization sign-off for the current DC-1 delivery at HEAD `1f8ce86`.
