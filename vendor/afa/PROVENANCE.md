# PROVENANCE — vendor/afa ontology extracts

- **Vault**: `/home/ishanp/Documents/knowledge-base/KosmOS/_Ontology`
- **Vault git sha**: `868b2239da3af06909ec10e82345b98d1082f04a` (KosmOS repo)
- **Extracted**: 2026-08-25
- **Extractor**: `vendor/afa/extract.py` (stdlib-only; deterministic directory order)

## Reproduction command

```
python3 vendor/afa/extract.py /home/ishanp/Documents/knowledge-base/KosmOS/_Ontology vendor/afa
```

## Tables and counts (cross-checked at extraction time)

| Artifact | Source | Rows | Verification |
|---|---|---|---|
| `ladder.csv` | `_Ontology/stages.md` §"The Canonical 17-Stage Ladder" markdown table | 17 stages (+header = 18 lines) | row count equals the canonical table's 17 stage rows |
| `lines.csv` | `_Ontology/lines/*.md` YAML frontmatter (`line`, `kind`, `quadrant`, `status`, `matrix_cells_required`, `matrix_depth`) | 49 lines (+header = 50 lines) | equals count of non-underscore `.md` files in `lines/` carrying a `line:` frontmatter key |
| `couplings.csv` | `_Ontology/stages/by-line/<line>/<NN-slug>/_index.md` frontmatter per cell | 851 couplings (+header = 852 lines) | source has 852 stage-cell directories; 851 carry `_index.md` — the single gap is `religion-statistical/12-formal-landmark` (directory exists, no `_index.md` in vault), so it is ABSENT here rather than invented |
| `cells/<line>/<NN>-<slug>.md` | verbatim copies of each cell `_index.md` | 851 files | one per couplings.csv row |

Note: the AP3 doctrine speaks of **19 curated lines**; the vault carries **49 line
registries**. This vendor extract takes ALL lines present in the vault (honest
superset); curation to the AP3 19-line set happens downstream in codegen/WP-0Ab via a
curated-list filter, NOT by dropping data here.

## Deliberate exclusions

- Vault root documents (`CONSTITUTION.md`, `dynamics.md`, `pathologies.md`,
  `polarity.md`, `realms.md`, lenses/, method/, morphs/, …) are NOT vendored — they are
  theory-of-record reading material, not tabular substrate. Cell frontmatter fields
  relevant to codegen (altitude, depth_status) ride inside `couplings.csv`.
- `stages/by-line/_template/` skipped (convention template).
