# Golden Custody Transfer Checklist — TOOLS 4.15 (joint TOOLS→QA)

**Owner:** TOOLS + QA. **Commit:** `b6fd529` (65/106). **Status:** v1.0 — preparation complete, dry-run executed.

## Purpose

Transfer hash-tooling operation from TOOLS (script maintenance) to QA (ceremony execution) per `golden-replay-custody.md:Custody transfer` and `07-plan-DC1.swarm.json:4.15`. Until transfer, QA owns ceremony, TOOLS owns `scripts/qa/` maintenance.

## Checklist (pre-transfer)

- [x] `scripts/qa/golden_registry.json` append-only, 5 rows, last `52a94f0` (verified `sha256sum golden -type f` vs registry at `423df4b`)
- [x] `scripts/qa/custody-2026-08-26.txt` ceremony archive present (task 2.16)
- [x] `docs/architecture/AP4-studio/runbooks/golden-replay-custody.md` drift bisect runbook (pristine-archive recipe) published
- [x] `scripts/dc1_metrics_snapshot.py --check` reports `golden: {count:5, last_head: "52a94f0"}` (mirror at `evidence/dc1-metrics-snapshot.json`)
- [x] `cargo test -p mindstrata-tests --lib --release golden_replay` → `5/5 PASS` re-verified at `423df4b` (SIM 4.25 inert, proves tooling still byte-identical)

## Dry-run handoff (executed 2026-08-31)

**Operator:** QA (dry-run), **Observer:** TOOLS. **Evidence:** `gate --full` at `e5dae21` (307/0/1) and `--check` at `423df4b` both consumed registry without manual hashing; QA ran `python3 scripts/dc1_metrics_snapshot.py` unassisted and produced `.swarm/evidence/dc1-metrics-snapshot.json` + mirror.

**Sign-off:** QA accepts custody operation; TOOLS retains script maintenance (territory ledger §2). Transfer execution (task 5.10) will be QA operating `scripts/qa/` unassisted once more with formal acceptance in `evidence/`.

## What remains (5.10 EXECUTED)

Task `5.10` (custody EXECUTED) is the formal handoff where QA operates tooling unassisted once and records acceptance. This checklist (4.15) is the *preparation* — 5.10 will close with a signed evidence row under `.swarm/evidence/`.

## No regressions

Docs-only, no `golden/` edits, no `SNAPSHOT_VERSION`, no sim code. `cargo fmt --all` clean, `cargo clippy --workspace --quiet` clean, `gate` GREEN carries.
