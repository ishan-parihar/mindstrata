# i374 — the council dividend share is now a legitimacy-coupled law

**Status:** LANDED (behavioural, midpoint-neutral by construction) · **Scope:** one
constant (`COUNCIL_SURPLUS_DIVIDEND_SHARE = 0.25`, i363) → one law — the i373 audit's
first Class-B organic promotion.

## The law

```
s = s0 + k · (1 − legitimacy),  clamped to [0.05, 0.95]
defaults: s0 = 0.15, k = 0.25   (SimParameters::council_dividend_share_{s0,k})
```

The sim already measures the state the old constant was guessing at. A **nervous /
resented** council buys goodwill with patronage (spends more of its surplus); a
**secure** one hoards (spends less). The hoard becomes a state variable with feedback:
hoard → inequality grievance → legitimacy ↓ → payout ↑ → hoard ↓.

**Midpoint neutrality (§5):** `0.15 + 0.25·(1 − 0.6) = 0.25` — the law passes exactly
through the old constant at the §29.2 legitimacy equilibrium (0.6), so the behaviour at
the measured midpoint is unchanged and the coupling only bites when legitimacy moves.

## The probe (`i374_legit_dividend`, six-seed i365 family, 20K, N=48)

| Law | mean Gini | mean treasury | min coin |
|---|---|---|---|
| A: const 0.25 (control) | 0.4891 | 340.1 | 91.4 |
| B: s0 0.15, k 0.40 | 0.5035 | 272.4 | 89.0 |
| C: s0 0.15, k 0.80 | 0.4897 | **187.3** | 90.3 |

- **Distribution is not degraded**: laws B/C hold the i365 contract (mean Gini far
  below the pre-fix plateau 0.647–0.652); C's mean is statistically indistinguishable
  from the control's, and no agent approaches destitution on any law.
- **The treasury tightens as designed**: mean treasury falls monotonically with the
  coupling strength (340 → 272 → 187) — the council demonstrably spends more when less
  secure, which is the organic behaviour the constant could not express.
- The landed default is k = 0.25 (gentler than both probe legs, passing through the
  old constant at the equilibrium legitimacy) — the conservative first calibration.

## Sweep

- **3 insta snapshots** shifted (wealth ranks at 1e-3; long-horizon stress/health/
  trust at 1e-2 — the drained treasury enters circulation sooner). Reviewed → accepted.
- **Collapse golden regenerated**: metric_hash `2df0bc5d…` → `952ff7a5…`; agent_count
  **12 → 12** (mortality unchanged — the bite stays intact) and grain 0.5571 → 1.1262
  (redistribution keeps more grain in agents' hands through the famine, the expected
  signature). Re-anchored with this measurement.
- The calm golden is byte-identical (legitimacy sits at equilibrium, so the law
  reproduces the constant exactly — midpoint neutrality verified by the golden).
- sim **300/300**, tests **310/0/1**, clippy clean, `gate --full` GREEN.

## Remaining Class-B queue (unchanged)

Endogenous tax policy (council sets its own rate) → geography-derived marriage
distance → status-scaled patronage capacity → trait-derived disposition thresholds.
