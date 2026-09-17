//! Era IV meme genesis (PLAN_DC2 next-horizon item, i273) — the first
//! collective-line consumer that *generates* behavior rather than modulating
//! existing channels.
//!
//! Substrate §5: content generation is "causal domain selection weighted by
//! collective line stages" and "replaces the `seed_initial_memes` fixed
//! roster". v1 operationalization: when a bucket's lines have *advanced* past
//! their founding stage (any line stage ≥ 2.0), the village's culture has
//! developed enough collective depth to birth a new meme keyed to that
//! bucket's domain. The description is deterministic (no RNG): template +
//! line slug + stage epoch, cite-first per the Era III render law.
//!
//! Zero-blast proof (probe `i273_collective_stage_probe`): at the 2K golden
//! horizon every collective line sits at stage exactly 1.0 (CV 0.000) — no
//! line is "advanced", so genesis is inert for every golden/snapshot window.
//! At 20K, stages differentiate (mean 2.93, σ 2.0, max 5.0) and genesis fires.
//!
//! Dedup without new serialized state: a bucket+stage-epoch meme is registered
//! only if no live meme carries its tag in the description. Restore-safe for
//! `from_snapshot` (re-derives from registry scan), identity below stage 2.

use crate::culture::{Meme, MemeContent};
use mindstrata_core::clock::Tick;
use mindstrata_development::collective::{bucket_for_line, CollectiveBucket, CollectiveField};
use mindstrata_social::culture::MemeRegistry;

/// Stage at which a bucket's collective depth first births culture.
pub const GENESIS_STAGE_GATE: f64 = 2.0;

/// Genesis templates per bucket — deterministic STRUCTURE (substrate §5:
/// "grammar templates are canon-frozen STRUCTURE; referent bindings are
/// runtime VARIABLE"). The line slug is the runtime variable here.
const GENESIS_TEMPLATES: &[(CollectiveBucket, MemeContent, &str, f64, f64)] = &[
    (
        CollectiveBucket::Relational,
        MemeContent::Historical,
        "We remember when the {line} first bound us together",
        0.5,
        0.8,
    ),
    (
        CollectiveBucket::Safety,
        MemeContent::Moral,
        "The {line} keeps the peace our elders won",
        0.4,
        0.7,
    ),
    (
        CollectiveBucket::Identity,
        MemeContent::Historical,
        "The {line} is what our people have suffered and kept",
        0.6,
        0.9,
    ),
    (
        CollectiveBucket::Meaning,
        MemeContent::Theological,
        "Through the {line} the village glimpses what endures",
        0.7,
        0.6,
    ),
];

/// Tag prefix marking a genesis meme in the registry (dedup key).
const GENESIS_TAG: &str = "[genesis:";

