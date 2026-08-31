# Tools converge (TOOLS 4.17-4.18, 2026-08-31)

Owner: TOOLS. Generated 2026-08-31 against `09d330c` (83/106).
Status: **FILED — bench/afa/codegen/scenario converge docs, ponytail.**

## What landed

* `tools-bench-polish.md 4.16` + `dry-run-2-bundle.md 5.15` + `tools-hosted-dashboards.md 5.16` are the converge bundle — `bench_index 27 ok / 36 legacy / 0 violations`, `probe gen` skeletons proven, `afa_codegen` idempotent, `scenario presets 8/8`, `metrics diff` stdlib.
* `4.17-4.18` are file-ownership ledger polish (TOOLS owns `scripts/` + `vendor/afa/` + `crates/mindstrata-benches/examples/i<iter>_*`), already `ponytail:` per `03-interlock-map.md §2`.

## Why ponytail

TOOLS converge needs `WP-I` + `dev/collective.rs` to be code, not docs — `gate --full` `307/0/1 @141s`, `fmt+clippy` clean, `golden 5/5`, `bench_index 27 ok` prove no operational regression without it.

## Next

Close `TOOLS` at `DC-1` with `WP-I` — same batch as `STORY 15` + `SIM` reserve + `DESIGN 4.24`.
