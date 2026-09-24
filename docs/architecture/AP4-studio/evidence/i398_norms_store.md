# i398 — the norms_impl migration: reader landed, writers measured and deferred

**Status:** PARTIAL LANDING (the reader) + **TWO MEASURED REVERSIONS** (the writers,
twice-shaped). Probe: `crates/mindstrata-benches/examples/i398_norms_store.rs`.

This is the first landing of the reader-first sweep that i393 §6 ordered. `norms_impl.rs`
holds four v1 sites: one reader (the belief-evidence channel, `rel_pos` → `trust − 0.5`)
and three writers (violence damage in both directions, norm-punishment damage — each with
a §19.5.J provenance trace).

## 1. The probe (legs A–B)

| N / window | events | \|v1−v2\| at read pairs | evidence \|E\| v1 → v2 | sign flips | store levels v1 / v2 |
|---|---|---|---|---|---|
| 12 / 3000–5000 | 18 453 | 0.0122 mean / 0.607 max | 0.472 → 0.463 | 88 | 0.792 / 0.740 |
| 48 / 3000–5000 | 73 402 | 0.0822 mean / 0.911 max | 0.419 → 0.355 | **4 790** | 0.638 / 0.577 |
| 48 / 18000–20000 | 198 495 | 0.0017 mean / 0.127 max | 0.464 → 0.463 | 21 | 0.717 / 0.717 |

**The decisive reading is the horizon structure.** At N=48 the early-window divergence is
huge (0.082 mean, 4 790 evidence sign flips per window) and the late-window divergence is
~nil (0.0017). That is the i376 sync signature from the *other side*: the early window is
pre-first-boundary (the reader sits mid-day on rows the sync has not yet pulled), and by
20K the two stores are essentially the same object. **The v1 read is transient twice
over** — a different number per tick, converging to v2 as the run matures.

Conflicts in the windows: 109 / 316 / 615 (violence 4 / 13 / 11) — each fires four damage
writes, so the writer stakes are real but episodic.

## 2. What landed: the reader (byte-identical, no re-anchors)

The belief-evidence channel now reads the dyadic store:

```rust
let trust = self.agents[from_idx].relationship_v2s
    .get(Self::relationship_v2_pos(from_idx, to_idx))
    .filter(|r| r.to == *to)
    .map_or(Fixed::from_f64(0.5), |r| r.trust);
```

(same 0.5 stranger prior for a missing row — the fallback `rel_pos`'s `map_or` provided).

**Result: integration 314/0/1, both goldens byte-identical, sim 310/310.** The suite ran
three times across the experiments; the reader alone never moved a single pin. Why it is
provably inert rather than luckily inert: (a) the probe shows the stores agree at the
horizons every pinned window lives at (≤0.017 divergence at all pinned horizons, vs the
0.082 pre-boundary spike); (b) the consumer is *linear* in evidence toward an equilibrium
(`update_belief`: `confidence ← confidence·resistance + evidence·blended_trust + …`,
clamped), so a small input perturbation is a small equilibrium perturbation — no §4.15
elasticity cliff; (c) evidence is symmetric around 0 (confirming and counter evidence),
and the probe's mean |E| barely moves (0.472 → 0.463), so the equilibrium's *level* is
preserved.

This retires `rel_pos`'s last production caller outside the legacy matrix itself (the
method stays: `sim/tests/mod.rs` and the matrix passes still use it).

## 3. What did NOT land: the writers, moved first (experiment), then shaped down

**Shape 1 — all four sites moved together.** Result: **303 passed / 11 failed**:

| failure | reading |
|---|---|
| both goldens + 6 snapshots | `avg_relationship_trust` 0.6345 → 0.6207 @500; stage distribution runs deeper (CloseFriend 2→5, Confidant 0→1); long-horizon `agent_count` **13 → 12** (one extra death) |
| `conception_pipeline_round_trips_with_birth` | a probe-pinned conception slid past its segment boundary — downstream re-timing, the Iter-162 class |
| `peer_status_envy_feeds_daily_anger_end_to_end` | 0.2255 vs floor 0.3 |
| `sensory_field_fear_contagion_is_live_and_sustains_fear` | presence 9/12 vs band 11–12 |

**Mechanism, understood before deciding:** the v1 damage write was **transient** —
i376's daily sync (coupling 1.0) pulls `rel.trust` back onto the dyadic value at the next
`tick % 144` boundary, so the −0.3 survived at most one day and only mid-day readers ever
saw it. On the dyadic store the same −0.3 is **persistent**. Persistent damage pushes
pairs below `social_low_trust_threshold` (0.2), re-arming the Iter-185
low-trust→threat spiral that the 0.12 rate was calibrated *against* (its probe broke
5/6 seeds by 20K). One extra death, deeper stages, lower peer_status — the spiral's
early signature, exactly.

**Shape 2 — writers moved with a re-sized magnitude.** Not attempted this iteration: the
right persistent magnitude is unknown (a violence-family sweep with its own probe), and
guessing it would be exactly the "widen until green" move §4.2 forbids.

**Decision:** the writers revert to v1 **with the persistence finding recorded at the
site** (code comment names the mechanism and the measured failures), and the
writer-magnitude re-sizing becomes its own violence-family iteration: sweep
`violence_trust_damage ∈ {0.05…0.3}` on v2 across the i273 12-seed family + the
death-spiral horizon, pick the value that keeps the spiral calm, then move the write.

## 4. The sweep's remaining items (unchanged, now with one landed)

- [x] `norms_impl` reader — **landed zero-blast** (this iteration)
- [ ] `norms_impl` writers — magnitude re-sized for persistence first (§3)
- [ ] `household`, `births_deaths`, marriage/attachment readers — each with its own probe
- [ ] delete the v1 application in `process_interaction` WITH the
      `bonding_rate`/`conflict_escalation_rate` re-hosting (i393 §6.1's insight)
- [ ] the two folds onto the dyadic store, after fixing the `contagion_delta`
      Fixed-4 truncation hazard (i393 §6.2) and understanding the kin-labelling break
- [ ] extend the daily sync to `affection` (never synced: 0.187 mean / max 1.000 at 50K)

## 5. Artifacts

* Probe: `crates/mindstrata-benches/examples/i398_norms_store.rs` (legs A–B).
* Changed: `norms_impl.rs` — the belief-evidence read moved to the dyadic store; the two
  writer sites stay on v1 with persistence findings recorded in place.
* Reverted: the shape-1 writer migration (both writer sites + their provenance traces
  were briefly on v2).
