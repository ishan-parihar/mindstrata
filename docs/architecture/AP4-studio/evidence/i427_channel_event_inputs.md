# i427 — channel event-stream inputs: the gates do not need re-derivation

**Iteration:** i427 · **Date:** 2026-09-25 · **Status:** MEASURED — i403 §6
  precondition 2 (per-channel re-derivation) CLOSED as unnecessary at the
  bar level; the re-attempt's live candidates narrow to the degree re-point
  and the saturated kind-schedule move.
**Probe:** `crates/mindstrata-benches/examples/i427_channel_event_inputs.rs`
**Corpus:** pestilence {5, 42, 12345}, 20 000 ticks; every InteractionOccurred
  event drained per tick from the bounded ring (watchdog `event_count`;
  measured ring loss = **0 events** in all three runs), trust read on both
  stores at the tick the event fired, binned by kind × surface × epoch
  (split at 5 000 — the i426 founding window).

## 1. Which gate actually reads trust (verified at the sites this time)

- **Diffusion** (`social_cluster.rs:880-915`, triggers on Talk/Help/Trade):
  `trust×0.5 + recipient.cultural.openness×0.5 ≥ 0.5`. Production
  `cultural.openness ≡ 0.5` (i426), so the running gate is exactly
  `trust ≥ 0.5`.
- **ToM** (`infer_intent`, theory_of_mind.rs:158-168, call site
  social_cluster.rs:643-669): Friendly ⟺ `observed_helpful > 0.3 && trust >
  0.5`; the call site passes per-event values (Help/Comfort → 0.6, Trade →
  0.3, else 0.1), so **in production the Friendly arm needs a Help/Comfort
  event AND trust > 0.5**.
- **Gossip** (`process_gossip`, gossip.rs:105-137,172-222 — read for the
  first time here): there is **no trust acceptance bar**. Acceptance is
  rumor-salience vs `gossip_acceptance_threshold`; trust enters only via the
  source-fidelity discount `0.9 + 0.08·trust`, so a surface shift of
  Δt ≤ 0.10 moves delivered belief strength by ≤ 0.008. The i426 channel
  inventory's "gossip confidence gate" line was wrong; corrected.

## 2. The event-scoped table

Per-cell: event count n, mean trust, median (p50), and share above 0.5 for
both stores. (Help events feed both Diffusion and WarmContact; counted once
per channel they feed.)

**Seed 5** (loss 0):

| class | epoch | v1 n / p50 / >50% | v2 n / p50 / >50% |
|---|---|---|---|
| Diffusion | ≤5K | 18354 / 0.995 / 97.5 | 18354 / 0.995 / **89.6** |
| Diffusion | >5K | 56572 / 0.995 / 99.6 | 56572 / 0.995 / **98.0** |
| Gossip    | ≤5K | 395 / 0.615 / 95.2 | 395 / 0.655 / 80.0 |
| Gossip    | >5K | 395 / 0.505 / 90.4 | 395 / 0.995 / 73.4 |
| WarmContact | ≤5K | 11570 / 0.995 / 100.0 | 11570 / 0.995 / **99.6** |
| WarmContact | >5K | 53211 / 0.995 / 100.0 | 53211 / 0.995 / **99.7** |
| Other | ≤5K | 209 / 0.855 / 93.3 | 209 / 0.995 / 94.3 |
| Other | >5K | 770 / 0.875 / 97.8 | 770 / 0.995 / 96.2 |

**Seed 42:**

| class | epoch | v1 n / p50 / >50% | v2 n / p50 / >50% |
|---|---|---|---|
| Diffusion | ≤5K | 23456 / 0.835 / 99.0 | 23456 / 0.995 / **89.2** |
| Diffusion | >5K | 69949 / 0.995 / 99.8 | 69949 / 0.995 / **94.7** |
| Gossip    | ≤5K | 2015 / 0.505 / 98.3 | 2015 / 0.815 / 75.7 |
| Gossip    | >5K | 5651 / 0.505 / 99.7 | 5651 / 0.995 / 78.6 |
| WarmContact | ≤5K | 12926 / 0.995 / 100.0 | 12926 / 0.995 / **99.6** |
| WarmContact | >5K | 44767 / 0.995 / 100.0 | 44767 / 0.995 / **99.9** |
| Other | ≤5K | 884 / 0.505 / 97.3 | 884 / 0.985 / 81.3 |
| Other | >5K | 2508 / 0.505 / 99.8 | 2508 / 0.995 / 95.4 |

(Seed 12345 rows in the committed probe output; same shape: Diffusion e0
97.7 → 90.0 on v2, WarmContact ≥ 99.6 both surfaces.)

## 3. Verdict

1. **At the channels' own scope the bars are already mostly open on v2.**
   Diffusion's founding-window share falls only ~8–10 points (89–90% vs
   v1's 97–99%); ToM's Friendly leg is above 99.6% on the eligible kind
   stream on BOTH surfaces; gossip has no bar to fail. The i426
   contacted-graph numbers (30–48-point founding gap) measured
   contacted-but-not-interacting pairs that no channel reads — a scope
   mismatch, correct under §4.12 (measure at the predicate's scope).
2. **The i403 conviction halving (0.486 → 0.215) therefore is not a
   gate-eligibility effect.** The i401/i403 decompositions already pointed
   elsewhere: moving the kind schedule changed the mix (Help/Comfort 90%+ →
   83–86%, Talk 4–7% → 11–16%, Gossip 0.3% → 2.6%). The kind schedule's
   saturation defect (§4.10, recorded at i402: 98–99% of interactions above
   the 0.70 bar) is the piece that needs work — a re-derivation of the
   0.7/0.2 kindness THRESHOLDS on the honest surface — NOT the three belief
   channels' trust bars.
3. **Precondition 2 simplifies accordingly**: `stash@{1}` re-application no
   longer needs distributional bar re-derivation for diffusion/ToM/gossip.
   The required companions are exactly (a) the degree re-point (i403), (b)
   the kind-schedule thresholds re-derived against v2's event distribution
   (the 0.7/0.2 pair, the saturated surface), (c) witness channel on v1.

## 4. Gate-form note

`recent_positive`/`recent_negative` at the ToM call site are *the current
event's* kind (no rolling window — 0.6/0.3/0.1 by kind, negatives always 0
in this call), so the Friendly conjunction reduces cleanly to
kind∈{Help,Comfort} ∧ trust>0.5; the other infer_intent branches
(`recent_negative > 0.3`, `trust < 0.2`) are unreachable from these two call
sites today — consistent with the i400 band observation.
