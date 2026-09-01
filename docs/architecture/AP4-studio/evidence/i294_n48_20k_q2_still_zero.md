# i294 — N=48 20K Q2/Q4 still zero (natural rate)

Owner: DESIGN + SIM. Probe `crates/mindstrata-benches/examples/i294_n48_q2_sweep.rs` at HEAD `3ab7d67` (post-absence-fix).

## Method

3 seeds (42, 1, 7) × 2 populations (N=12, 48) × 20K ticks,
`SimConfig` default (24×24 world for N=48).  Per-seed mean Q1–Q4
(`a.development.pathology.*.intensity` averaged over agents),
cumulative `event_count` and `agent_count`.

```
cargo run --release --example i294_n48_q2_sweep -p mindstrata-benches
```

## Results

| seed | N | Q1 dark_addiction | Q2 dark_allergy | Q3 golden_addiction | Q4 golden_allergy | events | agents |
|---|---|---|---|---|---|---|---|
| 42 | 12 | 0.3859 | 0.0000 | 0.0353 | 0.0000 | 291021 | 13 |
| 42 | 48 | 0.3689 | 0.0000 | 0.0403 | 0.0000 | 1017182 | 53 |
| 1 | 12 | 0.3894 | 0.0000 | 0.0374 | 0.0000 | 301105 | 14 |
| 1 | 48 | 0.3878 | 0.0000 | 0.0414 | 0.0000 | 1072697 | 52 |
| 7 | 12 | 0.4279 | 0.0000 | 0.0329 | 0.0000 | 259952 | 12 |
| 7 | 48 | 0.4119 | 0.0000 | 0.0417 | 0.0000 | 1108970 | 53 |

* Q1 and Q3 fire at all 6 conditions (Q1 0.36–0.42 at 20K, monotone;
  Q3 0.03–0.04, small but non-zero).
* Q2 (Transgression→dark_allergy) and Q4 (Grief→golden_allergy) are
  **exactly 0.0000 at all 6 conditions**, even at N=48/20K with
  ~1M events.

## Mechanism

Q2's trigger is `SimEvent::NormViolated` (0–2 per 20K at N=12/48 in
observed seeds); Q4's trigger is `SimEvent::AgentDied` via the same
rare demographic path.  At natural rates these events never fire for
any agent in 20K, so `dark_allergy`/`golden_allergy` never leave
neutral.  The infrastructure fix in `3ab7d67` (absence-driven growth
for Allergy when `intensity > 0` but no trigger this tick) is
structurally correct — `development.rs:190-205` and
`dynamics.rs:96` zero-at-zero — but dormant because the first
trigger never arrives.

## Verdict

* **Infrastructure**: FIXED and verified GREEN (gate 308/0/1,
  `development.rs` absence pass, `dynamics.rs` early return).
* **Calibration**: Q2/Q4 remain **CALIBRATION-PENDING** — the natural
  emergent rate at N=12/48 × 5K/20K is too low to produce the
  `Transgression`/`Grief` catalysts.  Calibration requires **forced
  scenario shocks** that inject `NormViolated`/`AgentDied` (or
  `Transgression`/`Grief` `CatalystKind`) at a controlled rate,
  or a dedicated `i<iter>_forced_q2q4` probe.  This is DC-2
  scenario work, not a code gap.

## Follow-up

* DC-2 should add a forced-Q2/Q4 scenario (e.g., scenario editor
  injecting `NormViolated` at 1 per 100 ticks for a single agent)
  and record the first trajectory where Q2 mean exceeds 0.01.
* Q1/Q3 remain CALIBRATED (weak for Q3) and the Allergy
  infrastructure will correctly show recoil accumulation once the
  first forced trigger fires.

`evidence/i294_n48_20k_q2_still_zero.md:1` is the evidence that
natural-rate Q2/Q4 calibration is not reachable at 20K/N=48 and
forced scenario is required.
