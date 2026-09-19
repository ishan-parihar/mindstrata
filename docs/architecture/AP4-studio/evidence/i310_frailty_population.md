# Iteration 310 — the frailty population: measured, and its driver named

**Status:** LANDED (measurement iteration, **no behaviour change**) · **Root
cause owned:** the i309 finding 1 ("derived health equilibrates at 0.21–0.25 for
the most stressed; the 0.25 gate is a chronic-frailty state — calibration debt").

i309 deliberately left the gate-split question open rather than guess. This
iteration measures the population the gate actually binds, and finds that the
chronic condition holding it there is not stress at all.

## How large the population is

Probe `i310_frailty_share`, 12-seed family, share of agents below the 0.25
derived-health gate at the horizon:

| context | population | below gate |
|---|---|---|
| calm @2 000 | 144 | **0 (0.0%)** |
| calm @20 000 | 157 | 5 (3.2%) |
| calm @50 000 | 173 | 6 (3.5%) |
| collapse @4 320 | 145 | **0 (0.0%)** |
| pestilence @4 320 | 146 | 0 (0.0%) |
| drought @4 320 | 146 | 0 (0.0%) |

Two things follow immediately. The population is **small and stable** (≈3.5%,
reached by 20K and flat thereafter), and — the reason i309 carried **zero
re-anchors** — it does not exist inside any pinned horizon, including the
4320-tick crisis scenarios.

## What holds them below the gate

Post-i309 those agents split their time 50% Rest / 50% Trade and never Work
(Work is vetoed by the very flag i309 added — the veto is doing its job). The
component means over the below-gate population:

| component | mean |
|---|---|
| endocrine stress level | 0.75 |
| chronic stress load | 0.90 |
| pain | **0.00** |
| **sickness** | **0.84** |

Pain is zero and stress is the ordinary calibrated level — the gate is held by
**sickness**, and `sickness_level = 0.6 × infection_load + 0.4 × inflammation`.
The immune anatomy (leg 2, all 6 agents) is identical across seeds, which is the
signature of a pinned equilibrium rather than bad luck:

| health | sickness | infection | inflammation | resistance | recovery_cap | injury | stress |
|---|---|---|---|---|---|---|---|
| 0.209–0.249 | 0.840 | **1.0000** | 0.600 | **0.0002–0.0199** | 0.040 | 0.000 | 0.73–0.79 |

## The mechanism, and the two defects it exposes

`ImmuneState::tick_update` drives resistance with saturating nutrition/sleep
boosts against a flat stress suppression, then computes the pathogen clearance
as `fight = resistance × recovery_capacity × 0.005 × infection_load × (0.7 +
0.6 × recovery_rate)`. For these agents resistance has **collapsed to ~0**, so:

1. **The fight term is sub-resolution.** At R = 0.0007, C = 0.04, I = 1.0:
   `fight = 1.4e-7` — computed in `Fixed`, that quantizes to **zero** at the
   4-decimal scale. The body clears nothing at all, so `infection_load` pins at
   1.0 forever. This is the AGENTS §5 sub-resolution-rate class (the same defect
   family that killed thermal convergence, faction pressure, gestation advance
   and epidemic exposure), and the immune path is a new instance of it.
2. **Resistance is absorbing at zero.** With stress suppression ~0.00075/tick
   against nutrition ~0.0002 + sleep ~0.0005, the net drift is negative, so
   resistance decays to ~0 and no equilibrium above zero exists for that agent.
   An immunocompromised state is then permanent: nothing in the model restores
   competence, and the resulting sickness keeps the derived-health gate shut,
   which (pre-i309) kept the agent out of every action that could have improved
   its circumstances — the trap i309 dismantled.

So the i309 "chronic frailty" reading was one layer short: the frailty is
**chronic infection held open by a quantized-to-zero clearance term over a
collapsed resistance**, not by stress per se.

## Verdict

**`FRAILTY_POPULATION_MEASURED_DRIVER_IS_IMMUNE_COLLAPSE`** — 3.5% of the calm
village, stable across horizons, held below the gate by pinned infection
(1.0000) over collapsed resistance (≈0.000), with the clearance term truncated to
zero in `Fixed`.

## Pre-registered plan for the fix (i311, NOT done here)

Deliberately not implemented in the same iteration, because the blast radius is
the epidemic knife-edge — AGENTS §5 records the R0≈1 TRANSIENT↔ENDEMIC flip as
systemic debt, and i252's R0 shift "slipped through" and is the reason the
collapse golden exists. The fix has two parts, and each needs its own
before/after measurement:

1. compute the clearance term in f64 and quantize once (§5's rule), so a
   low-resistance body still clears a proportional amount of pathogen;
2. decide the resistance floor / recovery path (competence must not be absorbing
   at zero) — with a probe sweep over the epidemic scenarios, since this is the
   parameter that moves R0.

Expected blast: the collapse golden, the pestilence tests, and possibly the
epidemic knife-edge pins. Every one of those must be re-anchored only with
measured evidence, and the fraction of the calm village below the gate should
fall from 3.5% toward 0 in the same measurement.

## Verification

- Workspace unchanged except a new bench example and docs: core **34/34**, sim
  lib **277/277**, tests-crate **309/0/1**, golden byte-identical, snapshots
  untouched, `scripts/gate --full` **GATE GREEN**, bench index `--strict` 0
  violations.

## Queue after this iteration

1. **Immune clearance fix** (this iteration's pre-registered plan) — the next
   behavioural candidate; it owns the epidemic knife-edge re-anchors.
2. Re-measure the i306 meaning channel over the frailty population (i309
   finding 3).
3. Levers row-3 residuals (ceiling band, Q2 dark-allergy saturation); row 1
   stays prohibited (AGENTS §5 H5); DC-4's CLIENT asset-viewer + graphical shell.
