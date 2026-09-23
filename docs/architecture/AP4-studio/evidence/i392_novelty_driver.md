# i392 row 2 — the `novelty_seeking` gene is measured against its named consumer, and rejected

**Status:** LANDED (measurement — **no behaviour change**, both goldens byte-identical, suite
313/0/1) · **Root cause owned:** row 2 of the dead-gene act i391's census scheduled. The plan
named the consumer to try: *"`novelty_seeking` → the i351 `Wander` driver's exploration
bonus"*. The wiring was built, measured, and **reverted**; this document is why, and the
precondition it exposes.

The doctrine this iteration is obeying is the one that made i380 a landing rather than a
backlog item: a probe that refutes a candidate repair is a result, and the alternative —
shipping the repair and re-anchoring what it breaks — is the failure mode §4.1 and §2.3 exist
to prevent.

---

## 1. The candidate that was evaluated

The i391 census found `novelty_seeking` drawn at `random()`, defaulted, blended at `inherit`,
and read by nothing outside `genome.rs`. It is a heritable trait the engine advertises and
ignores. The natural consumer is the i351 exploration driver, whose coefficient was
population-wide:

```rust
// actions/mod.rs — the shipped term
if *kind == ActionKind::Wander && ctx.novelty_pressure > Fixed::ZERO && ctx.needs_quiet {
    terms.add(UT_DRIVER, ctx.novelty_pressure * WANDER_NOVELTY_COEF);   // coef = 2.0
}
```

The candidate law makes the coefficient heritable, **midpoint-neutral by construction (§4.6)**
— exactly `1.0` at the gene's `Default` (0.5):

```text
coef = WANDER_NOVELTY_COEF × (1 + (gene − 0.5) × span)
```

Reproduction: this is the whole patch. `DecisionContext` gains `novelty_seeking: Fixed` (fed
from `agents[i].embodied.genome.trait_predispositions.novelty_seeking`; the nine test
constructors pass `0.5`), and the term above becomes `… * WANDER_NOVELTY_COEF *
novelty_driver_scale(ctx.novelty_seeking)`. Everything below was measured with it in tree.

Two spans were measured. `0.8` (coefficient 1.36–2.64) was rejected immediately: the sweep
below shows what a 28× spread in exploration does. `0.4` (coefficient **1.68–2.32**) is the
conservative variant and is the one the tables report — **it is also rejected.**

---

## 2. Leg A — the manipulation is real (§4.13): the gene varies, and it bites

| world | n | gene mean | gene range | multiplier range | implied coefficient |
|---|---|---|---|---|---|
| village s42 | 12 | 0.355 | 0.120–0.838 | 0.848–1.135 | 1.696–2.270 |
| town s42 | 48 | 0.466 | 0.113–0.863 | 0.845–1.145 | 1.690–2.290 |

The helper is exact at the reference points: gene 0.1 → 0.68, **0.5 → 1.0000**, 0.9 → 1.32
(span 0.4: 0.84 / 1.0000 / 1.16). Note the realized founder means (0.355, 0.466) sit *below*
the draw's 0.5 midpoint at these N — the §5 small-N variance already on record.

## 3. Leg B — the channel is live and monotone

Same world, same seed, one number pinned. **`0.5` is the control: it is the pre-i392 engine
exactly** (multiplier 1.0).

| world | gene 0.1 (homebody) | 0.5 (**control**) | 0.9 (seeker) | monotone |
|---|---|---|---|---|
| N=12 s42 | 132 Wander decisions | 2 464 | 3 927 | ✓ |
| N=48 s7 | 87 | 8 562 | 16 879 | ✓ |

The channel works. That is not the problem.

## 4. Leg D — the response surface is **elastic**, which is the problem

The same world and seed with the gene pinned across its whole range, candidate span in place:

| gene | 0.1 | 0.2 | 0.3 | 0.4 | 0.5 | 0.6 | 0.7 | 0.8 | 0.9 |
|---|---|---|---|---|---|---|---|---|---|
| coefficient | 1.68 | 1.76 | 1.84 | 1.92 | 2.00 | 2.08 | 2.16 | 2.24 | 2.32 |
| Wander decisions | 900 | 1 194 | 1 531 | 1 853 | 2 464 | 2 946 | 3 256 | 3 593 | 3 718 |

Monotone, and **elasticity ≈ 4**: a 1.18× coefficient step buys a 1.32× behavioural step
(1.92 → 2.16 gives 1 853 → 3 256, 1.76×), and the gene's full range is a **4.1× spread in
exploration**. There is no threshold here — but there is amplification, and the amplification
is what the aggregate hides: the population mean multiplier is 0.942 (N=12) / 0.987 (N=48),
i.e. the *average* agent moves almost not at all while the top carrier's coefficient rises to
2.29 and the bottom's falls to 1.69.

