#!/usr/bin/env python3
"""i391 — field-level write/read census (stdlib only).

The recurring dead-leg class this exists to catch (§5 hazards): a struct field that
is *written* every tick but never read, or written and read by nothing, is a silent
producer with no consumer — i313 (injury), i124 (emotion context), i307 (habit gate)
and `sensory_acuity` were all found by hand, one at a time.

Usage:
    python3 scripts/field_census.py [Struct ...]        # default: the four W0 structs
    python3 scripts/field_census.py --file <path>...    # every struct in the named files
    python3 scripts/field_census.py --json             # machine-readable
    python3 scripts/field_census.py --sites            # print every read site (file:line)
    python3 scripts/field_census.py --report-only      # exit 0 always (same as default)

Exit code is always 0: this is an instrument, not a gate. Suspect rows are a
*sieve*, not a verdict — a field name can be shared by two structs, so every
flagged row is confirmed by hand before it is wired or deleted.

Classification per field, over production code only (`#[cfg(test)]` blocks and
`*tests.rs` / `tests/` files are excluded — a field read only by its own tests is
still a dead consumer in the engine):

    write   = `x.field = v`, `x.field += v`, …  (assignment forms)
    literal = `field: v` in a struct literal / constructor
    read    = `x.field` in any other position

Verdicts:
    PASS-THROUGH-ONLY  written, and the only reads are inside its OWN file (the
                       constructor / `inherit` / `blend` / serde plumbing) → the
                       field is copied around and no behaviour consumes it
    DEAD-FIELD         no writes, no literals, no reads → the field does nothing
    READ-ONLY          read but never written anywhere → set only by deserialization?
    LIVE               at least one read crosses the module boundary
"""

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
CRATES = ROOT / "crates"

DEFAULT_STRUCTS = ("Genome", "EmbodiedState", "BodyState", "DevelopmentFieldState")

STRUCT_RE = r"(?:pub(?:\([^)]*\))?\s+)?struct\s+{name}\b"
ANY_STRUCT_RE = re.compile(
    r"^\s*(?:pub(?:\([^)]*\))?\s+)?struct\s+([A-Za-z_][A-Za-z0-9_]*)", re.MULTILINE
)
FIELD_RE = re.compile(r"^\s*(?:pub(?:\([^)]*\))?\s+)?([a-z_][a-z0-9_]*)\s*:")
CFG_TEST_RE = re.compile(r"^\s*#\[cfg\(test\)\]")


def rust_files() -> list[Path]:
    return sorted(p for p in CRATES.rglob("*.rs"))


def is_test_file(p: Path) -> bool:
    parts = set(p.parts)
    return (
        "tests" in parts
        or "integration_tests" in parts
        or "mindstrata-tests" in parts
        or p.name == "tests.rs"
        or p.name.startswith("test_")
    )


def strip_test_blocks(src: str) -> str:
    """Drop everything from the first top-level `#[cfg(test)]` to end of file."""
    lines = src.splitlines()
    for i, line in enumerate(lines):
        if CFG_TEST_RE.match(line):
            return "\n".join(lines[:i])
    return src


def struct_fields(src: str, name: str) -> list[str]:
    """Field names of `name`'s definition, in declaration order."""
    m = re.search(STRUCT_RE.format(name=name), src)
    if not m:
        return []
    # Scan forward from the `{` that opens the body to its matching `}`.
    i = src.find("{", m.end() - 1)
    if i < 0:
        return []
    depth = 0
    fields: list[str] = []
    for line in src[i:].splitlines():
        stripped = line.strip()
        depth += line.count("{") - line.count("}")
        if depth <= 0 and fields:
            break
        if stripped.startswith("//") or stripped.startswith("#["):
            continue
        fm = FIELD_RE.match(line)
        if fm:
            fields.append(fm.group(1))
    return fields


