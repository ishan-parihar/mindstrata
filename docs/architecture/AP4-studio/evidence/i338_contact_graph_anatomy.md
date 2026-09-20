# i338 — Anatomy of the contact graph: the "sparse store" is refuted, and the Ω(N²) floor is a housing artifact

**Question.** After i335 re-contracted i326's demotion of the contact-driven sparse
relationship store ("scope was too narrow"), the store sat in the scale ledger as the
top-ranked structural fix. i338 was chartered to **build** it. The probe says do not.

**Probe.** `crates/mindstrata-benches/examples/i338_sparse_store_sizing.rs` — four legs,
seed 42, horizons 2K/20K, N = 12…96, world 32×32 and 64×64.

---

## 1. The contact graph is quadratic in N, at any density

| N | ticks | total R | touched R | share | partners/agent | max |
|---|-------|---------|-----------|-------|----------------|-----|
| 12 | 2 000 | 132 | 74 | 56.1% | 6.2 | 10 |
| 12 | 20 000 | 132 | 74 | 56.1% | 6.2 | 10 |
| 48 | 2 000 | 2 256 | 1 032 | 45.7% | 21.5 | 42 |
| 48 | 20 000 | 2 550 | 1 232 | 48.3% | 25.7 | 50 |
| 96 | 2 000 | 9 506 | 4 124 | 43.4% | 43.0 | 97 |
| 96 | 20 000 | 10 100 | 4 511 | 44.7% | 47.0 | 100 |

Local exponent 48 → 96 @20K: **total R α = 1.986, touched R α = 1.872.** A store that kept
only contacted edges would hold ~45% of the rows today — a **≤2× constant**, and the win
*shrinks* with horizon (mean degree 21.5 → 25.7 at N=48; 43.0 → 47.0 at N=96). At N=96 the
best-connected agent has met **all 100** possible partners.

**i335's open caveat — "this may be the fixed 32×32 world (density rising with N)" — is
now resolved: it is not density.** At constant density (world scaled with N, 20K ticks):

| N | world | total R | touched R | partners/agent |
|---|-------|---------|-----------|----------------|
| 24 | 23 | 702 | 349 (49.7%) | 14.5 |
| 48 | 32 | 2 550 | 1 232 (48.3%) | 25.7 |
| 96 | 45 | 10 302 | 4 834 (46.9%) | 50.4 |

**touched α = 1.896 at constant density** — statistically the same graph as the fixed-world
leg. Density is not the governor.

## 2. Agents never move (a dead producer)

Per-tick position tracking, seed 42, N=12, 5 000 ticks:

```
per-tick position changes: [0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0]
distinct cells visited:    [1, 3, 1, 1, 1, 4, 1, 1, 1, 1, 1, 1]
```

Ten of twelve agents never occupy a second cell in 5 000 ticks (the two that "move" are
death-compaction index shuffles — confirmed by `run(1)`-loop ≡ `run(20000)` giving an
identical end state). The action histogram over 5 000 ticks × N:

| action | N=12 | N=48 |
|---|---|---|
| Work | 53.6% | 51.9% |
| Rest | 26.9% | 27.4% |
| Trade | 5.3% | 8.0% |
| Eat / Socialize / Worship / Drink / Idle | remainder | remainder |
| **Wander** | **0.00%** | **0.00%** |
| **Move** | **0.00%** | **0.00%** |

`ActionKind::Wander` and `ActionKind::Move` are **never selected**. They are not gated
out of `select_action` — both are candidates (and `Move` is reachable via the §19.5.G
feud branch) — they simply never win: `Wander` carries no need-relief term at all
(only a ±0.05-scale normative nudge), and the routine ladder (`pass_action.rs`) dominates
the utility path (Work+Rest = 80% of agent-ticks). So locomotion — with its biology
exertion cost, economy cost, §19.5.G feud approach, and the memory pass's Wander-position
encoding — is **dead in the live pipeline**, and space is frozen at `populate`.

