# Iteration 269 — Era III content rendering into annals (UM-2)

**Status:** LANDED · **Doctrine:** consumer wiring over existing live data; render layer carries its own unit evidence

## What was wrong

The Era III grammar types (`realm.rs` `RealmTriple`/`is_legal`, `template.rs`
`Template::render` + `SAMPLE_TEMPLATES`) existed as cited types with **no
consumer**. Claims from the live polarity path (`system_polarity_claim_emit`,
i278: 13/13 agents accumulate claims in 2000 ticks) were reconciled but never
*rendered* — the village told no stories (UM-2 "the village tells stories").

## What landed

1. **`mindstrata-development/src/render.rs` (new)** — the bridge:
   - `project_to_era3(&ThreeRealmClaim) -> RealmTriple`: total, deterministic
     projection of the live 3×4×4 grammar into Era III's 4×4×3. Material
     Event/Fact → Agency, other Material → Entropy; Relational → Communion;
     Symbolic → Structure. Site/Resource → World, Event → Person,
     Institution → Institution. Fact → Value, Norm → Practice,
     Value/Identity → Belief.
   - `render_claim`: legality-gated (`is_legal()` — today unreachable over
     the live projection; becomes load-bearing when the WP-I vendor mapping
     lands), cite-first, template selected by FNV-1a over the claim's line
     slug so a claim renders identically forever.
   - `render_lore_section`: dedup by text, insertion order, capped,
     tension/synthesis tagged. Zero-at-zero.
   - 8 unit tests (liveness, determinism, tagging, cap, dedup, zero).
2. **Chronicle consumer (`sim/chronicle.rs`)** — closing annal "The lore of
   the village": all agents' `polarity_claims` in index order → rendered,
   capped at 12 lines. Read-only, TUI/legibility-only, no golden surface.

## Honest limitations (recorded, not hidden)

- **Vendor coupling gap**: `realms.md` is deliberately not vendored
  (PROVENANCE.md: root docs are theory-of-record), so the full
  legality mapping is blocked at source. The ratified single rule
  (Entropy×Institution×Belief illegal) stands; `is_legal()` is
  currently unreachable from the live projection. Recorded as the
  WP-I vendor-coupling ponytail.
- **Doc-test rot found in `referent.rs`** (asserts a non-existent
  `GrossReferent::Family` variant; never compiled since doc tests
  don't run in the gate) — noted as debt, not fixed here (one root
  cause per iteration).

## Verification

- `cargo test -p mindstrata-development --lib`: **80/80** (render 8/8)
- `cargo test -p mindstrata-sim --lib chronicle`: **7/7**
  (new `chronicle_lore_section_appears_after_run` pins liveness +
  determinism; seed 42, 2000 ticks)
- `cargo clippy --workspace --quiet`: 0 warnings
- Golden surfaces untouched: chronicle is not hashed by goldens;
  render is a pure function with no sim-state writes.

## Debt recorded

- Doc-test rot in `referent.rs` (pre-existing).
- `LineId` doc-example in `line.rs` uses "attachment"/"aesthetic" slugs —
  same doc-test class.
