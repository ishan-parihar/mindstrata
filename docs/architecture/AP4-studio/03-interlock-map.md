---
name: ap4-interlocks
description: "AP4 interlock map: cross-department dependency contracts, territory ledger pointers, change-order rules."
type: Architecture-Plan-Reference
plan_id: AP4
---

# 03 — Interlock Map & Territory Ledger

## §1 Interlock matrix (who owes whom an interface)

| # | Provider → Consumer | Contract surface | IC file |
|---|---|---|---|
| IC-1 | SIM → STORY | catalyst vocabulary (`CatalystEvent` variants, line tags) | contracts/IC-1-catalysts.md |
| IC-2 | STORY → CLIENT | annals/trace event schema + stage_lines export format | contracts/IC-2-observability.md |
| IC-3 | PLATFORM → all | crate DAG, determinism contract (f64 shadows, quantize-once, RNG stream rules), save schema versioning | contracts/IC-3-determinism.md |
| IC-4 | TOOLS → QA+SIM+STORY | probe harness conventions, bench naming, metrics emission | contracts/IC-4-probes.md |
| IC-5 | DESIGN → SIM/STORY | canon parameter schema (CALIBRATION-PENDING markers → measured values w/ evidence) | contracts/IC-5-canon.md |
| IC-6 | QA → PROD | gate definitions per milestone (the UM-N table is QA-authored) | contracts/IC-6-gates.md |
| IC-7 | CLIENT → DESIGN | UI telemetry for balance review (what the player can see/affect) | contracts/IC-7-ui-telemetry.md |
| IC-8 | PLATFORM → CLIENT | render data budget (what TUI/client may read per frame without sim coupling) | contracts/IC-8-render-budget.md |

Contract file = templates/INTERLOCK.md filled: purpose, exact types/schemas, version,
frozen-at, changelog. **Rule**: a consumer coding against an unfrozen contract is a phase
violation; a provider changing a frozen contract mid-cycle files a change-order with PROD.

## §2 Territory ledger (authoritative file-scope table)

Extends AP3 04-waves §3 to department granularity. Whole-file ownership; ⓦ windows
scheduled by PROD. Live in this file — departments propose edits at P0 only.

```
mindstrata-core/**            PLATFORM
mindstrata-person/**          SIM
mindstrata-psych/**           SIM        (belief/journal polarity hooks: STORY ⓦ via IC-1)
mindstrata-institutions/**    SIM        (read-side multipliers: STORY ⓦ Era IV)
mindstrata-social/**
  mechanics side              SIM
  culture generators          STORY
mindstrata-world/**           SIM
mindstrata-development/**     STORY      (canon.rs calibration VALUES: DESIGN via IC-5)
mindstrata-sim/src/sim/**
  pass_*.rs, core.rs, mod.rs  SIM ⓦ      (hook lines: STORY/PLATFORM by window)
  population.rs               SIM ⓦ      (founder draws: STORY by window)
  collective.rs (new)         STORY
  {scheduler,snapshot,population_cap,mods}.rs   PLATFORM
  actions/, routines/, api.rs SIM
mindstrata-tui/**             CLIENT
mindstrata-benches/**         TOOLS      (department probes write own examples; naming <iter>_*)
mindstrata-tests/**           QA         (departments REQUEST test moves, never edit)
scripts/**                    TOOLS      (gate scripts co-signed QA)
docs/architecture/AP3-afa/**  STORY      (technical arc docs)
docs/architecture/AP4-studio/**  PROD
docs/balance/**               DESIGN
```

Unlisted paths default UNASSIGNED — claim at P0, never silently.

## §3 Change-orders

Mid-cycle contract or territory changes require: written justification + migration cost +
affected-phase list; PROD approves/rejects within one beat; approved changes version the IC
(minor bump) or re-open P0 (major bump = cycle restart consideration).
