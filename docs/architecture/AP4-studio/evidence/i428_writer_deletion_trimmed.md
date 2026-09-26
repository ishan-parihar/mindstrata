# i428 — the v1 interaction-writer deletion, landed trimmed

**Iteration:** i428 · **Date:** 2026-09-25 · **Status:** LANDED (rejects the
full i403 arc's bundle; ships its measured sound core)
**Record of custody:** collapse + riverford goldens re-baselined; 7 snapshots
accepted; 4 behavioural pins re-contracted with probe evidence
**Precondition trail:** i426 (formation-time verdict), i427 (the event-scoped
gates are already open) — i403 §6 preconditions 1 and 2 both DISCHARGED.

## 1. What shipped

The i403 measured rejection was re-assembled minus its two measured-poison
pieces. The stashed arc (`stash@{1}`) applied onto the i425 tree, then two
specific reverts:

1. **The kind-schedule read move** (`interaction.rs::system_social_interactions`)
   reverted to the v1 dense lookup — i427's event-stream census shows every
   trust bar is already 89–100% open on v2 at the pairs the channels read, so
   the read move buys nothing, and i401/i403's kind-mix decomposition
   measured the read move's Talk↑ drift as a belief-kind disruption the
   schedule's own saturated inputs (i402's §4.10) cannot absorb. The
   0.70/0.20 kind-schedule thresholds are a separate organic-redesign item.
2. **The `social_cluster.rs` ToM and §19.5.I read moves**, reverted to v1 —
   i401 measured these two read moves ALONE killing the conception producer
   (zero pregnancies in 2 000 accelerated seed-46 ticks); re-running the
   reproduction pin on the arc + this revert: **green in 402 s**.

Kept from the arc: the process_interaction v1 write deletion,
bonding/conflict-escalation re-host onto the dyadic magnitudes, the credibility
move, the memory_ops fold move, the affection sync leg, `contacted_degrees`
re-pointed at the dyadic rows (the §4.17 degree-freeze companion), the witness
channel staying on v1 (scope decision), and all of i403's test re-contracts
that did not ride on the two reverted read moves. `rel_pos` is production
again (the kind-schedule revert + the arc's social_cluster revert each audit
its own reader).

## 2. Verdict path (measured, order as performed)

| step | suite | note |
|---|---|---|
| stash@{1} applied | — | applies cleanly on the i425 tree |
| kind-schedule reverted to v1 | hang (conception pin) | the i399-class stale-fixture signal, traced below |
| + social_cluster reads reverted to v1 | **297/14** (392 s) | dump: 2 goldens + 4 pins + 7 snapshots + = full failure set |

**Crisis-engine liveness (the arc's own §6 acceptance criterion, measured
post-trim):**
- `i399_revolution_sweep` (mutation off, 70K, 10 seeds): **17 revolutions,
  6-of-10 firing** {99:5, 23:4, 5:3, 42:3, 11:1, 1:1} — the engine is alive
  (i403's rejected full arc measured total 2 revolutions, 1-of-10 firing).
- `i381_blast_radius` (pestilence, the revolution-family config): panics sit
  on the crisis corpora again — seed 1: 17, seed 23: 41, seed 99: 14,
  seed 12345: 5 events (4 distinct seeds firing), with per-seed council
  legitimacy 0.50–0.59.

Then the 14-pin disposition (§4.2 per item):

| pin/golden/snapshot | measured | disposition |
|---|---|---|
| both goldens (riverford + collapse) | writer deletion shifts metric+event+agent hashes; collapse agent_count 13→12 | custody re-baseline via `i276_golden_recompute`, programmatic diff (metric_hash, event_hash, agent_hash, counts) — one change, measured |
| attachment_separation (bio) | 27/48 @42; family {7: 20/48, 11: 34/48} — means 0.0033/0.0035/0.0062 (all above the 0.002 liveness floor), maxes ≤ 0.052 | re-contracted to a **majority floor** (measured 56.2% on the pin's own world; 12.5% margin) with the family numbers in the comment. Mechanism (i401): the deleted write was a Comfort-parking ratchet; the honest surface re-paces reunions |
| noospheric conviction | high 0.2038 / low 0.1316, ratio 1.548, fear-legs byte-identical | re-contracted to the RATIO invariant (`high > 1.35 × low`) plus a dead-producer floor (`high > 0.15`); the old absolute delta 0.15 was calibrated on the ratchet's saturated trust |
| peer_status_envy leg A | mean 0.2975 @42 (pinned floor 0.3), family {7: 0.3859, 11: 0.3814} | floor 0.3 → 0.25; NB. the writer is a proximity max over `effective_status` (social_cluster.rs:78) — neither store — so it is a composition effect with the co-presence mechanism explicitly NOT isolated (honest limit named in the comment) |
| fear-contagion pin | perceiving occupancy: 152/154 folds above the i425 floor, 2 sub-quantum (min 0.00060) | pin scope shrunk to `perceived_stress > 0.001` (the from_f64 quantum floor the fold shares with i425's measured band) + a half-village above-floor liveness assertion |
| revolution pin | sweep says 6/10 fire; BOTH-legs sweep records handover per member: {99: ✓, 23: ✗, 5: ✗, 42: ✓, 11: ✓, 1: ✓} | family re-discovered per §4.8 as **{99, 42, 11}** (the three strongest members that satisfy BOTH legs). Recorded finding: 23 and 5 fire coups that never turn the Elder seat — regime change without office succession on those worlds |
| 7 snapshots | droplets consistent with the honest-surface regime (trust means down, event counts up, kind shares re-timed) | accepted after per-file diff review |

## 3. Suite

297/14 (392 s) → **311/0/1** (719 s). No behavioural pin is re-anchored
without its probe numbers in the diff.

## 4. Precedents this close-out discharges

- i403 §6's precondition 1 (formation-time vs steady-state) — i426.
- i403 §6's precondition 2 (channel gate re-derivation) — i427 measured the
  gates already open at event scope; the distributional re-derivation row
  is retired as unnecessary (the i402 kind-schedule saturation remains the
  open §4.10 item the former precondition mistakenly named).
- §4.17 (degree freeze) held: the arc carries the v2-row re-point and its
  test companion.

**`stash@{1}` is SUPERSEDED — DO NOT re-apply it wholesale.** Its two
read-move pieces are the measured conception/crisis kill and are deliberately
NOT in this landing. The remaining unlanded pieces (`legal_impl`/`marriage`
writers, the witness channels) each need their own behavioural iteration.

## 5. Instruments

- `i428_pin_reanchor_sweep` (contract-mirror of the four failing pins'
  measured legs)
- `i428_revolution_leg_sweep` (both revolution-pin legs over all 6 firing
  seeds — the handover finding's instrument; its per-seed revolution counts
  5/4/3/3/1/1 match the i399 sweep exactly, so the ring buffer did not clip
  any seed's count — the MAX_EVENTS window concern is checked, not assumed)
- `i399_revolution_sweep` + `i381_blast_radius` (crisis liveness)
- `i276_golden_recompute` (golden custody)
