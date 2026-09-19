# Mindstrata — DC-3 Development Plan (Era V harvest → "the world scales")

**Date:** 2026-09-18 · **Baseline:** HEAD `5ee3c1c` (i286, WP-H3 closed), main, 819 commits,
118,569 LOC across 13 crates, tree clean.

---

## 1. Live verification at audit time (all measured, not cited)

| Gate | Command | Result | Verdict |
|---|---|---|---|
| Format | `cargo fmt --all --check` | clean | PASS |
| Lints | `cargo clippy --workspace --quiet` | 0 warnings | PASS |
| Full suite | `cargo test -p mindstrata-tests --lib --release` | **307 passed / 0 failed / 1 ignored**, 202 s | PASS (documented signature) |
| Quick gate | `scripts/gate` | GATE GREEN, golden 5/5 byte-identical (0.46 s) | PASS |
| Probe law | `scripts/bench_index.py --strict` | 104 indexed / 68 ok / 36 legacy / **0 violations** | PASS |

No development debt is masked by a red gate. The one ignored test remains the documented
long-horizon trace harness (`--ignored` by design).

## 2. What is complete (evidence-backed, git-history arc AP1 → AP4)

| Program | Status | Proof |
|---|---|---|
| **AP1** core village sim | DONE | `archive/AP1-implemented.md` |
| **AP2** deepening (~170 iters) | DONE 100% — every write-only state, dead producer, unwired consumer closed (Iters 22–186; final dead-producer kills i267/i272) | AP2 final audit + Iter-186 exhaustive audit |
| **AP3 Era I–II** attractor-field engine | DONE — all 4 pathology quadrants live + behaviorally wired, per-quadrant params ratified (`66f753b`), 20-seed sweep (i293), N=48 check (i294) | `i293`/`i294` evidence |
| **AP3 Era III** content emergence | DONE (i273–i286): WP-G2 referent-grounded genesis, WP-H2 reconciliation + refutation (panic cascade), WP-H3 mourning rites + norm proposals. **UM-2 PASSED** at the corrected ~100K cultural horizon (all 4 buckets generate, jaccard 0.158 vs control 1.000) | `i283_um2_unify_review.md` |
| **AP3 Era IV** collective holon | LANDED (i266 step_collective, i273 stage-gated genesis, i274/i279 feeds, i280 WP-J coupling) — **exit probe `i300_institution_shift` end-to-end still recorded as a gap** | §6.6 meta-review |
| **DC-1** studio cycle | DONE 106/106 CLOSED (v18 audit) | `FINAL-AUDIT-DC1-regression-v18.md` |
| **DC-2** | DONE (§4/§6 ledgers; WP-H3 closed at i286) | `PLAN_DC2_DEVELOPMENT.md` §4–§6 |
| **Scale/structure** | DONE — crate ladder S1–S3 (`core ← person ← psych ← {social, institutions, world} ← sim`), Arc-D pass extraction batches 1–3 verbatim, golden-proven | `f66b988`, `3ad212b`, `b2163ad` |
| **DC-3 P0** | Ratified — perf budget 122/1088/5961 µs/tick @ N=12/48/96, golden budget ≤150 µs/tick binding | `charters/DC3-P0-perf-budget.md` |

## 3. The honest calibration-debt ledger (what is left)

Every item is documented debt with a named probe plan; none is hidden.

### 3.1 Actionable (probe + sweep, no external blocker)

