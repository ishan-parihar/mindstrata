# A9 — the world-area policy is adopted as interim, and single-sourced

**Status:** LANDED (policy + single-sourcing refactor). **Operator decision (2026-09-23):**
adopt the constant-density law **as the interim policy**, and keep the first-principles path
to the organic replacement in view (§5). Verification: `cargo fmt --all`, `cargo clippy
--workspace --quiet`, sim **310/310**, integration **313/0/1**, `gate --full` GREEN, both
goldens byte-identical (recorded in the iteration's commit).

---

## 1. The decision

The world's area follows the population at the simulator's own calibrated density:

```text
world_side_for_population(n) = max(16, ceil(sqrt(CELLS_PER_AGENT * n)))
CELLS_PER_AGENT = 256.0 / 12.0 = 21.333…   // N=12 in 16x16 and N=48 in 32x32 agree exactly
```

`SimConfig::for_population(seed, max_ticks, num_agents, snapshot_interval)` is the caller-side
entry point: a run that does not pin a size asks for *a world sized for this many villagers*.

## 2. A correction this iteration must record: **capacity is A11's fix, not A9's**

The earlier framing (including a line in `PLAN_DC5` §5) credited A9 with taking
**max co-location 19 -> 4** at N=192. That is wrong, and the sources make it clear:

- i344 measured `19` co-located in the **ring** world and proposed the density law, listing
  "the housing ring degenerates in a fixed world" as a *separate* new item (A11).
- **i345 landed A11** — spacing-aware area packing above 24 houses, one house per tile — and
  *that* is what took max co-location **19 -> 4** (contacted rows -27%, partners/agent
  26.3 -> 19.1), at **zero blast radius** for the calibrated corpus.
- A9's own measured effect is therefore **contact dilution, not capacity**: near-pair share
  11.3% -> 4.7%, contacted-row share 11.5% -> 5.3%, mean partners/agent **22.1 -> 10.2**,
  contacted alpha **1.276 -> 0.866** (i344's density column, measured against the pre-A11 ring).

Two consequences worth keeping: the two items were **not redundant** (A11 fixed placement
correctness; A9 changes what a *town* is), and A9's remaining justification is the **contact
graph** — which is exactly what W2's encounter-driven work (i397) will re-measure.

## 3. What the policy does and does not do

| | Effect |
|---|---|
| Cost | **+5.6% µs/tick at N=192** (+9.1% at N=96, -2.3% at N=144). i344's verdict stands: **world area is a fidelity lever, never a throughput one** — the interaction store is the complete row set `N(N-1)`, which a bigger world leaves untouched |
| Calibrated corpus | **untouched by construction.** The law is anchored on the engine's own two calibrated points, so `world_side_for_population(12) == 16` and `(48) == 32` exactly; the village and the N=48 tier are byte-identical under either policy — **both goldens verifying this is the acceptance criterion** |
| Town tier (N >= 96) | contact stops saturating (alpha 1.276 -> 0.866), near-pair share 11.3% -> 4.7%, partners/agent 22.1 -> 10.2 |
| The N=256 cap | the law emits **74** (5 476 cells / 256 = 21.4 cells/agent, i.e. the density holds rather than drifting) |

## 4. Single-sourcing (the cleanup half)

The law had been **copied into six probes** (`i344_world_area_law`, `i359_town_scale`,
`i360_clustered_world`, `i362_cross_polity_diffusion`, `i366_cluster_cap`,
`i367_lod_town_scale`) and once into a sim test (`sim/tests/mod.rs`). Seven copies of a policy
is how a policy silently forks, so the definition now lives once — in the world generator
beside `houses_for_population` / `cluster_count_for` / `MAX_RING_HOUSE_COUNT`, where the rest
of the spatial grammar already lives — and every copy is replaced by an import
(`use mindstrata_sim::world_gen::world_side_for_population as density_side;`, so the call sites
did not churn).

Two pins carry the law (in `mindstrata-world::world_gen::tests`):
`world_side_law_reproduces_both_calibrated_points` (the anchoring is the *reason* the policy
is safe to adopt, so it is pinned rather than assumed) and
`world_side_law_is_monotone_and_floored` (never shrinks, never dips below 16, density at the
cap still tracks 21.33).

## 5. The organic replacement — what to keep in view (operator request)

**This policy is still an input, not an emergent property.** `side(N)` says "make the world
this big for the population you chose", when the emergent form is the inverse: a **fixed
physical area** whose population is what varies. Four first principles, in dependency order:

1. **Population is an output bounded by carrying capacity.** The world's ecology (food/water
   regeneration, fertility fields, spoilage) should set a supportable population, and growth
   should *feel* the ceiling — through crowding, disease, conflict over space, and
   out-migration — rather than hitting `MAX_POPULATION = 256`. Every one of those pressures
   already has a live producer; what is missing is that they bound growth instead of merely
   registering it.
2. **Space must be causal.** Today distance shapes almost nothing: locomotion is one tile per
   decision and reachable only through the feud approach. Travel should cost time and energy,
   so distance to the well / farm / market / temple determines a villager's *actual daily
   reach* (i396).
3. **Contact must come from encounters, not assignment.** Ties should form where activity puts
   people together — the market on market day, the temple at ritual, the farm at harvest —
   rather than because two agents were handed the same home cell. This is i397, and it is the
   item that makes a town behave like a town.
4. **Density becomes a pressure with both ends wired.** Crowding already feeds infection
   exposure (`crowding_amplifier = 0.5 + crowding`, wired at 0.3). The full axis extends it to
   stress, housing quality, and conflict over space — and then to the demographic response:
   **fission**, a settlement past its local capacity founding a new site. That is what would
   make the settlement-size distribution *emerge* instead of being
   `cluster_count_for(houses) = clamp(houses/16, 1, 4)`.

**Reading of the path:** A11 is landed, A9 is the interim policy, i396/i397 are the next two
steps that eat into it, and the carrying-capacity/fission work is what finally **retires the
function** — at which point the interesting question is how many agents the world supports,
not how big to make the world for a chosen N. Recorded here and in
`docs/PLAN_DC5_DEVELOPMENT.md` §4 so the target survives the intervening iterations.
