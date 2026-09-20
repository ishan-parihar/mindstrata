# Iteration 319 — the blood-loss threshold was unreachable (i318 §2.5 re-audit)

**Status:** LANDED (behavioural) · **Found by:** the mandatory post-i318 re-audit
(AGENTS §2.5). **Root cause owned:** a producer that i313 had just revived
(`cardiovascular.blood_volume` → `shock_risk` → the derived-health `shock_penalty`)
went dead again two iterations later, silently.

## The finding

Running the i313 liveness probe at HEAD showed `max shock 0.00000` and
`min blood-vol 1.0000` in the calm leg — exactly the pre-i313 dead-state numbers —
even though i313's own evidence recorded `shock 0.845 / blood-vol 0.300` there.

Isolation: checking out **only** `crates/mindstrata-sim/src/sim/pass_action.rs` at
`8c600f6` (i313) restores `shock 0.84490 / blood-vol 0.3000 / max injury 0.51400`.
So the cause is **i314's exertion veto**, not i318 (which was isolated separately and
reproduces the same values at `82cd81c`). An injured agent is vetoed from exertion —
and therefore from further conflict — so it no longer stacks the second and third
wounds the bleed rule needed. i314 measured its own veto firing; it never measured the
downstream cardiovascular channel it had just starved.

## Probe `i319_wound_reachability` — the threshold is unreachable by construction

12-seed family, 20K and 50K ticks, calm config:

| metric | 20K | 50K |
|---|---|---|
| `Violence` conflicts | 175 | 237 |
| **`Combat` conflicts** | **0** | **0** |
| max single wound | 0.1656 | 0.1656 |
| max stacked injury | 0.2950 | 0.2950 |
| agent-ticks with injury **> 0.30** | **0** (0.0000%) | **0** (0.0000%) |
| agent-ticks with injury > 0.20 | 1 295 (0.0436%) | 1 496 (0.0186%) |
| agent-ticks with injury > 0.15 | 3 735 (0.1257%) | 4 720 (0.0585%) |
| agent-ticks with injury > 0.10 | 20 816 (0.7008%) | 27 954 (0.3466%) |

`Combat` (severity 0.5) fires **zero** times, so the reachable wound range is single
`Violence` wounds `0.12 + aggression × 0.1 ∈ [0.12, 0.166]` plus two-wound stacks to
0.295. The `injury_severity > 0.3` bleed rule is crossed by **0 agent-ticks in
12 seeds × 20K/50K** — the producer was unreachable, and the derived-health `shock`
penalty with it. This is the AGENTS §4.3 dead-producer class, third instance
(i244 exposure, i311 clearance, i313 injury are the earlier ones).

## The fix (two parts, the second caught by the pin)

1. **`BLOOD_LOSS_INJURY_THRESHOLD = 0.15`** (was a bare `0.3`), named and documented,
   chosen on the probe's threshold table: `0.15` is just below the measured maximum
   single wound (0.1656), so a *serious single beating* bleeds while the common
   0.12–0.13 scuffle stays dry; `0.20` would need a two-wound stack; `0.30` is
   unreachable.
2. **Band alignment.** The recovery gate was `injury < 0.2` against a bleed threshold
   of `0.3` — a dry gap. Reconciling the threshold to 0.15 made the bands **overlap**
   in `(0.15, 0.2)`, where recovery (`0.5 × 0.002 = 0.001/tick`) exceeds the loss
   (`0.16 × 0.005 = 0.0008/tick`) and a serious wound still nets zero. `blood_loss_
   threshold_admits_a_serious_wound_but_not_a_scuffle` failed on exactly this, so the
   recovery gate now uses `BLOOD_LOSS_INJURY_THRESHOLD` too — a body at or above it
   only bleeds, and recovers once the wound drops below.

## Probe verdict — `BLOOD_LOSS_CHANNEL_REACHABLE`

`i313_injury_channel` at HEAD:

| context | max injury | max shock | min blood-vol |
|---|---|---|---|
| pestilence @4 320 | 0.3186 | 0.0000 | 0.7678 (was 1.0000) |
| collapse @4 320 | 0.3186 | 0.0000 | 0.6122 (was 1.0000) |
| calm family @20K | 0.2950 | **0.6651** (was 0.0000) | **0.3770** (was 1.0000) |
| calm family @50K | 0.2950 | **0.6651** | **0.3770** |

Bleeding now fires in every scenario, and shock is reachable in the calm family. The
calm-leg values are *more moderate* than i313's pre-veto 0.845/0.300, which is the
correct sign: the veto removed the pathological stacking that produced the extreme,
and the reconciled threshold restores a reachable-but-serious bleeding channel.
(`shock` stays 0 in the crisis legs because their blood volume bottoms at 0.61–0.77,
above the `0.6` shock gate — recorded, not smoothed; the producer is live.)

## Drift — zero re-anchors

`shock_penalty` only fires below `blood_volume 0.6`, and the pinned windows do not
reach it, so **both goldens and every snapshot pass unchanged**: sim **280/280**,
person **120/120**, tests **309/0/1**, clippy **0**, `scripts/gate --full` **GATE
GREEN**.

## Pins

- `cardiovascular::tests::blood_loss_threshold_admits_a_serious_wound_but_not_a_scuffle`
  — threshold below the measured max single wound; a 0.16 wound bleeds; a 0.12 scuffle
  does not (this pin found the band overlap).
- `sim::tests::conflict::violence_records_injury_on_the_substrate` extended — the
  20K seed-42 wound must also reach the cardiovascular channel (`min blood_volume < 1`).

**Probe:** `cargo run --release -p mindstrata-benches --example i319_wound_reachability`.
