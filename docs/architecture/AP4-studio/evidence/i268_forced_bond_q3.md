# i268 — forced-Bond Q3 golden_addiction calibration + behavioral wiring

Owner: STORY/SIM (PLAN_DC2 Iteration-268). Probe
`crates/mindstrata-benches/examples/i268_forced_bond_q3.rs`. Follows
`i293_20seed_q2q3q4_sweep.md` (Q3 weak-live verdict).

## Question

Q3 golden_addiction fires at all seeds but only reaches mean 0.0329–0.0414 at
5K/12 natural event rates (i293). Does the operator reach its ratified ceiling
band [0.70, 0.90] under sustained Bond pressure, do the current per-quadrant
params (growth 0.07 / decay 0.015 / ceiling 0.85) measure correctly, and what
does wiring Q3's *behavioral* consumer (the grasping → connection nudge)
do to the observables the goldens pin?

## Method

Two regimes × 3 seeds (42, 4242, 2026) × 5K ticks, N=12:

- **natural** — vanilla `sim.run(5000)`.
- **forced/{50,200}** — every K ticks, 2 `MarriageFormed` + 2 `ChildBorn`
  events fed directly to the pure `system_development(agents, events)` pass
  (production `tick()` untouched; the pass is `pub` and pure).

```
cargo run --release -p mindstrata-benches --example i268_forced_bond_q3
```

## Results (mean across agents / final / peak / Bond-line altitude / #agents ever >0.5)

```
            regime       mean      final       peak   bond_alt    >0.5
       natural s42     0.0464     0.0470     0.0470     0.6881        0
     forced/200s42     0.1967     0.2442     0.2442     0.7970        4
      forced/50s42     0.2363     0.2507     0.2507     0.7970        4
     natural s4242    0.0386     0.0391     0.0391     0.4859        0
   forced/200s4242    0.1918     0.2402     0.2402     0.6576        4
    forced/50s4242    0.2322     0.2468     0.2468     0.6596        4
     natural s2026    0.0380     0.0391     0.0391     0.4022        0
   forced/200s2026    0.1913     0.2402     0.2402     0.5713        4
    forced/50s2026    0.2317     0.2468     0.2468     0.5956        4
```

## Findings

1. **Per-subject equilibrium ≈ 0.75 in the forced regime.** The printed mean
   is village-wide (4 carriers of 12), so carrier-level Q3 = mean × 12/4 ≈
   0.72–0.75 — *inside* the ratified ceiling band [0.70, 0.90]. The current
   per-quadrant params need **no re-pin**; the plateau the event-rate-limited
   i293 regime could not see is where the spec put it.
2. **Event-rate sensitivity is the mechanism, not a defect**: natural 0.038–0.046
   vs forced ≈0.25 village-mean. Q3 is live and scales with Bond-catalyst
   frequency exactly as the metabolics predict (addiction saturating growth).
3. **Altitude coupling confirmed**: Bond-line altitude rises concurrently
   (0.69→0.80 at s42), the "grasping while advancing" pattern — altitude is
   NOT gated off, matching the v1 fan-out design (the gate discriminant
   remains a WP-C open question, unchanged here).

## Behavioral wiring: Q3 positive nudge (the missing fourth channel)

Before this iteration `development_pathology_nudge`'s predecessors covered
Q1 (Work −, Rest +), Q2 (Socialize −), Q4 (Worship −) — Q3 had **no consumer**.
Per `pathology-curves.md` Q3 semantics (grasping reaches for connection/
transcendence), the helper now applies **Socialize/Worship +0.04 × Q3** —
the only positive pathology channel. Helper extracted as a pure `pub fn`
so the pin tests all four quadrants without a `DecisionContext`.

Magnitude discipline: at natural mean 0.046 the shift is ≤ 0.0019 (a genuine
nudge, not a reordering lever); at forced-regime equilibrium 0.75 the shift is
+0.03 (comparable to the dread channel at moderate dread) — intentional:
villages swimming in Bond catalysts become visibly connection-seeking.

## Golden/snapshot re-anchor (AGENTS.md §4.2)

The Q3 nudge is a **real behavioral consumer**, so exact-tie argmax decisions
flip and trajectories diverge (butterfly through Fixed ties — expected and
intended for any live nudge). Drift direction confirms the mechanism:

| observable | old → new | mechanism |
|---|---|---|
| relationship stages @2K | Friend 0→2, Ally 47→48 | grasping → more Socialize |
| social memories @10K | 376 → 441 (+65) | same |
| avg_stress @10K | 0.3870 → 0.3498 | connection relieves stress |
| avg_fatigue @2K | 0.4220 → 0.3544 | more socializing, less grinding |
| agent_count @10K | 12 → 13 | more bonding → a birth |
| total_grain @10K | 1.809 → 0.913 | the cost: less provisioning |

The grain cost is the pathology semantics working (grasping competes with
provisioning), not a hazard: avg_health *improves* (0.781→0.790), no starvation
signature. Goldens (`golden/{collapse,riverford_minor}/seed_42/baseline.json`)
and 3 snapshots re-accepted on this evidence. Post-re-anchor: full gate GREEN,
307/0/1.
