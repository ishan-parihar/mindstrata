# Iteration 356 — `Idle` revived: the last dead action, and the `Play` dead motive behind it

**Status:** LANDED (behavioural) · **Root cause owned:** `Idle` was the last action
that won **0 arbitrations in 96 000 agent-ticks** (i346/i351) and `MotiveCategory::Play`
was a dead motive — it competed in `update_dominant` with **zero relief sites anywhere**,
so its deficit sat pinned at the cap and its pressure at 0.20 for every agent. Same
shape as the A8 `Wander` closure (i351), applied to the last remaining pair.

## The anatomy (probe-first, `i356_idle_driver`)

`Idle`'s only relief is 0.05 fatigue/tick — the **same effective rate** as `Rest`
(0.4 over 8 ticks), but with no energy recovery (`bonus_energy_recovery`), no motive
write-back (the `core::tick` completion match had no `Idle` arm) and a
behavioural-inhibition trait (`is_withdrawal`, `is_disobedient`). `Rest` strictly
dominated it.

In parallel, leg A measured `Play` pressure at **mean 0.2019 / p50 0.2006 (N=12)**,
**mean 0.2046 / p50 0.2006 (N=48)** — the deficit is pinned at its `cap = 1.0` and the
pressure is exactly `1.0 × urgency 0.2` for everyone. A **saturated, uniform** signal:
the dead-motive signature (§4.3).

## The design act

`MotiveCategory::Play` (recreation) is `Idle`'s natural expression, exactly as
`Novelty` is `Wander`'s. Three parts, mirroring i351:

1. **Driver term** — `play_pressure × IDLE_PLAY_COEF`, gated on the same
   `needs_quiet` predicate (`max(hunger, thirst, fatigue) < 0.5`). Extracted as the
   pure helper `idle_play_driver(play_pressure, needs_quiet)` so the utility path and
   the census's pre-driver correction read **one** expression (the i351 lesson: an
   in-vivo/census disagreement is how a mis-sized constant surfaces, and that needs a
   single source of truth).
2. **Urgency wiring** — `MotiveCategory::Play => action.kind == ActionKind::Idle` in
   the §8.1.5 dominant-need match (the `Novelty → Wander` pattern).
3. **Relief write-back** — `play −0.03`, `autonomy −0.004` per tick of `Idle` in the
   `core::tick` completion match. `Play` had **no** relief outlet before this, so the
   write-back is what de-saturates the motive into a loop rather than a ratchet.

## Sizing (§4.2)

The driver term is `0.20 × coef` (play saturates), while `Novelty` runs ~0.30 at
settlement, so **`0.20 × 3.0 = 0.30 × 2.0 = 0.60`** — coefficient 3.0 gives the same
driver magnitude the A8 Wander closure shipped.

The binding evidence is the quiet-window sweep (`i356_idle_driver` leg B). The
`winner − Idle` gap is **sharp**: p50 0.77–0.96 across seeds and a knee between 0.4
and 0.6 (p10 0.44–0.71, p25 0.47–0.76). At **c = 2.0** Idle stays dead on three of
live seeds; **c = 3.0 is the smallest coefficient live on all five** measured
(42/7/11/46 + N=48). Choosing 2.0 would have been the §4.1 lucky-seed pin.

## In-vivo liveness (`i356_idle_driver` leg C — post-driver share)

Nine seed/N runs at 20 000 ticks, census-enabled:

| seed | N | Idle share | Wander share |
|---|---|---|---|
| 42 | 12 | 0.47% | 5.93% |
| 7 | 12 | **0.01%** | 3.38% |
| 11 | 12 | 1.76% | 2.06% |
| 46 | 12 | 2.04% | 5.92% |
| 42 | 48 | 0.43% | 5.25% |
| 5 | 12 | 0.21% | 3.29% |
| 23 | 12 | 3.67% | 3.85% |
| 99 | 12 | 0.47% | 9.24% |
| 7 | 48 | 0.84% | 3.99% |

