---
name: ap4-departments
description: "AP4 department roster: seven departments, missions, territories, velocity classes. Charters in charters/."
type: Architecture-Plan-Reference
plan_id: AP4
---

# 02 — Department Roster

| Dept | Slug | Mission | Territory (crates/files) | Velocity |
|---|---|---|---|---|
| Simulation Core | **SIM** | the living world: biology, psychology, social mechanics, tick passes | `mindstrata-person`, `mindstrata-psych`, `mindstrata-social` (mechanics side), `mindstrata-world`, `sim/pass_*`, `sim/{core,mod}.rs` ⓦ | medium |
| Emergent Narrative & Culture | **STORY** | AFA fields, content generation, chronicles, lore systems | `mindstrata-development`, `social/culture/*` generators, annals/chronicle data | medium |
| Client & Presentation | **CLIENT** | TUI today → graphical client later; UX, rendering, dashboards | `mindstrata-tui/**`, future `client-*` crates | fast |
| Tools & Pipeline | **TOOLS** | probes, benches, scenario editor, observability, devex | `mindstrata-benches`, scripts/, scenario/spec_lint/provenance modules | fast |
| Verification & Release | **QA** | test infrastructure, golden replay custody, calibration audits, release gates | `mindstrata-tests`, gate scripts, evidence schemas | slow-deep |
| Game Design & Balance | **DESIGN** | player-facing tuning canon, pacing, difficulty, economy curves | `development/src/canon.rs` calibration sections, parameters crate tables, balance docs | medium |
| Platform & Scale | **PLATFORM** | determinism, performance budgets, save/migration, modding surface, CI | `mindstrata-core`, sim infra ({scheduler,snapshot,population_cap,mods}), build/CI | slow-deep |

Production Office (**PROD**) is a role, not a territory-holder — it owns ledgers and
windows only. Eight entries total; charters: [charters/](charters/).

## Territorial notes

- ⓦ files (`sim/core.rs`, `sim/mod.rs`) are SIM-owned but open as windows to STORY
  (pass hook) and PLATFORM (wiring) under PROD scheduling.
- DESIGN owns *values inside* frozen structures, never structure itself: canon.rs is
  SIM/STORY-built, DESIGN-calibrated via change-order against probe evidence.
- QA holds no product code; its rejections are process power, not write access.
- Boundary between SIM mechanics and STORY content = the three-realm type line from AP3:
  SIM provides signals/catalysts (gross+mechanism), STORY composes meaning (subtle).

## What departments deliberately do NOT exist yet

Art, audio, localization, marketing/community — they enter at DC-3+ once PLATFORM lands
the asset pipeline and CLIENT lands the graphical shell (01-operating-model §6). Scaffolding
them now would be fiction.
