---
name: roadmap
description: "Master project trajectory: layer map and milestone ladder. Updated at cycle boundaries; authoritative for the intended path, not for current engine behaviour (that is ENGINE_STATUS.md)."
type: Authority
status: AUTHORITY
scope: "project trajectory"
reconciled_commit: adc5985
created: 2026-07-28
---

# mindstrata Master Roadmap

*Single entry point for the whole project trajectory. Everything else hangs off this.*
*Last reconciled: 2026-09-22, post Iteration 352 / commit `adc5985`.*

## Deadlines at a glance

| Layer | Document | Status |
|---|---|---|
| Doctrine (how we work) | `AGENTS.md` | binding, always |
| **Current engine truth** | `docs/ENGINE_STATUS.md` | **authoritative for behaviour/scale** |
| **Live work ledger** | `docs/PLAN_DC3_DEVELOPMENT.md` | **authoritative for what remains** |
| Documentation governance | `docs/DOCUMENTATION.md` | authority map + index |
| Technical arc | `docs/architecture/AP3-afa/` | **Eras I–V complete** (reference/theory) |
| Organizational model | `docs/architecture/AP4-studio/` | active |
| Theory sources | KosmOS vault (`AP3-afa/refs/kosmos-index.md`) | live upstream; vendor-blocked items parked |

## Trajectory (milestone ladder)

```
DONE ──► AP1 core sim ──► AP2 deepening (~170 iters) ──► AP3 Eras I–V
     ──► DC-1 ──► UM-1 "the village develops"          ✅ CLOSED 106/106
     ──► DC-2 ──► UM-2 "the village tells stories"      ✅ CLOSED (jaccard 0.158)
     ──► DC-3 ──► UM-3 "the world scales"               ✅ CLOSED 4/4 legs
                  (Era IV collective holon; multi-village worlds;
                   N≥48 perf budget + unified gate; asset pipeline v0)

NOW  ──► DC-4 ──► UM-4 vertical slice ──► AA alpha
                  (graphical client shell, chronicle lens, difficulty
                   levers — mostly landed; scale program i330–i352)
     ──► DC-5 ──► infrastructure calibration (planned)
                  (dual-store unify, bounded event buffer, city-path
                   experiment, wealth dynamics)
```

**Where we are:** construction is complete through DC-3 and most of DC-4. The engine is a
deep *village*-scale simulator that stretches to a small town (~192–256 agents at ~130–240
tps); it is **not yet** a city/country/planet simulator (see `ENGINE_STATUS.md` §8). The
remaining work is a short, measured calibration queue (`PLAN_DC3_DEVELOPMENT.md` §3) plus
the DC-4 game-layer closeout.

Milestone gate definitions are QA-owned (AP4 IC-6). Each DC's concrete plan is written at
its P0 — see `AP4-studio/04-cycle-plan-DC1.md` for the historical pattern.

## Standing coordination rules

1. Multiple sessions share clones. `git log --oneline -5 && git status --short` before
   every session; re-read owned files if HEAD moved.
2. Territory ledger is authoritative (AP4 `03-interlock-map.md` §2); AP3 wave ledger remains
   authoritative for Era I–II file-level splits inside SIM/STORY lanes.
3. Every landed unit updates its own ledger row — status lives where the work is.
4. **Documentation alignment:** a doc that contradicts `ENGINE_STATUS.md` is stale. Classify
   every new doc in `DOCUMENTATION.md` §6 and reconcile AUTHORITY/ACTIVE docs when facts move;
   `scripts/gate` enforces this (`scripts/doc_index.py`).
