# Iteration 314 — the exertion veto regains a reachable crisis signal

**Status:** LANDED (behavioural) · **Root cause owned:** i312's queued item — the
i255/i309 "a body at its limits does not exert itself" veto was dormant because
its only trigger (`derived health < 0.25`) was unreachable after i311, and no
absolute health threshold can separate crisis from calm (the distributions
overlap).

i313 made the pain channel live. Pain is structurally crisis-only — it exists
only where a wound does — so it is the signal the veto needed.

## Threshold calibration (`i314_pain_veto`, 12 seeds, every tick)

| context | p50 | p90 | p99 | max | ≥0.5 | ≥0.7 | ≥0.9 |
|---|---|---|---|---|---|---|---|
| pestilence @4 320 | 0.0000 | 0.0000 | 0.647 | 1.000 | 1.55% | 0.75% | 0.18% |
| pestilence @20K | 0.0000 | 0.0000 | 0.470 | 1.000 | 0.94% | 0.42% | 0.06% |
| collapse @20K | 0.0000 | 0.0000 | 0.484 | 1.000 | 0.97% | 0.42% | 0.05% |
| drought @4 320 | 0.0000 | 0.0000 | 0.708 | 1.000 | 2.09% | 1.05% | 0.31% |
| calm @20K | 0.0000 | 0.0000 | 0.560 | 1.000 | 1.16% | 0.53% | 0.10% |
| calm @50K | 0.0000 | 0.0000 | 0.155 | 1.000 | 0.56% | 0.25% | 0.04% |

p90 is 0.0000 everywhere: 90% of agent-ticks are pain-free, so any threshold is
a genuine crisis band. Calm is not excluded — a calm village still fights
("trauma/drama", Iter-185), so occasional firing there is correct.

## Threshold selection (blast radius as evidence)

The band is chosen on measured drift, not aesthetics:

- **0.7** (0.25–1.05% firing) drifts **11 pins** — both golden baselines
  (`riverford_minor`, `collapse`), two culture integration tests
  (`meme_registry_seeds_founding_memes`,
  `meme_transmission_multiplier_affects_meme_count`) and seven snapshots. Too
  broad for a crisis guard; it becomes a routine action governor.
- **0.9** (0.04–0.31% firing) drifts **one pin** — the 10K long-horizon surface
  (`agent_count 13 → 12`). Both goldens and every integration test pass.

0.9 is therefore the guard's band: rare enough to be crisis-only, reachable
enough to be live.

## Change

`exertion_vetoed(health, pain) = health < HEALTH_CRITICAL_THRESHOLD ||
pain >= PAIN_VETO_THRESHOLD`. The health clause is retained as a dormant safety
net (i312); the pain clause is the live trigger. When true, exerting actions
(`Work`/`Wander`) are downgraded to `Rest` after every selection path — the
i309 structure, unchanged. Non-trapping by construction: pain clears when the
wound heals (i313), unlike the old i255 Rest mutex that locked on `derived
health`.

## Drift re-anchor

One snapshot: `long_horizon_surface_10000_ticks` (`agent_count 13 → 12` and the
associated relationship/memory distributions). Mechanism: a wounded agent whose
pain crosses 0.9 rests instead of working, shifting the deterministic
trajectory's RNG stream. Golden baselines are byte-identical (no violence in
their windows crosses the band).

## Regression pins

- `mindstrata-sim::sim::pass_action::exertion_veto_tests` — fires on severe pain,
  not below it; keeps the dormant health safety net.
- `violence_records_injury_on_the_substrate` now also asserts the recorded
  violence reaches the 0.9 band, proving the guard is reachable in a real run.

## Verification

`cargo fmt --all` clean · clippy **0** · sim **280/280** · person 120/120 ·
tests **309/0/1** · `scripts/gate --full` **GATE GREEN**.
