# i401 — the bundled dual-store close-out, measured, decomposed, and NOT landed

**Status:** MEASURED REJECTION (a recorded rejection is a landing) · **Code landed:** none
(the bundle is reverted; the tree returns to `ce4e983`-era behaviour) · **Probe:**
`crates/mindstrata-benches/examples/i401_attachment_mechanism.rs` · **Files that carried
the bundle:** `social/src/social/interaction.rs`, `sim/src/sim/pass_social.rs`,
`sim/src/systems/cognitive.rs`, `core/src/parameters.rs`, `sim/src/sim/social_cluster.rs`,
`sim/src/sim/memory_ops.rs`, `sim/src/sim/tests/{mod,culture}.rs`.

i393 §6 ordered the last structural inconsistency in the engine — two coexisting
relationship stores — closed reader-first: migrate each v1 reader with its own probe,
then delete the v1 **application** (the interaction write), then move the folds. i398
landed the `norms_impl` reader, i399 the marriage closed loop, and i400 measured and
reverted the `social_cluster` reads on the argument that the whole trust path had to move
at once. i401 was that "at once": the writer deletion, the `bonding_rate` /
`conflict_escalation_rate` re-host on the dyadic application, the `affection` leg of the
daily sync, the `social_status_counts` fold move, the `social_reciprocal_factor`
retirement, and the i400-bundled `social_cluster` reads.

It does not land. This doc records what the two verification runs bought, because three
of the four findings are invisible from reading the code.

## 1. The bundle, as built and run

| Shape | Result | Verdict |
|---|---|---|
| Full bundle | **304/16/1 in 656 s** — 2 goldens, 7 snapshots, and 7 behavioural pins: `conception_pipeline_round_trips_with_birth`, `attachment_separation_distress_coupling_is_live_after_tuning`, `culture::meme_transmission_multiplier_increases_host_adoption`, `psychology::attention_executive::sensory_field_fear_contagion_is_live_and_sustains_fear`, `psychology::beliefs_memory::noospheric_belief_confidence_sustains_conviction`, `social::status_familiarity::relational_fields_refresh_deterministically`, `governance::revolution_is_regime_change_not_repeat_loop` | Not landable. §2.3 forbids re-pinning a producer that went dark, and §5.1 forbids re-anchoring on a single seed. |
| Writer deletion + rate re-host + parameter retirement only | `attachment` **27/48** (floor 32) — the historic i185/i191/i381 band is 34–39/48 | The single blocking behavioural consequence, isolated below. |

## 2. Finding A — the `social_cluster` read move kills the conception producer

Re-running the bundle with **only** `social_cluster.rs` returned to its v1 reads:

```
conception_pipeline_round_trips_with_birth ... ok        ← was FAILED in the bundle
attachment_separation_distress ............ 21/48 → 27/48 ← unchanged verdict, 6 agents recovered
```

Zero pregnancies in the first 2 000 ticks of the accelerated seed-46 world is a **dead
producer**, not a paced one (§4.3). So i400's revert was right *and* is now proven under
the bundle it asked for: this move is not "waiting for the writer deletion", it is
independently un-landable and belongs in a behavioural iteration with its own re-pacing
probe. The i400 finding stands unchanged: the divergence tail sits on the live 0.5
`infer_intent` verdict (0.8–1.8% of sampled pairs straddle it).

## 3. Finding B — the attachment channel's producer is the v1-reading *kind schedule*

The writer deletion leaves the attachment coupling visibly quieter across three seeds
(probe `i401_attachment_mechanism`, seed 42 / N=48 / 5 000 ticks, and the same shape on
seeds 7 and 11):

| seed | nonzero distress, HEAD | writer deleted | mean, HEAD | mean, deleted |
|---|---|---|---|---|
| 42 | 38/48 (79%) | **15/48 (31%)** | 0.0079 | 0.0022 |
| 7 | 29/46 (63%) | **10/48 (21%)** | 0.0041 | 0.0022 |
| 11 | 43/46 (93%) | **13/46 (28%)** | 0.0120 | **0.0016** (under the 0.002 floor) |

The probe partitions the numerator, and the partition is the mechanism: co-residency is
**not** the explanation (co-resident couples fell 10 → 4 and mean partner distance rose
14.75 → 15.42 while distress *fell*), so the coupling is not being replaced by the i185
"correct zero" of a fully co-resident couple. The producer itself thinned.

