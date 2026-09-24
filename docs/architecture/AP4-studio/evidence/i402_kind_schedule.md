# i402 — The interaction-kind schedule's v1 input, sized (probe + verdict)

**Status:** DONE as a **measured deferral** (2026-09-25). The kind-schedule read
move (`choose_interaction`'s v1 trust/affection pair → the dyadic store) is
**folded into i403's single commit**, where the writer deletion changes the
affection surface it reads. The probe refuted the standalone read-move's
premise: the schedule's input is saturated at the ceiling **on both stores**,
so the read choice moves 0–3.5% of branch selections and admits no stable
threshold re-derivation.

Probe: `crates/mindstrata-benches/examples/i402_kind_schedule.rs`
(legs selectable: `d` latch, `a` sweep; default `abcd`). All numbers below are
the probe's own output, release mode, 20K-tick villages, window 18K–20K.

## Legs

**Z — pass liveness.** 12,863 / 44,098 / 51,539 `InteractionOccurred` events
per 2K-tick window (0.46–0.54/agent/tick) — the schedule decides every one.

**A — the ratchet, signed** (mean v1 − v2 on the exact pairs the schedule read):

| village | trust signed | affection signed |
|---|---|---|
| N=12 seed 42 | +0.0006 | **+0.0021** |
| N=48 seed 42 | +0.0117 | **+0.0185** |
| N=48 seed 7 | +0.0233 | **+0.0330** |

i401's ratchet hypothesis is **confirmed in sign** — v1 affection sits above v2
(the v1 write only ever pushed up; the daily sync is trust-only) — and the
magnitude is 0.002–0.033.

**B — the branch boundaries** (what the migration would actually move):

| village | high-aff (>0.70) v1→v2 | flips | low-trust (<0.20) v1→v2 | flips |
|---|---|---|---|---|
| N=12 s42 | 99.1% → 99.1% | **0.0%** | 0.0% → 0.0% | 0.0% |
| N=48 s42 | 98.5% → 96.4% | **2.9%** | 0.6% → 0.6% | 0.0% |
| N=48 s7 | 97.8% → 94.4% | **3.5%** | 0.8% → 1.9% | 1.1% |

**C — the kind mix the schedule produced** (window share):

| village | Help | Comfort | Talk | Insult | Trade | Threaten | Gossip | Teach |
|---|---|---|---|---|---|---|---|---|
| N=12 s42 | 71.8 | 18.3 | 6.8 | 0.8 | 1.5 | 0.3 | 0.3 | 0.1 |
| N=48 s42 | 75.2 | 18.7 | 4.2 | 0.8 | 0.5 | 0.3 | 0.3 | 0.1 |
| N=48 s7 | 73.3 | 18.1 | 6.4 | 0.7 | 0.6 | 0.3 | 0.5 | 0.2 |

90%+ of interactions land in the high-affection branch — Help/Comfort dominate;
the openness branch (Gossip/Teach) and the default branch (Trade) are 0.3–1.5%.

**D — the latch** (dyadic store, 1000-tick samples, transition shares over the
full 20K horizon):

| village | from-above stays | from-below rises | below-drift/sample |
|---|---|---|---|
| N=12 s42 | 100.00% | 5.02% | +0.0242 |
| N=48 s42 | 99.93% | 4.33% | +0.0187 |

Above-gate affection **never comes back**; below-gate pairs climb through the
gate at 4–5% per kyr against a 0.0002/tick decay. The latch lives in the
**write rate**, not the read.

## Verdict — the gate is the defect; the read move + its redesign land inside i403

1. **The surface is saturated on both stores.** Affection on contacted pairs:
   v1 p50 = 1.0000, p90 = 1.0000; v2 p50 = 0.9986–0.9997, p90 = 1.0000. The
   ratchet parked v1 at the ceiling, and the dyadic store's own write/decay
   balance parks v2 there too. The schedule is **already reading a saturated,
   ratcheted signal** — i401's hypothesis, now measured with signed means — and
   the branch it selects (Help/Comfort, 90%+) is the healthy one.
2. **The 0.70 gate is the defect, and it is §4.10's exact class**: an absolute
   threshold on a self-driven aggregate. `affection > 0.70` opens for 98–99% of
   contacted pairs on BOTH stores — as vacuous as the deleted `trust < 0.5`
   faction arm (i379/i382). And leg C's "re-derivation point" (0.4460/0.3415)
   is **not a re-derivation**: quantile-matching a 98.5% share of a point mass
   just re-codes the saturation — 98.5% of pairs would STILL take the
   comfort/help branch. Those numbers are explicitly NOT adopted.
3. **No standalone read move.** Moving the read alone would re-anchor goldens
   twice (0–3.5% branch flips now, again after the writer deletion) while
   leaving the non-discriminating gate in place. **Sequence, decided here:**
   i403 first (v1 writer deletion + `bonding_rate`/`conflict_escalation_rate`
   re-host on the dyadic magnitude — the surface change, leg D says the latch
   is write-side), THEN re-run this probe against the post-deletion surface,
   THEN the read move + the gate redesign **in the same commit**, re-derived
   once against the whole new affection surface (the i400 rule: never perturb
   a band edge three separate times).
4. **The gate redesign's leading candidate is the relative/cohort form** per
   §4.9/§4.10 and the i381 (`EMOTION_SHOCK_RATIO`) / i383 (anger arms)
   precedents — e.g. affection in the **top quartile of the current
   population's contacted-pair affection distribution** (N-sized, no absolute
   constant) — decided by the post-deletion measurement, not by the saturated
   numbers above. If the operator prefers a different form (anomaly multiples,
   dyadic-stage-driven), that decision lands with i403's probe, not silently.
   Three design constraints, recorded so the i403 probe answers them before
   anything is wired: **(a) horizon-stability** — leg D shows below-gate pairs
   cross 0.7 at 4–5%/kyr, so the high-affection share is strongly
   horizon-dependent and a gate quantile-matched at 20K will under-fire at
   50–100K; measure the share at a second horizon (with v1's for comparison)
   and prefer a rate/relative rule over a single-horizon quantile if the gap
   grows. **(b) the denominator** — leg C's 98–99% share was measured over
   *event-linked* pairs (they already interacted in the window), but
   `choose_interaction` runs at candidate time, including first-contact pairs
   whose v2 affection is still genesis-level (~U(0.2,0.6), verified dense at
   genesis — population.rs pushes every ordered pair, so the v2 seed copy is
   correct); a cohort quantile over all dyadic pairs is a different population
   than leg C matched, so the share it preserves will not be the measured 98%.
   Decide all-pairs vs contacted-so-far vs per-source-agent from the
   post-deletion probe. **(c) per-tick cost** — a population quantile every
   interaction decision must be sized on the N=96 budget rung (i423 records
   the gate is structurally blind at 192; do not add an O(N²)-per-tick scan).
5. **The kind mix's loss of discrimination is the measured open question**
   this leaves behind: Gossip/Teach/Trade are 0.3–1.5% of the mix because
   nothing ever comes back below the gate. Whether that mix SHOULD
   discriminate again is an operator taste question (depth: a village where
   everyone loves everyone acts the same); the lever either way is the
   write-rate-vs-decay balance on the dyadic store, sized in i403.

## Probe-harness note (recorded for reuse)

The latch leg originally called `sim.run(t + step)` per sample — but
`Simulation::run(n)` executes n ticks (`for _ in 0..n`), it does not advance
*to* tick n, so the leg ran Σ(t+step) = 10.5× the intended horizon and read a
deep-future world. Fixed to `sim.run(step)` per sample; the shipped leg-D
numbers above are from the corrected stepping. The A/B/C legs were unaffected
(one `run(horizon)` from zero).

## Concurrent-session provenance

The probe was authored by a concurrent session (2026-09-24 22:53, restructured
23:01, its run died at 66 bytes); after ~80 min of silence this session took
ownership, fixed the gate-blocking format/lint issues mechanically (cargo fmt,
`sort_by_key`, an unused import — noted per AGENTS §7 etiquette), fixed the
latch stepping bug above, and ran the sweep.
