# Iteration 323 — the per-tick clan-enmity scan is immaterial (candidate closed)

**Status:** LANDED (**measurement only, no behaviour change**) · **Candidate
owned:** one of i316's ranked scale levers — the per-tick O(C²·N·feuds) scan in
`Simulation::decay_clan_enmities`.

## Why it was a candidate

`decay_clan_enmities` runs **unconditionally every tick** from
`tick_social_cluster`, and for each clan *pair* it scans **every agent** asking
"does any member of clan i hold an active feud with a member of clan j?" — the
same shape as the i320 accidents (a per-agent scan inside a pair loop). If the
clan registry grew with horizon (feuds forge enmities, enmities heal into
alliances, later feuds re-forge), that scan would become the next
superlinear term.

## What was measured (`i323_clan_enmity_scan`, release, N=12, 12 seeds)

Clan-registry size sampled every 100 ticks, plus the count of ticks with ≥2
clans (below that the pair loop is empty and the pass is free) and the total
enmity observations.

| horizon | max clans | mean clans | ticks with ≥2 clans | enmity observations |
|---|---|---|---|---|
| 2 000 | **3** | 3.00 | 100% | 100 |
| 20 000 | **3** | 3.00 | 100% | 646 |
| 50 000 | **3** | 3.00 | 100% | 882 |
| 100 000 | **3** | 3.00 | 100% | 882 |

## Verdict

`CLAN_SCAN_IMMATERIAL`

The registry is **pinned at exactly 3 clans at every horizon and every seed** —
it does not grow. The pair loop is therefore 3 pairs, and the inner agent scan
costs ≈ 3 × 12 = 36 comparisons per tick, i.e. **below the measurement floor**.
Enmities do form and clear (they are observed), so the pass is *live* — it is
simply cheap. Optimizing it would be churn with no measurable return, so it is
**closed as a candidate**, not carried as debt (§4.5: record immaterial findings
rather than flip-flopping fixes).

The remaining i316 scale levers stand unchanged: the sparse relationship store
(the Ω(N²) floor, i294), a Secondary-reduced biology path (does not exist), and
Background-tier reachability (broad re-anchors).

## Verification

- Measurement probe only — **no source change**, golden byte-identical by
  construction.
- `cargo fmt` clean, clippy 0 warnings, `scripts/gate --full` **GATE GREEN**.

**Probe:** `cargo run -p mindstrata-benches --release --example i323_clan_enmity_scan`
