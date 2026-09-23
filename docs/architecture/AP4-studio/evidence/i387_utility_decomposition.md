# i387 — the utility decomposition: which family of terms decides an arbitration?

**Status:** LANDED (instrumentation; behaviour unchanged) · **Scope:** the action-selection
utility is now computed as a **named bucket vector** (`UT_NEED`, `UT_URGENCY`,
`UT_LEARNED`, `UT_POLICY`, …) instead of an opaque sum, and the decision census records
the winner's, the runner-up's and the `Socialize` candidate's vector for every
arbitration.

## Why

i380 closed with a *quantity* ("`Socialize` wins 0 of 112 669 arbitrations") and two
refuted single-knob repairs (×6 gain: 0→0; ×20 need accrual: 1→9). It could not say
**which family of terms** decides an arbitration, so it could not name what the social
candidate was missing rather than merely under-weighting. DC-5's W0 puts this census
first for exactly that reason: a deficit concentrated in buckets the candidate has *no
term for* is a missing-channel finding (fix = give it the channel); one concentrated in
`need` is a calibration finding (fix = re-derive the coefficient).

## Instrument

- `actions::compute_utility_terms` returns the labelled vector; `compute_utility` is a
  thin sum wrapper (identical arithmetic — `Fixed` addition is exact), so the refactor is
  behaviour-preserving and the census can never drift from the real selection math.
- `decision_census::{record_utility_terms, report}` accumulates winner / runner-up /
  `Socialize` vectors plus the **dominant motive** (`MotiveCategory` argmax) per
  arbitration. Disabled by default; `census_records_decisions_without_changing_them`
  guards the inertness.
- Probe: `crates/mindstrata-benches/examples/i387_utility_decomposition.rs`.

## Measured (warmup 500, window 20 000, before the i388 fix)

| bucket | village N=12 winner | socialize | town N=48 winner | socialize |
|---|---|---|---|---|
| need | 0.2089 | 0.0015 | 0.2650 | 0.0009 |
| urgency | 0.0531 | **0.0000** | 0.0663 | **0.0000** |
| trade | 0.1088 | 0.0000 | 0.1884 | 0.0000 |
| driver | 0.0913 | 0.0000 | 0.0948 | 0.0000 |
| goal | 0.0618 | 0.0000 | 0.0796 | 0.0000 |
| policy | 0.3774 | 0.3109 | 0.3420 | 0.2757 |
| learned | −0.3045 | −0.4620 | −0.3134 | −0.4612 |
| **total gap** | — | **0.7315 mean loss** | — | **0.8912 mean loss** |

`Socialize`: **0 wins, 0 within-noise** in 120 222 arbitrations across both worlds
(13359 more in a pestilence town, gap 0.9588).

**Dominant motive** (the arbitration's own argmax): calm village — `Attachment` 34.6%,
`Safety` 25.3%, `Sleep` 21.1%, `Thirst` 13.4%, `Belonging` 5.5%; town — `Sleep` 34.6%,
`Attachment` 24.6%, `Thirst` 19.9%, `Safety` 17.2%, `Belonging` 3.6%.

## The finding this instrument produced

1. **The candidate is not under-weighted, it is miss-channeled.** `policy` — the one big
   term it *has* — is nearly the winner's (0.31 vs 0.38). The deficit sits in terms it
   has **no** channel for: `urgency` (0.053/0.066), `trade` (0.109/0.188), `goal`
   (0.062/0.080), `driver` (0.091/0.095).
2. **Two fifths of the deliberative layer was asked to serve a drive with no response.**
   `Attachment` + `Belonging` dominate 40.1% of village arbitrations (28.9% in the town)
   and the §8.1.5 urgency map — `Hunger/Thirst/Sleep/Meaning/Novelty/Play` only — sent
   every one of them through `_ => false`. That is the *exact* dead-dominant-motive class
   i351 closed for `Novelty` and i356 for `Play`, measured at twice their size.
3. The candidate's `learned` term is structurally depressed (−0.46 vs −0.31): the RL
   components EMA-converge toward the profiles of *executed* actions, and `Socialize` is
   never executed via the utility leg — a self-reinforcing withdrawal lock. Recorded, not
   patched: the fix would be exploration, not a coefficient.
4. The utility leg still cannot cross the bar (0.73–0.96) on need terms alone — the i380
   conclusion holds and is now explained bucket by bucket.

## Disposition

i388 (same arc) wires the two missing channels: the **relational urgency family** and the
**population-relative relational goal band**. The `needs.social` utility term stays as it
is — measured below the bar, with the reason recorded at the call site.
