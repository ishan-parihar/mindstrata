# Iteration 294 — N≥96 superlinearity: measured, attributed, one accident fixed

**Status:** LANDED · **Charter:** DC3-P0 §5.2 item 2 (superlinearity probe at N=192,
never run). Probe: `i294_superlinearity` (bench-index law-clean).

## What was measured (release, 2000 ticks, seed 42, world 32×32, populate included)

| N | µs/tick (post-i294) | charter-i283 | Δ | events/tick | µs/event |
|---|---|---|---|---|---|
| 12 | 98.5 | 122.4 | **−20%** | 14.9 | 6.6 |
| 48 | 871.8 | 1 087.6 | **−20%** | 54.9 | 15.9 |
| 96 | 4 768.3 | 5 960.6 | **−20%** | 111.6 | 42.7 |
| 192 | 38 950.3 | *(first measurement)* | — | 223.5 | 174.3 |

Exponents (log-log least squares over the four rows):

- **α_total(µs/tick) = 2.115** — above the charter's "interaction-healthy ≈1.4" fit.
- **α_volume(events/tick) = 0.975** — event volume is **linear**; the emergence engine
  is not the cause of superlinear cost.
- **α_cost(µs/event) = 1.140** — cost per event climbs with N ⇒ the growth is
  **algorithmic**, contradicting the charter-i283 attribution
  ("interaction-driven event volume, not per-agent state").
- **α_rels(R vs N) = 2.030** — the dense legacy relationship matrix
  (R = N(N−1); 36 672 rows at N=192) is the quadratic structural floor.

## The accident found and fixed (behavior-identical, golden-proven)

`systems/cognitive.rs` §8.1.4 emotion-regulation: for **each** agent, the
social-support block scanned the **entire** legacy relationship matrix filtering
`r.from == i` — **O(N·R) = O(N³) per tick**. Iter-218's probe *named* this suspect
and never measured it; trust sync (its sibling suspect) was already an O(R) prepass.

Fix: one O(R) prepass computing per-agent top-3-trust means before the agent loop;
the per-agent block reads the precomputed value. Order-independence proof: the
sorted-top-3 insert never re-inserts an equal value and always shifts past smaller
ones, so row order cannot change the multiset — prepass is bit-identical. Referee:
**`scripts/gate` green, golden 5/5 byte-identical** before and after.

Attribution of the uniform −20%: at N=96 the removed scan was 96 × 9 120 ≈ 875K
row-checks/tick; at N=12 only 12 × 132 ≈ 1.6K — hence a constant-fraction
improvement at every scale, visible as the flat −20%.

## Legs that retired alternative hypotheses (measured, not inferred)

- **Density** (N=192, world 32×32 vs 64×64, 1K ticks): 28 605 vs 28 442 µs/tick,
  **+0.6%** — tile-crowding is not a scaling term.
- **Daily/weekly cadence** (N=192, 2K per-tick samples bucketed by scheduler phase):
  daily-boundary ticks +132% over fast ticks but **0.8% amortized** — the daily
  O(N×R) power-balance/mean-reversion loops are second-order (144×-amortized).
- **Event buffer**: append-only, no shift cost (charter §3 unchanged).

## Hot-path inventory updates (DC3-P0 §3)

1. **RESOLVED (this iteration):** social-support per-agent full-matrix scan
   (O(N³)/tick) → O(R) prepass. Was the dominant scaling accident.
2. **Structural floor (recorded, not scheduled):** the dense legacy matrix itself —
   α_rels = 2.03 puts Ω(N²)/tick under snapshot capture + trust prepass. The lever
   is a sparse-neighborhood relationship store (capped like `gossip::MAX_GOSSIP_EDGES`);
   deferred — N≤96 is the DC-3 Phase-1 envelope and N=96 sits **27% under** the
   6 500 µs/tick budget (4 768 measured).
3. **Confirmed healthy:** event volume linear (α_volume=0.975); density-insensitive;
   daily passes 0.8% amortized.
4. **Standing debt (unchanged):** per-agent recent-claims index if runs exceed
   250K ticks (claims/agent 12→30 across N=12→192 at 2K ticks feeds the
   O(claims)² salience-filter trigger recorded at i283).

## Budget-table deltas (DC3-P0 §1/§2)

- N=12 golden budget ≤150 µs/tick: **98.5 measured — 34% headroom** (was 22%).
- N=96 target ≤6 500 µs/tick: **4 768 measured — 27% headroom** (was +5% over pre-i281-era basis).
- Superlinearity classification corrected: ≈N^1.4 → **α_total = 2.1**, of which
  volume contributes ≈1.0 and per-event cost ≈1.14 (matrix-structural, post-fix).

**Probe:** `cargo run -p mindstrata-benches --release --example i294_superlinearity`
(≈4 min wall; min-of-3 at N≤48, min-of-2 at N≥96; density leg min-of-2; phase leg 2K samples).