**But interactions *are* distance-gated.** At N=12, **74 of 74** touched relationships are
within the radius; at N=48, 1 111 near vs 121 far (91.4%). Proximity does govern who meets
whom — which makes the quadratic contact graph a puzzle, since a static proximity graph at
density 48/1024 should touch ~6% of pairs, not 45%.

## 3. Root cause: the population occupies exactly 8 cells

| N | world | occupied cells | max co-located | near-pair share (r=5) |
|---|-------|----------------|----------------|------------------------|
| 12 | 32 | **8** | 2 | 40.9% |
| 12 | 64 | **8** | 2 | 45.5% |
| 48 | 32 | **8** | 6 | 39.4% |
| 48 | 64 | **8** | 6 | 45.7% |
| 96 | 32 | **8** | 12 | 40.0% |
| 96 | 64 | **8** | 12 | 46.3% |

- `world_gen::generate_village` places houses with a hardcoded `for i in 0..8` on a
  jittered ring of radius ~4 around the map centre — **independent of N and of world size**.
- `population.rs` assigns `home_site = house_indices[i % 8]` — plain round-robin, ignoring
  the house's `capacity: 4`.
- `population.rs` then sets `position` from the home site, and nothing ever moves it.

So every agent lives at one of **8 points inside a ~10-tile disc**, with `ceil(N/8)` agents
at each *identical* position (2 at N=12, 6 at N=48, 12 at N=96 — exactly as measured).
Every within-house pair is at distance **0** and therefore trivially "near", contributing a
near-share of ≈ 1/8 = 12.5%; the ring's cross-house proximity adds a further ~28%. Hence
near-pair share ≈ 40–46%, **flat in N and in world size** — the measured row above, and the
arithmetic that closes the loop with §1.

The contact graph is quadratic because **the village is 8 co-residency buckets**, not
because space is degenerate or because interactions ignore it.

---

## Verdict

**`SPARSE_STORE_REFUTED_AS_SCALE_FIX`.**

A contact-driven store would save ≤2× of the relationship matrix at the charter horizon,
deliver less at longer horizons, make tick cost time-dependent, and — being a behavioural
change to every per-edge pass — would buy that constant at the price of a full re-anchor
sweep. §4.1/§4.2 forbid paying a sweep for a demoted lever; the ledger's top-ranked scale
item is therefore **not built**, and the probe is recorded instead.

**`OMEGA_N2_IS_A_HOUSING_ARTIFACT`.** i335 concluded the quadratic floor was "a
construction choice at populate". i338 identifies *which* construction: 8 fixed house sites
plus round-robin assignment plus zero mobility. The floor is not intrinsic to the social
model — it is an artifact of the world generator not scaling with the population, and it is
therefore removable.

## Consequences for the ledger

1. **The real scale lever is spatial spread, not storage**: house/site count must scale with
   N (and/or the ring must span the map), so that co-residency stops growing linearly in N.
   This is *behavioural* — it changes who meets whom — so it needs its own probe + re-anchor
   sweep, exactly as this item would have.
2. **Two dead producers recorded** (AGENTS §4.3): `ActionKind::Wander` (0.00% of
   agent-ticks) and `ActionKind::Move` (0.00%, including its §19.5.G feud-approach path).
   The §6 spatial subsystem — positions, locomotion costs, TUI rendering, memory-of-places —
   is inert for behaviour.
3. **`SiteKind::House.capacity = 4` is unenforced** at assignment, and co-location reaches
   12 agents per cell at N=96.
4. The world grid's *size* is decorative for contact: 32×32 and 64×64 produce the same
   near-pair share to within the ring's own geometry. Any charter reasoning that leans on
   world area as a scale knob is leaning on nothing.

## Reproduce

```
cargo run --release -p mindstrata-benches --example i338_sparse_store_sizing
```
(throwaway action-histogram / position-census probes used during the investigation were
removed before commit; their outputs are quoted above.)