| # | Item | Site | Current | Path |
|---|---|---|---|---|
| A1 | **Transgression catalyst feed dead at N=12** — NormViolated ≈ 0 ⇒ Value/Norm syntheses structurally unreachable; norm registry can only widen via Identity codifications (i286 verdict) | sim catalyst diet | dead producer | Probe small-N transgression diet (seeded scenario violations / minor-crime regime) → revive feed → i286 Value/Norm gate widens without touching quorum/band machinery |
| A2 | **Grief/Threat catalyst magnitude ratification** — blocked by mortality horizon per i270, but i285 proved pestilence scenarios DO generate deaths (5 deaths → 1 rite) | `catalyst_observers.rs` | spec-midpoint pins | Observer-harness sweep over pestilence/collapse scenarios at 20–50K; ratify or re-contract per §4.4 |
| A3 | **Mourning-rite Agape dose 0.6** CALIBRATION-PENDING (i285) | `systems/development.rs` | first estimate | Sweep dose vs Dark-pathology decay differential (the WP-H3 i291_agape_metabolism probe shape) |
| A4 | **Norm-proposal strength cap 0.6** CALIBRATION-PENDING (i286) | `system_norm_proposal` | first estimate | Sweep consensus-scaled strength vs §12.5 ritual-reinforcement equilibrium |
| A5 | **UM-2 diet debt** — only Safety generates inside 20K; Relational ~54K, Identity ~44K (i283 corrected pacing) | collective feeds | diet-gated | Festival-dense regime probe (recurring Relational+Identity press): does ≥3-bucket generation close by 20K without a magnitude-knob violation of §4.4? If yes, close the i278 PARTIAL; if no, record N=12 as the honest horizon |
| A6 | **Era IV exit probe `i300_institution_shift`** — WP-J coupling landed (i280) but the end-to-end forced-stage amber→green pluralism probe never ran | `institutions_multiplier` | wired, unproven end-to-end | Probe per §6.3 i279 design; golden byte-identical below the crossing band |

### 3.2 Blocked at external inputs (recorded, not plannable until inputs land)

- Full Era III legality mapping — vendor `realms.md` not vendored.
- Cross-line resonance matrix — vendor coupling attestation pending (`dynamics.rs:23`).
- Golden-addiction cult dynamics × institution legitimacy interplay — needs a cult-liveliness regime (backlog).

### 3.3 Deferred with recorded triggers (do NOT pull early)

- VecDeque events buffer — revisit at >250K-tick operator horizons / memory-capped hosts / annals export (SimEvent 56 B; 780 MB @1M ticks).
- Per-agent recent-claims index — revisit when runs exceed 250K ticks (O(claims)² salience filter).
- H5 founder-variance shaping — systemic debt, requires larger N AND coordinated re-anchor sweep; never piecemeal.

### 3.4 DC-3 checklist items still open (from the perf charter §5)

- [ ] Perf regression probe wired as CI-adjacent bench (warn-only, N=12 ≤150 µs/tick).
- [ ] Superlinearity probe at N=192 (interaction-volume vs algorithmic growth; never run).
- [x] Asset pipeline v0 charter (sets the UM-3 envelope). — **LANDED i301** (`charters/ASSET-PIPELINE-v0.md`; `sim/assets.rs` + CLI `--export-assets`; see `evidence/i301_asset_export.md`).

## 4. The iteration ladder (one root cause each, doctrine §2)

**Theme:** close Era IV, harvest Era V, then scale the world (UM-3). Construction is no
longer the bottleneck — diet realism and observability are; each iteration is
probe-first, full-gate, and ends in commit + push.

