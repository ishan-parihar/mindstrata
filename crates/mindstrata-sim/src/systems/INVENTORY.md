# Arc-D Extraction Inventory — bio/psych passes → `systems/`

**Scope**: verbatim-move plan per AGENTS.md §7 (pure refactor, golden replay is referee,
zero behavioral deltas). Prepared before any move lands. Line ranges verified against
working tree 2026-08-25.

## Current tick() pass sequence (`crates/mindstrata-sim/src/sim/core.rs`, tick at :53–687)

| # | Pass | Current location | Lines | Domain crate | Target module under `src/systems/` | State touched | Behavioral delta |
|---|---|---|---|---|---|---|---|
| 1 | `tick_biology_pass` | `sim/pass_biology.rs:6` | 6–192 | mindstrata-person (biology/) | `systems/biology.rs` | bodies, needs, emotions(read), weather, provenance, world food/water totals | NONE — verbatim |
| 2 | `tick_cognitive_pass` | `sim/pass_cognitive.rs:12` | 12–1060 | mindstrata-psych | `systems/cognitive.rs` | emotions, needs, personalities, phases, relationships, institutions, norms, kinship, households, factions, provenance; returns regulation strategies | NONE — verbatim |
| 3 | `system_need_decay_with_params` | `systems/mod.rs:57` | already extracted ✓ | — | (done) | needs | n/a |
| 4 | `system_body_update` | `systems/mod.rs:79` | already extracted ✓ | — | (done) | bodies, needs | n/a |
| 5 | `system_goal_generation` | `systems/mod.rs:107` | already extracted ✓ | — | (done) | goals, needs, emotions, personalities | n/a |
| 6 | `tick_action_pass` | `sim/pass_action.rs:12` | behavior domain | actions/ engine above | out of Arc-D scope (bio/psych only) | — | — |
| 7 | `tick_social_pass` | `sim/pass_social.rs:10` | social domain | — | out of Arc-D scope | — | — |
| 8 | `tick_appraisal_pass` | `sim/pass_appraisal.rs:9` | 9–800 | mindstrata-psych (appraisal) | `systems/appraisal.rs` | affects, emotions, needs, personalities, pre-tick events, reg strategies | NONE — verbatim |
| 9 | `tick_decay_pass` | `sim/pass_decay.rs:7` | 7–307 | psych/person hybrid | `systems/decay.rs` | affects, emotions, goals, needs, phases | NONE — verbatim |
| 10 | write-back loop | `sim/core.rs:288–298` | stays in core.rs | — | orchestration-owned (mem::take re-attach discipline, IC-3 §4 grounding) | agents | NEVER moves with passes |
| 11 | `health_disease_pass` | `sim/pass_health.rs:15` | 15–192 | mindstrata-person (health) | `systems/health.rs` | bodies, disease state | NONE — verbatim |

Also present as sibling pass files (non-Arc-D domains, listed for completeness):
`pass_weather.rs`, `pass_ecology.rs`, `pass_scenario.rs` — world/scenario domain,
separate future window.

## Ordering constraints

1 → 2 → {3,4,5} → 6 → 7 → 8 → 9 → [10]. The three extracted systems sit between
cognitive and action by design (§9.1 pressure ordering); a move must not perturb this
sequence. RNG: passes draw via `ctx.rng` streams in documented order — verbatim moves
preserve call order byte-for-byte, which is what makes golden identity the acceptance
referee.

## Risk notes

- **mem::take interplay**: every moved pass writes taken buffers; the write-back loop
  (:271–298) is orchestration-owned and must NOT travel with any pass (IC-3 trap
  narrative lives there).
- **RNG stream discipline**: cognitive/biology consume Behavior/Biology streams in fixed
  order; any accidental reshuffle of draws shifts all downstream seeds (AGENTS.md §5).
- **Visibility**: pass fns are `pub(super)` inside `mod sim`; moving to `systems/`
  requires either `pub(crate)` on the moved fn + import, or re-export shim — prefer the
  shim-first pattern used by prior extractions (lib.rs keeps legacy paths compiling).
- **Golden proof**: each move batch runs `scripts/gate --full`; hashes must match the
  pre-move registry entry (QA custody runbook).

## Move batches

- **Batch 1**: biology (+health) → `systems/{biology,health}.rs` — smallest blast radius.
- **Batch 2**: appraisal + decay → `systems/{appraisal,decay}.rs`.
- **Batch 3**: cognitive (largest, 1048 lines; consider internal split only AFTER
  verbatim move proves golden-neutral).
