# IC-1 — Catalyst Vocabulary

```yaml
provider: SIM (observer harness) + STORY (vocabulary co-author)
consumer: STORY (development pass), later DESIGN telemetry
frozen_at: PENDING (co-authoring window: SIM phase 5 ↔ STORY phase 5, PROD-scheduled)
version: 0.1.0-draft
change_orders: []
```

## Purpose

The typed event vocabulary by which mechanical simulation state becomes developmental
signal — the ONLY channel from SIM mechanics into STORY's attractor fields. Keeps passes
read-only and territories clean: SIM observes, never mutates to feed.

## Surface (draft from AP3 03-substrate §3; frozen at the co-authoring window)

```rust
pub struct CatalystEvent {
    pub line_tags: SmallVec<[(LineId, f64); 4]>, // weight per resonant line
    pub drive: Drive,                            // Agency|Communion|Eros|Agape
    pub realm_refs: RealmRefs,                   // gross entity ids cited
    pub magnitude: f64,                          // f64 shadow; quantize once at uptake
}
pub enum Drive { Agency, Communion, Eros, Agape }
```

Producers (SIM observer harness): motive pressure deltas, appraisal outputs,
courtship/marriage/birth/death events, norm violations + verdicts, gossip meme uptake,
ritual/festival participation. NOT producers: skill/craft mastery (scale firewall —
witnessed-evaluation context only, no direct stage credit).

## Guarantees

- Observers are side-effect-free reads of post-write-back snapshots.
- Zero-catalyst days produce zero field deltas (zero-at-zero law).

## Obligations

- STORY consumes via pure functions only (AP3 substrate determinism contracts).
- Vocabulary additions are minor versions; semantic changes are change-orders.

## Tests guarding this contract

- Observer purity pins (SIM); catalyst→field identity-at-neutral pins (STORY unit);
  differentiation matrix probe (Era II exit).

## Changelog

| ver | change | approved |
|---|---|---|
| 0.1.0-draft | initial from AP3 substrate sketch | co-authoring pending |
