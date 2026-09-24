# Iteration 286 — WP-H3 norm proposals from reconciled polarity clusters

**Status:** LANDED · **Doctrine:** probe-first; §4.3 (dead producer = bug) applied
to the channel design itself — the probe reshaped the gate before landing.

## What was wrong

WP-H3's second half ("norm proposals generated from reconciled polarity
clusters") had no implementation: the reconciliation pass synthesizes
Integrated claims (DC-2.1/i275), but nothing ever promoted a synthesis into
the norm registry. The registry's only writers were `default_norms()` (seeding)
and mods.

## What landed

`system_norm_proposal` (`systems/development.rs`), wired in `core.rs` after the
genesis call, reading the claim state the polarity pass just advanced:

1. **Band-III gate (i277 tetra-arising)**: the collective Safety bucket's
   deepest stage must be ≥ 4.0 — a registry norm is institutional-political
   content. All pinned horizons have max stage 1.0 exactly (probe i273), so
   every golden/snapshot window is inert **by construction** — zero re-anchors.
2. **Cluster census**: surviving Integrated syntheses per
   (referent, line, subtle) slot; engaged-cluster = distinct holders of any
   living claim on the slot (tension + synthesis + refutation).
3. **Majority quorum**: cluster > n/2 — a village-wide prescription.
4. **Registry append**: one norm per slot, ever (slot-encoded name = dedup +
   restore-safe); strength = consensus breadth (≤ 0.6), grows only through
   the EXISTING §12.5 ritual-reinforcement channel.

## The in-vivo verdict (the probe reshaped the design)

`i286_norm_proposal_census` (deleted after use — retired again at plan-rust-craft C1,
whose all-targets clippy gate the dead probe cannot pass; the numbers below were
produced via `cargo run`, which does not run clippy lints) + `i286_diag` (deleted
after use) + `i286_norm_proposal_vivo`:

- Value/Norm syntheses are **structurally unreachable in vivo**:
  reconciliation needs two different subtle claims on one slot, and the only
  in-vivo collider is (Event, cognitive) — Threat-Fact × Grief/major-Identity
  → **Identity** syntheses. Bond (Value/attachment) and Transgression
  (Norm/justice) are mono-claim per agent, and i280 proved NormViolated = 0
  at N=12 (zero Transgression claims at all).
- First synthesis @ ≤500–1000 ticks (Refuted-census growth; syntheses are
  consumed intra-pass — census-time Integrated = 0, Refuted = 16 @ 20K).
- Tension-cluster quorum at the contested slot: 8–9/12 agents — the consensus
  gate is genuinely reachable.

**Fix:** the proposal gate accepts **Identity syntheses** (codifying "we are
the kind of people who…" — the panic-crystallization arc the wave brief
describes) with Value/Norm still accepted for when a Transgression feed exists
(i280 recorded debt). `i286_norm_proposal_vivo` then measured, natural stages:

```
natural  @ 50000: proposed_norms=1 (registry 5→6)   ← fires in vivo
forced   @ all:  proposed_norms=0                   ← stage forcing ≠ synthesis
natural  @ 5K/20K: 0                                ← band-III gate holds
```

## Pins (3 new; sim lib 239/239)

- `norm_proposal_fires_on_majority_synthesis_past_band_gate` — 7/12 consensus,
  stage 4.5 → exactly one norm, consensus-scaled strength, idempotent dedup.
- `norm_proposal_is_band_gated_below_stage_four` — 3.9 blocks identical consensus.
- `norm_proposal_skips_fact_syntheses` — Fact = lore, not law (8/12 + open band).

## Calibration honesty

- No re-anchors: full gate green on first run (307/0/1, clippy 0).
- The `SubtleClaim::Identity` acceptance is a **contract statement, not a
  re-pin**: the wave brief's "norm proposals from reconciled polarity
  clusters" is satisfied by the only cluster that reconciles in vivo. When a
  Transgression feed lands (i280 debt), Value/Norm gate widens without
  touching the quorum/band machinery.
- Proposal strength 0.6 cap is CALIBRATION-PENDING(AP3) (first estimate).

## Remaining

- WP-H3 is now **closed** (ritual generation i285 + norm proposals i286).
- Era V items (WP-K transition traces, WP-L chronicle lens) remain the next
  unbuilt wave surfaces; Era IV probe gap (`i300_institution_shift`
  end-to-end) remains recorded.
