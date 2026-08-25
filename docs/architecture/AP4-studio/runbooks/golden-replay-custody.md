# Runbook — Golden Replay Custody & Drift Bisect

Owner: QA. The golden replay is the project's referee (AGENTS.md §7: structural changes
prove byte-identical goldens; behavioral changes re-pin with probe evidence).

## Registry

`scripts/qa/golden_registry.json` — append-only rows of `{head_commit, baselines{path:
sha256}, suite_evidence}`. Baseline files live under `golden/<world>/seed_<n>/baseline.json`
(currently `collapse`, `riverford_minor`; both seed_42).

## Ceremony (per structural batch)

1. Before the move: confirm working tree matches last green entry
   (`sha256sum $(find golden -type f)` vs registry).
2. Land the verbatim move. NO behavioral edits in the same commit — ever.
3. `cargo fmt --all && cargo clippy --workspace --quiet && cargo test -p mindstrata-tests --lib --release`
   — full release suite, not subsets.
4. Golden tests must pass with ZERO baseline regeneration. If a golden fails:
   STOP — this is drift, go to bisect below.
5. Append a registry row with the new head sha + re-hashed baselines (hashes must be
   UNCHANGED for a pure refactor; if they changed, the commit was not verbatim).

## Drift bisect (golden or long-horizon test moves with NO code change)

Stale binaries and host toolchain drift produce fix-less reproductions. Rule order:

1. **Rebuild clean**: `cargo clean -p mindstrata-tests && cargo test … --release`.
   A surprising fraction of "drift" is a stale incremental artifact.
2. **Pristine-archive test** (host/toolchain drift detector):
   ```
   git archive <last-green-sha> | tar -x -C /tmp/pristine
   cd /tmp/pristine && cargo test -p mindstrata-tests --lib --release
   ```
   If the pristine tree ALSO fails on this host with no code change → environment or
   toolchain drift, NOT code. Bisect the toolchain (rustup update, linker, insta
   version), never re-pin assertions to make it green. Precedent: Iter-263's endocrine
   snapshot re-anchor was later traced to a host toolchain transition, not simulation
   behavior.
3. **Real code drift**: `git bisect start <bad> <last-green>` driving the single failing
   golden test (`cargo test --release <golden_test_name>` per step).
4. Only after root cause is understood may pins move — via the AGENTS.md §4.2 form
   (measured value, old band, mechanism), recorded against a probe from IC-4 conventions.

## Snapshot drift (insta)

`cargo insta test -p mindstrata-tests --release`, review, then accept ONLY with
documented evidence of why the shift is expected. On hosts without cargo-insta, promote
`.snap.new` bodies into `.snap` by hand while dropping insta's extra `assertion_line`
header field (established local procedure).

## Custody transfer

TOOLS→QA handoff of hash tooling happens at the dedicated transfer task; until then QA
owns ceremony execution and TOOLS owns script maintenance (territory ledger §2).
