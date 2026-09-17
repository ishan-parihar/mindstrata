# Iteration 274 — Relational feed revival: ritual → event → catalyst (§4.3)

**Status:** LANDED · **Doctrine:** probe-first (i274 scale/diet probe → root cause → fix → re-probe)

## What was wrong

The i274 probe (N=12/48/96, 10K–20K) found the collective field's **Relational
bucket pinned at max_stage 1.000 at every scale** while Safety reached 6.0 and
drove all genesis. Root cause chain:

1. **No emission**: ritual executions bonded endocrine axes, trust, memories,
   and norms but never emitted a `SimEvent` — the collective field's catalyst
   pipeline only reads events.
2. **Even with emission, wrong pass order**: the ritual block ran at the tail of
   `tick_kinship_household_daily` (core.rs:685), *after* `system_development` /
   `system_polarity_claim_emit` / `system_collective_field_step` (core.rs:632–663)
   had consumed the tick's `&self.events[pre_tick_events..]` window. A ritual
   event pushed there would land in **no window at all**.
3. **Bond diet starvation**: the only other Bond feeds are MarriageFormed /
   ChildBorn — once-per-pair lifetime events. Threat (conflicts) is recurring.
   This is the same structural asymmetry the i272 mortality probe found for
   Grief, but here it was *fixable* without demographic horizons: rituals are
   monthly, not generational.

## The fix (three moves)

1. **`SimEvent::RitualPerformed`** (core/event.rs): participants, sponsor,
   ritual_id, bonding, tick. Mirrors the i272 `GriefStruck` precedent
   (event-on-the-canonical-bus, catalyst routing downstream).
2. **Pass extraction**: ritual execution block moved verbatim from
   `tick_kinship_household_daily` into `Simulation::tick_ritual_executions`
   (household.rs), now called in core.rs **before** `system_development` so
   same-tick ritual events are inside the catalyst window. The §12.2
   group-formation block keeps the original `is_duodeca` gate it shared.
3. **Catalyst route** (`collect_catalysts`): one Bond catalyst per participant,
   magnitude `(bonding × 2).clamp(0.2, 0.8)` — seeded rituals carry
   bonding ≈ 0.1–0.15, capped at the MarriageFormed precedent (0.8).

## Probe evidence (N=12, seed 42, 20K ticks)

| Metric | Pre-fix | Post-fix |
|---|---|---|
| Relational max_stage | 1.000 (pinned, all N) | 1.000 — but **press 0.200 accumulated** (was structurally 0×) |
| Ritual events in journal | 0 | 8 (2 rituals × 4 fires) |
| Bond catalysts delivered | ~2 (rare marriage/birth) | ~96 (8 fires × ~12 participants) |

**Pacing math** (why stage 2 lands at ~80K, and why that is correct, not a
missed knob): press per fire-tick = (participants/n) × press_growth(0.05) =
0.5 × 0.05 = 0.025. Stage requires press 1.0 → 40 fires ≈ 80K ticks at the
seeded monthly cadence. This is **scale-invariant** (attendance fraction ×
cadence — no hidden N-dependence) and matches the theory's cultural timescale:
institutions and rituals shape collective development over years-to-decades,
while Threat (recurring daily conflict) drives Safety on a weeks timescale.
No magnitude knob pulled (§4.4): the constant is measured, the mechanism is
live, the timescale is the theory's.

## Controlled comparison (stash-controlled, i273 method)

- Genesis wiring stashed → 10K snapshot passes on old baseline (grain 0.9127,
  events 147935).
- Ritual wiring live → snapshot delta is **exactly `event_count: 147935 →
  147939`** (+4 = 2 rituals × 2 fires: ticks 4320, 8640). Every other surface
  metric identical — the catalysts are behaviorally live at agent level but do
  not cross any pinned threshold in this window (Relational press 0.100 at 10K,
  no stage crossing, no genesis).
- Collapse golden (4320 ticks): exactly one fire at the final tick (+2 events).
  Hash deltas: metric_hash + event_hash moved, **agent_hash byte-identical**
  (hunger/thirst/fatigue/valence untouched — the catalysts act on development
  fields, not needs). Re-anchored with this evidence.
- riverford_minor golden (1000 ticks): below first fire (4320) — untouched,
  stays green.

## Re-anchors (§4.2 trail)

- `golden/collapse/seed_42/baseline.json`: mechanism = 2 RitualPerformed events
  at the final tick route 24 Bond catalysts; agent_hash unchanged proves
  needs-level behavior untouched.
- 10K long-horizon snapshot: mechanism = +4 events (fires at 4320/8640),
  all other fields byte-identical.
- New liveness test: `ritual_performed_presses_relational_bucket` — participant
  altitude moves under a RitualPerformed window (the check that would have
  caught the pre-fix dead producer).

## Remaining debt (honest ledger)

- Relational stage 2 requires ~80K ticks at seeded cadence — festival-dense
  regimes (more/frequent rituals) would accelerate; a pacing knob deliberately
  not pulled.
- Genesis remains Safety-only at N=12 (Threat diet dominance); the
  **Identity** bucket still has no recurring feed (Grief needs mortality
  horizons, i272) — same-family structural debt, recorded.
