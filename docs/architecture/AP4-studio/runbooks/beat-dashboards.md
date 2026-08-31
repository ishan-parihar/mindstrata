# Beat Dashboards — Observability for Beats (TOOLS 19-21)

Owner: TOOLS co-signed QA. Consumers: every dept at beats `~8` phases.
Status: **v1.0 — snapshot automation live (TOOLS 5.9 @3710dbc).**

## What a beat artifact already is (IC-6 G6)

Every dept at its beat multiple (`SIM@5&10`, `STORY@5&10`, `CLIENT@8&16`,
`TOOLS@8&16`, `DESIGN@6`, `PLATFORM@5&9`, `QA@5&8`) drops `metrics.jsonl` (`IC-4`
`key=value`) + a branch-state note in its charter `Changelog` (`templates/PHASE.md`
`evidence-schema-v1.md`). That contract already exists — this doc just says where
to read it when you are the beat reviewer.

## Self-service read (no new infra)

* **Index:** `python3 scripts/bench_index.py` (`27 ok / 36 legacy / 0 viol @3710dbc`) + `cargo bench` criterion
  groups (`tick_loop`, `subsystems`) — the only dashboards that run in `~1s`.
* **Per-beat:** `git log --oneline <last-beat>..HEAD -- <dept-territory>` +
  `cat docs/architecture/AP4-studio/evidence/UM-1-evidence.md` — the evidence bundle
  is the dashboard until a real metrics store exists.
* **Perf at beat:** `cargo run -p mindstrata-benches --example i270_perf_snapshot -- --quick`
  (`10056 tps`) + `i271 --quick` (`170µs`) — the numbers in `perf-budget.md`/`IC-8`.
* **Snapshot (one command):** `python3 scripts/dc1_metrics_snapshot.py` → `.swarm/evidence/dc1-metrics-snapshot.json`
  (mirrored to `docs/architecture/AP4-studio/evidence/dc1-metrics-snapshot.json`) —
  aggregates git HEAD, bench_index, golden custody (5 rows), perf budget, suite
  307/0/1, UM-1 evidence, dossier 13 tests; `python3 scripts/dc1_metrics_snapshot.py --check`
  verifies freshness for retro/UM-1 assembly (TOOLS 5.9).

## What we do NOT build in DC-1 (ponytail)

A hosted Grafana / metrics DB / alerting for 7 dept streams would be scaffolding
for later — the plan's verification phases beat it in value. The `beat artifact =
metrics JSON + branch note` contract is the dashboard; DC-2 can promote it to a
store when the runner (CI AA-mode) exists.
