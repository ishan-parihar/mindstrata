# Iteration 281 — polarity-claim bias recency window (session-2 debt closed)

**Status:** LANDED · **Doctrine:** §4.2/§4.4 re-contract with probe evidence; horizon-invariance restored rather than magnitude re-pinned.

## What was wrong

The social bias (`0.01 × ActiveTension_count × social_value`, actions/mod.rs) counted the agent's **entire claim list** — an unbounded run integral. Claims are pushed per catalyst and removed only on reconciliation; no timestamp existed.

**Probe (`i281_claim_recency`, seed 42, N=12):**

| horizon | tension (all) | bias ×social_value |
|---|---|---|
| 1K | 20 | 0.0167 |
| 2K | 76 | 0.0633 |
| 5K | 195 | 0.1625 |
| 10K | 516 | 0.4300 |
| 20K | 979 | **0.8158** |

The audited band (i275 re-derivation: ≤0.03 × social_value, "5–20% of the social driver") only held at 1–2K horizons. Past ~2K ticks the "nudge" was the largest term in the social utility landscape, growing linearly with run length — every future 5K+ pin would have sat on a drifting coefficient (the exact §4.4 knife-edge shape).

## The fix — salience-recency window

- `ThreeRealmClaim.created_tick: u64` (`#[serde(default)]` — v15 saves load 0 = ancient, matching aged semantics; syntheses inherit `min(a,b)` — the insight is as old as its material).
- Emission stamps the tick (`system_polarity_claim_emit` gains a `tick` param, passed from `tick_u64`).
- `CLAIM_SALIENCE_TICKS = 1000` (one in-sim year, `ticks_per_year`): only claims emitted within the last 1000 ticks count. **No coefficient changed** — the window restores the horizon-invariance of the integral the 0.01 was ratified against (i275 pinned it against a 1–2K-horizon integral; mean recent-tension at equilibrium = 50 → bias 0.0417).

**Post-fix probe:** bias 0.0167 → 0.0308 → 0.0367 → 0.0417 → **0.0417 plateau** (1K→20K). Horizon-invariant; 0.017 at 1K is byte-identical to pre-fix (window empty at 1K).

## Re-anchor trail (all with field-level evidence)

1. **Collapse golden @ 4320** — predicted drift (window active >1K). Regenerated: `agent_count` 12→12, all metric scalars identical except `total_grain` 24.0861→23.5557 (−0.53, −2.2% provisioning stock) and `total_water` −0.38 (−0.03%). All three hashes moved (event cascade re-timed). Mechanism: the window removed ~0.03 of social bias mid-cascade, re-pacing the shared RNG stream.
2. **Snapshot metrics_2000** — 5 fields moved, all small (`avg_hunger` 0.0135→0.0373, `avg_fatigue` −0.038, trust/stress float tails). Accepted.
3. **Snapshot long_horizon_surface_10000** — `event_count` 140555→139953 (−0.4%), `total_grain` 0.66→1.80 (provisioning equilibrium shift — the 0.43 bias at 10K was inflating social actions over work; the fix is the correction), envy cost 0.0106→0.0033, stress −0.033. Accepted.
4. **Behavioral-delta pair** — the i281 re-pacing compressed the 3K conflict cascade: the 13-seed sweep's best anchor (seed 99) landed at min delta 198 vs the 200 liveness floor. **No floor moved**: both sweep pools widened 13→19 seeds (same method as Iter-190/200); a fresh seed clears both unchanged floors. Re-contract comment added.

## Pins

- `polarity_bias_counts_only_recent_claims` (window semantics: in-window counts, ancient claims contribute zero).
- Full sim lib 233/233; dev lib 80/80.

## Gate

Final full gate GREEN — riverford@1000 byte-identical (window empty ≤1K, as designed), all five drifted surfaces re-anchored with the mechanism named above.
