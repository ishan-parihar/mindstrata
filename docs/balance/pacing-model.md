# Pacing Model — What the Player Experiences (skeleton)

Owner: DESIGN. Companion to canon-inventory row 6; feeds the UM-1 playability bar.

## Evidence base

- Tick economics: release-mode suite runs 100K+ tick probes routinely (AGENTS.md §5);
  formal ticks-per-second harness lands with the benchmark task (FR-006) — numbers here
  stay qualitative until that harness records baselines.
- Long-horizon probes already in tree: `p5_100k_probe.rs`, golden worlds at seed_42
  (`golden/collapse`, `golden/riverford_minor`).

## Horizons to characterize (each cites ≥1 probe artifact once authored)

| Horizon | Expected player-visible state | Probe pointer |
|---|---|---|
| 10K ticks | household formation, first factions, norm verdicts visible | existing p5-family probes |
| 50K ticks | faction crisis-pressure cycles, marriage webs, epidemic potential | i<iter>_midcourse |
| 100K ticks | multi-generation lineage, culture divergence, ideology drift | p5_100k_probe + culture-disjointness probe |

## Open questions

1. Where does AP3 development-stage visibility first change decisions (Era II live)?
2. Session length targets: what wall-clock minutes correspond to each horizon at the
   harness-measured tick rate?
3. Which canon levers (difficulty catalog) pace which horizon?
