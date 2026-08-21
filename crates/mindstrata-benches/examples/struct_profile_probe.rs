//! Iteration 218 — Structural profiling probe.
//! Measures struct sizes, identifies cache-unfriendly patterns, and
//! catalogs O(N²) hot paths in the tick loop.
//!
//! Run with: `cargo run -p mindstrata-benches --example struct_profile_probe --release`

fn main() {
    println!("=== STRUCT SIZES ===");

    // Core types from person.rs (all pub)
    println!(
        "  Personality:             {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::person::Personality>()
    );
    println!(
        "  Temperament:             {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::person::Temperament>()
    );
    println!(
        "  BodyState:               {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::person::BodyState>()
    );
    println!(
        "  NeedState:               {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::person::NeedState>()
    );
    println!(
        "  Affect:                  {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::person::Affect>()
    );
    println!(
        "  DiscreteEmotions:        {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::person::DiscreteEmotions>()
    );
    println!(
        "  CognitiveState:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::person::CognitiveState>()
    );

    // Biology
    println!(
        "  EmbodiedState:           {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::biology::EmbodiedState>()
    );

    // Psychology sub-types
    println!(
        "  NarrativeIdentity:       {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::narrative::NarrativeIdentity>()
    );
    println!(
        "  AttachmentSystem:        {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::attachment::AttachmentSystem>()
    );
    println!(
        "  MoralCognition:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::moral_cognition::MoralCognition>()
    );
    println!(
        "  SelfModel:               {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::self_model::SelfModel>()
    );
    println!(
        "  DecisionPolicy:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::decision_policy::DecisionPolicy>()
    );
    println!(
        "  ProspectionState:        {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::imagination::ProspectionState>()
    );
    println!(
        "  EmotionRegulationState:  {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::emotion_regulation::EmotionRegulationState>(
        )
    );
    println!(
        "  DevelopmentalPsychState: {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::developmental::DevelopmentalPsychState>()
    );
    println!(
        "  CulturalCognition:       {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::cultural_cognition::CulturalCognition>()
    );
    println!(
        "  SkillState:              {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::psychology::skill::SkillState>()
    );

    // Social
    println!(
        "  RelationshipV2:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::social::relationship_v2::RelationshipV2>()
    );
    println!(
        "  EpistemicState:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::social::epistemic::EpistemicState>()
    );

    // Culture
    println!(
        "  SacredValues:            {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::culture::sacred::SacredValues>()
    );
    println!(
        "  EducationState:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::culture::education::EducationState>()
    );
    println!(
        "  NarrativeFrameSet:       {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::culture::narrative_frame::NarrativeFrameSet>()
    );

    // Other
    println!(
        "  AttentionState:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::attention::AttentionState>()
    );
    println!(
        "  MemoryStore:             {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::memory::MemoryStore>()
    );
    println!(
        "  AgentTierState:          {:>8} bytes",
        std::mem::size_of::<mindstrata_sim::agent_tier::AgentTierState>()
    );

    println!();
    println!("=== CACHE-LINE ANALYSIS ===");
    let cache_line = 64u64;
    let body_emotions = std::mem::size_of::<mindstrata_sim::person::BodyState>()
        + std::mem::size_of::<mindstrata_sim::person::DiscreteEmotions>();
    println!("  x86_64 cache line:             {cache_line} bytes");
    println!("  BodyState + DiscreteEmotions:  {body_emotions} bytes (hot fields)");
    println!(
        "  → agents per cache line (hot): {:.1}",
        cache_line as f64 / body_emotions as f64
    );
    println!();

    println!("=== O(N²) HOT PATHS IN TICK LOOP ===");
    println!("  1. Trust sync:           iterates ALL relationships per agent");
    println!("     → O(N × R) where R = avg relationships per agent");
    println!("  2. Social support scan:  linear scan of ALL relationships per agent");
    println!("     → O(N × R) — could use sorted top-3 heap instead");
    println!("  3. Kinship BFS:          transitive coefficient scan for ALL pairs");
    println!("     → O(N²) per agent daily → O(N³) per day");
    println!("  4. Power balance:        nested agent × relationship loop");
    println!("     → O(N × R) per day");
    println!("  5. Relationship decay:   iterates all relationships per agent");
    println!("     → O(N × R)");
    println!("  6. Status institutional: scans ALL institutions per agent");
    println!("     → O(N × I × Roles)");
    println!();

    println!("=== PER-AGENT TICK WORK (from code analysis) ===");
    println!("  Biology tick_update:     ~30 lines arithmetic");
    println!("  Embodied sync:           ~5 lines");
    println!("  Cognitive update:        ~3 lines");
    println!("  Executive function:      ~5 lines (focal only)");
    println!("  Attachment decay:        ~10 lines (daily only)");
    println!("  Motivation update:       ~15 lines");
    println!("  Emotion regulation:      ~20 lines (focal only)");
    println!("  Moral cognition:         ~15 lines (focal only)");
    println!("  Prospection:             ~25 lines (focal only)");
    println!("  Narrative identity:      ~40 lines (focal only)");
    println!("  Developmental:           ~5 lines (centum only)");
    println!("  Psychopathology:         ~10 lines (focal only)");
    println!("  Cultural cognition:      ~5 lines (daily, focal only)");
    println!("  Decision policy:         ~3 lines (daily, focal only)");
    println!("  Skills update:           ~5 lines");
    println!("  Status + attraction:     ~30 lines");
    println!("  Relationship decay:      ~5 lines");
    println!("  ─────────────────────────────────────");
    println!("  Total per-agent:         ~230 lines (but ~180 are focal-only)");
    println!();

    println!("=== ALLOCATION HOTSPOTS (per tick) ===");
    println!("  1. trust_deltas Vec<Vec<(u64, Fixed)>> — rebuilt EVERY tick");
    println!("     → Pre-allocate once, clear+refill per tick");
    println!("  2. agent_ages Vec<Fixed> — rebuilt EVERY tick");
    println!("     → Store on Agent, update on birthday only");
    println!("  3. reg_strategies Vec<RegulationStrategy> — rebuilt EVERY tick");
    println!("     → Pre-allocate, already uses with_capacity (good)");
    println!("  4. rel_snapshot Vec<(AgentId, AgentId, Fixed, Fixed)> — rebuilt EVERY tick");
    println!("     → Pre-allocate once, clear+refill per tick");
    println!("  5. action_starts Vec<(usize, ActionKind)> — rebuilt EVERY tick");
    println!("     → Pre-allocate once, clear+refill per tick");
    println!("  6. personalities Vec<Personality> — cloned EVERY tick (read-only)");
    println!("     → Could use Rc<Personality> to avoid clone");
    println!();

    println!("=== RECOMMENDATIONS ===");
    println!("  PRIORITY 1 (high impact):");
    println!("    • Pre-allocate trust_deltas, agent_ages, rel_snapshot, action_starts");
    println!("    • These 4 Vecs are created+dropped every tick — ~4 heap allocs/tick");
    println!("    • At 100K ticks: 400K unnecessary heap allocations");
    println!();
    println!("  PRIORITY 2 (medium impact):");
    println!("    • Replace linear relationship scan in social_support with indexed lookup");
    println!("    • The scan iterates ALL relationships to find top-3 for ONE agent");
    println!("    • Could maintain a per-agent sorted edge list instead of flat Vec");
    println!();
    println!("  PRIORITY 3 (low impact, high complexity):");
    println!("    • Agent struct is large — iterating 24 agents means large working set");
    println!("    • Consider SoA (Structure of Arrays) for hot fields (body, emotions)");
    println!("    • Would improve vectorization and cache utilization in per-agent loops");
    println!();
    println!("  PRIORITY 4 (measurement needed):");
    println!("    • Personality clone: 12 agents × estimated ~200 bytes = ~2.4KB/tick");
    println!("    • Low per-tick cost but high cumulative — consider Rc<Personality>");
}
