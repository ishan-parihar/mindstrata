# The 2026-09-25 town-scale A/B — scale-envelope drift attribution, RESOLVED

**Status:** measurement complete (2026-09-25, 03:10–03:35, idle host, sequential
old/head interleave, 3 reps per arm, per-run exit codes captured). This closes
the **open attribution** recorded in `ENGINE_STATUS.md` §8 on 2026-09-24 and is
the first input of the §4.1 **i423 standing-envelope** row.

Protocol: `558538d` (the i359-era commit the recorded table was measured on)
archived at `/tmp/ms_ab_i359` vs HEAD (`ee2726c` at run time), both arms built
`--release`, each arm runs its own `i359_town_scale` probe (identical
instrument code), interleave **O,H,O,H,O,H**, same window, min-of-3 leg A
(2K-tick villages) per run. No other cargo processes were alive at start.

## Results — µs/tick, leg A (min-of-3 internal reps per run)

| N | old r1 | old r2 | old r3 | head r1 | head r2 | head r3 | paired h/o ratios |
|---|---|---|---|---|---|---|---|
| 48 | 446.9 | 447.5 | 437.6 | 456.8 | 473.5 | 448.3 | 1.02 / 1.06 / 1.02 |
| 96 | 1116.7 | 1080.2 | 1087.0 | **1900.5** | 1082.8 | 1096.7 | **1.70** / 1.00 / 1.01 |
| 144 | 2052.9 | 1971.9 | 2020.3 | 2188.9 | 2312.5 | 2219.1 | 1.07 / 1.17 / 1.10 |
| 192 | 3547.5 | 3442.0 | 3431.1 | 3854.2 | 3920.6 | **7269.6** | 1.09 / 1.14 / **2.12** |
| 256 | 6561.9 | 6582.9 | 6093.5 | 7143.2 | 6919.3 | **21990.0** | 1.09 / 1.05 / **3.61** |

(Bold = contaminated/spike-state readings; see below.)

## Verdict

1. **The 2026-09-24 "drift" (head 1.85×–3.24× the recorded table) is
   reclassified: host-state spikes, not code.** In this protocol every
   uncontaminated paired position reads **head/old = 1.00–1.17** (mean ≈1.08 at
   N=144/192/256). Yesterday's fresh-HEAD readings (4,448 @144 / 7,724 @192 /
   18,808 @256) sit *inside* the spike state the head arm entered in run 3
   (7,270 @192 / 21,990 @256) and brushed in run 1 (N=96). The drift numbers
   were spike-state samples presented as quiet-state ones — which is precisely
   the failure mode the load-pinned protocol was written to exclude.
2. **A residual ~5–17% (mean ≈8–9%) town-tier delta IS arc-owned — but not
   attributable to any single iteration**: the two arms differ by ~40
   iterations *and a world-law change* (the old arm is pre-i360 — one
   settlement, `gap 12/20 → 0` partitions vs HEAD's clustered 3–4-polity
   world), so the same N runs a structurally different workload. The honest
   unit is **cost-at-town-scale across the arc**, not a regression caused by
   any named delta. Candidates for its share (i376's daily sync pass, i399's
   dyadic reads in O(N²) passes) remain hypotheses only; this protocol cannot
   and does not decide them.
3. **The spike state is the real open finding — with an ordering caveat
   attached.** Head-arm runs entered a sustained 2–3.6× state twice in three
   runs (N=96 in r1; the whole N≥192 tail in r3); the old arm never did
   (spread ≤3% at every size). **Protocol limitation:** the fixed O,H pair
   order always ran head second in its pair, and head is the slower/longer
   arm — so head systematically inherits whatever state old's run just left
   (thermal, page cache, allocator), and the arm asymmetry may be ORDERING,
   not arm. The 2-of-3 vs 0-of-3 count is a **hypothesis for i423 to test
   with a balanced/randomized order**, not a finding about HEAD. What this
   protocol does prove: the spike state EXISTS (regardless of which arm
   triggers it), its signature matches every "drift" reading taken to date,
   and quiet-state readings are 1.00–1.17× paired. With n=3 it cannot decide
   host-environmental vs code-triggered either way. **i423 must record
   spike-state frequency and trigger conditions (with a balanced run order),
   not just quiet-state medians** — a median-only envelope will certify a
   machine that is fine on average and unusable for a third of runs.
4. **The recorded i359-era table remains approximately valid at quiet
   state**: today's old arm itself reads 5–13% above its own recorded table
   (446.9 vs 405.3 @48 … 6093.5–6582.9 vs 5805.2 @256) — a day/context band
   of the same magnitude as the arc delta. Quoting the table to ±10% at
   quiet state is defensible; quoting any single reading as a regression is
   not.

## Instrument notes

- `i359_town_scale` prints via `println!` (stdout capture suffices; the
  `eprintln!` hazard is `i330`'s, not this probe's).
- The probe's leg A measures new+populate+2K ticks from fresh — the
  world-law difference (settlement/partition structure) is *inside* leg A's
  cost from tick 1 (clustered housing → different contact topology), which is
  why point 2 refuses single-iteration attribution.
- Raw logs: `/tmp/ab_{old,head}_{1,2,3}.txt` + `/tmp/ab_master.txt`
  (timestamps + exit codes; all 6 runs EXIT=0).
