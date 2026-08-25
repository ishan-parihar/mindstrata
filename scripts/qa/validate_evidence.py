#!/usr/bin/env python3
"""Validate AP4 evidence bundles against the v1 schema.

Usage:
    python3 scripts/qa/validate_evidence.py <bundle.json> [more.json ...]

Checks (see docs/architecture/AP4-studio/templates/evidence-schema-v1.md):
  - top-level: schema_version == 1, phase is a positive int,
    verdict in {pass, fail}, gates is a list
  - each gate: type (non-empty str), status in {pass, fail, waived},
    summary (non-empty str)
  - artifacts: list of {path, sha256?}; path must exist relative to repo
    root when resolvable, otherwise a warning is emitted (not a failure)

Exit codes: 0 = valid, 1 = invalid (errors printed to stderr).
"""

from __future__ import annotations

import hashlib
import json
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
VALID_VERDICTS = {"pass", "fail"}
VALID_GATE_STATUS = {"pass", "fail", "waived"}


def err(msg: str) -> None:
    print(f"error: {msg}", file=sys.stderr)


def warn(msg: str) -> None:
    print(f"warning: {msg}", file=sys.stderr)


def validate(bundle_path: Path) -> bool:
    ok = True
    try:
        data = json.loads(bundle_path.read_text(encoding="utf-8"))
    except json.JSONDecodeError as e:
        err(f"{bundle_path}: not valid JSON: {e}")
        return False
    except OSError as e:
        err(f"{bundle_path}: unreadable: {e}")
        return False

    def fail(msg: str) -> None:
        nonlocal ok
        ok = False
        err(f"{bundle_path}: {msg}")

    if data.get("schema_version") != 1:
        fail(f"schema_version must be 1, got {data.get('schema_version')!r}")
    phase = data.get("phase")
    if not isinstance(phase, int) or isinstance(phase, bool) or phase < 1:
        fail(f"phase must be a positive integer, got {phase!r}")
    verdict = data.get("verdict")
    if verdict not in VALID_VERDICTS:
        fail(f"verdict must be one of {sorted(VALID_VERDICTS)}, got {verdict!r}")
    gates = data.get("gates", [])
    if not isinstance(gates, list):
        fail("gates must be a list")
        gates = []
    for i, gate in enumerate(gates):
        where = f"gates[{i}]"
        if not isinstance(gate, dict):
            fail(f"{where} must be an object")
            continue
        if not isinstance(gate.get("type"), str) or not gate["type"]:
            fail(f"{where}.type must be a non-empty string")
        status = gate.get("status")
        if status not in VALID_GATE_STATUS:
            fail(f"{where}.status must be one of {sorted(VALID_GATE_STATUS)}, got {status!r}")
        if not isinstance(gate.get("summary"), str) or not gate["summary"]:
            fail(f"{where}.summary must be a non-empty string")

    artifacts = data.get("artifacts", [])
    if not isinstance(artifacts, list):
        artifacts = []
    for i, art in enumerate(artifacts):
        where = f"artifacts[{i}]"
        if not isinstance(art, dict) or not isinstance(art.get("path"), str) or not art["path"]:
            fail(f"{where}.path must be a non-empty string")
            continue
        rel = REPO_ROOT / art["path"] if not Path(art["path"]).is_absolute() else Path(art["path"])
        if not rel.exists():
            warn(f"{where}: path does not exist on disk: {art['path']}")
        expected = art.get("sha256")
        if expected and rel.is_file():
            digest = hashlib.sha256(rel.read_bytes()).hexdigest()
            if digest != expected:
                fail(f"{where}: sha256 mismatch for {art['path']}")
    return ok


def main(argv: list[str]) -> int:
    if len(argv) < 2 or argv[1] in {"-h", "--help"}:
        print(__doc__.strip(), file=sys.stderr if len(argv) < 2 else sys.stdout)
        return 0 if len(argv) >= 2 else 1
    all_ok = True
    for raw in argv[1:]:
        all_ok &= validate(Path(raw))
    return 0 if all_ok else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
