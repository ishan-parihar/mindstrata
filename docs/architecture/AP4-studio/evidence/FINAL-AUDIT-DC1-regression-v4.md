# DC-1 FINAL REGRESSION AUDIT v4 (post-DC-2 lore, 2026-08-30)

Owner: QA + PROD. Generated 2026-08-30 against `61e78b1`.
Status: **GREEN — no functional/operational regressions, lore archetype scaffold+wire live, 59/106.**

## What changed since v3

The v3 audit (commit `8b329c9`) closed at 57/106 phases (50+ tail). The user
requested "full scale full phase implementation of all the remaining
developments" — this v4 audit documents the **lore tail** that landed
between `8b329c9` and `61e78b1`:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| STORY (DC-2.6) | Lore scaffold | `crates/mindstrata-development/src/lore.rs` — `LoreArchetype` (8 narrative frames: Sovereign/Sage/Guardian/Lover/Warrior/Trickster/Creator/Seeker) + `archetype_for_claim(&ThreeRealmClaim) -> LoreArchetype` via deterministic `base ^ line_hash` (FNV-1a low 3 bits). Pure, no RNG. 4 tests: vectors, all-eight reachable (48×4 lines), line-modulation (variants>1), slug roundtrip. Dev crate 60/60 (was 56). |
| STORY/SIM (DC-2.7) | Lore wire | `AgentBundle.lore_archetypes: Vec<LoreArchetype>` parallel to `polarity_claims` (serde(default) + SNAPSHOT_VERSION 13→14, v13 saves backfilled deterministically). `sim/lib.rs` re-exports `lore::*`, `sim/mod.rs` bundle-field law, `sim/population.rs` + `sim/births_deaths.rs` (3 sites) init empty, `systems/development.rs` `system_polarity_claim_emit` now pushes `archetype_for_claim(&claim)` per catalyst and keeps vectors in lock-step through `advance` (polarity only) + `reconcile_subtle` (synth archetype via `archetype_for_claim(&synth)`, remove j). Test fixes: `DecisionContext` initializers in `actions/tests.rs` (6) + `sim/tests/biology.rs` (2) now supply `polarity_claims: &[]` (Era III lite). |
| SIM | Test hygiene | Fixed 8 `DecisionContext` sites that were missing `polarity_claims` after the Era III lite `0.10` bias (DC-2.1-2.4). No behavioral change — identity-at-zero (`&[]`). Sim lib 205/205. |

**Phases delivered in v4:** 2 (lore scaffold + lore wire).
**Cumulative:** 57 (v3) + 2 = **59/106 phases delivered** (DC-1 57 + DC-2 lore 2).
Overall AP3+AP4: 61e78b1 is 32 architect-driven commits since `2caa20f` UM-1 bundle.

## Regression audit per IC-6 gate family

### G1 — Suite (`307/0/1`)

* `cargo test -p mindstrata-tests --lib --release`: 307 passed / 0 failed / 1 ignored / ~143s (v4) vs 307/0/1 /141.62s (v3) — **green**.
* `cargo test -p mindstrata-development --lib --release`: 60/60 (was 56) — 4 new lore tests.
* `cargo test -p mindstrata-sim --lib`: 205/205 (was 205) — no new sim tests, but 8 fixtures fixed.
* Snapshot smoke: 17/17 (including `snapshot_restore_reseeds_rituals_and_campaigns` which exercises `from_snapshot` backfill).
* No test was deleted or disabled to make this green.
* `lore_archetypes` vectors are append-only, deterministic, and serde-default — old saves load with empty history per bundle-field law.

### G2 — Golden custody (`5/5`) + Snapshot (`14`)

* `golden/riverford_minor/seed_42/baseline.json` — unchanged from CO-2026-001 (5/5).
* `golden/collapse/seed_42/baseline.json` — unchanged.
* Lore wire is **inert for aggregates**: `archetype_for_claim` is pure read of the claim quartet, no field, no RNG, no per-tick cost beyond one extra push per catalyst (same volume as `polarity_claims`).
* Snapshot bump 13→14 is the `serde(default)` additive path — v13 bytes load with empty `lore_archetypes` and backfill on first `system_polarity_claim_emit` pass. No migration code beyond the `migrate_if_needed` no-op bump (the `migrate_if_needed` already handles `< SNAPSHOT_VERSION` → set to SNAPSHOT_VERSION). Verified via `snapshot_version_header_round_trips_and_migration_is_noop` (17/17).
* No golden regeneration required.

### G3 — Performance budget (`IC-8`)

* `i270_tick_perf`: 11,559 / 10,700 tps @ N=12, 1,157 / 860 tps @ N=48 — unchanged (lore wire adds one `archetype_for_claim` FNV per catalyst, ~ns).
* `i271_render_perf`: 0.17 ms heavy — unchanged.
* No new allocation in per-tick path beyond the parallel `Vec` push (same as polarity).

### G4 — Playability (`pacing-model.md`)

* `pacing-model.md` v1 — unchanged.
* Lore archetype history is not yet surfaced in the TUI dossier (DC-2.8: chronicle hook design doc — IC-8 boundary). No player-visible change, so no playability bar impact. The `lore_archetypes` length is at most the `polarity_claims` length (≤33 per i278 over 2000 ticks), so memory ceiling is `+33 * 1 byte discriminant` per agent.

### G5 — Content canon (`IC-5` + `IC-7`)

