# i392 row 4 — `sensory_acuity` measured against the salience bias: REJECTED

**Status:** MEASURED AND REJECTED (`i394_sensory_acuity` legs A–D; wiring reverted;
tree byte-identical to `614ab2a`) · **Scope:** one dead gene's second candidate consumer.

## Why the census-named consumer was not the one probed

i391's follow-up table named "the i334 perception radius" as the consumer. That
consumer is **wrong for a heritable trait**: `DEFAULT_PERCEPTION_RADIUS` is an
**integer** (tiles), so an acuity multiplier over it is a 3-bin staircase — exactly
the §4.15 gate-2 cliff. The class-consistent candidate was instead the attention
system's hardcoded `salience_bias = 0.5` — written only by `Default`, multiplied into
every salience score, and numerically **the mean of the gene's own draw**
(`U(0.2, 0.8)` → mean 0.5). Wiring: `AttentionState::with_acuity(gene)` at the three
construction sites (founders + both birth paths), no RNG stream touched.

## The measurements that rejected it

**A — the anchor was exact (§4.6 at machine precision).** With every founder's gene
pinned to 0.5, the probe reproduces both stored goldens byte for byte:
`riverford_minor` MATCH/MATCH, `collapse` MATCH/MATCH. (Probe-harness note: collapse
diverged until the probe built the world with `Simulation::from_scenario` — the
scenario pre-sets `drought_until` from its drought shock, which a raw `SimConfig`
silently drops. The stored golden was always scenario-based; the first divergent
reading was the probe's bug, not the engine's.)

**B — the natural-gene world diverges immediately** (first divergence tick 18,
agent "Hans", gene 0.7459), and population-mean salience bias is 0.39 at N=12 —
the small-N founder-variance effect (§5 H5) puts the mean multiplier far from 1.0.
Memory occupancy at 2K: control 674 traces vs natural 229.

**C — the response is a threshold lottery, not a gradient (gate 2 fails).**

| pinned gene | traces @2K | mean/agent |
|---|---|---|
| 0.2 | 79 | 6.58 |
| 0.35 | 337 | 28.08 |
| 0.5 | 674 | 56.17 |
| 0.65 | 1 126 | 86.62 |
| 0.8 | 1 132 | 87.08 |

The encode threshold (`salience ≥ 0.2`) sits **inside the gene's draw range**, and
fresh own-help interactions compute salience ≈ **0.216** — marginally above it — so
the multiplier acts as an encode/no-encode lottery: a **13×** spread in memory
occupancy across the gene range, superlinear below the mean and saturating above it.
A band edge, not a coefficient, consumes the trait. This is the same shape failure
as row 2 (elasticity ≈4), found by a different signature: band-edge amplification
instead of smooth amplification.

**D — the revolution liveness family collapsed to 1/3 (the row-2 kill criterion).**
Pestilence @70K, meme mutation isolated, seeds {5, 42, 12345} (the i388 family
config): **0 / 0 / 2 revolutions** vs the pinned ≥2-of-3. Mechanism: suppressed
encoding shifts the first deca-tick `rehearse_random` draw on the **shared Behavior
stream** (§5 RNG discipline), re-timing every downstream draw. A producer going dark
is not re-pinnable (§2.3).

## Disposition

- The wiring was **reverted, not re-anchored** (the row-2 precedent). The reverted
  tree reads MATCH/MATCH on both goldens and no divergence in leg B — the revert is
  verified, not assumed.
- `sensory_acuity` stays **deliberately inert** (like `novelty_seeking`): the trait
  is real heritable variation whose consumer must be built before it can be wired.
- Recorded qualification for any future candidate: the consumer must (a) be
  continuous, (b) have no acceptance/encode threshold inside the gene's draw range,
  and (c) not share an RNG stream with a pinned liveness family's draws — or the
  family must be re-swept in the same iteration.

## Verification of the revert

fmt clean · clippy 0 warnings · sim 310/310 · integration 313/0/1 · both goldens
byte-identical · gate GREEN.
