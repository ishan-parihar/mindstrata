# Iteration 311 — immune clearance: the frailty population's mechanism, fixed

**Status:** LANDED (behavioural) · **Root cause owned:** i310's deferred chain —
the infection dynamics could not self-limit, for four coupled reasons in
`ImmuneState::tick_update`.

i310 sized the frailty population (≈2.5–3.5% of the calm village below the 0.25
derived-health gate) and named the driver: `infection_load` pinned at **1.0000**
over a resistance collapsed to **~0.000**. This iteration fixes the mechanism,
with the epidemic R0≈1 knife-edge (AGENTS §5 systemic debt) measured at every
step because it is the declared blast radius.

## Probe baseline (`i311_immune_clearance`, 12 seeds)

Pre-fix, the pinning is structural, not seed luck — every pinned agent is
anatomically identical (calm @20K, leg 1 sampled 18 pinned agents):

| R | C | rr | inflammation | fight_f | stress |
|---|---|---|---|---|---|
| **0.1500** | **0.1300** | 0.21–0.79 | 0.600 | **8.0e-5 – 1.15e-4** | 0.73–0.81 |

`fight_f = R·C·0.005·I·(0.7+0.6·rr)`. Two independent faults compose:

1. **Absorptive resistance.** Stress suppression is flat (`stress × 0.001`),
   while the nutrition and sleep boosts carry a `(1 − R)` factor. At the
   calibrated inputs suppression (~7.5e-4) outruns the boosts (~7e-4), so
   resistance decays to ~0 and never regains competence.
2. **Sub-resolution clearance (§5).** `Fixed::from_f64` rounds to nearest, so the
   clearance `0.975·I` raw units rounds to **zero** for any `I < 0.513`, and the
   whole fight chain truncated at every multiply before that. Combined with a
   constant (non-saturating) exposure intake, the equilibrium was
   `I* = E/(R·C·0.005) ≈ 2.6` — i.e. the 1.0 clamp.

The constant exposure is the third fault: a body already fully infected cannot be
infected further.

## Fix (all in `mindstrata-person/biology/immune.rs`)

1. `INNATE_IMMUNITY_FLOOR = 0.15` — competence is no longer absorbing at zero.
2. The clearance chain is computed in **f64 and quantized once** (§5 rule).
3. `quantize_rate` — a strictly-positive per-tick rate is never quantized to
   zero (rounds up to one step). The rate itself is sub-resolution here, so
   `from_f64`-once is insufficient; this is the §5 shadow-accumulator remedy
   without a new serialized field.
4. The stress exposure channel is **saturating** (`×(1 − infection_load)`), the
   same logistic form the resistance boosts use, giving `I* = E/(E+k) < 1` for
   any positive clearance.

Note the floor is applied to **clearance only**, not exposure: a `quantize_rate`
floor on exposure would turn the smooth `stress > 0.4` gate into a hard
one-step injection for marginally-stressed agents. The probe confirms the
exposure floor was inert here; the conservative form is in.

## Probe verdict — `IMMUNE_CLEARANCE_SELF_LIMITS`

| leg | pinned inf | R<0.02 | below gate | mean inf | mean sick | maxI | sick>0.5 |
|---|---|---|---|---|---|---|---|
| calm @2K | 0 | 0 | 0 | 0.000 | 0.000 | 0.023 | 0 |
| calm @20K | 0 | 0 | 0 | 0.020 | 0.017 | 0.384 | 0 |
| calm @50K | 0 | 0 | 0 | 0.032 | 0.027 | 0.502 | 0 |
| pestilence @4 320 | 0 | 0 | 0 | 0.003 | 0.002 | 0.123 | 0 |
| pestilence @20K | 0 | 0 | 0 | 0.015 | 0.013 | 0.346 | 0 |
| collapse @4 320 | 0 | 0 | 0 | 0.016 | 0.013 | 0.276 | 0 |
| collapse @20K | 0 | 0 | 0 | 0.028 | 0.023 | 0.398 | 0 |
| pestilence 5 @50K | 0 | 0 | 0 | 0.034 | 0.029 | 0.245 | 0 |

The **frozen-residual detector** (stress ≤ 0.4, i.e. no exposure, with
`0.001 < load < 0.5` — the band where the clearance used to truncate) reads
**0 in every leg** after `quantize_rate`; pre-fix it read 3 calm / 10 collapse
agents at 20K.

Epidemic axis (50K emergence leg, the knife-edge): infected-tick share
**0.958 → 0.905**, peak infection **1.0000 → 0.2454** (the 1.0 was the pin, not
the epidemic), and no agent pins in any scenario. The chronic-stress
equilibrium now lands at `I* ≈ 0.5` worst-case — a real, bounded low-grade state
in the band the original design comment intended ("self-limiting chronic
infection"), not the clamp.

## Drift re-anchors (documented, §4)

The fix raises derived health (sickness no longer pins at 0.84). Golden and
snapshots move on a **single metric** — `avg_health` — monotonically with
horizon, which is the frailty population accumulating over the run:

| artifact | old | new | Δ |
|---|---|---|---|
| metrics_500 snapshot | 0.829083 | 0.829925 | +0.0008 |
| metrics_2000 snapshot | 0.806408 | 0.808525 | +0.0021 |
| long_horizon_10K snapshot | 0.780517 | 0.801733 | +0.0212 |
| golden collapse/seed_42 (4320) | metric_hash `f3d878d9dffdc946` | `a409b81fc33088e9` | agent_count **12 preserved** |

No other metric in either snapshot moved. The crisis golden's `agent_count`
stays at 12 (mortality structure intact); `total_grain` 1.3104 → 2.128 reflects
healthier agents working/trading more under the collapse shock.

## Regression pins

`mindstrata-person::biology::immune::clearance_tests`:
`chronic_stress_infection_self_limits_below_saturation`,
`resistance_floor_is_non_absorptive_under_max_stress`,
`residual_infection_clears_at_the_resistance_floor`.

## Verification

`cargo fmt --all` clean · clippy **0** · `mindstrata-person` **116/116** ·
`mindstrata-sim` **277/277** · `mindstrata-tests` **309/0/1** ·
`scripts/gate --full` **GATE GREEN** with golden replay passing on the
re-anchored crisis baseline.