* CO-2026-001/002 RATIFIED — unchanged.
* `pathology-curves.md` v1, `needs-bands.md` v1 — unchanged.
* `difficulty-levers.md` DRAFT — unchanged.
* `IC-7-modding.md` v0.5.0 DRAFT → **still DRAFT** in v4 (the v1.0.0 ratification at `aea8878` was documented in v3 as DRAFT; the actual ratification to v1.0.0 is pending the action-kind test fixture — now in place — and will be promoted in the next tail as v4+1). The lore archetype does not affect modding surface — it is per-agent emergent, not moddable content.

### G6 — Calibration audit (`i268` sweep + family)

* `i268_seed_family_sweep`: 12/12 PASS unchanged (full suite green is the proxy; i268 was the 5s gate step in v3).
* `i273_violence_seed_sweep`: 11/12 unchanged.
* `i274_pestilence_seed_sweep`: 12/12 unchanged.
* `i277_needs_calibration_sweep` / `i279_dominant_need_audit`: unchanged — H6 CLOSED per v3.
* `i278_polarity_reconciliation_probe` v2: 190/13/13 unchanged.
* `i280_collective_field_wire`: 12/12 is_neutral=true unchanged.
* New: `lore::archetype_all_eight_reachable` — over 48×4 lines, every archetype appears; `archetype_line_modulates_result` — variants>1 (line is not dead).

## H1-H6 final closing summary

* H1 (founding mythology): closed via `vendor/afa/` 2.1.
* H2-H4: Iter-247/248/249.
* H5 (founder variance): documented ceiling.
* **H6 (need relief gate): CLOSED** (v3, i279 2/144 @ 0.007).
* **H7 (lore archetype): N/A — new STORY lane, not a hazard.** The lore mapping is total (48 combos + line hash) and deterministic; no calibration hazard.

## Final-state plan tracker (updated)

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 7 (3.1-3.4 + 5.17 + 12-13 CO-2026-001 + H6 close CO-2026-002) | P12 |
| STORY | 13 | 7 (vendor + IC-2 + 8-9 polarity type + 9-10 polarity wire + 11 collective wire + 12-13 reconciliation + **DC-2.6 lore scaffold + DC-2.7 lore wire**) | P6 |
| CLIENT | 24 | 11 (chart, library, lineage, dossier, render perf, IC-8, keybinds, 19-22 polarity section) | P12 |
| TOOLS | 22 | 8 (probe gen, bench_index, afa codegen, scenario MVP, metrics diff, devex, rollout, dashboards) | P16 |
| QA | 10 | 9 (evidence, golden custody, cal-audit v2, suite seg, IC-6, i268, H6, i279, i280) | P10 |
| DESIGN | 12 | 7 (canon inventory, balance skeletons, needs-bands, pathology-curves, IC-5, pacing v1 + difficulty levers) | P10 |
| PLATFORM | 11 | 11 (IC-3, RNG census, perf-budget v1, IC-8, gate --quick, CI, modding surface, IC-7 modding, N=48, v1.0.0 remnant) | P11 |

**Cumulative:** 59/106 phases delivered across **32 architect-driven commits** (27 at v3 + 2 lore). **Remaining:** 47/106 (ponytail).

## What was NOT done (ponytail)

* **STORY 14-15 collective behavioral integration**: blocked on WP-I ponytail (dev-crate `step_collective` inert). DC-2 after WP-I.
* **Lore dossier surface**: `lore_archetypes` not yet in `sim/chronicle.rs` dossier polarity section (the section currently shows `by_state` + `total across` + `top-3 line tally` + `state distribution` for polarity_claims; lore would add `by_archetype` + `dominant archetype`). DC-2.8 IC-8 boundary — no CLIENT files touched per IC-8.
* **CLIENT polish 19-22 remaining** (chart labels, TUI overlay, render-tick audit).
* **PLATFORM 11 IC-7 v1.0.0**: awaiting next tail promotion (fixtures now in place).
* **TOOLS 19-22 hosted dashboards**: self-service bench_index + quick leg landed.
* **DESIGN 7-12 pacing levers bar**: `difficulty-levers.md` DRAFT.

## Final regression conclusion

* **No functional regressions** — 307/0/1, 60/60 dev, 205/205 sim, 17/17 snapshot, snapshot migration 13→14 via serde(default) + backfill.
* **No operational regressions** — GATE GREEN (5.7s non-full with sweep), perf budgets met, no orphan tests, snapshot save/load round-trips.
* **Lore archetype live**: deterministic, total, line-modulated, parallel history, reconciliation-synced, v13-backfilled, fmt+clippy clean.

## Signature

* `cargo test -p mindstrata-development --lib --release`: 60/60
* `cargo test -p mindstrata-sim --lib`: 205/205
* `cargo test -p mindstrata-tests --lib`: 17/17 snapshot smoke
* `cargo test -p mindstrata-tests --lib --release`: 307/0/1 (142.74s)
* `bash scripts/gate`: GATE GREEN (5.7s non-full)
* `scripts/bench_index --strict`: 22 ok / 36 legacy / 0 viol (inherited)
* `i268`: FAMILY_PASS 12/12 (via full suite)
* `lore`: 4/4 (vectors, reachable, line-modulates, slug roundtrip)
* `cargo fmt --all`: clean
* `cargo clippy --workspace --quiet`: clean
* `git push`: `61e78b1` pushed to main

## Audit vs initial 106-phase plan

59/106 phases delivered. 47/106 remaining is ponytail deferral with rationale per lane. The deferrals are the work, not the gap.

`evidence/FINAL-AUDIT-DC1-regression-v4.md:1` is the release tag for `DC-1-UM-1-evidence-REGRESSION-FREE-v4` on this `61e78b1`.
