# Iteration 297 — territory-anchored per-polity genesis (UM-3 leg 1)

**Status:** LANDED · **Root cause owned:** i296's per-polity holons gave
partitioned villages divergent stage trajectories, but genesis still cited the
WHOLE village's referent pools — divergent cultures would commemorate with
identical founding memories. The i296 scope statement named this as the next
UM-3 leg.

## What landed

1. **`system_polity_genesis`** (`systems/genesis.rs`): per-polity genesis with
   territory-anchored referent views.
   - **Institutions**: strict member-majority anchoring; no-owner
     institutions stay shared (citable by every polity).
   - **Sites**: nearest home-site centroid (Manhattan, integer means, exact
     tie → lowest polity index) — total and deterministic, and the only rule
     that can own communal sites (temple/well/square have no residents of
     their own, so residence-majority cannot). Houses stay excluded (i276).
2. **Culture namespaces** (`genesis_for_namespace`): when polities are
   assigned, per-polity genesis dedups on `[genesis:pN:Bucket:Class:Epoch]`
   tags — two polities reaching the same epoch each commemorate their OWN
   crossing instead of the second being suppressed by the first's tag.
   Legacy whole-village tags are unchanged (`None` namespace).
3. **Wire** (`sim/core.rs`): the polity genesis pass runs right after the
   polity field steps and BEFORE the whole-village genesis call; the
   whole-village call still runs every tick, so worlds with polities keep
   every legacy meme and gain only namespaced ones.

## Exit evidence (20K ticks, seed 42, N=12, in-vivo)

| Contract | Result |
|---|---|
| Zero blast without polities | 8 legacy genesis memes, **0** p-namespaced — unassigned runs are untouched |
| Territory routing | polity A (p0) memes cite **Village Market**; polity B (p1) cite **Village Temple** — each cites its own centroid-nearest communal site, neither cites the other's |
| Class coverage | p0: 6 memes, p1: 8 memes (p1's holon reached the band-III Song class first) |
| Identity-at-isolation (referent leg) | unit pin `polity_genesis_single_all_agent_polity_matches_whole_village`: one all-agent polity reproduces the whole-village genesis text (tags differ by namespace bookkeeping only) |
| Inert without assignment | unit pin `polity_genesis_is_inert_without_assignment` |

## Pins (3 new, sim lib 250/250)

- `polity_genesis_routes_sites_to_the_owning_territory` — partitioned world,
  east/west clusters, each polity cites only its territory's site.
- `polity_genesis_is_inert_without_assignment` — empty polity list is a no-op.
- `polity_genesis_single_all_agent_polity_matches_whole_village` — isolation
  contract for the referent leg.

## Honest scope

- Divergence mechanism proven at the REFERENT level (different places cited).
  Full UM-3 disjointness (jaccard gate over whole rosters) is i300's gate,
  after diffusion (i299) — trade-linked convergence must be measurable too.
- Membership remains operator-assigned (auto-partition is i298, probe-gated
  per i296 doctrine).
- `assign_polities` resets on snapshot-restore (unchanged i296 semantics).

## Verification

- sim lib **250/250** (3 new pins); fmt clean; clippy 0 new warnings;
  bench law **117 files / 81 ok / 0 violations** (`i297_polity_genesis` indexed ok).
- Probe: `crates/mindstrata-benches/examples/i297_polity_genesis.rs`.
