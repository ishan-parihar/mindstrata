#!/usr/bin/env python3
"""Vendor ontology extracts from the KosmOS vault into CSVs + cells/ (stdlib only).

Source of record: /home/ishanp/Documents/knowledge-base/KosmOS/_Ontology at the
commit pinned in PROVENANCE.md. Reproduce with:

    python3 vendor/afa/extract.py <vault-root> [out-dir default: same dir as script]

Outputs: ladder.csv, lines.csv, couplings.csv, cells/<line>/<NN>-<slug>.md
"""

from __future__ import annotations

import csv
import re
import shutil
import sys
from pathlib import Path


def parse_frontmatter(text: str) -> dict:
    """Minimal YAML-frontmatter parser for flat `key: value` headers."""
    fm: dict = {}
    if not text.startswith("---"):
        return fm
    lines = text.split("\n")
    if lines[0].strip() != "---":
        return fm
    for ln in lines[1:]:
        s = ln.strip()
        if s == "---":
            break
        m = re.match(r"^([A-Za-z0-9_]+):\s*(.*)$", s)
        if m and not s.startswith(("[", "{")):
            val = m.group(2).strip().strip("'\"")
            fm[m.group(1)] = val
    return fm


def extract_ladder(vault: Path) -> list[dict]:
    stages_md = (vault / "stages.md").read_text(encoding="utf-8")
    sec = re.search(r"## The Canonical 17-Stage Ladder\n(.*?)(?=\n## |\Z)", stages_md, re.S)
    if not sec:
        raise SystemExit("canonical ladder section not found in stages.md")
    rows = []
    for line in sec.group(1).splitlines():
        line = line.strip()
        if not line.startswith("|") or set(line) <= {"|", "-", " "}:
            continue
        cols = [c.strip().strip("`") for c in line.strip("|").split("|")]
        if len(cols) != 8 or not cols[0].isdigit():
            continue
        rows.append(
            {
                "stage": int(cols[0]),
                "slug": cols[1],
                "canonical_name": cols[2],
                "altitude": cols[3],
                "identity_band": cols[4],
                "mhc_order": cols[5],
                "mhc_name": cols[6],
                "kegan": cols[7],
            }
        )
    return rows


def extract_lines(vault: Path) -> list[dict]:
    rows = []
    lines_dir = vault / "lines"
    for md in sorted(lines_dir.glob("*.md")):
        if md.stem.startswith("_"):
            continue
        fm = parse_frontmatter(md.read_text(encoding="utf-8", errors="replace"))
        if not fm.get("line"):
            continue
        rows.append(
            {
                "line_slug": fm.get("line"),
                "kind": fm.get("kind", ""),
                "quadrant": fm.get("quadrant", ""),
                "status": fm.get("status", ""),
                "matrix_cells_required": fm.get("matrix_cells_required", ""),
                "matrix_depth": fm.get("matrix_depth", ""),
            }
        )
    return rows


def extract_cells(vault: Path, out_cells: Path) -> list[dict]:
    by_line = vault / "stages" / "by-line"
    rows = []
    if out_cells.exists():
        shutil.rmtree(out_cells)
    for line_dir in sorted(p for p in by_line.iterdir() if p.is_dir() and not p.name.startswith("_")):
        dest_line = out_cells / line_dir.name
        dest_line.mkdir(parents=True)
        for cell_dir in sorted(p for p in line_dir.iterdir() if p.is_dir() and not p.name.startswith("_")):
            idx = cell_dir / "_index.md"
            if not idx.exists():
                continue
            fm = parse_frontmatter(idx.read_text(encoding="utf-8", errors="replace"))
            dest = dest_line / f"{cell_dir.name}.md"
            shutil.copyfile(idx, dest)
            rows.append(
                {
                    "line": fm.get("line", line_dir.name),
                    "stage": fm.get("stage", ""),
                    "stage_slug": fm.get("stage_slug", cell_dir.name),
                    "altitude": fm.get("altitude", ""),
                    "depth_status": fm.get("depth_status", ""),
                    "cell_path": str(dest.relative_to(out_cells.parent)),
                }
            )
    return rows


def write_csv(path: Path, fieldnames: list[str], rows: list[dict]) -> None:
    with path.open("w", newline="", encoding="utf-8") as f:
        w = csv.DictWriter(f, fieldnames=fieldnames)
        w.writeheader()
        w.writerows(rows)


def main(argv: list[str]) -> int:
    if len(argv) < 2:
        print(__doc__)
        return 1
    vault = Path(argv[1]).resolve()
    out = Path(argv[2]).resolve() if len(argv) > 2 else Path(__file__).resolve().parent

    ladder = extract_ladder(vault)
    lines = extract_lines(vault)
    cells = extract_cells(vault, out / "cells")

    write_csv(out / "ladder.csv", ["stage", "slug", "canonical_name", "altitude", "identity_band", "mhc_order", "mhc_name", "kegan"], ladder)
    write_csv(out / "lines.csv", ["line_slug", "kind", "quadrant", "status", "matrix_cells_required", "matrix_depth"], lines)
    write_csv(out / "couplings.csv", ["line", "stage", "stage_slug", "altitude", "depth_status", "cell_path"], cells)

    print(f"ladder_rows={len(ladder)} lines_rows={len(lines)} cell_rows={len(cells)}")
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
