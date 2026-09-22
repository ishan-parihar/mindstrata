# i379 — the gate selectivity audit: which hardcoded thresholds still discriminate

**Status:** LANDED (audit + doctrine; no simulator behaviour changed) ·
**Method:** three methods, because each one alone produces the wrong answer.

| Method | Finds | Blind to |
|---|---|---|
| `const` census (i373) | named constants | numbers *inside* call sites (gains, per-act deltas) |
| store-equilibrium probe (i376) | saturated stores, mistuned counter-forces | whether a *gate* on that store is live |
| **gate selectivity census (i379)** | dead gates, dark-by-design gates, quantile gates | *why* a gate is dark (needs context) |

The third method measures the share of samples that OPEN each shipped gate, over a
two-axis grid: **distribution type** (fixed-founder vs endogenous) × **context** (calm vs
crisis). Context is not optional — a hunger gate that never opens in a calm village is
doing its job; the same gate dark in a famine is broken.

## The measured table (`i379_gate_selectivity`, seed 42)

`open%` = share of agent-samples clearing the gate.

| gate | calm village | calm town | crisis village | crisis town | reading |
|---|---|---|---|---|---|
| `needs.hunger > 0.85` (routine eat) | 0.0 | 0.0 | **0.7** (max 0.987) | 0.0 | **dark by design** — famine gate |
| `needs.hunger > 0.70` (health effect) | 0.0 | 0.0 | **2.1** | 0.0 | dark by design |
| `needs.thirst > 0.90` | 0.0 | 0.0 | 0.0 | 0.0 | scheduled-drink fallback (see below) |
| `needs.fatigue > 0.90` | 0.0 | 0.0 | 0.1 | 0.0 | scheduled-sleep fallback |
| **`needs.social > 0.30`** (socialise) | **0.0** | **0.3** | **0.5** | **0.1** | **isolate detector** — p95 0.03 |
| **`needs.social > 0.40`** (extravert seek) | **0.0** | 0.2 | 0.0 | 0.0 | same; **joint gate with joy > 0.5 ⇒ ~0%** |
| `personality.traditionalism > 0.60` | **46.0** | **41.4** | 50.0 | 50.0 | **quantile gate — stable** |
| `personality.traditionalism > 0.70` | 38.3 | 35.2 | 41.7 | 41.7 | quantile gate — stable |
| `personality.conscientiousness > 0.50` | 38.3 | 39.3 | 41.7 | 41.7 | quantile gate — stable |
| `emotions.fear > 0.50` | 30.7 | 31.1 | 37.5 | 25.2 | live |
| **`emotions.anger > 0.50`** | 3.4 | **0.0** (p95 0.005) | **0.0** (max 0.178) | **0.9** | **near-dead at town scale** |
| `emotions.joy > 0.50` | 9.2 | 9.1 | 10.1 | 8.8 | live |
| `cognitive.heuristic_bias > 0.50` | 30.7 | 31.1 | 39.7 | 25.2 | live |
| `v1 trust > 0.60` (memory encoding) | **82.9** | 42.3 | 50.6 | 60.3 | near-dead *at the calibrated N* |
| `council legitimacy > 0.55` | **99.4** | **100.0** | 62.1 | **100.0** | dead-high in calm |
| **`council legitimacy < 0.50`** (faction arm) | **0.0** | **0.0** | 0.0 (p05 0.539) | **0.0** | **never fires in any context** |

## The four classes this produces

### 1. Quantile gates over a fixed distribution — **robust by construction (keep hardcoded)**

Trait gates open at 31–54% and the rate is *stable across all four worlds* (traditionalism
> 0.6: 46.0 / 41.4 / 50.0 / 50.0). This is not coincidence: founder traits are drawn
**uniform**, so a trait threshold selects a fixed *quantile* of the population, and the sim
does not move the distribution afterwards. This is the structural reason trait gates never
generate the pin churn that the panic trigger (i378) does, and it is a direct corollary of
the §5 H5 finding that uniform founder draws are load-bearing: **reshape the founder
distribution and every one of these gates silently changes its quantile**. Endogenizing
them would buy nothing and cost the robustness.

### 2. Crisis gates on bounded need scales — **dark by design (keep hardcoded)**

`needs.hunger > 0.85` and `> 0.70` open only in the famine world (0.7% / 2.1%, max 0.987).
A gate that is dark in a fed village and live in a famine is *the mechanism working*. The
audit's rule: **read a `0.0%` open-rate against the world that should open it, not against
the run you happen to have.** Likewise `needs.thirst > 0.90` / `fatigue > 0.90` are
fallbacks behind the scheduled routine slots (i.e. the routine path is the primary driver),
which is why they stay dark even in crisis — consistent, not dead.

