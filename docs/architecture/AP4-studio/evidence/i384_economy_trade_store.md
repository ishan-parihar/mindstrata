# i384 — the economy trade read+write pair migrates to the dyadic store

**Status:** LANDED (behavioural) · **Scope:** `sim/economy.rs` — the §13.3 trade price's
trust read and the §19.5.J "trade builds trust" write. This was the **last
self-contained read+write pair on the legacy v1 `relationships` matrix**, and the
first landing of the v1→v2 migration's *writer* side after i376 (which made v1 a
projection and killed its saturation).

---

## 1. What the probe measured before anything was touched

`i384_economy_trade_store` — 3 worlds × 20 000 ticks, draining every
`TradeOccurred` and reading **both** stores on each trade's ordered pair
(buyer → seller).

| world | trades | v1 trust | v2 trust | Δ (v1−v2) | \|Δ price modifier\| | all-pair Δ | pairs ≥0.05 apart |
|---|---|---|---|---|---|---|---|
| village 16×16 N=12 s42 | 3 833 | 0.9174 | 0.8242 | **+0.0932** | 2.84% | +0.0162 | 3.0% |
| town 46×46 N=48 s42 | 16 353 | 0.8068 | 0.7036 | **+0.1032** | 3.12% | +0.0018 | 0.7% |
| town 46×46 N=48 s07 | 15 955 | 0.7892 | 0.6551 | **+0.1341** | 4.03% | +0.0036 | 0.9% |

Three things fall out:

1. **The divergence is concentrated on exactly the pairs that trade.** i376's
   population-wide offset is +0.002…+0.016 here, but on traded pairs it is
   **+0.09…+0.13** — 6–40×. The cause is measurable in this site's own numbers:
   it was itself a v1 *writer* at a flat `+0.02`/act, **2×** the dyadic store's
   `magnitude × volatility (0.5) × 0.02 = +0.0100`, applied to the pairs that
   trade repeatedly.
2. **The buyer was paying a price its own trust did not warrant** — 2.8–4.0%
   (mean 3.3%) below the correct modifier, on the busiest dyads.
3. Traded pairs are the *high-trust* subset (0.79–0.92 vs a 0.65–0.87
   population mean) — a positive feedback: trust selects the dyad, the dyad's
   trade builds trust at the legacy rate, the read rewards it.

## 2. The migration

```rust
// read  (was: rel_pos(i, seller) → relationships[p].trust)
let trust = self.relationship_v2_between(i, seller)
    .map_or(Fixed::from_f64(0.5), |r| r.trust);

// write (was: rel.trust = (rel.trust + 0.02).clamp_01()  on the v1 row)
self.agents[i].relationship_v2s[pos].record_positive(tick, Fixed::ONE);
```

- `map_or(0.5, …)` reproduces the legacy default exactly, including the
  self-trade case (`relationship_v2_between` returns `None` for `from == to`).
- The write carries a `i != seller` guard for the same reason (`rel_pos` found no
  self-edge; `relationship_v2_pos` requires `a != b`).
- **The dyadic schedule is now the contract.** One completed trade is one
  positive act: trust **+0.0100** (halved from the legacy +0.0200 at the default
  0.5 volatility), and the *whole* dyadic state moves — affection, gratitude,
  intimacy, commitment, a resentment damp — where the legacy write carried a
  bare scalar. `interaction_count` now advances per trade (it feeds the §10.2
  stage ladder), which is why the stage distribution in §4 shifted *up*.

## 3. Post-migration measurement (same probe, same worlds)

| world | trades | v1 | v2 | Δ | \|Δ modifier\| | all-pair Δ | ≥0.05 apart |
|---|---|---|---|---|---|---|---|
| village s42 | 3 886 | 0.9707 | 0.9771 | **−0.0064** | **0.34%** | +0.0020 | 1.5% |
| town s42 | 16 811 | 0.9227 | 0.9318 | **−0.0091** | **0.52%** | +0.0008 | 0.4% |
| town s07 | 16 393 | 0.9528 | 0.9589 | **−0.0060** | **0.36%** | +0.0012 | 0.5% |

- The read-source delta collapses **2.84–4.03% → 0.34–0.52%** (an 8× cut) and the
  two stores agree to within **1%** on traded pairs.
- **The write is live and strong on the dyadic store**: traded-pair v2 trust rose
  0.8242 → 0.9771 (village), 0.7036 → 0.9318, 0.6551 → 0.9589.
- Economy contracts moved: trades **+1.4% / +2.8% / +2.7%**, Gini
  **0.2531 → 0.3048 / 0.5171 → 0.5304 / 0.5336 → 0.5570**, avg coin up in all
  three worlds, destitution still 0.

## 4. Blast radius (attributed, not assumed)

