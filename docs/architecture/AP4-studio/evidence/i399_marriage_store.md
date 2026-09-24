# i399 — the marriage closed-loop migrates onto the dyadic store (LANDED, behavioural)

**Status:** LANDED (behavioural; first golden recalibration since Iteration 66-style
intentional re-baselines — see §5). Probe:
`crates/mindstrata-benches/examples/i399_marriage_store.rs` (+
`i399_revolution_sweep.rs` for the liveness leg).

**Scope:** one closed loop in `marriage.rs` — the formation gate's affection/trust
reads, the bond-boost write (+0.2 trust / +0.3 affection, both directions), and the
jealousy pass's dependence read.

---

## 1. Why this loop, and why it was the sharpest dual-store finding yet

The reader-first sweep (i393 §6) reached the marriage pass after i398's `norms_impl`
reader landed zero-blast. `marriage.rs` turned out not to be a reader at all but a
**closed loop inside the ghost store**:

```
formation gate  --reads-->  v1 affection (L70) + v1 trust (L101)
bond boost      --writes--> v1 (+0.2 trust, +0.3 affection, both directions)
jealousy pass   --reads-->  v1 trust (L448)
```

`Relationship.affection` on v1 has **no other production reader and no decay**, and
the i376 daily sync copies **trust only** — so the marriage bond the engine "forged"
was (a) invisible to the Sternberg/decay machinery, and (b) read back only by
marriage formation itself. Per the i384 rule (read + write move together) the loop
migrates wholesale.

## 2. What the probe measured before touching (`i399_marriage_store`)

| leg | N=12 | N=48 |
|---|---|---|
| A — affection divergence at the read sites (mean / max / sign-flips) | 0.043 / 0.59 / 8186 | 0.081 / 0.88 / 142623 |
| A — trust divergence (held tight by the i376 daily sync) | 0.008 | 0.019 |
| B — married-pair end affection: v1 **0.937** (pinned, no decay) vs v2 **0.878** (decayed, differentiated) | — | — |
| C — partner selection argmax flips | 9 | 45 |

Leg B is the finding: the +0.3 boosts currently land in a store that **cannot decay**,
so married affection saturates flat at 0.937 while the dyadic store (which has the
decay + stage machinery) ends at a differentiated 0.878. The migration makes the bond
visible to the machinery that was built to model it.

Founder-time check (why the migration does not shift unacquainted pairs): v2 rows are
**seeded from v1 at populate** (`population.rs`), so the two stores start identical and
diverge from dynamics only. v2 is a complete per-agent list, so every pair has a row —
the `map_or` priors (trust 0.4 / affection 0.3) are the `RelationshipV2::new` stranger
priors and are unreachable for `i != j`, recorded as such in the code.

## 3. The re-anchor contract, and the fixture hazard it exposed

Three separate things moved, all measured:

1. **Seven snapshots + two goldens** (behavioural, §5 below).
2. **The revolution family** — see §4; re-timed, not starved.
3. **A stale test fixture that HUNG the suite for >300 s.**
   `marriage_forges_spouse_and_inlaw_kinship` suppresses every other pair by zeroing
   **v1** trust/affection. Once the gate reads v2 that pin no longer suppresses
   anything — v2 keeps its populate-time values, and decisively its **affection**
   (never synced) — so agent 0 paired with agent 1 on the first pass and the
   `steps < 900` window ran to exhaustion (900 × 144 ticks). The fixture now pins
   **both stores**; the test is back to 0.03 s.
   *Operational lesson for the rest of the sweep: any fixture that pins a store the
   pass reads is a silent time bomb under this migration. Grep for v1-only pins
   before migrating the next reader.*

## 4. Revolution liveness: measured, not re-pinned (`i399_revolution_sweep`)

The family `revolution_is_regime_change_not_repeat_loop` broke: its i388 members
`{5, 42, 12345}` measured 7/5/3 before and **0/0/7** after — 1 of 3 firing, under the
≥2-of-3 bar. Per §2.3 a dark producer is a bug, so the sweep (pestilence @70K, mutation
off, the exact i381/i388 seed list) was re-run on both trees:

