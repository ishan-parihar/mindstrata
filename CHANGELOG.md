
## [Iteration 214] - 2026-08-19
### Fixed
- **S3-2-3 respiratory smoke/damp derivation**: smoke_exposure now derives from ambient temperature (cold = more fires = more smoke, scale 0-0.6), damp_housing from rainfall (heavy rain = damper houses, scale 0-0.5). The respiratory irritation channel now genuinely differentiates winter/cold/wet environments from summer/dry ones. The last two biology-layer hardcoded ZERO placeholders are eliminated.