## 5. Leg E — attribution is exact, and the midpoint claim is proven at machine precision

The golden scenario itself is the cleanest available control. `riverford_minor`, seed 42,
1 000 ticks, 12 agents, 16×16:

| arm | metric_hash | vs stored golden |
|---|---|---|
| control (gene pinned 0.5) | `0xb2a1be3b18fbb46d` | **EXACT — byte for byte** |
| stored golden baseline | `0xb2a1be3b18fbb46d` | — |
| natural genes (candidate law) | `0xd5e3a30627a9faff` | diverges |

So the candidate is *verifiably* identity at the midpoint — §4.6 satisfied, not asserted — and
the divergence in the natural-gene world is the gene's spread and nothing else. The first
divergent agent-position tick is **tick 657, agent 0 alone**, whose gene is 0.8381 →
multiplier 1.1352: the world's only high carrier. The mechanism is named, not inferred.

## 6. What the world does with it — the rejection

| instrument | control (no gene) | candidate (span 0.4) |
|---|---|---|
| full integration suite | 313 passed / 0 failed | **8 failed** |
| goldens | byte-identical | both changed (`riverford_minor`, `collapse`) |
| snapshots | accepted | 5 shifted (agent state/metrics/institution/relationship-stages/long-horizon) |
| **revolution liveness family** (3 seeds × 70K, `pestilence`) | passes (`firing ≥ 2`) | **`[(5, 0, 2, 0), (42, 1, 3, 1), (12345, 0, 3, 0)]` → firing 1 of 3** |

The last row is the decision. AGENTS §2.3: *"If a test fails because a producer went dead,
revive the producer — do not re-pin the assertion to accept zero."* Two of three seeds stop
producing revolutions; the assertion's own `firing >= 2` is a **liveness** pin on the
governance layer, and re-anchoring it to `firing >= 1` would be precisely the forbidden
re-pin. The snapshots and goldens are the *same* shift (exploration re-paces contact, contact
drives relationships, relationships drive grievance), which means the blast is not confined to
the action surface — it reaches the political layer.

**Verdict: rejected.** The clean goldens, the byte-exact midpoint control and the live channel
are not worth a governance-liveness regression, and no amount of re-anchoring attributions
makes the exploration rate a function of the founder draw a *feature* at N=12.

## 7. What the probe found on the way (the actual root cause)

Running the driver's **own** acceptance instrument (`i351_wander_bands`, same 32×32 world,
same horizons) against the **current, unmodified** tree:

| horizon | seed 42 | seed 7 | i351's documented target band |
|---|---|---|---|
| 2 000 | 2.82% | 1.57% | — |
| **20 000** | **5.44%** | **3.53%** | **0.5–3% of decisions at 20K** |

The band the coefficient `2.0` was chosen for **no longer holds**: the measured share is
1.2–1.8× the band's ceiling. Nothing pins it — `i351_wander_bands` is a probe, not a test, so
no gate ever re-measured it (the same class as the i386 citation drift: a claim with no
enforcement). The action layer has moved since i351 landed (i356 Idle driver, i387/i388
utility decomposition and the relational urgency family, i389 decree, i390 succession), and
the exploration share moved with it.

This is why the row cannot be closed by wiring: **the surface has no valid calibration for a
heritable trait to ride.** A rejection with the numbers beats a wiring on an uncalibrated
knob, and the recalibration is a root cause with its own probe — recorded in the ledger as the
blocking precondition for rows 2–5.

## 8. Verification

Reverted tree: `cargo fmt --all` clean, clippy 0 warnings, sim **310/310**, integration
**313 passed / 0 failed / 1 ignored**, both goldens **byte-identical**, `gate --full` GREEN,
`doc_index.py` 0 ghosts. The only artifact this iteration leaves is the instrument:
`cargo run --release -p mindstrata-benches --example i392_novelty_driver` (legs A–E) and the
existing `i351_wander_bands` band readout.

## 9. Ledger consequences

- **Row 2 (`novelty_seeking`): measured and rejected** — not wired, not deleted. Deletion was
  the other honest option; it is deferred rather than taken because the trait has a genuine
  consumer and a *fixable* blocker (§7), and deleting one row of five would leave the act
  internally inconsistent.
- **New root cause recorded, and it precedes rows 2–5:** the exploration driver's own band
  (§7) must be re-contracted or re-calibrated with a probe before any heritable trait rides
  it. Suggested as **i393**, ahead of the speech-act migration.
- **New measurement, recorded for the act's remaining rows:** a consumer's *response
  elasticity* is part of its suitability. `sensory_acuity` (perception radius), 
  `aggression_threshold` (the escalation bar) and `chronic_pain_risk` (the injury→chronic
  path) must each clear the same two gates before wiring — (a) the constant it shadows sits
  at the gene's midpoint, and (b) the response to that constant is *not* elastic, or the
  trait becomes a population-level lottery.
