# i429 — dyadic affection write-side saturation (probe-only)

**Iteration:** i429 · **Date:** 2026-09-25 · **Status:** MEASURED — repair candidates recorded; no code change
**Probe:** `crates/mindstrata-benches/examples/i429_affection_saturation.rs`
**Chain:** i402 measured the kind schedule's `affection > 0.70` branch firing 98–99% on both stores (§4.10 defect); i403 v1.1's instrument locked the channels' own scope; i428's comment keeps the schedule on v1 pending this exact write-side measurement.

## 1. The write-side law (verified in code, `relationship_v2.rs`)

- Stranger prior: `affection = 0.3` at row creation (`new`, :410).
- Per positive act: `affection += magnitude × volatility × 0.015` (526-543), clamped to 1.
- Per negative act: `affection -= magnitude × volatility × 0.02` (549-555).
- Decay: TWO legs (`cognitive.rs:800-812` + `relationship_v2::decay`): every
  row decays once on the daily boundary (0.0002 × 1 tick × 0.5 for affection
  − a hypothetical ~0.0001/day), and **dirty** rows (any row written by an
  interaction since the last boundary) decay EVERY tick while dirty — so a
  pair talked to once that day also loses 0.0144/day to the dirty leg. Rows
  never touched (contacted but cold) lose only the universal 0.0001/day —
  the dormant drain is essentially off.

Arithmetic consequence on a pair: 1 positive act/day → net ≈ +0.0004/day
(creeps to 1.0 forever); 3+ acts/day → ~+0.03/day (ceiling in ~23 days ≈
3 312 ticks). No headroom taper, no stalling term except the ceiling clamp,
and a dormant drain too weak to pull any written pair back.

## 2. Measured trajectory (both corpora)

calm seeds {42, 7, 11} 16×16/N=12 riverford-class, 20K ticks, contacted-pair
dyadic affection per 1440-tick bucket:

| t | n pairs | p10 | p50 | p90 | ceiling share | discriminating band [0.5, 0.95) |
|---|---|---|---|---|---|---|
| 1440 | 71–87 | 0.41 | 0.74–0.95 | 1.0 | 20–61% | 34–44% |
| 5760 | 96–102 | 0.54–0.83 | 0.99 | 1.0 | 42–50% | 13–18% |
| 12960 | 104 | 0.98 | 0.99 | 1.0 | 32% | 5.8% |
| 18720 | 104 | 0.985 | 0.995 | 1.0 | 35% | 1.9% |

The discriminating band [0.5, 0.95) is a **transient birth canal**, not an
equilibrium: pairs enter near the 0.3–0.5 prior, pass through the band in a
few days of contact, and pin at ≥ 0.99 permanently (negative acts are too
rare — under 2% of interactions — to hold the ledger). By 13K ticks the whole
population's p10 is 0.981: the store retains essentially ONE state.
The crisis arm (pestilence/42) shows the same shape with slightly slower
band drain (adversarial interactions subtract) — same destination.

## 3. Verdict (informs the repair, no commit yet)

`affection > 0.70` at the kind schedule sits inside the transient canal; the
moment any two agents have a couple of contacts they are above it forever.
The branch therefore cannot discriminate — and as §4.16(a) anticipated, NO
re-anchored constant can fix a store whose measure is a point mass at the
ceiling. The repair must be write-side, with the candidates, in scope order:

1. **Headroom-tapered gain**: `gain × (1 − affection)` — the organic "no
   pair deepens at a constant rate forever" form; equilibrium set by
   gain/decay, near 0.8–0.9 if sized to preserve today's discrimination
   window shape on the calibrated corpora.
2. **Maintenance decay on ALL contacted pairs** (not only dirty ones):
   a bond unwarmed decays; the dirty-only rule means a *talking* pair can
   never cool — arguably the bug itself (today cold rows decay on the daily
   pass only if they left the dirty set).
3. **Kind-weighted gains**: WarmContact/Help charges the bond, Talk charges
   far less — the relay's qualitative claim ("a favour is not a chat").

The honest sizing requires the pin corpus' tolerance envelope: the NOT-landed
piece is the choice between (1) adding headroom taper (one constant + a
multiplication at one site) vs (2) extending the decay pass. The decision
instrument is a sweep over the same four corpora used by i425–i428 (riverford
golden, collapse golden, calm snapshots, the pestilence family) — sized so
that (a) the discrimination band survives at 20K (share stayed ≥ ~10-30%),
(b) the golden calibration windows produce the same behavioral artifact
classes, (c) no pin's producer starves.

## 4. Instruments

`i429_affection_saturation` (this probe) — reuse with the two candidate arms
applied in-tree to compare trajectories before any commit.
