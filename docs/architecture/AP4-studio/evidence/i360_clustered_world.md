# Iteration 360 — the clustered world generator: a 256-agent town is now a town of villages

**Status:** LANDED (behavioural, world-gen) · **Item owned:** the plan's C-item
follow-up — i359 measured sim capacity reaching N=256 but found the density world
collapsed to **one settlement**, so the already-built multi-settlement machinery
(`auto_partition_polities` i298, polity-scoped collective fields i296/i297,
cross-polity trade diffusion i299) had nothing to orchestrate.

## The root cause, restated from i359

The i345 area packing is a single **Vogel/sunflower spiral**: every house gets an
equal share of the disc's area (`r ∝ √i`). That is exactly right for *spacing*, and
exactly wrong for *settlement structure* — at N=192/64² the mean inter-house spacing
is ≈9 tiles, below any partition threshold, so single-linkage agglomeration merges the
whole world. i359's probe: `gap 12 -> 0`, `gap 20 -> 0` settlements.

## The fix — `cluster_count_for(houses) = clamp(houses/16, 1, 4)`

Above the calibrated range (`house_count > MAX_RING_HOUSE_COUNT = 24`), houses are now
placed around **K village centres** instead of one uniform spiral:

- **K centres** sit analytically (consuming **no** World draws) on a ring of radius
  `centre_ring = ring_span/2` around the world centre.
- Each centre's balanced house share packs onto a **local** Vogel spiral of radius
  `local_r = (ring_span/(K·1.6)).clamp(3, 9)`, so intra-village spacing stays small
  while villages stand `≈ 2·centre_ring·sin(π/K)` apart.
- Placement is deterministic and **honours the count**: a local golden-angle walk,
  then a wide walk, then a `nearest_free_tile` world scan — the i344 invariant (one
  tile per house) holds even in a cramped synthetic world.
- The **four civic tiles** (farm/well/market/temple, placed after the houses) are
  **reserved** during the house search, so a civic site can never steal a house tile
  (the failure this iteration hit first: 48 house sites on 47 tiles at N=192 in 32×32).
- `K ≤ 1` routes through the **unchanged** i345 uniform spiral — every calibrated run
  (< 25 houses) is byte-identical by construction, and the cluster rule is inert
  through the whole measured range.

## Leg A — the world is now multi-settlement (was one blob)

`auto_partition_polities(max_gap)` on the i344 density world law, 4 gaps, against
i359's recorded baseline (`48/58 @ gap4, 20/21 @ gap8, 0 @ gap12, 0 @ gap20`):

| N | side | houses | gap4 | gap8 | gap12 | gap20 |
|---|---|---|---|---|---|---|
| 192 | 64 | 48 | 3 | **3** | **3** | 0 |
| 256 | 74 | 64 | 4 | **4** | **4** | **4** |

The regime flipped from *"many singleton clusters that merge into one settlement"* to
**K coherent villages that stay separate**: at the natural gap (8) the partition is now
exactly K = 3 and 4, balanced to 64 members each. `verdict = TOWN_OF_VILLAGES`.

## Leg B — liveness holds and polity genesis goes live

| N | agents | health | hunger | stress | grain | gini | zero-coin | polities | genesis-memes/polity |
|---|---|---|---|---|---|---|---|---|---|
| 192 | 192 | 0.775 | 0.043 | 0.240 | 16.98 | 0.473 | 0 | 3 (64/64/64) | [2, 2, 1] |
| 256 | 256 | 0.770 | 0.044 | 0.244 | 15.56 | 0.494 | 0 | 4 (64/64/64/64) | [6, 4, 2, 1] |

Provisioning is unaffected because site access is **institution-based, not
proximity-gated** (`World::can_access_resource`; `best_farm_for_work` keys on stock, not
distance) — so the single central market/farm/well/temple serves every village. Each
polity mints its **own** `[genesis:pN:...]` memes, i.e. the i158/i297 per-polity holon is
now live at town scale with ≥1 distinct novel meme per settlement.

## Verification

- **Byte-identity, calibrated range:** golden replay 5/5 byte-identical (every calibrated
  run is ≤ 24 houses ⇒ the ring path); doc-index reconciled.
- **Suites:** `mindstrata-world` 60/60 (incl. `cluster_count_matches_house_bands`);
  `mindstrata-sim` **299/299** (incl. `clustered_world_forms_multiple_settlements_at_town_scale`,
  multi-seed 42/7/1/99 × N=192/256); `mindstrata-tests` **310 passed / 0 failed / 1
  ignored**; `scripts/gate --full` **GREEN**.
- **Re-anchors:** none. The only test that moved was `large_villages_place_one_house_per_tile`
  (N=192 in a 32×32 world) — the clustered layout initially **failed** it (the civic-tile
  collision above), and the fix makes it pass *and* adds the multi-seed settlement pin.
  No behavioural pin shifted, because nothing calibrated touches > 24 houses.

## Consequence and queued next step

The blocker i359 named — *"world structure, not sim capacity"* — is cleared: a 256-agent
town is now spatially a town of four villages with per-polity holons. The natural next
item is the **cross-polity liveness measurement** the substrate was built for: run a
partitioned town long enough to see `system_trade_diffusion` (i299) move genesis memes
across village boundaries, and check whether cross-polity trade actually occurs now that
villages are geographically distinct (agent-to-agent trade needs a counter-party within
12 tiles, so an inter-village boundary channel is the question). Also unexercised:
whether a larger `cluster_count_for` cap (5–6) buys more polities without crowding a
density-law world.
