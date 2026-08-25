# QA Evidence Tooling

- `validate_evidence.py` — validates phase evidence bundles against the v1 schema
  (see `docs/architecture/AP4-studio/templates/evidence-schema-v1.md`).

Usage:

```
python3 scripts/qa/validate_evidence.py scripts/qa/fixtures/sample-bundle.json
```

Exit 0 = valid, 1 = invalid (errors on stderr). Artifact paths are checked relative to
the repo root; missing paths warn, sha256 mismatches fail.
