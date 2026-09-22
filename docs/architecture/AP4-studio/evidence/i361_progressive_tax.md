# Iteration 361 — the progressive tax is REFUTED; the fault is a hoarding treasury

**Status:** MEASUREMENT + DECISION (probe + a reverted behavioural change + the true root
cause) · **Item owned:** the plan's B-item follow-up scoped by i358 — "make the tax
progressive above the membership median".

## What was tried (and why it was reverted)

i358 concluded that `Institution::collect_taxes` was **proportional** (`wealth × rate`),
hence scale-invariant, hence provably incapable of bending the ~50%-of-all-coin tail —
and scoped the fix as a **surcharge above the membership median**. i361 implemented
exactly that (`TAX_PROGRESSIVE_SURCHARGE = 0.05`, excess over the membership median,
quantized once, zero-at-or-below the median so single-member institutions are
byte-identical), then ran the **identical** i358 harness (46×46, N=48, seed 42/7, 50K).

It made the target **worse**:

| seed | i358 baseline Gini | i361 Gini | i358 top-10% | i361 top-10% | i358 bottom-half | i361 bottom-half |
|---|---|---|---|---|---|---|
| 42 | 0.647 | **0.7046** | 52% | 49.7% | 10.3% | **3.1%** |
| 7 | 0.652 | **0.6714** | 54% | 56.6% | 8.8% | **6.8%** |

A steeper tax *raised* the Gini and shrank the bottom half's share. Per doctrine §4.3 a
change that worsens the measured pathology must not be pinned, so the behavioural change
was **reverted** (byte-identical to pre-i361); the probe and this diagnosis are kept,
because the *reason* it failed is more valuable than the attempt.

## The real root cause: the council treasury hoards

Instrumenting the institutions at 50K exposed it immediately:

| seed | institution | members | treasury |
|---|---|---|---|
| 42 | **Council** | **31** | **163 602.8** |
| 42 | Market | 1 | 917.1 |
| 42 | Temple | 1 | 918.8 |
| 7 | **Council** | **3** | 625.7 |

The council extracts tax every centum but its only outflow is the i261 **poor relief**,
sized deliberately as "**≤0.5 coins per recipient per cycle** … to stay inside the
calibrated grievance/legitimacy envelope". Against a 5% tax on a village whose wealth
reaches the tens of thousands, that drip is a rounding error — so **163K coins (seed 42)
sit idle in the treasury**, extracted from circulation and redistributed to nobody. The
Market treasury, by contrast, pays a **progressive dividend** (surplus × 0.5, weighted
`1/(1+coin)`, i186) and never accumulates.

**Therefore:** any *increase* in extraction — progressive or not — drains the economy
into an untouchable treasury and **widens** the gap, exactly as measured. i358's diagnosis
("the tax is proportional") was **incomplete**: proportionality is a real inertness, but
the dominant fault is that the collected coin is **hoarded, not spent**. The progressive
surcharge fails because it feeds the wrong sink.

## Corrected queued fix (recorded, not rushed)

The tail fix is **not** a steeper tax — it is **treasury surplus redistribution**: the
council should spend its surplus the way the market already does (a progressive dividend
above a reserve), instead of a capped drip. That is behavioural and sweep-carrying (it
moves the coin supply, the `wealth_inequality` grievance channel, and therefore
legitimacy/faction dynamics), so it needs its own probe + classified re-anchor sweep —
and specifically it must be sized against the i186 note's warning that an *equal* dividend
preserves the wealth ratio (Gini barely moves), so the weighting must be wealth-inverse.

Also recorded: council **membership is inconsistent** (31 members in one seed, 3 in
another) — a second latent driver of who is taxed at all.

## Verification

Behavioural change **reverted**; probe + docs only. Golden byte-identical, sim/integration
suites green, `gate --full` GREEN. No re-anchors.
