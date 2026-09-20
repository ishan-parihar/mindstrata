# Difficulty Levers Catalog — Post-AA Knobs (DESIGN 9-10)

Owner: DESIGN → PLATFORM/SIM implementation via `IC-5` `CO-` only.
Status: **v1.3 — rows 2 (need decay i303 + fulfillment thresholds i305) and 3
(pathology growth/decay i304 + ceiling i315) FULLY LIVE; row 1 still DRAFT
(prohibited, see below).**
Companion to `pacing-model.md` (which horizon each lever paces) and
`canon-inventory.md` rows 1/2/7.

## How a lever becomes a setting

A lever is a `canon.rs` constant (or small param group) that today is a single
tuned value and tomorrow is a `Low/Medium/High` enum mapped to three `CO-`
approved bands. The mapping is pure data — `match difficulty { Low => 0.85, … }`
— so `cargo test` guards each band and `i268` family sweeps guard stability.

| # | Lever (canon group) | Horizon paced | Low / Medium / High (candidate bands) | What the player feels | Probe that proves the knob works |
|---|---|---|---|---|---|
| 1 | **Founding variance** — `FOUNDER_SPREAD` shaping `U(0,1)` line-profile draws (AGENTS §5 debt, `needs` line) | 10K clusters | Narrow `σ=0.12` / Standard `σ=0.20` / Wide `σ=0.29` (variance 1/12) | Villages diverge visibly vs feel same — `inter` `0.12` floor fails on Narrow, `0.18` on Standard (current `i272` 0.1817), `0.25+` on Wide | `i272_differentiation` `inter/intra` sweep + `i268` family `12/12` must stay `PASS` at all three settings |
| 2 | **Need decay / fulfillment thresholds** — both halves: the six decay rates AND the five goal gates (`retain 0.3`, `Eat/Drink 0.5`, `Rest 0.6`, `Socialize/Worship 0.7`) | 50K midcourse | Lenient `0.6×` / Standard `1.0×` / Harsh `1.4×` — **BOTH HALVES LIVE (decay i303, thresholds i305)** | Village feels abundant vs scarcity-driven — decay half at 2000 ticks × 12 seeds: aggregate need pressure +27%, Worship share −19%, thirst +34%, social +83%; threshold half IN ISOLATION (decay pinned) at 2K/20K × 12 seeds: need-driven goal duty `0.2457/0.0587/0.0190` and `0.7639/0.6361/0.5130` (lenient/standard/harsh) — monotone, no producer killed | `i303_difficulty_bands` + `i305_goal_gate_bands` (isolation leg + per-horizon producer-liveness) + `parameters::tests` + `sim/tests/psychology.rs` — DONE |
| 3 | **Pathology growth/decay/ceiling** — `PATHOLOGY_GROWTH_*` `0.02–0.10` / `DECAY 0.01–0.05` / `CEILING 0.65–1.0` per 4 quadrants | 100K lineage | Resilient `0.5×` growth `1.2×` decay `0.85×` ceiling / Standard `1.0×` / Brittle `1.8×` growth `0.7×` decay `1.15×` ceiling — **FULLY LIVE (growth/decay i304, ceiling i315)** | Lineage diverges slowly vs brittly — measured at 20K × 12 seeds: dark-addiction mean `0.197 → 0.341 → 0.480` (resilient/standard/brittle), direction 12/12 per seed; event counts flat within 2% while quadrant intensities differ 2.4×. Ceiling in isolation (growth/decay pinned): Q2 `0.519 → 0.615 → 0.718`, direction 12/12 both comparisons, and every quadrant is ceiling-sensitive | `i304_pathology_bands` (family differential + resolved-band table + Standard-identity) + `i315_pathology_ceiling` (isolation leg) + `systems::development` pins + `parameters::tests` — DONE |

## Non-goals (not levers, deliberately)

* Stage band boundaries (`3..=7` somatic etc.) — theory-derived ladder, not difficulty.
* Resonance affinities (`STORY` vendored `2117`-row map via `WP-0A`) — frozen substrate.
* Tick/calendar math (`IC-3` `f64 shadows`, `Fixed::mul` quantization) — determinism hazard, never a knob.

## Promotion rule (FR-051 / IC-5)

A lever graduates from `DRAFT` to a difficulty setting only when its three bands
each have a `CO-` citing `measured/old_band/mechanism` + `i268` `11/12` stability
+ `suite+golden` green at that band. Until then the single `Medium` value is the
`CANON` and `Low/High` remain design hypotheses in this doc.

