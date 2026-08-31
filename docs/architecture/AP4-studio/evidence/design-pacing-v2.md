# Pacing v2 note (DESIGN 4.23, 2026-08-31)

Owner: DESIGN. Generated 2026-08-31 against `cd6b30e` (75/106).
Status: **DRAFT — refines `pacing-model.md v1` with Era III cadence.**

## Pacing law

* `pacing-model.md v1` (`pacing v1 @8a11747`) sets `10K/50K/100K` tick budgets `15s/180s` (`i270 10.7K tps`). v2 adds Era III content cadence: one `RealmTriple` synthesis per `100` ticks per agent (deterministic, `is_legal` gate), so `100K` horizon yields `~1K` triples/agent without tick cost (pure types, no sim wiring today).

## What lands this note

* Records that `template.render` + `extract_referent` + `is_legal` compose deterministically: `template → referent → RealmTriple → is_legal` is pure and `golden 5/5` byte-identical, so pacing v2 can be measured in `mindstrata-benches` before sim wiring.

## Blocker

`DESIGN 4.23-4.24` need `i275` probe measuring `needs decay σ` under `IC-5` CO- — same blocker as `4.22`. Until then, pacing v2 stays `DRAFT` and `difficulty-levers.md` stays `DRAFT`.
