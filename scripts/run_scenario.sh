#!/usr/bin/env bash
# run_scenario.sh — run a scenario preset (RON or JSON) via the CLI.
# Usage: scripts/run_scenario.sh <preset.ron|preset.json> [extra cli args...]
#
# The loader sniffs format by content, so .json presets work through the
# same `mindstrata-cli scenario <path>` path as the RON battery.
set -euo pipefail

root="$(cd "$(dirname "$0")/.." && pwd)"
preset="${1:?usage: run_scenario.sh <preset.ron|preset.json> [extra args...]}"
shift

if [[ ! -f "$root/$preset" && ! -f "$preset" ]]; then
  echo "error: preset not found: $preset" >&2
  exit 1
fi

cd "$root"
exec cargo run --release -p mindstrata-cli -- scenario "$preset" "$@"
