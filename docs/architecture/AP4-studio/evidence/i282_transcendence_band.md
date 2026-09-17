# Iteration 282 — self-transcendence needs-band sweep (the last needs-bands CALIBRATION-PENDING row)

**Status:** LANDED · **Doctrine:** §4.4 rule-4 re-contract — the draft band's *semantics* are invalidated by the motivation layer's construction; the replacement guards test the real invariant. No magnitude knob pulled anywhere.

## What the sweep asked

`needs-bands.md` v1 (frozen at 2.18) left the self-transcendence row CALIBRATION-PENDING with probe plan `needs_gate_transcendence`, "deferred until CollectiveField lands" — it landed at i266. Target: 0.55–0.65 (derived). This was the last unexecuted probe plan in the band table.

## Measured (3 seeds × {1K, 5K, 20K}, N=12; probes committed)

**`i282_transcendence_sweep` — NeedState-layer autonomy deficit:**

| horizon | mean median | IQR spread | verdict vs draft [0.55,0.65] |
|---|---|---|---|
| 1K | 0.108 | 0.016–0.038 | OUT |
| 5K | 0.128 | 0.085–0.182 | OUT |
| 20K | 0.094 | 0.103–0.436 | OUT |

**`i282_autonomy_dominance` — motivation-layer autonomy pressure:**

| horizon | p10 | median | p90 (=max) | in band | dominant argmax |
|---|---|---|---|---|---|
| 1K | 0.001 | 0.016 | 0.300 | 0/36 | 0/36 |
| 5K | 0.000 | 0.010 | 0.288 | 0/36 | 0/36 |
| 20K | 0.000 | 0.031 | 0.300 | 0/40 | 0/40 |

## The finding: unreachable by construction, not miscalibrated

The motivation layer computes `pressure = deficit × urgency_weight`, capped at `deficit ≤ 1.0`. Autonomy's urgency weight is **0.3** → **pressure ceiling 0.300**. The draft target [0.55, 0.65] is a pressure-threshold range (the table's own semantics: 0.5 Eat/Drink goal, 0.7 Socialize generate…). No agent can ever reach it — **the draft band is structurally dead**, a §4.3-shaped finding in a *document*, not the code. The sibling Esteem row (0.55–0.65, urgency 0.4 → ceiling 0.4) shares the defect.

## What IS live (the re-contracted guards)

1. **Band liveness** (i267's ODE working as built): deficit advances, workload-differentiated — growth 0.001/tick vs Work relief 0.002/tick → equilibrium at work-fraction 0.5; workers sit at deficit 0.05–0.10 (median pressure 0.016–0.031), low-workload agents **saturate at deficit 1.0** (p90 = ceiling 0.300).
2. **Behavioral expression is NOT dominance** (argmax 0/112 — urgency 0.3 correctly yields to survival, the Maslow encoding): autonomy acts through the **grievance coupling** (`compute_grievance`: autonomy_deficit × 0.15, institutions/factions.rs) and **depression risk** ((1−autonomy) × 0.2, person/mind.rs) — both live readers.
3. **CollectiveField/WP-I**: the row's theory citation names the village holon as the *collective* expression (rung 7), which is the collective-field governance line (measured 1.0→2.0 across the sweep horizons, feeding i280's WP-J coupling) — not a per-agent pressure channel. No missing producer.

## The re-contract (§4.4 rule 4)

`needs-bands.md` self-transcendence row rewritten: draft threshold range → **measured deficit equilibrium** (median ~0.10 workers, ≥10% low-workload agents at cap) with the mechanism named (pressure ceiling 0.3 makes the draft threshold unreachable) and the live behavioral channels cited. The probe plans column is now DONE for every row in the table.

## Files

- Probes: `i282_transcendence_sweep.rs`, `i282_autonomy_dominance.rs`
- Doc: `docs/balance/needs-bands.md` (self-transcendence row re-contracted with evidence)
- No code behavior changed → no re-anchors expected; gate must be green as-is.
