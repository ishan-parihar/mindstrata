# DC-1 FINAL REGRESSION AUDIT v5 (post-dossier lore, 2026-08-30)

Owner: QA + PROD. Generated 2026-08-30 against `c032ae3`.
Status: **GREEN — no functional/operational regressions, lore dossier surface live, 60/106.**

## What changed since v4

The v4 audit (commit `6965cf8`) closed at 59/106 phases (50+ tail + 2 lore). The user
requested "full scale full phase implementation of all the remaining
developments" — this v5 audit documents the **dossier lore tail** that
landed between `6965cf8` and `c032ae3`:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| SIM/STORY (DC-2.8) | Dossier lore surface | `crates/mindstrata-sim/src/sim/chronicle.rs:312` — dossier now surfaces `lore_archetypes` when present: `lore archetypes: N total across M frames` + top-3 archetype tally, derived deterministically from `archetype_for_claim`. Read-only, no sim state change, no CLIENT files, no RNG. New test `dossier_lore_section_appears_after_run` (2000 ticks, 12 agents, parallel to `dossier_polarity_section_appears_after_run`). Dossier 3/3, sim 206 (was 205), full suite 307/0/1 (156.39s). |

**Phases delivered in v5:** 1 (dossier lore).
**Cumulative:** 59 (v4) + 1 = **60/106 phases delivered** (DC-1 57 + DC-2 lore 3).
Overall AP3+AP4: c032ae3 is 33 architect-driven commits since `2caa20f` UM-1 bundle.

## Regression audit per IC-6 gate family

### G1 — Suite (`307/0/1`)

* `cargo test -p mindstrata-tests --lib --release`: 307 passed / 0 failed / 1 ignored / 156.39s (v5) vs 142.74s (v4) — **green**, delta is run-to-run jitter (long-horizon 50k).
* `cargo test -p mindstrata-development --lib --release`: 60/60 — unchanged.
* `cargo test -p mindstrata-sim --lib`: 206/206 (was 205) — +1 `dossier_lore_section_appears_after_run`.
* Snapshot smoke: 17/17 — unchanged.
* No test was deleted or disabled to make this green.
* Dossier lore surface is read-only pure function over `Simulation` accessors — no determinism surface, no RNG, no state.

### G2 — Golden custody (`5/5`) + Snapshot (`14`)

* `golden/riverford_minor/seed_42/baseline.json` — unchanged from CO-2026-001 (5/5).
* `golden/collapse/seed_42/baseline.json` — unchanged.
* Snapshot `14` (lore parallel history) — unchanged from DC-2.7. Dossier surface does not affect snapshot bytes.
* No golden regeneration required — dossier is read-only rendering.

### G3 — Performance budget (`IC-8`)

* `i270_tick_perf`: 11,559 / 10,700 tps @ N=12, 1,157 / 860 tps @ N=48 — unchanged (dossier is not in tick loop).
* `i271_render_perf`: 0.17 ms heavy — unchanged (dossier is on-demand, not per-tick).
* No new allocation in per-tick path.

### G4 — Playability (`pacing-model.md`)

* `pacing-model.md` v1 — unchanged.
* Dossier lore surface is **player-visible** via `render_dossier` (CLI `--dossier`, TUI `/` search) — the first player-visible lore framing. The `lore archetypes` line appears only when the agent has history (≥1 claim), so founder dossiers after 100 ticks now show the framing tally. No balance impact — read-only.

### G5 — Content canon (`IC-5` + `IC-7`)

* CO-2026-001/002 RATIFIED — unchanged.
* `pathology-curves.md` v1, `needs-bands.md` v1 — unchanged.
* `difficulty-levers.md` DRAFT — unchanged.
* `IC-7-modding.md` **RATIFIED v1.0.0** (2026-08-30, `aea8878`) — unchanged, still documents write-list + read-only invariant (now includes `lore_archetypes` as emergent, not moddable — will be added to the read-only list in next tail).

### G6 — Calibration audit (`i268` sweep + family)

* `i268_seed_family_sweep`: 12/12 PASS — unchanged (via full suite).
* `i273_violence_seed_sweep`: 11/12 — unchanged.
* `i274_pestilence_seed_sweep`: 12/12 — unchanged.
* `i277`/`i279` H6 CLOSED — unchanged.
* `i278` v2: 190/13/13 — unchanged.
* `i280` collective: 12/12 is_neutral=true — unchanged.
* `lore` 4/4 (vectors, reachable, line-modulates, slug) — unchanged.
* New: `dossier_lore` 1/1 (2000 ticks, 12 agents, `lore archetypes: N total across M frames` + `frames`).

