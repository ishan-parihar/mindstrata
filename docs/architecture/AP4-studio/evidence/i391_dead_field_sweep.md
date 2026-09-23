# i391 — the write-only field sweep: five dead genes and the instrument that finds them

**Status:** LANDED (measurement — **no behaviour change**, both goldens byte-identical)
· **Root cause owned:** the recurring dead-leg class. It has been found by hand four
separate times — i313 (`injury`), i124 (emotion context), i307 (habit gate) and
`sensory_acuity` — and each time the fix was a one-line wiring *after* a long hunt. This
iteration builds the instrument so the class is found by census instead.

## The instrument: `scripts/field_census.py`

A static field-level write/read census over production code only (`#[cfg(test)]` blocks,
`*tests.rs`, `tests/` and the whole `mindstrata-tests` crate are excluded — a field read
only by its own pin is still a dead consumer in the engine).

Per field it counts **writes** (`x.f = v`, `x.f += v`, …), **literals** (`f: v` in a
constructor) and **reads** (`x.f` in any other position), and then applies the one rule
that makes the class visible:

> A read inside the field's **own file** is *plumbing* — the constructor, `Default`,
> `random()`, `inherit`/`blend_gene`, a serde shim. It proves the value is copied, not
> that any behaviour consumes it. A read that **crosses the module boundary** is the only
> thing that counts as a consumer.

    PASS-THROUGH-ONLY   written, and every read is its own module's plumbing → dead
    DEAD-FIELD          nothing at all touches it
    READ-ONLY           read but never written → set only by deserialization?
    LIVE                at least one read crosses the module boundary

Run it with `--file <path>...` (every struct in the named files, including the nested
predisposition structs), `--sites` (every read site as `file:line`) or `--json`.

**Instrument calibration (honest):** the boundary rule has one false-positive class an
aggregate struct whose *consumer lives in its own file*. `EmbodiedState.metabolic` is the
example in this census — it is flagged, and it is live (`self.metabolic.tick_update(…)` and
`self.metabolic.energy_reserves` are both in `biology/mod.rs`, where the field is declared).
Six suspects were produced and five were confirmed by hand; **every suspect row is a
pointer, not a verdict** — the script prints read sites precisely so the confirmation costs
one grep.

## The census (i391)

Targets: `Genome` + its five nested predispositions, `EmbodiedState`, `BodyState`,
`NeedState`, `DevelopmentFieldState` — 59 fields (61 counting the two duplicates that
`BodyState`/`NeedState`/`EmbodiedState` share).

| struct | field | writes | literals | reads | x-module reads | verdict |
|---|---|---|---|---|---|---|
| `TraitPredispositions` | **`aggression_threshold`** | 0 | 4 | 2 | **0** | dead gene |
| `TraitPredispositions` | **`novelty_seeking`** | 0 | 4 | 2 | **0** | dead gene |
| `HealthPredispositions` | **`chronic_pain_risk`** | 0 | 4 | 2 | **0** | dead gene |
| `PhysicalPotential` | **`sensory_acuity`** | 0 | 4 | 2 | **0** | dead gene |
| `FertilityPredispositions` | **`puberty_age`** | 0 | 4 | 2 | **0** | dead gene (shadowed) |
| `EmbodiedState` | `metabolic` | 0 | 4 | 6 | **0** | **live — false positive** (consumer in its own file) |

All six rows are otherwise `LIVE` (the remaining 53 fields have consumer reads), so the
class is *five genes*, not a systemic rot. The five share an exact signature: drawn at
`random()`, defaulted, and blended at `inherit` — then never mentioned outside
`biology/genome.rs`. Hand-confirmed by full-tree grep:

```
aggression_threshold   genome.rs:{34,49,172,233,234,235}      ← nothing else, anywhere
novelty_seeking        genome.rs:{36,50,173,238,239,240}
chronic_pain_risk      genome.rs:{68,77,182,275,276,277}
sensory_acuity         genome.rs:{111,119,192,309,310,311}
puberty_age            genome.rs:{130,137,196,321,322,323}   + reproductive.rs:168–178
```

## The finding behind the fifth row

`puberty_age` is the interesting one, because its consumer **exists and was hardcoded over
the gene**:

```rust
// crates/mindstrata-person/src/biology/reproductive.rs:168
let puberty_age = Fixed::from_f64(13.0);          // ← the gene draws 11.0–15.0
let maturity_age = Fixed::from_f64(18.0);
```

So the engine already carries a per-agent puberty age (inherited, blended, heritable) and
the reproductive clock reads a village-wide constant instead. That is the i373 audit's
"Class-B promotion" in its purest form — the sim already measures the state the constant
guesses at — and it is the first row of the wiring act below. It is *not* fixed here: the
change is behavioural (conception timing, **goldens, birth pins**) and belongs in its own
iteration with its own probe, exactly as §2.1 requires.

## What this leaves: the wiring act (i392)

Exit criterion for the class is "every writer-only field is either wired or deleted", and
each row is a behavioural change, so they are scheduled as one act with per-field probes
(not one batch commit with an unattributable re-anchor):

| field | verdict | consumer it should have | risk |
|---|---|---|---|
| `puberty_age` | **wire** | the hardcoded `reproductive.rs:168` — thread the gene through `ReproductiveUpdateParams`/the call site | birth timing: goldens + the conception/birth pins |
| `sensory_acuity` | **wire** | the i334 perception radius in `tick_memory_encoding` — perception should scale with acuity | perf (that pass is 17.8% of the tick) + contact-derived pins |
| `aggression_threshold` | **wire or delete** | the anger→violence route (§19.5.G feud / escalation thresholds) — a gene-shaped disposition bar is the §4.6 ideal (midpoint-neutral at gene 0.5) | the violence window is a long-re-anchored calibration |
| `novelty_seeking` | **wire or delete** | the i351 `Wander` driver's exploration bonus | `Wander` share is small but newly live |
| `chronic_pain_risk` | **wire or delete** | the injury→chronic-pain path in `systems/health.rs` | small; likely the cheapest of the four |

Deleting is the honest alternative for any row that cannot be wired midpoint-neutrally
(§4.6): an inert gene is a **false affordance** — it tells the operator that trait
variation exists where the engine behaves identically for every carrier, and it dilutes
every genome-wide distribution claim (the H5 founder-shape debt in particular).

## Pins and verification

The census is a script, not a sim pass, so it changes no behaviour: **both goldens
byte-identical**, no snapshot drift, sim **310/310**, integration **313 passed / 0 failed /
1 ignored**, clippy 0 warnings, `gate --full` GREEN. No new pins — the runnable check this
iteration leaves is the instrument itself (`python3 scripts/field_census.py --file …`), plus
the hand-confirmation commands recorded above and re-run by the wiring act.
