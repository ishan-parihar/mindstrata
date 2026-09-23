# i393 — the dual-store writer: probe landed, migration shape decided

**Status:** PROBE LANDED (measurement + design decision) · **Migration not yet wired.**
Probe: `crates/mindstrata-benches/examples/i393_speech_act_store.rs`.

The v1→v2 migration is down to its last structural item. i384 moved the one
self-contained v1 read+write pair (the §13.3 trade price) and left the **general
writer** standing: `process_interaction` applies a per-kind gain to the legacy
`Relationship` row *and* `pass_social` applies a `magnitude` to the dyadic row
through `record_positive`, whose effective trust gain is
`magnitude × volatility × 0.02`. Two schedules, one act.

| kind | v1 Δtrust (applied) | v2 magnitude → Δtrust at vol 0.5 | ratio |
|---|---|---|---|
| Help / Comfort | 0.05 / 0.03 | 0.6 → 0.006 | ~8× / 5× |
| Talk / Gossip | 0.01 / 0.005 | 0.3 → 0.003 | ~3× |
| Trade | 0.02 | 0.2 → 0.002 | 10× |
| Insult / Threaten | −0.08 / −0.1 | 0.5 → −0.005 | ~16× |

## 1. Leg A — the sync signature: *when* the stores disagree

Reading each store's per-act gain out of `Δtrust / Δacts` was the first draft's
mistake and is worth recording: since i376's daily pass pulls `rel.trust` onto
the dyadic value at coupling 1.0, a v1 row's realised movement is dominated by
the sync, not by its own gain — the draft's negative v1 medians were exactly
that, and at N=12 every sampled pair sat at the clamp, so `n = 0`. The
measurable question is *when* the stores disagree:

| N / horizon | `|v1−v2|` trust on a daily boundary | mid-day (+72 ticks) | affection, boundary | affection, mid-day |
|---|---|---|---|---|---|
| 12 / 20 000 (tick 19 872) | **0.0009** (max 0.020) | **0.0220** (max 0.631) | 0.0433 (max 0.797) | 0.0422 |
| 48 / 20 000 (tick 19 872) | **0.0010** (max 0.108) | **0.0256** (max 0.879) | 0.0795 (max 0.901) | 0.0791 |
| 48 / 50 000 (tick 49 968) | **0.0039** (max 0.164) | **0.1689** (max 0.999) | **0.1873** (max 1.000) | 0.1868 |

Three findings, in order of usefulness:

1. **The sync works — for trust, and only on the boundary.** The boundary
   residual is 0.0009–0.0039; it is exactly the one-day quantum of the
   schedules' disagreement, not a leak.
2. **Affection is never synced at all.** i376's pass touches `rel.trust` only,
   so `rel.affection` has no path back to the dyadic value and diverges
   without bound: **0.187 mean / 1.000 max at 50K/N=48**, monotonically worse
   with horizon. i376's claim that "the divergence between the two stores
   closes here" is true for trust and false for affection. Any v1 affection
   reader is reading a store with no relation to v2.
