---
name: ap3-sim-inventory
description: "AP3 inventory of existing mindstrata code the arc touches: symbols, files, hazards. Verified 2026-08-25 @ HEAD 3705dc9."
type: Reference-Index
plan_id: AP3
verified_at: "3705dc9 (Iteration 265)"
reconciled_commit: 8a4ac31
---

# refs — Sim Inventory (what exists today)

> **Citation re-check (i386, 2026-09-23, `cd4ac0a`):** the *paths* below survived the crate
> ladder and the Arc-D pass extraction — every crate/file named here exists. The
> `symbol (:line)` anchors were verified at i265 and drift with ordinary edits, so treat
> them as pointers into the named file, not as facts. Two structural corrections are
> folded in below (the pass count and the development pass's home). For what the engine
> *does* today, `docs/ENGINE_STATUS.md` is authoritative and this index defers to it.

## Motivation (ray/needs signal source)

- `crates/mindstrata-psych/src/psychology/motivation.rs`
  - `pub enum MotiveCategory` (:183) — 19 categories: Hunger Thirst Sleep Warmth Health
    Safety Attachment Belonging Esteem Autonomy Competence Meaning Certainty Novelty Play
    Care Romance Justice Recognition
  - `pub struct MotivationState` (:108) — per-category `MotiveNeed` with rate/cap defaults

## Person / psyche

- `crates/mindstrata-person/src/person/psyche.rs`
  - `fn plasticity_rate(s: &PlasticitySignals, rate_const: f64) -> Fixed` (:245)
  - temperament plasticity apply (:320), core-trait (:397) — the push/pull attractor
    precedent AFA generalizes
- `crates/mindstrata-person/src/biology/genome.rs` — `TraitPredispositions` (:30),
  `Genome` (:145); inheritance midpoint+shrinkage+noise pattern to extend for lines

## Appraisal (drive-weighted catalyst source)

- `crates/mindstrata-psych/src/appraisal.rs` — `pub struct Appraisal` (:29)

## Culture / content layer (Era III targets)

- `crates/mindstrata-social/src/culture/meme.rs` — `MemeContent` enum (:39: Accusation,
  Praise, Theological, Political, Moral, Conspiracy, …), `Meme` struct (:91:
  emotional_charge, identity_relevance, moral_charge …)
- `crates/mindstrata-sim/src/sim/population.rs` — `fn seed_initial_memes` (:107) — the
  fixed roster WP-G2 replaces

## Institutions (Era IV coupling surface)

- `crates/mindstrata-institutions/src/{institutions,governance? no—}…`: institutions.rs,
  legal.rs, diplomacy.rs, military.rs, theology.rs, schools.rs, norms.rs, factions.rs

## Sim orchestration

- `crates/mindstrata-sim/src/sim/mod.rs` — Simulation struct, AgentBundle
- `crates/mindstrata-sim/src/sim/core.rs` — tick pipeline order **and the per-pass
  profiler**; five verbatim passes remain in `sim/` (`pass_{action,ecology,scenario,
  social,weather}.rs`) and the rest were extracted into `crates/mindstrata-sim/src/
  systems/` (`biology, cognitive, appraisal, decay, health, development, genesis,
  trade_diffusion, institutions_multiplier`) — there was never a `pass_development.rs`
- `AgentBundle` lives in sim/mod.rs; psyche-side DevelopmentField placement per WP-E

## Tests & probes conventions

- Integration tests: `crates/mindstrata-tests/src/integration_tests/{biology,psychology,
  social,culture,governance,economy,legal,infra}.rs`
- Probes: `crates/mindstrata-benches/examples/<iter>_*.rs`, release mode only

## Known hazards touching this work — see 01-doctrine §2 table.

## Parallel session watchlist

Files hot with the OTHER active session recently: TUI render/session (Iters 264–265),
annals provenance. Re-check `git log --oneline -5` before any TUI/annals WP.
