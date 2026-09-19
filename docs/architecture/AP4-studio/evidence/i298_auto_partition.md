# Iteration 298 — settlement-based auto-partition (UM-3 leg 2)

**Status:** LANDED · **Root cause owned:** polities were operator-assigned
(`assign_polities`); i296's doctrine explicitly deferred the assignment rule
until the mechanism proved valuable (i297 did: territory-routed genesis
diverges by place). This iteration delivers the rule: membership derives from
settlement geography.

## What landed

1. **`auto_partition_polities(max_gap)`** (`sim/population.rs`): deterministic
   single-linkage agglomeration over INHABITED home sites (Manhattan distance,
   `min_d <= max_gap` merges; site registry order = seed order; zero RNG).
   Agents map to the cluster of their home site; membership sorted per the
   i296 contract. Zero/one inhabited sites or a single merged cluster →
   `polity_fields` stays empty (the legacy default).
2. Threshold semantics verified: `max_gap` gates whether NEARBY inhabited
   sites merge into one settlement — distant sites are separate settlements
   for any threshold below their distance; a threshold above every gap yields
   ONE settlement → no partition.

## Exit evidence (10K ticks, seed 42, N=12, in-vivo — `i298_auto_partition`)

| Contract | Result |
|---|---|
| Single-settlement inertness | the calibrated one-ring village derives **0** polities (legacy default, zero blast) |
| Two-settlement derivation | houses 3 & 7 (opposite ring poles) → 2 polities, disjoint cover, sorted membership; trajectories diverge (p0 S=4.000 vs p1 S=3.000) |
| Derived ≡ assigned | the auto-derived partition stepped **bit-identically** to `assign_polities` with the same members over 10K real ticks |

## Pins (3 new, sim lib 253/253)

- `auto_partition_single_cluster_is_inert`
- `auto_partition_splits_two_settlements` (disjoint cover + sorted contract)
- `auto_partition_gap_threshold_is_respected` (threshold-above-all → inert;
  threshold 0 → every inhabited site its own settlement)

## Honest scope

- Clustering uses inhabited home sites only; an uninhabited hamlet casts no
  polity. Reasonable at N≤48 (the cap); revisit if settlement-size asymmetry
  matters for UM-3 scenarios.
- No migration between polities: membership is fixed at partition time (the
  re-partition call is idempotent replacement). Mobility-driven re-derivation
  is a scenario-level concern, not a tick pass.
- i297's genesis/territory machinery composes automatically: derived polities
  get namespaced genesis over their own centroids with no further wiring.

## Verification

- sim lib **253/253** (3 new pins); fmt clean; clippy 0 new warnings;
  bench law 0 violations (`i298_auto_partition` indexed ok); full gate green.
- Probe: `crates/mindstrata-benches/examples/i298_auto_partition.rs`.
