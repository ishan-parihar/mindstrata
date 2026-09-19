# Iteration 313 — the injury channel was dead state; wired end to end

**Status:** LANDED (behavioural) · **Root cause owned:** i312's queued item —
two of the five derived-health penalty channels (`pain`, `shock`) contributed
exactly 0.0000. Both trace to a single upstream producer, and it was never
written.

## The finding

`EmbodiedState.injury` (biology/mod.rs:130) is initialized to `ZERO` and had
**zero write sites anywhere in the workspace** — verified by exhaustive grep.
The violence path instead calls `health::apply_injury`, which damages `health`
and `energy` and rolls wound infection but never records the injury the field is
named for.

Probe `i313_injury_channel` (12 seeds, every tick):

| context | max injury | max pain | max shock | min blood-vol | violence events |
|---|---|---|---|---|---|
| pestilence @4 320 | **0.00000** | **0.00000** | 0.00000 | 1.0000 | 49 |
| pestilence @20K | **0.00000** | **0.00000** | 0.00000 | 1.0000 | 168 |
| collapse @20K | **0.00000** | **0.00000** | 0.00000 | 1.0000 | 176 |
| drought @20K | **0.00000** | **0.00000** | 0.00000 | 1.0000 | 209 |
| calm @50K | **0.00000** | **0.00000** | 0.00000 | 1.0000 | 264 |

Violence fires hundreds of times, and every downstream link reads zero. The
dead fan-out from this one unwritten field:

- **nervous acute pain** — `pain.update(injury)` → always 0 → `pain_penalty` 0
- **cardiovascular blood loss → shock** — blood loss needs `injury > 0.3`, so
  `blood_volume` never leaves 1.0 → `shock_risk` 0 → `shock_penalty` 0
- **immune wound exposure** — needs `injury > 0.5` → always 0
- **`chronic_damage = injury×0.5 + sickness×0.3`** — the injury half never fired
- **`BodyState.injury`** — the legacy facade field was never refreshed either, so
  bench/TUI consumers (`catalyst_observers`, `development`) read a birth zero

## The fix

1. `EmbodiedState::wound(severity)` — records/stacks a wound (clamped). The
   violence path now calls it after `apply_injury` (norms_impl.rs).
2. `EmbodiedState::heal_injury()` — a wound must close: 0.0005/tick ≈ 0.07/day,
   so a 0.12 violence wound clears in ~1.7 days, a life-threatening 1.0 in ~14
   days. Quantize-safe (0.0005 > the 1e-4 step, AGENTS §5). Called at the top of
   `tick_update`.
3. `core.rs` write-back mirrors `embodied.injury` into `BodyState.injury` — the
   same class of dead legacy-facade sync that i309 fixed for health.

Deliberately **not** removed: `apply_injury`'s direct health subtraction. The
violence severity (0.12) was calibrated around it ("~7 hits to kill", Iter-185);
the wound field adds the *pain/shock/chronic* consequences on top, it does not
replace the tissue damage.

## Probe verdict — `INJURY_CHANNEL_LIVE`

| context | max injury | max pain | max shock | min blood-vol |
|---|---|---|---|---|
| pestilence @20K | 0.334 | 1.000 | 0.000 | 0.897 |
| collapse @4 320 | 0.268 | 1.000 | 0.000 | 1.000 |
| drought @20K | 0.285 | 1.000 | 0.000 | 1.000 |
| calm @20K | 0.514 | 1.000 | **0.845** | **0.300** |
| calm @50K | 0.514 | 1.000 | 0.845 | 0.300 |

Injury, pain, blood loss and shock are all live. The derived-health floor drops
from ~0.40 to ~0.31–0.41 and now decomposes with live `pain` (0.06–0.10) and
`shock` terms — the i312 "no reachable crisis band" finding is materially
improved, though `health < 0.25` remains unreachable (the gate re-contract is
its own queued item).

**Recorded, not smoothed:** the calm-family leg reaches the blood-volume floor
(0.300) with shock 0.845. Resting blood recovery is `nutrition × recovery_rate ×
0.002 ≈ 1e-4/tick`, so a bled-out agent takes ~7 000 ticks (~48 days) to
refill — slow but bounded and monotone. Logged as an observation, not a pin.

## Drift re-anchors (§4)

- **RE-CONTRACT** (sim test `emotional_body_tone_resists_regulation_in_tick`):
  the assertion was an exact `assert_eq!` on conflict counts justified by i275's
  "bounded, not chaotic" rationale. The injury channel shifts behavioural
  closure enough that the two runs no longer coincide exactly. The real invariant
  the comment names is *bounded* divergence (the i275 failure state measured
  Δ24); the measured value is Δ2 (133 vs 135). The pin now guards
  `divergence <= 10` explicitly.
- **4 snapshots** re-anchored (`metrics_500`, `metrics_2000`,
  `agent_state_1000`, `long_horizon_surface_10000`). Mechanism: a wound raises
  `nervous.pain` and the endocrine axis "moves under pain" (Iter-188), so
  behavioural closure and the demographic trajectory shift — the 10K surface
  moves `agent_count 12 → 13` and its relationship/memory distributions.
- **Golden byte-identical** — the collapse/seed_42 crisis baseline does not
  cross a metric boundary inside its 4 320-tick window.

## Regression pins

- `mindstrata-person::biology::injury_channel_tests` — 4 tests (record/stack,
  heal, pain, health-penalty).
- `mindstrata-sim::sim::tests::conflict::violence_records_injury_on_the_substrate`
  — 20K seed-42 run: recorded violence must leave a wound that reaches pain.

## Verification

`cargo fmt --all` clean · clippy **0** · person **120/120** · sim **278/278** ·
tests **309/0/1** · `scripts/gate --full` **GATE GREEN**.
