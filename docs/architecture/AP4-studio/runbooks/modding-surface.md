# Modding Surface Survey — What Leaves the Sim Cleanly Today (PLATFORM 9-10)

Owner: PLATFORM, surveyed with STORY/DESIGN. Status: **SURVEY v0.9 at `8a11747`.**
No new API lands in DC-1 — this is the inventory that DC-2 will expose behind a
stable `mods/` crate without breaking `IC-3`.

## What is already clean (no sim internals leak)

* **World + agents snapshot:** `Simulation::save_snapshot()` `JSON` (`snapshot_bytes 3.3–10.1 MB`) — `IC-3` versioned `v13`, deterministic, the only persistence channel (`IC-6 G2`).
* **Metrics + annals:** `MetricsSnapshot` per-tick aggregates + `annals.jsonl` `100`-tick `development_snapshot` (`IC-2`) — pure data, no `Simulation` handle required in `CLIENT`/`TOOLS`.
* **World-gen:** `world_gen/` seeded `16×16` today — `seed` + `num_agents` + `world_width/height` are the only world-gen knobs (territory `WORLD`).
* **Balance canon:** `docs/balance/*.md` (`needs-bands`, `pathology-curves`, `pacing`, `difficulty-levers`) — `IC-5` `CO-` data, no code structure, ideal mod surface.

## What still leaks (territory debt, not exposed)

* `sim/core.rs` `tick()` pipeline order + `systems/` pass signatures — orchestration only, `SIM` owns, not mod-stable until `Arc-D` slimming (`SIM 1-4`) lands `<15K`.
* `person/` `psych/` `social/` field types (`Stage`, `LineId`, `MotiveCategory`) — `STORY` vendored substrate, will move behind `mindstrata-development` `canon.rs` at `WP-0B` before modding.
* `institutions/` / `economy/` settlement types — `SIM` internal, no versioned projection yet.

## Exposure order (DC-2/3 ladder, ponytail: survey first, crate second)

1. `mods/` reads `docs/balance/*.md` + `annals.jsonl` (read-only) — no sim crate import.
2. `mods/` gets `save_snapshot` JSON schema (already versioned) — write-back via `SIM` migration harness (`PLATFORM 4-5`), never direct.
3. Only then `world_gen` seed/band parameterization opens to the scenario editor (`TOOLS 7-9` already MVP).

## Verdict

DC-1 ships **read-only modding** (balance docs + annals) — the surface is data,
not API. Full `dyn Trait` mod crates wait for `Arc-D` slimming + `IC-7` telemetry.