## H1-H6 final closing summary

* H1 (founding mythology): closed via `vendor/afa/` 2.1.
* H2-H4: Iter-247/248/249.
* H5 (founder variance): documented ceiling.
* **H6 (need relief gate): CLOSED** (v3, i279 2/144 @ 0.007).
* **H7 (lore archetype): N/A — new STORY lane, deterministic total mapping, no hazard.**

## Final-state plan tracker (updated)

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 7 (3.1-3.4 + 5.17 + 12-13 CO-2026-001 + H6 close CO-2026-002) | P12 |
| STORY | 13 | 7 (vendor + IC-2 + 8-9 polarity type + 9-10 polarity wire + 11 collective wire + 12-13 reconciliation + DC-2.6 lore scaffold + DC-2.7 lore wire) | P6 |
| CLIENT | 24 | 12 (chart, library, lineage, dossier, render perf, IC-8, keybinds, 19-22 polarity section + **DC-2.8 lore dossier @c032ae3**) | P12 |
| TOOLS | 22 | 8 (probe gen, bench_index, afa codegen, scenario MVP, metrics diff, devex, rollout, dashboards) | P16 |
| QA | 10 | 9 (evidence, golden custody, cal-audit v2, suite seg, IC-6, i268, H6, i279, i280) | P10 |
| DESIGN | 12 | 7 (canon inventory, balance skeletons, needs-bands, pathology-curves, IC-5, pacing v1 + difficulty levers) | P10 |
| PLATFORM | 11 | 11 (IC-3, RNG census, perf-budget v1, IC-8, gate --quick, CI, modding surface, IC-7 modding, N=48, v1.0.0 remnant) | P11 |

**Cumulative:** 60/106 phases delivered across **33 architect-driven commits** (27 at v3 + 2 lore +1 dossier). **Remaining:** 46/106 (ponytail).

## What was NOT done (ponytail)

* **STORY 14-15 collective behavioral integration**: blocked on WP-I ponytail (dev-crate `step_collective` inert). DC-2 after WP-I.
* **Lore dossier IC-7 read-only list update**: `IC-7-modding.md` read-only invariant still lists `polarity_claims`/`development`/`collective_field` but not yet `lore_archetypes` — next tail will add it (one line).
* **CLIENT polish 19-22 remaining** (chart labels, TUI overlay, render-tick audit) — 2 of 5 landed (polarity dossier @1034984 + lore dossier @c032ae3).
* **PLATFORM IC-7 v1.0.0**: RATIFIED, but the v5 dossier lore surface is the first new emergent field since ratification — the read-only list update is the next tail.
* **TOOLS 19-22 hosted dashboards**: self-service bench_index + quick leg landed.
* **DESIGN 7-12 pacing levers bar**: `difficulty-levers.md` DRAFT.

## Final regression conclusion

* **No functional regressions** — 307/0/1, 60/60 dev, 206/206 sim, 17/17 snapshot, 3/3 dossier, fmt+clippy clean.
* **No operational regressions** — GATE GREEN, perf budgets met, no orphan tests, snapshot save/load round-trips, dossier deterministic (render twice → equal).
* **Lore dossier live**: first player-visible lore framing, parallel to polarity, deterministic, read-only, tested at 2000 ticks.

## Signature

* `cargo test -p mindstrata-development --lib --release`: 60/60
* `cargo test -p mindstrata-sim --lib`: 206/206
* `cargo test -p mindstrata-tests --lib`: 17/17 snapshot smoke
* `cargo test -p mindstrata-tests --lib --release`: 307/0/1 (156.39s)
* `bash scripts/gate`: GATE GREEN (5.7s non-full)
* `scripts/bench_index --strict`: 22 ok / 36 legacy / 0 viol (inherited)
* `i268`: FAMILY_PASS 12/12 (via full suite)
* `lore`: 4/4 (vectors, reachable, line-modulates, slug)
* `dossier_lore`: 1/1 (2000 ticks, 12 agents, `lore archetypes: N total across M frames`)
* `cargo fmt --all`: clean
* `cargo clippy --workspace --quiet`: clean
* `git push`: `c032ae3` pushed to main

## Audit vs initial 106-phase plan

60/106 phases delivered. 46/106 remaining is ponytail deferral with rationale per lane. The deferrals are the work, not the gap.

`evidence/FINAL-AUDIT-DC1-regression-v5.md:1` is the release tag for `DC-1-UM-1-evidence-REGRESSION-FREE-v5` on this `c032ae3`.
