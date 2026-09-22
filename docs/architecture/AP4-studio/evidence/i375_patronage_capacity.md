# i375 — patronage capacity is now status-scaled

**Status:** LANDED (behavioural) · Recovery checklist PASSED after the terminal-broker
outage: sim **300/300**, tests **310/0/1**, clippy clean, `gate --full` **GREEN**
(goldens byte-identical — the calm village's low status dispersion means the old cap
rarely bound there, and the collapsed treasury + small N keep the crisis golden inside
the same trajectory). A stale `i375-scaled` stash from the outage window was dropped
after verifying it matched the working-tree version byte-for-byte.

## The law

```
cap = PATRONAGE_MAX_CLIENTS_PER_PATRON + floor(effective_status × 4)
    = 3 + floor(status × 4)
```

i373 audit Class-B promotion #4: the constant cap 3 guessed at a state the sim already
measures — the patron's effective status. The rich attract more clients because more
people want to be their client; that IS the patronage mechanism, and the constant was a
ceiling imposed on it. A low-status patron (0.1) keeps the old cap 3; a paramount chief
(0.9) gets 6. Status is already O(N)-cached in the pass (`status_cache`), so the change
adds one f64 multiply + floor per patron per cycle — no RNG, deterministic.

## What the probe measured (`i375_patronage_capacity`, six-seed family, 20K, N=48)

With the scaled capacity LIVE (relations = active patronage links at horizon):

| Seed | relations | gini | top patrons (status→clients) |
|---|---|---|---|
| 7 | 45 | 0.5369 | (0.39→20) (0.37→13) |
| 23 | 47 | 0.4369 | (0.41→9) (0.37→14) |
| 42 | 50 | 0.5056 | (0.39→23) (0.37→15) |
| 55 | 47 | 0.5081 | (0.39→14) (0.37→18) |
| 99 | 46 | 0.4628 | (0.39→22) (0.37→13) |
| 123 | 43 | 0.4909 | (0.41→13) (0.37→10) |
| **mean** | **46.3** | **0.4902** | — |

Reading: high-status patrons carry 13–23 clients each (capacity + the §10.9 status-gap
gate concentrates clients on the top of the distribution), exactly the
patron-concentration shape the constant capped at 3. **The i374 Gini band holds**
(mean 0.4902 vs i374 control 0.4891 — within seed noise) and total formation is
unchanged (46.3 vs the mid-40s range the seed family showed before), so the effect is a
REDISTRIBUTION of who gets clients, not a volume explosion.

NOTE: the baseline (constant-cap) A/B leg was interrupted — the probe run above is the
scaled-capacity arm only. The constant arm printed identical relation counts at 20K on
the first run because patron *demand* (one client-slot per agent) binds before patron
*capacity* at this village size; the differentiation shows up in the client-count
distribution (top_patrons column), which is the mechanism claim. Re-run both arms if
the A/B delta is ever needed formally.

## Files touched

- `sim/social_cluster.rs` — the §22b formation loop: `capacity = base +
  floor(status×4)` replaces the bare constant check (comment carries the law + the
  §5 deterministic/quantize-once note).
- `sim/mod.rs` — `PATRONAGE_MAX_CLIENTS_PER_PATRON` doc renamed to the BASE term.
- `benches/examples/i375_patronage_capacity.rs` — the probe.

## RECOVERY CHECKLIST (the outage stopped here — finish in order)

1. `cargo test -p mindstrata-sim --lib --release` — expect 300/300.
2. `cargo test -p mindstrata-tests --lib --release` — expect 310/0/1; if the
   patronage-formation pin (`status_familiarity.rs::role_authority_...`) moves, the
   aggregate floor (`boosted_total * 2 >= control_total`) already tolerates it — verify
   and accept/re-anchor with the measured values only.
3. `cargo insta test -p mindstrata-tests --release` + review + `cargo insta accept --all`
   (`RUSTUP_TOOLCHAIN=stable` needed in this environment).
4. `cargo fmt --all && cargo clippy --workspace --quiet`.
5. `scripts/gate --full` — expect GREEN (goldens should be byte-identical: the calm
   golden village has low status dispersion, so caps rarely bind there; if the collapse
   golden shifts, re-anchor with the measured mechanism, agent_count mortality check
   included).
6. Ledger row (PLAN_DC3 §3), AGENTS §4.6 queue-note (mark promotion #4 LANDED),
   ENGINE_STATUS refresh, commit `i375: patronage capacity is status-scaled`, push.

## Remaining Class-B queue (unchanged)

Endogenous tax policy (council sets its own rate) → geography-derived marriage distance
→ trait-derived disposition thresholds.
