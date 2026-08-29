# Pathology Intensity Curves (v1.0 — spec draft)

Owner: DESIGN → STORY/SIM implementation (FR-024, FR-031, WP-C operator). Companion to
canon-inventory row 2 and `needs-bands.md`. Theory grounding: ratified 4-fold model
(dark/golden × addiction/allergy) with Agape/Eros metabolism — `docs/architecture/AP3-afa/
03-substrate.md` §5 + `docs/architecture/AP3-afa/04-waves.md` WP-C definition; vault source
`KosmOS/_Ontology/pathologies.md` + per-cell `pathology:` frontmatter in `vendor/afa/cells/`
is the ontology source-of-record (vendored sha `868b2239` per `vendor/afa/PROVENANCE.md`).
All numeric parameters are CANON constants — they become `crates/mindstrata-development/
canon.rs` entries and change only via IC-5 change-orders with probe evidence (AGENTS.md §4).

Status: **CALIBRATION-PENDING(AP3) — spec frozen, values not landed.** Engine placeholder
`OperatorParams::pending()` (`growth 0.05 / decay 0.02 / ceiling 1.0`) remains in tree until
`i<iter>_pathology_signature` probes supply measured trajectories. No sim-side pathology
behavior is live yet beyond the zero-at-zero identity pass (tasks 3.2–3.3).

## Engine mapping (shape, not values)

`crates/mindstrata-development/src/dynamics.rs` implements the operator as pure
`QuadrantState::step(metabolism, pressure, &OperatorParams)`:

- **Addiction** (`Metabolism::Addiction`): `next = intensity + growth·headroom·pressure − decay·intensity`
- **Allergy** (`Metabolism::Allergy`): `next = intensity + growth·headroom·(1−pressure) − decay·pressure·intensity`
- `headroom = ceiling − intensity`, clamped to `[0, ceiling]`; zero pressure + zero intensity
  is the bit-identity fixed point (FR-023/FR-012, golden-untouched until first stepped pressure).

`pressure` is the resonated catalyst exposure for that quadrant in `[0,1]`
(over the IC-1 `CatalystKind → Drive` projection + `resonance_weight` same-line gating;
cross-line affinities are `0.0` in v1). Per-quadrant pressure derivation is the
calibration surface — which kind of catalyst feeds which quadrant — and lives in the
table below as the **onset trigger** row.

## Per-quadrant specification

### Q1 — Dark Addiction (contracting fixation under deficit/threat)

