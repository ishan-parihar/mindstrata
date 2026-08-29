# IC-8 — Render Budget

```yaml
provider: CLIENT + PLATFORM
consumer: CLIENT (TUI) + PLATFORM (gate scripts)
owners: [CLIENT, PLATFORM]
frozen_at: DRAFT — pending ratification from 3.9 + 3.16 measurements (task 4.19)
version: 0.9.0-draft
status: DRAFT
change_orders: []
```

## Purpose

Bind the interactive TUI's per-frame render cost to the simulation's tick budget so
`UM-1 100K ticks` remains interactive at `N=12` and bounded at `N=48` without
the render pipeline becoming the bottleneck (FR-043).

## Measured basis (3.9 + 3.16)

*Tick (PLATFORM 3.16, `i270_perf_snapshot`, release):*
`N=12 10K 10.7K tps ≈93µs/tick`, `N=48 865 tps ≈1.16ms/tick`; `100K` wall-clock `9.3s/116s`.

*Render (CLIENT 3.9, `i271_render_perf`, release, 1000 iters, same host):*
`Trends 2K 9.7µs / 10K 22.6µs / 10K-heavy (10K samples, WINDOW 60) 163.9µs`,
`dashboard 0.9µs@12 1.1µs@48`, `agent_list 10.4µs@12 39.3µs@48`,
`world_map 3.9µs@12 8.3µs@48`; total `tick+render ≈118µs@12/1.23ms@48` (see
`docs/balance/render-perf.md` for methodology and raw rows; `docs/balance/perf-budget.md` for tick methodology).

## Draft budget (for 4.19 ratification)

| View | Budget | Rationale |
|---|---|---|
| `render_metric_charts` (Trends) | ≤1 ms per frame | Heavy 10K-heavy is 0.16 ms; 1 ms gives 6× margin before virtualization/downsampling (4.10) is required |
| `render_dashboard` + `render_agent_list` + `render_world_map` combined | ≤0.5 ms per frame | Measured ≤0.06 ms even at N=48 |
| Total per-frame `tick + render` | ≤15 s wall-clock for `100K@N=12` (PF budget table floor) and ≤180 s for `100K@N=48` | Already in `perf-budget.md`; render is <2% of interactive `100ms` frame (`delay_ms=120`) |

## Negotiation hand-off

CLIENT (3.9) publishes this draft with methodology; PLATFORM (4.19) ratifies
jointly with the tick budget table and adds the gate enforcement hook
(`scripts/gate` perf leg, PLATFORM 5.11) that fails on `render >1 ms` in the
`i271` quick mode and on `tick < floor` in the `i270` quick mode (QA 4.21).

## Enforcement (preview)

* `cargo run --release -p mindstrata-benches --example i271_render_perf -- --quick` (single seed, 2K) — assert `Trends ≤1 ms`.
* `cargo run --release -p mindstrata-benches --example i270_perf_snapshot -- --quick` — assert `tps ≥ floor`.

Until ratified, this draft is the provisional UM-1 gate for render (CLIENT 3.10).

## References

* `docs/balance/render-perf.md` — raw render rows and IC-8 hand-off section.
* `docs/balance/perf-budget.md` — tick budgets and N=48 projections.
* `crates/mindstrata-benches/examples/i271_render_perf.rs` — evidence.
* `crates/mindstrata-benches/examples/i270_perf_snapshot.rs` — tick evidence.
