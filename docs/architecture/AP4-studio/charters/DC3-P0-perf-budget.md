# DC-3 P0 — Performance Budget & Event-Buffer Decision

**Status:** RATIFIED (i283 session) · **Baseline:** i274 probe, re-measured post-i281 (`i274_scale_diet_probe --perf`, release, 2000 ticks, seed 42, world 32×32)

## 1. Measured baseline (fresh, post-i280/i281/i282)

| N | µs/tick | tps | vs i274 (102/928/5696) |
|---|---|---|---|
| 12 | 122.4 | 8 170 | +20% (i281 window adds a filter; i280 WP-J adds a per-agent membership scan — net cost small) |
| 48 | 1 087.6 | 919 | +17% |
| 96 | 5 960.6 | 168 | +5% |

Scaling remains **superlinear** (≈N¹·⁴ at N=48, ≈N⁰·⁷ additional to 96 — interaction-driven event volume, not per-agent state). Event volume @20K/N=12: 281K events.

## 2. Budget rules (binding for DC-3 work)

1. **N=12 golden budget: ≤ 150 µs/tick release.** Golden/snapshot suites (307 tests, ~4 min) must stay under 6 min wall. Current headroom: 22%.
2. **N=96 target: ≤ 6 500 µs/tick through DC-3 Phase 1.** Any pass exceeding +10% N=96 with <10% behavioral yield is a candidate for the §3 hot-path list.
3. **Per-tick allocation ban** on the six verbatim passes (Arc-D rule, unchanged): no `Vec::new()` in per-agent loops; buffers hoisted or `Vec::with_capacity`-reused.
4. **Determinism trumps perf**: any optimization that reorders event iteration is rejected regardless of gain (golden replay is the referee).

## 3. Hot-path inventory (from the i283 survey)

- `events: Vec<SimEvent>` is **append-only** today (per-tick drain was measured as a 30% regression at `96ea2c6+perf` and reverted). No shift cost exists; the cost is memory (§4).
- `norms_impl` / `social_cluster` scan the per-tick window `pre_tick_events..snap_count` — O(events/tick), correct.
- `recent_events(n)` is O(n) tail slice — fine.
- The i281 salience filter is O(claims) per agent per social-action selection; claims grow ~58/agent per 20K. At 100K+ this becomes O(claims)² per selection cycle — **recorded: add a per-agent recent-claims index if runs exceed 250K ticks** (not before; measured plateau means the working set is bounded by ~2× the window).

## 4. VecDeque re-decision (the i274 ponytail, closed)

**Measured:** `SimEvent` = **56 bytes** (`size_of` verified; `Vec` fields are handles). Append-only growth:

| horizon | events | memory |
|---|---|---|
| 20K @N=12 | 281K | 16 MB |
| 100K @N=12 (i283 UM-2 horizon) | ~1.4M | 78 MB |
| 1M ticks (mortality horizon, i272) | ~14M | **780 MB** |

**Decision: DEFER.** Perf is unaffected (no shifts in append-only mode; the drain experiment already proved trimming is the regression, not the cure). Memory becomes binding only beyond ~250K ticks on minimum-RAM hosts. The conversion touches ~15 slice sites (`&self.events[a..b]` → iterators) — a pure-refactor iteration with its own golden gate, costed at one full iteration.

**Recorded trigger to revisit:** operator scenarios targeting >250K-tick horizons, embedded/memory-capped hosts, or the annals-export feature (ROADMAP UM-3) needing bounded journals.

## 5. DC-3 Phase-1 checklist (next iterations)

1. [x] `i284_*` perf regression probe: assert the N=12 budget in CI-adjacent benches (warn-only; release variance ±8%). — **LANDED i295** (`i295_perf_budget_gate`, warn-only step in `scripts/gate` 2.55; hard floors remain in i270/i271 gate 2.6).
2. [x] Superlinearity probe at N=192 to separate interaction-volume growth from algorithmic accidents (i274 design, never run). — **LANDED i294** (α_total=2.115; volume linear α=0.975; O(N³) social-support scan fixed golden-identical, uniform −20%; see `evidence/i294_superlinearity.md`).
3. [ ] Asset pipeline v0 charter (ROADMAP UM-3 "the world scales") — separate doc; this budget sets its perf envelope.
