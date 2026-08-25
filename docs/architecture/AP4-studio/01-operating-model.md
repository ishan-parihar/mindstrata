---
name: ap4-operating-model
description: "AP4 cycle anatomy: phases, beats, milestones, governance roles, drift management, evidence rules."
type: Architecture-Plan-Reference
plan_id: AP4
---

# 01 — Operating Model

## §1 Anatomy of a Development Cycle

**P0 — Freeze (short, serial).** All department leads + PROD agree:
1. Cycle goal = the Unifying Milestone definition (UM-N), written as testable criteria.
2. Interlock contracts frozen and versioned (`contracts/IC-<n>-<name>.md` via template).
3. Territory ledger locked: each department's file scope for the cycle.
4. Phase ladders drafted per department (10–30 entries, each with exit gate).

**Parallel execution.** Departments run their ladders concurrently. Rules:
- A **phase** is one PR-sized unit: goal → probe/baseline → implement → gates → evidence
  note. Phases are department-internal; no cross-department coordination needed inside one.
- A **beat** every ~8 phases: department publishes a sync artifact (branch state + metrics
  snapshot) consumed by dependent departments; catches drift before it compounds.
- Velocity mismatch is expected; PROD manages it (§5), not the departments.

**Converge.** When all departments report ladder-complete, PROD opens integration windows:
cross-cutting files merge under single-owner windows (AP3 doctrine §4 pattern at dept scale).
Golden replay referees structural merges; behavioral merges bring their re-anchor evidence.

**UM-N — Unify gate (all must hold):**
| Gate | Owner | Instrument |
|---|---|---|
| Suite green release-mode | QA | `cargo test -p mindstrata-tests --lib --release` + `scripts/gate --full` |
| Determinism intact | PLATFORM | golden replay hashes; save/load round-trip |
| Performance budget | PLATFORM | tick-rate benchmark vs budget table |
| Playability bar | DESIGN+CLIENT | scripted playthrough probe; UI smoke |
| Content coherence | STORY | generated-culture review probe; no type-check violations |
| Calibration audit | QA | no lucky-pins; re-anchors documented |

**Stabilize → Retro → Re-plan.** Stabilization fixes gate failures only (no new features).
Retro feeds charter/process amendments into DC-N+1 P0.

## §2 Phase anatomy (template: templates/PHASE.md)

```
id / department / cycle
goal (one sentence)
baseline evidence (probe numbers BEFORE)
change scope (files within territory)
gates (test names, thresholds)
evidence (numbers AFTER, mechanism comment)
interlocks touched? → contract refs
```

A phase that cannot state its baseline evidence is not ready to run.

## §3 Department velocity classes

Not all departments move at the same natural speed; ladders are sized accordingly:
- **Fast-loop** (TOOLS, CLIENT): compile-feedback tight, many small phases (20–30/cycle).
- **Medium** (SIM, STORY, DESIGN): behavioral units with probes (12–18/cycle).
- **Slow-deep** (PLATFORM, QA): few large verification/infrastructure phases (8–12/cycle).

PROD sizes ladders so projected completion dates roughly align; slack lives in fast-loop
departments (they absorb stabilization tasks during converge).

## §4 Evidence & artifacts

Every phase lands: code + evidence note in its ladder row + updated charter changelog.
Department-level rollups live in charters; PROD keeps only the cross-department ledger.
Artifacts of record: probe outputs (committed as bench examples or docs), golden hashes,
benchmark tables, playthrough logs. Intent and diffs are NOT evidence (AGENTS.md §2).

## §5 Governance roles

| Role | Held by | Powers |
|---|---|---|
| Production Office (PROD) | architect session(s) | ledger authority, change-orders, window scheduling, milestone declaration |
| Department Lead | the session executing a charter | phase sequencing inside territory, beat publication |
| Contract Signatory | both departments on an interlock | only they may propose changes to their IC; PROD countersigns mid-cycle |
| Auditor | QA department | may REJECT any landed phase lacking evidence; escalation straight to PROD |

Conflict resolution path: departments resolve inside contracts → unresolved = PROD ruling
(recorded in IC changelog) → operator override last.

## §6 Scaling notes (AA trajectory)

DC-1..2 run with today's TUI reality and agent-executed labor. DC-3+ introduces art/audio
sub-charters under CLIENT once an asset pipeline exists (PLATFORM deliverable); headcount
(agents or humans) scales by adding sessions INSIDE a department's territory, never by
sharing territories.
