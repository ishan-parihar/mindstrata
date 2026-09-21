# i344 — A9 sized: the constant-density world is a fidelity policy, not a perf lever

**Question.** i340 closed the housing half of the Ω(N²) story and measured the remaining
limit: at the charter's **fixed 32×32** world the contact graph re-saturates past N≈144
(touched-R α 2.654, the best-connected agent has met everyone). It recommended defining the
envelope at **constant density** — world area growing with N — which i339's probe had measured
at touched-R α 0.898. That recommendation was left as a charter decision. Is it also a
**performance** fix?

**Instrument.** `crates/mindstrata-benches/examples/i344_world_area_law.rs` — the same
population run at two world policies, seed 42:

- **fixed** — the charter's 32×32 for every N;
- **density** — `side(N) = max(16, ceil(sqrt(21.333 · N)))`, a law anchored on the simulator's
  own two calibrated points (N=12 in 16×16 and N=48 in 32×32 *both* give 21.33 cells/agent),
  so it reproduces both exactly and holds that density above: N=48→32, 96→46, 144→56, 192→64.

Cost legs use **min of three independent processes** (the i332 charter method) because i337
measured whole-tick A/B at ±5–10% between processes — and this probe's own single-window
readings disagreed in sign at N=192 (fixed 3 795 vs 4 776 µs/tick across two runs).

## Result 1 — no tick-cost win (A9 is NOT a perf lever)

| N | fixed 32 | density | Δ |
|---|----------|---------|---|
| 96 | 1 053.0 | 1 148.7 | **+9.1%** |
| 144 | 2 298.6 | 2 244.9 | −2.3% |
| 192 | 3 734.6 | 3 942.3 | **+5.6%** |

Growth **volume per agent is flat under both policies** (361 → 360 contacts/agent, N=48 → 192),
so interaction work is not the superlinear term. What the world policy changes is the *contact
state*, not the *store*: `rows = N(N−1)` in both, and the per-edge passes (i341: 58% of the
tick) walk rows. A bigger world therefore buys nothing on cost and adds area-driven work.

**A9 refuted as a performance lever** — the same shape as i338's sparse-store refutation, and
for the same reason: the dominant cost is the complete row set, not contact density.

## Result 2 — but it is a large **fidelity** effect

| N=192 | fixed 32 | density 64 |
|-------|----------|------------|
| max agents co-located | **19** | **4** (the declared `House.capacity`) |
| near-pair share (Manhattan ≤5) | 11.3% | **4.7%** |
| contacted-row share | 11.5% | **5.3%** |
| mean partners/agent | 22.1 | **10.2** |
| contacted α (144→192) | 1.276 | **0.866** |
| house pairs within radius 5 | 8.4% | **3.3%** |
| population / avg health / stress @2K | 192 / 0.781 / 0.241 | 193 / 0.799 / 0.240 |

At the fixed world the ring degenerates: `ring_span = min(w,h)/2 − 2 = 14` regardless of house
count, so 48 houses get an angular stride of ~1.8 tiles — **min pairwise house gap 1**, houses
effectively adjacent, and agents pile 19 to a cell (nearly 5× the declared capacity of 4, which
i340 could only claim "respected" up to N=96). Density keeps min-co at capacity and halves the
contact state.

## Verdict

**`WORLD_AREA_IS_A_FIDELITY_LEVER_NOT_A_COST_LEVER`.**

- **A9 re-scoped**: do not ship a bigger default world expecting throughput — measured cost is
  +5.6% at N=192. Its honest value is spatial realism and capacity compliance at N ≥ 144
  (max co-location 19 → 4), plus halving the contact state a future sparse store would hold.
- **The scale ledger tightens**: with contact volume flat (i344), contact state halved by every
  spatial lever (i340, i344) but *rows* untouched, the only remaining fix for the tick's
  dominant per-edge cost is **shrinking the row set** — i344's originally queued sparse store.
  Neither world size nor contact density can substitute for it.
- **New item (A11) — the housing ring degenerates in a fixed world.** `ring_span` is a function
  of world size only, so above ~29 houses the angular stride collapses and houses sit 1 tile
  apart. This is measurable at a fixed world *without* changing world size (an area packing
  instead of a ring), which is the cheaper half of A9 and is queued as its own iteration.

## Reproduce

```
MINDSTRATA_PROFILE_TICK=1 cargo run --release -p mindstrata-benches --example i344_world_area_law
```
