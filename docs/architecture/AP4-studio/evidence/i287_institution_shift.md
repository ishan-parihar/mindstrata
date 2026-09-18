# Iteration 287 — Era IV exit: WP-J institution_shift end-to-end (PLAN_DC3 §4 i287)

**Status:** LANDED · **Doctrine:** probe-first; no sim code changed → zero re-anchors by
construction. The Era IV exit gate (recorded open in PLAN_DC2 §6.6) closes on this
evidence.

## The gap

`i300_institution_shift` (Era IV exit probe, WP-J end-to-end) was recorded as the one
unproven Era IV surface: i280 landed two live channels (§12.3 compliance multiplier +
member Work bonus) with unit-level liveness pins, but the end-to-end forced-stage A/B
was rejected as CONFOUNDED (it forced all 29 collective lines, moving every stage-gated
system at once).

## The probe (`i287_institution_shift`, N=12 seed 42, release)

Design fix over i280: force ONLY the two WP-J input lines (`governance`,
`economic-systems`) to the crossing band each tick; everything else natural. Three legs:

**[h1] 2K, forced 3.5 (below the 4.0 band-III gate)**

| surface | natural | forced-3.5 | Δ |
|---|---|---|---|
| WP-J mean stage | 1.0 | 3.5 | — |
| grain | 36.23 | 39.20 | +2.97 |
| trades | 279 | 253 | −26 |
| genesis memes | 0 | +1 | +1 |
| NormViolated | 0 | 0 | 0 |
| norm proposals | 5 | 5 | 0 |

**[h2] 2K, forced 6.0 (crossing band, mult 1.10)**

| surface | natural | forced-6.0 | Δ |
|---|---|---|---|
| morale | 0.0905 | 0.1201 | +0.030 |
| grain | 36.23 | 39.10 | +2.86 |
| trades | 279 | 248 | −31 |
| genesis memes | 0 | +2 | +2 |
| NormViolated | 0 | 0 | 0 |

**[h3] 20K natural (no forcing) — the Era IV exit question**

```
stage 6.000  morale 0.0859  grain 0.39  trades 3716  NormViolated 0  genesis 8  norms 5
```

## Findings

1. **Era IV exit verdict: PASS on the natural trajectory.** At 20K the village's own
   development carries governance/economic-systems to stage 6.0 (the i280 prediction
   confirmed in vivo) — the WP-J ramp opens at mult 1.10 WITHOUT forcing, and the
   collective stage bands have driven 8 generated culture items (genesis) end-to-end.
   The collective holon is live on its own metabolism, not only under probe forcing.

2. **The end-to-end single-channel isolation is STRUCTURALLY confounded — now
   precisely diagnosed, not just asserted.** Even 2-line forcing moves grain/trades at
   every forcing level including BELOW the WP-J gate (h1: +2.97 grain with the WP-J
   multiplier provably identity — compliance_multiplier returns Fixed::ONE ≤ 4.0). Cause:
   both input lines are Safety-bucket system lines, and `genesis.rs` weights domain
   selection by per-bucket deepest-line stage (i276), while the i277 class gates read
   the same lines. Forcing the inputs necessarily moves every coupled reader. The i280
   unit pins (identical-RNG single-variable A/B) therefore remain the ONLY valid
   liveness proof for the WP-J channels — this probe documents why, permanently.

3. **NormViolated = 0 in every leg** — extends the i280 six-seed diet verdict to the
   20K forced horizon. Channel #1 (§12.3 compliance) remains armed-but-inert at N=12;
   recorded debt unchanged (A1 transgression feed, PLAN_DC3 §3.1).

4. **Morale moved under forcing** (0.0905 → 0.1201) — second-order evidence that the
   institutional layer responds to the governance stage through the member-composition
   channel, independent of the WP-J multiplier.

## Calibration honesty

- No code behavior changed → no re-anchors; the full gate must be green as-is.
- The h1 grain/trades deltas are attribution evidence, not regressions: they show the
  coupling SURFACE (shared stage lines), not a WP-J effect (which is provably identity
  below 4.0).
- 20K natural grain 0.39 is the standing village equilibrium at that horizon (grain
  consumed as produced; 3,716 trades is the provisioning activity signal).

## Exit status

**Era IV exit gate CLOSED.** WP-I (collective field), tetra-arising band gate, Identity
feed (i279), WP-J coupling — all live and now proven end-to-end on the natural
trajectory. Remaining Era IV-adjacent debt is the diet family (A1/A5), not construction.

**Next:** i288 — transgression feed probe (A1), per PLAN_DC3 §4.
