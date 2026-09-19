# Iteration 312 — the health-critical guard is dormant (post-i311 re-audit)

**Status:** LANDED (measurement iteration, **no behaviour change**) · **Root
cause owned:** AGENTS §2.5 — every fix shifts downstream equilibria; i311 needs
a re-audit of what it invalidated. §4.3 — a dead producer is a bug even when
tests pass.

i255 introduced the health-critical reflex ("a body at its limits does not exert
itself"); i309 rebuilt it as a non-trapping veto (`health < 0.25` downgrades
`Work`/`Wander` to `Rest` after every selection path). i311 fixed the immune
pin that created the frailty population which was the *only* thing reaching that
gate. This iteration asks the follow-up question.

## The guard is dead state

Probe `i312_health_reflex_liveness`, every tick sampled (not just the horizon),
12 seeds:

| context | agent-ticks | below 0.25 | ever crossed | p0.1 | p1 | p5 | p50 |
|---|---|---|---|---|---|---|---|
| pestilence @4 320 | 626 582 | **0** | **0** | 0.548 | 0.620 | 0.700 | 0.819 |
| pestilence @20K | 2 951 859 | **0** | **0** | 0.489 | 0.509 | 0.610 | 0.814 |
| collapse @4 320 | 627 222 | **0** | **0** | 0.521 | 0.582 | 0.664 | 0.810 |
| collapse @20K | 2 927 264 | **0** | **0** | 0.449 | 0.467 | 0.578 | 0.806 |
| drought @4 320 | 627 622 | **0** | **0** | 0.433 | 0.549 | 0.644 | 0.814 |
| drought @20K | 3 024 609 | **0** | **0** | 0.396 | 0.482 | 0.525 | 0.810 |
| calm @20K | 2 996 485 | **0** | **0** | 0.422 | 0.487 | 0.547 | 0.809 |
| calm @50K | 8 112 745 | **0** | **0** | 0.415 | 0.468 | 0.529 | 0.809 |

**0 of ~22M agent-ticks** fall below the gate in any scenario; no agent ever
crosses it. The veto cannot fire.

## Why — the penalty decomposition

The worst-health agent in each context decomposes identically (formula in
`EmbodiedState::derived_health`, `biology/mod.rs:225`):

| context | health | base×immune | − stress | − chronic | − pain | − sickness | − shock |
|---|---|---|---|---|---|---|---|
| drought @20K | 0.394 | 0.740 | 0.158 | 0.135 | **0.000** | 0.053 | **0.000** |
| collapse @20K | 0.448 | 0.782 | 0.160 | 0.135 | **0.000** | 0.041 | **0.000** |
| calm @50K | 0.411 | 0.745 | 0.158 | 0.135 | **0.000** | 0.041 | **0.000** |

The floor is `base×immune (≈0.74–0.81) − stress (≈0.16) − chronic (≈0.135)`.
Two of the five penalty channels contribute **exactly 0.0000**:

- **pain** is gated on `nervous.pain.effective_pain()`, which is injury-driven,
  and injury stays ~0 in calibrated windows;
- **shock** (`cardiovascular.shock_risk`) never leaves 0.

The remaining floor is stress + chronic saturating at a *fixed* ~0.29 total —
the Iter-172 "stress pinned near 1.0" residual. So the health axis expresses
**chronic stress only**; it has no reachable acute-crisis tail.

## No threshold can restore liveness

The distributions overlap almost completely across contexts: p1 is 0.47–0.62 in
*calm* and in *drought*. Any absolute gate low enough to spare calm (≤ 0.40)
also spares every crisis; any gate high enough to fire in crisis (≥ 0.45) fires
for the calm tail as well. Confirmed by the table — there is no separating band.

## Disposition (§4.4 / §4.5)

This is a **re-contract**, not a re-pin: the 0.25 gate was implicitly
calibrated against the buggy sickness pin (`sickness 0.84` alone supplied a
−0.126 penalty, and with the old R≈0 anatomy it drove health to 0.21). The
immune fix legitimately invalidated it. The correct guard invariant — "an agent
whose derived health is in the lowest reachable band does not exert" — cannot be
guarded by an absolute constant today.

The veto is **kept** (it is a correct, non-trapping safety guard; removing it
would be churn) and its dormancy is documented at the call site. Restoring a
reachable crisis band is queued as its own behavioural iteration: it needs
**live pain and shock channels** (injury → pain, cardiovascular shock), which is
a mechanism gap, not a threshold tweak — exactly the kind of fix this doctrine
refuses to paper over with a re-pin.

**Resolution (i313 + i314):** i313 wired the pain/shock/blood-loss channels
(the injury field had no producer), and i314 keyed the veto on the now-live
pain signal at a measured 0.9 band — see `i313_injury_channel.md` and
`i314_pain_veto.md`. The `health < 0.25` clause remains as a dormant safety net.

**Zero drift:** no behaviour change, golden byte-identical, no snapshot
re-anchors, no new re-pins.
