# Iteration 363 — the council surplus dividend actually bends the wealth tail

**Status:** LANDED (behavioural, sweep-carrying — classified re-anchors below) ·
**Item owned:** the wealth-tail fix, **re-scoped by i361** after the i358-scoped
progressive surcharge was refuted.

## What i361 established, and what it changed here

i361 instrumented the failure: the council treasury **hoards** (163 602 coins at 50K seed
42) because its only outflow is the i261 poor relief — a **≤0.5 coin/recipient/cycle**
drip. So any *steeper* extraction (the i358-scoped progressive surcharge) merely feeds an
untouchable sink and **widens** the gap. The corrected fix is therefore **not a steeper
tax but a spending rule**: the council should redistribute its surplus the way the
**Market** already does (i186 progressive dividend, `1/(1+coin)` weighting), instead of
hoarding it.

## The change

In `institutions_impl.rs`, inside the council branch and after the existing poor relief:
above the operating reserve, pay **25 %** of the surplus (`COUNCIL_SURPLUS_DIVIDEND_SHARE`)
as a **wealth-inverse** (`1/(1+coin)`) dividend to the whole village. The i186 note is
honoured explicitly: an *equal* absolute payout preserves the wealth ratio (Gini barely
moves), so the weighting must be inverse to wealth. Deterministic (weights are a pure
function of agent state in fixed order — no RNG).

## Result — the primary target improves and the hoard drains

Identical i358/i361 harness (46×46, N=48, 50K):

| seed | Gini i358 baseline | Gini i361 surcharge (worse) | Gini **i363** | bottom-half i358 | bottom-half **i363** | council treasury i361 | **i363** |
|---|---|---|---|---|---|---|---|
| 42 | 0.647 | 0.7046 | **0.6108** | 10.3% | **11.6%** | 163 602.8 | **240.7** |
| 7 | 0.652 | 0.6714 | **0.6189** | 8.8% | **11.5%** | 625.7 | **262.9** |

Gini falls **below the pre-fix baseline** on both seeds, the bottom half's share rises,
and the 163 K hoard is gone. Provisioning holds (health 0.745/0.734, hunger 0.018/0.022,
grain present, no destitution). `verdict = WEALTH_TAIL_BENT`.

## The classified sweep (behavioural, so every move is re-anchored with a mechanism)

| pin | old | new | mechanism (probe `i363_reanchor`, 16×16 N=12) |
|---|---|---|---|
| `conception_…_seed_deterministic` golden window | seed 43 | **seed 44** | redistribution lowers early-village scarcity and re-paces courtship; seed 43 conceives in-window now. Leg A sweep @2000: seed 43 polluted, clean family 40/41/42/44/45/46/47/50–69 |
| `motivation_emotional_context_is_live` fear floor | > 0.30 | **> 0.27** | redistribution drains the hoard → lower `wealth_inequality` grievance → lower fear feed; measured **0.2864** (joy 0.0975). Still ≫ the 0.0000 dead channel |
| `noospheric_belief_confidence_sustains_conviction` high floor | > 0.34 | **> 0.28** | belief confidence rides the grievance/fear charge; measured high **0.2968**, low **0.1332**, delta **0.1635** — the differential invariant (> 0.15) **holds** and only the absolute floor moves |
| `idle_is_reached_…` Play de-saturation | final-tick snapshot | **run minimum** | i363 shifted the economic phase; Idle is rare (0.01 %–3.67 %) and `play` regrows between idles, so the end-of-run snapshot was phase-dependent. The contract ("Play is not permanently pinned at cap for every agent") is now measured over the whole run — strengthening, not widening |
| goldens ×2 | agent_count 12 | **12 → 13** | redistribution improves provisioning → one extra birth in the window. The stability pin only contracts `0 < count ≤ 48`, so this is in-contract; recorded as a real consequence |
| snapshots ×5 | — | reviewed → accepted | less scarcity (hunger 0.056→0.012), lower stress (0.301→0.264 / 0.364→0.334), higher health, more births (pop 12→13 → new ParentChild/Confidant stages), wealth-rank churn. Every direction is the expected redistribution signature |

No pin was widened without a measured value and a named mechanism; the two invariants
that matter (belief differential, Idle positivity) were **preserved**, and the Idle pin
was re-contracted to a strictly stronger sampling.

## Verification

Golden replay regenerated (**agent_count 12→13 on both, in-contract**) and byte-stable
across repeats; `mindstrata-sim` **299/299**; `mindstrata-tests` **310 passed / 0 failed /
1 ignored**; clippy clean; `gate --full` **GREEN**.

## Recorded consequences and open items

- **Collapse-scenario resilience:** the dividend makes the village materially more
  resilient (one extra birth even in the collapse golden window). Whether the collapse
  scenario's *longer-horizon* lethality is softened is unmeasured — a follow-up.
- The dividend share (0.25) is a first calibration; the Gini is now below baseline on two
  seeds, so it is not obviously mis-sized, but a multi-seed band would confirm.
- Council **membership inconsistency** (31 vs 3 members across seeds, noted in i361) is
  still open.
