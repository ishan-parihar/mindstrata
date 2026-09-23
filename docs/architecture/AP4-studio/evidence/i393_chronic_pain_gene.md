# i392 row 3 — `chronic_pain_risk` reaches the skeletal accumulation law

**Status:** LANDED (behavioural; mid-point neutral by construction; band closed in
every calibrated corpus) · **Scope:** one dead gene + one spec divergence.

## The row, per §4.15's two gates

**Gate 1 — is the band open?** Probe leg B, sampled every tick over the whole run
(2.29 M agent-ticks across five corpora):

| corpus | ever `fracture_risk > 0` | ever `chronic_pain > 0` | max cp |
|---|---|---|---|
| village 12 s42 @20K | no | no | 0.0000 |
| town 48 s42 @20K | no | no | 0.0000 |
| town 48 s7 @20K | no | no | 0.0000 |
| pestilence s42 | no | no | 0.0000 |
| collapse s42 | no | no | 0.0000 |

The band is **closed everywhere**: severe injury (`injury > 0.5`, the only opener of
`fracture_risk`) never happens in any calibrated corpus, including the crisis
scenarios. So the wiring is invisible to goldens and pins by construction — which
the byte-identical goldens confirm — and its value is realised only in futures where
injury exists (violence at scale, hazards, the i397 encounter world). Recorded as
**measured-scope**, not luck.

**Gate 2 — the response's shape.** Chronic pain is a pure linear accumulator (no
gate, no threshold), unlike the elastic Wander coefficient (i392 row 2, elasticity ≈4
that killed that wiring). Leg D, at fixed fracture risk 0.2 for 500 steps:

| gene | multiplier | chronic_pain | Δ vs neutral |
|---|---|---|---|
| 0.00 | ×0.600 | 0.30 | −0.20 |
| 0.25 (draw mid, neutral) | ×1.000 | 0.50 | 0.00 |
| 0.50 | ×1.400 | 0.70 | +0.20 |

Exactly proportional: ±40% gene ⇒ ±40% rate ⇒ ±40% state. The linear shape is what
makes span 1.6 safe — there is no gradient-to-threshold cliff to fall off.

## The §5 quantization check (leg C)

Below `fracture_risk 0.02`, every carrier loses the increment (0.01 × 0.005 = 5e-5
< 1e-4 quantum) — **independently of the gene**, i.e. already true today at the
hardcoded rate. The gene differentiates carriers in exactly the regime where the
state can move at all; it does not create a new dead zone.

## Two probe-harness traps found and fixed before trusting any number

1. **Final-snapshot blindness.** The first draft sampled only the last tick; decay
   (−0.0005/tick on both risk and pain) can empty the state between an injury spike
   and the sample. Fixed to max-over-run, every tick.
2. **The saturated harness (the §4.13 trap).** Driving injury through `tick_update`
   pushes `fracture_risk` to its 1.0 ceiling and clamps every carrier to
   `chronic_pain 1.0` — a harness that cannot separate carriers, and whose
   non-separation would have been misread as "gene does nothing." Leg D holds risk
   fixed instead.

## The anchor choice (why 0.25 and not the Default 0.2)

`reproductive.rs`'s row-1 precedent anchored the multiplier at the gene's `Default`
because that gene's draw was centred on the constant. Here the draw is `U(0.0, 0.5)`
with mean **0.25**: anchoring at the Default 0.2 would leave a **systematic +8%**
population drift (1 + 1.6 × 0.05) in every future injury world. The anchor is the
draw midpoint, so the population-mean multiplier is 1.0 by construction (§4.6) and
gene 0.25 reproduces the retired constant exactly. Leg A measures the realised means:
0.951 (n=12), 0.994 (n=48) — inside ±5%, per the midpoint rule.

## Second finding in the same row: the spec field is inert

`specs/biology/organs.ron:16` declares `chronic_pain_accumulation_rate: 0.001`.
The operative rate was hardcoded `0.005` while the code comment *claimed* the value
came from the spec. The identifier appears in no Rust code — comments only — and
the spec value **cannot work** at this state's resolution: 0.05 risk × 0.001 =
5e-5 < the 1e-4 quantum, so the accumulation would be permanently dead (§5). The
code was right, the spec is inert, and nothing said so. Now the module header
documents the divergence explicitly and the neutral rate is a named constant
(`CHRONIC_PAIN_ACCUMULATION_RATE`) with the reason it cannot drop to the spec value;
aligning the spec is recorded as spec debt in `PLAN_DC5_DEVELOPMENT.md`.

## What changed

- `mindstrata-person/src/biology/skeletal.rs`:
  - `chronic_pain_accumulation_rate(gene) -> Fixed` — f64-computed, quantized once (§5);
  - `SkeletalUpdateParams { chronic_pain_accumulation_rate }` — the row-1 params-struct
    pattern, so the four `Fixed` arguments cannot be transposed;
  - `tick_update` takes the params; the rate comment no longer misattributes to organs.ron;
  - two new unit tests: gene ordering (liveness) and midpoint identity (§4.6).
- `mindstrata-person/src/biology/mod.rs`: the caller feeds
  `genome.health_predispositions.chronic_pain_risk` through the params.
- Probe: `crates/mindstrata-benches/examples/i393_chronic_pain_gene.rs` (legs A–D).

## Verification

fmt clean · clippy 0 warnings · person 125/125 · sim 310/310 · integration
**314 passed / 0 failed / 1 ignored** · `gate --full` GREEN, **goldens byte-identical**
(expected: the band is closed in both golden windows — measured, not lucky).
