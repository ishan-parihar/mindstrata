# Iteration 288 — A1 transgression feed: structural diagnosis + revival (PLAN_DC3 §4 i288)

**Status:** LANDED · **Doctrine:** probe-first; §4.3 ("a dead producer is a bug even
when tests pass") applied to an *emission site*. Two golden re-anchors + one snapshot
re-pin, all with mechanism.

## The debt

A1: the Transgression catalyst feed was dead at N=12 — zero `NormViolated` events in
every regime ever probed (i280 six-seed sweep; i286 verdict "Transgression feed is
dead"). Downstream consequences: Q2 dark-allergy had only absence-growth (never a real
pressure tick), the justice-line altitude never advanced, no (Symbolic, Event) Norm
claim ever projected, and the i286 Value/Norm synthesis gate could never fire — norm
proposals were confined to Identity codifications.

## The diagnosis (probe `i288_transgression_diet`, committed)

The plan's candidate causes (insufficient scarcity: calm/drought/famine/famine_long at
seeds 42/1/7, 20K) were all falsified first — hunger never crosses the desperation
band even under sustained famine windows (0.21–0.24 max), because production and
household pooling meet needs. The real cause is **structural, two layers down**:

1. **Theft is unreachable by construction.** `enforce_theft` fires only when
   `inaccessible_farm_with_grain_amount` / `inaccessible_well_with_water` match — but
   every stock the world generator seeds (`world_gen.rs`: farm, well, market) carries
   `AccessRight::Public`, so `can_access_resource` is true for everyone everywhere and
   the inaccessible-* lookups can never return `Some` at any N, in any regime, ever.
   The §19.5.E ownership layer (and the legal layer built on it) exists but the
   generator never creates restricted stocks.
2. **Violence records violations but never emits the event.** `norms_impl.rs` calls
   `check_violation(NO_VIOLENCE_NORM_ID, …)` on every Violence conflict (registry
   record + crime history) yet pushes no `SimEvent::NormViolated` — the ONLY event
   emitter in the codebase was the (unreachable) caught-theft path.

## The fix (one root cause: the missing emission at the live violation site)

`norms_impl.rs` now emits `SimEvent::NormViolated` right after the
`check_violation` call, on the public-by-nature basis the Iter-88 audit already
established for violence (no detection roll is drawn — the act is inherently public).
Magnitude 0.5 = spec-midpoint Transgression pressure (CALIBRATION-PENDING, rides the
A2 magnitude sweep). Per-subject expansion is the aggressor only; targets already get
their Threat catalyst from `ConflictOccurred`.

## Evidence (probe `i288_transgression_verdict`, committed)

Post-fix, NormViolated == violence count exactly, and the downstream chains light up:

| horizon (seed 42) | NormViolated | justice claims | Q2 mean |
|---|---|---|---|
| 500 | 1 | 1 | 0.072 |
| 1000 (golden) | 4 | 4 | 0.128 |
| 2000 | 5 | 5 | 0.226 |
| 4320 (golden crisis) | 11 | 11 | 0.389 |
| 10000 (snapshot) | 17 | 17 | 0.609 |
| 20000 (s42/s1/s7) | 25/18/16 | 25/18/16 | 0.73/0.69/0.73 |

**Norm proposals: seed 7@20K produced the first natural Value/Norm-synthesis proposal
(`proposed_norms=1`)** — the i286 gate widened to Value/Norm in vivo for the first
time, on real justice-claim clusters. Seeds 42/1 stay at 0 (quorum not yet reached) —
honest variance, not a gate failure.

## Re-anchors (measured / old / mechanism)

- **golden/riverford_minor@1000**: metric_hash 17850327103058637090 →
  11923864686455196885; event_hash 522152500760272504 → 6386238131629250189;
  agent_hash UNCHANGED, agent_count 12, total_grain 81.847 unchanged. Mechanism: 4
  NormViolated events enter the event stream (event_hash) and their +0.05 witnessed-
  unfairness / catalyst ticks shift the metric mix (metric_hash). No death, no wealth
  shift.
- **golden/collapse@4320**: metric_hash 17341641428938981610 → 16488504089400215145;
  event_hash 822834278629050683 → 10974323282720682888; agent_hash UNCHANGED,
  agent_count 12, total_grain 28.8882 unchanged. Mechanism: 11 NormViolated events,
  same signature.
- **long_horizon_surface_10000 snapshot**: `event_count` 140466 → 140483 (+17 —
  exactly the violence count at that horizon; no other field moved).
- Suite: 307/0/1; sim lib 241/241 (+2 pins); bench law 107 indexed / 0 violations.

## Pins (+2)

- `norm_violated_event_drives_transgression_catalyst_pathways` — catalyst mapping
  (subject/kind/magnitude/major) + real-pressure Q2 routing strictly exceeds
  same-state absence growth.
- `norm_violated_projects_justice_norm_claim` — (Symbolic, Event) Norm claim on the
  justice line, the stream i286's Value/Norm gate consumes.

## Remaining debt (recorded, honest)

- Transgression magnitude 0.5 stays spec-midpoint until the A2 observer-harness sweep
  (i289).
- The theft path remains topologically dead (restricted stocks never seeded). That is
  now a SEPARATE question: it needs world-gen ownership semantics (households own
  their stores, temple holds InstitutionMembers grain) — its own iteration, not
  smuggled into this one. The legal layer (courts, trials) rides on it.
- Norm-proposal quorum reachability now has a real chance; revisit with the i293
  festival-dense regime probe.
