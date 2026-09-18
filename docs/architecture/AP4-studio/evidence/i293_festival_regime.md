# Iteration 293 — UM-2 diet realism: festival-dense regime (PLAN_DC3 §4 i293, A5)

**Status:** LANDED · **Doctrine:** probe-only iteration (zero production edits —
forcing lives in the probe, §4.4-clean). The plan's binary question resolved with a
verdict richer than either branch: **close the i278 PARTIAL as diet-pacing, ratify
N=12/~100K as the designed cultural horizon, and record the festival-pace cost.**

## The question

i278 measured UM-2 at 20K as PARTIAL (only Safety generates; jaccard 0.369); i283
corrected the pacing model (per-event press = 1/n × 0.05) and PASSED UM-2 at ~100K
with Relational ~54K / Identity ~44–74K stage-2 arrival. The plan's A5 accelerator:
a recurring Relational+Identity festival regime at 20K — does ≥3-bucket generation
close by 20K without a production magnitude knob?

## The probe (`i293_festival_regime`)

Festival forcing through the **production** `CollectiveLineState::step` path
(`CollectiveParams::pending()`), every duodeca (the ritual boundary), at
pressure = attendance fraction. No production constant touched.

### Pace result: ≥3 buckets by 5K, all seeds

| regime | 5K | 10K | 20K | buckets @20K |
|---|---|---|---|---|
| natural (control, s42) | 1 | 2 | 8 | Safety only — replicates i283 |
| festival 0.5 / 12t (s42/43/44) | 22–23 | 23–26 | 23–30 | **Identity+Relational+Safety** |
| festival 1.0 / 12t (s42) | 23 | 26 | 30 | Identity+Relational+Safety |

The i278 PARTIAL verdict is thereby closed: the mechanism was never dead — the
natural N=12 diet is thin. `system_norm_proposal`-style breadth is not the
constraint; press cadence is.

### Diversity result: uniform press homogenizes

| pairing | mean jaccard |
|---|---|
| natural 100K (i283) | 0.158 |
| **uniform festival** (0.5/12t, 3 seeds) | **0.822** |
| varied schedule (40% duodecas, 0.5) | 0.785 |

Mechanism (verified in source): genesis text = fixed template + `epoch %
referents.len()` citation, and institution/site names are world-constants
("Village Council", "Village Well"…). Seed differentiation rides **only** the stage
trajectory; identical press schedules erase it. The varied schedule recovers only
~0.04 — schedule noise is second-order next to schedule identity.

## Verdict (recorded, not flip-flopped)

1. **i278 PARTIAL → CLOSED**: festival-dense regimes reach ≥3 buckets by 20K (5K
   in fact) with zero production edits. Diet-pacing, not dead machinery.
2. **Designed cultural horizon RATIFIED**: N=12 natural gait ~100K stands (i283).
   Chasing 20K naturally would require the §4.4-forbidden magnitude knob.
3. **Festival pace buys speed at diversity's expense** — the quantitative form of
   §4.4's anticipation: press-synchronized villages converge on identical genesis
   epochs. An accelerator regime is therefore NOT valid UM-2 disjointness evidence.
4. **Forward path (i296+ UM-3)**: multi-village worlds press each village through
   its own catalyst stream (Bond/Grief events are per-village), making press
   timing seed/village-specific by construction — the natural cure for the
   homogenization found here. Citation-level differentiation additionally needs
   per-village institution/site naming (today's roster is world-constant);
   recorded for the UM-3 backlog.

## Verification

Probe-only: fmt clean, clippy 0 warnings, golden 5/5 byte-identical by
construction, suite untouched (307/0/1 from i292 stands; bench law 113/0).
