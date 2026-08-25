---
name: wp-era345
description: "AP3 Era III–V work packages: content grammar (G2), polarity belief engine (H2/H3), collective holon (I), institutional coupling (J), observability (K), chronicle lens + open loops (L). Structured briefs — expand to full briefs at wave entry."
type: Work-Package-Brief
plan_id: AP3
wave: "III,IV,V"
owned_paths:
  - "G2: crates/mindstrata-social/src/culture/content_gen.rs (NEW) + culture/meme.rs ⓦ"
  - "H2: belief/journal/gossip hooks in mindstrata-psych + mindstrata-social"
  - "H3: ritual/norm generators (institutions/norms.rs read-side, social/culture)"
  - "I: crates/mindstrata-sim/src/sim/collective.rs (NEW) + mod.rs ⓦ"
  - "J: institutions/* coupling read-side"
  - "K: crates/mindstrata-tui/src/render.rs"
  - "L: crates/mindstrata-tui/src/render/session.rs + benches loop backlog"
depends_on: ["prior era exit gate"]
coordination_warning: "TUI files are hot with the parallel session — re-confirm ownership before K/L dispatch"
---

# Era III — Content emergence (~281–292)

## WP-G2 — Three-realm grammar & meme generation

`content_gen.rs`: Domain(Causal)/Framework(Subtle)/Entity(Gross) types; compositional
generator domain × gross-referent × tension-state × line-signature → meme instances.
Replaces `seed_initial_memes` fixed roster. Type-check rule: every subtle item cites ≥1
gross entity within exactly one domain. Templates = canon-frozen structure; bindings =
runtime variable.

Probe i284_culture_disjoint (Q3): two seeds, same founder bands, different event histories
→ disjoint meme rosters by 20K ticks (jaccard < threshold).

## WP-H2 — Polarity reconciliation over beliefs

Belief/meme claims carry TensionState; gossip uptake proposes reconciliations;
contradictory evidence refutes (Reconciled→ActiveTension). Moral panic = active-tension
cascade probe (i288_panic_cascade): refutation storms produce measurable norm churn,
then re-crystallization.

## WP-H3 — Ritual & norm generation

Ritual forms generated as Agape-metabolizer vehicles (mourning rites bind grief referents);
norm proposals generated from reconciled polarity clusters. Probe i291_agape_metabolism:
post-casualty villages WITH generated mourning rites show faster Dark-pathology decay than
without.

# Era IV — Collective holon (~293–305)

## WP-I — Village stage_lines

`sim/collective.rs`: CollectiveField over 8 collective lines; daily pass extension reads
aggregate signals (meme pool composition, institution legitimacy, market state, legal
outcomes). Tetra-arising gate: collective stage bands gate WHICH content classes the
generator may emit; individual distribution gates UPTAKE weight.

## WP-J — Institutional altitude coupling

Institution behavior parameters become functions of governance/economic-systems line
stages (read-side multipliers, midpoint-neutral). Probe i300_institution_shift: village
crossing amber→green band on governance line shows pluralistic institution deltas.

# Era V — Harvest (~306–320+, then loops)

## WP-K — Observability completion

Per-quadrant transition traces complete (R6); TUI longitudinal lane per agent +
village panel. KosmOS-frontmatter export of all holon stage_lines maps (snapshot.rs).

## WP-L — Chronicle lens & flavor

Ray/density lens rendering for chronicles ONLY (doctrine D5 — lens never place):
altitude→ray mapping colors narrative text; no mechanical effect ever.

## Open-ended loop backlog (doctrine §5 recipe)

- Extend curated lines when ontology attests cells (coverage report drives).
- Deepen resonance matrix as lower-altitude couplings attest.
- Golden-Addiction cult dynamics vs institution legitimacy interplay.
- Cross-village cultural diffusion (trade partners exchange subtle items).
- Multi-village worlds: collective holon per polity, civilization-axioms line activation.
