---
name: ap4-phase-template
description: "AP4 phase brief template — every departmental phase is instantiated from this."
type: Template
plan_id: AP4
---

# Phase Brief — <DEPT>-<NN>: <title>

```yaml
department:
cycle: DC-N
phase: NN            # zero-padded, ladder order
goal:                # one sentence, observable outcome
baseline_evidence:   # probe/bench numbers or artifact BEFORE work — phase may not run without this
territory_files:     # subset of charter territory; must match ledger
interlocks_touched: []  # IC ids, or none
gates:               # test names / thresholds / artifacts required to exit
est_size: S|M|L      # S=≤1 session, M=few sessions, L=flag to PROD for splitting
```

## Execution note (fill during work)

- probe(s) written: `crates/mindstrata-benches/examples/<iter>_<slug>.rs`
- mechanism comment refs (what moved and why)
- contract deviations: none | change-order <id>

## Evidence (fill at exit)

| gate | result |
|---|---|

## Changelog

| iter | note |
|---|---|