/// Advance Era IV genesis: register culture memes for buckets whose
/// collective lines have advanced past the founding stage.
///
/// Deterministic and idempotent per (bucket, stage epoch): scanning the
/// registry for the tag makes this restore-safe (a `from_snapshot` sim
/// re-derives the same set). No RNG, no per-tick allocation beyond the
/// description strings for genuinely new memes.
pub fn system_collective_genesis(
    field: &CollectiveField,
    registry: &mut MemeRegistry,
    params_virality_scaling: mindstrata_core::fixed::Fixed,
    tick: Tick,
) {
    let slugs = CollectiveField::line_slugs();
    for (bucket, content, template, emotional, identity) in GENESIS_TEMPLATES {
        // The bucket's max line stage = its deepest collective development.
        let mut max_stage = 0.0_f64;
        for (i, line) in field.lines.iter().enumerate() {
            if i >= slugs.len() {
                break;
            }
            if bucket_for_line(slugs[i]) == *bucket && line.stage > max_stage {
                max_stage = line.stage;
            }
        }
        // Identity below the gate: founding-stage lines birth nothing.
        if max_stage < GENESIS_STAGE_GATE {
            continue;
        }
        // Stage epoch: which integer stage crossing this meme commemorates.
        // epoch 1 = first advance past founding (stage 2.x), etc. Cap at 9
        // keeps tags bounded; stages 11+ would need a new epoch anyway.
        let epoch = (max_stage as usize).min(9);
        let tag = format!("{GENESIS_TAG}{bucket:?}:{epoch}]");
        if registry.memes.iter().any(|m| m.description.contains(&tag)) {
            continue; // already commemorated this advance
        }
        // Pick the deepest advanced line slug of this bucket (deterministic:
        // first max in registry order).
        let mut best_slug = "";
        let mut best_stage = 0.0_f64;
        for (i, line) in field.lines.iter().enumerate() {
            if i >= slugs.len() {
                break;
            }
            if bucket_for_line(slugs[i]) == *bucket && line.stage > best_stage {
                best_stage = line.stage;
                best_slug = slugs[i].slug();
                if best_stage >= max_stage {
                    break;
                }
            }
        }
        let description = template.replace("{line}", best_slug);
        let tagged = format!("{description} {tag}");
        registry.register(Meme::new(
            0, // registry assigns the real id
            tagged,
            *content,
            mindstrata_core::fixed::Fixed::from_f64(*emotional),
            mindstrata_core::fixed::Fixed::from_f64(*identity),
            tick.as_u64(),
            params_virality_scaling,
            // Genesis memes mutate slowly: they are institutional memory,
            // not gossip.
            mindstrata_core::fixed::Fixed::from_f64(0.03),
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mindstrata_core::fixed::Fixed;

    fn virality() -> Fixed {
        Fixed::from_f64(0.8)
    }

    #[test]
    fn genesis_is_identity_below_stage_gate() {
        let field = CollectiveField::default(); // all lines at founding stage
        let mut registry = MemeRegistry::default();
        system_collective_genesis(&field, &mut registry, virality(), Tick::new(100));
        assert!(
            registry.memes.is_empty(),
            "founding-stage field must birth nothing"
        );
    }

    #[test]
    fn genesis_fires_on_advanced_line_and_dedups() {
        let mut field = CollectiveField::default();
        // Advance one line far enough for its bucket to cross the gate.
        // Relational = culture-kind lines; find one.
        let slugs = CollectiveField::line_slugs();
        let idx = (0..slugs.len())
            .find(|&i| bucket_for_line(slugs[i]) == CollectiveBucket::Relational)
            .expect("registry has culture-kind collective lines");
        field.lines[idx].stage = 2.4;
        let mut registry = MemeRegistry::default();
        system_collective_genesis(&field, &mut registry, virality(), Tick::new(100));
        assert_eq!(
            registry.memes.len(),
            1,
            "one bucket crossed the gate → one meme"
        );
        assert!(registry.memes[0]
            .description
            .contains("[genesis:Relational:2]"));

        // Re-run: dedup by tag — no second registration.
        system_collective_genesis(&field, &mut registry, virality(), Tick::new(200));
        assert_eq!(registry.memes.len(), 1, "idempotent per stage epoch");

        // Advance further: new epoch, new meme.
        field.lines[idx].stage = 3.1;
        system_collective_genesis(&field, &mut registry, virality(), Tick::new(300));
        assert_eq!(
            registry.memes.len(),
            2,
            "stage 3 crossing births epoch-3 meme"
        );
        assert!(registry.memes[1]
            .description
            .contains("[genesis:Relational:3]"));
    }

    #[test]
    fn genesis_descriptions_are_deterministic_and_cite_first() {
        let mut field = CollectiveField::default();
        let slugs = CollectiveField::line_slugs();
        let idx = (0..slugs.len())
            .find(|&i| bucket_for_line(slugs[i]) == CollectiveBucket::Meaning)
            .expect("registry has consciousness-kind collective lines");
        field.lines[idx].stage = 2.0;
        let mut a = MemeRegistry::default();
        let mut b = MemeRegistry::default();
        system_collective_genesis(&field, &mut a, virality(), Tick::new(50));
        system_collective_genesis(&field, &mut b, virality(), Tick::new(50));
        assert_eq!(a.memes[0].description, b.memes[0].description);
        // Cite-first law: the runtime variable (line slug) appears in text.
        let slug = slugs[idx].slug();
        assert!(
            a.memes[0].description.contains(slug),
            "genesis text cites its line: `{}`",
            a.memes[0].description
        );
    }
}