**Goldens re-anchored** (both scenarios; `agent_count` 12 → 12 both — mortality
identical):

| baseline | metric_hash | grain | water |
|---|---|---|---|
| riverford_minor 1000t | `4b8b8c8b0d54cfa6` → `ac9012d2036442c3` | 83.07 → 83.07 | 1980.1222 → 1980.1222 |
| collapse 4320t | `2f80d7a95d3e5ecd` → `47c91bc5ba9edcf0` | 0.3292 → 1.0251 | 1141.6559 → 1140.8211 |

**4 snapshots reviewed then accepted** (all signed as the mechanism predicts):
`metrics_500_ticks` trust 0.64019 → 0.64071; `metrics_2000_ticks` trust 0.69116 →
0.69460; `long_horizon_surface_10000_ticks` trust 0.79917 → 0.82905, quality
0.69620 → 0.73235, stress 0.36202 → 0.34994, event_count 125 425 → 124 716,
**stage ladder up** (Confidant 1 → 4, CloseFriend 6 → 7, Ally 62 → 65, Unnoticed
15 → 11); `relationship_stage_distribution_2000_ticks` same shape.

**Faction census re-run** (`i382_legitimacy_channel`, 15 worlds): formations
**12 → 14** — every world in the corpus now organizes at least one faction except
calm-town-42. The producer strengthened rather than starved.

**Three pins re-anchored, each with its own sweep:**

1. `neural_like_prediction_error_folds_are_live_and_directional` — **the seed
   history was never the fault; the manipulation was inert.** Grain was zeroed
   once at t=0, production/foraging refilled it, and by the 5000-tick sample the
   two arms were the same world: **starvation raised mean hunger on 1 of 12
   seeds**. The measured "differential" was trajectory noise (pinned set wins
   1/6, best +0.0220; widened 12-seed family 4/12 = chance) — which is why the
   sign flipped in all five previous re-anchors. Fix = **revive the producer**
   (§2.3): the set point is now held every tick, and the contract lands with
   margin — hunger higher under scarcity on **6/6** seeds (Δ 0.0029…0.0348), wins
   **5/6**, best Δconf **+0.0995**. The dead single-seed setup (a 5000-tick sim
   whose result was never read) was deleted with it.
2. `faction_attachment_styles_scale_upward_and_dynamics_run` — the i306 sweep
   (12 pestilence seeds × 30K, the test's own contract at the first live
   instant) still finds **7/12 valid**: the seed moved, the producer did not.
   Re-anchored seed 1 → **55** (first live @4000, earliest in the family = the
   longest window of daily dynamics; reg 1, non-Secure 1, supplies 0.6970,
   cohesion 0.6447, style clause true).
3. `faction_trigger_reads_the_councils_own_mandate` — **re-contracted from a
   ratio to a difference.** i384's price/Gini shift raised the *shared*
   grievance-driven component both arms carry (control 0.00097 → 0.00557, fall
   0.0147 → 0.02116), compressing the ratio 15.2× → 3.8× while the
   legitimacy-attributable delta did not shrink (0.0137 → 0.0156, +14%). A ratio
   against a moving control is the fragile quantity; the invariant is the
   difference (`fall > stable && fall − stable > 0.003`).

**New runnable pin:** `trade_price_reads_the_dyadic_store_and_writes_it` —
crafts a pair whose stores disagree, asserts the price reads the dyadic value
(0.1, not the v1 0.9), and asserts the trade write moves the dyadic row
(+0.0100 trust, affection up, interaction_count +1) while leaving v1 untouched.

## 5. Method lessons recorded

- **A gain-mismatch flag can be a *migration* flag, not a rescale flag.** i373's
  audit filed the v1 gains as a Class-3a rescale (`~10× the dyadic gain`). At the
  economy site the honest repair was to delete the v1 write entirely — the
  numbers agreed to 2× only because this site's flat +0.02 sits nearer the dyadic
  gain than the interaction schedule does. **Read the consumers before choosing
  between rescaling and migrating**: rescaling a writer that is queued for
  deletion buys nothing.
- **A pin can decay for five iterations because the EXPERIMENT died, not the
  channel.** Every previous PE re-anchor widened or re-seeded a measurement whose
  independent variable had stopped binding. Doctrine gains a fourth question for
  the audit ladder: after the gate census and the store probe, **check that the
  manipulation is live at the measurement horizon** (`|Δmanipulated state| > 0` on
  a majority of the family) before re-pinning anything about the response.

## 6. Verification

`sim 301/301` (+1 pin), `integration 312/0/1`, `insta` none to review, clippy
clean, `gate --full` **GREEN**. No RNG stream was added or reordered: the site
draws no randomness, so determinism holds by construction.
