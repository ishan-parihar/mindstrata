# Iteration 352 — the last accidental relationship scan: the marriage pass was O(N³) every tick

**Status:** LANDED (performance, byte-identical) · **Root cause owned:** the
marriage-formation pass still read `trust` and `affection` with two
`self.relationships.iter().find(..)` linear scans per candidate pair, inside its
`for i { for j { .. } }` loop, on a pass that runs **every tick** — the one
accidental matrix walk the i330–i345 scan-removal program missed.

## What the probe measured first (`i352_marriage_scan`)

Probed before touching (§2.2). Two legs, both decisive.

### Leg A — cost (per-pass profile, seed 42, 2000 ticks)

The marriage pass had **no profiling mark** (every other heavy pass has one), so
two `mark!` calls were added around the call — instrumentation-only, inert when
profiling is off.

| N | marriage pass (pre-fix) | share of tick | marriage pass (post-fix) | change |
|---|---|---|---|---|
| 48 | 6.55 µs/tick | 0.8% | **4.59 µs/tick** | **−30%** |
| 96 | 57.08 µs/tick | 4.9% | **15.72 µs/tick** | **−72%** |

The pre-fix growth is the sizing evidence: **8.7× cost for 2× N ⇒ α ≈ 3.1 =
O(N³)**, exactly the `O(N² pairs × R rows)` shape the two per-pair scans imply.
Post-fix the growth is 3.4× for 2× N ⇒ **α ≈ 1.8**. Extrapolated to the N=192
charter row (i332: ~7700 µs/tick total) the pre-fix pass sat at ≈ 500 µs/tick
(~6.5% of the tick) and climbing superlinearly.

### Leg B — semantics (v1 matrix vs v2 store)

The pass reads the legacy v1 `relationships` matrix, not the honest per-agent v2
store (i351 ledger item 4). The probe sized how far apart they are in vivo:

| seed | N | pairs | Δtrust p50 | Δtrust p90 | Δaff p50 | Δaff p90 | pairs > 0.01 |
|---|---|---|---|---|---|---|---|
| 42 | 12 | 132 | 0.0123 | 0.2688 | 0.0067 | 0.1716 | 75 |
| 42 | 48 | 2256 | 0.0162 | 0.1276 | 0.0091 | 0.0727 | **1431 (63%)** |
| 7 | 48 | 2256 | 0.0142 | 0.1595 | 0.0126 | 0.0787 | 1376 |
| 43 | 48 | 2256 | 0.0163 | 0.0641 | 0.0110 | 0.0416 | 1394 |

So migrating the read to v2 **would be behavioural** (63% of pairs differ
materially): it is a separate, sweep-carrying iteration, **not** folded into this
one. This iteration is therefore scoped as pure performance — same store, same
values, faster access.

## The fix

`tick_marriage_formation` begins with one `self.rebuild_rel_lookup()` (O(R)), then
both per-pair reads use `self.rel_pos(i, j).map_or(..)` — the O(1) dense lookup
i330 built, whose doc states it returns **the same element `iter().find(..)` would
(first occurrence)**. That equivalence is already pinned by
`rel_pos_matches_linear_scan_including_after_population_change` (i330), which is
what makes the swap byte-identical rather than merely plausible. `rel_pos` keeps
its revalidation fallback, so a structurally-changed matrix still degrades to the
correct linear scan rather than reading a stale slot.

**New pin:** `marriage_gate_reads_the_v1_matrix_deliberately` crafts a pair whose
v1 and v2 trust disagree (0.9 vs 0.1) and asserts the gate's read expression
yields the v1 value — making the dual-store choice explicit so an accidental
migration fails loudly instead of silently re-timing every marriage.

## Verification

- **Golden replay 5/5 byte-identical** (riverford_minor + collapse, both
  `agent_count` preserved) — the binding proof for a performance-only change.
- `mindstrata-sim` **295/295** (+1 new pin), tests **310/0/1**, `gate --full`
  **GATE GREEN**, clippy clean.
- **Zero re-anchors, zero snapshots moved.** No behavioural surface changed.

## Ledger

1. **Dual-store migration is the next behavioural candidate** — sized here:
   63% of pairs diverge > 0.01 at N=48, and the marriage bond-boost writes v1, so
   a migration must move the *write* too and carry a full re-anchor sweep. Probe
   first; do not fold into a perf commit.
2. **Gate tooling debt** (carried from i349/i351): `clippy` without
   `--all-targets` still hides broken test cfg in other crates.
3. The scan-removal program (i330–i345) is now genuinely complete for the daily
   and per-tick relationship walks: the marriage pass was the last `.iter().find`
   over the dense matrix inside a nested loop.
4. A9 (Idle) unchanged — still wins 0 arbitrations, same design-act shape as A8.
