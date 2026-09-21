# Iteration 346 — the decision census: A8 anatomized, and `Move`'s only producer is gated above reach

**Status:** LANDED (measurement + instrument) · **Root cause owned:** i338 measured
`Wander`/`Move` at **0.00%** of agent-ticks and recorded §6 locomotion as inert (ledger
A8), but a share cannot distinguish *the branch never runs* from *the branch runs and the
candidate loses*. Those two failures have different fixes — §2.3 revival versus §4.4
re-contract — and choosing between them was blocked on a measurement that did not exist.

## What landed

1. **`sim::decision_census`** — an opt-in, default-inert sink (the i330/i335 profiler
   pattern: process-global, never serialized, never read by a pass). It records, once per
   **selection decision**, the deciding source (reflex / command / routine / feud /
   utility / habit / veto), the finalized action, the source×action cross-tab, and — on
   the utility leg — the realized `winner − candidate` utility gap for the two candidates
   that carry no meaningful relief term (`Wander`, `Idle`). Enabled by
   `decision_census::enable()` or `MINDSTRATA_DECISION_CENSUS=1`; read via
   `report()`; cleared via `reset()`.
2. **`pass_action` records its provenance** — the five-deep branch chain now carries a
   source tag through its two post-transforms (habit substitution tags only when it
   actually changed the action; the exertion veto tags last). A pure refactor: the
   quick gate's golden 9/9 stayed byte-identical.
3. **Probe `i346_decision_census`** — census, feud-reachability, position-update and
   inertness legs, plus two pins (`mindstrata-sim` 294/294).

## Exit evidence (seed 42, 32×32, 500 warmup + 20 000 ticks)

**A decision is not an agent-tick.** The pass only re-decides when the previous action
finished (`action_progress == 0`), so 240 000 agent-ticks at N=12 hold **42 813
decisions** (17.8%; mean action duration 5.61 ticks). Every share below is a share of
decisions.

| source | N=12 | N=48 |
|---|---|---|
| routine (§10.3 ladder) | 25 342 (59.19%) | 107 202 (54.38%) |
| **utility AI** | 14 024 (32.76%) | 75 033 (38.06%) |
| habit (§8.1.19 substitution) | 3 341 (7.80%) | 14 681 (7.45%) |
| reflex (i255/i306/i309) | 34 (0.08%) | 156 (0.08%) |
| veto (i314 pain) | 72 (0.17%) | 74 (0.04%) |
| command / **feud** | 0 / **0** | 0 / **0** |

Live by construction, confirmed live by measurement: the physiological reflex layer, the
habit fallback and the pain veto all fire. The **utility AI is not a dead layer** — it
decides a third of all actions — which is what makes the next table a finding rather than
an artifact.

| final action | N=12 | N=48 |
|---|---|---|
| Work | 16 995 (39.70%) | 63 639 (32.28%) |
| Rest | 8 776 (20.50%) | 34 399 (17.45%) |
| Trade | 6 400 (14.95%) | 52 154 (26.45%) |
| Eat / Drink | 3 533 / 3 448 | 22 428 / 13 243 |
| Socialize | 2 916 (6.81%) | 10 286 (5.22%) |
| Worship | 745 (1.74%) | 997 (0.51%) |
| **Wander / Move / Idle** | **0 / 0 / 0** | **0 / 0 / 0** |

### The utility arbitration, sized

| candidate | N=12 gap (mean / max) | N=48 gap (mean / max) | within ±0.05 of winning |
|---|---|---|---|
| `Wander` | 1.6254 / 5.0787 | 1.5139 / 5.5063 | **0 of 107 085 losses** |
| `Idle` | 1.3540 / 4.8160 | 1.2253 / 5.2443 | **0 of 107 085 losses** |

**Both are structurally dominated, not mis-calibrated.** The per-candidate jitter is
±0.05, so a candidate within 0.05 of the winner is one lucky draw from winning; `Wander`
and `Idle` lose by **1.2–1.6 on average — 25–30× the entire jitter band — and were never
once inside it.** No coefficient tweak reaches them: `Wander` is imported with **zero**
relief on every channel (pinned by `wander_carries_no_need_relief_at_all`), and `Idle`'s
only relief is 0.05 fatigue per tick, which the village's always-pressing needs
out-compete. Reviving either is a **design** act (give locomotion a driver), not a
re-anchor — the i306 class, now quantified.

### `Move`'s only producer is gated above its reachable range

`Move` has exactly one source in the whole selection chain: the §19.5.G feud-approach
branch, gated on `!feuds.is_empty() && anger > 0.4 && hunger < 0.85 && thirst < 0.85`.

| | N=12 | N=48 |
|---|---|---|
| non-empty `feuds` | 5 486 / 24 000 agent-ticks (22.86%) | 12 777 / 96 000 (13.31%) |
| `anger > 0.4` | **0** | **1** (of 96 000) |
| both (the gate) | **0** | **0** |
| max `anger` observed | 0.2786 | 0.4095 |

The feuds are there; the **anger is not**. The §19.5.G branch is dead by construction
because its threshold sits *above* the reachable anger range — 0.4 is a knife-edge at
N=48 (max 0.4095) and unreachable at N=12 (max 0.2786). That is a §4.3 dead producer
with a spec contract behind it, and a cheap, surgical fix.

### What actually moves an agent

`Wander`/`Move` are never selected, so the §6 movement pass never fires. Position changes
still occur — from a *fourth* position writer the i338 audit did not name: the §10.4
courtship "seek proximity" walk (Iteration 118), one deterministic Manhattan step per day
for a pursuing agent beyond perception radius. Measured: N=12 **0 changes / 2 000 ticks**;
N=48 **61 changes / 2 000 ticks** from 6/48 agents, every one a single step taken while
the agent's `current_action` was `Rest` or `Work` (thus provably not the action-driven
movement pass). Space is decorative not because movement is broken but because the only
two actions that trigger it cannot win an arbitration.

## Inertness

The census draws no RNG and writes no simulation state, and that is pinned rather than
asserted: the same configuration run with the census on and off produces identical
fingerprints (agent/event/journal counts, tick, and a fold over positions, hunger, thirst
and health) at both N. 294/294 sim tests, golden 9/9 byte-identical, `scripts/gate`
GREEN, **zero re-anchors**.

## Ledger effect

- **A8 re-scoped**: not "a share is 0.00%" but "`Wander`/`Move`/`Idle` are structurally
  unreachable — `Wander` has no relief term at all (gap 1.5, never within 0.05), `Move`'s
  sole producer is gated above the reachable anger range". Two distinct fixes, both
  behavioural, both now sized.
- **New item A12 — the §19.5.G anger gate is above its reachable range** (`anger > 0.4`
  versus max 0.2786/0.4095). The cheap half of A8: the branch exists, the semantics are
  in the spec, and the fix is a threshold/relative-signal decision with a measured
  distribution in hand.
- **A9 residual noted**: the courtship proximity walk is the only live locomotion, so the
  housing/world-area work (i340/i344/i345) shapes *initial* contact only.
- **Retired suspicion**: the reflex, habit and veto layers are all live (0.08–7.8% of
  decisions), so they are not candidates for revival.

**Disposition:** measurement + instrument, landed green. i347 takes A12 with this
distribution as its evidence, per §2 (one root cause) — the census also becomes the
acceptance instrument for that fix: the `feud` row must move off zero.
