# i376 — the legacy trust store stops saturating: it converges onto the dyadic store

**Status:** LANDED (behavioural, sweep-carrying) · **Scope:** one writer
(`systems/cognitive.rs`, the `relationship_dormant_decay` row) + one parameter default.

The live ledger queued the **v1→v2 migration** subsystem by subsystem and named the
cognitive mean-reversion writer first "among writers (a divergence *source*)". The probe
found something sharper than a divergence: **the writer was failing at its own stated
purpose**, and every reader of that store was reading a saturated value.

## What the probe measured first (`i376_v2_trust_divergence`, 46×46/N=48, 50K)

Two stores hold the same quantity under two different laws:

| | law | equilibrium at 50K |
|---|---|---|
| v1 `Relationship.trust` | interaction gains; **mean reversion toward 0.5** at `relationship_dormant_decay` = 0.001/day | **0.887–0.920**, 79–85% of pairs ≥0.95, 42–57% ≥0.99 |
| v2 `RelationshipV2.trust` | interaction gains via `record_positive`; **decay toward ZERO** at `decay_rate` = 0.0002/tick | **0.588–0.628** |

1. **The v1 counter-force loses by ~14×.** Its own comment said it existed so "positive
   interactions ratcheted all trust to 1.0 … erasing the differentiation the witness
   system produces". It *did* ratchet to 1.0 anyway. The interaction path adds
   +0.02…+0.10 per act to v1 (`interaction.rs`, `marriage.rs` +0.2, `economy.rs` +0.02)
   while the reversion removes at most 0.001 × gap ≈ **0.0004/day** — doctrine §4.3's
   saturated-state hazard, in a store nobody had probed.
2. **The two stores diverged by |v1 − v2| mean 0.27–0.31, p95 0.92–0.99, across 55–63%
   of pairs** — and it *grew* over the horizon (d.mean 0.037 @5K → 0.31 @50K), so it was a
   live divergence source, not the redundancy i353 refuted for the sparse store.
3. **The divergence blinded every v1 trust gate.** The v1 store feeds `memory_ops`'s
   `r.trust > 0.6` encoding gate, the speech-act credibility scale
   (`social/speech_act.rs`), marriage partner trust, economy trade-partner trust and the
   norm-enforcement paths. A store where 4 in 5 pairs read ≥0.95 makes every one of them
   non-selective.

## The law

```
v1.trust ← v1.trust + (v2.trust − v1.trust) × relationship_dormant_decay     (daily)
default: relationship_dormant_decay = 1.0  (full sync; 0 = legacy store frozen)
```

**One quantity, one truth.** `RelationshipV2` is the charter's designated store (35+
fields, Sternberg triangle, attachment security) and the one the cognitive / appraisal /
attention passes already read, so the legacy row converges onto it. The baseline is no
longer a magic `0.5` — it is the sim's own dyadic estimate for that ordered pair. That is
the organic form of the i373 question ("does a state variable already measure what the
constant guesses at?"): here the state variable *is* the other store.
`relationship_dormant_decay` keeps its role as the relaxation rate and gains a second
reading (0 = frozen, 1 = full sync), so no parameter goes dead.

Determinism: pure function of state in fixed row order, no RNG, no allocation, O(R). The
dyadic row's endpoint is validated (`v2.to == rel.to`) so a stale v1 pair can never read
an unrelated row. `Fixed` is 1e-4-resolution, so a coupling leaving the daily step below
~1e-4 would quantize away — the shipped default is orders of magnitude above that.

## The sweep (leg B — why the *baseline*, not the rate, is the fix)

N=48, 50K, seeds 42/7/23. `sel%` = share of pairs in (0.30, 0.70), the band a trust gate
can actually separate:

| coupling | v1 mean | v1 %≥0.95 | sel% | d.mean | d>0.05% |
|---|---|---|---|---|---|
| 0.001 (the old rate) | 0.89–0.92 | 74–80% | 12–18% | 0.27–0.31 | 55–60% |
| 0.05 | 0.80–0.84 | 64–68% | 15–22% | 0.11–0.18 | 21–31% |
| 0.25 | 0.73–0.85 | 53–70% | 14–26% | 0.10–0.17 | 18–26% |
| 0.5 | 0.76–0.78 | 53–54% | 19–25% | 0.08–0.18 | 17–31% |
| **1.0 (shipped)** | **0.68–0.70** | **42–47%** | **22–30%** | **0.064–0.072** | **22–27%** |

Retuning the *rate* under the old baseline does nothing (row 1 still saturates) — the
divergence is cut 4× and the store becomes ~2× more discriminating only when the baseline
becomes the dyadic store. The residual +0.06–0.08 offset is the *next* genuine debt: v1's
interaction gains (+0.04…+0.10) remain ~10× v2's (`record_positive` × 0.02), so v1 sits one
day of gains above v2 by construction. Queued, with the gain mismatch measured.

## Blast radius and every re-anchor

| surface | pre-i376 | post-i376 | mechanism |
|---|---|---|---|
| panic seeds (`i376_panic_sweep`, 10 seeds, pestilence @20K) | {5:0, **7:11**, 42:0, **11:5**, **46:2**} — 3/5 | {5:0, **7:11**, 42:0, 11:0, 46:0, **1:4**, **23:5**, 13:0, 99:0, 3:0} — 3/10 | the rumor/credibility channels now see a differentiated trust field, so belief charges re-seat. **Mechanism alive** (seed 7 still fires 11 panics, peak 0.1535). Family re-anchored **{7,11,46} → {7,1,23}** in both integration tests (the same majority contract i343/i351 used, backed by the sweep). The determinism replay seed moved 11 → 1 because it must replay `crisis_second`. The halved firing density is recorded as its own queued iteration — the panic channel's residual dependence on the wide trust range is a real coupling to re-examine, not a pin to move. |
| biology golden window (`i376_golden_window_sweep`, seeds 1–70 @2000) | clean family 40–69, leg on seed 44 | **51/70 clean**, seed 44 conceives again | the trust-gated courtship/trade channels re-pace early scarcity. The leg returns to the **canonical seed 42** (clean for the first time since Iteration 185). |
| `riverford_minor` golden | metric_hash `e30d1689607ea4ef` (16360757800592123119), agent_count **13** | metric_hash `4b8b8c8b0d54cfa6` (5443599103459381158), agent_count **12** | the early birth that the saturated store produced inside the 1000-tick window no longer conceives — the same signature the biology sweep measured on seed 42. |
| `collapse` golden | metric_hash `952ff7a586c7d996` (10750083125859572118), agent_count 12, grain 1.1262, water 1140.0432 | metric_hash `69367314cf77147f` (7581373555942036607), agent_count **12**, grain **0.3340**, water 1141.5062 | mortality is unchanged; the famine's post-shock grain/water redistribution shifts with trust-gated trade. Same contract (12 agents survive), different magnitude. |
| 7 insta snapshots | — | accepted | the signature is `avg_relationship_trust` falling off the saturated plateau (~0.81 → the differentiated level) and the trust-gated stage distribution re-seating: `relationship_count` 156 → 132, `Ally` 74 → 62, `Unnoticed` 20 → 15, and `Friend`/`CloseFriend` gaining. Fewer saturated pairs read as "Ally"; the categories move to where the differentiated signal actually puts them. |

## Verification

`cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean (no new warnings),
sim **300/300**, integration **310/0/1**, `scripts/gate --full` GREEN.
Probes: `i376_v2_trust_divergence` (legs A/B), `i376_panic_sweep` (leg C),
`i376_golden_window_sweep` (leg D) — all runnable, all cited above.
