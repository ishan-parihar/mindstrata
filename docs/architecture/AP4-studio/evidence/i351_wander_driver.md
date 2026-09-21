# Iteration 351 — A8 closed: the exploration driver, a unit bug the census caught, and a sweep of re-anchors

**Status:** LANDED (behavioural) · **Root cause owned:** A8's design act (give
`Wander` a driver) plus the blast-radius sweep that the driver's locomotion
guaranteed: every pin written while agents never moved had to be re-measured.

## The design (probe-first)

`i351_wander_driver` measured the anatomy: `Wander` carries zero relief on
every channel, loses all arbitrations by 1.2–1.6, and `MotiveCategory::Novelty`
competes in `update_dominant` with **no relief mapping** — a dead dominant
motive. The design: a `novelty_pressure × coef` term gated on need quietude
(`max(hunger, thirst, fatigue) < 0.5`), `Novelty` wired into the §8.1.5
urgency match, and a relief write-back (`novelty −0.008`, `autonomy −0.003`
per completed wander) so the driver satisfies its own pressure — a loop, not
a ratchet.

## The unit bug (the important lesson)

The first draft shipped `WANDER_NOVELTY_COEF = Fixed::from_raw(2000)`
labelled "2.0" — but this codebase's `Fixed::from_raw` scale is **1e-4 per
1.0**, so the constant was 0.2. The driver delivered ~0.06 against a measured
quiet-window wall of mean 0.71 / max 1.40 and won nothing.

What caught it: the in-vivo census recorded **0 wins** while the offline
sweep over the *same realized samples* (`quiet_sweep`) predicted 56 wins at
c=2.0. The contradiction could only mean the term was not reaching the
argmax — and the sweep math was exact, so the constant was wrong. Fixed to
`from_raw(20_000)`; the sizing comment now carries the measured wall and the
incident.

**Rule:** every new `Fixed::from_raw` literal gets a runtime assertion or an
in-vivo confirmation — a labelled comment is not evidence.

## Sizing, in-band

Post-fix (`i351_wander_bands`): Wander share **2.65% / 1.64%** @2K (seeds
42/7), **5.39% / 3.18%** @20K. The 20K share sits above the guessed 0.5–3%
band but the displacement reading is benign: Work −3.8pp, Socialize −1.4pp,
no Idle wins anywhere (nothing was reclaimed — locomotion is new behaviour),
and the relief loop self-limits (in-vivo wins match the sweep at c=1.0 —
novelty deficit never accumulates). Provisioning holds (grain 1.0197 on the
10K surface); no saturation anywhere.

## The sweep, classified

The driver makes locomotion live for the first time since §6 movement
existed. Six pins written against the frozen-locomotion world moved:

| pin | shift | class | re-anchor |
|---|---|---|---|
| i314 pain-veto reach (`violence_records_injury_on_the_substrate`) | severe-wound ceiling compressed 0.9+ → 0.788–0.824 across 8 seeds; the 0.9 veto went **dormant** (0 agent-ticks) | §4.3 dead guard — NOT a re-pin | `PAIN_VETO_THRESHOLD` 0.9 → **0.75** (`i351_wound_band`: fires 0.083% of agent-ticks, inside the i314-ratified 0.04–0.31% band; 0.70's 0.226% keeps 2.7× margin over the pre-driver pin-drift zone) |
| i313 bounded divergence (`emotional_body_tone_resists_regulation_in_tick`) | conflict divergence 2 → 27 (132 vs 156); arousal ordering inverted at seed 42 | §4.4 re-contract ×2 — the somatic channel gained a behavioural lever | `i351_somatic_sign` (8 seeds): divergence NOT signed (2/8 fewer, 4/8 equal), arousal ordering 8/8 **per unit of conflict exposure**; pin guards Δ≤40 pacing + per-exposure ordering |
| fear-contagion reach (`sensory_field_fear_contagion…`) | 12/12 perceive → 11/12 (an agent alone at the daily snapshot reads `perceived_stress = 0`) | §4.4 re-contract — locality semantics, i349/i350 family | `i351_presence` (8 seeds, 16×16): band 11–12/12; ≥11/12 asserted; contagion positive for perceiving agents |
| meme registry freeze (`meme_registry_seeds_founding_memes`) | active 3 → 4: a genesis commemoration registered when the collective Safety line crossed stage 2.0 | §4.4 re-contract — the registry was frozen *before* i297's genesis landed | no seeded meme ever deactivated + genesis growth band-bounded (≤ seeded+4) |
| meme transmission monotonicity (`meme_transmission_multiplier…`) | baseline 4 vs high 3 — the epoch-crossing tie flipped the count | §4.4 re-contract — the count read genesis, not transmission | count floor ≥3 on the founding set + new `…_increases_host_adoption` pin on the multiplier's real surface (total hosts strictly increasing; measured 3.0× fully adopts 12/12) |
| moral-panic family (`moral_panic_lifecycle…`, `collective_fear_amplifies…`) | i343 family {5,7,42} → {7 fires 11, 42/5 fire 0}; {11: 5, 46: 2} fire | §4.4 re-anchor — the family IS the thing that moves when pacing shifts | family = {7, 11, 46} (`i351_panic_events`); determinism legs move with it (beliefs_memory replays seed 11, governance seed 7) |
| marriage-rate liveness (`marriage_formation_rate_parameter_is_live`) | 0.0002 no longer marries on seed 42 by 5K (trust diet re-paced: 88/132 pairs contacted vs 52/132 pre-driver) | §4.2 re-anchor | `i351_marriage_rate` sweep: **0.0004** → 4 marriages (first @453), non-saturating vs 0.1's ceiling; differential contract preserved one step up the axis |

## Baselines

Both goldens regenerated (riverford_minor `df3f…`, collapse `70bc…`) —
**agent_count 12 preserved** on both, crisis mortality check intact. Five
snapshots accepted after diff review: shifts bounded (10⁻⁴–10⁻²), all in the
driver's direction (energy/hunger re-pacing, stage-distribution reshuffle,
trust de-pinning 0.706→0.674-class drift, meme count 7→5 from re-timed
genesis crossings), no saturation anywhere.

## Census infrastructure (kept)

`decision_census` gained the quiet-window split: `quiet_samples`,
`quiet_wander` gap stats, winner-utility histogram, and `quiet_sweep(coefs)`
— the offline coefficient sweep over realized samples that sized this driver
and caught the unit bug. Instrumentation-only, byte-identical when disabled.

## Ledger

1. **Gate tooling debt** (carried from i349): clippy without `--all-targets`
   still hides broken test cfg in other crates.
2. **The 20K Wander share (5.39%) sits above the §4.2 band's top** — recorded
   as measurement, not debt: the guessed band was ratified before the wall
   was measured. Re-visit only if displacement shows up in provisioning.
3. **A9 (Idle) unchanged** — Idle still wins 0 arbitrations; its only relief
   (0.05 fatigue/tick) cannot reach argmax. Same design-act shape as A8 if
   pursued.
4. Marriage-gate trust reads the **v1 matrix** (`self.relationships`), not the
   v2 store — a remaining dual-store consumer for the i341 ledger.
