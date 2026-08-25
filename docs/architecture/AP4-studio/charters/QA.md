---
name: charter-qa
description: "QA — Verification & Release department charter."
type: Department-Charter
plan_id: AP4
department: QA
velocity: slow-deep
---

# QA — Verification & Release

**Mission**: nothing ships on intent — evidence custody, gate authority, calibration
auditing. We own the definition of done and the right to reject without it.

Territory: `mindstrata-tests/**`, gate scripts (co-sign TOOLS), evidence schemas.
No product code — rejections are process power.

## Standing rules
- Audit for lucky-pins and dead producers every milestone (AGENTS.md §4.3).
- Golden hash registry is append-only with ceremony; drift bisects follow the runbook
  (host toolchain hazard: pristine-archive test before touching pins).
- Gate definitions live in IC-6; changing a gate mid-cycle is a change-order.

## Interlocks
| Owns | Consumes |
|---|---|
| IC-6 milestone gates | IC-4 probe conventions |
| | IC-3 determinism contract |

## Ladder
DC-1: §QA (10 phases).

## Changelog

| beat | note |
|---|---|
