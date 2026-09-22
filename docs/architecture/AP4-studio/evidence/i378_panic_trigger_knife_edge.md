# i378 — the moral-panic trigger is knife-edge debt; the contract stops naming seeds

**Status:** LANDED (contract + debt record; no simulator behaviour changed) ·
**Scope:** the panic integration contracts, and the ledger's classification of the trigger.

i376 de-saturated the legacy trust store and the panic family had to be re-anchored
`{7,11,46}` → `{7,1,23}`. Before repairing or accepting that, the trigger's **headroom**
was measured — because "the seed list moved" and "the producer is starving" look identical
from a firing count, and they demand opposite responses.

## What the probe measured (`i378_panic_threshold_headroom`)

The trigger has two legs (`gossip::detect_moral_panic`):

```
fires  ⟺  avg_charge ≥ 0.55  AND  panic_ratio ≥ 0.30   (share of holders with charge > 0.4)
```

Sampled **every tick** across a 10-seed pestilence crisis (@20K), reporting the best
`min(avg/0.55, ratio/0.30)` score a run ever reached:

| seed | registry | panic events | best score | margin | reading |
|---|---|---|---|---|---|
| 7 | **11** | 11 | **1.094 / 1.072** | +9.4% / +7.2% | fires |
| 1 | **4** | 4 | **1.025 / 1.039** | +2.5% / +3.9% | fires |
| 23 | **5** | 5 | **1.036 / 1.043** | +3.6% / +4.3% | fires |
| 11 | 0 | 0 | **0.9955** (prop 1) | **−0.5%** | misses by *one tick's worth* |
| 5 | 0 | 0 | 0.760 / 0.685 | −24% | |
| 42 | 0 | 0 | 0.762 / 0.745 | −24% | |
| 99 | 0 | 0 | 0.686 / 0.677 | −31% | |
| 46 | 0 | 0 | 0.542 / 0.550 | −45% | |
| 3 | 0 | 0 | 0.496 / 0.460 | −50% | |
| 13 | 0 | 0 | 0.276 / 0.278 | −72% | |

**Verdict: knife-edge, not starved.** Every firing leg clears the bar by only **2.5–9.4%**,
and a swept seed misses by **0.5%**. The producer is not under-driven — it is a threshold
sitting inside the body of its own input distribution. That is precisely doctrine §4.5's
class (the epidemic R0≈1 case that flipped TRANSIENT↔ENDEMIC nine times): *record it as
systemic debt rather than flip-flopping pins forever.*

The registry and `ConflictOccurred { MoralPanic }` event counts agree exactly on every seed
(11/11, 4/4, 5/5 …), so both lenses the integration tests use see the same firing set.

## The structural fix (why another rename was the wrong move)

Three iterations (i343, i351, i376) had to **rename the family** because a threshold with
2–9% margin re-picks its winners on every pacing shift. Renaming is the fragility. So the
contract now names a **fixed crisis family** and *discovers* which members register:

```
const CRISIS_FAMILY: [u64; 10] = [7, 1, 23, 11, 46, 5, 42, 13, 99, 3];
firing = family.filter(|w| w registers)          // discovered, not hardcoded
assert!(!firing.is_empty())                       // the liveness invariant that cannot move
replay_seed = firing[0].seed                      // determinism replays a FIRING member
```

A pacing shift now moves *which member carries the downstream legs* instead of breaking the
pin; and the liveness contract stays strict — at least one member must fire, or the
producer is dead. The determinism leg replays the first firing member, so it can never
compare two different worlds (and never trivially pass on an empty registry).

Both panic tests (`psychology::beliefs_memory`, `governance`) adopt it; the governance
test's redundant `firing >= 2` bar becomes `>= 1` for the same reason the family became
discovered — one liveness invariant, expressed once.

## Recorded as systemic debt (queued, not fixed here)

1. **The trigger's absolute threshold is the knife-edge.** The principled organic redesign
   is a **relative / anomaly-based trigger** — a panic when the population's charge is
   anomalous against *its own* baseline (plus an absolute floor against quiet-world
   noise), which is the i373 audit's test satisfied exactly (the state variable that
   measures the right thing is the population's own charge history) and which is
   *structurally* free of knife-edge because a relative threshold adapts to its
   distribution. Sized by the table above. **Queued as its own iteration** — it is a
   behavioural change with its own sweep, and must not be batched into a contract fix.
2. **The firing density is 3/10 swept crisis seeds** (vs 3/5 before i376's de-saturation).
   With the table above this is now *explained* rather than mysterious: these worlds reach
   0.28–0.76 of the bar, so they were never close. Whether "most crises produce a moral
   panic" is the right realism target is a modelling question for the redesign above, not
   a pin to move.

## Verification

`cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean, integration
**310/0/1**, `scripts/gate --full` GREEN. Probe:
`i378_panic_threshold_headroom` (runnable, cited above).
