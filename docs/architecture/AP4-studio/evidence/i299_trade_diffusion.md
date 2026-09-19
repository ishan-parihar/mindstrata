# Iteration 299 — cross-polity cultural diffusion via trade (UM-3 leg 3)

**Status:** LANDED · **Root cause owned:** i296–298 gave polities
differentiated, territory-rooted culture — but no channel for culture to
cross the boundary. Without one, polities are hermetically sealed and the
UM-3 "trade-linked convergence" thesis is untestable.

## What landed

1. **`system_trade_diffusion`** (`systems/trade_diffusion.rs`): a
   `TradeOccurred` or Trade `InteractionOccurred` between agents of two
   DIFFERENT polities grows `host_count` of the SENDER's namespaced genesis
   memes in the shared registry (damped by the §13.1 trade weight 0.3;
   growth capped at the receiving polity's member count; determinism via
   event-order scan, zero RNG; f64 damped units quantized once per the §5
   truncation rule).
2. **Wire** (`sim/core.rs`): runs inside `tick()` on the same
   `polarity_window` slice as the development/genesis passes (no extra
   allocation), before the genesis block so same-tick trades inform the
   same-tick culture step. Zero-at-zero: no polities → early return.

## Exit evidence (10K ticks, seed 42, N=12 — `i299_trade_diffusion`)

| Contract | Result |
|---|---|
| Zero-at-zero in vivo | no polities → **0** namespaced memes; legacy runs untouched |
| Cross-boundary diet | 1172 trade events in the last-50K window, **1160 cross-polity** — the shared market/farm sites are natural contact boundaries; the channel is LOADED |
| Channel liveness | 4 unit pins prove mechanics; debug-instrumented run measured **1026 damped meme growths** over the run (instrumentation removed before landing) |

## The measurement boundary (recorded debt, shapes i300)

The in-vivo A/B (contact vs no-contact geometry) could NOT isolate the
diffusion contribution, for three measured reasons:

1. **Wandering erases geometry** — the movement pass dissolves static
   position separation within days; settlement distance cannot block
   contact at N=12 (arguably correct: shared market sites ARE contact).
2. **One-directional trade economy** — grain `TradeOccurred` has a single
   farm-owner seller, so only the farm-owning polity exports through the
   economic channel (cultural diffusion is diet-asymmetric by production
   structure, not by the channel).
3. **`host_count` conflates signals** — intra-polity gossip and cross-polity
   diffusion both grow the same counter, both saturating at the population
   cap. The clean signal is per-agent meme hosting (`meme.hosts: Vec<AgentId>`)
   — a frozen serde-schema change, therefore recorded for the i300 UM-3 gate
   design, not pulled here.

## Pins (4 new, sim lib 257/257)

- `cross_polity_trade_diffuses_sender_memes`
- `intra_polity_trade_diffuses_nothing`
- `diffusion_is_inert_without_polities`
- `foreign_namespace_must_not_grow_from_own_trade`

## Verification

- sim lib **257/257** (4 new pins); fmt clean; clippy 0 new warnings;
  bench law 0 violations; full gate green; zero re-anchors (the pass reads
  no golden-projected state).
- Probe: `crates/mindstrata-benches/examples/i299_trade_diffusion.rs`.
