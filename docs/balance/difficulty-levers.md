# Difficulty Levers Catalog — Post-AA Knobs (DESIGN 9-10)

Owner: DESIGN → PLATFORM/SIM implementation via `IC-5` `CO-` only.
Status: **DRAFT v0.9 — 3 levers that will become difficulty settings after AA.**
Companion to `pacing-model.md` (which horizon each lever paces) and
`canon-inventory.md` rows 1/2/7. No code touched until `IC-5` `CO-` lands.

## How a lever becomes a setting

A lever is a `canon.rs` constant (or small param group) that today is a single
tuned value and tomorrow is a `Low/Medium/High` enum mapped to three `CO-`
approved bands. The mapping is pure data — `match difficulty { Low => 0.85, … }`
— so `cargo test` guards each band and `i268` family sweeps guard stability.

| # | Lever (canon group) | Horizon paced | Low / Medium / High (candidate bands) | What the player feels | Probe that proves the knob works |
|---|---|---|---|---|---|
| 1 | **Founding variance** — `FOUNDER_SPREAD` shaping `U(0,1)` line-profile draws (AGENTS §5 debt, `needs` line) | 10K clusters | Narrow `σ=0.12` / Standard `σ=0.20` / Wide `σ=0.29` (variance 1/12) | Villages diverge visibly vs feel same — `inter` `0.12` floor fails on Narrow, `0.18` on Standard (current `i272` 0.1817), `0.25+` on Wide | `i272_differentiation` `inter/intra` sweep + `i268` family `12/12` must stay `PASS` at all three settings |
| 2 | **Need decay / fulfillment thresholds** — `NEEDS_BANDS` 6 bands `0.25–0.92` (safety/belonging/esteem/meaning) | 50K midcourse | Lenient `0.6×` decay / Balanced `1.0×` / Harsh `1.4×` | Village feels abundant vs scarcity-driven — `Socialize`/`Work`/`Worship` daily ratios shift, feud escalation `±30%` | `i<iter>_needs_gate_*` sweeps (planned in `needs-bands.md` probe table) + `World.tick()` resource EMA |
| 3 | **Pathology growth/ceiling** — `PATHOLOGY_GROWTH_*` `0.02–0.10` / `DECAY 0.01–0.05` / `CEILING 0.65–1.0` per 4 quadrants | 100K lineage | Resilient `0.5×` growth `1.2×` decay / Standard `1.0×` / Brittle `1.8×` growth `0.7×` decay | Lineage diverges slowly vs brittly — dark-addiction half-life `69→23→12` ticks, golden-allergy stickiness | `i<iter>_pathology_*_signature` 20K sweeps (planned `pathology-curves.md`) + fear equilibrium co-check (anti-pinning CA-2) |

## Non-goals (not levers, deliberately)

* Stage band boundaries (`3..=7` somatic etc.) — theory-derived ladder, not difficulty.
* Resonance affinities (`STORY` vendored `2117`-row map via `WP-0A`) — frozen substrate.
* Tick/calendar math (`IC-3` `f64 shadows`, `Fixed::mul` quantization) — determinism hazard, never a knob.

## Promotion rule (FR-051 / IC-5)

A lever graduates from `DRAFT` to a difficulty setting only when its three bands
each have a `CO-` citing `measured/old_band/mechanism` + `i268` `11/12` stability
+ `suite+golden` green at that band. Until then the single `Medium` value is the
`CANON` and `Low/High` remain design hypotheses in this doc.
