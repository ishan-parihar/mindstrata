---
name: charter-platform
description: "PLATFORM — Platform & Scale department charter."
type: Department-Charter
plan_id: AP4
department: PLATFORM
velocity: slow-deep
---

# PLATFORM — Platform & Scale

**Mission**: the ground doesn't move — determinism, performance budgets, save/migration,
modding surface, CI. AA scale means N≥48 villages at stable tick rates with portable saves.

Territory: `mindstrata-core/**`, sim infra modules (scheduler, snapshot, population_cap,
mods), build/CI.

## Standing rules
- The determinism contract (IC-3) is law; exceptions need `// ponytail:` ceiling+path notes.
- Performance budgets are measured on release builds only; debug numbers are fiction.
- Save schema changes version + migrate; never break old saves silently.

## Interlocks
| Owns | Consumes |
|---|---|
| IC-3 determinism law | IC-6 gates (enforcement hooks) |
| IC-8 render budget | |

## Ladder
DC-1: §PLATFORM (11 phases).

## Changelog

| beat | note |
|---|---|
