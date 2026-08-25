---
name: wp-efgh-era2
description: "AP3 Era II work packages: person state (E), development pass (F), motive gating consumer (G1), pathology-stress coupling + inheritance (H1)."
type: Work-Package-Brief
plan_id: AP3
wave: II
owned_paths:
  - "E: crates/mindstrata-person/src/person/psyche.rs (+ DevelopmentField placement)"
  - "F: crates/mindstrata-sim/src/sim/pass_development.rs, sim/core.rs ⓦ"
  - "G1: crates/mindstrata-psych/src/psychology/motivation.rs read-side consumer hooks"
  - "H1: crates/mindstrata-sim/src/sim/population.rs ⓦ (founder draws), inherit path in person crate ⓦ with E"
forbidden_paths: ["crates/mindstrata-development/** (read-only)", "crates/mindstrata-social/**", "crates/mindstrata-tui/**"]
depends_on: ["Era I merged & frozen"]
integration_windows: ["F touches core.rs alone; H1 touches population.rs alone — never concurrently"]
---

# Era II briefs

## WP-E — Person state placement (iter ~273)

Add `development: DevelopmentField` to the psyche-side bundle. Founder init: uniform draws
within per-line stage bands (bands: needs/cognitive/emotional mostly 4..=10 amber-centered;
somatic 3..=7; spiritual/values wide 3..=11; exact bands in canon with CALIBRATION-PENDING
markers). Draws appended at END of populate stream — count and order documented here:

```
Founder draw order (frozen after first landing):
1..=19 individual line stages (one uniform draw each, band-clamped)
```

Midpoint-neutrality: nothing yet multiplies existing math — pure addition of inert state.
Gate: golden byte-identical EXCEPT agent_hash-neutral (new fields excluded from hash until
first consumer lands; document exclusion in snapshot provenance).

## WP-F — The daily pass (iter ~274–275)

`sim/pass_development.rs`: build day's catalyst buffer from observers (motive deltas,
appraisals, relational events per 03-substrate §3), call dynamics/pathology/lambda pure
functions, write annals trace events (quadrant-tagged). `core.rs` gains ONE hook line.
Pass consumes snapshots AFTER biology write-back loop. Gate: zero-catalyst worlds produce
zero field deltas (zero-at-zero); golden unchanged on calm seed.

## WP-G1 — First consumer: needs-gating (iter ~276–277)

Needs-line fulfillment feeds back into ONE existing signal: chronic Red-band under-
fulfillment raises stress-response floor through appraisal path (theory: needs@stage≤6
cells — survival-class need dominance). Zero-at-zero gated. Probe i276_red_floor measures
calm vs scarcity villages pre/post. Re-anchor any moved pin WITH mechanism comment.

## WP-H1 — Pathology→behavior + heredity (iter ~278–280)

- Dark-Addiction signature #1: obsolete-strategy persistence (hoarding after scarcity
  resolution) via action-selection bias term scaled by intensity.
- Golden-Allergy signature #1: witnessed-evaluation transition refusal.
- Genome extension: line predispositions inherited parent-midpoint + shrinkage + noise
  (same shape as personality inherit); midpoint = 1.0 multiplier discipline.
Probes: i278_hoard_persistence, i280_jonah_refusal, i280_inherit_correlation.

## Era II exit gate (all WPs)

Behavioral differentiation measurable across founder line-band configurations (i280
matrix probe: ≥2 distinct behavioral clusters by line profile at fixed seed family).
If NOT met → doctrine §3.2 dead-channel STOP applies.
