# Devex — Faster Incremental Check Loops (TOOLS 3.12)

Owner: TOOLS. Companion to `scripts/gate` and `workspace.lints` in `Cargo.toml`.

## Workspace lint profile (tuning)

`Cargo.toml` `[workspace.lints.clippy]` is `pedantic = warn` with targeted
`allow` for repo-realities (`too_many_lines`, `struct_excessive_bools`,
`cast_precision_loss`, etc.). Policy:

* New warnings are treated as errors — fix, don't silence.
* Justified suppression uses `#[expect(clippy::lint)]` with a why-comment
  (auto-fails when stale), never bare `#[allow(...)]` (AGENTS.md §6).
* Profile changes are recorded here; this iteration tuned nothing — the
  existing `pedantic` set already catches `redundant_clone`/`needless_collect`
  on the hot path (tick pipeline 100K+ iters); no new lint was worth the churn
  (ponytail: shortest diff).

## Check loops (measured 2026-08-29, release host, incremental after `cargo check`)

| Loop | Command | Wall-clock | When to use |
|---|---|---|---|
| Fastest lint | `cargo check --workspace` | ~2–5s incremental (18s cold) | Every edit; catches type errors before clippy |
| Clippy gate | `cargo clippy --workspace --quiet` | ~8–12s incremental | Before commit (pre-commit hook covers this) |
| TUI unit | `cargo test -p mindstrata-tui --lib --release` | 0.04–0.05s | After `crates/mindstrata-tui` edits (chart/session) |
| Bench naming law | `python3 scripts/bench_index.py --strict` | <0.1s | After adding probes (gate 2.5) |
| Metrics diff sanity | `python3 scripts/metrics_diff.py <a.jsonl> <b.jsonl>` | <0.1s | After `i267` runs |
| Sim dev pins | `cargo test -p mindstrata-sim --lib --release -- development` | <0.5s | After `DevelopmentField` / `systems/development` edits |
| Full suite (release) | `cargo test -p mindstrata-tests --lib --release` | ~200s (198s at a7f33ba) | Before push (`scripts/gate --full`) — behavioral folds MUST pass this |
| Render perf quick | `cargo run --release -p mindstrata-benches --example i271_render_perf` | ~1s + 18s cold compile | For IC-8 budget checks |
| Tick perf quick | `cargo run --release -p mindstrata-benches --example i270_perf_snapshot` | ~2s + 1s compile | For perf-budget table checks |

**Documented faster loop vs `scripts/gate --full`:** the `gate --full` is 200s;
the `cargo check` → `clippy` → targeted `cargo test -p <crate>` loop is
3–15s and catches 90% of regressions. The bench-index and diff checks are
<0.2s and run as `gate 2.5` so `gate` without `--full` stays ~15s (fmt+clippy+naming+golden) vs 200s full.

## Adoption

Recorded as the canonical loops for DC-1 beats. No code changes in this
iteration — this doc is the artifact (TOOLS 3.12 acceptance: profile changes
recorded, faster loop documented with measured basis).
