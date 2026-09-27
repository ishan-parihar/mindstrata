# i429 — dyadic affection write-side saturation (probe-only)

**Iteration:** i429 · **Date:** 2026-09-25 · **Status:** MEASURED — repair candidates recorded; no code change
**Probe:** `crates/mindstrata-benches/examples/i429_affection_saturation.rs`
**Chain:** i402 measured the kind schedule's `affection > 0.70` branch firing 98–99% on both stores (§4.10 defect); i403 v1.1's instrument locked the channels' own scope; i428's comment keeps the schedule on v1 pending this exact write-side measurement.

## 1. The write-side law (verified in code, `relationship_v2.rs`)

- Stranger prior: `affection = 0.3` at row creation (`new`, :410).
- Per positive act: `affection += magnitude × volatility × 0.015` (526-543), clamped to 1.
- Per negative act: `affection -= magnitude × volatility × 0.02` (549-555).
- Decay: TWO legs (`cognitive.rs:800-812` + `relationship_v2::decay`), gated by
  `is_active_this_tick` = dirty ∨ daily-boundary (`t % 144 == 0`): **every** row
  decays once per day (−0.0001 affection/day — the cold drain, 144× weaker than
  one act's gain, so a cold contacted pair effectively never cools: THE
  ratchet), and dirty rows (written since the last boundary) decay EVERY tick
  while dirty — a pair contacted that day also loses 0.0144/day to this leg.
  i430's census measured the net: mean −0.00062/row-day across 14 537 row-days.

Arithmetic consequence on a pair: 1 positive act/day → net ≈ +0.0004/day
(creeps to 1.0 forever); at the measured median frequency 41 acts/day →
~+0.10/day (ceiling in ~7 days). No headroom taper, no stalling term except
the ceiling clamp, and a cold drain too weak to pull any written pair back.
The per-seed trajectory table and the four-writer mass census live in
`evidence/i430_writer_census.md` (the per-act writer owns 92% of the mass).

## 2. Measured trajectory (both corpora)

calm seeds {42, 7, 11} 16×16/N=12 riverford-class, 20K ticks, contacted-pair
dyadic affection (per-seed rows; i430 §5 carries the current table — every
seed lands bimodal, band drained to 2–4% by 20K):

| t | n pairs | p10 | p50 | p90 | ceiling share | discriminating band [0.5, 0.95) |
|---|---|---|---|---|---|---|
| 1440 | 71–87 | 0.41 | 0.74–0.95 | 1.0 | 20–61% | 34–44% (per-seed: 42: 33.8 / 7: 43.7 / 11: 58.8) |
| 5760 | 96–102 | 0.54–0.83 | 0.99 | 1.0 | 42–50% | 13–18% |
| 12960 | 104 | 0.98 | 0.99 | 1.0 | 32% | 5.8% (42) |
| 18720 | 104 | 0.985 | 0.995 | 1.0 | 35% | 1.9% (42) |

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
