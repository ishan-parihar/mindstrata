---
name: charter-client
description: "CLIENT — Client & Presentation department charter."
type: Department-Charter
plan_id: AP4
department: CLIENT
velocity: fast
---

# CLIENT — Client & Presentation

**Mission**: make the emergent legible and pleasant — TUI today, graphical client at AA
scale. We render what STORY/SIM emit; we never reach into sim state directly (IC-8 budget).

Territory: `mindstrata-tui/**`, future client crates.

## Standing rules
- Read only through published schemas (IC-2, IC-8); schema gaps = change-order, not hacks.
- Feature flags for anything consuming unfrozen contracts.
- Perf discipline: no per-frame allocation growth without a measured budget note.

## Interlocks
| Owns | Consumes |
|---|---|
| IC-7 UI telemetry | IC-2 observability |
| IC-8 render budget (co-author w/ PLATFORM) | IC-3 save schema |

## Ladder
DC-1: §CLIENT (24 phases).

## Changelog

| beat | note |
|---|---|
