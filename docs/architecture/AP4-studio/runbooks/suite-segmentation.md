# Suite Segmentation Audit (QA 3.14)

Owner: QA. Maps each standing test family to the contract it guards; flags orphans.
Status: **AUDITED 2026-08-29, suite 307/0/1 green (final_suite.log 198s).**

## Method

File-level segmentation (the `integration_tests/` namespace mirrors the
crate ladder). Full per-test list is `cargo test -p mindstrata-tests --lib --release -- --list`
(307 entries at this audit). Each family is tethered to a contract or
hazard class; a test that guards no contract is an orphan (keep/retire with
reason).

Counts below are from `git grep -c "fn .*test" crates/mindstrata-tests/src/integration_tests/*.rs`
(spot-checked 2026-08-29): top-level 6 files + `psychology/` 4 + `social/` 5
= 15 test modules. The suite's 307 tests split ~40% biology/infra, 30%
psychology/social, 30% culture/governance/economy/legal + unit tests in leaf
crates (person/psych/social/world/sim/tui).

## Coverage map

| Test family (file) | Approx. tests | Contract / Hazard it guards | Owner |
|---|---|---|---|
| `biology.rs` (conception/pregnancy/birth pipeline, health-sync) | ~40 | IC-3 determinism §5 (Fixed-4, mem::take write-back, RNG stream), `population.rs` founder draws, `systems/health` | SIM |
| `infra.rs` (golden replay, snapshot round-trip, determinism, seed-family) | ~20 | IC-3, `golden-replay-custody.md`, save-schema v13, RNG append-only | PLATFORM+QA |
| `culture.rs` (meme/knowledge/rumor/propaganda, noosphere) | ~25 | IC-1 catalysts → noospheric field, `culture/` registry | STORY/SOCIAL |
| `governance.rs` (institutions/factions, crisis-pressure) | ~30 | IC-3, faction crisis-pressure lifecycle, legitimacy | SIM |
| `economy.rs` (market, trade, Gini, patronage) | ~30 | Market price/trend, wealth inequality, `perf-budget.md` economy perf | SIM |
| `legal.rs` (norms, legal cases, verdicts) | ~20 | Norm registry, legal impl, `spec_lint` | SIM |
| `psychology/attachment.rs` | ~15 | Attachment system, `psych/attachment` | PSYCH |
| `psychology/attention_executive.rs` | ~15 | Attention/bounded rationality, executive | PSYCH |
| `psychology/beliefs_memory.rs` | ~15 | Belief update, memory store, `motivation` | PSYCH |
| `psychology/emotion_motivation.rs` | ~20 | Affect, motivation context, emotion regulation | PSYCH |
| `social/courtship_marriage.rs` | ~15 | Courtship/marriage registry, kinship | SOCIAL |
| `social/escalation_trust.rs` | ~15 | Conflict escalation, trust/relationship_v2 | SOCIAL |
| `social/kinship.rs` | ~15 | Kinship graph, households, clan | SOCIAL |
| `social/interaction_misc.rs` | ~15 | Gossip, social cluster, interaction | SOCIAL |
| `social/status_familiarity.rs` | ~10 | Status dimensions, familiarity | SOCIAL |
| Leaf crate unit tests (`person/`, `psych/`, `social/`, `world/`, `sim/`, `tui/`) | ~60 | IC-3 unit pins (Fixed, RNG, development field, TUI lanes) | Respective owners |
| `golden_replay` (5 tests) | 5 | IC-3 golden referee (structural vs behavioral) | QA |

## Orphan disposition

Scan `cargo test --list | rg -v "(biology|infra|culture|governance|economy|legal|psychology|social|golden|development|tui|person)"`
found **0 orphans** at this audit — every listed test family maps to a
contract above or to a named hazard in `AGENTS.md` §5 / `calibration-audit-v2.md`.

*Kept:* all families — even the 30-test `governance` family that historically
flaked on crisis-pressure (Iter-240) is now guarded by the `FACTION_CRISIS`
rate and its sweep (see `i272` differentiation).

*Retire candidates:* none at this audit. The next audit (after `4.x` Era III
content lands) will re-scan for new meme/culture tests that may outgrow this
map — add rows then.

## Evidence pointers

* `cargo test -p mindstrata-tests --lib --release -- --list > /tmp/list` (307 entries).
* `final_suite.log` (198s, 307/0/1) is the gate evidence for this map.
* Runbook `calibration-audit-v2.md` CA-1…CA-8 and `golden-replay-custody.md`
  are the downstream consumers of this map (which pin family a re-anchor
  touches).
