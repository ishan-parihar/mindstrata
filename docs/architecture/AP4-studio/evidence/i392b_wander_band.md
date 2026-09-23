# i392b — the exploration driver's band: re-measured on a family, re-contracted, and finally pinned

**Status:** LANDED (contract change; **zero behaviour change** — the coefficient is untouched,
so both goldens are byte-identical and the suite is 313/0/1 plus one new pin).

**Root cause owned:** i392 row 2 was rejected partly because the *consumer's own calibration*
turned out to be unenforced. `WANDER_NOVELTY_COEF`'s doc comment asserted that i351's
**0.5–3%-share target band** was "verified in vivo by `i351_wander_bands`" — but that instrument
is a **probe**, not a test, so no gate ever re-measured it, and the share had left the band
without anyone noticing. This iteration measures it on a family, decides re-contract vs
re-calibrate from the data, and puts a test behind whatever claim results.

Probe: `crates/mindstrata-benches/examples/i392b_wander_band.rs` (legs A–D).

---

## 1. Leg A — the band is stale, on a family not two seeds

12-seed family (`1,2,3,5,7,11,17,21,33,42,44,99`), N=12, 32×32, 20 000 ticks — i351's own
configuration:

| seed | share% | wins | ≤noise | mean-loss | quiet-win% |
|---|---|---|---|---|---|
| 1 | 5.64 | 2 887 | 786 | 0.9636 | 21.06 |
| 2 | 4.34 | 2 096 | 347 | 0.7775 | 11.65 |
| 3 | 6.67 | 2 699 | 595 | 0.9901 | 32.74 |
| 5 | 3.44 | 1 921 | 210 | 1.3597 | 12.95 |
| 7 | 3.53 | 1 593 | 475 | 1.3160 | 24.20 |
| 11 | 1.74 | 943 | 685 | 0.7902 | 5.29 |
| 17 | 1.86 | 735 | 554 | 0.8315 | 10.06 |
| 21 | 3.44 | 1 714 | 361 | 0.9532 | 15.09 |
| 33 | 5.04 | 2 534 | 462 | 0.9497 | 19.52 |
| 42 | 5.44 | 2 398 | 514 | 1.3470 | 18.83 |
| 44 | 1.67 | 781 | 14 | 0.8416 | 4.54 |
| 99 | 8.81 | 5 542 | 3 741 | 0.9164 | 21.91 |

**share: min 1.67% · mean 4.30% · max 8.81%**, against the documented target band 0.5–3% and the
documented over-drive guard 8%. Eight of twelve seeds sit above the band's ceiling; seed 99
exceeds the guard as well. Liveness holds on 12/12.

**And the breach was already on the record.** `ENGINE_STATUS.md` §5 — the authoritative
status doc, re-measured at i384 with the same instrument — carries **Wander 4.4 / 5.4** in its
own action table. The band was stale when i384 measured it, and no reader noticed, because the
only thing that would have noticed is a test.

## 2. Leg B — the displacement check, which decides re-contract vs re-calibrate

The band's *stated* purpose is a Work-displacement guard ("a share > 8% would over-drive (Work
displacement)"). So the question is not "is the share above 3%" but "is anything being eaten".
Family means confound seed spread with displacement, so the check is run at **i384's exact
config** (seed 42, 500-tick warm-up, 20K, 32×32) and diffed against §5's recorded table:

| action | N=12 recorded | now | Δ | N=48 recorded | now | Δ |
|---|---|---|---|---|---|---|
| Work | 32.70 | **33.88** | **+1.18** | 30.10 | **29.34** | **−0.76** |
| Wander | 4.40 | 5.58 | +1.18 | 5.40 | 5.35 | −0.05 |
| Trade | 20.50 | 14.46 | −6.04 | 24.30 | 23.73 | −0.57 |
| Worship | 1.10 | 4.72 | +3.62 | 0.30 | 1.86 | +1.56 |
| Socialize | 4.90 | 5.87 | +0.97 | 5.10 | 5.18 | +0.08 |
| Rest | 17.90 | 18.49 | +0.59 | 16.90 | 16.84 | −0.06 |
| Eat | 10.40 | 8.27 | −2.13 | 10.60 | 10.69 | +0.09 |
| Drink | 6.60 | 7.08 | +0.48 | 6.40 | 6.42 | +0.02 |
| Move | 0.39 | 1.13 | +0.74 | 0.45 | 0.18 | −0.27 |
| Idle | 1.20 | 0.55 | −0.65 | 0.40 | 0.42 | +0.02 |

