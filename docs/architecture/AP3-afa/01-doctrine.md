---
name: ap3-doctrine
description: "AP3 governance: verification discipline, calibration honesty extensions, parallel-agent conflict protocol, iteration recipe. Read before any AP3 work."
type: Architecture-Plan-Reference
plan_id: AP3
---

# 01 — Doctrine

## §1 Verification discipline (inherited, restated for AP3 agents)

From AGENTS.md §3, binding without exception:

```
cargo fmt --all && cargo clippy --workspace --quiet \
  && cargo test -p mindstrata-tests --lib --release
```
before every commit. Behavioral iterations additionally run `scripts/gate --full` before
push. Full suite is release-mode. Snapshot drift goes through `cargo insta` review with
documented evidence; host has no cargo-insta binary — hand-promote `.snap.new` bodies
dropping the `assertion_line` header.

**AP3 additions:**

- D1 — *Observational-first landing*: every new subsystem lands inert (zero consumers)
  and proves golden byte-identity before any consumer wires in. One consumer per
  subsequent iteration, each gated zero-at-zero (no input ⇒ no output delta).
- D2 — *Contract freeze*: shared types get a `// FROZEN(AP3): <wave-id>` comment;
  edits after freeze require the wave owner's recorded sign-off in the WP brief.
- D3 — *Derivation citations*: test re-anchor comments and WP changelogs cite either a
  probe path + measured values or an ontology cell path (`stages/by-line/<line>/<NN>-<slug>/_index.md`).
- D4 — *Nothing silently deleted*: superseded calibration tables move to
  `docs/architecture/AP3-afa/archive/` with a pointer; git history is backup, not first resort.
- D5 — *Lens never place*: the sim's canonical axis is the neutral 17-stage ladder. The
  ray/density overlay exists only in chronicle rendering flavor (see 02-theory-map §2).

## §2 Hazards carried forward (AGENTS.md §5 — how they bite AFA specifically)

| Hazard | AFA exposure | Rule |
|---|---|---|
| Fixed-4 truncation (<5e-5 quantizes to 0) | stage press/velocity are far below resolution | all field math in f64 shadows; quantize once per tick into display/state Fixed |
| `mem::take` write-back discards writes inside pass loops | development pass reads biology snapshots | pass consumes snapshots taken AFTER write-back loop; never mutates biology arrays |
| Midpoint neutrality | genome-carried line predispositions multiply existing math | shape multipliers = 1.0 at population midpoint gene 0.5 |
| RNG stream discipline | founder line-stage draws | append at END of populate stream; document draw count/order in WP-E brief |
| Founder variance IS budget (H5) | line initial conditions could be bell-shaped "realistically" | uniform draws within stage bands; population-distribution bands are aspiration not assertion |

## §3 Calibration honesty extensions

AGENTS.md §4 rules all apply. Additions:

1. **Theory-derived ≠ evidence-free**: a pathology row or resonance weight enters code
   only with BOTH its ontology citation AND a measured pre/post probe. If the ontology
   attests it but the sim shows no measurable effect, that is a finding to record, not a
   constant to tune.
2. **Dead-channel stop**: if by end of Era II the fields produce no behavioral
   differentiation across line configurations, STOP the arc and record dead-channel debt.
   No constant-tuning to force liveness.
3. **Re-contract language required** where staged transitions legitimately invalidate old
   pins (faction formation becoming line-dependent rather than constant-driven). Guard the
   real invariant (liveness, positivity, decoupling bounds).
4. **Scale firewall honesty** (KosmOS `_Ontology/scale.md`): intra-holonic complexity
   (skill levels, trait sophistication) NEVER feeds developmental altitude. Separate
   accumulators, separate tests. A master craftsman is not thereby stage-12.
5. **Knife-edge debt recording**: any pin sitting on an unstable equilibrium (e.g., a
   pathology onset threshold flipping behavior) is recorded as systemic debt in the wave
   ledger, not flip-flopped.

## §4 Parallel-agent conflict protocol

The repo is shared by multiple concurrent agent sessions (proven: Iters 263–265 ran
concurrently on this clone). Protocol:

1. **Session start**: `git log --oneline -5 && git status --short`. If HEAD moved past
   your WP's base commit, re-read your owned files before editing; never force-fix a file
   another session may be mid-write (check mtimes; poll compile state instead).
2. **Ownership is whole-file**: the ledger (04-waves.md §3) assigns entire files. Two WPs
   needing the same file = a serialization edge in the wave graph, resolved by planning,
   not by coordination at runtime.
3. **Cross-cutting files** (`sim/mod.rs`, `core.rs`, `lib.rs` wiring lines) are touched only
   by their designated Integration WP within a window where no other WP is dispatched.
4. **Commit boundaries belong to the WP owner**; if your edit rides inside someone's
   uncommitted file, note it honestly in the commit message (AGENTS.md §7).
5. **Landing order inside a wave** is the ledger's dispatch order; later WPs rebase onto
   landed contracts, never onto working-tree state.

## §5 The iteration recipe (how any agent continues the arc)

Every AP3 iteration, regardless of era:

1. Pick ONE root cause from your WP's backlog (or the wave ledger's next item). Never batch.
2. Probe first: `crates/mindstrata-benches/examples/<iter>_*.rs` measuring actual behavior
   at actual horizon/seed. Record baseline numbers in the probe's header comment.
3. Implement against frozen contracts; new interfaces go through contract-freeze first.
4. Re-run probe → compare → full suite → fmt/clippy → commit → push.
5. Update your WP brief's changelog block (iteration #, one line, evidence refs).
6. Re-audit downstream equilibria; hunt for new gaps BEFORE declaring done.
7. Terse "continue" prompts mean: resume from last actionable state, no re-summarizing.

## §6 Governance adopted from KosmOS CONSTITUTION (translated)

| KosmOS rule | AP3 translation |
|---|---|
| R4 ontology amendments ratification-gated | ladder constants, resonance matrix defaults, pathology signatures live in `crates/mindstrata-development/src/canon.rs`, marked FROZEN; changes require user confirmation + citation of changed ontology cells |
| R6 tetra-quadrantic depth | every mechanism ships with its quadrant trace (UL/UR/LL/LR observability hooks) before Era III closes |
| R7 altitude claims cite the scale | any stage coordinate in snapshots/chronicles resolves to ladder slugs; no ad-hoc stage names |
| R12 claims cite derivation | see D3 |
| R5 nothing deleted | see D4 |
