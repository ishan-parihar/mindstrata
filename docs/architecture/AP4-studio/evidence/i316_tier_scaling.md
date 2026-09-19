# Iteration 316 — tier/scaling audit: the Background tier is unreachable

**Status:** LANDED (measurement iteration, **no behaviour change**) · **Root
cause owned:** the §17 scalability strategy. `docs/PLAN_DC3_DEVELOPMENT.md`'s
standing scale-debt row named the dead `AgentTier::runs_action_selection()`
gate; this audit measures the whole tier mechanism and the cost curve.

§17 classifies agents into Focal / Secondary / Background, each running less, so
that large populations cost less per agent than small ones. This probe asks
whether that machinery is actually scaling the sim.

## Finding 1 — `Background` is unreachable by construction

Tier census and an ever-Background count (`i316_tier_scaling`, seed 42):

| N | ticks | Focal | Secondary | Background | ms/tick |
|---|---|---|---|---|---|
| 12 | 500 | 6 | 6 | **0** | 0.09 |
| 24 | 500 | 11 | 13 | **0** | 0.21 |
| 48 | 500 | 19 | 30 | **0** | 0.70 |
| 96 | 500 | 32 | 64 | **0** | 3.92 |

Over 10K-tick runs at N=12 and N=48 the Background tier is held for **0
agent-ticks**, and the minimum `narrative_importance` ever observed is **0.3000**.

The mechanism is arithmetic, not luck: the Secondary→Background edge requires
`narrative_importance < 0.1` (`agent_tier.rs`), but
`update_narrative_importance`'s target carries a **base term of `+ 0.15`** plus
non-negative role/network/emotion/event bonuses. The reachable floor is
therefore ≥ 0.15 in the best case (0.30 measured across real runs) — strictly
above the 0.1 entry threshold. The "aggregate simulation" tier the scaling
strategy is built around can never be entered.

## Finding 2 — two tier predicates have zero production call sites

| predicate | production call sites |
|---|---|
| `runs_full_psychology` | 6 |
| `runs_memory_encoding` | 4 |
| `runs_theory_of_mind` | 2 |
| `runs_heuristic_cognition` / `runs_prospection` / `runs_narrative` / `runs_relationship_updates` / `runs_belief_updates` / `runs_social_interactions` | 1 each |
| **`runs_full_biology`** | **0** |
| **`runs_action_selection`** | **0** |

So the biology pass and action selection run for **every** agent regardless of
tier. §17 specifies Secondary runs "simplified biology" — but no reduced biology
path exists, so wiring `runs_full_biology` as-written (Focal-only) would strip
biology from the *majority* of agents (Secondary is 64/96), not just
Background. The gap is a missing mechanism, not a missing call.

## Finding 3 — cost is superlinear, on the documented floor

ms/tick at 500 ticks: `0.09 → 0.21 → 0.70 → 3.92` over N = 12→24→48→96.
Log-log exponent ≈ **1.8** across the range (48→96 alone is 2.5). This is
consistent with i294's `α_total = 2.115` (2000-tick windows) and its attribution:
the **Ω(N²) dense legacy relationship matrix** (`R = N(N−1)`; 36 672 rows at
N=192) is the structural floor, and i294 already removed the one O(N³) accident.
N=192 at 20K ticks did not complete inside a 600 s budget on this host, which is
the practical face of that floor.

## Disposition (recorded, not smoothed)

This is a **scaling-readiness gap**, not a current correctness bug: the operating
envelope is N ≤ 96, where Focal/Secondary is a genuine differentiation gradient
(6/6 at N=12, 32/64 at N=96) and the law-clean gates are green. No pin is
re-anchored and no threshold is widened.

The levers, in the order the evidence supports:

1. **Sparse relationship store** — i294's deferred item; the direct attack on the
   Ω(N²) floor (cap the neighbourhood like `gossip::MAX_GOSSIP_EDGES`).
2. **A Secondary-reduced biology path** — the honest way to make
   `runs_full_biology` live; a mechanism to build, not a call to add.
3. **Making the Background tier reachable** — requires either lowering the
   importance base term or raising the `0.1` entry threshold, both of which
   change tier assignment and therefore re-anchor behaviour broadly; deferred as
   its own behavioural iteration rather than folded into a measurement commit.

`runs_action_selection` is Background-only in predicate terms, so wiring it
would be behaviour-neutral today (Background never occurs) and live only once
(3) lands — not worth a dead call site in the meantime.

**Zero drift:** no behaviour change, golden byte-identical, no re-anchors.
