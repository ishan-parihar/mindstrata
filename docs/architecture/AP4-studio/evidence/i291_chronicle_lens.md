# Iteration 291 — Era V WP-L: chronicle ray lens (PLAN_DC3 §4 i291)

**Status:** LANDED · **Doctrine:** D5 lens-never-place — the ray/density overlay tints
chronicle narrative text ONLY; zero mechanical effect, pinned by test. **Era V exit
gate CLOSES on this evidence** (both Era V WPs now landed: WP-K i290, WP-L i291).

## The lens table

`RAY_DENSITY_BY_STAGE` in `mindstrata-development/src/render.rs`: one ray per ladder
rung 1..=17, extracted from the vendored vault's own per-cell `ray:` frontmatter
tallied by stage (50 cells/rung; stage 9 unanimous 49/49 Indigo-Turq):

| Rung | 1–3 | 4 | 5 | 6 | 7 | 8 | 9 | 10 | 11 | 12 | 13 | 14 | 15 | 16 | 17 |
|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|---|
| Ray | Red | Green | Blue-In | Blue-Out | Blue-Ext | Indigo-Teal | Indigo-Turq | Indigo-Ext | Indigo-Master | Violet-7th | Violet+ | Violet-Teal | Violet-Turq | Kosmic | Unity |

Ties at rungs 3/4 resolve by the ladder's own climb order (3→Red closing egocentric,
4→Green first plural rung). **Provisional until `KOSMOS/lenses/rays.md` is vendored** —
the tallies are the in-repo evidence; re-vendor replaces the table, zero call-site churn.

## Implementation

- `render.rs` (development): `ray_density_for_stage(u8)` lens lookup (clamped),
  `stage_of_agent_altitude(f64)` shadow→rung (0.0 = unattuned rung 0 — a neutral
  field must not claim a ray), `render_claim_tinted` / `render_lore_section_tinted`
  with an injected `Option<&dyn Fn(&str) -> Option<&'static str>>` resolver;
  `LoreLine.ray` field (transient — never serialized, no wire impact). Untinted
  paths (`render_claim` / `render_lore_section`) byte-stable with pre-WP-L output.
- `chronicle.rs` (sim): unified village resolver — collective field stages first
  (rung units; every line ≥ 1.0 so village lines always tint), personal altitude
  shadows as fallback for claim lines outside the collective registry (any live
  signal → rung 1, until the cross-scale shadow↔rung contract is attested).
  Lens header cites the scale: `Seen through the lens of {ray} (village stage {n})`.

## In-vivo (`i291_chronicle_lens`, golden seed 42, 2K ticks)

```
- the lore says: Institution holds Belief via Communion (Red-Ray (1st) tint)
- the lore says: Person holds Belief via Entropy (now called into question)
- the lore says: Structure Practice in Person (Red-Ray (1st) tint)
...
Seen through the lens of Red-Ray (1st) (village stage 1.0)
```

Village stage 1.0 → all lines in the Red band, tension/refuted tags untouched,
tint strictly append-only.

## Pins

- `ray_lens_is_render_only_and_tints_live_lines` (development): live line tints,
  no-lens path unchanged.
- `ray_lens_covers_all_rungs_and_zero_is_unattuned`: 17 cells, clamps, shadow-0.
- `chronicle_lore_ray_lens_tints_without_rewriting` (sim): mechanical-effect pin —
  header present, ≥1 tint, and **every untinted line survives verbatim** in the
  tinted render (text selection/tags/dedup/cap are lens-independent).

## Verification

- fmt clean; clippy workspace 0 warnings; development 82/82 (+2 new); sim chronicle
  8/8 (+1 new); full release suite 307/0/1; golden 5/5 byte-identical (read-only
  render pass, no state touched — zero re-anchors by construction).

**WP-L CLOSED. Era V exit gate CLOSED.**
