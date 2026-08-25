---
name: ap4-deployment-manifest
description: "AP4 deployment manifest — per-agent startup packets for parallel dispatch: what each agent loads, its first phase, owned files, and what is blocked on P0. The dispatch-ready view."
type: Deployment-Manifest
plan_id: AP4
cycle: DC-1
dispatch_ready: true (after baseline commit + P0 freeze of IC-1/IC-3)
---

# 05 — Agent Deployment Manifest

## Startup packet (identical shape for every agent)

Each dispatched agent session receives this prompt skeleton:

```
You are the <DEPT> department lead for mindstrata DC-1.
Load, in order:
1. AGENTS.md (doctrine — binding)
2. docs/ROADMAP.md (trajectory)
3. docs/architecture/AP4-studio/PLAN.md → your charter charters/<DEPT>.md
4. docs/architecture/AP4-studio/04-cycle-plan-DC1.md §<DEPT> (your ladder)
5. docs/architecture/AP4-studio/templates/PHASE.md (instantiate per phase)
6. Interlock contracts you consume (see blocked-on table below)
7. git log --oneline -5 && git status --short  ← ALWAYS before editing
Rules: territory exclusivity (03-interlock-map §2); evidence-gated phases;
probe-first; never touch files outside your ledger rows; beats at ladder
multiples; commit only your own phase units with honest messages.
Begin at your ladder's first incomplete phase.
```

## Dispatch order & parallelism

| Wave | Agents | Rationale |
|---|---|---|
| 1 | **PLATFORM**, **QA**, **TOOLS** | their early outputs (IC-3 determinism law, IC-6 gates, IC-4 probes) unblock everyone; no dependencies on others |
| 2 | **SIM**, **CLIENT** | need IC-3 frozen (PLATFORM wave-1 phase 1–3); CLIENT also reads IC-8 draft |
| 3 | **STORY** | AP3 Era I is deliberately SERIAL inside STORY (freeze discipline: WP-A → B/C/D). Other departments run wide while STORY runs deep. Dispatches as one agent walking iters 266–272 per PLAN_AP3_EXECUTION.md |
| 4 | **DESIGN** | needs first beat artifacts from SIM/STORY to have anything to calibrate; starts with inventory phases which only need code reading — may join wave 2 if operator wants earlier |

Max concurrent: 6 (all but DESIGN) without conflict; territories are disjoint by ledger.

## Blocked-on table (do not start these phases before…)

| Dept phase | Blocked on | Provider |
|---|---|---|
| SIM 8–13 (pass wiring, consumers) | IC-1 catalyst vocabulary frozen | STORY+SIM co-author at SIM phase 5 |
| CLIENT 7–9, 23 (line-lane UI) | IC-2 schema v1 | STORY phase 12 |
| CLIENT 15 (save/load UI) | IC-3 save schema section | PLATFORM |
| DESIGN 4–11 (calibration values) | IC-5 canon schema | STORY phase 4 (canon.rs exists) |
| ANY behavioral fold | golden replay custody live | QA phase 3–4 |
| STORY 3+ (substrate crate) | vendor tables exist | STORY phase 1–2 self-serial |

## Swarm-native loading

The DC-1 plan exists as a machine payload: [07-plan-DC1.swarm.json](07-plan-DC1.swarm.json)
(save_plan-ready, 102 tasks in 5 dispatch phases, deps + files_touched + acceptance per
task) with its FR catalog in [08-spec-source.md](08-spec-source.md). Loading procedure,
id remapping, and guardrails: [06-swarm-integration.md](06-swarm-integration.md).
This document remains the human-readable source of truth for ladder content.

## P0 checklist (PROD runs before wave-1 dispatch)

- [ ] Baseline committed: all planning docs tracked (AP3, AP4, ROADMAP, knowledge-base)
- [ ] IC-3 drafted by PLATFORM agent as its FIRST act (skeleton already in contracts/)
- [ ] IC-1 co-authoring scheduled (SIM phase 5 ↔ STORY phase 5 window via PROD)
- [ ] Ladder trim decision recorded (operator time budget)
- [ ] Status tables initialized in 04-cycle-plan-DC1.md

## Session hygiene (every agent, every session)

- One phase in flight per session; finish or hand back cleanly.
- Evidence note filled BEFORE marking a phase done in the status table.
- Never rebase another agent's uncommitted work; poll instead.
- Escalate contract disputes to PROD via the IC changelog, not in code comments.
