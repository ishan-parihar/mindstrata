# Iteration 295 — perf envelope as warn-only gate visibility

**Status:** LANDED · **Root cause owned:** the charter's golden budgets (N=12 ≤150,
N=96 ≤6500 µs/tick) existed only in prose — no gate step measured them, so drift was
invisible until a hard floor (i270 tps / i271 render) broke. Charter DC3-P0 §5 item 1.

## Deliverable

`crates/mindstrata-benches/examples/i295_perf_budget_gate.rs` — warn-only budget
probe, wired into `scripts/gate` as step **2.55** (runs on every gate, before the
hard perf leg at 2.6):

```
perf_budget n= 12 us_per_tick=    102.6 budget=  150.0 status=OK  (golden budget, charter DC3-P0 §2.1)
perf_budget n= 96 us_per_tick=   4577.7 budget= 6500.0 status=OK  (DC-3 Phase-1 target, charter DC3-P0 §2.2)
```

- **Warn-only by design** (charter: release variance ±8% on shared hosts must not
  block commits): breach prints `perf_budget_warning` to stderr and continues; exit 0.
  Hard enforcement stays with i270/i271 (`--quick` floors at gate 2.6).
- Method matches the i274/i294/charter baseline (world 32×32, 2000 ticks, seed 42,
  timer spans new+populate+run, min-of-3 at N=12, min-of-1 at N=96 to bound gate wall).
- Budget rows re-baselined by i294: N=12 headroom 31–34%, N=96 headroom 27%.

## Trigger review (charter §3/§4 standing items, re-checked against i293/i294 horizons)

| Lever | Trigger | Status |
|---|---|---|
| VecDeque event-buffer conversion | operator horizons >250K ticks / memory-capped hosts / annals export | **NOT hit** — suite horizon 10K, largest probe 20K (i293). Unchanged. |
| Per-agent recent-claims index | runs >250K ticks (O(claims)² salience filter) | **NOT hit.** Claims/agent measured 12→30 across N=12→192 at 2K ticks (i294) — the working-set plateau holds. |
| Sparse relationship store (NEW, i294) | N>96 operations or budget breach at N=96 | **Deferred with evidence** — α_rels=2.03 structural floor, but N≤96 is the DC-3 Phase-1 envelope with 27% headroom. |

## Charter §5 checklist

1. [x] perf regression probe warn-only in CI-adjacent benches — **this iteration**
2. [x] superlinearity N=192 — **i294**
3. [ ] asset pipeline v0 charter — separate doc, unchanged.

## Verification

- `cargo build --release` clean, `cargo clippy --workspace --quiet` clean, fmt clean.
- Probe run: both budgets OK (numbers above; also visible in gate output henceforth).
- Zero sim code touched → zero re-anchors by construction.
