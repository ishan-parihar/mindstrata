#!/usr/bin/env bash
# new_probe.sh — emit a compiling probe skeleton per IC-4 conventions.
# Usage: scripts/new_probe.sh <iter> <slug>
#   e.g. scripts/new_probe.sh 266 needs_gate_delta
# Output: crates/mindstrata-benches/examples/i<iter>_<slug>.rs
set -euo pipefail

iter="${1:?usage: new_probe.sh <iter> <slug>}"
slug="${2:?usage: new_probe.sh <iter> <slug>}"
root="$(cd "$(dirname "$0")/.." && pwd)"
target="$root/crates/mindstrata-benches/examples/i${iter}_${slug}.rs"

if [[ ! "$iter" =~ ^[0-9]+$ ]] || [[ ! "$slug" =~ ^[a-z0-9_]+$ ]]; then
  echo "error: iter must be numeric; slug must be lowercase snake_case" >&2
  exit 1
fi
if [[ -e "$target" ]]; then
  echo "error: $target already exists" >&2
  exit 1
fi

cat > "$target" <<EOF
//! Probe: TODO one sentence — what behavior is measured?
//!
//! Evidence context: TODO which iteration/audit question this answers.
//! Horizon/seed: TODO ticks at seed(s) TODO — MUST match the assertion being calibrated.
//!
//! Run: cargo run --release -p mindstrata-benches --example i${iter}_${slug}

use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    let config = SimConfig {
        seed: 42,
        max_ticks: 10_000,
        ..SimConfig::default()
    };
    let ticks = config.max_ticks;
    let mut sim = Simulation::new(config); // adjust to the scenario under question

    for t in 0..ticks {
        sim.tick();
        if t % 1000 == 0 {
            // Sample ONLY observable output; print stable key=value rows.
            println!("tick={} sample=TODO", t);
        }
    }
}
EOF

echo "wrote $target"
echo "verify with: cargo check -p mindstrata-benches --example i${iter}_${slug}"
