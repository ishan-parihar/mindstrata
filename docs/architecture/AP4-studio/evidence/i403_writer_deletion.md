# i403 — The v1 writer deletion: MEASURED REJECTION (the crisis route starves on the honest trust surface)

**Status:** REVERTED WITH FULL DECOMPOSITION (2026-09-25). The arc was built,
run, and measured end-to-end; the full working tree is preserved in
`git stash` ("i403 measured rejection") for the re-attempt. Per the i400/i401
precedent, a rejection recorded with these numbers is a landing of evidence,
not a failure to ship.

## What was built (all reverted except §5)

1. **Writer deletion** — `process_interaction`'s two v1 write blocks (direct +
   reciprocal × `social_reciprocal_factor`) removed; the §5.1 kind table
   kept as the `RelationshipChanged` PERCEPT (attention/memory consumers);
   `social_reciprocal_factor` retired.
2. **Rate re-host** — `bonding_rate`/`conflict_escalation_rate` scale the
   dyadic per-act magnitudes (identity-1.0, value-neutral).
3. **Affection sync leg** — v1 affection joins the i376 daily convergence.
4. **Kind-schedule read move** — `choose_interaction` reads the dyadic rows
   (the i402-folded move).
5. **Credibility move** — the speech-act/courtship trust input reads the
   speaker's dyadic standing trust.
6. **social_cluster read moves** — §8.1.9 ToM pair reads + §19.5.I
   `source_trust` onto the dyadic rows (the i400 one-commit rule).
7. **`contacted_degrees` re-point** — to `RelationshipV2.interaction_count`
   (see §2 — this fix was REQUIRED by the deletion and is preserved in the
   stash for the re-attempt).

## §2 The degree-freeze — the bug the suite caught (lesson landed as AGENTS §4.17)

First suite run: **18 failures** with an unpredictable signature — the
tier-mix pin ("Background appeared at 6 agents") plus a 35–65% interaction-
volume collapse from tick 0. Root cause: `contacted_degrees()` counted v1
`interaction_count > 0` — the exact bookkeeping the deletion froze. Every
degree read 0 → the tier-importance network bonus collapsed → mass Background
demotions → demoted agents left the social pass → courtship (seed 42: zero
courtships by 10K), memes, and every volume-sensitive pin died together.
The dyadic re-point recovered conception/courtship/meme/memory/tier-mix/anger
in one move. **Rule (§4.17): a deleted write's *bookkeeping* is a consumer;
sweep every reader of ALL stamped fields before deleting a write path.**

## §3 The witness re-host — built, measured, reverted inside the arc

Moving the ±0.02/0.03 witness deltas to the dyadic rows produced the textbook
i398 magnitude change: the v1 bump was transient (erased daily by the
convergence sync); persistent-on-v2, the inflow pinned v2 trust at 1.0 and
saturated the meme baseline (`baseline=36, high=36`). Reverted; the channel
keeps its i349-calibrated transient semantics on v1.

## §4 The rejection evidence — the crisis route starves

- **Revolution family sweep** (i399's instrument, pestilence @70K, mutation
  off, 10 seeds): total collapsed **18 → 2** (i388: 15, i399: 18), firing
  seeds 3-of-10 → **1-of-10** (seed 11 only). By the probe's own criterion
  this is a STARVED PRODUCER, and §2.3 forbids re-anchoring onto `{11}`.
- **Contagion fix exonerated by A/B**: the 10-seed sweep reports
  **identical per-seed revolution counts with and without** the
  `contagion_delta` f64 fix (total 2, seed 11 only, both arms) — the fix does
  not move the revolution route; i425's fold-boundary census later measured
  the crisis INPUT story as outcome-insensitivity, not input-absence (collapse
  fold stress mean 0.1770, min 0.0000, 103/374 agent-days below 20 quanta —
  the sub-quantum class fires in crisis worlds too). The fix is
  LANDED SEPARATELY as **i425** (`evidence/i425_contagion_f64.md`) with the
  custody re-baseline: only the collapse golden moved (+2.3 quanta avg_fear,
  fold-on-final-tick phase alignment), riverford reproduces bit-for-bit.
- **Panic route** (`i381_blast_radius`): panics 0 on 9 of 10 seeds; seed 11
  fires 2 panics → 2 coups (the panic→coup coupling is intact — the input
  died, not the gate).
