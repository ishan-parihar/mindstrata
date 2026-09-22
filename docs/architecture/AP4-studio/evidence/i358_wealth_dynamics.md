# Iteration 358 — wealth dynamics: the counter-forces all exist, and the one that *would* matter is scale-invariant

**Status:** MEASUREMENT + DECISION (probe + a precisely-scoped queued fix) ·
**Item owned:** the plan's B-item "wealth dynamics — Gini concentration has weak
counter-forces; add/inspect inheritance, charity, redistribution norms".

## Inspection first: all three named forces already exist

- **Inheritance** — a death splits the estate among heirs (`births_deaths.rs`).
- **Charity / patronage** — patron → client transfers gated on a destitution floor
  (`factions_impl.rs`).
- **Redistribution** — two forces: the i186 **progressive market dividend** (surplus
  weighted `1/(1+coin)`, deliberately designed to drop the Gini) and the i261 **council
  poor relief** (relative-poverty relief keyed below 10% of the membership median).
- **Taxation** — council 0.05 / market 0.03 / temple 0.02, collected every centum.

So the plan's premise ("add") is already satisfied; the question is whether the
concentration is bounded.

## Probe (`i358_wealth_dynamics`, N=48, 46×46, 50 000 ticks)

| seed | Gini 5K → 50K | top-decile share | max coin | bottom-half share | zero-wealth |
|---|---|---|---|---|---|
| 42 | 0.388 → **0.647** | 33% → **52%** | 611 → **85 761** | 15.4% → **10.3%** | 0/56 |
| 7 | 0.419 → **0.652** | 35% → **54%** | 960 → **82 650** | 12.7% → **8.8%** | 0/53 |

**Verdict — bounded, not runaway:** both seeds **plateau at ~0.65** (seed 7 is flat
over its last 5K: 0.652 @45K → 0.652 @50K; seed 42's increments decelerate
0.637 → 0.647). **Nobody is destitute** (0 zero-wealth agents at every sample), the
bottom half still holds 9–10% of all coin, and trades keep clearing. A mid-band Gini
of ~0.65 is historically plausible for a stratified pre-modern village, so the
concentration is **not** a §4.3 saturation hazard on the distribution — the
counter-forces work.

## The real finding: a proportional tax is scale-invariant

The tail is the fault. One agent ends up holding **~50% of the village's entire coin
supply** (max 85 761 against a median of 3 014), and it compounds **~3× faster than the
median** (max ×5.5 vs median ×1.9 over the second 25K) — a rich-get-richer loop
(farming skill → productivity → wage → survival/health → more practice).

`Institution::collect_taxes` (`mindstrata-institutions/src/institutions.rs`) deducts
`wealth × tax_rate` — a **proportional** tax. Proportional taxation is **scale-invariant**:
it removes the same *fraction* from everyone, so it cannot change the Gini at all. The
one lever pointed at the tail is mathematically inert on it. That is the precise reason
the counter-forces bound the distribution but never bend the tail.

## Queued fix (scoped, not rushed)

**Make the tax progressive above a reference.** The self-contained change is inside
`collect_taxes` (it already receives `member_wealth`, so the membership median is
computable in place): apply an additional surcharge to wealth above the membership
median, so the effective rate rises with the tail instead of staying flat. This is
behavioural and **sweep-carrying** (taxes fire every centum in every scenario, so the
goldens, the snapshots, and the `wealth_inequality` grievance/legitimacy channels all
move), and per §2/§4.1 it is deliberately **not** folded into a measurement session —
it needs its own probe (predicted Gini asymptote and tail share) and its own classified
re-anchor sweep, with this evidence as the starting point.

## Verification

Probe + docs only; **no source changed**. Golden replay byte-identical, no re-anchors.
(`i358_wealth_dynamics` clippy-clean.)
