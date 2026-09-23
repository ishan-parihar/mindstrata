# i390 — office succession: authority that survives its holder

**Status:** LANDED (behavioural) · **Root cause owned:** the lifecycle step i389 named —
`handle_agent_death` vacates every office its holder held (`role.holder = None`) and
**nothing ever refilled it**, so an institution outlived its authority.

This is the "dead producer" class in its institutional form, exactly as §4.3 describes it:
the *institution* stays alive (members, treasury, legitimacy 0.567, policies) while the
**authority** is structurally absent. No test failed, no other pass noticed — the seat
simply stayed empty.

## The vacancy, measured before anything was wired

Probe `i390_office_succession` (500 warmup / 20 000 window), run with the new pass
**disabled** (§4.2 — the before number must be measured, not asserted):

| world | any role vacant | Elder seat vacant | Elder vacant **with a candidate pool** | orphaned vacancies | directive share |
|---|---|---|---|---|---|
| calm village s42 | 0.00% | 0.00% | 0.00% | 0 | 0.0000% |
| collapse s42 | 0.00% | 0.00% | 0.00% | 0 | 0.0515% |
| pestilence s07 | **100.00%** | **100.00%** | **100.00%** | 0 | **0.0000%** |

The pestilence row is the finding: the seat is dark for **every tick of the window** while a
living membership exists to fill it **100% of the time**. `orphaned vacancies = 0` in all
three worlds — every vacancy in this corpus had a candidate, so recruitment is *not* what
this probe is measuring.

Calm and collapse read 0.00% because nobody dies there in the window: succession is
conditionally live, which is why its cost is a no-op in two of the three worlds.

## The law

`systems/succession.rs` — on a 50-tick cadence, every vacant office whose institution still
has a living member is filled from that membership, by **the institution's own criterion**:

```
candidate = argmax over living members of status_v2.effective_status()   # §11.1 composite
            (wealth · office · prestige · network centrality), ties on AgentId
```

- **One seat per member per institution.** A repeat appointment would just move the vacancy,
  so the pool is consumed as seats are filled.
- **No living member → no appointment.** An empty roster cannot succeed its own office; that
  is a different root cause (recruitment) and is left vacant rather than invented. The probe
  counts this explicitly (`orphaned vacancies`) so the distinction is visible when it starts
  happening.
- **No RNG draw.** The criterion is the sim's own prestige-ordered state, so the pass touches
  no stream (`status_v2` is read-only here).

Appointments run **before** the decree pass, so a council that regains a holder can speak in
the same tick rather than one cadence later.

## Measured after

| world | vacancy (any / Elder / with pool) | directive share | council at end |
|---|---|---|---|
| calm village s42 | 0.00% / 0.00% / 0.00% | 0.0000% (0) | 2 members · 2/2 filled |
| collapse s42 | 0.00% / 0.00% / 0.00% | 0.0515% (30) | 3 members · 3/3 filled |
| pestilence s07 | **0.00% / 0.00% / 0.00%** | **0.0654% (31)** ← was 0.0000% (0) | 3 members · 3/3 filled |

The calm leg is the item's discipline check and it holds: **authority still does not speak
when nothing is wrong** (0.0000%, 0 selections). Collapse is *unchanged* (0.0515%, 30
selections, before and after) — the fix is inert there because no office-holder died in that
window, which is also why its golden needed no touch.

## Blast radius

**Both goldens byte-identical** — `riverford_minor` s42 and `collapse` s42 replayed to their
existing hashes with no re-anchor. That is the expected result rather than a lucky one: the
golden windows contain no office-holder death (calm/collapse vacancy = 0.00% in the table
above), so the new pass makes zero appointments in them. There is no pestilence golden, so
the one world that does change is measured by the probe instead of pinned by a replay.

No snapshot drift. Integration **313 passed / 0 failed / 1 ignored**, sim lib **310/310**
(+3), clippy 0 warnings.

## Pins (3 new, `systems::succession::tests`)

- `succession_is_cadence_gated_and_needs_a_candidate` — off-cadence is a no-op, and an empty
  roster is left vacant (recruitment must not be faked).
- `vacant_offices_are_filled_from_the_living_membership` — with a pool, every seat fills, and
  **no member holds two seats in the same institution**.
- `the_most_respected_member_takes_the_seat` — the criterion is the institution's own
  (`effective_status`), not roster order and not a draw.

## What this does NOT fix (recorded, not papered over)

- **Recruitment.** An institution whose members have *all* died stays vacant forever. The
  probe counts it (`orphaned vacancies`), and it is a separate root cause: membership
  formation is a different mechanism from seat succession.
- The institutional *effects* of a long vacancy (treasury accumulation with no office-holder
  to spend it, legitimacy drift) are unmeasured. i389's directive channel is the first
  consumer and it now answers; if another consumer turns out to read "office holder present",
  it will show up as a liveness pin moving rather than as a silent zero.
