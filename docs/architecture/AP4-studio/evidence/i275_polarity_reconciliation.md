# Iteration 275 — WP-H2 polarity reconciliation reachability (probe evidence)

**Status:** PROBE LANDED (pre-implementation evidence) · **Doctrine §2:** probe before touching.

## Probe: `i275_polarity_reconciliation.rs` (N=12, seed 42, 20K)

### Part 1 — natural diet (the dead-producer verdict, now measured)

| Surface | Value |
|---|---|
| Total claims accumulated | 1,292 |
| Undiscovered | 1,292 |
| ActiveTension | **0** |
| Integrated | **0** |

**Verdict confirmed:** the reconciliation pass (live since DC-2.1 in
`system_polarity_claim_emit`) never fires in natural runs. Root cause is not
the pass — it is the *projection*: `project_catalyst` maps each CatalystKind
to ONE fixed (domain, referent, claim, line) quartet:

| Kind | Quartet |
|---|---|
| Threat | Material / Event / Fact / cognitive |
| Bond | Relational / Institution / Value / attachment |
| Transgression | Symbolic / Event / Norm / justice |
| Grief | Material / Event / Identity / cognitive |

`advance_to_active_tension` requires a sibling on the SAME (referent, line)
with a DIFFERENT claim or domain. The only in-vivo collision is
Threat+Grief on (Event, cognitive) — and Grief is mortality-blocked at N=12
(i272: first natural death ≈ 2.3M ticks). Downstream consumers that read
ActiveTension (action bias `actions/mod.rs:845`, gossip narrative in
`norms_impl.rs:966`) are therefore reading a permanently-zero channel:
**the tension/reconciliation graph is a dead producer even though every
component is wired.**

### Part 2 — forced-collision upper bound (mechanism viability)

Injecting GriefStruck windows (20 rounds × 12 events, 2K ticks) so Threat and
Grief claims coexist on (Event, cognitive):

| Surface | Value |
|---|---|
| Undiscovered | 20 |
| ActiveTension | **524** |
| Integrated | 0 |

Mechanism verdict: the tension gate *does* open under collision-rich diets
(524/544 claims promoted). Integrated remains 0 at this horizon —
`reconcile_subtle` requires BOTH claims ActiveTension on the same
(domain, referent, line) with different subtle claims; after synthesis
removes the pair, the next collision starts fresh, so Integrated accrues on
slower multiplicative schedule. Not a bug; the observable chain is
collision → tension (fast) → integration (slow).

## What the fix must do (i275 implementation design, probe-gated)

The projection is a DC-1 v1 pin (one quartet per kind). The fix must NOT
re-pin magnitudes; it must make *distinct agents' observations of the same
world* land as distinguishable claims. The minimal root-cause candidate:
**referent grounding** — project the claim's GrossReferent from the actual
event content (which institution, which event kind) instead of the constant
per-kind referent, so that (referent, line) collisions arise from the world's
event variety, not from a single hardcoded pairing. Collision rate then
tracks event diversity — the substrate's intent ("every subtle item cites ≥1
gross entity").

Zero-blast constraint: golden horizons (≤2000) — claims exist (1292 at 20K
implies ~129 at 2K) but tension stays 0 there under the natural diet unless
the grounding change creates collisions at the golden horizon too. The
2K-window tension count must be probed before/after; if grounding creates
tension inside golden windows, the action-bias coefficient (0.10 × count)
will shift utility — re-anchor with mechanism or gate the bias.

## Measurement trail

- Probe committed as `crates/mindstrata-benches/examples/i275_polarity_reconciliation.rs`.
- Natural-diet numbers above are the "old band" for §4.2 re-anchoring once
  the fix lands: expect undiscovered to fall, tension to rise from 0.
- The dead-producer finding supersedes PLAN_DC2 §6.3's i275 framing
  ("reconcile_claims has zero call sites" was WRONG — the pass is wired; the
  projection starves it). Corrected here per §4 calibration honesty.