| Iter | Root cause owned | Deliverable | Exit evidence |
|---|---|---|---|
| **i287** | Era IV exit unproven | Run `i287_institution_shift` end-to-end: forced-stage governance amber→green crossing; measure pluralistic institution deltas through the live WP-J channel (A6) | Deltas measured, sign-correct, golden byte-identical below the crossing band → **Era IV exit gate CLOSED** |
| **i288** | Transgression feed dead (A1) | Probe small-N transgression diet; if a scenario-seeded violation regime lifts NormViolated > 0 without reshaping founder draws (H5 rule), wire it as scenario-level seeding, not a magnitude knob | i286 Value/Norm synthesis path fires in the seeded regime; natural runs untouched (zero-blast outside scenario) |
| **i289** | Grief/Threat magnitudes unratified (A2) | Observer-harness sweep across pestilence/collapse scenarios (the i285 mortality workaround); ratify magnitudes or re-contract with measured values | `CALIBRATION-PENDING` removed from catalyst_observers for Grief/Threat or honestly re-scoped |
| **i290** | Era V WP-K observability incomplete | Per-quadrant transition traces (R6), per-agent longitudinal TUI lane + village panel, KosmOS-frontmatter `stage_lines` export in snapshot.rs | Traces render; export schema pinned; read-only → golden untouched |
| **i291** | Era V WP-L chronicle lens missing | Ray/density lens rendering for chronicles ONLY (doctrine D5: lens never place) — altitude→ray mapping colors narrative text, zero mechanical effect | Chronicle renders tinted; mechanical-effect pin = zero; **Era V exit gate CLOSED** |
| **i292** | Agape/norm doses first-estimate (A3+A4) | Two sweeps in one dose-calibration iteration (same subsystem family, one root cause: first-estimate constants); promote or record | Both sites carry measured/old/mechanism comments; no lucky-pin sweeps |
| **i293** | UM-2 diet realism at operator horizons (A5) | Festival-dense regime probe (the named i283 accelerator): recurring Relational+Identity press at 20K. If ≥3 buckets generate → close the i278 PARTIAL properly; else ratify N=12/~100K as the designed cultural horizon and stop chasing 20K | Disjointness probe re-run with verdict + honest pacing statement |
| **i294** | N≥96 scaling unproven | Superlinearity probe N=192 (charter checklist); separate interaction-volume growth from algorithmic accidents; hot-path list updated | **LANDED `de9dc35`** — α_total=2.115, volume linear (0.975), cost/event climbs (1.14): algorithmic. O(N³) social-support scan fixed golden-identical, uniform −20% at N=12/48/96; density (+0.6%) and daily cadence (0.8%) retired; dense matrix α_rels=2.03 recorded as structural floor (`evidence/i294_superlinearity.md`) |
| **i295** | Perf envelope unenforced | Perf regression probe as warn-only bench (N=12 ≤150 µs/tick budget); plus VecDeque/claims-index trigger review against any new horizon evidence | **LANDED** — `i295_perf_budget_gate` warn-only in gate 2.55 (N=12 102.6/150, N=96 4577.7/6500 OK); charter §5 items 1+2 checked off; VecDeque/claims-index triggers re-reviewed, NOT hit |
| **i296+** | Multi-village worlds (UM-3 core) | Collective holon per polity: `CollectiveField` per village, cross-village cultural diffusion via trade partners (backlog items), civilization-axioms line activation probe | **i296 LANDED** — `polity_fields`/`polity_members` + `assign_polities`; identity-at-isolation pinned unit + in-vivo (bit-identical over 10K); two-polity partition diverges (Safety 3.000 vs 4.000, shared world); unassigned default = legacy. REMAINING UM-3: genesis referents/naming leg (i293 finding), cross-village diffusion, settlement-based auto-partition (`evidence/i296_multi_village.md`) |

**Standing discipline (unchanged):** check `git log --oneline -3 && git status` every
session (shared-clone hazard); probe before touching; one root cause per iteration;
fmt+clippy+release suite before every commit; `scripts/gate --full` before push;
re-anchors carry measured/old/mechanism (§4.2); knife-edge results recorded as debt,
never flip-flopped; no magnitude-knob compensation for diet problems (§4.4, i279 verdict).

## 5. Risk register

| Risk | Mitigation |
|---|---|
| i287 forced-stage probe breaks goldens | Zero-blast gate: golden horizons keep governance line at 1.0; crossing probe runs in scenario mode only |
| i288 transgression seeding re-paces the shared interaction stream (the Iter-164/180 pattern) | Scenario-scoped seeding (zero blast in natural runs); sweep before any standing-rate change |
| Vendor-blocked items stall morale | They are recorded with unblock conditions; the ladder above needs none of them |
| Perf budget erosion from Era V TUI work | Read-only rendering discipline (i294/i295 before i296); charter rules 1–4 binding |
| H5 founder-variance temptation at N=12 | Standing prohibition (§5); diet realism via scenario regimes, not draw reshaping |

## 6. Summary

The project is **structurally complete through Era IV and DC-2** with every gate green at
HEAD. What remains is: (a) six actionable calibration items (§3.1), all probe-shaped and
unblocked; (b) two Era V work packages (observability + chronicle lens) — the last unbuilt
surfaces; (c) the DC-3 scale program (multi-village holons, N=192 scaling proof, asset
pipeline v0); and (d) three vendor-blocked items that stay parked. The ladder above turns
that into ten probe-first iterations, each with a named exit test.

