# IC-1 — Catalyst Vocabulary

```yaml
provider: SIM (observer harness) + STORY (vocabulary co-author)
consumer: STORY (development pass), later DESIGN telemetry
frozen_at: commit 9d31f98 (vocabulary types in crates/mindstrata-development/src/catalyst.rs; observer harness draft f11eefb in crates/mindstrata-sim/src/sim/catalyst_observers.rs)
version: 1.0.0
status: RATIFIED-P0 (PROD sign-off: architect session, 2026-08-26, per AP4 interlock map; operator counter-sign optional — no workflow-mode change involved)
change_orders: []
```

## Purpose

The typed event vocabulary by which mechanical simulation state becomes developmental
signal — the ONLY channel from SIM mechanics into STORY's attractor fields. Keeps passes
read-only and territories clean: SIM observes, never mutates to feed.

## Surface (FROZEN v1.0.0 — compiles as `mindstrata_development::catalyst`)

```rust
pub struct CatalystEvent {
    pub tick: Tick,                       // core clock tick of the producing event
    pub subject: AgentId,                 // affected agent
    pub kind: CatalystKind,               // producer-side classification
    pub drive: Drive,                     // resolved via kind_drive_map (frozen projection)
    pub line_tags: Vec<(LineId, f64)>,    // weight per resonant line, clamped [0,1]
    pub magnitude: f64,                   // f64 shadow; quantize once at uptake
}
pub enum Drive { Agency, Communion, Eros, Agape }
pub enum CatalystKind { Grief, Bond, Threat, Transgression }
pub const fn kind_drive_map(kind: CatalystKind) -> Drive;
```

Frozen kind→drive projection:
- Grief → Agape (loss metabolized as release/care beyond the self)
- Bond → Communion (attachment formation is belonging itself)
- Threat → Agency (dominance struggle engages effective power)
- Transgression → Agency (self-assertion against the collective;
  guilt/reparation routes back through Agape in the pathology operator's golden quadrant)

RealmRefs from the AP3 sketch is DEFERRED to a minor version: gross entity ids are
recoverable from the producing event window and no consumer needs them yet.

Producers v1.0.0 (SIM observer harness drafts): death (Grief via guarded widow
heuristic — co-resident kin targeting needs an inert side-buffer in the deaths pass,
tracked for task 3.x wiring), marriage + birth (Bond), conflict + feud (Threat),
norm violation (Transgression). Producer extensions queued behind wiring: appraisal
motive-pressure deltas, gossip meme uptake, ritual/festival participation.
NOT producers: skill/craft mastery (scale firewall — witnessed-evaluation context
only, no direct stage credit).

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
| 1.0.0 | FROZEN at 9d31f98: typed vocabulary compiles (38/38 development-crate tests); kind/drive split reconciles observer draft with AP3 drive axis; RealmRefs deferred; widow-heuristic ambiguity documented | PROD sign-off recorded 2026-08-26 |
