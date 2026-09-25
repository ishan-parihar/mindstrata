# i426 — belief-channel surfaces: formation-time vs steady-state

**Iteration:** i426 · **Date:** 2026-09-25 · **Status:** MEASURED — i403 §6 precondition 1 ANSWERED
**Probe:** `crates/mindstrata-benches/examples/i426_belief_formation_surface.rs`
**Corpus:** pestilence (the i388 revolution family {5, 42, 12345}), 20 000 ticks, sampled every 1440 ticks on the CURRENT tree (channels read v1); the v2 store exists simultaneously, so both surfaces are measured in one run.

## 1. What was asked

i403 §6, precondition 1: is the honest-surface belief deficit **formation-time**
(the founding ecology forms while the dyadic store still climbs from its
stranger prior, and the scar persists) or **steady-state** (the dyadic store
sits below the channel bars permanently)? The answer decides whether the
writer-deletion re-attempt needs an early-world ramp repair or a channel-law
re-derivation.

## 2. Method

Per sampled bucket, over **contacted ordered pairs** (v2 `interaction_count >
0` — the pairs the channels could plausibly see):

- **Diffusion acceptance** (§19.5.I site, `social_cluster.rs:893-915`):
  `trust×0.5 + recipient.cultural.openness×0.5 ≥ 0.5`, computed per surface.
- **ToM Friendly band** (`infer_intent`, `social_cluster.rs:650,669`):
  `trust > 0.5`, per surface.
