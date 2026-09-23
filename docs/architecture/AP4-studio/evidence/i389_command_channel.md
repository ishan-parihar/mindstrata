# i389 — the directive channel gets a producer (and the vacancy behind it)

**Status:** LANDED (behavioural) · **Root cause owned:** DC-5's G1 — authority was
**nominal**: offices existed, the directive API existed, the consumption path existed,
the census counted the layer, and **no shipped producer ever emitted a directive**.

## Producer census before (probe `i389_command_channel`, 500 warmup / 20 000 window)

| world | `command` share | directive holders | council legitimacy | Elder office filled |
|---|---|---|---|---|
| calm village s42 | **0.00%** | 0/12 | 0.5542 | true |
| collapse s42 | **0.00%** | 0/13 | 0.5732 | true |
| pestilence s07 | **0.00%** | 0/12 | 0.5673 | **false** |

The crisis *duty cycles* were measured before anything was wired (§4.13 — the
manipulation must be live before the response is sized):

| world | panic active | mean fear > 0.5 | mean hunger > 0.6 |
|---|---|---|---|
| calm | 0.00% | 0.00% | 0.00% |
| collapse | 0.00% | **1.95%** | 0.00% |
| pestilence | **90.69%** | 0.00% | 0.00% |

Two things fall out: a decree keyed on *hunger* would be inert in this corpus (the
collapse scenario's famine is over before the window opens — mean hunger 0.026–0.040
against the 0.6 bar), and a **250-tick cadence can only carry a condition that lasts**
— the fear route's 1.95% duty cycle is enough, a 30-tick spike would not be.

## The producer

`systems/decree.rs` — on the cadence, an office-holding council whose mandate holds and
whose settlement is in crisis directs its **four nearest villagers**:

- **crisis → ask**: `famine_open || mean hunger > 0.6` → `Work` (the granary is short);
  `panic_active || mean fear > 0.5` → `Worship` (public ritual — the
  legitimacy-restoring response the institution already models); neither → **no decree**.
- **mandate scales the ask**: priority `legitimacy × 0.8`, floor `0.35` (two thirds of
  the measured 0.54–0.58 mandate equilibrium) — a resented council is not heard.
- **a decree DECAYS**: it ships as `GoalSource::Decree`, a directive in every
  behavioural respect (`command_goal_action` steers from it, acting on it consumes it)
  **except** that `system_goal_generation` does not exempt it from the ordinary priority
  decay. That is the distinction an in-sim authority needs and the operator channel does
  not: authority is a repeated ask (~800-tick life), not a standing order. No clearing
  logic, no way for a stale decree to steer a village years later, and the TUI's
  `Command` semantics are untouched.

## Measured after

| world | before | after |
|---|---|---|
| calm village s42 | 0.00% | **0.00%** (0 selections, 0 decree agent-ticks) |
| collapse s42 | 0.00% | **0.05%** (30 selections, 181 decree agent-ticks) |
| pestilence s07 | 0.00% | **0.00%** — see the vacancy below |

The calm leg is the item's exit criterion (authority must not speak when nothing is
wrong) and it holds **byte-identically**: the calm golden is unchanged.

## The finding this iteration exposed

**The pestilence leg is a VACANCY, not a dark producer.** Its panic duty cycle is
**90.69%** — the crisis condition is maximally live — and the census is 0 because
`Elder office filled = false`: after a pestilence the council has **no office holder at
all**, and nothing in the engine appoints a successor when one dies. An office that
empties and stays empty is the institutional form of the dead-producer class: the
*institution* survives (members, treasury, legitimacy 0.567) while its **authority** is
structurally absent.

Isolated by pin (`council_decrees_are_issued_in_crisis_and_never_in_calm`): with the seat
filled, a frightened village draws a decree inside one cadence and a calm one never does.
**Succession is the next root cause** (recorded; not attempted here — one root cause per
iteration, and the vacancy lives in the institution lifecycle, not the directive path).

## Pins (4 new)

- `sim::command_channel_tests::decree_is_a_directive_that_decays` — consumption parity
  with `Command`, and the retain arm as the expiry (the documented distinction).
- `sim::command_channel_tests::council_decrees_are_issued_in_crisis_and_never_in_calm` —
  the conditional liveness proof, with the office explicitly filled so it measures the
  producer rather than the vacancy.
- `systems::decree::tests::decree_priority_reads_the_mandate` — floor + mandate share.
- `systems::decree::tests::decree_for_maps_crisis_to_an_ask` — crisis → kind, calm → none.

## Blast radius

**Collapse golden re-anchored** (`agent_count` **12 → 12** — the mortality/replacement
invariant intact; the hash moved because the collapse world now contains authority-steered
decisions, 30 of them). **Calm golden byte-identical** (no decree exists to move it),
**no snapshot drift**, integration **313 passed / 0 failed / 1 ignored**.

Verification: `cargo fmt --all` clean · `cargo clippy --workspace` 0 warnings · sim lib
**307/307** (+4) · integration **313/0/1** · collapse golden re-anchored with evidence ·
`scripts/gate --full` GREEN.
