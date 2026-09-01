# i293 — 20-seed Q2/Q3/Q4 sweep (5K ticks, N=12)

Owner: DESIGN + STORY/SIM. Probe `crates/mindstrata-benches/examples/i293_q2q3q4_seed_sweep.rs` at HEAD `45d4158`.

## Question

Do Q2 (dark_allergy), Q3 (golden_addiction), Q4 (golden_allergy) fire
at any seed in the 5K/12-agent regime, or is seed 42's zero an
outlier? This determines whether Q2/Q3/Q4 can be calibrated at
5K/12 or need forced events / larger N / longer horizon.

## Method

20 seeds × 5000 ticks, N=12, `SimConfig` default. Per-seed mean
intensity for each quadrant (`a.development.pathology.*.intensity`
averaged over agents). Event count and agent count recorded.

```
cargo run --release --example i293_q2q3q4_seed_sweep -p mindstrata-benches
```

## Results

| seed | Q1 dark_addiction | Q2 dark_allergy | Q3 golden_addiction | Q4 golden_allergy | events | agents |
|---|---|---|---|---|---|---|
| 1 | 0.2929 | 0.0000 | 0.0395 | 0.0000 | 69933 | 12 |
| 2 | 0.2936 | 0.0000 | 0.0395 | 0.0000 | 62978 | 12 |
| 7 | 0.2574 | 0.0000 | 0.0329 | 0.0000 | 67647 | 12 |
| 13 | 0.1837 | 0.0000 | 0.0353 | 0.0000 | 68887 | 13 |
| 21 | 0.3278 | 0.0000 | 0.0414 | 0.0000 | 67589 | 13 |
| 42 | 0.3015 | 0.0000 | 0.0414 | 0.0000 | 77699 | 13 |
| 46 | 0.1131 | 0.0000 | 0.0395 | 0.0000 | 67874 | 12 |
| 55 | 0.1592 | 0.0000 | 0.0395 | 0.0000 | 59390 | 12 |
| 77 | 0.2185 | 0.0000 | 0.0329 | 0.0000 | 62961 | 12 |
| 99 | 0.2857 | 0.0000 | 0.0414 | 0.0000 | 69943 | 13 |
| 123 | 0.2084 | 0.0000 | 0.0395 | 0.0000 | 62523 | 12 |
| 12345 | 0.1767 | 0.0000 | 0.0395 | 0.0000 | 61342 | 12 |
| 999 | 0.2289 | 0.0000 | 0.0329 | 0.0000 | 61468 | 12 |
| 2026 | 0.1170 | 0.0000 | 0.0329 | 0.0000 | 57594 | 12 |
| 4242 | 0.2152 | 0.0000 | 0.0329 | 0.0000 | 68161 | 12 |
| 7777 | 0.1692 | 0.0000 | 0.0414 | 0.0000 | 66813 | 13 |
| 100 | 0.1260 | 0.0000 | 0.0329 | 0.0000 | 67745 | 12 |
| 200 | 0.2466 | 0.0000 | 0.0329 | 0.0000 | 61039 | 12 |
| 300 | 0.2386 | 0.0000 | 0.0353 | 0.0000 | 67128 | 13 |
| 400 | 0.1758 | 0.0000 | 0.0329 | 0.0000 | 65254 | 12 |

* Q1 (Threat→dark_addiction) fires at all 20 seeds: mean 0.11–0.32,
  monotone with horizon (i288: 0.30 at 5K → 0.41 at 20K for seed 42).
* Q2 (Transgression→dark_allergy) and Q4 (Grief→golden_allergy)
  are **exactly 0.0000 at all 20 seeds** at 5K/12.
* Q3 (Bond→golden_addiction) is small but non-zero at all seeds:
  0.0329–0.0414 (marriage/child events do fire, but rarely).

## Mechanism

Q2 and Q4 never fire at 5K/12 because their onset triggers are rare
emergent events:

* Q2 `Transgression` requires `SimEvent::NormViolated` (norm/legal
  violation observation) — fires 0–2 times per 5K at N=12 in the
  observed seeds (see `i288` 5K→20K at seed 42: both 0.0000).
* Q4 `Grief` requires `SimEvent::AgentDied` — also 0–2 deaths per
  5K at N=12/20 seeds.

With `OperatorParams::pending()` the Allergy quadrants have the early-
return identity `if pressure == 0.0 && intensity == 0.0` (dynamics.rs:96)
so a quadrant that receives zero pressure on most ticks stays at
exact zero. The few ticks that do carry Transgression/Grief pressure
(0.5/1.0 admitted via `Gate::pending` threshold 0.05) are too rare
to lift the mean above 0.0001 at 5K.

Q1 and Q3 fire because `Threat` (conflict/feud, 0.4 mag) and `Bond`
(marriage/child, 0.8/0.7 mag) are common at all seeds.

## Verdict

* **Q1: CALIBRATED** — fires at all seeds, trajectory evidenced at
  5K/20K (pending() shape correct per i269/i286/i287/i288).
* **Q3: CALIBRATED (weak)** — fires at all seeds but small
  (0.03 mean); needs longer horizon or larger N to reach the
  `golden_addiction` growth curve's plateau; pending() holds.
* **Q2/Q4: CALIBRATION-PENDING** — cannot be calibrated at
  5K/12/natural events. Requires one of:
  1. forced `Transgression`/`Grief` catalysts via scenario shocks
     (scenario editor injecting norm-violation/death events),
  2. larger N (48) and longer horizon (20K) where emergent
     violations/deaths are more frequent, or
  3. a seed known to produce a violation/death burst (not found
     in the 20-seed sweep; none of 20 hit it).

The pending() values (`growth 0.05 / decay 0.02 / ceiling 1.0`)
are symmetric and correct as placeholders for Q2/Q4; the per-
quadrant specialization question (pathology-curves.md open #3)
remains open until Q2/Q4 trajectories are measurable.

## Follow-up

* DC-2 calibration should run the same sweep at N=48/20K or with
  forced `Transgression`/`Grief` scenario shocks and record the
  first seed where Q2/Q4 mean exceeds 0.01.
* The `bench_index` gate is unaffected; the pathology operator's
  zero-at-zero law keeps Q2/Q4 neutral until a real catalyst
  arrives, so no pin risk at calibration-pending defaults.

`evidence/i293_20seed_q2q3q4_sweep.md:1` is the 20-seed sweep
evidence for the Q2/Q3/Q4 calibration-pending verdict at HEAD
`45d4158`.
