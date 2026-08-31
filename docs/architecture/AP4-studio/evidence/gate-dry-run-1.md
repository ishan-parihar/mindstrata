# Gate Dry-Run #1 — Six Families (QA 5.13)

**Commit:** `19aab44` (v6 audit, 61/106)
**Run:** `bash scripts/gate` (non-full, 2026-08-31)
**Result:** **GATE GREEN**

## Raw output (tail)

```
[gate 1/3] fmt --check          — PASS (clean)
[gate 2/3] clippy --workspace   — PASS (quiet)
[gate 2.5/3] bench naming law   — PASS 27 ok / 36 legacy / 0 violations
DC-1 TOOLS 19-22 sweep: verdict=FAMILY_PASS 12/12
[gate 3/3] golden baselines     — PASS 5/5
  golden_replay_crisis_vs_baseline ... ok
  golden_replay_vs_baseline ... ok
  golden_replay_deterministic ... ok
  golden_replay_agent_count_stable ... ok
  golden_replay_different_seeds_differ ... ok
GATE GREEN
```

## Six families vs IC-6

| Family | Check | Evidence | Verdict |
|---|---|---|---|
| G1 suite | `golden_replay 5/5` (non-full) + documented `307/0/1 @c032ae3` | `cargo test -p mindstrata-tests --lib --release golden_replay` 5 passed | PASS |
| G2 golden custody | `scripts/qa/golden_registry.json` 5 rows, last `52a94f0` hash `ad253a79… / de761046…` | `gate` golden_replay vs baseline PASS | PASS |
| G3 perf budget | `IC-8` floors `Trends ≤1 ms / tick+render ≤15/180s` + `dc1-metrics-snapshot.json` `perf_budget_exists: true` | `i270 10.7K tps / i271 0.17 ms` documented at v5, `bench_index` clean | PASS (deferred full perf leg to `gate --full`) |
| G4 playability | `pacing-model.md v1` + `beat-dashboards.md v1.0` | dossier 3/3 + TUI lanes, no sim coupling | PASS |
| G5 content canon | `IC-5 v1.0.0 @c2016f6` + `IC-7 v1.0.0 @aea8878` + read-only patch @1155e49 | `content_pack_writable_field_count_is_four` PASS (via v5) | PASS |
| G6 calibration audit | `i268_seed_family_sweep` `verdict=FAMILY_PASS 12/12` caught by gate | `CA-1/CA-2/CA-4` 12/12 | PASS |

## Routing

No failures to route — all six families GREEN. Next dry-run (#2) will be `bash scripts/gate --full` (≈200s) for final UM-1 release audit. Failures, if any, route per `IC-6 §4` to owning dept (SIM/STORY/CLIENT/TOOLS/DESIGN/PLATFORM) via `phase_complete` evidence.

## Artifact

This file is the `5.13` evidence for `.swarm/evidence/` and `04-cycle-plan-DC1.md` QA ladder. Full suite dry-run (#2) remains as converge polish; non-full dry-run already gates every push.
