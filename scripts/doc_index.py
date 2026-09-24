#!/usr/bin/env python3
"""Documentation authority-map checker (see docs/DOCUMENTATION.md §5).

Fails when documentation drifts out of alignment with the code base:

  1. coverage  — a non-exempt doc under docs/ or a root *.md is not classified
                  in docs/DOCUMENTATION.md §6 (a new doc was added unclassified);
  2. ghosts    — an indexed path does not exist (a doc was deleted/renamed and
                  the index was not updated);
  3. freshness — an AUTHORITY or ACTIVE doc under docs/ carries no
                  `reconciled_commit:` marker, or the marker does not resolve in
                  git (the doc was never reconciled against the code);
  4. citations — a `*.md|rs|py|sh` token cited in an AUTHORITY or ACTIVE doc does
                  not resolve (i386's drift class: ghost filenames, moved files,
                  and shipped-behaviour claims the docs say never landed).

Exempt directories are HISTORICAL/REFERENCE by construction (docs/DOCUMENTATION.md
§3) and are deliberately not indexed one-by-one. The citation check likewise
applies only to the must-be-true set (AUTHORITY/ACTIVE + root docs) — HISTORICAL
and REFERENCE docs are frozen records of their era and may cite paths that have
since moved (the same scope the freshness rule uses).

Citation resolution ladder (the project's own path conventions, in order):
  repo-relative (`crates/...`, `scripts/...`), docs-relative, doc-parent-relative,
  evidence shorthand (`evidence/X` → `docs/architecture/AP4-studio/evidence/X`),
  crate-relative (`sim/core.rs` → `crates/mindstrata-sim/src/sim/core.rs`, and
  `core/src/X` → `crates/mindstrata-core/src/X`). Bare filenames resolve to a
  unique basename anywhere in the repo (`git ls-files`).

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

# 4. citations — a `*.md|rs|py|sh` token cited in a must-be-true doc (AUTHORITY or
# ACTIVE per §6, including root docs) must resolve. Historical citations — a
# token a governing doc names AS a past ghost/retired path while describing
# drift (AGENTS.md §4.14 names i386's ghosts and moved files as the drift class
# it fixed) — are curated here, one entry per frozen historical record, with the
# mechanism named. Anything not in this set must resolve now.
HISTORICAL_CITES: set[tuple[str, str]] = {
    # i386's ghost interlock contracts, cited as ghosts that never existed (AGENTS §4.14,
    # and the PLAN_DC3 i386 ledger row that records the same sweep).
    ("AGENTS.md", "IC-2-observability.md"),
    ("AGENTS.md", "IC-7-ui-telemetry.md"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "IC-2-observability.md"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "contracts/IC-2-observability.md"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "IC-7-ui-telemetry.md"),
    # i386's moved-file examples, cited as retired pre-extraction paths (AGENTS §3/§4.14
    # bare + path forms; the PLAN_DC3 i386 row quoting what it re-pointed).
    ("AGENTS.md", "sim/pass_health.rs"),
    ("AGENTS.md", "sim/pass_biology.rs"),
    ("AGENTS.md", "pass_health.rs"),
    ("AGENTS.md", "pass_biology.rs"),
    ("AGENTS.md", "psychology/lore.rs"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "sim/pass_health.rs"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "sim/pass_biology.rs"),
    # AGENTS §7 names the historical "sim.rs split" — the god-file is deliberately gone.
    ("AGENTS.md", "sim.rs"),
    # Vendor-blocked external vault artifacts, cited as the unblock-pending items they are
    # (AGENTS §8, ENGINE_STATUS vendor block, PLAN_DC3 §2/§6, i291 row: "until rays.md vendors").
    ("AGENTS.md", "realms.md"),
    ("docs/ENGINE_STATUS.md", "realms.md"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "realms.md"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "rays.md"),
    # The i421 ledger row quotes the tokens it curates (same records, cited as records).
    ("docs/PLAN_DC5_DEVELOPMENT.md", "realms.md"),
    ("docs/PLAN_DC5_DEVELOPMENT.md", "rays.md"),
    ("docs/PLAN_DC5_DEVELOPMENT.md", "tests/comparison.rs"),
    # rust-craft C2's documented deletion, cited as the suite-count delta (ENGINE_STATUS §1).
    ("docs/ENGINE_STATUS.md", "tests/comparison.rs"),
    # The PLAN_DC3 i386 row names the never-shipped artifacts it swept (G2/G4 rows).
    ("docs/PLAN_DC3_DEVELOPMENT.md", "scripts/golden_replay.sh"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "scripts/playthrough_smoke.sh"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "keybind_cheatsheet.md"),
    ("docs/PLAN_DC3_DEVELOPMENT.md", "dossier_flow.md"),
}

# Cited-token shape: backtick-quoted or path/bare-name, one of the four
# gate-relevant extensions. URLs, absolute paths, ~-paths and globs are not
# resolvable repo citations and are skipped upstream.
CITE_TOKEN_RE = re.compile(r"`?([A-Za-z0-9_./-]+\.(?:md|rs|py|sh))\b`?")
FENCE_RE = re.compile(r"^(```|~~~)")
URL_RE = re.compile(r"https?://\S+")
EVIDENCE_DIR = "docs/architecture/AP4-studio/evidence"
AP4_STUDIO = REPO / "docs" / "architecture" / "AP4-studio"


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


def _tracked_basenames() -> set[str]:
    """Basenames of every tracked file (for bare-filename citation resolution)."""
    proc = subprocess.run(
        ["git", "ls-files"], cwd=REPO, capture_output=True, text=True, check=True
    )
    return {line.rsplit("/", 1)[-1] for line in proc.stdout.splitlines() if line}


def citation_resolves(tok: str, doc: str, basenames: set[str]) -> bool:
    """One cited token against the project's path conventions (header §4)."""
    if "/" in tok:
        candidates = [REPO / tok, REPO / "docs" / tok, (REPO / doc).parent / tok]
        # AP4-studio shorthands (mirroring the project's own §4.14-era prose):
        # `evidence/X`, `charters/X`, `contracts/X`, `runbooks/X` are all rooted
        # under docs/architecture/AP4-studio/; `archive/X` under
        # docs/architecture/archive/; and any `AP…/…` token is a path under
        # docs/architecture/.
        for prefix, base in (
            ("evidence/", AP4_STUDIO / "evidence"),
            ("charters/", AP4_STUDIO / "charters"),
            ("contracts/", AP4_STUDIO / "contracts"),
            ("runbooks/", AP4_STUDIO / "runbooks"),
            ("archive/", REPO / "docs" / "architecture" / "archive"),
        ):
            if tok.startswith(prefix):
                candidates.append(base / tok.removeprefix(prefix))
        if tok.startswith(("AP2", "AP3", "AP4")):
            candidates.append(REPO / "docs" / "architecture" / tok)
        for cand in candidates:
            if cand.is_file():
                return True
        crate_paths = [
            REPO / "crates" / f"mindstrata-{stem}" / "src" / rest
            for stem, rest in _crate_decompositions(tok)
        ]
        return any(c.is_file() for c in crate_paths)
    return tok in basenames