**Idle is live on every run (was 0 on all).** The share spans 0.01%–3.67%; the
quiet-window wall varies ~3× across seeds, so the pin is **positivity over the family**
(which holds on all nine), not a magnitude band — a tight band there would be the §4.1
lucky-seed pin the doctrine forbids. Idle landing *rarer* than Wander (3–9%) is
semantically right.

Play de-saturated as designed: pressure mean **0.09–0.20** (p50 0.17–0.20) versus the
pre-driver pinned 0.20.

## Provisioning check (leg D)

The driver displaces some `Work`, so the 10K snapshot's `total_grain` moved. A stock is
phase-sensitive; hunger is not. Long-horizon runs confirm no starvation:

- seed 42, 10K: agents 12, hunger **0.049**, grain 1.41, health 0.796
- seed 42, 50K: agents **12 → 14**, hunger **0.013**, health 0.789
- seed 7, 50K: agents **12 → 16**, hunger **0.021**, health 0.737

## The sweep, classified

**No behavioural pin broke.** The only failures were the two goldens and five
snapshots, all bounded and in the driver's direction:

- Both goldens regenerated — **`agent_count 12` preserved on both scenarios** (the
  crisis-mortality check intact); riverford grain 83.39 → 83.13, collapse 1.1749 → 1.1653.
- Five snapshots reviewed then accepted: 2K `avg_relationship_trust` 0.6994 → 0.6892,
  `avg_stress` 0.2902 → 0.3011, `avg_health` 0.7861 → 0.8055; 10K `avg_stress`
  0.3762 → 0.3635, `avg_health` 0.7934 → 0.7847, `event_count` 125 669 → 124 880,
  `total_grain` 3.32 → 1.20 (stock phase — provisioning verified above),
  relationship-stage redistribution (Ally 69 → 66, CloseFriend 2 → 4, Unnoticed 11 → 13);
  institution legitimacy 0.5853 → 0.5758. No saturation anywhere.

## Runnable checks

- `idle_play_driver_is_gated_saturating_and_zero_at_zero` — the pure term: open gate →
  `pressure × coef` (competitive), closed gate → 0, zero pressure → 0 (identity).
- `idle_is_reached_once_the_recreation_driver_is_live` — sim-level positivity on the
  `Idle` agent-tick count **and** the dead-motive signature gone (`Play` deficit no
  longer at its cap for *every* agent).
- The stale census commentary (`wander_carries_no_need_relief_at_all` claimed Idle
  "never wins either") is corrected in place; the definition is deliberately unchanged.

## Instrumentation

`decision_census` gained the `Idle` analogue of the i351 quiet-window sizing: a
`(pre-driver winner − Idle gap, play_pressure)` pair column, `quiet_idle_sweep`, and
`quiet_idle_gaps`. `DecisionContext` gained `play_pressure`. All opt-in / inert when
disabled; `record_utility_sample` now takes the pre-driver gap separately from the
realized one so a driven winner can't produce a negative "gap".

## Verification

`cargo test -p mindstrata-sim --lib --release` **298/298** (+2 pins), `-p mindstrata-tests
--lib --release` **310 passed / 0 failed / 1 ignored**, golden replay byte-identical after
regeneration, `cargo clippy --workspace` clean, 186-probe bench law clean, `scripts/gate
--full` **GATE GREEN**.

## Ledger

1. **`Idle` (A9) CLOSED** — the last dead action is live; the `Play` dead motive has a
   relief outlet.
2. The share is **seed-variable** (0.01%–3.67%) because the quiet-window wall varies
   ~3×; recorded as a measured property, not debt — only revisit if the displacement
   shows up in provisioning (it did not).
3. The §4.2 band language for a *driver* remains loose (Wander's 5.39% 20K share was
   already recorded above its guessed band); both drivers are positivity-pinned by
   construction.
