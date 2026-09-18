# Iteration 292 — dose calibration A3+A4 (PLAN_DC3 §4 i292)

**Status:** LANDED · **Root cause:** one — two constants shipped as first estimates
and never swept. **Both RATIFIED at their existing values with measured evidence** —
no behavioral change, zero re-anchors by construction (comments + pins only;
suite 307/0/1 unchanged).

## A3 — mourning-rite Agape dose (`household.rs`, pending since i285)

The Allergy consumption law is `next = I + g·0.1·headroom·(1−p) − decay·p·I`;
the dose is the rite's pressure. Closed-law sweep (`i292_dose_calibration`, production
Q4 params growth 0.03 / decay 0.025 / ceiling 0.75, at the i285-measured quadrant
levels I ≈ 0.2–0.7):

| dose | dI @ I=0.2 | dI @ I=0.3 | dI @ I=0.5 | dI @ I=0.7 | share of I |
|---|---|---|---|---|---|
| 0.30 | −0.00035 | −0.00131 | −0.00322 | −0.00514 | 0.2–0.7% |
| **0.60** | **−0.00234** | **−0.00396** | **−0.00720** | **−0.01044** | **1.2–1.5%** |
| 0.90 | −0.00434 | −0.00661 | −0.01117 | −0.01574 | 2.2–2.5% |

Reference: 500 no-rite ticks grow I 0.2 → 0.6276, so one rite-cycle's decay at 0.6
is a measurable-but-minor correction to the quadrant's own arc. Verdict: **0.6
RATIFIED** — inside the designed 0.5–3% metabolizer band, matching i285's in-vivo
village-level −0.019 (1 rite, 2–3 attendees over 12 agents). Dose 0.9 approaches
the single-rite override guard; dose 0.3 drifts toward the fixed-4 truncation floor.

Implementation: dose + production params lifted to pub consts
(`MOURNING_AGAPE_DOSE`, `PROD_QUADRANT_PARAMS`) so pins read live values, emitter
comment carries measured/old/mechanism per §4.2.

## A4 — norm-proposal strength cap 0.6 (`system_norm_proposal`, pending since i286)

In-vivo census at the i286 vivo shape (drought, 50K, seed 42):

- **natural stages: 1 proposal, strength exactly 0.6** — the cap was BINDING (the
  proposal carried quorum 8/12 = 0.67; the cap, not consensus, set its strength).
- forced-6.0 stages: 0 proposals (trajectory shifted downstream of i288 — i286's
  forced leg measured 4; recorded honestly as a downstream equilibrium shift).

Verdict: **0.6 RATIFIED as a binding guard** — it caps the founding dose of a
brand-new norm at majority breadth, deliberately leaving headroom for the §12.5
reinforcement channel to grow it; one-norm-per-slot dedup means an over-strong
founder could never be re-scaled down. The census also confirms the proposal
channel is now **reachable naturally** (i286's verdict was "structurally
unreachable without forcing") — the i288 Transgression feed revival closed that gap.

## Pins (sim lib 243/243)

- `mourning_rite_agape_dose_measures_in_metabolizer_band`: per-rite decay fraction
  within [0.5%, 3%] of I at I ∈ {0.2, 0.3, 0.5, 0.7}, reading `MOURNING_AGAPE_DOSE`
  and `PROD_QUADRANT_PARAMS.3` — any future drift of dose OR params trips it.
- `norm_proposal_cap_binds_above_and_passes_below_majority_breadth`: quorum 8/12
  caps to exactly 0.6.

## Verification

- fmt clean; clippy workspace 0 warnings; bench law 112 ok / 0 violations;
  full release suite **307/0/1** (202.9s); golden 5/5 (no behavioral edits —
  comment + const-lift + pins only).

**A3 + A4 CALIBRATION-PENDING markers CLOSED.**