**Row 2 promoted (i303)** on this rule: the surface is
`mindstrata_core::parameters::DifficultyProfile` +
`SimParameters::with_difficulty` (quantize-once band mapping), persisted in
Snapshot v16 and exposed as the CLI `--difficulty lenient|standard|harsh`.
Standard is **byte-identical to the ratified canon by construction**
(`with_difficulty(Standard)` returns `default()` untouched) — probe-pinned
params-serde + end-state-digest identity — so the promotion carries **no
golden/snapshot re-anchor**. Measured bands and the honest quantization
findings (Lenient social rounds to half canon; Lenient meaning is
sub-resolution and collapses onto Standard) live in
`docs/architecture/AP4-studio/evidence/i303_difficulty_levers.md` and are
pinned in `parameters::tests`. Row 2 residual (not claimed): the
*fulfillment-threshold* half of the lever — the 0.3/0.5/0.6/0.7 goal gates
are still inline constants in `system_goal_generation`.

**Row 2 threshold half promoted (i305)** on the same rule: the surface is
`SimParameters.goal_gate_scale` + `systems::GoalGates` (resolved once per tick),
with Standard **bit-identical by construction** (canon gates × 1.0), so again no
golden/snapshot re-anchor. Evidence and the findings that go with it — the
meaning channel saturating at 20K (Worship duty 41.5% vs Socialize 0.43%), the
harsh band dropping the short-horizon Drink producer, and the measured
non-monotonicity of the two halves combined — live in
`docs/architecture/AP4-studio/evidence/i305_goal_gate_bands.md`.

**Row 1** stays Medium-only under the standing AGENTS §5 H5 prohibition
(founder-variance draws are load-bearing at N=12; reshaping requires larger
N + a coordinated re-anchor sweep, never piecemeal). **Row 3 promoted (i304)** on the same rule: the surface is
`SimParameters::{pathology_growth_scale, pathology_decay_scale}` +
`systems::development::pathology_params` (resolve once per tick) +
`system_development_with_params`, with Standard **bit-identical by
construction** (the resolver multiplies by exactly 1.0 — IEEE-754 identity),
so again no golden/snapshot re-anchor. Measured bands and findings (Q2
near-saturation debt, ceilings deliberately not a band) live in
`docs/architecture/AP4-studio/evidence/i304_pathology_bands.md` and are pinned
in `parameters::tests`, `sim/tests/development.rs` and the two snapshot pins.

**Row 3 ceiling half promoted (i315)** on the same rule: the surface is
`SimParameters.pathology_ceiling_scale` (scaled in `pathology_params`, clamped
`[0, 1]`), with Standard **bit-identical by construction** (`x * 1.0`), so again
no golden/snapshot re-anchor. Resilient `0.85×` (lower caps) / Brittle `1.15×`
(higher caps) map the canon ceilings (0.75–0.85) into the catalog's `0.65–1.0`
range. Isolation probe (`i315_pathology_ceiling`, growth/decay pinned at canon):
Q2 `0.519 → 0.615 → 0.718`, direction 12/12 both comparisons; every quadrant is
ceiling-sensitive because growth is `headroom = ceiling − intensity`. Row 3 is
now **fully live** (growth/decay i304, ceiling i315).

**Residual, CLOSED (i318)**: the Allergy absence-growth law had no
pressure-independent relaxation, so pure absence was monotone growth to the
ceiling and the quadrant's long-horizon equilibrium was the *horizon*, not the
agent's reconciliation diet (probe `i318_allergy_dynamic_range`: 0% of Q2 agents
within 5% of the ceiling at 20K, 50% at 50K, **64% at 100K**). Added
`ALLERGY_RESTING_RELAXATION = 0.05 × decay`, giving the absence attractor
`I* = growth·0.1·ceiling / (growth·0.1 + decay·0.05)` — strictly below the
ceiling for every quadrant and scaled by the row-3 **decay band**. Post-fix the
ceiling pile is **0% at every horizon** and the Q2 asymptote (0.643) lands inside
the catalog's `0.6–0.7` band. Both row-3 lever verdicts re-run live at HEAD;
one long-horizon snapshot re-anchored (mechanism in
`evidence/i318_allergy_resting_relaxation.md`), goldens byte-identical.
