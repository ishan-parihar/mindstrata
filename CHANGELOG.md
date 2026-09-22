
> **HISTORICAL:** this changelog stops at Iteration 214. It is not maintained; the
> authoritative history is `git log` plus the per-iteration evidence docs
> (`docs/architecture/AP4-studio/evidence/`) and the execution ledgers in
> [`docs/PLAN_DC3_DEVELOPMENT.md`](docs/PLAN_DC3_DEVELOPMENT.md).

## [Iteration 214] - 2026-08-19
### Fixed
- **S3-2-3 respiratory smoke/damp derivation**: smoke_exposure now derives from ambient temperature (cold = more fires = more smoke, scale 0-0.6), damp_housing from rainfall (heavy rain = damper houses, scale 0-0.5). The respiratory irritation channel now genuinely differentiates winter/cold/wet environments from summer/dry ones. The last two biology-layer hardcoded ZERO placeholders are eliminated.

