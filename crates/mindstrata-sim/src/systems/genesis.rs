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
use mindstrata_institutions::institutions::Institution;
use mindstrata_social::culture::MemeRegistry;
use mindstrata_world::world::{Site, SiteKind};

/// Stage at which a bucket's collective depth first births culture.
pub const GENESIS_STAGE_GATE: f64 = 2.0;

/// Genesis templates per bucket — deterministic STRUCTURE (substrate §5:
/// "grammar templates are canon-frozen STRUCTURE; referent bindings are
/// runtime VARIABLE"). i276: TWO runtime variables now — the line slug
/// (individual line-signature) and the GROSS REFERENT (a real site or
/// institution name, substrate §5's "cites ≥1 gross entity within exactly
/// one domain"). {ref} is the gross citation; {line} is the signature.
const GENESIS_TEMPLATES: &[(CollectiveBucket, MemeContent, &str, f64, f64)] = &[
    (
        CollectiveBucket::Relational,
        MemeContent::Historical,
        "We remember when the {ref} first bound us together through the {line}",
        0.5,
        0.8,
    ),
    (
        CollectiveBucket::Safety,
        MemeContent::Moral,
        "The {ref} and the {line} keep the peace our elders won",
        0.4,
        0.7,
    ),
    (
        CollectiveBucket::Identity,
        MemeContent::Historical,
        "The {ref} is what our people have suffered and kept — the {line} remembers",
        0.6,
        0.9,
    ),
    (
        CollectiveBucket::Meaning,
        MemeContent::Theological,
        "Through the {ref} the {line} glimpses what endures",
        0.7,
        0.6,
    ),
];

/// Tag prefix marking a genesis meme in the registry (dedup key).
const GENESIS_TAG: &str = "[genesis:";

