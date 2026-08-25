# Evidence Schema v1

Owner: QA. Consumed by gate tooling (`scripts/qa/validate_evidence.py`) and milestone
assembly (UM-1 per IC-6). Layout on disk mirrors the swarm orchestration's runtime state:

```
.swarm/evidence/
  {phase}/                  # phase-level bundles
    drift-verifier.json     # drift verification verdicts
    ...                     # one EvidenceBundle per producer
  {taskId}.json             # exact-task QA records
```

> Runtime files under `.swarm/` are owned by the orchestration plugin; QA-authored
> bundles follow the same shapes so one validator covers both.

## Phase bundle — `{phase}/*.json`

```json
{
  "schema_version": 1,
  "phase": 1,
  "verdict": "pass",
  "summary": "one-line human summary of what this evidence attests",
  "produced_at": "2026-08-25T00:00:00Z",
  "producer": "<tool-or-agent-name>",
  "gates": [
    {
      "type": "suite | golden-policy | perf-budget | playability | content-coherence | calibration-audit | custom-tag",
      "status": "pass | fail | waived",
      "summary": "what was checked and the observed result",
      "artifacts": ["optional/relevant/paths"]
    }
  ],
  "artifacts": [
    { "path": "repo-relative/or/absolute", "sha256": "hex, optional but checked" }
  ]
}
```

Rules:

1. `schema_version` is exactly `1`.
2. `phase` is a positive integer matching `.swarm/plan.json` phase ids.
3. `verdict` is `pass` or `fail`; individual gates may be `waived` with justification in
   their `summary` (waiver authority: operator or IC-6).
4. Gate `type` SHOULD come from the six UM families when applicable.
5. Artifact `sha256`, when present, is validated against the file bytes — mismatches fail.
6. Missing artifact paths produce warnings, not failures (bundles may reference
   ephemeral run outputs).

## Exact-task record — `.swarm/evidence/{taskId}.json`

```json
{
  "schema_version": 1,
  "taskId": "1.1",
  "produced_at": "...",
  "gates": { "required": ["pre_check"], "passed": ["pre_check"], "missing": [] },
  "notes": "free-form"
}
```

## Retention

Bundles are append-only history: never edit an archived bundle; supersede with a new
file. Milestone assembly (task UM-1) composes the final bundle from phase bundles plus
probe artifacts and registers sha256s in the golden custody registry.
