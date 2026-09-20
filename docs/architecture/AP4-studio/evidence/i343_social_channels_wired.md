# i343 — The contacted-degree channels are wired: a dead producer revived, and two seed-flip treadmills ended

**What landed.** i342's survey, wired: `social_visibility` and normal-life anxiety's
`social_factor` now read `Simulation::contacted_degrees()` (rows carrying interaction state)
instead of `relationship_v2s.len()` (the complete graph's N−1 for every agent, so
`min(len,4)==4` for everyone and both channels were constants). One O(R) pass in `core` feeds
the appraisal pass; the clamp and the 0.1 weight are unchanged.

## Probe: the change is small, bounded, and targeted

`i343_social_channels` (seed 42, 32×32; PRE computed inline from the list length — exactly what
the sim did — so one run yields both columns):

| ticks | N | contacted deg | isolates (<4) | `social_visibility` PRE min/mean | POST min/mean | anxiety PRE max | POST max |
|-------|---|---------------|---------------|----------------------------------|---------------|-----------------|----------|
| 2 000 | 12 | 4.5 | **4/12** | 0.900 / 0.900 | **0.700 / 0.850** | 0.000 | **0.004** |
| 2 000 | 48 | 5.3 | **15/48** | 0.900 / 0.900 | **0.800 / 0.869** | 0.000 | **0.002** |
| 2 000 | 96 | 10.5 | 0/97 | 0.400 / 0.885 | 0.400 / 0.885 | 0.000 | 0.000 |
| 20 000 | 12 | 5.7 | **4/13** | 0.400 / 0.862 | 0.400 / **0.815** | 0.000 | **0.004** |
| 20 000 | 48 | 9.4 | **8/52** | 0.400 / 0.862 | 0.400 / **0.846** | 0.000 | **0.002** |
| 20 000 | 96 | 14.9 | 0/101 | 0.400 / 0.865 | 0.400 / 0.865 | 0.000 | 0.000 |

Only agents below 4 contacts move (their visibility drops, their anxiety term becomes
non-zero); agents at or above 4 keep the calibrated value. At N=96 population-scaled housing
(i340) leaves **no** isolates, so the channel is naturally quiet there — it is live exactly
where isolation exists.

## The sweep: 13 tests, and what each one actually needed

The blast was exactly the 13 i342 predicted.

**Two liveness anchors were re-contracted onto seed families — not re-pinned, and not
re-seeded.** `i343_liveness_sweep` (pestilence, charter world):

```
panic @20K  — seeds {1: 4, 3: 0, 5: 0, 7: 13, 11: 5, 42: 10}      → 4/6 fire
revolution @70K — seeds {12345: 0, 1: 0, 5: 3, 7: 0, 11: 3, 42: 3} → 3/6 fire
```

Neither producer is dead, so §2.3's "revive the producer" does not apply and §4.1 forbids
flipping to a seventh lucky seed. Both anchors were **single-seed pins that had already been
re-seeded repeatedly** — the revolution leg six times (42 → 7 → pestilence 42 → 5 → 1 →
12345), which is precisely the flip-flop §4.5 records as debt. They are now **families**:

- panic: `≥2 of {1, 7, 42}` register, with determinism checked on one member;
- revolution: `≥2 of {5, 11, 42}` fire, and **every** seed that fires must satisfy the
  absorption contract (peak council ≥ 5) — the mechanism is guarded on each live instance
  rather than one trajectory;
- the moral-panic leg (already a family after i334) had the same latent flaw — it required
  **every** member to register, which is single-seed fragility in family clothing. Seed 5 is
  now its cold member; the assertion is `≥2 of {5, 7, 42}` and Leg C's determinism replay moved
  to seed 7 (a member measured to register), so it is not trivially satisfied by an empty
  registry.

**One magnitude band was re-pinned with probe evidence (§4.2).** The conception pin's seed-1
volume band @175K measured **18 births (i242) → 5** now. `i343_birth_pacing` (seed 1 @250K)
distinguishes the two readings: births keep accumulating past the pin horizon — **5 by 175K, 6
by 200K, 9 by 250K**, final population 21 with 10 partnered and mean contacted degree 16.5 — so
this is **re-pacing, not suppressed fertility**. Band floor 8 → 3, same contract (liveness,
healthy-not-explosive volume, no birth inside a calibrated window, bookkeeping intact).

**Two goldens and seven snapshots moved** (documented, reviewed, not waved through):
regenerated goldens show `agent_count` stable (12/12) with modest grain/water shifts; the
snapshot diffs were reviewed before acceptance — relationship-stage redistribution
(Acquaintance 19→20, Ally 48→45), `avg_relationship_trust` 0.7424 → **0.7065**,
`avg_relationship_quality` 0.6202 → **0.5738**, `event_count` 140 737 → 132 282,
`avg_stress` 0.3761 → 0.3658, `avg_health` 0.7909 → 0.7881 — all bounded, all in the
social/relational direction the change touches.

## §2.5 re-audit

- **Both channels are now live and discriminating** (visibility spans 0.700–0.900 across the
  family, anxiety 0.000–0.004), and quiet at N=96 where no isolation exists.
- **No new saturation**: `avg_stress` 0.3658 village-wide at 10K (not pinned at 0 — a single
  agent's `stress_level: 0.0` in `agent_states_1000_ticks` is one trajectory, not the village),
  no value clamps at 1.0, `agent_count` stable in both goldens.
- **Panic, revolution, moral-panic, conception and meaning/immune liveness pins all hold** on
  the shipped build (309/0/1, GATE GREEN).
- **Debt resolved by contraction**: seed 5's knife-edge panic saturation (§4.5, recorded at
  i334/i338) no longer anchors anything — both panic legs are families now, so one cold seed
  can no longer flip a contract.

## Ledger effect

- **A10 CLOSED.**
- The seed-flip treadmill is closed for the revolution and panic legs; the families are the
  stable form, and the next re-pacing event should not need a re-anchor.
- `contacted_degrees` is the first concrete contract for the queued sparse store (i344): it is
  the "how many social connections" answer a materialised-row store must still provide.

## Reproduce

```
cargo run --release -p mindstrata-benches --example i343_social_channels -- 12 48 96
cargo run --release -p mindstrata-benches --example i343_liveness_sweep
cargo run --release -p mindstrata-benches --example i343_birth_pacing
```