def census(
    fields: list[str], corpus: list[tuple[str, str]], own_file: str | None = None
) -> dict[str, dict]:
    out: dict[str, dict] = {}
    for f in fields:
        write_re = re.compile(rf"\.{f}\s*(?:=|\+=|-=|\*=|\/=|\|=|&=|\^=)(?!=)")
        literal_re = re.compile(rf"(?<![.\w]){f}\s*:")
        read_re = re.compile(rf"\.{f}\b(?!\s*(?:=|\+=|-=|\*=|\/=|\|=|&=|\^=)(?!=))")
        rows = {
            "write": 0,
            "literal": 0,
            "read": 0,
            "consumer_read": 0,
            "sites": [],
            "read_sites": [],
            "own_file": own_file,
        }
        for path, src in corpus:
            for kind, rx in (("write", write_re), ("literal", literal_re), ("read", read_re)):
                rows[kind] += len(rx.findall(src))
            for i, line in enumerate(src.splitlines(), start=1):
                if read_re.search(line):
                    rows["read_sites"].append(f"{path}:{i}")
                    if path == rows["own_file"]:
                        continue
                    rows["consumer_read"] += 1
                    if path not in rows["sites"]:
                        rows["sites"].append(path)
        # A read in the type's OWN file is plumbing (constructor, inherit/blend,
        # serde shim) — it proves the field is copied, not that any behaviour
        # consumes it. Only a read that crosses the module boundary is a consumer.
        if rows["consumer_read"] == 0 and (rows["write"] or rows["literal"]):
            rows["verdict"] = "PASS-THROUGH-ONLY"
        elif rows["read"] == 0 and rows["write"] == 0 and rows["literal"] == 0:
            rows["verdict"] = "DEAD-FIELD"
        elif rows["write"] == 0 and rows["literal"] == 0:
            rows["verdict"] = "READ-ONLY"
        else:
            rows["verdict"] = "LIVE"
        out[f] = rows
    return out


def main(argv: list[str]) -> int:
    as_json = "--json" in argv
    args = [a for a in argv[1:] if not a.startswith("-")]
    # `--file <path>...` scans EVERY struct in the named files — the nested
    # predisposition/potential structs live in the same modules as the aggregate
    # structs, and a nested field is exactly where the known dead legs hid
    # (`sensory_acuity` lives in `PhysicalPotential`, not in `Genome`).
    files = args if "--file" in argv else []
    names = () if files else (tuple(args) or DEFAULT_STRUCTS)

    corpus: list[tuple[str, str]] = []
    for p in rust_files():
        if is_test_file(p):
            continue
        corpus.append((str(p.relative_to(ROOT)), strip_test_blocks(p.read_text(encoding="utf-8"))))

    report: dict[str, dict] = {}
    if files:
        for rel in files:
            src = (ROOT / rel).read_text(encoding="utf-8")
            for m in ANY_STRUCT_RE.finditer(strip_test_blocks(src)):
                nested = m.group(1)
                fields = struct_fields(src, nested)
                if fields:
                    report.setdefault(nested, {})[rel] = census(fields, corpus, own_file=rel)
    else:
        for name in names:
            defs = []
            for path, src in corpus:
                fields = struct_fields(src, name)
                if fields:
                    defs.append((path, fields))
            if not defs:
                report[name] = {"error": "struct not found in production code"}
                continue
            report[name] = {}
            for path, fields in defs:
                rows = census(fields, corpus, own_file=path)
                report[name][path] = rows

    if as_json:
        for per_struct in report.values():
            for rows in per_struct.values():
                for r in rows.values() if isinstance(rows, dict) else []:
                    if isinstance(r, dict):
                        r.pop("sites", None)
                        r.pop("read_sites", None)
        print(json.dumps(report, indent=2))
        return 0

    show_sites = "--sites" in argv

    dead = 0
    for name, per_struct in report.items():
        if "error" in per_struct:
            print(f"== {name}: {per_struct['error']}")
            continue
        for path, rows in per_struct.items():
            print(f"== {name}  ({path})")
            print(
                f"   {'field':<28} {'write':>6} {'lit':>5} {'read':>5} {'x-mod':>5}  verdict"
            )
            for field, r in rows.items():
                suspect = r["verdict"] in ("PASS-THROUGH-ONLY", "DEAD-FIELD", "READ-ONLY")
                mark = "  <-- " if suspect else ""
                print(
                    f"   {field:<28} {r['write']:>6} {r['literal']:>5} {r['read']:>5} "
                    f"{r['consumer_read']:>4}  {r['verdict']}{mark}"
                )
                if suspect:
                    dead += 1
                if show_sites and r["read_sites"]:
                    for site in r["read_sites"][:12]:
                        print(f"        read @ {site}")
            print()
    print(
        f"suspect rows: {dead} (a sieve — confirm each by hand before wiring/deleting; "
        "a row whose only x-mod reads are the field's own module plumbing is real)"
    )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv))
