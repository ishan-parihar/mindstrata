# Iteration 307 — the habit fallback was overriding the survival-integrity reflex

**Status:** LANDED · **Root cause owned:** the i306 residual (21 long ceiling
excursions, all agents pinned at thirst 1.00 **and** fatigue 1.00 for 12–15K
ticks whose physiological reflex selected `Drink`/`Rest` and never cleared).

## The probe found a contract violation, not a supply problem

Probe `i307_physio_saturation` tested three candidate mechanisms before touching
code — supply, access, execution:

| mechanism | measured (seed 99 agent 2, 50K ticks) | verdict |
|---|---|---|
| A. SUPPLY — no water in the village | well stock mean **1952.91** raw units, min 1941.83 | ruled out |
| B. ACCESS — water exists but is blocked | accessible **50 000 / 50 000** ticks, blocked 0 | ruled out |
| C. EXECUTION — the action runs but relief doesn't land | **4 Drink ticks in 50 000**, action census `Trade 0.997` | **the defect** |

The reflex layer (Iteration 255) selects `Drink` when `thirst > 0.9`. The agent's
thirst sat at 1.0000 for **49 212 consecutive ticks** while its action census was
Trade 0.997 — i.e. the reflex was being *overwritten*, every tick, by something
downstream of it in the same pass.

That something is the §8.1.19 stress-habit fallback (Iteration 398-era), which
runs after selection and substitutes the agent's strongest habit when
`stress > 0.5 && automaticity > 0.5`. Two facts make this a **contract
violation rather than a tuning choice**:

1. The i255 block's own comment states the contract verbatim: *"Critical deficits
   force the relief action directly — no utility contest, **no habit
   substitution**, no command override can outrank a body at its limits."* The
   habit fallback removed exactly that guarantee.
2. The habit substitution table is `{Work, Trade, Socialize, Worship, Eat}` — it
   **cannot produce `Drink` or `Rest` at all**. A chronically stressed,
   habituated agent was therefore structurally incapable of drinking or
   sleeping, no matter how critical the need.

So the audit's headline E3 guarantee ("agent starving while working is impossible
by construction") was **false for exactly the agents most likely to hit it**.

## What landed

One gate in `pass_action.rs`:

```rust
if reflex_override.is_none()          // ← i307: the survival reflex is absolute
    && stress > Fixed::from_f64(0.5)
    && agents[i].psych_skills.automaticity > Fixed::from_f64(0.5)
{ /* habit substitution */ }
```

**2 sim pins** (`sim/tests/psychology.rs`):

- `habit_substitution_cannot_override_a_physiological_reflex` — an agent with a
  strong thirst-triggered Trade habit, `automaticity = 1.0`, stress 1.2 and thirst
  1.0 (every gate condition satisfied) still selects `Drink`, and its thirst falls.
- `habit_substitution_still_fires_when_no_reflex_is_active` — the mirror clause:
  the same agent below every threshold still gets its habit substituted, so the
  gate did not disable §8.1.19.

## Exit evidence

### Vanilla family, 12 seeds (probe legs 1–2)

| horizon | longest run in a physiological reflex zone | drink relief landings |
|---|---|---|
| 20 000 | 13 813 → **0** | 0.00811 → 0.00855 per agent-tick |
| 50 000 | 43 813 → **0** | 0.00748 → 0.00823 per agent-tick |

Per-agent anatomy, seed 99 agent 2 @50K: thirst final 1.0000 → **0.2460**, Drink
ticks 4 → **470** (landings 2 → 396), Rest ticks 40 → 7 528, census `Trade 0.997`
→ `Trade 0.560 | Work 0.278 | Rest 0.151 | Drink 0.009`. 12/12 seeds alive at
every horizon.

### Crisis path — the collapse golden's own scenario (seed 42, 4320)

| metric | pre-fix | post-fix |
|---|---|---|
| in-reflex-zone duty | 0.08800 | **0.00258** (−97%) |
| final thirst / hunger / fatigue | 0.257 / 0.105 / 0.237 | **0.111 / 0.027 / 0.125** |
| final health | 0.790 | 0.782 |
| final population | 12 | 12 |
| habit-gate inputs (stress>0.5 ∧ auto>0.5) | 0.29925 | 0.27118 |

The gate is therefore **not** structurally zero-blast: 27% of crisis agent-ticks
open the habit gate, so the crisis golden legitimately re-rolls (below).

## Re-anchors (three, all probe-evidenced, one is a re-contract)