/// Referent eligibility per bucket (substrate §5 type-check law: a subtle
/// item "cites ≥1 gross entity within exactly one domain" — the bucket IS
/// the domain selector, so each bucket draws from ONE referent source):
/// relational/safety genesis cites INSTITUTIONS (the bonding and order
/// surfaces); identity/meaning genesis cites communal SITES (the village's
/// physical self). Houses are excluded (private dwellings are not the
/// village's self-image); the registry order is the deterministic order.
fn eligible_referents(
    bucket: CollectiveBucket,
    institutions: &[Institution],
    sites: &[Site],
) -> Vec<String> {
    match bucket {
        CollectiveBucket::Relational | CollectiveBucket::Safety => {
            institutions.iter().map(|i| i.name.clone()).collect()
        }
        CollectiveBucket::Identity | CollectiveBucket::Meaning => sites
            .iter()
            .filter(|s| s.kind != SiteKind::House)
            .map(|s| s.name.clone())
            .collect(),
    }
}

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
    institutions: &[Institution],
    sites: &[Site],
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
        // i276: ground the meme in a GROSS referent — deterministic
        // epoch-rotation through the bucket's eligible entities (registry
        // order). Falls back to the ungrounded text only if the world has
        // no eligible entities at all (empty institution+site registries).
        let referents = eligible_referents(*bucket, institutions, sites);
        let grounded = if referents.is_empty() {
            description
        } else {
            let referent = &referents[epoch % referents.len()];
            description.replace("{ref}", referent)
        };
        let tagged = format!("{grounded} {tag}");
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

    /// Fixture institutions (names only used by genesis).
    fn fixture_institutions() -> Vec<Institution> {
        use mindstrata_institutions::institutions::InstitutionKind;
        vec![
            Institution::new(0, InstitutionKind::Council, "Village Council".into()),
            Institution::new(1, InstitutionKind::Temple, "Village Temple".into()),
        ]
    }

    /// Fixture sites (communal only — Houses are ineligible).
    fn fixture_sites() -> Vec<Site> {
        use mindstrata_core::id::EntityId;
        vec![
            Site {
                id: EntityId::new(0),
                kind: SiteKind::Well,
                name: "Village Well".into(),
                owner: None,
                capacity: 10,
                storage_capacity: Fixed::from_f64(100.0),
                inventory: Vec::new(),
            },
            Site {
                id: EntityId::new(1),
                kind: SiteKind::House,
                name: "House 1".into(),
                owner: None,
                capacity: 1,
                storage_capacity: Fixed::from_f64(10.0),
                inventory: Vec::new(),
            },
        ]
    }

    #[test]
    fn genesis_is_identity_below_stage_gate() {
        let field = CollectiveField::default(); // all lines at founding stage
        let mut registry = MemeRegistry::default();
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(100),
            &fixture_institutions(),
            &fixture_sites(),
        );
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
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(100),
            &fixture_institutions(),
            &fixture_sites(),
        );
        assert_eq!(
            registry.memes.len(),
            1,
            "one bucket crossed the gate → one meme"
        );
        assert!(registry.memes[0]
            .description
            .contains("[genesis:Relational:2]"));

        // Re-run: dedup by tag — no second registration.
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(200),
            &fixture_institutions(),
            &fixture_sites(),
        );
        assert_eq!(registry.memes.len(), 1, "idempotent per stage epoch");

        // Advance further: new epoch, new meme.
        field.lines[idx].stage = 3.1;
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(300),
            &fixture_institutions(),
            &fixture_sites(),
        );
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
        system_collective_genesis(
            &field,
            &mut a,
            virality(),
            Tick::new(50),
            &fixture_institutions(),
            &fixture_sites(),
        );
        system_collective_genesis(
            &field,
            &mut b,
            virality(),
            Tick::new(50),
            &fixture_institutions(),
            &fixture_sites(),
        );
        assert_eq!(a.memes[0].description, b.memes[0].description);
        // Cite-first law: the runtime variable (line slug) appears in text.
        let slug = slugs[idx].slug();
        assert!(
            a.memes[0].description.contains(slug),
            "genesis text cites its line: `{}`",
            a.memes[0].description
        );
    }

    #[test]
    fn genesis_cites_a_gross_referent_within_one_domain() {
        // Substrate §5 type-check law: every generated item cites ≥1 gross
        // entity. Safety (institution-bucket) genesis must cite an
        // INSTITUTION name; identity (communal-site bucket) must cite a
        // non-House SITE.
        let mut field = CollectiveField::default();
        let slugs = CollectiveField::line_slugs();
        let safety_idx = (0..slugs.len())
            .find(|&i| bucket_for_line(slugs[i]) == CollectiveBucket::Safety)
            .expect("registry has system-kind collective lines");
        field.lines[safety_idx].stage = 2.1;
        let mut registry = MemeRegistry::default();
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(10),
            &fixture_institutions(),
            &fixture_sites(),
        );
        assert_eq!(registry.memes.len(), 1);
        let desc = &registry.memes[0].description;
        let cites_institution = fixture_institutions()
            .iter()
            .any(|i| desc.contains(&i.name));
        assert!(
            cites_institution,
            "safety genesis must cite a gross institution: `{desc}`"
        );

        // Identity bucket cites a communal site — House 1 must NOT appear.
        let mut field2 = CollectiveField::default();
        let identity_idx = (0..slugs.len())
            .find(|&i| bucket_for_line(slugs[i]) == CollectiveBucket::Identity)
            .expect("registry has collective-system-kind lines");
        field2.lines[identity_idx].stage = 2.1;
        let mut registry2 = MemeRegistry::default();
        system_collective_genesis(
            &field2,
            &mut registry2,
            virality(),
            Tick::new(10),
            &fixture_institutions(),
            &fixture_sites(),
        );
        let desc2 = &registry2.memes[0].description;
        assert!(
            desc2.contains("Village Well"),
            "identity genesis must cite the communal site: `{desc2}`"
        );
        assert!(
            !desc2.contains("House 1"),
            "private dwellings are not the village's self-image: `{desc2}`"
        );
    }

    #[test]
    fn genesis_referent_rotation_is_deterministic_per_epoch() {
        // Successive epochs rotate through the eligible referents in
        // registry order (epoch % len) — deterministic, no RNG. Identity's
        // only eligible site is the Well (House filtered), so both epochs
        // cite it while their stage tags differ.
        let mut field = CollectiveField::default();
        let slugs = CollectiveField::line_slugs();
        let idx = (0..slugs.len())
            .find(|&i| bucket_for_line(slugs[i]) == CollectiveBucket::Identity)
            .unwrap();
        let sites = fixture_sites();
        let inst = fixture_institutions();
        let mut registry = MemeRegistry::default();
        // Epoch 2: first crossing.
        field.lines[idx].stage = 2.1;
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(10),
            &inst,
            &sites,
        );
        // Epoch 3: second crossing (one epoch per call — i273 semantics).
        field.lines[idx].stage = 3.1;
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(20),
            &inst,
            &sites,
        );
        assert_eq!(registry.memes.len(), 2);
        assert!(registry.memes[0].description.contains("Village Well"));
        assert!(registry.memes[1].description.contains("Village Well"));
        assert!(registry.memes[0]
            .description
            .contains("[genesis:Identity:2]"));
        assert!(registry.memes[1]
            .description
            .contains("[genesis:Identity:3]"));
    }
}
