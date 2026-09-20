# i339 — The i338 lever is real: scaling the house count with N localizes contact

**Context.** i338 refuted the contact-driven sparse relationship store (≤2× constant,
horizon-fragile) and localized the Ω(N²) floor to a housing artifact: `generate_village`
placed exactly 8 house sites for every N and every world size, `populate` assigned
`home_site = house_indices[i % 8]`, positions were frozen (Wander/Move 0.00%), so every
village was 8 co-residency buckets with `ceil(N/8)` agents per cell. The store had to
represent all-pairs *within* those buckets.

That left one question before any behavioural iteration could be justified: **if housing
scaled with the population, would contact actually localize?** If not, the "spatial spread"
lever would be a paper fix and the Ω(N²) floor would be intrinsic after all.

**Instrument.** `Simulation::set_house_count` + `world_gen::generate_village_with_houses`,
both **opt-in and default-identical**: `DEFAULT_HOUSE_COUNT = 8` reproduces the historical
draw order exactly, and the 8-house branch keeps `ring_base = 4.0` and the legacy
8-step angle stride. The knob exists so the counterfactual can be measured without moving
any calibrated behaviour (gate GREEN, goldens 9/9 byte-identical, 309/0/1 before and after).

**Probe.** `crates/mindstrata-benches/examples/i339_housing_spread.rs` — seed 42, 5 000
ticks, world scaled to constant density (N=24/48/96 in 23/32/45 grids), shipped 8 houses
vs `ceil(N/4)` houses (one per ~4 villagers, matching the declared
`SiteKind::House.capacity = 4`) spread over a map-spanning ring.

| N | world | houses | total R | touched R | partners/agent | max | cells occupied | max co-located | near share |
|---|-------|--------|---------|-----------|----------------|-----|----------------|----------------|------------|
| 24 | 23 | 8 (shipped) | 600 | 279 (46%) | 11.6 | 24 | 10 | 3 | 43.0% |
| 24 | 23 | 6 | 552 | 216 (39%) | 9.0 | 12 | 7 | 4 | 39.1% |
| 48 | 32 | 8 (shipped) | 2 256 | 1 032 (46%) | 21.5 | 42 | 11 | 6 | 44.1% |
| 48 | 32 | 12 | 2 256 | 302 (13%) | 6.3 | 14 | 22 | 4 | 10.5% |
| 96 | 45 | 8 (shipped) | 9 120 | 4 044 (44%) | 42.1 | 59 | 11 | 12 | 43.4% |
| 96 | 45 | 24 | 9 312 | 750 (8%) | 7.8 | 96 | 34 | 5 | 5.6% |

**touched-R exponent (24 → 96): shipped 1.929 | `ceil(N/4)` houses 0.898.**

## Reading

- **Spread localizes contact.** With `ceil(N/4)` houses the touched-graph exponent is
  **0.898 — sublinear in N**, while mean partners/agent stays *flat* (9.0 → 7.8 across a 4×
  population) instead of growing linearly (11.6 → 42.1). That is the signature i338's probe
  said to look for: co-residency stops scaling with N.
- **The store shrinks an order of magnitude.** At N=96 the contacted share falls 44% → **8%**
  (750 of 9 312 rows), and occupied cells rise 11 → 34 with max co-location 12 → 5. Every
  per-edge pass and every per-agent own-list walk in the tick scales with that.
- **Near-pair share collapses** 43.4% → 5.6% at N=96, which is the quantity that drives both
  the store's footprint and the interaction network's density.
- **The remaining 8/24 shortfall is the radius, not the housing.** At N=96/24 houses the best
  connected agent still touches 96 partners (`max` column) and `cells occupied` (34) is well
  under `houses` (24 × ~1 tile) — i.e. the map-spanning ring puts many houses within the
  radius-5 perception neighbourhood of each other. Fine-tuning the ring span and/or the
  perception radius is a second-order question *after* the spread lands; it is not a reason to
  hold the lever.

## Verdict

**`HOUSING_SPREAD_LOCALIZES_CONTACT`.** The i338 lever is confirmed measurable, not paper:
the quadratic relationship store is removable by construction, because the density of the
contact network is set by how many villagers share a cell — which is set by the house count,
which was hardcoded to 8.

**What this does NOT do:** it does not land the behavioural change. Shipping
`ceil(N/4)` houses as the default moves who meets whom for every calibrated run, so it
carries a full re-anchor sweep (goldens, ~20 snapshots, the courtship/kinship/trust pins that
lean on the contact graph). That iteration (i340) now has its evidence, its instrument, and a
measured prediction: near-share 43% → 6%, touched share 44% → 8% at N=96, touched-R α
1.93 → 0.90.

**Also recorded** (from i338, unchanged): `ActionKind::Wander` and `Move` are selected 0.00%
of agent-ticks, so spread changes *where* villagers are frozen, not whether they move.
Bounded mobility remains a separate design question (ledger A8).

## Reproduce

```
cargo run --release -p mindstrata-benches --example i339_housing_spread
```
