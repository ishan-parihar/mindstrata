#!/usr/bin/env python3
"""metrics_diff.py — field-level delta between two JSONL metric streams (DC-1 task 2.14).

Each input is the JSONL emitted by `i267_metrics_jsonl` (one MetricsSnapshot
per line, serde field order stable). Compares aligned ticks and reports per-
field absolute deltas with mean/max, plus a threshold gate.

Usage:
  python3 scripts/metrics_diff.py <a.jsonl> <b.jsonl> [--threshold 0.01]
  python3 scripts/metrics_diff.py --help

Exit 0: always (report mode). With --strict, exit 1 if any field's max delta
exceeds threshold. Stderr carries the gate verdict.

Stdlib only — ponytail: one file, no deps.
"""

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_jsonl(path: Path) -> dict[int, dict]:
    rows: dict[int, dict] = {}
    with path.open() as f:
        for lineno, line in enumerate(f, 1):
            line = line.strip()
            if not line:
                continue
            try:
                obj = json.loads(line)
            except json.JSONDecodeError as e:
                print(f"{path}:{lineno}: invalid JSON: {e}", file=sys.stderr)
                sys.exit(2)
            tick = obj.get("tick")
            if tick is None:
                print(f"{path}:{lineno}: missing 'tick' field", file=sys.stderr)
                sys.exit(2)
            rows[int(tick)] = obj
    return rows


def main(argv: list[str]) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("a", help="first JSONL stream")
    ap.add_argument("b", help="second JSONL stream")
    ap.add_argument("--threshold", type=float, default=0.01, help="max-delta gate (default 0.01)")
    ap.add_argument("--strict", action="store_true", help="exit 1 if any field exceeds threshold")
    ap.add_argument("--fields", nargs="*", help="restrict to named fields (default: all numeric)")
    args = ap.parse_args(argv)

    a_path = Path(args.a)
    b_path = Path(args.b)
    if not a_path.exists() or not b_path.exists():
        print(f"missing input: {a_path} or {b_path}", file=sys.stderr)
        return 2

    a_rows = load_jsonl(a_path)
    b_rows = load_jsonl(b_path)
    common_ticks = sorted(set(a_rows) & set(b_rows))
    if not common_ticks:
        print("no overlapping ticks between inputs", file=sys.stderr)
        return 2
    if len(common_ticks) != len(a_rows) or len(common_ticks) != len(b_rows):
        print(
            f"warning: tick sets differ — a={len(a_rows)} b={len(b_rows)} overlap={len(common_ticks)} "
            f"(a_only={len(set(a_rows)-set(b_rows))} b_only={len(set(b_rows)-set(a_rows))})",
            file=sys.stderr,
        )

    # Collect numeric fields: those that are int/float in both streams at first tick.
    sample_tick = common_ticks[0]
    all_fields = sorted(set(a_rows[sample_tick]) & set(b_rows[sample_tick]) - {"tick"})
    if args.fields:
        fields = [f for f in args.fields if f in all_fields]
        unknown = set(args.fields) - set(fields)
        if unknown:
            print(f"warning: unknown fields ignored: {sorted(unknown)}", file=sys.stderr)
    else:
        fields = [f for f in all_fields if isinstance(a_rows[sample_tick][f], (int, float))]

    # Per-field deltas across ticks.
    deltas: dict[str, list[float]] = {f: [] for f in fields}
    for t in common_ticks:
        ar = a_rows[t]
        br = b_rows[t]
        for f in fields:
            av = ar.get(f)
            bv = br.get(f)
            if isinstance(av, (int, float)) and isinstance(bv, (int, float)):
                deltas[f].append(abs(float(av) - float(bv)))
            # non-numeric fields: skip (counts as 0 delta)

    # Report.
    print(f"compared {len(common_ticks)} ticks (overlap {common_ticks[0]}..{common_ticks[-1]})")
    print(f"{'field':<28} {'mean':>10} {'max':>10} {'p90':>10}  {'exceeds' if args.threshold else ''}")
    print("-" * 72)

    def p90(vals: list[float]) -> float:
        if not vals:
            return 0.0
        s = sorted(vals)
        idx = max(0, min(len(s) - 1, int(len(s) * 0.9 + 0.5) - 1))
        return s[idx]

    exceeded = []
    for f in fields:
        vals = deltas[f]
        if not vals:
            continue
        mean = sum(vals) / len(vals)
        mx = max(vals)
        q90 = p90(vals)
        flag = " *" if mx > args.threshold else ""
        if mx > args.threshold:
            exceeded.append(f)
        print(f"{f:<28} {mean:10.6f} {mx:10.6f} {q90:10.6f}{flag}")

    # Flush table before verdict so stdout/stderr ordering is stable.
    sys.stdout.flush()
    if exceeded:
        print(f"\nthreshold {args.threshold}: {len(exceeded)} field(s) exceed max delta: {', '.join(exceeded)}", file=sys.stderr)
    else:
        print(f"\nthreshold {args.threshold}: all fields within gate", file=sys.stderr)

    if args.strict and exceeded:
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