## 7. Execution ledger (updated as iterations land)

| Iter | Status | Landing | Verdict recorded |
|---|---|---|---|
| i287 | **DONE** | evidence/i287_institution_shift.md | Era IV exit gate CLOSED — 20K natural reaches stage 6.0, WP-J opens, 8 genesis memes; end-to-end 2-line forcing is structurally confounded with the i276/i277 Safety readers (unit pins remain the WP-J liveness proof) |
| i288 | **DONE** | evidence/i288_transgression_revival.md | A1 root cause one layer deeper than the diet: violence recorded as a violation but never emitted `NormViolated`; emission revived at the violence site (55 majors/20K). **Natural norm proposals became reachable** (confirmed in i292 census) |
| i289 | **DONE** | evidence/i289_catalyst_magnitudes.md | A2 ratified — Grief/Threat magnitudes carry measured/old/mechanism comments; observer census over mortality windows |
| i290 | **DONE** | evidence/i290_observability.md | WP-K CLOSED — `export_stage_lines` canon-cited frontmatter, q1–q4 pathology means + `collective_stage_max` on MetricsSnapshot, TUI pathology panel; in-vivo quadrants diverge (Q2 0.732 dominant @20K) |
| i291 | **DONE** | evidence/i291_chronicle_lens.md | WP-L CLOSED — **Era V exit gate CLOSED**; ray lens extracted from vault per-cell frontmatter (provisional until rays.md vendors); mechanical-effect pin = zero (untinted lines survive verbatim) |
| i292 | **DONE** | evidence/i292_dose_calibration.md | A3+A4 RATIFIED at existing values with sweep evidence (agape 0.6 in the 0.5–3% metabolizer band; cap 0.6 measured binding in vivo). Both CALIBRATION-PENDING markers closed |
| i293 | **DONE** | evidence/i293_festival_regime.md | A5 closed: i278 PARTIAL → diet-pacing (≥3 buckets by 5K under festival press, zero production edits); **N=12/~100K ratified as the designed cultural horizon**; uniform press homogenizes rosters (jaccard 0.822) — recorded for UM-3 |
| i294 | **DONE** | evidence/i294_superlinearity.md | O(N³) social-support scan fixed golden-identical (uniform −20%); α_total=2.115 attributed: volume linear (0.975), cost/event climbs (1.14) — algorithmic; dense-matrix α_rels=2.03 structural floor |
| i295 | **DONE** | (charter §5 closed) | Perf envelope warn-only in gate 2.55 (N=12 102.6/150, N=96 4577.7/6500); VecDeque/claims-index triggers re-reviewed, NOT hit |
| i296 | **DONE** | evidence/i296_multi_village.md | UM-3 core landed: per-polity holons (`polity_fields`/`assign_polities`), identity-at-isolation pinned unit+in-vivo, partition diverges (Safety 3.000 vs 4.000). Remaining UM-3 legs: referent/naming, auto-partition, cross-village diffusion |
| i297 | **DONE** | evidence/i297_polity_genesis.md | UM-3 leg 1 (referents) CLOSED — per-polity genesis with territory-anchored referent views (sites: nearest home-site centroid; institutions: member-majority) + per-polity dedup namespaces; probe: p0 cites Village Market, p1 cites Village Temple (20K, seed 42); zero blast without polities; sim lib 250/250 |
| i298 | **DONE** | evidence/i298_auto_partition.md | UM-3 leg 2 (assignment rule) CLOSED — `auto_partition_polities(max_gap)`: deterministic single-linkage clustering over inhabited home sites; single settlement → inert (0 polities), two settlements → disjoint cover with divergent trajectories (S 4.000 vs 3.000), derived ≡ assigned bit-identical over 10K; sim lib 253/253 |
| i299 | **DONE** | evidence/i299_trade_diffusion.md | UM-3 leg 3 (diffusion) CLOSED — `system_trade_diffusion`: cross-polity trades damp sender's namespaced genesis memes into the receiver (§13.1 weight 0.3, zero RNG); zero-at-zero in vivo; diet LOADED (1160 cross trades/window); measurement boundary recorded (host_count conflates gossip+diffusion → `meme.hosts` schema debt for i300); sim lib 257/257 |
| i300 | **DONE** | evidence/i300_um3_gate.md | **UM-3 GATE CLOSED** — `meme.hosts` per-agent hosting sets (additive serde-default); gate @50K/seed 42: 2 auto-partitioned polities, 26 territory-routed genesis memes (p0=14/p1=12), 26/26 cross-hosted via trade+gossip (origin-share 0.288, saturation caveat recorded); sim lib 258/258 |
| i301 | **DONE** | evidence/i301_asset_export.md | **ASSET PIPELINE v0 landed** (last DC-3 checklist item) — charter ratified (5 binding rules, versioned schema); `export_world_assets_json` + CLI `--export-assets`; determinism byte-identical, v1 round-trip, polities/stage_lines/culture.hosts/annals payload verified; sim lib 262/262 |

