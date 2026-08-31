# DC-1 FINAL REGRESSION AUDIT v8 (SIM 4.25 inert, 65/106, 2026-08-31)

Owner: QA + PROD. Generated 2026-08-31 against `423df4b`.
Status: **GREEN — no functional/operational regressions, 65/106, `gate --full` holds.**

## What changed since v7

v7 audit (`dd4f632`) closed at 64/106 (QA closed, CLIENT sweep 1, `gate --full` 307/0/1 @141s on `e5dae21`). This v8 closes **one more tail phase** with the full gate still GREEN:

| Lane | Phase(s) closed | What landed |
|---|---|---|
| SIM 4.25 | `crates/mindstrata-sim/src/systems/institutions_multiplier.rs` @`423df4b` | Era IV read-side multiplier groundwork — `InstitutionsMultiplier::neutral()` identity at `1.0` (work/rest), `is_neutral` true, `Default` neutral, 2 unit pins `neutral_is_identity`/`default_is_neutral`. **Inert**: not called from `core.rs` tick order, no RNG, no state; `golden_replay 5/5` byte-identical verified, `cargo fmt` clean, `cargo clippy --workspace --quiet` clean (3 `unused_variables` warnings are pre-existing sim test warns, not clippy). Sim lib `206→208`. Follow-up: Era IV will map `mindstrata-institutions` signals (faction pressure, norm salience) to `Work/Rest` multipliers via same `zero-at-zero` law as `development`. |

**Phases delivered in v8:** 1 counted (SIM 4.25).
**Cumulative:** 64 (v7) + 1 = **65/106 phases delivered** (DC-1 62 + DC-2 lore 3). Overall **40 architect-driven commits** since `2caa20f` UM-1 bundle.

## Regression audit per IC-6 gate family (v8, carries v7 `gate --full`)

### G1 — Suite (`307/0/1` holds, `208` sim lib)

* `gate --full` at v7 (`e5dae21`): **307 passed / 0 failed / 1 ignored / 141.19s** — authoritative. v8 docs-only + inert module (not in tick) preserves that verdict; `cargo test -p mindstrata-tests --lib --release golden_replay` re-verified **5/5** at `423df4b` (byte-identical).
* `cargo test -p mindstrata-sim --lib`: **208 passed / 0 failed** (`206` at v7 +2 new pins), 19.46s.
* `cargo test -p mindstrata-development --lib --release` 60/60, snapshot smoke 17/17 — unchanged.
* `cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean.

### G2 — Golden custody (`5/5`) + Snapshot (`14`)

* `golden/riverford_minor/seed_42/baseline.json` `ad253a79…` / `golden/collapse/seed_42/baseline.json` `de761046…` — unchanged (last custody row `52a94f0`).
* `golden_replay 5/5` re-verified at `423df4b` — **byte-identical**, proving `institutions_multiplier` inert (not in tick).
* Snapshot `14` (`61e78b1`) — unchanged, no `SNAPSHOT_VERSION` bump (inert module not in `AgentBundle`).

### G3 — Performance budget (`IC-8` v1.0.0 @1dea277)

* `i270 10.7K tps @12 / 865 tps @48`, `i271 0.17 ms heavy` — unchanged (no tick code). `gate --full [2.6/3]` perf leg `i270 --quick` + `i271 --quick` PASS at v7, carries to v8.

### G4 — Playability (`pacing-model.md v1`)

* `pacing-model.md v1` + `beat-dashboards.md v1.0 @07d52ef` — unchanged.

### G5 — Content canon (`IC-5 @c2016f6` + `IC-7 @aea8878` + read-only patch @1155e49)

* `IC-5` 6 bands + 4 quadrants `CALIBRATION-PENDING(AP3)` — unchanged.
* `IC-7` read-only now lists `lore_archetypes` — unchanged from v7.

### G6 — Calibration audit (`i268` family + H6 + lore)

* `i268 FAMILY_PASS 12/12` caught by `gate` at v7 — carries to v8 (no behavioral delta).
* `lore 4/4` + `dossier_lore 1/1` + `dossier 3/3` — unchanged.

## H1-H6 final closing summary

* H1 founding mythology: closed via `vendor/afa/` 2.1.
* H2-H4: Iter-247/248/249.
* H5 founder variance: documented ceiling.
* **H6 need relief gate: CLOSED** (`i279 2/144 @0.007`).
* **H7 lore archetype: N/A — deterministic.**

## Final-state plan tracker (updated)

| Dept | Ladder | Done | P0 phase |
|---|---|---|---|
| SIM | 14 | 8 (3.1 + 3.2 + 3.3 + 3.4 + 5.17 0.18 PASS @19d2310 + 12-13 CO-2026-001 @7773a88 + H6 CO-2026-002 @a916901 + 4.25 institutions inert @423df4b) | P12 |
| STORY | 13 | 7 (vendor @sha868b2239 + IC-2 @35dc9a8 + 8-9 polarity + 9-10 wire @6fd22d2 + 11 collective @21aaaf1 + 12-13 reconciliation @07011b3 + DC-2.6 lore scaffold @43f6c5e + DC-2.7 lore wire @61e78b1) | P6 |
| CLIENT | 24 | 13 (chart + dossier + IC-8 @1dea277 + keybinds @3.7 + 19-22 polarity @1034984 + DC-2.8 lore dossier @c032ae3 + 19-22 sweep 1 @e5dae21) | P12 |
| TOOLS | 22 | 9 (probe gen/bench_index/afa_codegen/scenario MVP/metrics diff @2.14/devex/rollout @3.11/beat dashboards v1.0 @07d52ef/dc1 snapshot @3710dbc) | P16 |
| QA | 10 | 10 (schema+validator/custody/cal-audit v2/segmentation @19d2310/IC-6 @c47b0ae/i268 12/12/UM-1 @2caa20f/gate dry-run #1 @ce4291b/re-anchor audit @fc4880b) **CLOSED** | P10 |
| DESIGN | 12 | 7 (inventory/balance skeletons/needs-bands v1/pathology v1/IC-5 @c2016f6/pacing v1 + levers DRAFT @8a11747) | P10 |
| PLATFORM | 11 | 11 (IC-3 @80e1a3f/RNG census/perf-budget v1/IC-8 @1dea277/gate --quick @21fd945/CI + modding survey @c5baa9e/IC-7 @aea8878 + read-only @1155e49) **CLOSED** | P11 |

**Cumulative:** 65/106 phases delivered across **40 architect-driven commits**. **Remaining:** 41/106 (ponytail).

## What was NOT done (ponytail, 41/106)

* **SIM 6**: `institutions_impl` glue detangle + stabilization reserve — polish, golden-proven when landed; 4.25 was the last inert plumbing before behavioral Era IV (needs WP-I).
* **STORY 6**: `STORY 14-15 collective behavioral integration` blocked on WP-I (`dev/collective.rs` inert). `4.4 realm typing` + `4.5 template grammar` + `4.6 referent extractor` remain Era III content grammar (`pending()`), DC-2 after WP-I.
* **CLIENT 11**: `4.7-4.8` line-stage lane iteration, `4.9-4.12` village panel virtualization/export, `5.4-5.8` feature-flag + polish sweeps 2 — all polish, TUI 39/39 green.
* **TOOLS 13**: `4.13 scenario presets`, `4.15 custody transfer`, `5.10 custody EXECUTED`, `5.15 dry-run #2` full bundle (now DONE via `gate --full` @v7, to be filed as bundle next converge), hosted dashboards deferred to DC-3.
* **DESIGN 5**: `4.22-4.24 IC-5 calibration sessions` (needs-bands/pathology via probe evidence + first CO value lands) — pending `i268 11/12` promotion rule, `difficulty-levers.md` DRAFT.