**Work does not move** (+1.18 pt at N=12, −0.76 pt at N=48). The N=12 churn is elsewhere
(Trade −6.04, Worship +3.62) and belongs to **i388's** relational urgency family
(`Belonging → Socialize|Worship`), which landed *after* i384 — not to exploration. With no
displacement, the band's rationale does not bite, so **re-calibrating the coefficient would be
a magnitude knob pulled for nothing (§4.4)** — the claim was stale, not the operating point.

## 3. Leg C/D — the statistic is stable, and the pin's own numbers

Horizon sensitivity (N=12): seed 7 gives 1.57% @2K → 3.53% @10K → 3.53% @20K; seed 42 gives
2.82% → 5.12% → 5.44%. Stable from 10K, so the pin runs at **10K** (≈6 s for the family,
release).

The pin's exact configuration — family `[1, 7, 11, 42, 44, 99]` (both measured extremes plus
the calibrated seed, so the bound is not asserted on a friendly subset, §4.1):

| seed | decisions | Wander (delta) | share% | quiet wins | exclusive | ≤noise/wins |
|---|---|---|---|---|---|---|
| 1 | 25 128 | 1 382 | 5.50 | 1 382 | yes | 28% |
| 7 | 21 314 | 752 | 3.53 | 752 | yes | 30% |
| 11 | 25 956 | 457 | 1.76 | 458 | yes | 76% |
| 42 | 21 838 | 1 118 | 5.12 | 1 119 | yes | 24% |
| 44 | 22 791 | 372 | 1.63 | 372 | yes | 3% |
| 99 | 29 151 | 2 682 | 9.20 | 2 694 | yes | 63% |

**max share 9.20% · liveness 6/6 · gate exclusivity 6/6.**

## 4. The new contract

Pinned by `exploration_driver_is_live_gated_and_bounded_across_the_seed_family`
(`statistical_emergence.rs`) — three invariants, each measured on all six seeds:

1. **liveness** — the driver reaches the deliberative layer on every seed (a driver that stops
   winning is the dead-producer class, §2.3);
2. **gate exclusivity** — ≥99% of wins lie inside the need-quietude window (measured 100%),
   i.e. exploration never outbids provisioning; the 1-point slack absorbs the two counters'
   ±1 instrumentation skew (visible above: seed 11 reads one more quiet win than arbitration);
3. **boundedness** — per-seed share ≤ **12%**, a guard *sized from the sweep* rather than
   picked: calibrated coefficient 2.0 maxes at **9.20%** and the over-drive coefficient i351
   flagged (**3.0**) maxes at **13.30%**, so 12% leaves the calibrated spread ≈30% headroom and
   still trips at the over-drive regime.

The **magnitude is recorded, not asserted** (§4.10): a band tight enough to discriminate the
share re-creates the unenforceable knife-edge pin, on a decision the census measures as
**noise-decided in 3–76% of wins** (mean 32%; 76% on seed 11, 63% on seed 99). That spread is
recorded as systemic debt (§4.5) — it says exploration volume on some seeds is substantially a
coin flip rather than a determined choice, which is a *design* question for the action layer,
not something a threshold can fix.

**The gate was proven to trip, both halves** (the i170 lesson — a check that cannot fail is not
a check):

| perturbation | result |
|---|---|
| coefficient **0** (raw 0) | FAILS the liveness leg: *"seed 1 produced 0 Wander selections in 10000 ticks"* |
| coefficient **5.0** (raw 50 000) | FAILS the bound: *"seed 42 spent 15.31% of decisions wandering (guard 12%)"* |
| coefficient **3.0** (over-drive) | passes at 13.30% max on the *probe* family, i.e. the guard's sensitivity sits between the calibrated and over-drive regimes |

## 5. Verification

`cargo fmt --all` clean, clippy **0 warnings**, sim **310/310**, integration
**313 passed / 0 failed / 1 ignored** + the new pin, both goldens **byte-identical** (the
coefficient is unchanged, so this iteration is behaviour-neutral by construction — the sim
crate's whole diff is a doc comment), `gate --full` GREEN, `doc_index.py` 0 ghosts.

## 6. Ledger consequences

- **i392b: DONE.** The exploration driver's contract is enforced by a test for the first time.
- **i392 rows 3–5 are unblocked** (the precondition is discharged): `sensory_acuity`,
  `aggression_threshold`, `chronic_pain_risk` may now be measured against their consumers —
  each still needing both gates from §4.15 (the shadowed constant at the gene's midpoint, and a
  proportional response).
- **New systemic debt:** Wander's win margin is noise-decided in 3–76% of wins across the
  family. The driver's *volume* is defensible; its *determinism* at the margin is not, and the
  fix is the action layer's quiet-window bar, not another threshold.
- **Retired claim:** "Wander share in the 0.5–3% band at 20K" (i351's reading) is gone from the
  code comment, replaced by the measured family and the three invariants.