- **Ontology cell:** `pathology: dark-addiction` — repeated drive-satisfying catalysts without line progress; fixation on existing stage's security affordances.
- **Onset trigger (pressure source):** sustained `Threat`/`Transgression` catalysts tagged to the agent's *current* stage's deficient line (same-line resonance) without an altitude-advancing catalyst in the same window. Candidate driver: `CatalystKind::Threat` at `pressure ≥ 0.5` on 3 consecutive days.
- **Growth law candidate:** saturating exponential `growth·headroom·pressure` with `growth 0.04–0.08`, `decay 0.01–0.03` (dayscale half-life 23–69 ticks at `decay 0.02`). Ceiling `0.85–1.0` pending gorge vs hard clamp decision.
- **Agape/Eros metabolism coupling:** Eros intake (agency drive) modulates `growth` multiplier; Agape deficit widens `headroom` (open question #1 below — probe varies both).
- **Interaction hazard:** must not pin fear at 0.99 (AGENTS.md §5 dead-producer — fear saturation already guarded by liveness probes). Pathology intensity probes run alongside fear equilibrium checks.
- **Probe plan:** `i<iter>_pathology_dark_addiction_signature` — 20K-tick sweep at seeds 42/4242: sustain `Threat` catalysts via scenario editor, measure quadrant intensity curve, `Work` action suppression delta (3.4 wiring), and fear co-movement. Expect monotone rise to plateau; heredity-neutral controls flat at 0.

### Q2 — Dark Allergy (recoil from contradictory exposure)

- **Ontology cell:** `pathology: dark-allergy` — recoil/aversion to belief-contradicting exposure; withdrawal from the line that delivered contradiction.
- **Onset trigger:** `Transgression` catalysts carrying the agent's *allergic* line tag after a norm/legal violation observation; Allergy pressure = `1 − engagement`, so avoidance accumulates.
- **Growth law candidate:** threshold-linear + Allergy inversion: intensity grows as `growth·headroom·(1−pressure)` when `pressure < 0.3` for ≥5 days, resolves as `decay·pressure·intensity` on sustained `pressure ≥ 0.6`. `growth 0.03–0.06`, `decay 0.02–0.04`.
- **Ceiling candidate:** `0.70–0.90` (allergic recoil rarely saturates to 1.0 in vault exemplars).
- **Probe plan:** `i<iter>_pathology_dark_allergy_signature` — expose agents to contradicting memes/norm verdicts (future Era III grammar), measure Allergy intensity vs exposure schedule and Socialize avoidance delta.

### Q3 — Golden Addiction (premature reach / inflation)

- **Ontology cell:** `pathology: golden-addiction` — grasping at next stage's affordances without transcend-and-include; vault exemplar `health/10-concrete-peak` "excellence without distribution" pattern.
- **Onset trigger:** repeated *golden-path* catalysts (next-stage-tagged `Bond`/`Grief` with Agape/Eros weighting) at pressure `≥ 0.6` when `development_field.altitudes[line]` is still ≥1.0 stage below the catalyst's target stage (gate would block altitude advance, but pathology still accumulates).
- **Growth law candidate:** logistic `growth·headroom·pressure` with `growth 0.05–0.10`, slower `decay 0.01–0.02` (golden fixation is stickier). Ceiling `0.80–1.0`.
- **Probe plan:** `i<iter>_pathology_golden_addiction_signature` — inject next-stage catalysts via seeded `line_tags` override in scenario, measure Golden-Addiction intensity and meaning-deficit co-movement; gated-off altitude advancement must stay 0 while intensity rises (discriminant).

### Q4 — Golden Allergy (refusal of opening)

- **Ontology cell:** `pathology: golden-allergy` — refusing the genuine opening the next stage offers; vault exemplar `developmental/01-prehension` "entrapment in maintenance without trajectory".
- **Onset trigger:** avoidance of golden catalysts (same family as Q3) — Allergy pressure = `1 − golden_exposure`, so sustained `pressure = 0` on golden tags over 10+ days accumulates Allergy even with no dark pressure.
- **Growth law candidate:** slow ramp `growth 0.02–0.04·headroom·(1−pressure)`, resolves only through sustained golden engagement `pressure ≥ 0.7` for 5+ days at `decay 0.02–0.05` rate. Ceiling `0.65–0.85`.
- **Probe plan:** `i<iter>_pathology_golden_allergy_signature` — withhold golden catalysts from high-altitude founders, measure Golden-Allergy rise and Worship/meaning-action suppression; re-introduce golden exposure and measure decay half-life.

## Common calibration discipline

- All four rows land via `crates/mindstrata-development/canon.rs` as `PATHOLOGY_GROWTH_*`, `PATHOLOGY_DECAY_*`, `PATHOLOGY_CEILING_*` constants, each annotated `// CALIBRATION-PENDING(AP3): measured <value> via i<iter>_… on <seed>`.
- Re-anchor comments follow AGENTS.md §4.2 form: measured value, old band, mechanism (`"dark-addiction plateau 0.62 pushed Work suppression −0.05 via pathology_dark 0.62·0.08"`).
- Zero-at-zero discriminant required: neutral founders stay neutral under empty catalyst windows (existing pins `liveness_moves_on_real_catalysts`, `tick_is_deterministic`); pathology movement is the *only* allowed drift at fixed seeds once probes activate.

## Open questions (carried as probe hypotheses, not resolved here)

1. Metabolism coupling: does Agape/Eros intake rate modulate `growth` (rate) vs `ceiling` (capacity)? Probe varies both axes via `kind_drive_map` drive weights.
2. Interaction with existing psych state (fear/anger saturation hazards — AGENTS.md §5 dead-producer rule): every pathology probe records fear/anger/psychopathology equilibrium values alongside quadrant intensities to detect pinning.
3. Whether a single shared `OperatorParams` suffices or per-quadrant parameter sets are required — current engine takes one `&OperatorParams`; per-quadrant specialization is a `// ponytail:` deferral if probe trajectories diverge beyond `0.05` tolerance.