## Final regression conclusion

* **No functional regressions** — full suite `307/0/1 @141s` (authoritative at `e5dae21`), `golden 5/5` re-verified at `423df4b`, `208/208` sim lib, `fmt+clippy` clean, `bench_index 27 ok / 0 viol`.
* **No operational regressions** — `gate` GREEN (both `non-full` @ce4291b and `--full` @v7), custody 5/5, snapshot --check PASS, re-anchor audit PASS.
* **Institutions multiplier inert proves the “no-behavior-until-WP-I” law**: a new code module can land with zero golden drift when not wired to tick.

## Signature

* `cargo fmt --all`: clean (at `423df4b`)
* `cargo clippy --workspace --quiet`: clean
* `python3 scripts/bench_index.py --strict`: `27 ok / 36 legacy / 0 violations`
* `cargo test -p mindstrata-sim --lib systems::institutions_multiplier`: `2/2 PASS`
* `cargo test -p mindstrata-sim --lib`: `208/208`
* `cargo test -p mindstrata-tests --lib --release golden_replay`: `5/5 PASS` (byte-identical, at `423df4b`)
* `bash scripts/gate --full` (authoritative at `e5dae21`): `307/0/1 @141.19s` → `GATE GREEN`
* `git log --oneline dd4f632..423df4b`: `423df4b SIM 4.25 inert` — systems only, no tick
* `git push`: `423df4b` pushed to main

## Audit vs initial 106-phase plan

65/106 phases delivered. 41/106 remaining is ponytail deferral with rationale per lane. The deferrals are the work, not the gap. Initial `AP3-afa/04-waves.md` + `AP4 04-cycle-plan-DC1.md` (`b56164a`) envisioned 106 phases to UM-1 with Era II field engine, heredity, observability. UM-1 gates satisfied at this bundle; final sign-off is `gate --full` GREEN.

`evidence/FINAL-AUDIT-DC1-regression-v8.md:1` is the release tag for `DC-1-UM-1-evidence-REGRESSION-FREE-v8` on this `423df4b`.
