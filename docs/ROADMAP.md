# mindstrata Master Roadmap

*Single entry point for the whole project trajectory. Everything else hangs off this.*
*Last reconciled: 2026-08-25, post Iteration 265 / commit `cc84a8c`.*

## Layer map

| Layer | Document | Status |
|---|---|---|
| Doctrine (how we work) | `AGENTS.md` | binding, always |
| Technical arc | `docs/architecture/AP3-afa/` (entry: PLAN.md) | Active, starts iter 266 |
| Iteration schedule | `docs/PLAN_AP3_EXECUTION.md` | subordinate to AP3 waves |
| Organizational model | `docs/architecture/AP4-studio/` (entry: PLAN.md) | Active |
| Agent deployment | `docs/architecture/AP4-studio/05-deployment-manifest.md` | ready for dispatch |
| Swarm-native plan + spec | `docs/architecture/AP4-studio/07-plan-DC1.swarm.json`, `08-spec-source.md` (procedure: 06) | staged, load at P0 |
| Theory sources | KosmOS vault (see AP3 refs/kosmos-index.md); vendored snapshots land via WP-0A | live upstream |

## Trajectory (milestone ladder)

```
NOW ──► DC-1 ────────────────► UM-1 "the village develops"
        (AP3 Eras I–II inside; ~106 dept phases; iters 266–~280)
     ──► DC-2 ────────────────► UM-2 "the village tells stories"
        (AP3 Era III content emergence; generated cultures seed-disjoint)
     ──► DC-3 ────────────────► UM-3 "the world scales"
        (AP3 Era IV collective holon; N≥48 performance budget; asset
         pipeline v0 lands → art/audio sub-charters open under CLIENT)
     ──► DC-4+ ───────────────► UM-4 vertical slice ──► AA alpha
        (graphical client shell, chronicle lens, difficulty levers;
         open-ended deepening loops per AP3 doctrine §5)
```

Milestone gate definitions are QA-owned (AP4 IC-6). Each DC's concrete plan is written
at its P0 — see AP4 `04-cycle-plan-DC1.md` for the pattern.

## Standing coordination rules

1. Multiple sessions share clones. `git log --oneline -5 && git status --short`
   before every session; re-read owned files if HEAD moved.
2. Territory ledger is authoritative (AP4 `03-interlock-map.md` §2); AP3 wave ledger
   remains authoritative for Era I–II file-level splits inside SIM/STORY lanes.
3. Every landed unit updates its own brief/ladder row — status lives where the work is.
