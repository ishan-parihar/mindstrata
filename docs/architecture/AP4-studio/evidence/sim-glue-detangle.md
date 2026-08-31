# Glue detangle (SIM 1-2, 2026-08-31)

Owner: SIM. Generated 2026-08-31 against `1241eda` (86/106).
Status: **DRAFT — `*_impl` glue detangle batch 1, `IDENTICAL` when landed.**

## Scope

* `SIM 1-2` detangle `sim/*_impl.rs` + `household.rs`/`economy.rs`/`social_cluster.rs` glue — pure `impl Simulation` blocks using `pub(super)`, `use super::*` transitional wildcards already settled at `2.6` (`dc8c05f`), so detangle is file splits, not new logic.
* Referee is `golden_replay 5/5` `IDENTICAL` + `307/0/1 @141s` + `fmt+clippy` clean — same as `Arc-D` batch 1 `1.12` `systems/` move `@80e1a3f`.

## Why ponytail

`SIM 5` remaining (`1-2` detangle + `3-4` remaining stabilization) are converge polish — `SIM 9/14` already proves `SIM` tail is not gate-blocking ( `208/208` sim, `golden 5/5`, `14` frozen, `IC-7` read-only). Land with `WP-I` drives `STORY 15` — same batch as `DESIGN` final CO.

## Next

Land with `dev/sim` crate boundary when `WP-I` lands — `cargo test --release` + `golden_replay` `IDENTICAL` proven.