1. **`golden/collapse/seed_42` regenerated.** The crisis is exactly where
   automaticity and stress both climb (27% of agent-ticks satisfy the habit gate
   within the 4320-tick golden horizon) — the §8.1.19 comment claiming the
   fallback only bites "long-horizon runs where practice has accumulated" was
   never probed under crisis. Mechanism: agents at their physiological limits
   now act on them instead of working through them, so the famine-era village
   actually consumes its grain — `total_grain 28.8882 → 1.3104`,
   `total_water 1145.3427 → 1141.5698`, mortality unchanged (12 agents), health
   essentially unchanged (0.790 → 0.782). The scenario's purpose (compound
   crisis → breakdown dynamics) is preserved; the stress it produces is now
   *metabolized* rather than ignored.
2. **`snapshot_long_horizon_surface_10000_ticks` regenerated.** Modest,
   mechanism-consistent drift: `event_count 140207 → 140412` (+0.15%), avg stress
   `0.3787 → 0.3702`, memory traces `969 → 830` (rest-heavy agents encode fewer
   traces), grain `1.2081 → 1.4614`, agent_count 12 unchanged, memes 7 unchanged.
3. **`long_horizon_50k_is_deterministic_and_emerges` — RE-CONTRACT, not a
   re-pin.** The leg asserts an average belief-charge band on a **pestilence**
   world (seed 5 @50K) using a ceiling taken from the §7.2 panic trigger (0.5).
   Measured both sides: avg charge **0.3877 → 0.5242**, reflex-zone duty
   **0.2260 → 0.0013**, stress 0.591 → 0.470, health 0.739 → 0.676, panics
   30 → 27, factions 5, population 15. The pre-fix 22.6% reflex-zone duty means
   that village was substantially *desensitized by chronic dehydration* — a
   village living through a pestilence **should** charge its beliefs near the
   trigger; that is the scenario's own emergence signal (panics fire). The old
   sub-trigger ceiling was the wrong contract for a crisis leg, so the guard was
   re-contracted to what it must actually protect — **alive and un-saturated**:
   floor 0.10 (the dead-channel floor from the Iteration-241 audit finding),
   ceiling 0.90 (saturation guard), with the panic/faction/ritual assertions in
   the same function carrying the "regulated, not numb" burden. Measured
   post-fix dispersion: n 30, p10 0.470, p50 0.542, p90 0.625, max 0.695, **zero
   beliefs at ≥ 0.99** — a live spread, nothing pinned.

## Findings recorded, not smoothed

1. **New equilibrium: an agent whose habit stopped pinning it can settle into
   Rest.** Seed 123 agent 6 @50K post-fix: `Rest 0.813` with fatigue mean 0.040
   and fatigue < 0.05 for 88% of the run (pre-fix census: `Trade 0.986`). This is
   *not* a reflex loop (longest reflex run 0) — it is the utility layer's own
   equilibrium for that agent once the habit override stops forcing Trade. It
   looks like a rest-dominance bias in the selector for low-pressure agents, and
   it is the next behavioural candidate (it needs its own probe; it is not part of
   this root cause).
2. **Fatigue equilibrium at 50K is high (family p90 mean 0.359) even with the
   reflex live** — Rest relief exists and lands, so this is a supply/pace
   question, not a dead producer. Recorded with the leg-1 table as its baseline.
3. **`runs_action_selection()` is dead code** (the same class as
   `runs_social_interactions`, Iteration 144): defined on `AgentTier` but with
   zero call sites, so Background agents still run full action selection. Found
   while hunting this bug; recorded, not fixed (wiring it is a calibrated change).

## Verification

- core **34/34**; sim lib **274/274** (2 new pins); tests-crate release suite
  **309/0/1** with three documented re-anchors above.
- fmt clean; clippy 0; full `scripts/gate --full` **GATE GREEN**; bench index
  `--strict` 0 violations.

## Queue after this iteration

- **Rest-dominance equilibrium** (finding 1) — probe the utility selector's Rest
  bias for low-pressure agents.
- **Fatigue pace at 50K** (finding 2).
- **Meaning channel**: the i306 reflex still leaves agents whose physiological
  reflex outranks it pinned (recorded there); unchanged by i307, and the i307
  gate should sharply reduce that population (physiological reflex-zone duty
  0.2260 → 0.0013 in the crisis leg) — worth re-measuring.
- **Row 3 residuals** (ceiling band, Q2 dark-allergy saturation); **row 1** stays
  prohibited (AGENTS §5 H5); DC-4's CLIENT asset-viewer + graphical shell.
