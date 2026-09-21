# Iteration 350 — the mean-folds were stranger-diluted (and the sparse store is re-demoted)

**Status:** LANDED (behavioural) · **Root cause owned:** the i341-queued "contacted-only
per-edge iteration" was probed before building (§2.2) and the probe **re-demoted the
sparse store** while exposing the real fault one layer up: the per-agent mean-folds
over the complete row set.

## What the probe measured first (`i350_fold_semantics`)

**Leg A — the win's decay (the re-demotion).** Live v2 rows (interacted ∨ kin-assigned,
post-i349 predicate) as a share of the complete store at N=48: **13.3% @2K → 15.6% @10K
→ 73.7% @40K**. The contacted graph re-saturates at behavioural horizons; the i341
"~50% of the tick, contacted share 12%" headline was a 150-tick-window artifact. The
i338/i344 conclusion re-asserts itself at long horizon: a contact-keyed sparse store
buys a ≤2× constant that shrinks toward 1 as the run lengthens. Not built — §4.1/§4.2
forbid paying a full re-anchor sweep for a decaying constant.

**Leg B — the real fault.** The per-agent mean-folds (appraisal trust sum/min/count,
cognitive `network_centrality`, social_cluster `social_trust`/`social_obligation`)
ran over ALL N−1 rows, so their outputs were diluted toward the frozen stranger prior
(trust 0.4) by N−1−degree rows that never move: **|all − contacted| p50 ≈ 0.30 at
N=48–96** (centrality 0.40–0.45), scaling with N — an isolate in a big village folded
identically to a socialite. Worst: `min_trust` was **pinned at the 0.4 stranger prior
for 19/48 agents**, so the §8.1.4 "closest relationship trust < 0.5" sadness producer
fired at the identical rate for hermit and socialite — the §4.3 zero-discrimination
shape, in a *fold*, invisible to every per-row census.

**Leg C — cost sizing (record).** Per-edge passes at N=192: cognitive 1123 µs,
trust_sync 382, appraisal 318, rel_traces 192, social_cluster 118, kinship_daily 95
of 3911 µs/tick. The contacted-gated folds ride in this cost at zero extra pass.

## The fix — three fold sites, one predicate

`RelationshipV2::is_contacted()` (interaction_count > 0 ∨ kin-assigned; authority
labels excluded — an unmet authority row carries exactly the prior). The mean-folds
now run over contacted rows only, with the **fallback = the stranger prior itself**
(sum 0.4/count 1, min 0.4, prior `quality()`) so a contact-less agent reads exactly
what the old all-rows fold produced — the fallback IS the old constant. The decay
walk in `·cog row sweep` still visits every row (state maintenance, not sociality);
pair scans (marriage/patronage/stage) untouched — courting a stranger is legitimate.

## Classified sweep

- **`relational_dominance_feeds_violence_escalation`** — the aggregate flipped (72 vs
  74 at 5K) because the de-diluted `social_trust` now differentially pacifies
  (`trust_pacify_factor`): escalation → interaction → contacted trust → restraint,
  a self-limiting feedback the diluted fold never allowed. Horizon sweep
  (`i350_dominance_family`, 16 seeds): 5K 77/78, 10K 132/122, 20K 187/174,
  30K 233/206, 40K 156/159 — noise-dominated, non-monotone. Re-contracted (§4.4):
  the structural reach assertion (power_balance strictly higher, per-seed) is the
  surviving contract; the aggregate-magnitude clause is superseded with the feedback
  mechanism documented, plus a family-liveness guard. No eighth seed flip.
- **Both goldens regenerated** — agent_count 12 preserved on riverford_minor and
  collapse; grain/water shifts bounded.
- **7 snapshots reviewed-then-accepted** — bounded shifts at 10⁻⁵–10⁻² (health,
  stress, polarization, morale); 10K surface: trust 0.674→0.706 (de-diluted, as
  designed), envy cost 0.011→0.002 (the diluted fold was inflating it ~5×), memory
  mix shifts with re-paced events, **birth re-paced 13→12 on that seed** — pacing
  verified by `i349_kinship_seed44` (identical births/kinship to i349's table, so
  the courtship producer is intact).
- **No saturation** anywhere in the reviewed surface (stress 0.37, health 0.79,
  trust 0.71 — all mid-band).

## Result

tests **309/0/1**, sim **295/295**, workspace clippy clean, GATE GREEN.

**Ledger effect:** the sparse store is re-demoted at long horizon (leg A); the
mean-fold dilution class is closed; remaining per-edge cost (leg C table) is now
dominated by `cognitive`'s row sweep, whose contacted gating is already in — its
next reduction is behavioural (§17.3 dirty-window pacing), not structural.
