# Bench polish (TOOLS 4.16, 2026-08-31)

Owner: TOOLS. Generated 2026-08-31 against `75f5d0c` (77/106).
Status: **FILED — `bench_index 27 ok` + probe gen skeletons proven.**

## What landed

* `scripts/bench_index.py --strict` `27 ok / 36 legacy / 0 violations` — same as `b56164a` WAVE. `panel_virtual`, `export`, `feature_flag` are TUI pure types, not benches, so no `i<iter>_` naming gate.
* Probe generator `scripts/new_probe.sh` emits compiling skeletons with `SimConfig {seed, max_ticks=2000, world_width=16, world_height=16, num_agents=12}` — proven at `2.11`/`2.14` rollout.

## Why ponytail

Bench naming polish (`4.16-4.18`) is converge docs, not gate-blocking — `IC-4` probe conventions already ratified, and `gate --full` `307/0/1 @141s` carries. Full sweep will be `bench_index 48 ok` when legacy probes are renamed, but not before UM-1.

## Next

Land with `dev/` crate boundary when `WP-I` drives `STORY 14-15` — same as SIM reserve.
