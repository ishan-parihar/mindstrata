# Iteration 280 — WP-J institutional coupling: dead-channel diagnosis + live Work-axis rewiring

**Status:** LANDED · **Doctrine:** probe-first; §4.3 ("a dead producer is a bug even when tests pass") applied to a *channel*, not a producer.

## What was wrong

WP-J: "institution behavior parameters become functions of governance/economic-systems line stages (read-side multipliers)." The groundwork `InstitutionsMultiplier` (SIM 4.25, inert) got its live constructor `compliance_multiplier` (band-III gated, ramp 0.05/stage, mult 1.10 at the i280-measured 20K stage 6.0) and was wired into the §12.3 institution-morale→norm-compliance channel in `pass_action`.

Result: **stash-controlled A/B byte-identical at every horizon.** The multiplier executed; nothing observable changed.

## The diagnosis (liveness probes, all committed)

1. `i280_institution_coupling.rs` — zero-blast window: governance/economic-systems sit at **1.0 / 2.0 / 3.0** at the 2K/5K/10K horizons (seed 42), 6.0 at 20K. Band-III gate (4.0) is therefore structurally zero-blast at all pinned horizons and live beyond.
2. `i280_liveness_diag.rs` — channel inputs are **live**: 3 institutions, 10/12 members, mean morale 0.114. Yet forced-stage (all lines → 12.0) still produced no compliance-surface movement.
3. `i280_violations_sweep.rs` — the decisive probe: **zero `NormViolated` events across 6 seeds × {5K, 20K}**. The §12.3 compliance surface is *dead at N=12 in every natural regime*: there is nothing for a compliance multiplier to modulate. WP-J had coupled a live multiplier to a dead channel — the same §4.3 pattern as the i274 Relational feed, one level down.

## The fix — channel #2 on the live Work axis

- `member_work_bonus(morale, mult)` in `institutions_multiplier.rs`: zero below the band-III gate (identity multiplier ⇒ zero bonus), above it `morale × 0.1 × mult` — at the natural 20K regime (morale ~0.11, mult 1.10) the bonus is **0.0121, dread/hope-class nudge**, not a reordering lever. Computed in f64, quantized once (§5 truncation rule).
- New `DecisionContext.institution_work_bonus` field; summed per agent per tick over live memberships in `pass_action` and added to Work utility in `select_action` (Work is the provably live provisioning surface: 4K trades, constant grain production).
- Channel #1 (morale→compliance) is **retained**: its surface is dead at N=12 but it is the honest §12.3 semantics and revives in violation-positive regimes (larger N, weaker norms).

## Liveness evidence (unit-level A/B, dread-pin method)

`institution_work_bonus_shifts_selection_toward_work` — identical RNG streams, only the bonus differs:
- bonus 0.0121 (live 20K magnitude) → strictly more Work selections than the zero-bonus control (which is simultaneously the zero-blast leg: bonus 0 ⇒ byte-identical legacy selections);
- monotone at the forced-stage ceiling (0.07);

plus `member_work_bonus` unit pins (zero at identity multiplier, 0.11×0.1×1.10 = 0.0121 exact, monotone in morale, zero morale ⇒ zero).

The end-to-end forced-stage A/B was **rejected as confounded**: forcing all 29 collective lines to 12.0 also shifts the tetra-arising genesis bands and every stage-gated system (memesgen 14→29), so no end-to-end delta can isolate this channel. The unit pin carries the liveness proof; the end-to-end zero-delta at pinned horizons carries the zero-blast proof.

## Gate + re-anchor status

Full gate **GREEN 307/0/1**, clippy 0 warnings, zero snapshot/golden re-anchors — the bonus is structurally zero at 1000/4320/10K (governance ≤ 3.0 < gate 4.0). Sim lib 232/232 (2 new function pins + 1 liveness pin).

## Recorded debt

- Institution morale equilibrium (~0.11 mean) is itself uncalibrated — no probe has ever pinned its producer sensitivity. If WP-J is to matter behaviorally below band III, that calibration comes first.
- The violations sweep makes §12.3's natural-regime surface a recorded N=12 diet fact (same family as i272 mortality-horizon debt): violation-positive regimes are the untested accelerator.
