# i340 — Population-scaled housing shipped: the quadratic store's governor is removed

**What landed.** `world_gen::houses_for_population(N) = max(8, ceil(N/4))` is now the
default house count, resolved at `populate`. `DEFAULT_HOUSE_COUNT = 8` is the floor, so:

- **N ≤ 32 keeps the historical 8-house village byte-identically** — same house count, same
  ring radius (4.0), same angle stride, same world-RNG draw order. This is *by construction*,
  not by luck, and it is why the calibrated N=12 windows and the goldens needed no re-anchor.
- **N > 32 spreads**: 12 houses at N=48, 24 at N=96, 36 at N=144, 48 at N=192, on a
  map-spanning ring.

`Simulation::set_house_count` (i339) remains as an explicit override; the snapshot path
derives the count from the restored world's `SiteKind::House` sites so a save/load boundary
cannot silently re-house a run.

## Verification

**GATE GREEN, zero re-anchors.** `mindstrata-sim` 289/289, `mindstrata-tests` 309/0/1,
golden 9/9 **byte-identical**, clippy 0, fmt clean. Four test sites do use N=48 (12 houses
now) and all remained green — the change is absorbable by the existing contracts rather than
merely unexercised.

## The store's governor is removed

`i340_housing_default` (seed 42, 32×32, 5K ticks), against i338's shipped-8-house baseline:

| N | houses | cells occupied | max co-located | touched R | partners/agent | near share | i338 baseline |
|---|--------|----------------|----------------|-----------|----------------|------------|----------------|
| 12 | 8 | 8 | 2 | 74 (56%) | 6.2 | 56.1% | **identical** |
| 32 | 8 | 10 | 4 | 408 (41%) | 12.8 | 41.1% | identical |
| 48 | 12 | 22 | 4 | **302 (13%)** | **6.3** | **10.5%** | 1 032 (46%), 21.5, 44.1% |
| 96 | 24 | 34 | 4 | **936 (10%)** | **9.8** | **8.4%** | 4 044 (44%), 42.1, 43.4% |
| 144 | 36 | 47 | 8 | 2 745 (13%) | 19.1 | 10.6% | — |

At N=96 the store keeps **936 contacted rows instead of 9 120** (8.9× smaller), max
co-location is 12 → 4, and the near-pair share is 43.4% → 8.4%. Max co-location is pinned at
the declared `House.capacity = 4` for every N up to 96 — the round-robin assignment finally
respects capacity as a side effect.

## Tick cost

`i329_local_exponent` (400 ticks, seed 42, 32×32), against the i334 record at identical
probe/params:

| N | i334 (8 houses) | i340 (scaled) | Δ |
|---|-----------------|---------------|---|
| 48 | 457.2 | 409.4 | **−10.5%** |
| 96 | 1 380.8 | 1 083.6 | **−21.5%** |
| 144 | 3 004.7 | 2 072.4 | **−31.0%** |
| 192 | 5 421.4 | 3 830.7 | **−29.3%** |

Local α 1.404 / 1.599 / 2.136 (was 1.609 / 1.988 / 2.079-ish at i331/i334). Per i337's
measurement discipline, whole-tick figures carry ±5–10% between-process noise — these deltas
are 2–3× that and monotone in N, matching the mechanism (the removed work is per-edge, so it
only grows with N). **No sim code other than the housing default changed; no re-anchor.**

## Charter envelope

`i332_envelope_expansion` (i295 charter method, 2 000 ticks, min-of-3):

```
N=96  1187.4 µs/tick  → 82% headroom vs the 6500 budget   (was 71% at i332, 72% at the i334 re-audit)
N=144 2598.5 µs/tick  → fits the N=96 charter budget
N=192 5049.9 µs/tick  → FITS the N=96 charter budget      (was 7700.8 — a 16% breach)
verdict=ENVELOPE_EXPANDED_2X                              (upgraded from ENVELOPE_EXPANDED_1_5X)
```

## The honest limit: at a fixed world size, spread re-saturates

The i339 measurement that justified this (touched-R α **0.898**) scaled the **world with N**.
At the charter's **fixed 32×32** the gain is real but finite, and the probe shows exactly
where it stops:

| N | houses | ring radius | max partners | touched-R α vs previous N |
|---|--------|-------------|--------------|---------------------------|
| 48 | 12 | 14 | 14 | — |
| 96 | 24 | 14 | 28 | 1.632 |
| 144 | 36 | 47 cells | **144 = N−1** | **2.654** |

At N=144 the 36 houses are packed ~2.4 tiles apart on the same radius-14 ring, so a
radius-5 neighbourhood again spans many houses and the contact graph re-saturates (the
best-connected agent has met **everyone**). The binding constraint at that point is the
**world's area**, not the housing count — and the same signal appears in the cost column
(α 2.136 for 144 → 192).

**Recommendation (recorded, not taken unilaterally):** the charter's "fixed 32×32 for every
N" envelope definition is now the limiter of the scale story. Defining the envelope at
**constant density** (world area growing with N — i339 measured α 0.898 there) is what keeps
this gain; that is a charter decision and a much larger behavioural change (every distance,
ecology and interaction site reads world geometry), so it is queued as its own item rather
than folded in here.

## Ledger effect

- **A7 (spatial spread does not scale with N) — CLOSED.** The governor i338 identified is
  removed, measured, and now the shipped default.
- The **contact-driven sparse store (i338's refutation) is now worth re-sizing**: with 8.9×
  fewer contacted rows at N=96 the "≤2× constant" argument was an artifact of the 8-house
  world. It is still *not* the top lever — the store's per-edge passes are now ~⅓ of the tick
  at the top of the envelope — but it is no longer refuted by construction; re-measure before
  deciding.
- **New item: world area must scale with N** for the gain to survive past N≈144 (charter).
- A8 (dead `Wander`/`Move`) is unchanged — spread changes *where* frozen villagers are.

## Reproduce

```
cargo run --release -p mindstrata-benches --example i340_housing_default
cargo run --release -p mindstrata-benches --example i329_local_exponent
cargo run --release -p mindstrata-benches --example i332_envelope_expansion
```
