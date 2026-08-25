---
name: ap3-waves
description: "AP3 wave decomposition: work packages, file-ownership ledger, dependency graph, iteration budget across 5 eras (~50–100 iterations). The authoritative dispatch plan."
type: Architecture-Plan-Reference
plan_id: AP3
---

# 04 — Waves & Work Packages

## §1 Era structure & iteration budget

| Era | Theme | Iterations | WPs | Exit gate |
|---|---|---|---|---|
| **I** Foundation | vendor/codegen + inert substrate crate | 266–272 | 0A, 0B, A, B, C, D | golden byte-identical; unit pins green; zero consumers |
| **II** Person wiring | state placement, inheritance, first consumers | 273–280 | E, F, G1, H1 | first behavioral differentiation measurable; zero-at-zero gates pass |
| **III** Content emergence | three-realm grammar, meme/polarity generation | 281–292 | G2, H2, H3 | seed-disjoint cultures in 20K-tick probe; old roster retired |
| **IV** Collective holon | village stage_lines, institutional coupling | 293–305 | I, J | collective stages move with documented equilibria; institution behavior line-coupled |
| **V** Harvest | observability, chronicle flavor, long-horizon studies | 306–320 then open-ended | K, L, loop recipe §5 | tetra-arising traces complete; civilization differentiation study |

Eras II–V estimates are budgets not promises; the recipe in doctrine §5 generates
iterations until exit gates hold. 50–100 total is realistic if Era III–IV probes surface
calibration debt (they will).

## §2 Dependency graph

```
0A ──► 0B ──► A ──┬─► B ──► D ──┐
                  └─► C ────────┤
                                ├─► F ─► G1 ─► H1 ─► │
                          E ─────┘                    │
                                                      ▼
                              G2 ─► H2 ─► H3 ─► I ─► J ─► K ─► L ─► (loop)
```

Contract-freeze points: after A (`stage.rs`, `line.rs` types), after B/D/C merge
(`field.rs`, `dynamics.rs`, `pathology.rs`, `lambda.rs` APIs), after E (person-side
placement), before G2 fanout (content trait surfaces).

## §3 FILE OWNERSHIP LEDGER (authoritative — whole-file scopes)

Legend: ● owned (may create/modify) ○ forbidden ⓦ integration-window only.

| Paths | 0A | 0B | A–D | E | F | G* | H* | I | J | K | L |
|---|---|---|---|---|---|---|---|---|---|---|---|
| `docs/architecture/AP3-afa/vendor/**` | ● | | | | | r | r | r | r | r | r |
| `crates/mindstrata-development/**` | | ● | ● (split per brief) | | | | | | | | |
| `scripts/afa_codegen.py` | ● | ● | | | | | | | | | |
| `crates/mindstrata-person/src/person/psyche.rs` (+bundle field) | | | | ● | | | | | | | |
| `crates/mindstrata-sim/src/sim/pass_development.rs` | | | | | ● | | | | | | |
| `crates/mindstrata-sim/src/sim/core.rs` | | | | | ⓦ | | | ⓦ | | | |
| `crates/mindstrata-sim/src/sim/mod.rs` (bundle fields) | | | | ⓦ | | | | ⓦ | | | |
| `crates/mindstrata-sim/src/sim/population.rs` | | | | ⓦ(draws) | | | | | | | |
| `crates/mindstrata-social/src/culture/content_gen.rs` | | | | | | ● | | | | | |
| `crates/mindstrata-social/src/culture/meme.rs` | | | | | | ⓦ | | | | | |
| belief/journal/gossip polarity hooks (psych+social) | | | | | | | ● | | | | |
| village collective field (`sim/collective.rs` NEW) | | | | | | | | ● | | | |
| institutions coupling (`institutions/*` read-side) | | | | | | | | | ● | | |
| TUI charts lane (`mindstrata-tui/render.rs`) | | | | | | | | | | ● | |
| chronicle flavor lens (`render/session.rs`) | | | | | | | | | | | ● |
| probes `benches/examples/<iter>_*.rs` | own iteration's probe file — unique names prevent collision |

ⓦ = single integration window: WP holding ⓦ dispatches alone that day; no other WP in
flight (doctrine §4.3).

**Parallel session coordination**: the OTHER active session owns TUI/annals polish work
(Iters 264–265 pattern). Before any WP touching `mindstrata-tui/**` or annals rendering,
confirm with operator / check `git log` for fresh TUI commits; re-plan ownership if hot.

## §4 Dispatch rules

1. Wave order is topological per §2. Within a wave, WPs with disjoint ledger columns may
   run concurrently as separate agents.
2. Each WP = one agent session at a time; the agent reads PLAN.md → its brief → cited
   theory sections → owned code.
3. A WP that finishes early picks up the next serial item in ITS column, not another
   column's files.
4. Merge conflicts are a planning bug: record them in the wave ledger and fix the ledger,
   not just the conflict.
5. Every landed WP updates its brief changelog + this ledger's status column.

## §5 Open-ended deepening loops (Era V+ recipe)

After L, the arc continues by looping: pick weakest liveness pin or thinnest ontology
attestation → probe → deepen mechanism or extend canon with newly attested cells → suite →
land. Candidates backlog lives in [waves/WP-L-loop.md](waves/WP-L-loop.md).

## Status

| WP | Status | Iterations consumed |
|---|---|---|
| all | pending | — |
