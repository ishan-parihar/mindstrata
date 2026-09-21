# i345 — A11: the housing layout must fit the world it is placed in

**What landed.** Above the largest **measured** house count the ring layout is replaced by a
spacing-aware **area packing**:

- `world_gen::MAX_RING_HOUSE_COUNT = 24` — the largest count ever measured/calibrated
  (i339/i340 probed to N=96 ⇒ `ceil(96/4) = 24` houses). Every calibrated run therefore keeps
  the historical ring **byte-for-byte** (same ring radius rule, same draw order, same
  RNG consumption).
- Above it, houses are placed on a **Vogel/sunflower spiral** (`r ∝ √i`, golden-angle stride)
  so each house gets an equal share of the disc's area rather than crowding the rim. A
  candidate tile that is water or already occupied is nudged outward along a golden-angle ray
  until a free tile is found, so the house count is honoured exactly.

## The defect this fixes, measured

`ring_span = min(w,h)/2 − 2` is a function of **world size only**, so the ring's angular
stride shrinks as houses are added. i344 measured the consequence; this iteration A/B'd it
with one probe (`i345_house_packing`, seed 42, fixed 32×32, 5 000 ticks) against the same
probe run at HEAD:

| N | house sites | distinct house **tiles** | max agents co-located | house pairs ≤ 5 | contacted rows | partners/agent | near share |
|---|-------------|--------------------------|-----------------------|-----------------|----------------|----------------|------------|
| 96 (ring, control) | 24 | 23 | 5 | 5.1% | 1 193 | 12.4 | 9.0% |
| 144 ring | 36 | **34** | 7 | 8.0% | 2 448 | 17.0 | 10.2% |
| 144 packing | 36 | **36** | 5 | 6.7% | 2 666 | 18.5 | 9.1% |
| 192 ring | 48 | **43** | **19** | 8.4% | 5 050 | 26.3 | 11.3% |
| 192 packing | 48 | **48** | **4** | 6.9% | 3 660 | 19.1 | 8.8% |

- **`place_site` silently overwrote occupied tiles**: at N=192 the ring put **48 houses onto 43
  tiles** (5 collisions; N=144 lost 2) and agents piled **19 to a cell** against the declared
  `SiteKind::House.capacity = 4`. The packing places **one house per tile** at every N.
- The **N=96 row is the control** and is identical in every geometry/contact column across the
  two runs — the probe is deterministic and the deltas above are real, not run-to-run drift.
- Crowding falls with it: house pairs inside the radius-5 neighbourhood 8.0 → 6.7% (N=144),
  8.4 → 6.9% (N=192); contacted rows −27% and partners/agent 26.3 → 19.1 at N=192.

## Cost: unchanged (and the control proves why the probe can't say otherwise)

| N | ring (min-of-3) | packing | Δ | control noise |
|---|-----------------|---------|---|---------------|
| 96 (**no code change**) | 940.4 | 974.5 | **+3.6%** | this *is* the noise floor |
| 144 | 2 024.4 | 1 935.7 | −4.4% | |
| 192 | 3 366.4 | 3 761.8 | +11.8% | |

The N=96 row changes **nothing** in the code and still moves +3.6%; i337 recorded ±5–10%
between processes. So the cost column is **inconclusive** and is reported as such — A11 is a
placement-correctness fix, **not** a throughput win (the same conclusion i344 reached for its
parent item A9, and bounded by the same cause: a fixed world's area is the real constraint on
house crowding, so spacing cannot improve much and rows are untouched either way).

## Blast radius: zero

- Max test population is **N=48 ⇒ 12 houses**, below `MAX_RING_HOUSE_COUNT`, so the legacy ring
  path executes for every test and golden (verified: `mindstrata-sim` **292/292**, +1 new pin).
- New pin `large_villages_place_one_house_per_tile` (N=192): house sites == distinct house
  tiles == `houses_for_population(192)` — the round-robin in `population.rs` is only honest if
  that equality holds, so it is pinned rather than left to a probe.
- Golden baselines regenerate nothing: the N=12 village is 8 houses on the ring.

## Verdict

**`HOUSE_LAYOUT_RESPECTS_TILES_AND_CAPACITY_ABOVE_THE_RING_RANGE`.** A declared-capacity
contract (`House.capacity = 4`) that the ring silently violated at large N now holds by
construction; the layout change is confined to uncalibrated territory (N > 96) and the
calibrated village is byte-identical. Residual recorded: the ring's 1-tile minimum gap is *not*
fixed by the packing in a fixed world (measured min gap still 1–2) because 48 houses cannot be
spread in 1 024 tiles — that is A9's finding, and it is why A9 exists as a charter option
(i344: fidelity, not throughput).

## Reproduce

```
cargo run --release -p mindstrata-benches --example i345_house_packing
git stash push crates/mindstrata-world/src/world_gen.rs   # ring baseline, same probe
cargo run --release -p mindstrata-benches --example i345_house_packing
git stash pop
```
