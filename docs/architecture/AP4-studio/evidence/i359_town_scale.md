# Iteration 359 — the scale envelope reaches N=256; the world is still one settlement

**Status:** MEASUREMENT + DECISION · **Item owned:** the plan's C-item — "multi-settlement
orchestration at N≥192 on the density world law, with the LOD tier carrying background
agents — this is the actual 'can we reach a town/city' experiment".

## Leg A — cost: the envelope is now 2.7× the DC-3 Phase-1 target

Same method as i295/i332 (release, seed 42, `new+populate+run`, 2 000 ticks, min-of-3),
on the i344 **density world law** (`side(N) = max(16, ceil(sqrt(21.333·N)))`), against
the fixed N=96 charter budget of **6 500 µs/tick**:

| N | side | µs/tick | vs 6 500 |
|---|---|---|---|
| 48 | 32 | 405.3 | −6 094.7 |
| 96 | 46 | 987.3 | −5 512.7 (**85% headroom**; was 27% at i295, 71% at i332) |
| 144 | 56 | 1 856.0 | −4 644.0 |
| **192** | **64** | **3 239.4** | **−3 260.6 (fits with 50% headroom**; i332 measured 7 700.8, a **16% breach**) |
| **256** | **74** | **5 805.2** | **−694.8 (fits the N=96 budget)** |

`verdict = ENVELOPE_EXPANDED_2_7X`. The charter's N=96 per-tick budget now covers
**N=256** — 2.7× the population — and the N=192 row alone improved 2.4× since i332.
The gain is the housing spread (i340/i345, which removed 90% of the walkable rows), the
i352 marriage-scan removal, and the i356 Idle re-pacing, all landing on a cost curve
whose Ω(N²) interaction floor i331 left in place.

## Leg B — liveness holds at 192 and 256

| N | side | agents | health | hunger | stress | grain | gini | zero-coin | partnered |
|---|---|---|---|---|---|---|---|---|---|
| 192 | 64 | 193 | 0.778 | 0.0462 | 0.239 | 15.14 | 0.359 | **0** | 192/193 |
| 256 | 74 | 256 | 0.777 | 0.0448 | 0.243 | 9.52 | 0.400 | **0** | 256/256 |

A 256-agent town runs healthy at ~5.8 ms/tick: `MAX_POPULATION` is 256, so this is the
demographic ceiling and it is reachable with ~11% of the per-tick budget to spare.

## Leg C — but the world is ONE settlement

`auto_partition_polities(max_gap)` (i298 single-linkage over house-site geography) on the
density-law world:

| max_gap | N=192 clusters | N=256 clusters |
|---|---|---|
| 4 | 48 | 58 |
| 8 | 20 | 21 |
| 12 | **0** | **0** |
| 20 | **0** | **0** |

Because i345's Vogel/sunflower spiral deliberately **distributes houses evenly** (every
house gets an equal share of the world), the mean inter-house spacing at N=192/64² is
≈9 tiles, so a gap of 12+ merges everything into a single settlement, while gap ≤8
fragments into near-singleton house clusters. **There is no natural 3–5 settlement town
to orchestrate** — the density world is spatially uniform by construction.

`verdict = DENSITY_WORLD_IS_ONE_SETTLEMENT`.

## Consequence and queued next step

Sim capacity is **not** the blocker: the engine runs a 256-agent town in real time and
stays healthy. The blocker is **world generation** — a uniform spiral cannot produce
distinct villages. The natural next iteration is a **clustered world generator**
(multiple village centres, each with intra-village jitter, separated by ≥ the partition
gap), which is what makes the already-built multi-settlement machinery — i298
auto-partition, polity-scoped collective fields (i296/i297), cross-polity trade
diffusion — go live at town scale. That is a world-gen change with its own probe and
sweep (it moves where every agent lives, exactly the i339/i340 blast class), so it is
recorded here rather than folded into a measurement session.

Smaller recorded follow-up: the **LOD tier carrying Background agents** was not
exercised — i348 measured Background at 25–40% of *crisis*-world agent-ticks and the
calm town runs here are below the importance gate, so the LOD saving at town scale is
unmeasured.

## Verification

Probe + docs only; **no source changed**. Golden replay byte-identical, no re-anchors.
(`i359_town_scale` clippy-clean.)
