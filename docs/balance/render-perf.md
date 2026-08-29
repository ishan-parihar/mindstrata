# Render Hot-Path Measurements (CLIENT 3.9)

Owner: CLIENT → PLATFORM IC-8 input (FR-043).
Probe: `crates/mindstrata-benches/examples/i271_render_perf.rs`
Harness: `cargo run --release -p mindstrata-benches --example i271_render_perf`
Status: **MEASURED 2026-08-29, release profile.**

## Methodology

Fixture-driven render (no live sim coupling): `fixture_history(n)` builds `n`
`MetricsSnapshot`s with monotonic stress/health ramps; `sim.populate()+run(1000)`
supplies 12- and 48-agent dashboards, agent lists, and world maps (16×16).
Each render fn is warmed once, then timed over 1000 iters (500 for the
10K-heavy trends case). Times are per-frame wall-clock (`Instant::now()`),
bytes are total `String` length (proxy for allocation pressure).

Host at measure: same Linux x86_64 release profile as `perf-budget.md`
(`i270_perf_snapshot`). Tick rate for comparison: N=12 mean 10.7K tps
(≈93 µs/tick), N=48 mean 865 tps (≈1.16 ms/tick) at 10K horizon.

## Raw rows (probe stdout)

```
render_hot_path perf (release, iters=1000 unless noted)
metric_charts_2k       iters=1000 per_frame_us=9.7   total_bytes=2045000
metric_charts_10k      iters=1000 per_frame_us=22.6  total_bytes=2047000
metric_charts_10k_heavy iters=500 per_frame_us=163.9 total_bytes=1025000  # 10K samples, WINDOW 60 cap
dashboard_12           iters=1000 per_frame_us=0.9   total_bytes=655000
agent_list_12          iters=1000 per_frame_us=10.4  total_bytes=1258000
dashboard_48           iters=1000 per_frame_us=1.1   total_bytes=655000
agent_list_48          iters=1000 per_frame_us=39.3  total_bytes=4066000
world_map_12           iters=1000 per_frame_us=3.9   total_bytes=451000
world_map_48           iters=1000 per_frame_us=8.3   total_bytes=451000
```

## Budget assessment (for IC-8)

| View | N=12 | N=48 | Notes |
|---|---|---|---|
| `metric_charts` (Trends, 2K history) | 9.7 µs | 22.6 µs (10K) / 163.9 µs (10K heavy, 10K samples) | WINDOW 60 caps sparkline; heavy case still <0.2 ms. No virtualization needed at 2K; 10K-heavy is worst-case and still <1 ms |
| `dashboard` | 0.9 µs | 1.1 µs | Fixed cost, institution count flat |
| `agent_list` | 10.4 µs | 39.3 µs | Linear in agents (≈0.8 µs/agent); 48-agent list <0.05 ms |
| `world_map` (16×16) | 3.9 µs | 8.3 µs | Marker scan O(n), negligible |
| **Total per-frame (tick+render)** | tick 93 µs + render ~25 µs ≈ 118 µs | tick 1.16 ms + render ~70 µs ≈ 1.23 ms | At 10 tps interactive (100 ms frame budget, `delay_ms=120`), render is <2% of budget; at 60 fps headless, still <5% |

**Verdict:** No regression. All hot paths are <0.2 ms even in the heavy
10K-trends case; the 60-sample window keeps Trends bounded. No downsampling
or virtualization required for DC-1 (deferred to 4.10 if N=48 village scales
to 100K-tick histories). Budgeted in `perf-budget.md` IC-8 hand-off: render
budget `1 ms` per frame is satisfied with 5× margin.

## Probe pointers

- `crates/mindstrata-benches/examples/i271_render_perf.rs` — this table's evidence.
- `crates/mindstrata-benches/Cargo.toml` now depends on `mindstrata-tui`
  (added for this probe; workspace `Cargo.toml` carries `mindstrata-tui`
  in `workspace.dependencies` — ponytail note: dependency was missing, added
  to unblock the probe; no runtime cost).
- Future `crates/mindstrata-tui/benches` or criterion harness can promote
  these ad-hoc timings to CI if IC-8 tightens the budget.
