# Iteration 296 — multi-village holons (UM-3 core)

**Status:** LANDED · **Root cause owned:** `collective_field` was a single
whole-village holon — no way to express per-village/per-polity collective
development. **Additive by construction:** the legacy field is untouched; empty
`polity_fields` reproduces today's behavior exactly (golden untouched, zero
re-anchors by construction).

## What landed

1. **State** (`sim/mod.rs`): `polity_fields: Vec<CollectiveField>` +
   `polity_members: Vec<Vec<usize>>` (sorted member indices per polity,
   disjoint cover contract; unassigned agents simply have no holon).
2. **API** (`sim/population.rs`): `assign_polities(Vec<Vec<usize>>)` —
   idempotent partition assignment; resets on snapshot-restore (polities are
   a probe/scenario-level construct until a saved-world assignment rule exists).
3. **Press law** (`systems/development.rs`): the bucket accumulation was
   extracted verbatim into `accumulate_bucket_presses` (bit-identical for the
   whole-village path) and reused by the new
   `system_polity_collective_field_step`, which filters catalysts to polity
   members and normalizes per-capita WITHIN the polity (n = members.len()).
4. **Wire** (`sim/core.rs`): per-polity step runs on the same
   `polarity_window` right after the whole-village step — zero extra event
   walks, no allocation.
5. **Probe** (`i296_multi_village`): the exit-contract harness below.

## Exit evidence (10K ticks, seed 42, N=12, in-vivo)

| Contract | Result |
|---|---|
| Identity-at-isolation (unit) | one all-agent polity reproduces the whole-village field **exactly** (`single_all_agent_polity_equals_whole_village_field`) |
| Identity-at-isolation (in-vivo) | whole-village field == single-polity field **bit-identical** after 10K real ticks (stages S=3.000/I=1.000/R=1.000/M=1.000 on both) |
| Partition correctness (unit) | member order irrelevant; solo-population press equals partition press; foreign catalysts never press another polity's holon (`polity_partition_matches_solo_population`) |
| Two polities, shared world | even/odd split **diverges**: Safety 3.000 vs 4.000 — partitioned catalyst diets produce measurably different collective trajectories in the same simulation |
| Unassigned default | `polity_fields` empty; legacy behavior, zero blast |

## Honest scope statement (what this does NOT yet claim)

- **Cultural disjointness (UM-3's genesis leg)** is NOT claimed: i293 proved
  genesis text needs village-specific referents + differentiated stage
  trajectories. This iteration delivers the differentiated-trajectory
  substrate (partitioned press → divergent stages); the referent/naming leg
  and cross-village cultural diffusion via trade partners remain the next
  UM-3 iterations.
- Membership is operator-assigned (`assign_polities`); no emergence claim
  (settlement-based auto-partition) is made. Doctrine: probe-first — the
  mechanism must prove valuable before the assignment rule is wired.

## Verification

- sim lib 241/241 (30 dev-module tests incl. the two new pins); full release
  suite **307/0/1**; fmt + clippy clean; bench law 116 files / 0 violations;
  GATE GREEN with perf budgets OK (N=12 97.7/150, N=96 4574/6500).