| tree | total | firing seeds |
|---|---|---|
| before (v1 loop) | **15** | {5→7, 42→5, 12345→3} — 3/10 |
| after (dyadic loop) | **18** | {12345→7, 7→3, 11→6, 23→1, 99→1} — **5/10** |

The producer fired **more**, on **more** seeds, and merely re-timed — the i388 pattern
exactly (stronger, decaying bonds raise dyadic trust → appraisal threat falls → the
grievance route to a coup fires in different seed/tick worlds). Re-anchored onto the
three best-evidenced members `{12345, 7, 11}`; liveness bar unchanged at ≥2 of 3; the
§4.5 knife-edge debt stands unchanged.

## 5. The re-baselined observables (all mechanism-explained)

`riverford_minor` seed 42 (1000 ticks): metric_hash `12871778371033085037` →
`174523123631603418`; agent_count 12 (unchanged); grain 82.0872 → 83.3373;
water 1979.981 → 1980.1136.

`collapse` seed 42 (4320 ticks): metric_hash `11026946663692602518` →
`8993835882449929447`; agent_count **12 → 13** (one more crisis survivor — the crisis
mortality check the baseline exists to pin); grain 1.5799 → 0.1104 (both are a
depleted store at the end of drought→famine→pestilence); water 1140.6856 → 1136.2803.

Snapshot drift, and the direction that identifies the mechanism:

| snapshot | move | mechanism |
|---|---|---|
| metrics_500 | trust 0.6345 → 0.6475, quality 0.4124 → 0.4217, health 0.8162 → 0.8300, stress 0.2897 → 0.2848 | the bond boost is now counted by the dyadic store's `is_contacted` folds |
| metrics_2000 | trust 0.6895 → …, health 0.7897 → … | same, 4× the horizon |
| long_horizon_10000 | trust 0.8245 → 0.8427, quality 0.7281 → 0.7569, stress 0.3605 → 0.3679, **agent_count 13 → 12** | stronger bonds; one fewer survivor (stream reshape) |
| relationship_stage_distribution_2000 | `Confidant` appears; `Familiar` 17 → 24, `Unnoticed` 26 → 24 | the boost now feeds the stage ladder instead of a store with no reader |
| agent_states_1000 / institution_states_1000 / endocrine_states_500 | small bounded drifts | stream reshape; Bran's endocrine stress joins Anna/Cara at 0.0 (a value two of three founders already hold) |

Direction check that this is the intended fix and not a new saturation: relationship
**trust and quality rise**, stage distribution **deepens** (`Confidant` becomes
reachable), and the 10K stress level rises slightly rather than collapsing — i.e. the
engine now credits the marriage bond that it was previously writing into a store with
no decay and no readers.

## 6. Verification

`cargo fmt --all` · `cargo clippy --workspace` clean · sim **310/310** · integration
**314 passed / 0 failed / 1 ignored** · both goldens regenerated and re-run · gate
GREEN.

## 7. What this changes for the rest of the sweep

* i393 §6's remaining order stands, with one correction: the next step is the
  **v1-application deletion with the `bonding_rate` / `conflict_escalation_rate`
  re-hosting** (both default to `Fixed::ONE` and their only consumer was the deleted
  write — a re-host at the dyadic site is value-neutral, a re-anchor would not be).
* The remaining v1 readers are unchanged in count except for the ones migrated here:
  `economy.rs` L339, `social_cluster.rs` (×5), `pass_social.rs` L313, `memory_ops.rs`
  L155 (the `trust > 0.6` status fold).
* `affection` remains **unsynced** between the stores (i376 covers trust only), so the
  mid-day divergence leg A measured is now *resolved by construction at the marriage
  gate* (it reads the dyadic store) but not elsewhere; extending the sync to affection
  is still queued.
