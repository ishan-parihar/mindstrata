# Iteration 329 — a surviving superquadratic term above N=96 (i294's fit hid it)

**Status:** LANDED · **Item owned:** i294's scale attribution — the one remaining
top-level scale question after i326 (memory: bounded) and i328 (tier gating:
single-digit) closed the other two.

## The measurement (probe `i329_local_exponent`, release, seed 42, fast ticks)

i294 fitted **one** log-log slope across N = 12/48/96/192 and reported
α_total = 2.115, reading the residual as the dense relationship matrix's Ω(N²)
floor. Measuring **local** exponents at the top of the envelope instead:

| N | µs/tick | µs/agent | local α (vs previous N) |
|---|---|---|---|
| 48 | 676.8 | 14.10 | — |
| 96 | 3 484.6 | 36.30 | **2.364** |
| 144 | 11 385.8 | 79.07 | **2.920** |
| 192 | 24 842.8 | 129.39 | **2.712** |

Per-agent cost rises **14.1 → 129.4 µs (9.2×) for a 4× population**. The local
exponent above N=96 is **≈2.7–2.9, not 2.1** — a single fit across four rows
masks it because the low-N rows are floored by fixed per-tick overhead.

`verdict=SURVIVING_SUPERQUADRATIC_TERM`

**This corrects i294's attribution**: the top of the envelope is **not** on the
Ω(N²) relationship-matrix floor. There is a surviving term of roughly N^2.7–2.9
that decides the envelope, and it is of the accident class i294's fixed
emotion-regulation scan belonged to (an O(N²) loop with O(N) inner work).

## The hypothesis tested — and refuted

The obvious suspect was the per-event trust reads in the gossip/knowledge pass:
four `self.relationships.iter().find(|r| r.from == from && r.to == to)` sites, each
O(R) = O(N²), inside a loop over O(N) events per tick — i.e. O(N³)/tick.

Landed a fix: a dense `(from·n + to) → relationships position` lookup
(`Simulation::rel_lookup`, built once per pass with `rebuild_rel_lookup()`, read
through `rel_pos()`), replacing all four scans. It is **behaviour-identical**
(golden 9/9 byte-identical; a new pin asserts `rel_pos` equals a linear scan for
every pair at populate *and* after 3 000 ticks, where the revalidation fallback
covers births/deaths that moved the matrix).

**Measured payoff: ~4% at N=192** (24 842.8 → 23 939.2 µs/tick; N=144 11 385.8 →
10 804.0). The hypothesis is **refuted as the dominant term**: the scan
early-exits (matching rows sit early in the matrix for low `from`), so its real
cost is far below the O(N³) bound.

The lookup is **kept despite the small payoff** because it is provably
value-identical, removes an O(N³)-shaped access pattern from the hot path, and
removes the pattern from future reach — not because it fixed the exponent
(recorded plainly so no one mistakes it for the answer).

## What remains

The ≈N^2.8 term is **localized but not identified**. Candidate classes to test
next, in order:

1. Events × per-event O(N) work *other than* the relationship scans (the event
   volume itself is linear — α_volume = 0.975 — so it is per-event cost).
2. Per-tick O(N²) pair loops whose bodies carry O(N) work — e.g. the group
   formation peer scan in `household.rs` (O(N²) with a per-agent `Vec` allocation
   per tick).
3. Fixed per-tick passes that scale with a population-derived quantity rather
   than N.

## Verification

- **Golden 9/9 byte-identical** before and after the lookup change; new
  `rel_pos_matches_linear_scan_including_after_population_change` pin;
  `scripts/gate --full` **GATE GREEN** (309/0/1), zero re-anchors, no snapshot drift.
- `cargo fmt` clean, clippy 0 warnings (one `checked_conversions` hit on the new
  code fixed, not silenced).

**Probe:** `cargo run -p mindstrata-benches --release --example i329_local_exponent`
