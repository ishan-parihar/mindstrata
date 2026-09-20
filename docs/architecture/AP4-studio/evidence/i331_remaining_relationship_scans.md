# Iteration 331 — the last accidental relationship scans; tick cost reaches the quadratic floor

**Status:** LANDED · **Item owned:** i330's recorded remainder — "`instit+faction+panic+derived`
is the next superlinear block (unsplit)" and the memory-encoding pass at 45% of the tick.

## What i330 left behind

i330's profiler attributed the ≈N^2.8 term to the *social pass* and removed four
`relationships.iter().find(..)` scans there. It then recorded two surviving
blocks: the memory-encoding pass (`tick_memory_encoding`, **45% of the tick**,
believed O(N·E) "by design") and the unsplit `instit+faction+panic+derived`
block (local α ≈ 2.9). Both turned out to hide **more instances of the same
accident class** i329/i330 had already found.

## The three scans removed

| Site | Shape before | Per-tick bound |
|---|---|---|
| `tick_memory_encoding` (§19.5.G social-status fold) | per-agent `relationships.iter().filter(r.from == i)` | **O(N·R) = O(N³)** — inside the 45% pass |
| `tick_derived_states_and_beliefs` (`norms_impl.rs`) | `relationships.iter().find(..)` **per interaction event** | O(E·R) = **O(N³)** — the bulk of the 17% block |
| `tick_resource_operations` (economy, trade trust read + trade-trust write) | `relationships.iter()/iter_mut().find(..)` **per trade** | O(T·R) = O(N³) (T = trades/tick) |

All three now use the i329/i330 machinery:

- **Memory encoding** — hoisted to **one O(R) pass**. Each row contributes to
  exactly its own `from` agent, so the per-agent counts are identical by
  construction. Extracted as `memory_ops::social_status_counts(&[Relationship], n)`
  so the equivalence is pinned by a unit test rather than only by goldens.
- **Derived/belief + economy** — `Simulation::rel_pos(from, to)`: O(1) dense
  `(from·n + to) → position` lookup with the revalidating fallback (a birth or
  death shifts ids → degrades to the linear scan, correct on every tick). The
  lookup is already fresh: `tick_gossip_and_knowledge` rebuilds it (i330) and the
  derived/belief pass runs immediately after, with the population unchanged
  across resource-ops (marriages/births run later in the tick).

`rel_pos` returns the **first** matching element — exactly what
`iter().find(..)` / `iter_mut().find(..)` return — so all three reads are
value-identical.

## Measured result

Exponent probe (`i329_local_exponent`, release, seed 42, 32×32, fast ticks).
Baseline is **HEAD i330 measured by `git stash` in this session**, not cited:

| N | µs/tick before | µs/tick after | Δ | µs/agent after |
|---|---|---|---|---|
| 48 | 539.8 | 522.0 | −3.3% | 10.87 |
| 96 | 2 332.1 | 1 591.6 | −31.8% | 16.58 |
| 144 | 6 295.3 | 3 563.1 | −43.4% | 24.74 |
| 192 | 12 811.6 | 6 480.2 | **−49.4%** | 33.75 |

Local exponents: **2.111 / 2.449 / 2.470 → 1.609 / 1.988 / 2.079** — the top of
the envelope now sits **on** the structural Ω(N²) floor, not above it. (The
`i329` probe's own verdict string prints `SUBQUADRATIC` below α=2.15; read it as
*at the floor*, not literally below quadratic — the reading is a per-agent cost
that rises 10.87 → 33.75 µs for a 4× N, i.e. ≈N^2.)

Per-pass at N=192 (`i330_pass_profile`, tick 400) — baseline (i330 HEAD) → after:

| pass | before ns | after ns | Δ |
|---|---|---|---|
| memory encoding | 6 259 199 | 1 391 721 | **−77.8%** |
| instit+faction+panic+derived | 2 274 760 | 488 161 | **−78.5%** |
| resource_ops | 55 371 | 3 600 | **−93.5%** |

The two big drops grow with N exactly as removing an O(N³) term predicts. The
memory pass is no longer the tick's dominant term — the tick is now spread
across cognitive (1 108, 17%), appraisal (1 198, 19%), memory (1 392, 21%),
and the derived/belief sub-block (460, 7%).

## Verdict

**`ACCIDENTAL_RELATIONSHIP_SCANS_SWEPT` → tick cost at the quadratic floor.**
A full-crate sweep confirms **no** remaining `relationships.iter().find/filter`
scan in `mindstrata-sim` (the only `.position(..)` calls left are the
revalidating fallbacks inside the i329/i330 lookup machinery). Every accidental
per-agent / per-event / per-trade matrix scan found since i329 is now removed.

## What remains (recorded, not chased)

1. **`tick_memory_encoding` O(N·E) — still the design term.** Now ~21% of the
   tick (was 45%). The principled reduction is §2.4's own doctrine — gate
   attention on the same perception radius the interaction engine already uses —
   but it turns O(N·E) into O(N·local) and is **behavioural**, needing its own
   probe + re-anchor sweep. Queued.
2. **The structural Ω(N²) interaction matrix** is now the floor that decides the
   envelope, exactly as the architecture predicted; breaching it needs a
   spatial/index structure (i294's original question), not a scan removal.

## Verification

- **Golden 9/9 byte-identical**, `scripts/gate --full` **GATE GREEN** (309/0/1),
  **zero re-anchors**, no snapshot drift. `cargo fmt` clean, clippy 0 warnings.
- New pin `social_status_counts_matches_per_agent_fold` (in `sim/tests/mod.rs`):
  the hoisted O(R) pass must equal the per-agent fold on real sim data (12
  agents × 500 ticks) and on a synthetic list with **duplicate pairs** (both rows
  counted) and an **out-of-range `from`** (skipped). `mindstrata-sim` 284/284.
- `rel_pos` value-identity already pinned by i330's
  `dense_relationship_lookup_matches_linear_scan` (populated / empty / stale).
- The opt-in per-pass profiler (`MINDSTRATA_PROFILE_TICK`) gained sub-marks for
  the `instit+faction+panic+derived` block, so its split is now visible.

**Probes:** `i329_local_exponent`, `i330_pass_profile`.
