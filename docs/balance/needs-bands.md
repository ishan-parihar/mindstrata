# Needs Bands — Fulfillment Thresholds (v1.0)

Owner: DESIGN → consumed by SIM gating (WP-G1, FR-027). Companion to canon-inventory
row 1. Theory grounding: `docs/architecture/AP3-afa/03-substrate.md` §6 + vault `needs` line
matrix (`needs` line kind `system`, collective-system quadrant) + KosmOS ladder rungs
1–4 (survival→belonging→achievement→self-transcendence→self-transcendence collective).
Frozen at 2.18; values CALIBRATION-PENDING(AP3) until probe-measured per FR-027.

## Evidence

- Existing mechanics: need decay rates live in `SimParameters`
  (`system_need_decay_with_params`, crates/mindstrata-sim/src/systems/mod.rs);
  goal generation gates on 0.3/0.5/0.6/0.7 (`systems::GoalGates::CANON`, the
  pre-i305 inline consts). These are empirical, NOT theory-cited —
  CALIBRATION-PENDING(AP3) per this spec.
- **i305 status**: the gates are now a runtime surface
  (`SimParameters.goal_gate_scale`, banded 0.6×/1.0×/1.4× by
  `DifficultyProfile`) resolved once per tick into `systems::GoalGates`. The
  MEASURED distributions the bands sit on, at 12 seeds × 2K/20K agent-ticks:
  hunger p50 0.025 / p99 0.413; thirst p50 0.116 / p99 1.0; fatigue p50 0.145 /
  p99 1.0; social p50 0.005 / p99 0.196; meaning p50 0.471 / p90 1.000
  (saturated). Two consequences for the rows below: (a) the proposed ranges for
  the survival/belonging/meaning gates sit ABOVE the range those channels
  occupy at the calibrated horizon, so the canon gate values were kept rather
  than moved to the spec's bands; (b) the meaning channel's saturation is the
  binding calibration problem in this table, and it is a decay-vs-relief
  balance issue, not a gate issue. Full tables:
  `docs/architecture/AP4-studio/evidence/i305_goal_gate_bands.md`.
- Motivation-context amplifications couple fear/anger/joy/sadness into drive pressures
  (Iteration-124 fix; see docs/MINDSTRATA_CURRENT_STATE.md historical note).
- Development field altitudes now live on every agent (task 3.1) and are updated
  daily by `systems/development::system_development` (task 3.2); gating (3.4) will
  read this spec's thresholds to decide action selection.

## Targets (probe-measured before values land — all ranges CALIBRATION-PENDING)

| Band (Drive) | Current constant(s) | Theory citation | Target range (v1) | Probe plan |
|---|---|---|---|---|
| Survival — hunger/thirst | 0.9 health damage, 0.5 Eat/Drink goal | Vault `needs` rung 1–2 (prehension/sensory-motor), Maslow physiological floor; AP3 03-substrate §6 survival→belonging ordering; `needs` line status `draft` but band shape attested by canon ladder | 0.85–0.92 damage / 0.45–0.55 goal | `i<iter>_needs_gate_survival_sweep` — sweep damage gate 0.8→0.95, measure health collapse days and daily Work/Wander ratio at seed 42/4242 family; expect cliff at ~0.9 |
| Safety — safety/scarcity | 0.7 analog to social | Vault `needs` rung 2–3 (red→amber transition), scarcity vs threat distinction in pathology operator (§5 collective-system) | 0.60–0.75 | `i<iter>_needs_gate_safety_delta` — vary safety retain 0.6→0.8, measure SeekSafety goal frequency and feud escalation rate |
| Belonging — social/attachment | 0.3 retain / 0.7 generate Socialize | `needs` rung 3 (amber), Communion drive (IC-1), attachment system (psych/attachment.rs) | 0.25–0.35 retain / 0.60–0.75 generate | `i<iter>_needs_gate_belonging_pacing` — sweep generate threshold, measure Socialize actions/day and kinship edge count at 10K ticks |
| Esteem — esteem/competence/recognition | derived at 2/3 meaning rate | `needs` rung 4 (achievement), Agency drive; Work/Trade relieve paths (sim/core.rs:657–665) | 0.55–0.65 (derived) | `i<iter>_needs_gate_esteem_scan` — sensitivity scan 0.5→0.7, measure Work vs Socialize trade-off and wealth Gini at 20K |
| Self-actualization — meaning/worship | 0.4 traditional gate, 0.7 Worship goal | `needs` rung 5–6 (formal/postformal), Agape/Eros drives, `meaning` need decay | 0.35–0.45 gate / 0.60–0.75 goal | `i<iter>_needs_gate_meaning_accum` — sweep gate 0.3→0.5, measure Worship actions/day and meaning deficit EMA at 10K |
| Self-transcendence — autonomy | 2/3 meaning rate | `needs` rung 7 collective field (Era II village holon), CollectiveField lambda | MEASURED (i282): deficit equilibrium median ~0.10 (workers) with ≥10% low-workload agents at deficit cap 1.0 — workload-differentiated per the i267 ODE design; pressure ceiling 0.3 (urgency 0.3 × cap 1.0) makes the draft [0.55,0.65] pressure threshold structurally unreachable (§4.4 re-contract — semantics invalidated by construction, not magnitude). Behavioral expression is via grievance (factions `compute_grievance`) + depression-risk (person/mind.rs) couplings, live; dominance argmax 0/112 by design (higher needs yield to survival). | `i282_transcendence_sweep` + `i282_autonomy_dominance` (3 seeds × 1K/5K/20K) — DONE |

## Open questions

1. Should thresholds be per-personality modulated (ambition/extraversion already shift
   goal generation)?
2. Zero-at-zero invariant: all bands must be identity-neutral until a development field
   moves (FR-027).
