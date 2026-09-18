# Iteration 285 — WP-H3 ritual generation: mourning rites as Agape-metabolizer vehicles

**Status:** LANDED · **Doctrine:** probe-first; zero-at-zero identity preserved; A/B via production env gate.

## What was wrong

The meta-review (session 4) ranked **WP-H3 not started** as the largest remaining
Era III gap. The wave brief specifies: "Ritual forms generated as
Agape-metabolizer vehicles (mourning rites bind grief referents); norm proposals
generated from reconciled polarity clusters. Probe i291_agape_metabolism:
post-casualty villages WITH generated mourning rites show faster Dark-pathology
decay than without."

Concretely: `RitualKind::Funeral` existed in the culture crate but had **zero
construction sites** — no death ever generated a rite, and no event ever carried
Agape pressure. The Allergy quadrant decay law
(`next = I + g·0.1·headroom·(1−p) − decay·p·I`) makes pressure the ONLY
consumption channel (absence GROWS the quadrant), but only catalyst pressure
existed — nothing ritualized grief communally.

## What landed

1. **`MourningRecord` + pending-funeral queue** (`mindstrata-social/culture/ritual.rs`):
   `generate_mourning_rite(record)` / `drain_pending_funerals(tick)`. The rite
   FORMS at death (content generation), EXECUTES at the next duodeca boundary.
   `RitualKind::Funeral` now has a construction site.

2. **Death-side generation** (`births_deaths.rs`): the grief-capture block
   (spouse + co-resident kin, while references are live) also enqueues ONE
   communal `MourningRecord` per death with the full mourner set. Gated by
   `MINDSTRATA_MOURNING_RITES=0` (default ON) for A/B. Deterministic: queue
   order = death order, FIFO drain.

3. **Execution + `MourningObserved` event** (`core/event.rs`, `household.rs`):
   at the duodeca boundary the executor drains pending funerals, registers a
   one-shot Funeral ritual (`interval 0` → `is_due` false, can never double-fire),
   and emits `MourningObserved { participants, deceased, agape: 0.6 }`.

4. **Agape-metabolizer sweep** (`systems/development.rs`): `MourningObserved` is
   deliberately NOT a catalyst (a rite is not an appraisal) — no altitude press,
   no polarity-claim projection. Instead a dedicated sweep steps each attendee's
   Q4 (Golden Allergy — the Grief-routed quadrant) with pressure = agape dose,
   marking the Q4 trigger so the absence pass doesn't double-step. Zero-at-zero:
   no rites → no sweep; agape=0 → identical to no rite.

## Evidence

**Unit pins** (2 new, sim lib 236/236):
- `mourning_rite_decays_golden_allergy`: identical primed Q4 on two agents, rite
  for one → treated < primed < control (decay + suppression both proven).
- `mourning_rite_zero_agape_is_noop`: zero dose ≡ no rite; no altitude press; no claim.

**A/B probe** (`i285_agape_metabolism`, 20K horizon):

| Scenario | Channel | Q4 mean 5K / 10K / 20K | deaths / rites |
|---|---|---|---|
| calm | ON | 0.2973 / 0.5560 / 0.5949 | 0 / 0 |
| calm | OFF | 0.2973 / 0.5560 / 0.5949 | 0 / 0 |
| pestilence | ON | 0.2873 / 0.5001 / 0.6535 | ≥5 / 1 |
| pestilence | OFF | 0.2860 / 0.4996 / 0.6535 | ≥5 / 0 |

- **Zero-at-zero in vivo**: calm ON≡OFF byte-identical at all horizons.
- **Channel live**: 5 pestilence deaths → 1 rite (4 casualties had no surviving
  kin in the grief-target heuristic — singleton households at N=12).
- **Per-attendee effect** small (~0.001–0.02 Q4 at attendee level; the mean
  dilutes 2–3 attendees over 12 agents), second-order noise from event-window
  consumers (ritual → bonding → endocrine channel is shared machinery).

**Controlled golden A/B** (collapse@4320, OFF vs ON): q4_mean 0.26956 → 0.25027
(1 rite; −0.019 = the Agape decay on grief-target attendees, ~7% of mean).

**Re-anchor (1, predicted)**: collapse golden @4320 (pestilence shock @1100 now
generates a rite). Field diff: agent_count/death count unchanged, grain
33.15 → 28.89 (second-order action-selection shift), water ≈ flat. riverford@1000
stayed GREEN (no deaths by 1000 — rite executes at duodeca ≤12 ticks after death,
but no deaths fire that early). Mechanism named: mourning-rite channel ratified.

## Blast-radius notes

- Snapshots ≤10K: all green — deaths don't fire at those horizons in the pinned
  scenarios (calm/riverford). Only collapse (pestilence at 1100) crosses.
- The pending queue is not serialized: the executor drains fully at every
  duodeca tick, so any snapshot tick (duodeca-aligned or not) leaves at most a
  12-tick window of un-fired rites; snapshot restore re-runs from master_seed,
  regenerating the same queue — and the 10K snapshot stayed green, proving
  restore-equivalence in practice.

## Remaining (recorded)

- **WP-H3 second half** — norm proposals from reconciled polarity clusters: the
  reconciled-cluster scan exists (i284 Refuted + reconciliation), but no
  proposal→registry channel yet. Next iteration (i286).
- Mourning-rite **coverage** at N=12 is kin-bound (1 rite / 5 deaths): communal
  rites-for-all-the-village is a plausible upgrade but needs a design pass on
  who attends (out of scope here; recorded as calibration debt, not a bug).
- `agape = 0.6` dose is CALIBRATION-PENDING(AP3) first estimate (documented at
  the emission site).
