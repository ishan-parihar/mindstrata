# Dossier Polish — CLIENT 19-22 Sweep 1

**Owner:** CLIENT. **Commit:** `fc4880b` (63/106). **Status:** DRAFT → v1.0 sweep 1.

## What dossier renders today (post-lore, @c032ae3)

`crates/mindstrata-sim/src/sim/chronicle.rs:268` `render_dossier` is **pure, read-only** over `Simulation` accessors:

* Header: agent id, age, status
* Needs: hunger/thirst/fatigue/valence window
* Personality: 6 traits
* Development: `DevelopmentFieldState` altitudes (neutral 1.0/0.00 zero-at-zero)
* Relationships: kin, trust
* Lore: `lore archetypes: N total across M frames` + top-3 tally (`lore.rs:9-46` deterministic mapping, 4/4 tests)
* Polarity: `polarity: N claims` + breakdown (STORY 12-13)
* Chronicle slots: last 10 events

All sections are **deterministic** (render twice → equal) and **inert** (no RNG, no state mutation).

## Polish sweep 1 — what was NOT built

Per `04-cycle-plan-DC1.md` CLIENT 19-22 "Polish backlog (dossier flows, annals browsing depth)":

* **Dossier flows** (CLIENT 19-20): agent inspector → chart lanes navigation — already tested (`dossier_integration_flow` 39/39 TUI green at 3.8), no new code.
* **Annals browsing depth** (CLIENT 21-22): navigate from chronicle entries to underlying `annals.jsonl` evidence records — **ponytail**: requires `village_annals` 100-tick decimation store to be populated beyond fixtures (currently `IC-2` trace behind feature flag, `session.rs` loader). DC-2 after WP-I.

## Sweep 1 close

* No new TUI code in this sweep — dossier already covers lore+polarity, help text accurate, keyboard nav reaches every pane (`KEYBIND_HELP` @3.7).
* This doc closes sweep 1 as **polish dispositioned** (no regressions, no new code). Sweep 2 (panel ergonomics) remains as next CLIENT tail when `annals.jsonl` is populated.

## Evidence

* `cargo test -p mindstrata-sim --lib dossier` 3/3 (polarity, lore, determinism)
* `cargo test -p mindstrata-tui --lib` 39/39
* `python3 scripts/dc1_metrics_snapshot.py --check` PASS

This runbook satisfies CLIENT 19-22 sweep 1 disposition per `IC-8` boundary (no CLIENT files touched beyond docs).
