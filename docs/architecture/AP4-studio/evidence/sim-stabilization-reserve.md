# SIM stabilization reserve (SIM 14, 2026-08-31)

Owner: SIM. Generated 2026-08-31 against `cd6b30e` (75/106).
Status: **RESERVE — filed, no tick impact.**

## Scope

* `institutions_impl` glue detangle + `stabilization` reserve (SIM 13-14) are the last pure-refactor phases before DC-2. Both are `ponytail:` — golden-proven when landed, `IDENTICAL` referee, no behavioral `CO-`.
* Reserve records that `mindstrata-sim/src/sim/mod.rs:711` `MetricsSnapshot` shape is frozen at `14` (snapshot smoke `17/17`, `golden 5/5`), so any `institutions_impl` move must preserve `agent_count`/`total_grain`/`family_count` field order.

## What lands this note

* Documents that SIM 14 is intentionally deferred: the `Arc-D` debt is read-only modding today (IC-7 `read-only` @`aea8878`), so detangling does not unblock UM-1.
* When landed, procedure is `cargo test --release` `307/0/1` + `golden_replay 5/5` `IDENTICAL` + `fmt+clippy` clean — same as `423df4b` (institutions inert) and `a916901` (H6).

## Next

Land with `dev/sim` crate boundary when `WP-I` drives collective behavioral (STORY 14-15) — not before.
