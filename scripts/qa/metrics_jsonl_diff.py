#!/usr/bin/env python3
"""metrics_jsonl_diff.py — field-level diff of two MetricsSnapshot JSONL streams.

Usage:
    python3 scripts/qa/metrics_jsonl_diff.py a.jsonl b.jsonl [--tol T] [--verbose]

Algorithm (per DC-1 blueprint):
  pass 1 — parse both streams into tick-keyed dicts; coverage sets reported
           separately from value deltas (run-length differences must never
           silently zip-misalign).
  pass 2 — walk shared ticks ascending; compare fields in the first record's
           key order. u64 counts exact; f64 metrics use abs tol with rel
           fallback for large magnitudes.

NaN/null tokens are hard errors (determinism-honesty: never skip silently).

Exit codes: 0 = identical within tolerance, 1 = divergence or parse error,
2 = usage.
"""

from __future__ import annotations

import json
import sys
from pathlib import Path


def load(path: Path) -> dict[int, dict]:
    rows: dict[int, dict] = {}
    with path.open(encoding="utf-8") as f:
        for lineno, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                rec = json.loads(line)
            except json.JSONDecodeError as e:
                raise SystemExit(f"error: {path}:{lineno}: bad JSON: {e}")
            tick = rec.get("tick")
            if not isinstance(tick, int):
                raise SystemExit(f"error: {path}:{lineno}: missing integer tick")
            if tick in rows:
                raise SystemExit(f"error: {path}:{lineno}: duplicate tick {tick}")
            for k, v in rec.items():
                if v is None or (isinstance(v, float) and v != v):
                    raise SystemExit(f"error: {path}:{lineno}: NaN/null in field {k}")
            rows[tick] = rec
    return rows


def close(a: float, b: float, tol: float) -> bool:
    if a == b:
        return True
    return abs(a - b) <= tol or abs(a - b) <= tol * max(abs(a), abs(b))


def main(argv: list[str]) -> int:
    verbose = "--verbose" in argv
    tol = 1e-12
    rest = argv[1:]
    if "--tol" in rest:
        i = rest.index("--tol")
        try:
            tol = float(rest[i + 1])
        except (IndexError, ValueError):
            print("error: --tol needs a float", file=sys.stderr)
            return 2
        del rest[i : i + 2]
    args = [a for a in rest if a != "--verbose"]
    if len(args) != 2:
        print(__doc__)
        return 2

    a = load(Path(args[0]))
    b = load(Path(args[1]))

    only_a = sorted(set(a) - set(b))
    only_b = sorted(set(b) - set(a))
    shared = sorted(set(a) & set(b))

    if shared:
        keys = list(a[shared[0]].keys())
    else:
        keys = []

    per_field: dict[str, dict] = {}
    divergent_ticks: list[int] = []
    for t in shared:
        ra, rb = a[t], b[t]
        for k in keys:
            va, vb = ra.get(k), rb.get(k)
            if isinstance(va, float) and isinstance(vb, float):
                ok = close(va, vb, tol)
            else:
                ok = va == vb
            if not ok:
                divergent_ticks.append(t)
                st = per_field.setdefault(
                    k, {"n_diffs": 0, "max_abs_delta": 0.0, "first_divergent_tick": t}
                )
                st["n_diffs"] += 1
                delta = abs(float(va) - float(vb)) if va is not None and vb is not None else float("inf")
                st["max_abs_delta"] = max(st["max_abs_delta"], delta)

    print(f"coverage: A={len(a)} B={len(b)} shared={len(shared)} "
          f"onlyA={len(only_a)} onlyB={len(only_b)}")
    if only_a[:5]:
        print(f"  first only-in-A ticks: {only_a[:5]}")
    if only_b[:5]:
        print(f"  first only-in-B ticks: {only_b[:5]}")
    if per_field:
        print(f"divergent fields ({len(per_field)}) across {len(set(divergent_ticks))} ticks:")
        for k in keys:
            if k in per_field:
                st = per_field[k]
                print(f"  {k}: n_diffs={st['n_diffs']} max_abs_delta={st['max_abs_delta']:.6g} "
                      f"first_tick={st['first_divergent_tick']}")
                if verbose:
                    for t in divergent_ticks[:20]:
                        if k in a[t] and k in b[t]:
                            print(f"    tick {t}: {a[t][k]} vs {b[t][k]}")
        verdict = "DIVERGENT"
    elif only_a or only_b:
        verdict = "COVERAGE_MISMATCH"
    else:
        verdict = "IDENTICAL"
    print(f"verdict: {verdict}")
    return 0 if verdict == "IDENTICAL" else 1


if __name__ == "__main__":
    sys.exit(main(sys.argv))
