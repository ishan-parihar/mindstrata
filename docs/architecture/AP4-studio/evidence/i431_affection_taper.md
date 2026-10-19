# i431 — the affection headroom taper (the kind-schedule write-side repair)

**Iteration:** i431 · **Date:** 2026-09-27 · **Status:** LANDED (behavioural)
**Change:** `relationship_v2.rs::record_positive` — the affection gain is now
headroom-tapered: `a += g·(1−a)` (computed in f64, quantized once — §5).
Trust, gratitude, intimacy, commitment, memory weights and `record_negative`
are untouched. One root cause, one site (§2.1).
**Chain:** i402 (kind-schedule branch fires 98–99% — §4.16(a) rules the read
bar unfixable on a point-mass input) → i429 (trajectory: the store is a
ratchet-to-ceiling) → i430 (census: the per-act writer owns 92% of positive
mass; no hidden writer) → this repair.

## 1. The repair's intended law

The untapered gain (+0.015×vol×mag/act) at the measured contact frequency
(p50 = 41 acts/day on contacted rows) pinned pairs at the ceiling in ~7 days
and drained the discriminating band [0.5, 0.95) to 1.9% by 18K ticks,
vacating every affection gate. The taper's equilibrium
`a* = 1 − D/(g_eff·k)` makes bond depth proportional to contact frequency —
the state variable that already measures what the constant gain guessed at
(§4.6's purest instance: the i376 pattern applied to the affection store).

## 2. Post-taper trajectory (probe `i429_affection_saturation`, rerun)

| arm | t=1440 p50 / band | t=10080 p50 / band | t=18720 p50 / band | ceiling share |
|---|---|---|---|---|
| calm/42 | 0.559 / 59.7% | 0.837 / 80.5% | 0.871 / 83.3% | 0–1.7% |
| calm/7 | 0.484 / 46.0% | 0.665 / ~74% | ~0.70 / ~74% | 0% |
| calm/11 | — | ~0.68 / ~76% | ~0.68 / ~80% | 0% |
| crisis/42 | ~0.55 / ~60% | 0.687 / 66.0% | 0.735 / 65.7% | 0% |

The point mass is gone: the band holds 66–83% of contacted pairs at 20K on
every arm, low-frequency pairs settle low (p10 0.06–0.51), and the crisis
world keeps the widest spread — the discrimination the crisis engine needs.

## 3. Blast radius and custody (the 17 suite failures, each dispositioned)

**Pre/post control** (`i431_tier_probe`, seed 42 @2000): quality p50
0.751 → 0.548 (−27%, the intended re-shape), **tiers STABLE** (0 Background;
6 Secondary; 6→7 Focal), interaction volume UP 4.8% (13 645 → 14 303) — no
i403-class tier cascade.

1. **marriage_formation_rate** — RE-CONTRACT (§4.4): the count differential
   saturates across the whole rate axis (sweep `i431_marriage_rate_sweep`:
   0.0002→0 marriages, 0.0004→5, 0.001→5, 0.1→5 — a cliff; the eligible-pair
   pool is the binding constraint). The parameter's live meaning is URGENCY:
   first-marriage 0.1@1 vs 0.0004@921. Contract: both arms marry, high
   strictly earlier, high count ≥ low. (i242 saturation precedent.)
2. **conception golden-window leg** — RE-ANCHOR: seed 42 → **44** (probe
   `i431_golden_seed_sweep`, seeds 1–70: 9 polluted {40,42,47,50,56,57,61,
   65,66}, 50 clean; 44 = the i363 anchor, minimal churn). Mechanism: the
   taper revives the marriage gate's partner discrimination, re-pacing early
   courtship.
3. **anger uniform leg** — RE-CONTRACT to conflict-attribution: member 6's
   emission is a Threaten/Insult-backed in-tick appraisal spike past the
   relative bar (probe `i431_anger_emission_cause`: conflict+threat at t=27,
   the goal persists after anger decays below the pin). The pin cannot hold
   anger inside the tick; the honest contract is that any emission is
   conflict/threat-attributed, never pin-driven. Attribution set =
   `ConflictOccurred` + `InteractionOccurred{Threaten|Insult}`.
4. **noospheric differential** — RE-CONTRACT (§4.4): the sustaining channel
   is quality-coupled; the 1.548× differential (i428) was the saturated
   store's artifact (high arm 0.2038 → 0.1340; the weak ecology's
   equilibrium held, 0.1316 → 0.1309). Tiers/interactions unchanged, so the
   channel is the quality read itself. Contract: strict high > low + floor
   re-derived at 0.12 (measured 0.1340). **Recorded debt:** the conviction-
   persistence magnitude fell 55% → 2.4% — whether the sustaining channel's
   own quality coupling re-sizes is its own behavioural iteration.
5. **attachment distress** — RE-ANCHOR (§4.2, probe `i431_attachment_family`):
   family {42: 15/48 = 31.2%, 7: 11/48 = 22.9%, 11: 18/46 = 39.1%}, means
   0.00249–0.00333 (all above the 0.002 liveness floor), maxes ≤0.057. The
   majority floor re-contracts to a ONE-QUARTER floor (the i388→i428→i431
   trajectory: 3/4 → 2/3 → 1/2 → 1/4 — each move measured, never widened).
6. **tenderness channel** — FAMILY RE-DISCOVERED (§4.8, sweep
   `i431_tenderness_sweep`, seeds 1–60): 41/60 qualify (lift ≥1.05, several
   at 1.2–1.8×); old member 46 INVERTS (0.727). Family {55, 46, 5} →
   **{55, 42, 5}**.
7. **meme host adoption** — RE-ANCHOR to the rising segment (§4.8, sweep
   `i431_meme_sweep`): the multiplier's total-host surface is non-monotone
   on the old seed 42 (0.5→40, 1.2→52, 2.0→37, 3.0→48); seed 7 carries the
   monotone rise (43→43→48→52). The i164/tenderness precedent.
8. **propaganda alignment** — RE-ANCHOR at the creation gate: the pin's 0.6
   high bar sat above every post-taper institution (councils park 0.552–0.594;
   census `i431_propaganda_census`); 0.55 is the implementation's own
   campaign-creation gate. Seed 0's council fell to 0.2147 with its founding
   campaign persisting (created@0) — the fallen-sponsor artifact class.
   Recorded limitation: all active campaigns are the founding pair; the
   §13.4 emergent re-arm producer needs a campaign to expire first.

**Goldens ×2 and snapshots ×7 re-baseline in the same commit** (`i276_golden_recompute`, values written programmatically): riverford_minor metric_hash 16379479100959651867 → 10217598687009991342 with **agent_count 12 → 13 — a founding-window birth on the canonical calm seed**, the same early-courtship re-pacing the conception pin's sweep measured (the pin's golden leg moved to seed 44 for the identical reason; both windows previously sat below the formation threshold on the saturated store); collapse metric_hash 10419835243253872331 → 15624657572515688469, agent_count 12 unchanged. The **reproduction fixture** (seed 5 @220K ×2 arms) exceeded every timeout window under the session's host load — its verdict is deferred to the gate run (recorded open, not assumed green).

## 4. What this changes about the engine

`affection` now carries information again: bond depth tracks contact
frequency, low-frequency pairs cool, and every affection gate (the kind
schedule's 0.70 bar when its read move lands, marriage formation, comfort
targeting, help contexts) reads a discriminating surface. The store had one
state ("contacted"); it now has the full band. The recorded debts: the
noospheric sustaining channel's quality coupling (above), and the kind
schedule + `social_cluster` read moves still pending on v1 (the i400/i402
rule: they move against the NEW surface in their own commit).