3. **The divergence a reader sees depends on the tick it samples.** Two
   readers of the same pair can differ by 0.169 mean / 0.999 max within one
   day, and by 0.004 on the boundary. This is the §4.3 class ("the number
   depends on when you look") sitting inside the migration's own subject.

## 2. Leg B — the divergence already produced

Over all contacted directed pairs, sampled mid-day at the window end:

| N / horizon | `|v1−v2|` mean | max | >0.05 | saturation ≥0.95 (v1 / v2) |
|---|---|---|---|---|
| 12 / 20 000 | 0.0094 | 0.631 | 2% | 92% / **92%** |
| 48 / 20 000 | 0.0383 | 0.956 | 10% | 70% / 62% |
| 48 / 50 000 | **0.0699** | 0.724 | **28%** | 44% / 38% |

The N=48/50K row reproduces i384's recorded +0.09…+0.13-on-traded-pairs /
0.27–0.31 population figure's *direction* at this horizon; the absolute level
grew between i384's 50K probe and this one only through the horizon, not
through any code change.

**New observation, recorded not chased:** at **N=12 the dyadic store is as
saturated as the legacy one** — 92% of contacted pairs are ≥0.95 in *both*
stores by 20K. i376 diagnosed saturation on v1 at N=48; at village scale the
condition is present in both stores. The i350 contacted-row fold already
compensates semantically (folds gate on `is_contacted`), so this is recorded as
systemic debt — a village-scale trust-discrimination question, not an i393 item.

## 3. Leg C — the decisive leg: regime projection

For every directed pair, the acts actually performed in the window, by kind,
projected gain-only onto the dyadic store (an upper bound; decay only lowers it):

| N / horizon | 1 delete-only (v2 schedule kept) | 2 import-v1 | 3 import-effect |
|---|---|---|---|
| 12 / 20 000 | 0.954 → 0.964, ≥0.95 **95%** | 0.964, **95%** | 0.964, **95%** |
| 48 / 20 000 | 0.813 → 0.837, ≥0.95 **66%** | 0.850, **70%** | 0.849, **69%** |
| 48 / 50 000 | 0.614 → 0.693, ≥0.95 **50%** | 0.744, **61%** | 0.741, **60%** |

Regimes 2 and 3 raise the pooled level and the saturation share together
(50% → 61% at 50K) — i.e. importing the v1 schedule reproduces on the dyadic
store the exact condition i376 diagnosed as the failure on v1 ("saturated,
79–85% of pairs ≥0.95, every v1 trust gate read a dead signal"). Regime 1
leaves the store it keeps untouched.

**Decision: regime 1.** The migration deletes the v1 gain and keeps the dyadic
schedule; the alternative re-imports the disease the migration exists to cure.
A corollary follows immediately: per-act bonding drops ~3–10× to the dyadic
schedule, so trust accrues more slowly and every trust-gated subsystem
(marriage, courtship, factions, economy prices) will move. That is the
iteration's behavioural surface, and it must be re-anchored with attribution —
not softened back toward regime 2 to keep pins green.

## 4. Leg D — the blast radius (source census)

Two census results, and both **correct the plan text**:

**(a) `Relationship.kind` has no production reader.** Every occurrence outside
`interaction.rs` is a construction (`kind: RelationshipKind::Stranger`/`Kin` in
`population.rs`, `births_deaths.rs`, `mod.rs`, `speech_act.rs`), a re-export
(`person/mod.rs`), or a test/bench read (`i326_relationship_sparsity`,
`sim/tests/mod.rs`). The only writer is `evolve_relationship_kind`, called from
`process_interaction` — so the §19.5.G ladder is a **write-only producer**: the
i391 dead-field class, at the relationship layer.

*Consequence for the plan:* the row's stated deliverable — "the `RelationshipKind`
ladder … must move to the dyadic store in the same commit" — is **retired, not
migrated**. Moving a consumerless ladder onto the other store transfers
bytes, not behaviour. Its live replacement already exists and *is* read:
`RelationshipV2.stage` (gated on by `is_contacted`/`is_kin_stage`, counted by
`snapshot_relationship_stages_2000_ticks`, labelled by `derive_public_label`).
The commit that deletes the v1 gain should delete `evolve_relationship_kind`
and the `Relationship.kind` field with it, and pin the stage ladder's liveness
in its place.

**(b) Deleting the interaction gain does not make v1 derived.** Other v1
trust/affection writers remain:

| site | write |
|---|---|
| `legal_impl.rs:179` | `trust −= 0.2` on a legal outcome |
| `marriage.rs:262–263, 290–291` | `trust += 0.2`, `affection += 0.3` on marriage/bond events |
| `cognitive.rs:1033` | the i376 daily sync (trust only) |

So the v1 row keeps being written after the migration by two subsystems at
±0.2/±0.3 per event — an order of magnitude *above* the gain being deleted.
The migration's honest end state is therefore **three** steps, not one: delete
the interaction gain, move those two sites onto the dyadic store (each
behavioural, each with its own probe), and extend the sync (or delete the field)
so affection is no longer a store with no inbound path. Deleting only the
interaction gain would leave v1 *less* accurate, not derived — the i387 lesson
(a doc asserting a value was not landed while the code had shipped it) in
mirror image.

## 5. The migration contract (next commit, sized by the above)

1. **Delete the interaction gain** from `process_interaction` (both directions),
   keeping the `InteractionOccurred`/`RelationshipChanged` events — the pass's
   downstream speech-act machinery (credibility, ToM, courtship, familiarity)
   reads the *events*, not the v1 row, so it is unaffected.
2. **Re-point the speech-act model.** `base_delta`'s doc comment claims to hold
   "the EXACT magnitudes `process_interaction` applies", guarded by
   `model_sign_matches_applied_deltas`. After step 1 the applied magnitudes are
   the dyadic schedule's, so the model's documented deltas and its guard must
   move in the same commit (§4.3: a doc asserting a stale value is worse than a
   stale line number).
3. **Retire the write-only ladder** (`evolve_relationship_kind` + the field),
   and pin `RelationshipV2.stage`'s ladder liveness in its place.
4. **Re-anchor with attribution**, per §4.2: every trust-gated pin that moves
   gets the measured value, the old band, and the mechanism (per-act bonding
   reduced to the dyadic schedule). Goldens/snapshots re-baselined only with
   that evidence. i384's precedent applies: this is a subsystem migration.
5. **Do not** import the v1 schedule to keep pins green (leg C), and **do not**
   leave `affection` unsynced (finding 2).

Recorded as the open item: `legal_impl`/`marriage` v1 writers are deliberately
*out* of this commit's scope — they are behavioural moves with their own probes,
and bundling them would violate the one-root-cause rule (§2.1).