def _crate_decompositions(tok: str) -> list[tuple[str, str]]:
    """Crate-stem guesses for a module path token (superset; existence decides)."""
    out: list[tuple[str, str]] = []
    parts = tok.split("/")
    # `core/src/parameters.rs`, `mindstrata-development/src/lore.rs` — explicit
    # crate/src prefixes.
    if len(parts) >= 3 and parts[1] == "src":
        out.append((parts[0].removeprefix("mindstrata-"), "/".join(parts[2:])))
    # `sim/core.rs`, `person/mind.rs`, `development/lore.rs` — the first
    # component may be the crate stem, the remainder its module path.
    if len(parts) >= 2:
        out.append((parts[0], "/".join(parts[1:])))
    # …or the whole token is a module path under some crate's src/.
    for stem in KNOWN_CRATE_STEMS:
        out.append((stem, tok))
    return out


KNOWN_CRATE_STEMS = (
    "core", "person", "psych", "institutions", "social", "world",
    "development", "sim", "tui", "cli", "render", "benches", "tests",
)


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


def iter_cited_tokens(rel: str) -> list[str]:
    """Every citation-shaped token in a doc, outside fenced code blocks."""
    out: list[str] = []
    in_fence = False
    for line in (REPO / rel).read_text(encoding="utf-8").splitlines():
        if FENCE_RE.match(line.strip()):
            in_fence = not in_fence
            continue
        if in_fence:
            continue
        for m in CITE_TOKEN_RE.finditer(URL_RE.sub("", line)):
            tok = m.group(1)
            if tok.startswith(("~", "/")) or "*" in tok:
                continue  # external, absolute, glob — not resolvable repo citations
            out.append(tok)
    return out


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

    # 4. citations — must-be-true docs only (AUTHORITY/ACTIVE per §6, which
    # includes the root docs). Historical/REFERENCE docs are frozen records.
    basenames = _tracked_basenames()
    for path in sorted(p for p, s in indexed.items() if s in FRESH_STATUSES):
        if not (REPO / path).is_file():
            continue  # already reported as a ghost above
        for tok in iter_cited_tokens(path):
            if (path, tok) in HISTORICAL_CITES:
                continue
            if not citation_resolves(tok, path, basenames):
                failures.append(
                    f"CITATION: {path} cites `{tok}` — no such file "
                    f"(repo/docs/doc-parent/evidence/crate-relative, bare basename)"
                )

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