The cause is one hop upstream of attachment, and it is a v1 reader:

```rust
// social/src/social/interaction.rs, system_social_interactions
let (trust, affection) = pair.map_or(
    (params.social_default_trust, params.social_default_affection),
    |p| (relationships[p].trust, relationships[p].affection),   // ← v1, the deleted write
);
let kind = choose_interaction(trust, affection, … , rng, params);
```

`choose_interaction` gates the interaction **kind** on those two v1 numbers, and the kind
is what feeds §8.1.14:

* `affection > social_high_affection_threshold` (0.7) → the Comfort / Help / Talk branch;
  `Comfort` is the `receive_comfort` drain;
* otherwise, `openness > social_openness_threshold` → the **Gossip** branch; `Gossip` is
  the `on_reunion` recovery, applied to *both* agents on every gossip event.

The v1 write being deleted was a **ratchet**: every positive interaction pushed v1
affection up, and the daily i376 sync only pulled it back toward v2 (it never pushed the
ratchet down). So the v1 value this schedule read sat systematically above its v2
counterpart — which parked pairs in the high-affection branch and *out* of the Gossip
branch. Delete the write and v1 becomes a one-day-stale mirror of v2: pairs fall back
into the openness branch, gossip density rises, and reunion recovery drains the distress
the separation pass keeps adding. Direction, magnitude and partition all agree.

**The consequence for the migration is not "re-anchor the band".** It is that the kind
schedule is a v1 consumer whose calibration was riding the ratchet — exactly the class
i401 found for `bonding_rate` (a parameter whose only consumer was the deleted write) and
i398 found for the violence damage (a persistent-penalty magnitude calibrated while the
penalty was transient). Migrating the schedule's read is a behavioural iteration with its
own probe: the kind mix is the thing to measure (Comfort/Gossip/Help/Talk shares), and the
two thresholds it gates on (0.2 / 0.7) are calibrated against the ratchet's inflation, so
they need re-derivation in the same sweep as the read move — never a band widening.

## 4. Finding C — the `social_status_counts` fold move is orthogonal

The fold move (`memory_ops::social_status_counts` reading `agents[i].relationship_v2s`
instead of the v1 matrix) was carried into the bundle as the "contact-layer" companion.
Isolated, it contributes **nothing** to the attachment shift (27/48 with it, 27/48
without) — its blast, if any, is on the attention/peer-status consumers it feeds, and its
own test was rewritten to pin the structural invariant it now relies on (one row per
peer, no self-row, endpoints in range) rather than the v1 fold's duplicate/stale-id
defences, which are structurally impossible on the dyadic store. It is a clean candidate
for its own small landing once the schedule question (§3) is settled, because that is the
change it will be measured against.

## 5. Finding D — the retirement has no home yet

`social_reciprocal_factor` (the v1 reverse-direction gain, 0.3) was retired in the bundle
on the grounds that its only consumer was the deleted reverse write. That reasoning holds
only *given* the deletion, so the retirement travels with the writer deletion and not
before it — a dead parameter, like a dead producer, is a hazard (§4.3), not an early win.
The i398 doc already recorded the same coupling for the violence writers.

## 6. What the next iteration is

**i402 — the interaction-kind schedule reads the dyadic store**, with:

1. a **kind-mix probe** (Comfort / Gossip / Help / Teach / Talk shares at N=12 and 48, on
   both stores, over a 20K corpus) — the i398/i399/i400 pattern applied to the schedule;
2. the read move *plus* a re-derivation of the two gating thresholds against the new
   affection surface, in one commit;
3. only then the writer deletion, with the rate re-host (§5.1 re-host, not re-anchor) and
   the `affection` sync leg, and only then the fold move and the parameter retirement.

The two findings in §2 and §3 are the reason this is a queue and not a commit: they are
both *producer* findings, and every producer finding in this codebase has cost its own
iteration to repair honestly.

## 7. Cost accounting

Two full release-suite runs (656 s and 143 s) and one probe cycle bought: the isolation of
the `social_cluster` producer kill (§2), the identity of the attachment channel's v1
producer and the direction of its calibration bias (§3), the null result on the fold move
(§4), and the ordering constraint on the parameter retirement (§5). None of the four is
visible from reading the diff; all four are now load-bearing for i402.