- **Panic charge distribution** (`i381_panic_anomaly_trigger`, same
  `Scenario::pestilence` corpus): the CALM and CRISIS charge distributions
  now OVERLAP — calm/42 p50 0.33–0.36 with peaks 0.49–0.51 sits on top of
  crisis/23 p50 0.36 / peak 0.54, and crisis/42/13/3 carry almost no charge
  at all (p50 0.15–0.26, max ≤ 0.29). The i381 gap (calm 0.4188 vs crisis
  0.5475, 1.31×) that sized the shipped floor 0.47 is GONE (≈1.06×).
  **This is not a re-sizable bar** (§4.10: lowering 0.47 would fire calm
  worlds); the per-proposition charge PRODUCER itself is starved — the same
  belief-ecology surface whose noospheric conviction halved
  (0.4860 → 0.2150 in the confident-belief pin).

**Attribution (the measured part):** v1 converged onto the dyadic surface
(the probe read v1 ≈ v2: flips 0.0–2.7% on contacted pairs), the calm/crisis
charge distributions overlap, and the belief ecology's conviction halved.
**The per-channel mechanism (re-attempt hypotheses, not yet measured
individually):** the knowledge-diffusion acceptance floor
(`trust×0.5 + openness×0.5` vs 0.5), the ToM Friendly band (> 0.5), gossip
confidence, and the panic charge law are the candidate trust-gated channels
whose calibration may have ridden the ratcheted v1 surface — the re-attempt
must measure each channel's input distribution on both surfaces before
any constant moves. The producer's-calibration-surface class (i401) is the
working frame, at the ecology scale.

## §5 What lands from this arc

1. **This record + the stashes** — the full arc (writer deletion, read moves,
   degree re-point, credibility move, test re-contracts) preserved in
   **`stash@{1}`** (`"i403 measured rejection: full arc preserved …"`),
   the re-attempt's starting map.
2. **AGENTS §4.17** — the bookkeeping-consumer rule (the one durable lesson
   the suite taught at the structural level).
3. **The `contagion_delta` f64 fix — LANDED SEPARATELY as i425**
    (was `stash@{0}`, `"i403 companion: contagion_delta f64 fix …"`;
    `evidence/i425_contagion_f64.md`): exonerated on the crisis corpora by
    identical per-seed revolution counts in both A/B arms, then measured by
    its own calm-band probe and landed with custody — the 307/4 blast radius
    resolved to the COLLAPSE golden only (+2.3 quanta avg_fear: its 4320-tick
    horizon ends on a fold tick; riverford reproduces its baseline
    bit-for-bit) plus 2 snapshots at +1 quantum avg_stress and the 10K
    snapshot's memory-kind stock redistributed at a constant 789 traces
    (quantum-margin eviction tie-breaks, the chain in the evidence doc). The
    i403-era claim that it moved "both goldens" is corrected: only collapse.
    Landed with the i425 commit; no behavioural pin moved.

## §6 The re-attempt's preconditions (the next W1 arc)

1. **First question — ANSWERED (i426, `evidence/i426_belief_formation_surface.md`):**
   **formation-time**, with a smaller steady-state residual. The founding
   belief ecology forms in [0,5 000) while the v2-reading diffusion gate
   clears 49–67% of contacted pairs (v1: 94–97%); v2 climbs to 81–94% by
   18K. The v1 bar mass is spawn-draw lottery (`trust = U(0.3,0.7)` dense
   matrix at populate), v2 starts at its stranger prior and earns — absolute
   `> 0.5` bars are a different contract on each store. Also found:
   `cultural.openness` is an unwired constant 0.5 (diffusion acceptance ≡
   ToM band in production; separate iteration's defect), and the panic
   charge law is NOT trust-gated (distress-driven) — refuted as a channel.
2. **The belief-channel re-derivation** precedes the deletion re-attempt:
   knowledge-diffusion acceptance floor, ToM Friendly band, gossip
   confidence, and the panic charge law re-derived against the honest
   surface — the i381 gap method applied per channel, sizing bars from the
   measured must-fire/must-not-fire distributions on BOTH stores.
3. **Then** the stashed arc re-applies (writer deletion + read moves +
   degree re-point + test re-contracts, all still valid code) — with the
   degree re-point as a REQUIRED companion from day one (§2) and the
   witness channel explicitly left on v1 (§3).
4. The kind-schedule read move is measured safe on its own (i402: 0.0–3.5%
   branch flips; the post-change probe in this arc read flips 0.0–2.7%,
  v1 ≈ v2 converged surfaces) — it can ride with the re-attempt.