### 3. Gain/scale mismatches — **the class i376 and the utility term share**

`needs.social` accrues at `social_decay_rate = 0.0002`/tick and is relieved continuously by
village life, so its **reachable band is 0–0.03** (p50 0.006, p95 0.03) in every world.
Two consequences, and they are different defects:

- the **gates** at 0.30 / 0.40 are reachable only after ~1 500 ticks of total isolation, so
  they are an *isolate detector* (max 0.31 in the crisis village, 0.80 in the town). That is
  a legitimate narrow trigger, not a dead producer;
- the **action-selection term** `utility += needs.social × social_value × extraversion`
  (`actions/mod.rs`) multiplies by a quantity whose typical value is ~0.006 — the term
  contributes ~0.5% of typical utility, so the need barely shapes action choice. **This is
  the same defect as i376's ~10× trust-gain mismatch**: a gain calibrated so that the
  common case rounds to nothing. Queued.

### 4. Thresholds on endogenous distributions — **the knife-edge class (queued, i378 started it)**

`council legitimacy < 0.50` (the faction arm) never fires in any of the four worlds — the
legitimacy band is 0.54–0.69 and its crisis floor is p05 0.539, so the arm is decorative;
`factions_impl.rs` ORs it with `pressure_arms`, which is the live driver (the i240
crisis-pressure lifecycle). Same class as the panic charge threshold: an absolute bar on a
self-driven distribution whose operating point drifted away from the bar.
`emotions.anger > 0.50` is the same shape at town scale (p95 0.005–0.071), which means the
negativity channel's anger arm is effectively inert and **conflict is driven by the
`agreeableness < 0.35` quantile arm** — a good example of a quantile arm quietly carrying a
mechanism whose other arm has gone dark.

## The implementation queue this produces

1. **The `needs.social` utility scale** (Class-3 gain mismatch) — either raise the need's
   accrual so its band reaches its own gates, or scale the utility term to the band the
   need actually occupies. Probe first: measure the Socialize action's decision share
   against `needs.social`.
2. **The anger arm of the negativity channel** — decide explicitly whether conflict should
   respond to anger at all at town scale; if yes, the arm is calibrated above anger's
   reachable band (the i347 pattern) and needs re-anchoring, not deletion.
3. **The legitimacy arm of the faction trigger** — either remove it (superseded by crisis
   pressure, and a dark arm is a false affordance of the i372 variety) or re-anchor it below
   the measured legitimacy floor.
4. **The relative/anomaly redesign** for the endogenous-distribution class (i378's queued
   panic trigger is the first instance; the faction legitimacy arm and any collective-mean
   gate are the same shape).

## Which aspects benefit from hardcoded values — the direct answer

| Category | Verdict | Why |
|---|---|---|
| **Founder trait thresholds** | **keep hardcoded** | they are quantile gates over a fixed uniform draw — self-balancing, robust across worlds, and their robustness is *destroyed* by changing the distribution, not by endogenizing them |
| **Physiology / need decay / mutation noise / the cognitive ontology** | **keep hardcoded** | the hard number *is* the modelled natural law (§4.6) |
| **Crisis gates on bounded normalised scales** (famine hunger, collapse thirst) | **keep hardcoded** | dark-in-calm is the semantics; a relative form would fire in a fed village |
| **Solver/damping numerics** (trade diffusion, logistics, the i376 convergence timescale) | **keep hardcoded** | endogenizing a timescale yields numerics pretending to be psychology |
| **Action-utility gains** (`needs.social × …`, the v1 trust per-act gains) | **make organic / rescale** | a gain whose common case rounds to nothing is a dead channel by accident |
| **Absolute thresholds on self-driven aggregates** (panic charge, faction legitimacy arm) | **make relative** | the operating point drifts with the distribution; a relative/anomaly form is structurally free of knife-edge |

The dividing line is **not** "dynamic good, fixed bad". It is: *does the sim already
measure the quantity this number guesses at, and is that quantity's distribution controlled
by the sim or by design?* Uniform traits and natural laws are controlled by design — their
fixity is what makes the emergent behaviour above them meaningful.

## Verification

No code changed. Suite state carried: sim **300/300**, integration **310/0/1**. Probe:
`i379_gate_selectivity` (four legs, runnable, cited above).
