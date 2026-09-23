# i392 (row 1) — the puberty gene reaches the reproductive clock

**Status:** LANDED (behavioural, **zero blast radius — measured**) · **Root cause owned:**
row 1 of the dead-gene act i391's census scheduled. `FertilityPredispositions.puberty_age`
was drawn U(11.0–15.0) at founder creation, defaulted, blended at `inherit` — and read by
nothing, because the clock it exists to shape read a constant instead:

```rust
// reproductive.rs, before
let puberty_age = Fixed::from_f64(13.0);   // ← the gene's midpoint, hardcoded
let maturity_age = Fixed::from_f64(18.0);
```

This is the i373 "Class-B promotion" in its purest form: the engine already measures a
per-agent value and reads a village-wide guess in its place.

## The law

`ReproductiveUpdateParams` gains `puberty_age` and `maturity_age`; the clock reads them,
and the caller (`EmbodiedState::tick_update`) feeds the genome:

```rust
puberty_age: self.genome.fertility_predispositions.puberty_age,
..reproductive::ReproductiveUpdateParams::default()
```

**Midpoint-neutral by construction (§4.6).** The gene's draw is U(11.0, 15.0), midpoint
**13.0**, and 13.0 is exactly the constant this clock read. So the default is the retired
constant, the population the old constant described does not move, and only carriers at the
tails do. That is the property the wiring act needs to be landable at all — it is why
`maturity_age` was left at 18.0 rather than invented a second law (the gene's span keeps the
ramp `[3, 7]` years wide, so the division is never degenerate and no clamp is needed).

## Measured (probe `i392_puberty_clock`, 20 000 ticks)

**The gene has real variation to begin with** — this is the *manipulation* half of §4.13:

| world | n | mean | min | max | spread |
|---|---|---|---|---|---|
| village s42 | 12 | 13.512 | 11.285 | 14.970 | 3.684 yr |
| village s07 | 12 | 12.875 | 11.154 | 14.434 | 3.280 yr |
| town s42 | 48 | 13.345 | 11.072 | 14.970 | 3.897 yr |

**The band the gene governs is empty in every corpus** — the reason the wiring cannot move a
calibrated window, measured rather than assumed:

| corpus | agent-ticks in age [10, 16) | total agent-ticks | age range |
|---|---|---|---|
| village s42 | **0** | 240 000 | 20.5–54.0 yr |
| town s42 | **0** | 987 352 | 0.0–54.8 yr |

`TICKS_PER_YEAR = 35 040` and founders are drawn U(18–55), so no agent occupies the puberty
ramp at t=0, and a child born at t=0 first enters it at **~385 000 ticks** (11 yr). Every
calibrated corpus is far below that: the goldens are 1 000 ticks, the long-horizon suite
50 000, the deepest behavioural windows 100K. **The wiring is invisible in all of them by
construction.**

**The clock itself is live** — the per-agent manipulation check, three agents all aged 12.0:

| arm | gene | stage | `sexual_maturity` |
|---|---|---|---|
| early tail | 11.0 | `Early` | **0.1428** |
| late tail | 15.0 | `Prepubescent` | 0.0000 |
| retired constant | 13.0 | `Prepubescent` | 0.0000 |

The third arm is the point: an agent at the retired constant behaves **exactly** as it did
before the wiring, so midpoint neutrality is *measured*, not asserted — and the same age on
the early gene is already 14% through puberty. The unit test adds the tail boundary
(`puberty_onset_follows_the_params_gene`).

## Honest scope: wired and live, not yet observable in a corpus

This iteration cannot report a behavioural delta in any shipped corpus, and does not pretend
to. What it reports is: the manipulation is live at the clock (table above), the affected
band is measurably empty (table above), and therefore the target of the fix is the **next
generation at horizons beyond ~400K ticks** — where a child's reproductive onset will follow
its own inherited gene instead of a village-wide constant. Recording that plainly is the
§4.2 requirement; the alternative (dressing a zero-effect wiring as a realism win) is the
calibration dishonesty the doctrine exists to prevent.

## Blast radius: zero, verified

Both goldens **byte-identical** (`riverford_minor` s42, `collapse` s42), no snapshot drift,
sim lib **310/310**, integration **313 passed / 0 failed / 1 ignored**, clippy 0 warnings,
`gate --full` GREEN. Zero re-anchors, zero re-contracts — the band measurement above predicts
exactly this, so the clean suite is confirmation rather than luck.

## Pins (2 new, `biology::reproductive::tests`)

- `puberty_onset_follows_the_params_gene` — a 12-year-old is mid-puberty (`Early`, 0.1428) on
  the early gene and `Prepubescent` on the late one.
- `the_default_puberty_age_is_the_gene_midpoint` — the default stays the pre-i392 constant
  13.0 and a 12.9-year-old is still pre-pubescent under it.

## Remaining rows of the act (i392, not yet landed)

| field | consumer it should have | risk |
|---|---|---|
| `sensory_acuity` | the i334 perception radius in `tick_memory_encoding` | perf (that pass is ~17.8% of the tick) + contact-derived pins |
| `aggression_threshold` | the §19.5.G anger→violence escalation bar | the violence window is a long-re-anchored calibration |
| `novelty_seeking` | the i351 `Wander` driver's exploration bonus | `Wander` share small but newly live |
| `chronic_pain_risk` | the injury→chronic-pain path in `systems/health.rs` | smallest; likely cheapest |

Each is midpoint-neutral only if its gene's midpoint equals the constant it replaces —
`sensory_acuity` (draw 0.2–0.8, mid 0.5) and `chronic_pain_risk` (0.0–0.5, mid 0.25) qualify
only if the constants they shadow sit at those midpoints; where they do not, the wiring needs
a **rescale with probe evidence**, or deletion is the honest option. An inert gene is a false
affordance — it advertises heritable variation in a trait the engine treats identically for
every carrier, and it dilutes the H5 founder-shape debt in particular.
