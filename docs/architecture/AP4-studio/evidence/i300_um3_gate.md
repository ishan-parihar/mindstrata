# Iteration 300 — UM-3 GATE: multi-village culture differentiates by territory, interlinks by trade

**Status:** LANDED · **Root cause owned:** i299's measurement debt —
`host_count` conflates intra-polity gossip with cross-polity diffusion. The
UM-3 exit gate needs a per-agent signal.

## What landed

1. **`meme.hosts: Vec<usize>`** (`mindstrata-social/src/culture/meme.rs`):
   per-agent hosting sets, additive with `#[serde(default)]` (pre-i300 saves
   load empty — exactly the old semantics). `add_host` is idempotent
   (set semantics, bounded by the population cap); `is_hosted_by` is the
   query. The schema-freeze rule is respected: additive field, no reorder,
   old saves round-trip.
2. **Hosts recorded at both transmission sites**:
   - gossip meme transmission (`sim/social_cluster.rs`) records the listener;
   - trade diffusion (`systems/trade_diffusion.rs`) records the trade
     counterparty — the i299 debt closes: diffusion contributions are now
     directly countable per agent.
3. **UM-3 gate probe** (`i300_um3_gate`): one shared world, two auto-
   partitioned polities (i298 ring-pole geography), 50K ticks.

## Gate evidence (50K ticks, seed 42, N=12)

| Gate question | Measured |
|---|---|
| Do polities derive from geography? | 2 polities auto-partitioned, disjoint cover |
| Does culture exist per polity? | **26 namespaced genesis memes** (p0=14, p1=12), each citing its origin territory (i297 routing) |
| Do cultures interlink via trade? | **26/26 memes have ≥1 foreign host** — cross-polity hosting observed for every generated meme |
| Diffusion share | mean origin-share 0.288 (fraction of hosts in the origin polity) |

## Honest caveats (recorded, per §4)

- **Saturation ceiling**: at 50K/N=12, meme transmission (gossip channel)
  approaches full-population saturation, so the origin-share measures the
  *ceiling* of interlinking, not the trade-only contribution. The
  territorial differentiation claim rests on the i297 referent-routing
  evidence (distinct origin texts), while the interlinking claim rests on
  the `hosts` ledger (26/26 cross-hosted). A longitudinal origin-share
  decay curve (hosting at first-crossing time vs 50K) would separate the
  channels further — recorded as the next observability refinement if the
  DC-3 review wants it.
- **One-directional economic trade**: grain `TradeOccurred` has a single
  farm-owner seller at this scale, so the economic diffusion direction is
  diet-asymmetric (i299 finding); the gossip channel is symmetric.

## Pins (1 new, sim lib 258/258)

- `trade_records_the_receiver_in_hosting_set` (idempotent set semantics).

## Verification

- sim lib **258/258**; fmt clean; clippy 0 new warnings; bench law
  0 violations; full gate green; zero re-anchors (`hosts` is additive
  observability — no golden-projected state reads it).
- Probe: `crates/mindstrata-benches/examples/i300_um3_gate.rs`.
