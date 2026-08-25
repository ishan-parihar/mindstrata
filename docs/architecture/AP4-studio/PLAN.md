---
name: ap4-studio
description: "AP4 Studio Scaffold — departmental operating model for scaling mindstrata to AA-class development: departments own territories, run 10–30 phase ladders per development cycle in parallel, converge at unifying milestones, then re-loop. Entry point. Triggers: 'start DC-N', department onboarding, milestone planning, interlock disputes."
type: Architecture-Plan
plan_id: AP4
status: Active
created: 2026-08-25
composes_with: AP3-afa (technical arc lives INSIDE department backlogs)
cycle_model: "DC-N = Phase 0 contract-freeze → N×10–30 departmental phases w/ beats → UM-N unify gate → retro → DC-(N+1)"
---

# AP4 — Studio Operating Model

## What this is

The organizational architecture that lets mindstrata scale from R&D instrument to
AA-class game: **seven departments** with exclusive territorial ownership, each running
its own **phase ladder** (10–30 phases per cycle) in parallel; a production office holding
the contract ledger; and a repeating loop — *freeze → parallel execute → converge →
unify → refine → re-plan*. Departments never share write scopes (extends AP3 doctrine §4);
every phase exits on observable evidence (inherits AGENTS.md §3 unconditionally).

## The loop (one Development Cycle, DC-N)

```
┌─ P0 ─────────────┐   ┌─ PARALLEL EXECUTION ──────────────┐   ┌─ CONVERGE ─┐
│ contract freeze  │──►│ SIM ▏STORY ▏CLIENT ▏TOOLS ▏QA ▏   │──►│ integration │
│ territory lock   │   │ DESIGN ▏PLATFORM                  │   │ windows     │
│ phase ladders    │   │  (beats every ~8 phases: mini-sync)│   └─────┬──────┘
└──────────────────┘   └───────────────────────────────────┘         ▼
┌─ RE-PLAN ◄─ RETRO ◄─ STABILIZE ◄─ UM-N UNIFY GATE ◄────────────────┘
│ (suite+golden+perf+playability+content review all green)
└──► DC-N+1
```

## Reading order

| Role | Read |
|---|---|
| Anyone joining | this file → [01-operating-model.md](01-operating-model.md) → your charter |
| Department lead starting a phase | your charter → [templates/PHASE.md](templates/PHASE.md) |
| Two departments touching one boundary | [03-interlock-map.md](03-interlock-map.md) → [templates/INTERLOCK.md](templates/INTERLOCK.md) |
| Production office | [01-operating-model.md](01-operating-model.md) §5 → [04-cycle-plan-DC1.md](04-cycle-plan-DC1.md) |
| Current cycle status | [04-cycle-plan-DC1.md](04-cycle-plan-DC1.md) status tables |
| **Deploying agents** | [05-deployment-manifest.md](05-deployment-manifest.md) — startup packets, dispatch order, blocked-on table |
| Swarm integration | [06-swarm-integration.md](06-swarm-integration.md) — concept mapping + loading procedure |
| Machine plan payload | [07-plan-DC1.swarm.json](07-plan-DC1.swarm.json) — save_plan-ready (102 tasks, 5 phases) |
| Spec source | [08-spec-source.md](08-spec-source.md) — FR catalog → spec_write at P0 |

## Non-negotiables (inherited + new)

1. Verification gate before EVERY commit (AGENTS.md §3) — no department is exempt.
2. Calibration honesty (AGENTS.md §4) + AP3 extensions — DESIGN cannot tune constants
   without probe evidence; QA audits for lucky-pins.
3. Territory exclusivity — one owner per file at any moment; ledger in
   [03-interlock-map.md](03-interlock-map.md).
4. Contracts frozen at P0; mid-cycle breaks are change-orders signed by PROD (§5).
5. `git log --oneline -5 && git status` at session start — multiple sessions share clones.