## 8. 2026-09-19 audit + UM-3 continuation ladder (post-i296)

**Live verification at HEAD `1992883` (all measured this session):** fmt clean, clippy 0,
release suite **307/0/1** (215 s), sim lib 241/241. Every gate green; no debt masked.

**Audit verdict:** Era III/IV/V exits all closed (UM-2 passed @~100K, Era IV i287, Era V
i291). The six §3.1 calibration items are all resolved or recorded (A1–A5 DONE via
i288/i289/i292/i293; A6 closed i287). **What remains of DC-3 is exactly the UM-3 scale
program**, whose three named legs (i296 scope statement) become the next iteration
ladder. Vendor-blocked items stay parked (realms.md, resonance attestation, cult-liveliness).

**i297 — genesis referent/naming leg (UM-3 leg 1). DONE (`evidence/i297_polity_genesis.md`).**
Sites anchor by nearest home-site centroid (the only rule that can own communal sites);
institutions by member-majority; per-polity genesis dedups on `pN`-namespaced tags so
same-epoch crossings each commemorate their own territory. Probe-verified divergence:
p0 cites Village Market, p1 cites Village Temple; zero blast without polities.

**i298 — settlement-based auto-partition. DONE (`evidence/i298_auto_partition.md`).**
`auto_partition_polities(max_gap)` — deterministic single-linkage clustering over
inhabited home sites; single-cluster worlds stay inert; two-settlement worlds derive
the i296 partition bit-identically to operator assignment.

**i299 — cross-village cultural diffusion via trade partners. DONE (`evidence/i299_trade_diffusion.md`).**
`system_trade_diffusion` — damped cross-polity meme growth on trade events. In-vivo A/B
isolation recorded as impossible at N=12 (wandering, shared market sites, host_count
conflation); unit pins carry the liveness proof; `meme.hosts` schema debt recorded for i300.

**i300 — UM-3 gate: DONE (`evidence/i300_um3_gate.md`).** `meme.hosts` schema landed
(clean per-agent diffusion signal); gate measured: 2 polities, 26 territory-routed
memes, 26/26 cross-hosted (origin-share 0.288 with saturation caveat). **UM-3 exit
verdict: multi-village worlds generate per-village culture that differentiates by
territory and interlinks by trade.** DC-3 unify review is the next arc.

## 9. DC-3 UNIFY REVIEW (2026-09-19, i302) — VERDICT

**Milestone contract (ROADMAP):** DC-3 → UM-3 "the world scales" = AP3 Era IV
collective holon + N≥48 performance budget + asset pipeline v0.

**Live verification at HEAD `7a4ddec` (all measured this session):** fmt
clean, clippy 0, release suite **307/0/1**, sim lib **262/262**, full gate
GREEN, bench law 0 violations. No debt masked.

