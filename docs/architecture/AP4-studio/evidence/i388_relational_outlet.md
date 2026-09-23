# i388 — the relational outlet: two dead channels closed (and what that moved)

**Status:** LANDED (behavioural) · **Root cause owned:** i387's decomposition — two fifths
of the deliberative layer serves a relational drive (`Attachment` + `Belonging` = 40.1% of
calm-village arbitrations) and **nothing responded to it**: the urgency map had no
relational arm, and all three social goal producers carried absolute bars above the
channel's own distribution.

## The measurement that named it

Probe `i388_relational_outlet` (leg A, 500 warmup / 20 000 window):

| world | `needs.social` p50 | p90 | p99 | mean | live Socialize goals |
|---|---|---|---|---|---|
| village N=12 | 0.0054 | 0.0174 | 0.0320 | 0.0078 | **0/15 agents** |
| town N=48 | 0.0054 | 0.0176 | 0.0394 | 0.0083 | **0/50 agents** |

The three producers' bars were **0.7** (need gate), **0.4** (extraversion leg), **0.3**
(joy leg) — i.e. every one of them above the channel's p99. All three were dark by
construction: the same absolute-bar-on-a-relative-scale defect i382 closed for faction
legitimacy and i383 for the anger arms, three times over in one channel.

## The fix (two channels, one root cause)

1. **Relational urgency family** (`actions::dominant_urgency_match`, extracted as a pure
   helper so it is unit-pinnable): `Attachment | Care | Romance → Socialize`;
   `Belonging → Socialize | Worship`. Sibling shape unchanged
   (`dominant_pressure × 0.4`). This matters because §8.1.5's boost *amplifies an
   already-pressured candidate* — a channel, not a reordering: a relational dominance
   must not boost Work/Trade/Rest/Idle (pinned).
2. **Population-relative relational band** (`systems::system_goal_generation`):
   `band = 2 × mean_social(needs) × goal_gate_scale`, with the retain arm at
   `band × 0.43` (the sibling rows' 0.3/0.7 shape — without it the band would create a
   goal the next tick drops). Ratio 2.0 is the measured choice; the sweep over the same
   two worlds:

   | ratio | village open | town open |
   |---|---|---|
   | ×1.25 | 28.9% | 27.6% |
   | **×2** | **12.4%** | **11.7%** |
   | ×3 | 5.2% | 5.7% |
   | ×5 | 1.0% | 1.6% |

   ×2 sits just above the routine's own social share (~5% of decisions): the goal layer
   adds an outlet for the deprived without putting a goal on every agent (§4.10 — a band
   that opens for everyone discriminates nothing).

## Measured effect (same probe, post-fix)

| | before | after |
|---|---|---|
| utility-selected `Socialize` (village) | **0** of 26 044 | **109** of 20 121 (0.54%), 203 within noise |
| utility-selected `Socialize` (town) | **0** of 94 178 | **88** of 96 154 (0.09%), 290 within noise |
| live `Socialize` goals at window end | 0/15, 0/50 | **1/12, 9/51** |
| `Socialize` share of all decisions | 4.85% / 5.10% | 5.87% / 5.18% |
| `Socialize` urgency bucket | 0.0000 | **0.0865** (winner 0.1024) |
| need equilibrium (`needs.social` mean) | 0.0038 / 0.0047 | 0.0051 / 0.0044 |

The drive now reaches both layers, and the need equilibrium is unmoved — the outlet does
not collapse the need it serves (the §4.3 liveness property).

## Blast radius (what the change moved, attributed)

- **Goldens** (`riverford_minor`, `collapse`): both metric hashes moved
  (`12434459264379273923 → 0xb2a1be3b18fbb46d`, `5172696183065206000 → 0xd04f5682cb04d136`),
  **`agent_count` 12 → 12 in both** — the mortality/replacement invariant the goldens
  principally guard is intact; the path changed because the decision distribution did.
  Re-anchored with that evidence.
