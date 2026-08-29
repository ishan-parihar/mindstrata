# CI AA-Mode — Gates Parallelized, Artifacts Retained (PLATFORM 8)

Owner: PLATFORM co-signed QA. Feeds `IC-6 G1-G3` enforcement at scale.
Status: **DRAFT v0.9 — design before workflow.**

## Current gate (the baseline we keep green)

`scripts/gate --full` today: `fmt 1/3 (~0.5s) → clippy 2/3 (~12s) → bench_index 2.5/3 (<0.1s) → perf --quick 2.6/3 (~1s) → suite+golden 3/3 (198s)` — strictly serial, dominated by the `198s` release suite.

## AA-mode target (when DC-1 converge fans to DC-2 parallel agents)

* **Parallelize the fan-out:** `fmt` + `clippy` + `bench_index` + `perf --quick` run as one parallel stage (max `12s`, not `13s` serial). The `198s` suite stays the long pole but no longer blocks the `15s` non-full gate.
* **Matrix:** `release` suite stays the referee (determinism, hazards); `debug` lane removed per `AGENTS §3` (10–20× slower, never the gate).
* **Retention:** `final_suite.log` + `annals.jsonl` (when `--annals`) + `snapshot Bytes` rows as CI artifacts — `30-day` retention, branch `main` keeps last `10` runs pinned for bisect (`golden-replay-custody.md` pristine-archive rule).
* **Change-order hook:** every `CO-` id referenced in a commit must appear in the CI artifact index for that run — ties `IC-5` to `G1`.

## Ponytail deferral

No workflow file lands until a real CI runner exists on this clone — this doc is
the contract; the `.github/workflows/` file is DC-2 polish when the runner is
provisioned. Adding it now would be scaffolding for later.