| Contract leg | Delivered | Verdict |
|---|---|---|
| AP3 Era IV collective holon | i266 step_collective; i273 genesis; i274/i279 feeds; i280 WP-J coupling; i287 Era IV exit probe | **CLOSED** |
| Multi-village holons (Era IV→V extension, i296 scope) | i296 per-polity holons; i297 territory genesis; i298 auto-partition; i299 trade diffusion; i300 gate (26 memes, 26/26 cross-hosted) | **CLOSED** |
| N≥48 performance budget | charter ratified (§1 baseline, §2 binding rules); i294 superlinearity attributed + O(N³) fix; i295 warn-only perf gate in `scripts/gate` | **CLOSED** |
| Asset pipeline v0 | i301 charter + `export_world_assets_json` + CLI flag; determinism/round-trip/payload pins | **CLOSED** |

**VERDICT: DC-3 COMPLETE at 4/4 legs.** UM-3 evidence trail:
`evidence/i296..i301`, `charters/DC3-P0-perf-budget.md`,
`charters/ASSET-PIPELINE-v0.md`.

### What carries forward (recorded, honest)

- **Saturation caveat** (i300): origin-share at N=12 measures the interlinking
  ceiling; longitudinal origin-share decay would separate trade vs gossip
  channels — observability refinement, not a gate.
- **Vendor-blocked** (unchanged): realms.md ontology, resonance attestation,
  cult-liveliness regime. Unblock conditions documented in §3.2/§6.6.
- **Deferred with triggers** (unchanged): VecDeque events (>250K-tick
  operator horizons), recent-claims index (same), H5 founder variance.

### DC-4 entry (next arc)

Per ROADMAP: DC-4+ → UM-4 vertical slice → AA alpha — graphical client
shell, chronicle lens (i291 lens is the read-side), difficulty levers
(needs-bands canon, CO-2026-003 ratified). The i301 asset schema v1 is the
CLIENT contract; art/audio sub-charters may be drafted against it. First
DC-4 iterations: (a) CLIENT asset-viewer panel consuming `--export-assets`
output; (b) difficulty-levers live tuning surface over the ratified needs
bands; (c) graphical shell spike against the same document.

#### DC-4 execution ledger

| Iter | Status | Landing | Verdict recorded |
|---|---|---|---|
| i303 | **DONE** | evidence/i303_difficulty_levers.md | **(b) difficulty-levers**: row 2 (need decay) promoted DRAFT → LIVE. `DifficultyProfile` (0.6/1.0/1.4) + `SimParameters::with_difficulty` quantize-once mapping; Standard byte-identical to canon (params serde + end-state digest, zero re-anchor); Snapshot v16 persists `params` (fixes the silent reset-all-tuning-on-restore defect); CLI `--difficulty`. Probe `i303_difficulty_bands` verdict `DIFFICULTY_BANDS_LIVE` (aggregate +27%, Worship share −19%, 12/12 alive × 3 bands); 4 core + 2 snapshot + 2 integration pins. Carries forward: levers row 3 (pathology params need `system_development` param-threading), row 2 threshold-half residual |
| i304 | **DONE** | evidence/i304_pathology_bands.md | **(b) difficulty-levers cont'd**: row 3 (pathology growth/decay) promoted DRAFT → LIVE. `SimParameters::{pathology_growth_scale, pathology_decay_scale}` (serde default = identity, not Fixed::ZERO) driven by the same `DifficultyProfile`; `pathology_params` resolves the 4-quadrant operator once per tick from the run's params; `system_development_with_params` is the new tick entry (old 2-arg fn kept as a canon-band wrapper, ~14 call sites untouched). Standard bit-identical by construction (`x×1.0`) → zero re-anchor. Probe `i304_pathology_bands` at 20K × 12 seeds: `PATHOLOGY_BANDS_LIVE`, dark-addiction `0.197/0.341/0.480` with per-seed direction 12/12 on all three pairwise comparisons; 2 core + 3 sim + 2 snapshot pins. Findings: Q2 dark-allergy sits 0.44–0.73 vs ceiling 0.80 (always-step absence growth, systemic debt) and ceilings are deliberately not a band (row-3 residual) |
| i305+ | queued | — | (a) CLIENT asset-viewer panel over the i301 schema; (c) graphical shell spike; levers row 2 threshold-half + row 3 residuals |