- **Gossip confidence** (`process_gossip`, `social_cluster.rs:726-737`):
  reads the rumor, source trust, both personality structs and the sender's
  emotions (the channel's internal weighting was not read for this probe);
  continuous, no bar at the call site — reported as the trust mean per
  surface.
- **Belief stock**: per-agent `beliefs` count, mean `confidence`, holder count
  and mean `emotional_charge` for panic propositions 0/1.
- **Formation cohort**: first-seen tick per distinct (agent, proposition)
  (beliefs carry only `last_reinforced_tick`, so formation is tracked by the
  sampler itself, bucketed into horizon quarters).

**Phase caveat (named, per the v1↔v2 sync contract):** i376's daily sync is
full (coupling 1.0), so v1 == v2 at sync boundaries. Samples land at
start-of-tick t = 1440k, i.e. 144 intra-day ticks after the last sync, so the
v1 column carries one day of ratchet interaction gains on top of v2. The
v1>v2 per-pair share (seed 5: 55.6% → 17.5%; 42: 31.8% → 25.8%; 12345:
38.2% → 5.0% over the run) measures exactly that intra-day inflation decaying
as v2 climbs — not a persistent surface gap.

## 3. Measured series

Gate-clearing shares (%) per contacted pair, first and last bucket:

| seed 5        |  t=1440 | t=18 720 |
|---------------|---------|----------|
| KD v1 / v2    | 94.4 / 48.9 | 98.3 / 83.3 |
| ToM v1 / v2   | 38.9 / 48.9 | 52.5 / 83.3 |
| v1mean / v2mean | 0.654 / 0.655 | 0.747 / 0.818 |

| seed 42       |  t=1440 | t=18 720 |
|---------------|---------|----------|
| KD v1 / v2    | 95.3 / 67.1 | 97.5 / 80.8 |
| ToM v1 / v2   | 45.9 / 67.1 | 59.2 / 80.8 |
| v1mean / v2mean | 0.687 / 0.727 | 0.771 / 0.842 |

| seed 12345    |  t=1440 | t=18 720 |
|---------------|---------|----------|
| KD v1 / v2    | 97.1 / 58.8 | 98.8 / 93.8 |
| ToM v1 / v2   | 36.8 / 58.8 | 38.8 / 93.8 |
| v1mean / v2mean | 0.671 / 0.719 | 0.668 / 0.829 |

Belief stock: 36→42/45/36 beliefs; p0/p1 holders 12–15 throughout
(near-universal panic-proposition coverage); p1 mean charge oscillates
0.14–0.46 — sub-floor against the i381 trigger bar `max(baseline×1.25, 0.47)`
at most buckets on all seeds, consistent with i381's recorded knife-edge.

Formation cohorts (distinct (agent,prop) first-seen):

| seed | [0,5K) | [5K,10K) | [10K,15K) | [15K,20K) |
|------|--------|----------|-----------|-----------|
| 5    | 36     | 3        | 3         | 0         |
| 42   | 39     | 0        | 3         | 3         |
| 12345| 36     | 0        | 0         | 0         |

## 4. Verdict — **formation-time**, with a smaller persistent residual

1. **The founding ecology forms entirely inside the deficit window.
[0,5K) carries 36–39 of 36–45 distinct (agent,prop) beliefs; at t=1440 the
v2-reading diffusion gate clears 48.9–67.1% of contacted pairs vs v1's
94.4–97.1% — a ~30–48-point gap exactly while the founding stock forms.**
Pre-existing (genesis-seeded) beliefs are immune; the deficit bites the
beliefs that must be *written during* the ramp.
2. **The bars are calibrated against the v1 spawn draw, not a signal.**
`populate` constructs a DENSE ordered-pair v1 matrix with `trust =
U(0.3,0.7)` per pair (`sim/population.rs:774-779`): half the graph sits above
0.5 by founder lottery before any interaction. v2 instead starts at its
stranger prior and earns pair-by-pair. So `> 0.5` on v1 means "the lucky half
of the spawn draw, plus odds of interaction gains", and on v2 it means
"earned" — the same constant is a different contract on each store.
3. **Steady state converges upward but not fully**: v2 reaches 81–94% late
while v1 sits at 97–99% resident. A residual gap persists; it is much
smaller than the founding-window gap and is dominated by the intra-day
ratchet phase noted in §2.
4. **One i403 candidate channel refuted**: the panic charge law is
distress-driven (daily fold: `0.075·distress − 0.05·charge`, norms_impl.rs:
1149-1163) and applies only to HELD propositions — it is NOT trust-gated.
Its carrier coverage is near-universal (12–15/12–15), so the panic route's
input to re-derive is the *formation/charging eligibility of propositions 0/1*,
which runs through the diffusion channel, not a panic-side trust floor.
5. **Dead field found (i391 class), recorded as its own row**:
`cultural.openness ≡ 0.500` (min=max=mean) on all three seeds — never written
in `populate` (only `personality.openness` is drawn). The diffusion gate's
openness leg therefore contributes nothing; diffusion acceptance ≡ `trust ≥
0.5` ≡ the ToM band in production. Two names, one signal. Repair (wire it to
the psyche openness, or drop the term) is a separate iteration — not bundled.

## 5. Consequence for the re-attempt (i403 §6 precondition 2 input)

On the honest surface, an absolute `> 0.5` trust bar asks "has this pair
out-earned the stranger prior" — which is false for EVERY pair at founding
time, so the ramp starves exactly the founding ecology. The re-derivation
per channel (next iteration) should make the bar **distributional**, not
absolute:

- Option A (preferred, i381 pattern): an anomaly/percentile bar over the
  contacted graph's own v2 distribution (e.g. source within the top quantile
  of that pair's own v2-trust history or the population's live baseline),
  sized by the calm/crisis gap method from the measured must-fire /
  must-not-fire plateaus.
- Option B (conservative): anchor the bar to the measured v2 prior curve
  (e.g. mean v2 trust on contacted pairs at that agent's own formation tick).
- In both cases the i403 stashed arc (`stash@{1}`) re-applies AFTER the bars
  are re-derived, with the degree re-point as required companion and the
  witness channel left on v1.

Before any edit per channel: measure that channel's actual input distribution
on v2 in the channel's own event stream (the KnowledgeAdded / InteractionOccurred
journalled events), not just the contacted-graph snapshot — the gate census
must match the predicate's own scope (§4.12).

## 6. Instruments

`i426_belief_formation_surface` (this repo). Re-run:
`cargo run --release -p mindstrata-benches --example i426_belief_formation_surface`.
