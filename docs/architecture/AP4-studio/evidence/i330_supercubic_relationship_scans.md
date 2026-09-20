# Iteration 330 — the ≈N^2.8 term: four accidental relationship scans, removed

**Status:** LANDED · **Item owned:** i329's open question — "the ≈N^2.8 term is
localized but not identified". Answer: it was **four more instances of the same
accident class** i329 had already found once and refuted as non-dominant.

## Method — attribute by pass, then by sub-pass

i329 measured the exponent but could not say *where* the cost sat. Added an
**opt-in per-pass profiler** (`Simulation::pass_profile_tick()`, driven by
`MINDSTRATA_PROFILE_TICK=<n>`, parsed once via `OnceLock`): a `mark!` macro
stamps nanoseconds per pass and prints to stderr for exactly one tick. Off by
default and costing one `Instant::now()` per tick when off. Left in place — the
next scale question asks the same question. `social_pass` additionally
sub-profiles into `+interactions` / `+speech_acts`.

Probe: `cargo run -p mindstrata-benches --release --example i330_pass_profile -- 192`
(tick 400 = a `deca` boundary at every N, so samples are comparable).

## What the profile found

At N=192, before the fixes (working tree carrying only the trust-sync fix):
`social_pass` **10 022 900 ns — 41% of the entire tick**, the largest term.

Sub-profiling put **2 667 719 ns of it inside `system_social_interactions`**, and
the pass' own split confirmed it: the interaction engine was the cost, not the
speech-act wiring above it.

Four scans, all `relationships.iter()/iter_mut().find(..)` — each **O(R)=O(N²)**
— run *per interaction* (`I ≈ c·N` of them per tick):

| Site | Shape | Per-tick bound |
|---|---|---|
| `update_witnesses`, per witness (O(N) witnesses) | `iter_mut().find(..)` | O(I·N·R) ≈ **O(N⁴)** |
| `system_social_interactions`, `trust` read | `iter().find(..)` | O(I·R) = O(N³) |
| `system_social_interactions`, `affection` read | `iter().find(..)` | O(I·R) = O(N³) |
| `process_interaction`, both directions | `iter_mut().find(..)` ×2 | O(I·R) = O(N³) |

i329's refutation ("the scans early-exit, so only ~4%") held for *those* four
sites — the gossip/knowledge reads — because those match rows that sit early in
the matrix. These four sit behind a **proportional-to-N witness loop** and behind
*every* interaction, so no early exit rescues them.

## The fix — the same lookup, threaded into the social pass

`Simulation::rel_lookup` (built by `rebuild_rel_lookup_into`, the split-out body
of i329's `rebuild_rel_lookup`) is now threaded into `tick_social_pass` →
`system_social_interactions` → `update_witnesses` / `process_interaction` as a
`&[u32]` dense `(from·n + to) → position` index. Every site resolves O(1) and
**revalidates** the stored slot against the requested pair, falling back to the
linear scan when the lookup is empty or stale (a mid-tick population change
shifts ids); `relationships` is a **slice** for the whole pass, so positions
cannot move within it.

**First-occurrence semantics.** `rebuild_rel_lookup_into` now records the *first*
occurrence of a pair (before i330 it overwrote, i.e. the *last*) — the element
`iter().find(..)` returns. Goldens are byte-identical under the change, so the
lookup now matches the scans it replaces exactly rather than approximately.

**Trust sync** (same class, outside the social pass): the `trust_sync+reset`
prepass called `TrustNetwork::{trust_for_agent, update_agent_trust}`, each a
linear `Vec` scan per relationship — O(N³)/tick. Replaced by one reusable dense
position buffer per tick (refilled per agent, O(N)), preserving the
first-occurrence and update/append branches exactly.

## Measured result

Exponent probe (`i329_local_exponent`, release, seed 42, 32×32, fast ticks).
Baseline is **HEAD i329 measured by `git stash`**, not cited from the doc:

| N | µs/tick before | µs/tick after | Δ | µs/agent after |
|---|---|---|---|---|
| 48 | 686.4 | 555.0 | −19% | 11.56 |
| 96 | 3 496.4 | 2 441.7 | −30% | 25.43 |
| 144 | 11 365.2 | 6 444.1 | −43% | 44.75 |
| 192 | 24 932.1 | 12 908.2 | **−48%** | 67.23 |

Local exponents: **2.349 / 2.907 / 2.731 → 2.137 / 2.393 / 2.415**.

`verdict=SURVIVING_SUPERQUADRATIC_TERM` → **`QUADRATIC_FLOOR_CONFIRMED`**

The drop grows with N (−19% → −48%), which is what removing an O(N³) term from
an O(N²)-floored tick predicts. i329's question is answered and the verdict it
produced is retired.

Per-pass at N=192, final (top terms):

| pass | ns | share |
|---|---|---|
| memory encoding | 5 726 268 | 45% |
| instit+faction+panic+derived | 2 124 422 | 17% |
| appraisal | 1 610 109 | 13% |
| cognitive | 1 555 277 | 12% |
| social_pass | 481 721 | 3.8% (was 10 022 900 pre-fix) |
| rel_traces | 414 930 | 3.2% |
| trust_sync+reset | 392 229 | 3.0% |

## What the profile says remains (recorded, not chased)

1. **`tick_memory_encoding` — 45% of the tick, O(N·E) by design.** For every
   agent it iterates *this tick's whole event window* computing
   `attention.compute_salience` (agent-state-dependent: needs, affect,
   habituation, bias — no hoist is possible). The principled reduction is §2.4's
   own doctrine: agents perceive within a radius, so attention should be gated on
   the same perception radius the interaction engine already uses — that turns
   O(N·E) into O(N·local) but is a **behavioural** change needing its own probe
   and re-anchor sweep. Queued as i331's candidate, not attempted here.
2. **`instit+faction+panic+derived`** is the next superlinear block (local
   α ≈ 2.9 in this profile's N=96→192 pair); unsplit — needs its own sub-marks.

## Verification

- **Golden 9/9 byte-identical**, `scripts/gate --full` **GATE GREEN** (309/0/1),
  zero re-anchors, no snapshot drift. `cargo fmt` clean, clippy 0 warnings.
- New pin `dense_relationship_lookup_matches_linear_scan` (in
  `social/interaction.rs`): a populated lookup, the empty fallback, and a
  **stale** lookup that points at the wrong pair must all produce identical
  relationship state. `mindstrata-social` 393/393.
- Call-site churn from the two new `&[u32]`/`usize` parameters is confined to the
  sim caller and the crate's own tests (which pass `&[]`, exercising the
  fallback).

**Probes:** `i329_local_exponent`, `i330_pass_profile`.