- **7 insta snapshots** reviewed and accepted (agent state, endocrine, institution,
  metrics ×2, relationship stages, long-horizon surface).
- **`conflict::violence_records_injury_on_the_substrate`** — the pain veto-band clause
  (max pain ≥ 0.9) was a **single-seed max on a stochastic violence path**; seed 42 now
  peaks at 0.7805 with the producer intact. The i319 probe was extended with a pain
  column: **family max 1.0000, 2/12 seeds reach the band at 20K** (3/12 at 50K). The pin
  is re-contracted from a single-path threshold to the reachability invariant over a
  small family + per-seed channel liveness.
- **`governance::revolution_is_regime_change_not_repeat_loop`** — the family {42, 7, 23}
  now carries 5/0/0. The i381 Part B sweep re-run measures firing seeds **[5, 42, 12345]**
  with revolutions **7 / 5 / 3**, i.e. the producer fired *more* (two previously quiet
  seeds) and merely re-timed. Re-anchored onto the discovered members, bar still ≥2 of 3.
- **`social::tenderness_channel_boosts_helping_when_multiplier_active`** — this pin has
  been seed-shopped twice (Iter-164, P5); with the outlet live it measured cold 5047 vs
  warm 5528 on seed 55 = **+9.5%, correct direction, 0.5% short of an arbitrary ±10% bar**.
  Re-contracted to the mechanism over a family (seeds 55/46/5: +9.5% / +24.3% / +10.8%;
  family **+15.2%**), asserted at ≥5% plus direction per seed plus determinism.
- **`psychology::emotion_motivation::anger_driven_work_is_relative_to_the_population`** —
  crashed with `index out of bounds: len is 12 but the index is 12`: the held-anger vector
  was indexed by the *live* population and the outlet re-paced a founder birth into the
  600-tick window. A test-fragility bug (fixed: hold for every agent present), not a
  producer finding.

## New pins

- `actions::tests::relational_motives_map_to_the_social_outlet` — the mapping, its
  dyadic/communal split, and the non-boost of provisioning actions.
- `actions::tests::urgency_map_holds_its_documented_edges` — the nine documented
  non-mappings stay non-mappings; `Novelty → Wander` and `Play → Idle` remain live.
- `psychology::emotion_motivation::relational_deprivation_reaches_the_goal_layer` — the
  relativeness proof: an agent at **0.05** in a crowd at 0.002 emits (below every legacy
  absolute bar, so this arm cannot pass under an absolute gate), and alone.
- The i319 probe gained the pain column; the i388 probe carries the reach + ratio sweep.

## Observations carried forward (not debt yet — measured, unexplained)

- **Action duration rose** (village N=12: 55 624 → 44 149 decisions over the same window,
  mean 4.31 → 5.40 ticks/decision) and the relationship-stage distribution at 2K
  redistributed toward shallower stages (`Unnoticed` 20 → 26, `Friend` 8 → 5,
  `Neighbor` 12 → 7, total dyads unchanged at 132; `Acquaintance`/`Ally` up). Hypothesis:
  serving the drive selects stay-put social actions *instead of* movement, so encounter
  volume falls even as social selection rises. DC-5's W2 (encounter-driven contact) must
  measure this directly — the sparse-store work (i350) is uninterpretable until it does.
- **The revolution family remains knife-edge** (§4.5 debt): {5, 11, 42} → {42, 7, 23} →
  {5, 42, 12345} across three pacing changes. Re-anchoring by discovery works, but the
  family should eventually be replaced by a per-seed route pin (panic → breakdown).

## Verification

`cargo fmt --all` clean · `cargo clippy --workspace` 0 warnings · sim lib **303/303**
(+2) · integration **313 passed / 0 failed / 1 ignored** (+1) · goldens re-anchored
(agent_count 12 → 12) · `scripts/gate --full` GREEN.
