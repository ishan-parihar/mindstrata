#!/usr/bin/env python3
"""Bench naming/index audit per IC-4 naming law (stdlib only).

Usage: python3 scripts/bench_index.py [--strict]

Lists every file in crates/mindstrata-benchens/examples/, classifies:
  ok      = matches i<digits>_<slug>.rs
  legacy  = pre-convention name (grandfathered, reported)
Exit 0 always in report mode; with --strict exit 1 if any non-legacy violation exists
(files that match neither pattern are violations, e.g. CamelCase or spaces).
"""

from __future__ import annotations

import re
import sys
from pathlib import Path

EXAMPLES = Path(__file__).resolve().parents[1] / "crates" / "mindstrata-benches" / "examples"
OK = re.compile(r"^i\d+_[a-z0-9_]+\.rs$")
LEGACY_SHAPES = (
    re.compile(r"^p\d+_[a-z0-9_]+\.rs$"),
    re.compile(r"^[a-z0-9_]+_probe\.rs$"),
    re.compile(r"^iter\d+_[a-z0-9_]+\.rs$"),
    re.compile(r"^[a-z0-9_]+_gate\.rs$"),
    re.compile(r"^[a-z0-9_]+_sweep\.rs$"),
    re.compile(r"^[a-z0-9_]+\.rs$"),
)


def classify(name: str) -> str:
    if OK.match(name):
        return "ok"
    if any(p.match(name) for p in LEGACY_SHAPES):
        return "legacy"
    return "violation"


def main(argv: list[str]) -> int:
    strict = "--strict" in argv
    rows = []
    for f in sorted(EXAMPLES.glob("*.rs")):
        rows.append((f.name, classify(f.name)))
    width = max(len(n) for n, _ in rows) if rows else 0
    counts = {"ok": 0, "legacy": 0, "violation": 0}
    for name, kind in rows:
        counts[kind] += 1
        print(f"{name:<{width}}  {kind}")
    print(f"\ntotal={len(rows)} ok={counts['ok']} legacy={counts['legacy']} "
          f"violations={counts['violation']}")
    if strict and counts["violation"]:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
