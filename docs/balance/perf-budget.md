# Performance Budget Table v1 (PLATFORM 3.16)

Owner: PLATFORM co-signed QA. Feeds IC-8 and gate scripts (FR-006, FR-043).
Status: **DRAFT — measured basis from `i270_perf_snapshot` on 2026-08-29.**

## Measured basis

Harness: `cargo run --release -p mindstrata-benches --example i270_perf_snapshot`
(3 seeds × 2 horizons, release profile; criterion bench `tick_loop` remains the
authoritative regression detector — this probe is the budget-table's seed).

Host at measure: Linux x86_64, release `opt-level` workspace default.
Raw rows (from probe stdout, `target/release/examples/i270_perf_snapshot`):

```
n=12 horizon=2000  mean_tps=11559  min=9619  max=13112  (seeds 42/123/999)
n=12 horizon=10000 mean_tps=10700  min=9622  max=11663
n=48 horizon=2000  mean_tps=1157   min=1120  max=1187
n=48 horizon=10000 mean_tps=860    min=839   max=872
snapshot_bytes n=12 3461802  (~3.3 MB JSON)
snapshot_bytes n=24 5309368  (~5.1 MB)
snapshot_bytes n=48 10603546 (~10.1 MB)
```

Snapshot bytes = `serde_json::to_string(&sim.save_snapshot()).len()` at
500-tick warmup (includes world 16×16 + all agents; JSON proxy for RSS floor).
Criterion `benches/tick_loop.rs` groups `single_tick`/`burst_100`/`run_1000`/
`run_10k` at same N sweep remain the CI-facing harness (FR-006).

## Budget table (v1)

All budgets are **floor** checks — gate fails when measured < budget × 0.85
(noise margin) on the same host profile. Wall-clock projections use the
10K-horizon mean_tps (steady-state, past 4320 ritual horizon).

| Metric | N=12 baseline | Budget (floor) | N=48 projection | Budget (floor) | Notes |
|---|---|---|---|---|---|
| Tick rate (tps) | 10700 mean | ≥ 8000 | 860 mean | ≥ 700 | N=48 is ~12.4× slower than N=12 at 10K; projection assumes linear per-agent cost in gossip/economy passes (seating, market) dominates. Gate: `i270_perf_snapshot` 10K/seed-42 single-run ≥ floor; criterion `run_10k_ticks` median ≥ floor×1.1 |
| Wall-clock 100K ticks | ~9.3 s (100K/10700) | ≤ 15 s | ~116 s (100K/860) | ≤ 180 s | UM-1 100K playthrough target; N=48 is DC-3+ village-scale, not UM-1 gate but tracked for DC-3 planning |
| Snapshot JSON bytes | 3.4 MB | ≤ 6 MB | 10.1 MB | ≤ 18 MB | Linear model: `bytes ≈ 1.6MB + n·0.18MB + agent_state·n` (empirical 12→48); RSS will be lower (binary) but JSON is the stable proxy |
| Peak RSS (process) | ~ (snapshot × 2) | ≤ 50 MB @ N=12 | ~ (snapshot × 2.5) | ≤ 150 MB @ N=48 | Heuristic: in-memory bincode snapshot + sim state ≈2–2.5× JSON; gate to promote to measured `procfs` RSS when PLATFORM instruments it (4.16 task) |
| Criterion `single_tick` p50 | — (see tick_loop bench) | ≤ 120 µs @ N=12 | — | ≤ 1.2 ms @ N=48 | 1/tps at 10K is 93 µs / 1.16 ms; single-tick bench isolates per-tick overhead from warmup |

### Projection assumptions (N=48, v1)

1. **Linear per-agent cost dominates** beyond N=24: social gossip (O(n²) pair checks capped by `gossip::MAX_GOSSIP_EDGES`), economy market clearing, and agent-tick passes scale ~n; world 16×16 fixed so ecology/logistics stay flat. Hence 12→48 (4× agents) → ~12× tps drop is plausible and the budget's 700 floor allows headroom for the remaining Era II/III consumers (development pass, needs-gating) now landed.
2. **Memory scales ~linearly** with agent count (snapshot_bytes 12→48 is 3.06× for 4× agents, sub-linear due to fixed world/region overhead); budget ceiling 18 MB JSON gives 1.7× margin.
3. **No asset pipeline** yet (DC-3 deferred), so render/memory budgets exclude client textures/audio; PLATFORM 4.16 probes will re-measure at N=48 synthetic population with the full Era II field stack before raising IC-8.

## IC-8 hand-off

This table is PLATFORM's input to IC-8 ratification with CLIENT (task 4.19):
CLIENT `crates/mindstrata-tui` render measurements (task 3.9 `run_1000_ticks` frame-time) will be laid against the tick-rate floor to set the `render_budget_ms` and decide virtualization/downsampling policy (4.10). Until 4.19, this table is the provisional UM-1 gate for perf.

## Gate enforcement (preview for 5.11)

`scripts/gate --full` will add a perf leg:
`cargo run --release -p mindstrata-benches --example i270_perf_snapshot -- --quick` (single seed, 2K horizon) and assert `tps ≥ floor` and `snapshot_bytes ≤ ceiling`; criterion benches remain the long-form regression suite. Enforcement lands with PLATFORM 5.11 + QA 4.21.

## Probe pointers

- `crates/mindstrata-benches/examples/i270_perf_snapshot.rs` — this table's evidence.
- `crates/mindstrata-benches/benches/tick_loop.rs` + `benches/subsystems.rs` — authoritative criterion harnesses (FR-006).
- Future `crates/mindstrata-benches/examples/i<iter>_n48_scaling_probe.rs` (PLATFORM 4.16) will replace the synthetic 3-seed snapshot with the N=48 world-gen scaling study.
