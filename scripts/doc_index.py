#!/usr/bin/env python3
"""Documentation authority-map checker (see docs/DOCUMENTATION.md §5).

Fails when documentation drifts out of alignment with the code base:

  1. coverage  — a non-exempt doc under docs/ or a root *.md is not classified
                 in docs/DOCUMENTATION.md §6 (a new doc was added unclassified);
  2. ghosts    — an indexed path does not exist (a doc was deleted/renamed and
                 the index was not updated);
  3. freshness — an AUTHORITY or ACTIVE doc under docs/ carries no
                 `reconciled_commit:` marker, or the marker does not resolve in
                 git (the doc was never reconciled against the code).

Exempt directories are HISTORICAL/REFERENCE by construction (docs/DOCUMENTATION.md
§3) and are deliberately not indexed one-by-one.

Usage:
    python3 scripts/doc_index.py --check   # exit 1 on any violation
    python3 scripts/doc_index.py --list    # print every unclassified doc
"""
from __future__ import annotations

import os
import re
import subprocess
import sys
from pathlib import Path

REPO = Path(__file__).resolve().parent.parent
INDEX = REPO / "docs" / "DOCUMENTATION.md"

# Auto-classified by directory (DOCUMENTATION.md §3). Any *.md beneath these is
# historical/reference by construction and needs no index row.
EXEMPT_DIR_PARTS = (
    os.sep + "evidence" + os.sep,
    os.sep + "archive" + os.sep,
    os.sep + "waves" + os.sep,
    os.sep + "knowledge-base" + os.sep,
    os.sep + "change-orders" + os.sep,
)

ROOT_DOCS = ("AGENTS.md", "README.md", "CHANGELOG.md", "OPERATIONAL_AUDIT.md")

STATUSES = ("AUTHORITY", "ACTIVE", "REFERENCE", "HISTORICAL", "SUPERSEDED")
# Statuses whose docs must carry a resolvable reconciliation marker.
FRESH_STATUSES = ("AUTHORITY", "ACTIVE")

# Index rows in §6 look like: | `path` | STATUS | note |
ROW_RE = re.compile(
    r"^\|\s*`([^`]+)`\s*\|\s*(" + "|".join(STATUSES) + r")\s*\|"
)
MARKER_RE = re.compile(r"reconciled_commit:\s*([0-9a-f]{7,40}|pending)")


def exempt(rel: str) -> bool:
    norm = os.sep + rel.replace("/", os.sep)
    return any(part in norm for part in EXEMPT_DIR_PARTS)


def iter_docs() -> list[str]:
    """Every doc the checker governs, repo-relative POSIX paths."""
    docs: list[str] = []
    for name in ROOT_DOCS:
        if (REPO / name).is_file():
            docs.append(name)
    for path in sorted((REPO / "docs").rglob("*.md")):
        rel = path.relative_to(REPO).as_posix()
        if not exempt(rel):
            docs.append(rel)
    return docs


def parse_index() -> dict[str, str]:
    """Map indexed path -> declared status, from §6 of DOCUMENTATION.md."""
    text = INDEX.read_text(encoding="utf-8")
    # Restrict to the final index section so the authority-map tables (§2) and
    # the directory table (§3, whose first cell is a glob) cannot be mistaken
    # for index rows.
    marker = "\n## 6. Index"
    body = text.split(marker, 1)[1] if marker in text else ""
    out: dict[str, str] = {}
    for line in body.splitlines():
        m = ROW_RE.match(line)
        if not m:
            continue
        path, status = m.group(1), m.group(2)
        if "*" in path:
            continue
        out[path] = status
    return out


def marker_resolves(rel: str) -> tuple[bool, str]:
    content = (REPO / rel).read_text(encoding="utf-8")
    m = MARKER_RE.search(content)
    if not m:
        return False, "no `reconciled_commit:` marker"
    sha = m.group(1)
    if sha == "pending":
        return True, ""
    proc = subprocess.run(
        ["git", "cat-file", "-e", f"{sha}^{{commit}}"],
        cwd=REPO,
        capture_output=True,
        text=True,
    )
    if proc.returncode != 0:
        return False, f"`reconciled_commit: {sha}` does not resolve in git"
    return True, ""


def main() -> int:
    list_only = "--list" in sys.argv
    indexed = parse_index()
    governed = iter_docs()

    unclassified = [d for d in governed if d not in indexed]
    ghosts = [p for p in indexed if not (REPO / p).is_file()]

    if list_only:
        print("unclassified docs:")
        for d in unclassified:
            print(f"  {d}")
        return 0

    failures: list[str] = []
    for d in unclassified:
        failures.append(f"UNCLASSIFIED: {d}  (add a row to DOCUMENTATION.md §6)")
    for g in ghosts:
        failures.append(f"GHOST: {g}  (indexed but missing on disk)")
    for path, status in sorted(indexed.items()):
        if status not in FRESH_STATUSES:
            continue
        if not path.startswith("docs/"):
            continue  # root docs carry their own reconciliation lines
        ok, why = marker_resolves(path)
        if not ok:
            failures.append(f"STALE: {path} ({status}) — {why}")

    if failures:
        print(f"doc_index: {len(failures)} violation(s)\n")
        for f in failures:
            print(f"  {f}")
        print("\nSee docs/DOCUMENTATION.md for the taxonomy and the reconciliation rule.")
        return 1

    fresh = sum(1 for p, s in indexed.items() if s in FRESH_STATUSES)
    print(
        f"doc_index: OK — {len(governed)} governed docs all classified "
        f"({len(indexed)} indexed, {fresh} carrying a reconciliation marker); "
        f"0 ghosts"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main())
