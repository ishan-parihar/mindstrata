---
name: ap4-swarm-integration
description: "Maps AP4/AP3 plan documents onto opencode-swarm primitives: id remapping, loading procedure, guardrails. The bridge layer between human-readable plans and the swarm machinery."
type: Integration-Spec
plan_id: AP4
---

# 06 — Swarm Integration Layer

How the AP4/AP3 document suite loads into opencode-swarm's native machinery
(`.swarm/spec.md`, `.swarm/plan.json`, QA gate profile, declared scopes, evidence).

## Concept mapping

| Plan-document concept | Swarm primitive | Notes |
|---|---|---|
| Department | Task-tag + declared scope | Swarm has no dept concept; `[SIM:]` prefixes + `files_touched` per task encode it |
| DC cycle | One swarm plan (`save_plan`) | DC-1 = swarm_id `ap4-dc1`; later DCs get fresh plans |
| Dispatch wave (05-manifest) | Plan **phase** | 5 phases; parallelization_enabled within phase |
| Ladder phase (04-cycle-plan) | **Task** (`N.M`) | Dept-phase ids remapped, see below |
| Charter territory | `files_touched` + `declare_scope` at dispatch | Scope declarations refine dirs to exact files per task |
| IC contract | FR-### in `.swarm/spec.md` | Source of truth: `08-spec-source.md` → loaded via `spec_write` |
| UM-1 gate | `phase_complete` gates on phase 5 + QA task 5.20 evidence bundle | Gate profile selected via `set_qa_gates` before plan save |
| Beat (every ~8 ladder phases) | `commit_after_each_completed_task: true` + beat metrics task per dept | Checkpoints are the crash-recovery surface |
| Golden referee | FR-050 + QA task 2.16 (custody live) | Behavioral-fold tasks depend on 2.16 |
| ⓦ shared-file windows | `declare_scope(replace_existing)` serialized by conflict_policy | Only one in-flight task may hold a window |

## Task-id remapping (dept-phase → swarm N.M)

Ladder phases consolidated 106 → 102 tasks (merged: SIM detangle 3–4→2.6/2.7 pair retained;
added explicit contract tasks IC-6 [4.20], IC-8 ratify [4.19], playability bar [4.26],
UM-1 assembly [5.20]; folded CLIENT polish 19–22 → 5.7/5.8 sweeps). Prefixes in task
descriptions (`SIM:` …) carry department identity; the authoritative per-task ladder text
remains in `04-cycle-plan-DC1.md`.

| Dept | Ladder phases | Where they landed |
|---|---|---|
| PLATFORM | 1–11 | 1.1–1.3, 2.17, 3.15–3.16, 4.16–4.19, 5.11–5.12 |
| QA | 1–10 | 1.4–1.5, 2.15–2.16, 3.13–3.14, 4.20–4.21, 5.13–5.16, 5.20 |
| TOOLS | 1–22 | 1.6–1.8, 2.13–2.14, 3.11–3.12, 4.13–4.15, 5.9–5.9, 5.10 |
| STORY | 1–13 (= AP3 Era I–II WPs) | 1.9–1.10, 2.1–2.5, 2.9, 3.19, 4.1–4.6, 5.1–5.3 |
| SIM | 1–14 | 1.11–1.12, 2.6–2.8, 3.1–3.6, 4.25, 5.17–5.18 |
| CLIENT | 1–24 | 1.13–1.15, 2.10–2.12, 3.7–3.10, 3.20, 4.7–4.12, 5.4–5.8 |
| DESIGN | 1–12 | 1.16–1.17, 2.18–2.19, 3.17–3.18, 4.22–4.24, 4.26, 5.19 |

## Loading procedure (run at P0, after baseline commit)

```
1. spec_write(content = 08-spec-source.md body)          # .swarm/spec.md
2. set_qa_gates(reviewer=true, test_engineer=true,
                hallucination_guard=true, drift_check=true,
                sast_enabled=false, mutation_test=false)  # ratchet-tighter, choose now
3. save_plan(payload = 07-plan-DC1.swarm.json)            # .swarm/plan.{json,md}
4. Per phase, before dispatch:
   declare_scope(taskId, files) for every in-flight task  # exact-file refinement
   plan_conflict_check(task_ids)                          # advisory disjointness proof
5. Dispatch wave; collect; update_task_status per task;
   epic_record_divergence if turbo.epic active.
```

Steps 2–3 are order-sensitive: the QA profile locks at critic approval and binds to
plan identity (`swarm_id` + title) — select gates BEFORE the first save.

## Guardrails & operational notes

- **Protected paths**: `.opencode/opencode-swarm.json` protects `.git`,
  `.github/workflows`, `.opencode`, `.swarm`, `AGENTS.md`, `Cargo.lock`,
  `docs/architecture`. Consequences: PLATFORM CI-hardening task 4.17 is
  **proposal-doc only** — a human applies workflow edits. Story/SIM never edit
  plan docs mid-run (change-orders route through the operator).
- **Shared probe directory**: `crates/mindstrata-benches/examples/` receives
  probes from SIM, STORY, TOOLS. Filename law (`i<iter>_<slug>.rs`) keeps
  paths disjoint; `conflict_policy: "serialize"` catches violations.
- **Rust worktree lanes**: `worktree_isolation: true` gives each lane its own
  tree; cold `target/` rebuild per lane is the cost. Optional accelerator:
  `turbo.lean.runtime_isolation.env_overrides.CARGO_TARGET_DIR` pointed at a
  shared absolute path (cargo file-locking serializes builds but shares
  artifacts). Enable only if lane startup dominates.
- **Freeze discipline**: STORY tasks 2.2 and 2.5 record contract-freeze hashes
  in their acceptance; downstream consumers quote those hashes in evidence.
- **Evidence flow**: every task's completion evidence lands under
  `.swarm/evidence/`; QA task 5.20 assembles the UM-1 bundle from it. Beats =
  checkpoint commits tagged `beat/<dept>-<n>` in the message.
