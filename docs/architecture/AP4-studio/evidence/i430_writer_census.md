# i430 — the four-writer affection census (the i429 repair's target)

**Iteration:** i430 · **Date:** 2026-09-27 · **Status:** MEASURED — repair target identified and sized; no code change
**Probe:** `crates/mindstrata-benches/examples/i430_writer_census.rs` (daily-boundary diff of every ordered dyadic row, seed 42 calm 16×16 N=12, 139 days ≈ 20K ticks, plus end-state splits for seeds {42, 7, 11})
**Chain:** i429 measured the store's trajectory (ratchet-to-ceiling); this census answers the follow-up the repair needs — **which writer owns the mass**, and whether any writer the code review missed exists.

## 1. The four-writer mass table (139 days, seed 42)

Attribution: daily Δ per ordered row; |Δ| ≥ 0.05 is unreachable for the per-act
law's single gain (max measured +0.0387/act) and is cross-checked against
`RitualPerformed` participant pairs and `MarriageFormed` spouse pairs; rows
with interactions that day carry the per-act class.

| writer | total mass | events / row-days | per-event magnitude |
|---|---|---|---|
| **per-act (`record_positive`)** | **+67.59** | 483 chunky-classified row-days (mean 42 acts/day) + the slow remainder (2 498 acts) | mean net **+0.0006/act** (gross ≈ +0.0023; code law 0.015×vol×mag) |
| marriage (`+0.3` + in-law rows) | +3.26 | 5 events / 10 row-days | +0.33/row-day |
| household ritual (`bonding×0.2`) | +0.47 | 8 events (mean bonding 0.180) / 2 row-days | +0.036/row-day |
| decay (all legs) | **−8.95** | 14 537 row-days | mean −0.00062 (cold −0.0001; dirty-day −0.0145) |

**The per-act writer owns 92% of positive mass (+67.6 of +73.8).** The advisory
hypothesis that the chunky household/marriage writers dominate the saturation
is **measured-refuted**: together they are 4.8%. The saturation is not a few
big events — it is the per-act law compounded by **contact frequency**.

## 2. The hidden-writer check: none

All 483 chunky row-days (|Δ| ≥ 0.05/day) resolved against interaction-count
deltas: **dc = 0 on exactly 0 of 483** (mean dc = 42.18, max 195). The
"chunky" class is the per-act writer on high-frequency pairs, not an
uninstrumented writer. 35 of the 483 rows are day-1 founding-day rows.

## 3. The real driver: interaction frequency (the taper's sizing input)

Measured interactions/day over contacted row-days: **p10 = 4, p25 = 20,
p50 = 41, p75 = 62, p90 = 90, max = 245.** The median contacted pair interacts
~41×/day — against a dirty-day decay of 0.0145, that is a gross gain of
~+0.10/day, which is why p50 affection reaches 0.945 by day 10 (i429).

## 4. Repair sizing (headroom-tapered gain, candidate 1)

With the taper `affection += 0.015×vol×mag×(1 − affection)`, a contacted pair's
equilibrium is `a* = 1 − D/(g_eff·k)` (D = dirty-day decay 0.0144, g_eff ≈
0.0023 measured gross, k = acts/day). Against the measured frequency
distribution this yields a **populated spread** instead of the point mass:

- k ≈ 4 (p10): a* < 0.5 — low-frequency pairs settle low (the bond costs more
  to hold than the contact pays);
- k ≈ 20 (p25): a* ≈ 0.69; k ≈ 41 (p50): a* ≈ 0.85; k ≈ 90 (p90): a* ≈ 0.93.

The kind schedule's 0.70 bar would then fire for roughly the upper half of
the contact-frequency distribution — **live and discriminating** for the first
time since the store's equilibria collapsed. Known cost, stated honestly: the
taper alone cannot give a 1-act/day pair a mid-band bond (a* < 0 below
k ≈ 6); if the corpus shows that is too harsh, the same commit must re-size
the dirty-day decay leg (i429 candidate 2) — the two levers are separable in
sign but must be sized together against the same corpus.

## 5. Per-seed end-state splits (the advisory's columns)

| seed / horizon | n | p50 | <0.70 | band [0.5,0.95) | ≥0.95 |
|---|---|---|---|---|---|
| 42 @ 20K | 104 | 0.994 | 1.0% | 1.9% | 98.1% |
| 7 @ t=1440 | 87 | 0.744 | 46.0% | 43.7% | 39.1% |
| 7 @ t=10080 | 102 | 0.998 | 7.8% | 8.8% | 89.2% |
| 7 @ 20K | 108 | 0.998 | 3.7% | 3.7% | 94.4% |
| 11 @ t=1440 | 80 | 0.642 | 57.5% | 58.8% | 25.0% |
| 11 @ t=10080 | 95 | 0.998 | 11.6% | 11.6% | 84.2% |
| 11 @ 20K | 98 | 0.998 | 4.1% | 3.1% | 94.9% |

All three seeds land in the same place by 20K: the store is **bimodal**
(stranger tail + saturated mass) with the discriminating band drained to
~2–4% — consistent across seeds, so the verdict is a property, not a sample
(§4.11).

## 6. Verdict and the next commit

The repair target is **`record_positive`'s affection term** — the headroom
taper is the organic form (a bond cannot deepen at a constant rate forever),
the equilibrium it creates is proportional to contact frequency (the state
variable that already measures what the ratchet guesses at, §4.6's purest
instance), and its blast radius must be sized by the i425–i428 corpus sweep:
both goldens, the 7 snapshots, the revolution family {99, 42, 11} with the
handover invariant, panic liveness, and the 7 pins re-contracted at i428.
Household/marriage chunky writers are recorded as **not the repair target**
(4.8% of mass); re-open only if the post-taper corpus still pins a mass at
the ceiling on ritual/marriage pairs specifically.
