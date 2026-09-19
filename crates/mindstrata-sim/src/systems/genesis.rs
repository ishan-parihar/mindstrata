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

/// Genesis templates per bucket, laddered by collective stage band
/// (Iteration-277, WP-I tetra-arising gate: "collective stage bands gate
/// WHICH content classes the generator may emit"). Each row is
/// (bucket, min_band, content, template, emotional, identity) where min_band
/// is the stage the bucket's deepest line must reach before THIS class
/// unlocks. Band semantics: band II (2 ≤ s < 4) = foundational founding-
/// memory classes only; band III (4 ≤ s < 6) adds institutional-political /
/// communal-celebration classes; band IV (s ≥ 6) adds reflective classes.
/// Midpoint-neutral by construction: below stage 2 nothing is emitted at
/// all (i273 identity floor), and the pre-277 behavior is exactly band II.
/// {ref} = gross citation; {line} = individual line signature.
const GENESIS_TEMPLATES: &[(CollectiveBucket, f64, MemeContent, &str, f64, f64)] = &[
    // ── Band II: foundational (unlock at 2.0) ──
    (
        CollectiveBucket::Relational,
        2.0,
        MemeContent::Historical,
        "We remember when the {ref} first bound us together through the {line}",
        0.5,
        0.8,
    ),
    (
        CollectiveBucket::Safety,
        2.0,
        MemeContent::Moral,
        "The {ref} and the {line} keep the peace our elders won",
        0.4,
        0.7,
    ),
    (
        CollectiveBucket::Identity,
        2.0,
        MemeContent::Historical,
        "The {ref} is what our people have suffered and kept — the {line} remembers",
        0.6,
        0.9,
    ),
    (
        CollectiveBucket::Meaning,
        2.0,
        MemeContent::Theological,
        "Through the {ref} the {line} glimpses what endures",
        0.7,
        0.6,
    ),
    // ── Band III: institutional-political / communal celebration (4.0) ──
    (
        CollectiveBucket::Safety,
        4.0,
        MemeContent::Political,
        "The {ref} must answer to the {line} of this village",
        0.6,
        0.7,
    ),
    (
        CollectiveBucket::Identity,
        4.0,
        MemeContent::Song,
        "Sing of the {ref}, where the {line} of our people is kept",
        0.6,
        0.9,
    ),
    // ── Band IV: reflective (6.0) ──
    (
        CollectiveBucket::Meaning,
        6.0,
        MemeContent::Prophecy,
        "From the {ref} the {line} speaks: what endures is not yet seen",
        0.8,
        0.7,
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
    genesis_for_namespace(
        field,
        registry,
        params_virality_scaling,
        tick,
        institutions,
        sites,
        None,
    );
}

/// Namespaced variant (Iter-297): `Some(ns)` scopes the dedup tag to one
/// culture (`[genesis:p0:Bucket:Class:Epoch]`), so per-polity genesis passes
/// dedup independently in the shared registry. `None` is the legacy
/// whole-village tag — byte-identical output for unassigned worlds.
fn genesis_for_namespace(
    field: &CollectiveField,
    registry: &mut MemeRegistry,
    params_virality_scaling: mindstrata_core::fixed::Fixed,
    tick: Tick,
    institutions: &[Institution],
    sites: &[Site],
    namespace: Option<&str>,
) {
    let slugs = CollectiveField::line_slugs();
    // Precompute each bucket's deepest line stage once (O(buckets × lines)).
    let mut bucket_stage = [0.0_f64; 4];
    for (i, line) in field.lines.iter().enumerate() {
        if i >= slugs.len() {
            break;
        }
        let b = bucket_for_line(slugs[i]) as usize;
        if line.stage > bucket_stage[b] {
            bucket_stage[b] = line.stage;
        }
    }
    for (bucket, min_band, content, template, emotional, identity) in GENESIS_TEMPLATES {
        // Tetra-arising gate (i277): this content class emits only when the
        // bucket's deepest collective line has reached the class's band.
        let max_stage = bucket_stage[*bucket as usize];
        if max_stage < *min_band {
            continue;
        }
        // Stage epoch: which integer stage crossing this meme commemorates.
        // epoch 1 = first advance past founding (stage 2.x), etc. Cap at 9
        // keeps tags bounded; stages 11+ would need a new epoch anyway.
        let epoch = (max_stage as usize).min(9);
        // i277: tag carries the content class so band-II and band-III/IV
        // classes dedup independently (a Moral epoch-4 meme must not block
        // the Political epoch-4 unlock).
        let class_tag = format!("{content:?}");
        // Iter-297: a culture namespace disambiguates polities that reach the
        // same (bucket, class, epoch) — each polity commemorates ITS OWN
        // crossing with ITS territory's referents. Built only on fire (after
        // the band gate), so non-firing ticks allocate nothing.
        let tag = match namespace {
            None => format!("{GENESIS_TAG}{bucket:?}:{class_tag}:{epoch}]"),
            Some(ns) => format!("{GENESIS_TAG}{ns}:{bucket:?}:{class_tag}:{epoch}]"),
        };
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

/// Iter-297 (UM-3 leg 1): territory-anchored per-polity genesis.
///
/// The i296 per-polity holons give partitioned villages divergent stage
/// trajectories (measured: Safety 3.000 vs 4.000), but genesis still cited
/// the WHOLE village's referent pools — divergent cultures would generate
/// identical founding memories from identical templates + referents. This
/// pass gives each polity its own referent view:
///
/// **Anchoring law (deterministic, zero RNG, total):** every entity anchors
/// to exactly ONE polity —
/// - *Institutions*: member-majority (strict majority of `members` in the
///   polity; ties/empty stay with the first polity in list order only if that
///   majority exists, else no owner → whole-village fallback below).
/// - *Sites*: the polity whose members' home-site centroid is nearest in
///   Manhattan distance (works for communal sites — temple/well/square have
///   no residents of their own, so residence-majority cannot own them).
///
/// Sites of kind House are excluded from genesis pools as before (i276 law:
/// private dwellings are not the village's self-image).
///
/// Legacy contract preserved: with no polities assigned (`polity_fields`
/// empty) nothing runs; with polities assigned, the whole-village genesis
/// call in `tick()` still runs FIRST in tick order and wins the shared-registry
/// dedup on (bucket, class, epoch) tags — so single-polity worlds reproduce
/// byte-identical meme sets, and multi-polity worlds gain polity-specific
/// memes only for epochs the whole-village pass did not register.
///
/// Uses agent positions (`home_site` tile coordinates) as the spatial anchor.
pub fn system_polity_genesis(
    fields: &[CollectiveField],
    members: &[Vec<usize>],
    registry: &mut MemeRegistry,
    params_virality_scaling: mindstrata_core::fixed::Fixed,
    tick: Tick,
    institutions: &[Institution],
    sites: &[Site],
    site_positions: &[(i32, i32)],
    agent_home_site: &[Option<usize>],
) {
    if fields.is_empty() || members.len() != fields.len() {
        return;
    }
    // Per-polity home-site centroid (integer mean; deterministic tie-break by
    // first occurrence).
    let mut centroids: Vec<(i64, i64)> = Vec::with_capacity(members.len());
    for polity in members {
        let (mut sx, mut sy, mut n) = (0i64, 0i64, 0i64);
        for &agent in polity {
            if let Some(site) = agent_home_site.get(agent).and_then(|h| *h) {
                if let Some((x, y)) = site_positions.get(site) {
                    sx += *x as i64;
                    sy += *y as i64;
                    n += 1;
                }
            }
        }
        centroids.push(if n > 0 { (sx / n, sy / n) } else { (0, 0) });
    }

    // Site → nearest-centroid polity (Manhattan; exact tie → lowest polity
    // index). Deterministic and total.
    let site_owner: Vec<Option<usize>> = sites
        .iter()
        .map(|site| {
            if site.kind == SiteKind::House {
                return None; // never a genesis referent anyway
            }
            site_positions
                .get(
                    sites
                        .iter()
                        .position(|s| s.id == site.id)
                        .unwrap_or(usize::MAX),
                )
                .and_then(|(x, y)| {
                    let (mut best, mut best_d) = (None, i64::MAX);
                    for (pid, (cx, cy)) in centroids.iter().enumerate() {
                        let d = (*x - *cx as i32).abs() as i64 + (*y - *cy as i32).abs() as i64;
                        if d < best_d {
                            best_d = d;
                            best = Some(pid);
                        }
                    }
                    best
                })
        })
        .collect();

    for (pid, (field, polity)) in fields.iter().zip(members.iter()).enumerate() {
        // Institutions: strict member-majority owned by this polity, plus
        // no-owner institutions (shared order) — every polity cites those.
        let inst_view: Vec<Institution> = institutions
            .iter()
            .filter(|inst| {
                let total = inst.members.len();
                if total == 0 {
                    return true; // shared
                }
                let in_polity = inst
                    .members
                    .iter()
                    .filter(|m| polity.contains(&(m.as_u64() as usize)))
                    .count();
                in_polity * 2 > total
            })
            .cloned()
            .collect();
        // Sites: owned by this polity (nearest centroid), Houses excluded.
        let site_view: Vec<Site> = sites
            .iter()
            .enumerate()
            .filter(|(si, site)| site.kind != SiteKind::House && site_owner[*si] == Some(pid))
            .map(|(_, s)| s.clone())
            .collect();

        genesis_for_namespace(
            field,
            registry,
            params_virality_scaling,
            tick,
            &inst_view,
            &site_view,
            Some(&format!("p{pid}")),
        );
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
            .contains("[genesis:Relational:Historical:2]"));

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
            .contains("[genesis:Relational:Historical:3]"));
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
            .contains("[genesis:Identity:Historical:2]"));
        assert!(registry.memes[1]
            .description
            .contains("[genesis:Identity:Historical:3]"));
    }

    #[test]
    fn tetra_arising_band_gate_blocks_and_unlocks_classes() {
        // i277: the Political (Safety) class unlocks at stage 4; below the
        // band it must not emit, at the band it must.
        let mut field = CollectiveField::default();
        let slugs = CollectiveField::line_slugs();
        let safety_idx = (0..slugs.len())
            .find(|&i| bucket_for_line(slugs[i]) == CollectiveBucket::Safety)
            .unwrap();
        let inst = fixture_institutions();
        let sites = fixture_sites();

        // Stage 3.x: band II only — the Moral founding class emits per
        // stage epoch, but NO Political meme may exist.
        field.lines[safety_idx].stage = 3.5;
        let mut registry = MemeRegistry::default();
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(10),
            &inst,
            &sites,
        );
        assert!(
            !registry
                .memes
                .iter()
                .any(|m| m.content_type == MemeContent::Political),
            "Political class must stay gated below stage 4"
        );

        // Stage 4.1: band III unlocks — Political may now emit.
        field.lines[safety_idx].stage = 4.1;
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(20),
            &inst,
            &sites,
        );
        assert!(
            registry
                .memes
                .iter()
                .any(|m| m.content_type == MemeContent::Political),
            "crossing stage 4 must unlock the Political class"
        );
    }

    #[test]
    fn tetra_arising_band_iv_reflective_class_unlocks_at_six() {
        // Meaning's Prophecy class unlocks at stage 6 (band IV).
        let mut field = CollectiveField::default();
        let slugs = CollectiveField::line_slugs();
        let meaning_idx = (0..slugs.len())
            .find(|&i| bucket_for_line(slugs[i]) == CollectiveBucket::Meaning)
            .unwrap();
        let inst = fixture_institutions();
        let sites = fixture_sites();

        field.lines[meaning_idx].stage = 5.9;
        let mut registry = MemeRegistry::default();
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(10),
            &inst,
            &sites,
        );
        assert!(
            !registry
                .memes
                .iter()
                .any(|m| m.content_type == MemeContent::Prophecy),
            "Prophecy must stay gated below stage 6"
        );

        field.lines[meaning_idx].stage = 6.1;
        system_collective_genesis(
            &field,
            &mut registry,
            virality(),
            Tick::new(20),
            &inst,
            &sites,
        );
        assert!(
            registry
                .memes
                .iter()
                .any(|m| m.content_type == MemeContent::Prophecy),
            "crossing stage 6 must unlock the Prophecy class"
        );
    }

    // ── Iter-297 (UM-3 leg 1): territory-anchored per-polity genesis ──────

    /// Two spatially separated clusters (polities A at x≈2, B at x≈12) with
    /// communal sites near each cluster plus a centered shared site.
    fn partitioned_world() -> (
        Vec<Institution>,
        Vec<Site>,
        Vec<(i32, i32)>,
        Vec<Option<usize>>,
    ) {
        use mindstrata_core::id::EntityId;
        use mindstrata_institutions::institutions::InstitutionKind;
        let sites = vec![
            Site {
                // 0: well near cluster A (x=2)
                id: EntityId::new(0),
                kind: SiteKind::Well,
                name: "East Well".into(),
                owner: None,
                capacity: 10,
                storage_capacity: Fixed::from_f64(100.0),
                inventory: Vec::new(),
            },
            Site {
                // 1: temple near cluster B (x=12)
                id: EntityId::new(1),
                kind: SiteKind::Temple,
                name: "West Shrine".into(),
                owner: None,
                capacity: 10,
                storage_capacity: Fixed::from_f64(100.0),
                inventory: Vec::new(),
            },
            Site {
                // 2: centered shared square
                id: EntityId::new(2),
                kind: SiteKind::Square,
                name: "Meeting Square".into(),
                owner: None,
                capacity: 20,
                storage_capacity: Fixed::from_f64(100.0),
                inventory: Vec::new(),
            },
            Site {
                // 3: house (never a referent)
                id: EntityId::new(3),
                kind: SiteKind::House,
                name: "House A1".into(),
                owner: None,
                capacity: 1,
                storage_capacity: Fixed::from_f64(10.0),
                inventory: Vec::new(),
            },
        ];
        let positions = vec![(2, 2), (12, 2), (7, 7), (2, 3)];
        // 4 agents: 0,1 homed at site 0 (east); 2,3 homed at site 1 (west).
        let homes = vec![Some(0), Some(0), Some(1), Some(1)];
        let inst = vec![Institution::new(
            0,
            InstitutionKind::Council,
            "East Council".into(),
        )];
        (inst, sites, positions, homes)
    }

    /// Advance every Identity line to a genesis-firing stage on `field`.
    fn fire_identity(field: &mut CollectiveField) {
        let slugs = CollectiveField::line_slugs();
        for (i, line) in field.lines.iter_mut().enumerate() {
            if i < slugs.len() && bucket_for_line(slugs[i]) == CollectiveBucket::Identity {
                line.stage = 2.1;
            }
        }
    }

    #[test]
    fn polity_genesis_routes_sites_to_the_owning_territory() {
        let (inst, sites, positions, homes) = partitioned_world();
        let mut field_east = CollectiveField::default();
        fire_identity(&mut field_east);
        let mut field_west = CollectiveField::default();
        fire_identity(&mut field_west);

        let mut registry = MemeRegistry::default();
        system_polity_genesis(
            &[field_east, field_west],
            &[vec![0, 1], vec![2, 3]],
            &mut registry,
            virality(),
            Tick::new(10),
            &inst,
            &sites,
            &positions,
            &homes,
        );

        // Both polities' identity lines fire → exactly 2 memes (one per
        // polity: the whole-village pass is not part of this unit call, so
        // no dedup collision).
        assert_eq!(
            registry.memes.len(),
            2,
            "each polity generates its own identity meme"
        );
        // Polity A (east, centroid (2,2.5)) owns the East Well; polity B
        // (west, centroid (12,2.5)) owns the West Shrine.
        let east = registry.memes[0].description.clone();
        let west = registry.memes[1].description.clone();
        assert!(
            east.contains("East Well") && !east.contains("West Shrine"),
            "east polity must cite ITS well, not the foreign shrine: `{east}`"
        );
        assert!(
            west.contains("West Shrine") && !west.contains("East Well"),
            "west polity must cite ITS shrine, not the foreign well: `{west}`"
        );
    }

    #[test]
    fn polity_genesis_is_inert_without_assignment() {
        let (inst, sites, positions, homes) = partitioned_world();
        let mut registry = MemeRegistry::default();
        system_polity_genesis(
            &[], // no polities assigned → identity at isolation (zero blast)
            &[],
            &mut registry,
            virality(),
            Tick::new(10),
            &inst,
            &sites,
            &positions,
            &homes,
        );
        assert!(
            registry.memes.is_empty(),
            "empty polity list must be a no-op (legacy single-village contract)"
        );
    }

    #[test]
    fn polity_genesis_single_all_agent_polity_matches_whole_village() {
        // Identity-at-isolation for the REFERENT leg: one polity covering all
        // agents owns every non-House site (nearest centroid = their own), so
        // its genesis output equals the whole-village pool.
        let (inst, sites, positions, homes) = partitioned_world();
        let mut field = CollectiveField::default();
        fire_identity(&mut field);

        let mut via_polity = MemeRegistry::default();
        system_polity_genesis(
            &[field],
            &[vec![0, 1, 2, 3]],
            &mut via_polity,
            virality(),
            Tick::new(10),
            &inst,
            &sites,
            &positions,
            &homes,
        );

        let mut whole_field = CollectiveField::default();
        fire_identity(&mut whole_field);
        let mut whole = MemeRegistry::default();
        system_collective_genesis(
            &whole_field,
            &mut whole,
            virality(),
            Tick::new(10),
            &inst,
            &sites,
        );

        assert_eq!(
            via_polity.memes.len(),
            whole.memes.len(),
            "single all-agent polity must fire the same classes"
        );
        for (p, w) in via_polity.memes.iter().zip(whole.memes.iter()) {
            // Compare the RENDERED text (before the dedup tag): the namespace
            // suffix `[genesis:p0:...]` vs `[genesis:...]` is bookkeeping, the
            // contract is that the pools coincide so the text is identical.
            fn strip(d: &str) -> &str {
                d.split(" [").next().unwrap_or(d)
            }
            assert_eq!(
                strip(&p.description),
                strip(&w.description),
                "referent pools must coincide at isolation: `{}` vs `{}`",
                p.description,
                w.description
            );
        }
    }
}
