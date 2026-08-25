//! Pure rendering functions: world map, dashboards and inspectors.
//!
//! Mindstrata TUI — debug instrument for observing simulations.
//!
//! Provides ASCII world map, agent list, event log, and system dashboard,
//! plus the interactive session state (view tabs, selection, command keys)
//! consumed by the `mindstrata-tui` binary's event loop.

use crate::charts;
use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use mindstrata_sim::culture::{EchoChamberState, MemeRegistry, PropagandaRegistry, RumorRegistry};
use mindstrata_sim::institutions::{Institution, InstitutionalRecord};
use mindstrata_sim::market::MarketState;
use mindstrata_sim::military::MilitaryRegistry;
use mindstrata_sim::noosphere::{LegitimacyField, MoralPanicRegistry, NoosphericField};
use mindstrata_sim::person::{Belief, Relationship};
use mindstrata_sim::provenance::CausalProvenance;
use mindstrata_sim::psychology::attachment::{AttachmentStyle, CaregivingStyle};
use mindstrata_sim::sim::AgentSummary;
use mindstrata_sim::social::clan::ClanRegistry;
use mindstrata_sim::social::patronage::PatronageRegistry;
use mindstrata_sim::theology::{TheologicalBelief, TheologyRegistry};
use mindstrata_sim::world::{SiteKind, Terrain, World};

/// §6: Agent position marker for map rendering.
pub struct AgentMarker {
    /// Index of the agent in the simulation's agent list.
    pub index: usize,
    /// X coordinate on the world grid.
    pub x: i32,
    /// Y coordinate on the world grid.
    pub y: i32,
    /// Single character used to represent the agent on the map.
    pub name: char,
}

/// Render the ASCII world map with optional agent markers.
pub fn render_world_map(world: &World, agent_markers: &[AgentMarker]) -> String {
    let mut out = String::new();
    out.push_str("  ");
    for x in 0..world.width {
        out.push_str(&format!("{:1}", x % 10));
    }
    out.push('\n');

    for y in 0..world.height {
        out.push_str(&format!("{:1} ", y % 10));
        for x in 0..world.width {
            // Check if an agent is at this position
            let agent_at_pos: Option<&AgentMarker> = agent_markers
                .iter()
                .find(|m| m.x == x as i32 && m.y == y as i32);

            if let Some(marker) = agent_at_pos {
                out.push(marker.name);
            } else if let Some(tile) = world.tile(x as i32, y as i32) {
                let ch = if tile.site.is_some() {
                    // Find site kind
                    world
                        .sites
                        .iter()
                        .find(|s| tile.site == Some(s.id))
                        .map_or('?', |s| match s.kind {
                            SiteKind::House => 'H',
                            SiteKind::Farm => 'F',
                            SiteKind::Well => 'W',
                            SiteKind::Market => 'M',
                            SiteKind::Temple => 'T',
                            SiteKind::Barracks => 'B',
                            SiteKind::Workshop => 'K',
                            SiteKind::Square => 'S',
                            SiteKind::Prison => 'P',
                            SiteKind::School => 'L',
                        })
                } else {
                    match tile.terrain {
                        Terrain::Grassland => '.',
                        Terrain::Forest => '♣',
                        Terrain::Hill => '^',
                        Terrain::Mountain => '△',
                        Terrain::Water => '~',
                        Terrain::Desert => ':',
                        Terrain::Swamp => '%',
                    }
                };
                out.push(ch);
            } else {
                out.push('?');
            }
        }
        out.push('\n');
    }
    out
}

/// Render the agent list as a table.
pub fn render_agent_list(agents: &[AgentSummary]) -> String {
    let mut out = String::new();
    out.push_str(&format!(
        "{:<3} {:<8} {:>5} {:>5} {:>5} {:>5} {:>6} {:>6} {:>6} {:<12} {:>5} {:<3}\n",
        "ID", "Name", "H", "T", "F", "HP", "Val", "Joy", "Fear", "Action", "Attn", "Int"
    ));
    out.push_str(&format!("{}\n", "─".repeat(80)));

    for s in agents {
        out.push_str(&format!(
            "{:<3} {:<8} {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>+5.2} {:>5.2} {:>5.2} {:<12} {:>5.2} {:<3}\n",
            s.index,
            s.name,
            s.hunger.to_f64(),
            s.thirst.to_f64(),
            s.fatigue.to_f64(),
            s.health.to_f64(),
            s.valence.to_f64(),
            s.joy.to_f64(),
            s.fear.to_f64(),
            s.current_action,
            s.attention_budget.to_f64(),
            if s.has_intention { "Y" } else { "-" },
        ));
    }
    out
}

/// Render the event log (last n events) — delegates to detailed version.
pub fn render_event_log(events: &[SimEvent], n: usize) -> String {
    render_event_log_detailed(events, n)
}

/// §8.1.14: Human-readable label for an attachment style.
fn attachment_style_name(style: AttachmentStyle) -> &'static str {
    match style {
        AttachmentStyle::Secure => "Secure",
        AttachmentStyle::Anxious => "Anxious",
        AttachmentStyle::Avoidant => "Avoidant",
        AttachmentStyle::Disorganized => "Disorganized",
    }
}

/// §8.1.14: Human-readable label for a caregiving style.
fn caregiving_style_name(style: CaregivingStyle) -> &'static str {
    match style {
        CaregivingStyle::Sensitive => "Sensitive",
        CaregivingStyle::Intrusive => "Intrusive",
        CaregivingStyle::Dismissive => "Dismissive",
        CaregivingStyle::Frightening => "Frightening",
    }
}

/// §17.1: Render agent inspector — detailed single-agent view.
pub fn render_agent_inspector(summary: &AgentSummary, relationships: &[Relationship]) -> String {
    let agent_id = AgentId::new(summary.index as u64);
    let mut out = String::new();
    out.push_str(&format!(
        "╔══════════════════════════════════════════╗\n\
         ║  Agent Inspector                        ║\n\
         ╚══════════════════════════════════════════╝\n\
         Agent: {} (ID {})\n\
         ── Body ──\n\
         Health:   {:5.2}    Energy:   {:5.2}\n\
         ── Needs ──\n\
         Hunger:   {:5.2}    Thirst:   {:5.2}\n\
         Fatigue:  {:5.2}\n\
         ── Emotions ──\n\
         Valence:  {:>+5.2}   Joy:      {:5.2}\n\
         Fear:     {:5.2}    Anger:    {:5.2}\n\
         ── State ──\n\
         Action:   {}\n\
         Intention: {}
\
         Attention: {:5.2}\n",
        summary.name,
        summary.index,
        summary.health.to_f64(),
        summary.energy.to_f64(),
        summary.hunger.to_f64(),
        summary.thirst.to_f64(),
        summary.fatigue.to_f64(),
        summary.valence.to_f64(),
        summary.joy.to_f64(),
        summary.fear.to_f64(),
        summary.anger.to_f64(),
        summary.current_action,
        if summary.has_intention {
            "active"
        } else {
            "none"
        },
        summary.attention_budget.to_f64(),
    ));

    // Show attachment system state (§8.1.14) — style, caregiving, security, anxiety,
    // avoidance, protest threshold, soothing receptivity, separation distress.
    let style_str = attachment_style_name(summary.attachment_style);
    let caregiving_str = caregiving_style_name(summary.attachment_caregiving_style);
    let sec_bar_len = (summary.attachment_security.to_f64() * 20.0) as usize;
    let sec_bar: String = "█".repeat(sec_bar_len.min(20)) + &"░".repeat(20 - sec_bar_len.min(20));
    let anx_bar_len = (summary.attachment_anxiety.to_f64() * 20.0) as usize;
    let anx_bar: String = "█".repeat(anx_bar_len.min(20)) + &"░".repeat(20 - anx_bar_len.min(20));
    let avo_bar_len = (summary.attachment_avoidance.to_f64() * 20.0) as usize;
    let avo_bar: String = "█".repeat(avo_bar_len.min(20)) + &"░".repeat(20 - avo_bar_len.min(20));
    let prot_bar_len = (summary.attachment_protest_threshold.to_f64() * 20.0) as usize;
    let prot_bar: String =
        "█".repeat(prot_bar_len.min(20)) + &"░".repeat(20 - prot_bar_len.min(20));
    let sooth_bar_len = (summary.attachment_soothing_receptivity.to_f64() * 20.0) as usize;
    let sooth_bar: String =
        "█".repeat(sooth_bar_len.min(20)) + &"░".repeat(20 - sooth_bar_len.min(20));
    let dis_bar_len = (summary.attachment_separation_distress.to_f64() * 20.0) as usize;
    let dis_bar: String = "█".repeat(dis_bar_len.min(20)) + &"░".repeat(20 - dis_bar_len.min(20));
    out.push_str(&format!(
        "\n── Attachment ──\n\
         Style:      {style_str}\n\
         Caregiving: {caregiving_str}\n\
         Security:   {:5.2} [{}]\n\
         Anxiety:    {:5.2} [{}]\n\
         Avoidance:  {:5.2} [{}]\n\
         Protest thr:{:5.2} [{}]\n\
         Soothing:   {:5.2} [{}]\n\
         Sep. dist:  {:5.2} [{}]\n",
        summary.attachment_security.to_f64(),
        sec_bar,
        summary.attachment_anxiety.to_f64(),
        anx_bar,
        summary.attachment_avoidance.to_f64(),
        avo_bar,
        summary.attachment_protest_threshold.to_f64(),
        prot_bar,
        summary.attachment_soothing_receptivity.to_f64(),
        sooth_bar,
        summary.attachment_separation_distress.to_f64(),
        dis_bar,
    ));

    // Show relationships for this agent
    let rels: Vec<&Relationship> = relationships
        .iter()
        .filter(|r| r.from == agent_id)
        .collect();
    if !rels.is_empty() {
        out.push_str("\n  ── Relationships ──\n\n");
        for r in rels.iter().take(5) {
            out.push_str(&format!(
                "  → Agent {} trust={:.2} affection={:.2}\n",
                r.to.as_u64(),
                r.trust.to_f64(),
                r.affection.to_f64()
            ));
        }
        if rels.len() > 5 {
            out.push_str(&format!("  ... and {} more\n", rels.len() - 5));
        }
    }

    out
}

/// §17.1: Render relationship view between two agents.
pub fn render_relationship_view(
    from_id: AgentId,
    to_id: AgentId,
    relationships: &[Relationship],
    agents: &[AgentSummary],
    v2_edge: Option<&mindstrata_sim::social::relationship_v2::RelationshipV2>,
) -> String {
    let from_name = agents
        .get(from_id.as_u64() as usize)
        .map_or("?", |s| s.name.as_str());
    let to_name = agents
        .get(to_id.as_u64() as usize)
        .map_or("?", |s| s.name.as_str());

    let forward = relationships
        .iter()
        .find(|r| r.from == from_id && r.to == to_id);
    let backward = relationships
        .iter()
        .find(|r| r.from == to_id && r.to == from_id);

    let mut out = String::new();
    out.push_str(&format!(
        "╔══════════════════════════════════════════╗\n\
         ║  Relationship View                      ║\n\
         ╚══════════════════════════════════════════╝\n\
         {from_name} → {to_name}\n"
    ));

    // Iteration 201 (observability closure): the V2 edge exposes the §10.3
    // stage ladder + the continuous within-stage progress toward the next
    // stage (previously a dead field — never produced, never consumed).
    if let Some(v2) = v2_edge {
        out.push_str(&format!(
            "  stage:      {:?} (progress {:.2} → next)\n",
            v2.stage,
            v2.stage_progress.to_f64(),
        ));
    }

    if let Some(r) = forward {
        out.push_str(&format!(
            "  trust:      {:.2}\n\
             affection:  {:.2}\n\
             respect:    {:.2}\n\
             obligation: {:.2}\n",
            r.trust.to_f64(),
            r.affection.to_f64(),
            r.respect.to_f64(),
            r.obligation.to_f64(),
        ));
    } else {
        out.push_str("  (no relationship)\n");
    }

    out.push_str(&format!("\n  {to_name} → {from_name}\n"));
    if let Some(r) = backward {
        out.push_str(&format!(
            "  trust:      {:.2}\n\
             affection:  {:.2}\n\
             respect:    {:.2}\n\
             obligation: {:.2}\n",
            r.trust.to_f64(),
            r.affection.to_f64(),
            r.respect.to_f64(),
            r.obligation.to_f64(),
        ));
    } else {
        out.push_str("  (no relationship)\n");
    }

    out
}

/// Dashboard configuration for the render_dashboard function.
pub struct DashboardConfig {
    /// Current season name (e.g. "Spring").
    pub season: String,
    /// Current year of the simulation.
    pub year: u64,
    /// Total grain in the world.
    pub grain: f64,
    /// Total water in the world.
    pub water: f64,
    /// Number of active institutions.
    pub institution_count: usize,
    /// Number of active factions.
    pub faction_count: usize,
}

/// §17.1: Enhanced system dashboard with season, institutions, factions.
/// Iteration 251: ASCII longitudinal charts over the metric history.
/// DC-1: migrated to the charts component library (lane/sparkline over
/// plain-data Series); sim types stop at the metric_series adapter.
/// Legacy sparkline/series_chart helpers retired on migration (see
/// docs/chart-component-api.md migration map).
/// Longitudinal trends view: the village's vital signs over time.
pub fn render_metric_charts(history: &[mindstrata_sim::sim::MetricsSnapshot]) -> String {
    use crate::charts;
    use crate::charts::Band;
    if history.is_empty() {
        return "No metric history yet — run the simulation.".into();
    }
    const WINDOW: usize = 60;
    let mut out = String::from("── Village Trends (most recent 60 samples) ──\n");
    const KEYS: [(MetricKey, Band); 9] = [
        (MetricKey::Stress, Band::UnitInterval),
        (MetricKey::Health, Band::UnitInterval),
        (MetricKey::FearP90, Band::UnitInterval),
        (MetricKey::JoyP90, Band::UnitInterval),
        (MetricKey::Gini, Band::UnitInterval),
        (MetricKey::BestSkill, Band::UnitInterval),
        (MetricKey::Families, Band::ObservedMax),
        // Iteration 262: trait variance — heredity-stationarity monitor;
        // variance of 0-1 traits tops out at 0.25, flat-at-zero = collapse.
        (MetricKey::TraitVariance, Band::Fixed(0.0, 0.25)),
        (MetricKey::MeanKinship, Band::Fixed(0.0, 0.5)),
    ];
    for (key, band) in &KEYS {
        let mut s = metric_series(history, *key);
        s.band = *band;
        out.push_str(&charts::lane(&s, WINDOW));
        out.push('\n');
    }
    out.push_str(&format!(
        "samples {} · ticks {}..{}\n",
        history.len(),
        history[0].tick,
        history[history.len() - 1].tick
    ));
    out
}

/// Semantic slot for a charted metric (never a raw sim field name).
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum MetricKey {
    Stress,
    Health,
    FearP90,
    JoyP90,
    Gini,
    BestSkill,
    Families,
    TraitVariance,
    MeanKinship,
}

/// Adapter: sim metric history → chart-library [`charts::Series`].
/// This is the ONLY place sim snapshot fields are named for charting.
pub fn metric_series(
    history: &[mindstrata_sim::sim::MetricsSnapshot],
    key: MetricKey,
) -> charts::Series {
    use mindstrata_sim::sim::MetricsSnapshot;
    type Pick = fn(&MetricsSnapshot) -> f64;
    let (name, pick): (&'static str, Pick) = match key {
        MetricKey::Stress => ("stress", |m| m.avg_stress),
        MetricKey::Health => ("health", |m| m.avg_health),
        MetricKey::FearP90 => ("fear p90", |m| m.fear_p90),
        MetricKey::JoyP90 => ("joy p90", |m| m.joy_p90),
        MetricKey::Gini => ("gini", |m| m.gini),
        MetricKey::BestSkill => ("best skill", |m| m.avg_best_skill),
        MetricKey::Families => ("families", |m| m.family_count as f64),
        MetricKey::TraitVariance => ("trait var", |m| m.trait_variance),
        MetricKey::MeanKinship => ("kinship", |m| m.mean_kinship),
    };
    charts::Series {
        name,
        unit: "",
        // Bands are assigned by the view (KEYS table); neutral default here.
        band: charts::Band::UnitInterval,
        samples: history.iter().map(pick).collect(),
    }
}

/// Iteration 261: the village chronicle as a TUI pane — the annals text
/// (already rendered by `sim::chronicle`) framed with a header and trimmed
/// to the most recent `max_lines` lines so long histories stay scroll-free.
pub fn render_chronicle_view(chronicle_text: &str, max_lines: usize) -> String {
    let mut out = String::from("── Village Chronicle ──\n");
    let lines: Vec<&str> = chronicle_text.lines().collect();
    let skip = lines.len().saturating_sub(max_lines);
    for line in &lines[skip..] {
        out.push_str(line);
        out.push('\n');
    }
    if skip > 0 {
        out.push_str(&format!("… {} earlier year headers elided …\n", {
            // count only "Year" headers among the skipped lines for honesty
            lines[..skip].iter().filter(|l| l.contains("Year ")).count()
        }));
    }
    out
}

/// Iteration 264: the selected agent's dossier as a TUI pane. Dossiers are
/// short (identity + lineage + drift + genome + timeline), but a long-lived
/// elder can accumulate a tall timeline — trim from the TOP (keep the most
/// recent lines) exactly like the chronicle pane so the pane stays fixed.
pub fn render_dossier_view(dossier_text: &str, max_lines: usize) -> String {
    let mut out = String::from("── Agent Dossier ──\n");
    let lines: Vec<&str> = dossier_text.lines().collect();
    let skip = lines.len().saturating_sub(max_lines);
    for line in &lines[skip..] {
        out.push_str(line);
        out.push('\n');
    }
    if skip > 0 {
        out.push_str(&format!("… {skip} earlier dossier lines elided …\n"));
    }
    out
}

/// Render the main dashboard: agent summaries, events, tick, metrics.
pub fn render_dashboard(
    agents: &[AgentSummary],
    events_count: usize,
    tick: u64,
    config: &DashboardConfig,
) -> String {
    let n = agents.len() as f64;
    if n == 0.0 {
        return "No agents alive.".into();
    }
    let avg_hunger: f64 = agents.iter().map(|a| a.hunger.to_f64()).sum::<f64>() / n;
    let avg_thirst: f64 = agents.iter().map(|a| a.thirst.to_f64()).sum::<f64>() / n;
    let avg_fatigue: f64 = agents.iter().map(|a| a.fatigue.to_f64()).sum::<f64>() / n;
    let avg_health: f64 = agents.iter().map(|a| a.health.to_f64()).sum::<f64>() / n;
    let avg_valence: f64 = agents.iter().map(|a| a.valence.to_f64()).sum::<f64>() / n;
    let avg_joy: f64 = agents.iter().map(|a| a.joy.to_f64()).sum::<f64>() / n;
    let avg_fear: f64 = agents.iter().map(|a| a.fear.to_f64()).sum::<f64>() / n;

    format!(
        "╔══════════════════════════════════════════╗\n\
         ║  Mindstrata Dashboard                    ║\n\
         ╚══════════════════════════════════════════╝\n\
         Tick:        {tick}\n\
         Season:      {} (year {})\n\
         Agents:      {}\n\
         Events:      {events_count}\n\
         Institutions: {}\n\
         Factions:    {}\n\
         ── Resources ──\n\
         Grain:       {:.1}\n\
         Water:       {:.1}\n\
         ── Population Averages ──\n\
         Hunger:      {avg_hunger:.3}\n\
         Thirst:      {avg_thirst:.3}\n\
         Fatigue:     {avg_fatigue:.3}\n\
         Health:      {avg_health:.3}\n\
         Valence:     {avg_valence:+.3}\n\
         Joy:         {avg_joy:.3}\n\
         Fear:        {avg_fear:.3}\n",
        config.season,
        config.year,
        agents.len(),
        config.institution_count,
        config.faction_count,
        config.grain,
        config.water,
    )
}

// ── §17.1: Market Dashboard ─────────────────────────────────────────────

/// §17.1: Render market dashboard showing prices, inequality, trade volume.
pub fn render_market_dashboard(market: &MarketState) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Market Dashboard                        ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");

    for (i, tracker) in market.prices.iter().enumerate() {
        let resource_name = match i {
            0 => "Grain",
            1 => "Water",
            _ => "Unknown",
        };
        let trend = tracker.trend();
        let trend_char = if trend > Fixed::from_f64(0.1) {
            "↑"
        } else if trend < Fixed::from_f64(-0.1) {
            "↓"
        } else {
            "→"
        };
        out.push_str(&format!(
            "  {:<8} Price: {:5.1} {}  Supply: {:5.1}  Demand: {:5.1}\n",
            resource_name,
            tracker.price.to_f64(),
            trend_char,
            tracker.avg_supply.to_f64(),
            tracker.avg_demand.to_f64(),
        ));
    }

    out.push_str(&format!(
        "\n  ── Market Metrics ──\n\
         Inequality (Gini): {:.3}\n\
         Avg Wealth:        {:.1}\n\
         Median Wealth:     {:.1}\n\
         Trades completed:  {}\n",
        market.inequality.to_f64(),
        market.avg_wealth.to_f64(),
        market.median_wealth.to_f64(),
        market.total_trades,
    ));

    out
}

// ── §17.1: Belief Inspector ────────────────────────────────────────────

/// §17.1: Render an agent's beliefs.
pub fn render_belief_inspector(agent_name: &str, agent_id: usize, beliefs: &[Belief]) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Belief Inspector                        ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");
    out.push_str(&format!("Agent: {agent_name} (ID {agent_id})\n\n"));

    if beliefs.is_empty() {
        out.push_str("  (no beliefs)\n");
        return out;
    }

    out.push_str(&format!(
        "  {:<5} {:<8} {:>8} {:>8} {:>8}\n",
        "ID", "Prop", "Conf", "Emotion", "Identity"
    ));
    out.push_str(&format!("  {}\n", "─".repeat(42)));

    for b in beliefs.iter().take(15) {
        let prop_name = match b.proposition_id {
            0 => "market_fair",
            1 => "ruler_legit",
            2 => "neighbor_trust",
            3 => "foreigners_danger",
            4 => "hard_work_wealth",
            5 => "harvest_fail",
            6 => "temple_corrupt",
            7 => "council_protect",
            8 => "grain_price_high",
            9 => "guards_unjust",
            10 => "violence_necessary",
            11 => "sharing_duty",
            12 => "community_strong",
            13 => "strangers_honest",
            14 => "well_water_safe",
            _ => "unknown",
        };
        out.push_str(&format!(
            "  {:<5} {:<8} {:>8.3} {:>8.3} {:>8.3}\n",
            b.proposition_id,
            prop_name,
            b.confidence.to_f64(),
            b.emotional_charge.to_f64(),
            b.identity_linkage.to_f64(),
        ));
    }
    if beliefs.len() > 15 {
        out.push_str(&format!("  ... and {} more\n", beliefs.len() - 15));
    }

    out
}

// ── §17.1: Faction Dashboard ────────────────────────────────────────────

/// §17.1: Render faction/institution dashboard.
pub fn render_faction_dashboard(institutions: &[Institution]) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Institution Dashboard                   ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");

    if institutions.is_empty() {
        out.push_str("  (no institutions)\n");
        return out;
    }

    for inst in institutions {
        let kind_str = format!("{:?}", inst.kind);
        out.push_str(&format!(
            "  {} [{}]\n\
             │ Legitimacy: {:.3}  Cohesion: {:.3}  Members: {}\n",
            inst.name,
            kind_str,
            inst.legitimacy.to_f64(),
            inst.collective.unity.to_f64(),
            inst.members.len(),
        ));
        out.push_str(&format!(
            "  │ Morale: {:.3}  Enforcement: {:.3}\n\n",
            inst.collective.morale.to_f64(),
            inst.enforcement_capacity.to_f64(),
        ));
    }

    out
}

// ── §10.8: Clan Dashboard ───────────────────────────────────────────

/// §10.8: Render the clan dashboard — membership, prestige, cohesion,
/// grievance, and the alliance/enmity network (forged by marriages in
/// tick_marriage_formation and by feuds in tick_social_cluster).
pub fn render_clan_dashboard(clans: &ClanRegistry) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Clan Dashboard                          ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");

    if clans.clans.is_empty() {
        out.push_str("  (no clans)\n");
        return out;
    }

    for clan in &clans.clans {
        out.push_str(&format!(
            "  Clan {} — {} members\n",
            clan.id,
            clan.core_households.len(),
        ));
        out.push_str(&format!(
            "  │ Prestige: {:.3}  Cohesion: {:.3}  Grievance: {:.3}\n",
            clan.prestige.to_f64(),
            clan.cohesion.to_f64(),
            clan.grievance.to_f64(),
        ));
        if !clan.enemies.is_empty() {
            let enemies: Vec<String> = clan.enemies.iter().map(|e| format!("Clan {e}")).collect();
            out.push_str(&format!("  │ ⚔ Enemies: {}\n", enemies.join(", ")));
        } else {
            out.push_str("  │ ⚔ Enemies: none\n");
        }
        if !clan.allies.is_empty() {
            let allies: Vec<String> = clan.allies.iter().map(|a| format!("Clan {a}")).collect();
            out.push_str(&format!("  │ 🤝 Allies:  {}\n", allies.join(", ")));
        } else {
            out.push_str("  │ 🤝 Allies:  none\n");
        }
        if let Some(founder) = clan.founder_memory {
            out.push_str(&format!("  │ Founder: agent {founder}\n"));
        }
        out.push('\n');
    }

    out
}

// ── §10.9: Patronage Dashboard ───────────────────────────────────────

/// §10.9: Render the patronage dashboard — patron-client relations with
/// provision, loyalty, labor, political support, and satisfaction (the
/// §10.9 economic safety net activated in Iteration 19).
pub fn render_patronage_dashboard(registry: &PatronageRegistry) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Patronage Dashboard                     ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");

    if registry.relations.is_empty() {
        out.push_str("  (no patronage relations)\n");
        return out;
    }

    for rel in &registry.relations {
        if !rel.active {
            continue;
        }
        out.push_str(&format!(
            "  Agent {} → Client {}  [{} days]\n",
            rel.patron, rel.client, rel.duration,
        ));
        out.push_str(&format!(
            "  │ Provision: {:.3}  Loyalty: {:.3}  Satisfaction: {:.3}\n",
            rel.provision.to_f64(),
            rel.loyalty.to_f64(),
            rel.satisfaction.to_f64(),
        ));
        out.push_str(&format!(
            "  │ Labor: {:.3}  Polit. support: {:.3}  Dependence: {:.3}\n",
            rel.labor_contribution.to_f64(),
            rel.political_support.to_f64(),
            rel.client_dependence.to_f64(),
        ));
        out.push('\n');
    }

    out
}

// ── §19.5.J: Observability — Institutional Records ──────────────────

/// §19.5.J: Render institutional records for debugging.
pub fn render_institutional_records(
    institution_name: &str,
    records: &[InstitutionalRecord],
    n: usize,
) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Institutional Records                   ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");
    out.push_str(&format!("Institution: {institution_name}\n\n"));

    if records.is_empty() {
        out.push_str("  (no records)\n");
        return out;
    }

    let start = records.len().saturating_sub(n);
    for r in &records[start..] {
        let status = if r.success { "✓" } else { "✗" };
        out.push_str(&format!(
            "  [{:>5}] {} {} ({} affected)\n",
            r.tick,
            status,
            r.action,
            r.affected.len()
        ));
    }

    out
}

// ── §19.5.J: Observability — Decision Traces ──────────────────────────

/// §19.5.J: Render decision traces for a specific agent.
pub fn render_decision_traces(
    provenance: &CausalProvenance,
    agent_id: AgentId,
    n: usize,
) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Decision Traces                         ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");
    out.push_str(&format!("Agent: {}\n\n", agent_id.as_u64()));

    let traces = provenance.decisions_for_agent(agent_id);
    let start = traces.len().saturating_sub(n);
    if traces[start..].is_empty() {
        out.push_str("  (no decision traces)\n");
        return out;
    }

    for trace in &traces[start..] {
        let routine = if trace.from_routine { " [routine]" } else { "" };
        let interrupted = if trace.interrupted_by_critical_needs {
            " [interrupted]"
        } else {
            ""
        };
        out.push_str(&format!(
            "  [{:>5}] {}{}{}\n",
            trace.tick, trace.action_name, routine, interrupted
        ));
        for factor in &trace.factors {
            out.push_str(&format!(
                "        {}: {:.3} — {}\n",
                factor.kind,
                factor.magnitude.to_f64(),
                factor.description
            ));
        }
    }

    out
}

// ── §17.1: Provenance Timeline ──────────────────────────────────────────

/// §17.1: Render event log with more detail for debugging.
pub fn render_event_log_detailed(events: &[SimEvent], n: usize) -> String {
    let mut out = String::new();
    let start = events.len().saturating_sub(n);
    for (i, ev) in events[start..].iter().enumerate() {
        let tick = start + i;
        match ev {
            SimEvent::AgentAte { agent, .. } => {
                out.push_str(&format!("  [{tick:>5}] 🍞 Agent {} ate\n", agent.as_u64()));
            }
            SimEvent::AgentDrank { agent, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 💧 Agent {} drank\n",
                    agent.as_u64()
                ));
            }
            SimEvent::AgentRested { agent, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 😴 Agent {} rested\n",
                    agent.as_u64()
                ));
            }
            SimEvent::InteractionOccurred { from, to, kind, .. } => {
                let icon = match kind {
                    mindstrata_core::event::InteractionKind::Help => "🤝",
                    mindstrata_core::event::InteractionKind::Threaten => "⚔️",
                    mindstrata_core::event::InteractionKind::Insult => "💢",
                    mindstrata_core::event::InteractionKind::Gossip => "💬",
                    mindstrata_core::event::InteractionKind::Trade => "💰",
                    mindstrata_core::event::InteractionKind::Comfort => "❤️",
                    _ => "→",
                };
                out.push_str(&format!(
                    "  [{tick:>5}] {} Agent {} → Agent {}\n",
                    icon,
                    from.as_u64(),
                    to.as_u64()
                ));
            }
            SimEvent::RelationshipChanged {
                from,
                to,
                trust_delta,
                affection_delta,
                ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 📊 {}→{} trust {:+.3} affection {:+.3}\n",
                    from.as_u64(),
                    to.as_u64(),
                    trust_delta.to_f64(),
                    affection_delta.to_f64()
                ));
            }
            SimEvent::RumorSpread {
                source,
                target,
                content_hash,
                distortion,
                ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 🗣️ Rumor {}→{} prop={} distortion={:.3}\n",
                    source.as_u64(),
                    target.as_u64(),
                    content_hash,
                    distortion.to_f64()
                ));
            }
            SimEvent::KnowledgeTransferred {
                source,
                target,
                knowledge_id,
                ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 📚 Knowledge {}→{} id={}\n",
                    source.as_u64(),
                    target.as_u64(),
                    knowledge_id
                ));
            }
            SimEvent::ConflictOccurred {
                aggressor,
                target,
                kind,
                injury,
                fear_induced,
                ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] ⚔️ Conflict {}→{} {} injury={:.3} fear={:.3}\n",
                    aggressor.as_u64(),
                    target.as_u64(),
                    kind,
                    injury.to_f64(),
                    fear_induced.to_f64()
                ));
            }
            SimEvent::FeudFormed {
                party_a, party_b, ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 🔥 Feud formed {}↔{}\n",
                    party_a.as_u64(),
                    party_b.as_u64()
                ));
            }
            SimEvent::MarriageFormed {
                spouse_a, spouse_b, ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 💍 Marriage {}↔{}\n",
                    spouse_a.as_u64(),
                    spouse_b.as_u64()
                ));
            }
            SimEvent::ChildBorn {
                child,
                parent_a,
                parent_b,
                ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 👶 Child {} born ({} + {})\n",
                    child.as_u64(),
                    parent_a.as_u64(),
                    parent_b.as_u64()
                ));
            }
            SimEvent::NormViolated {
                agent,
                norm_id,
                witnesses,
                ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] ⚖️ Agent {} violated norm {} ({} witnesses)\n",
                    agent.as_u64(),
                    norm_id,
                    witnesses.len()
                ));
            }
            SimEvent::AgentSpawned { agent, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 🌱 Agent {} spawned\n",
                    agent.as_u64()
                ));
            }
            SimEvent::AgentDied { agent, cause, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 💀 Agent {} died ({:?})\n",
                    agent.as_u64(),
                    cause
                ));
            }
            SimEvent::TradeOccurred {
                buyer,
                seller,
                good,
                quantity,
                price,
                ..
            } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 💰 Trade {}→{} good={} qty={:.2} price={:.1}\n",
                    buyer.as_u64(),
                    seller.as_u64(),
                    good,
                    quantity.to_f64(),
                    price.to_f64()
                ));
            }
            _ => {
                out.push_str(&format!("  [{tick:>5}] ❓ {ev:?}\n"));
            }
        }
    }
    out
}

// ── §22: Full Agent Psychology Inspector ────────────────────────────────

/// Full cognitive pipeline state for an agent — needs, emotions, beliefs,
/// identity, moral values, cognitive state, derived mental states, skills,
/// current intention, and active goals.
pub fn render_psychology_inspector(
    agent_index: usize,
    agent_name: &str,
    agent: &mindstrata_sim::sim::AgentBundle,
) -> String {
    use mindstrata_sim::person::{GoalSource, IdentityKind};
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════════════╗\n");
    out.push_str("║  Psychology Inspector — Full Cognitive Pipeline          ║\n");
    out.push_str("╚══════════════════════════════════════════════════════════╝\n\n");
    out.push_str(&format!("Agent: {agent_name} (ID {agent_index})\n\n"));

    // ── Body ──
    out.push_str("── §22.1: Body State ──\n\n");
    out.push_str(&format!(
        "  Health:      {:5.2}   Energy:    {:5.2}\n",
        agent.body.health.to_f64(),
        agent.body.energy.to_f64()
    ));
    out.push_str(&format!(
        "  Age:         {:5.1} years\n\n",
        agent.age.to_f64()
    ));

    // ── Needs (nonlinear pressure) ──
    out.push_str("── §9.1: Need State (nonlinear pressure) ──\n\n");
    let needs = [
        ("Hunger", agent.needs.hunger),
        ("Thirst", agent.needs.thirst),
        ("Fatigue", agent.needs.fatigue),
        ("Safety", agent.needs.safety),
        ("Social", agent.needs.social),
        ("Esteem", agent.needs.esteem),
        ("Autonomy", agent.needs.autonomy),
        ("Meaning", agent.needs.meaning),
    ];
    for (name, val) in &needs {
        let bar_len = (val.to_f64() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len.min(20)) + &"░".repeat(20 - bar_len.min(20));
        out.push_str(&format!("  {:<10} {:5.2} [{}]\n", name, val.to_f64(), bar));
    }
    out.push('\n');

    // ── Personality ──
    out.push_str("── §22.1: Personality Traits ──\n\n");
    let traits = [
        ("Openness", agent.personality.openness),
        ("Conscien.", agent.personality.conscientiousness),
        ("Extraver.", agent.personality.extraversion),
        ("Agreeab.", agent.personality.agreeableness),
        ("Neurotic.", agent.personality.neuroticism),
        ("Risk Tol.", agent.personality.risk_tolerance),
        ("Conformity", agent.personality.conformity),
        ("Ambition", agent.personality.ambition),
        ("Altruism", agent.personality.altruism),
        ("Tradition", agent.personality.traditionalism),
        ("Dominance", agent.personality.dominance),
        ("Impulsiv.", agent.personality.impulsivity),
    ];
    for (name, val) in &traits {
        let bar_len = (val.to_f64() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len.min(20)) + &"░".repeat(20 - bar_len.min(20));
        out.push_str(&format!("  {:<12} {:5.2} [{}]\n", name, val.to_f64(), bar));
    }
    out.push('\n');

    // ── Dimensional Emotion (Affect) ──
    out.push_str("── §22.2: Dimensional Emotion (Affect) ──\n\n");
    out.push_str(&format!(
        "  Valence:   {:+5.2}   (positive=joy, negative=sadness/fear)\n",
        agent.affect.valence.to_f64()
    ));
    out.push_str(&format!(
        "  Arousal:    {:5.2}   (high=activated, low=calm)\n",
        agent.affect.arousal.to_f64()
    ));
    out.push_str(&format!(
        "  Control:    {:5.2}   (high=in control, low=helpless)\n\n",
        agent.affect.control.to_f64()
    ));

    // ── Discrete Emotions ──
    out.push_str("── §22.2: Discrete Emotions ──\n\n");
    let emotions = [
        ("Fear", agent.emotions.fear),
        ("Anger", agent.emotions.anger),
        ("Joy", agent.emotions.joy),
        ("Sadness", agent.emotions.sadness),
        ("Shame", agent.emotions.shame),
        ("Pride", agent.emotions.pride),
        ("Guilt", agent.emotions.guilt),
        ("Trust", agent.emotions.trust),
    ];
    for (name, val) in &emotions {
        let bar_len = (val.to_f64() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len.min(20)) + &"░".repeat(20 - bar_len.min(20));
        out.push_str(&format!("  {:<10} {:5.2} [{}]\n", name, val.to_f64(), bar));
    }
    out.push('\n');

    // ── Cognitive State (Bounded Rationality) ──
    out.push_str("── §22.1: Cognitive State (Bounded Rationality) ──\n\n");
    out.push_str(&format!(
        "  Attention Cap:  {:5.2}\n",
        agent.cognitive.attention_capacity.to_f64()
    ));
    out.push_str(&format!(
        "  Executive Cap:  {:5.2}\n",
        agent.cognitive.executive_capacity.to_f64()
    ));
    out.push_str(&format!(
        "  Stress:         {:5.2}\n",
        agent.cognitive.stress.to_f64()
    ));
    out.push_str(&format!(
        "  Planning Horiz: {} ticks\n",
        agent.cognitive.planning_horizon
    ));
    out.push_str(&format!(
        "  Heuristic Bias: {:5.2}   (high=uses shortcuts)\n\n",
        agent.cognitive.heuristic_bias.to_f64()
    ));

    // ── Moral Values (§22.1) ──
    out.push_str("── §22.1: Moral Values (Moral Foundations) ──\n\n");
    let morals = [
        ("Care", agent.moral_values.care),
        ("Fairness", agent.moral_values.fairness),
        ("Loyalty", agent.moral_values.loyalty),
        ("Authority", agent.moral_values.authority),
        ("Purity", agent.moral_values.purity),
        ("Liberty", agent.moral_values.liberty),
    ];
    for (name, val) in &morals {
        let bar_len = (val.to_f64() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len.min(20)) + &"░".repeat(20 - bar_len.min(20));
        out.push_str(&format!("  {:<12} {:5.2} [{}]\n", name, val.to_f64(), bar));
    }
    out.push('\n');

    // ── Identity State ──
    out.push_str("── §22: Identity State ──\n\n");
    for identity in &agent.identity.identities {
        #[allow(unreachable_patterns)]
        let kind_str = match identity.kind {
            IdentityKind::Farmer => "Farmer",
            IdentityKind::Parent => "Parent",
            IdentityKind::Believer => "Believer",
            _ => "Other",
        };
        let bar_len = (identity.strength.to_f64() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len.min(20)) + &"░".repeat(20 - bar_len.min(20));
        out.push_str(&format!(
            "  {:<12} {:5.2} [{}]\n",
            kind_str,
            identity.strength.to_f64(),
            bar
        ));
    }
    out.push('\n');

    // ── Derived Mental States ──
    out.push_str("── §22: Derived Mental States ──\n\n");
    out.push_str(&format!(
        "  Trauma Risk:  {:5.2}\n",
        agent.derived.trauma_risk.to_f64()
    ));
    out.push_str(&format!(
        "  Depress Risk: {:5.2}\n",
        agent.derived.depression_risk.to_f64()
    ));
    out.push_str(&format!(
        "  Resilience:   {:5.2}\n",
        agent.derived.resilience.to_f64()
    ));
    out.push_str(&format!(
        "  Ambition:     {:5.2}\n",
        agent.derived.ambition.to_f64()
    ));
    out.push_str(&format!(
        "  Resentment:   {:5.2}\n\n",
        agent.derived.resentment.to_f64()
    ));

    // ── Attachment System (§8.1.14) ──
    out.push_str("── §8.1.14: Attachment System ──\n\n");
    let style_str = attachment_style_name(agent.attachment.style);
    let caregiving_str = caregiving_style_name(agent.attachment.caregiving_style);
    out.push_str(&format!("  {:<14} {style_str}\n", "Style"));
    out.push_str(&format!("  {:<14} {caregiving_str}\n", "Caregiving"));
    let att_rows = [
        ("Security", agent.attachment.security),
        ("Anxiety", agent.attachment.anxiety),
        ("Avoidance", agent.attachment.avoidance),
        ("Protest thr.", agent.attachment.protest_threshold),
        ("Soothing", agent.attachment.soothing_receptivity),
        ("Sep. distress", agent.attachment.separation_distress),
    ];
    for (name, val) in &att_rows {
        let bar_len = (val.to_f64() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len.min(20)) + &"░".repeat(20 - bar_len.min(20));
        out.push_str(&format!("  {:<14} {:5.2} [{}]\n", name, val.to_f64(), bar));
    }
    out.push('\n');

    // ── Status ──
    out.push_str("── §19.5.G: Status ──\n\n");
    out.push_str(&format!(
        "  Wealth:       {:5.2}  (coins: {})\n",
        agent.status.wealth_status.to_f64(),
        agent.wealth.coin.to_f64()
    ));
    out.push_str(&format!(
        "  Social:       {:5.2}\n",
        agent.status.social_status.to_f64()
    ));
    out.push_str(&format!(
        "  Role:         {:5.2}\n\n",
        agent.status.role_status.to_f64()
    ));

    // ── Skills ──
    out.push_str("── §4.2: Skills ──\n\n");
    let skills = [
        ("Farming", agent.skills.farming),
        ("Trading", agent.skills.trading),
        ("Social", agent.skills.social),
    ];
    for (name, val) in &skills {
        let bar_len = (val.to_f64() * 20.0) as usize;
        let bar: String = "█".repeat(bar_len.min(20)) + &"░".repeat(20 - bar_len.min(20));
        out.push_str(&format!("  {:<12} {:5.2} [{}]\n", name, val.to_f64(), bar));
    }
    out.push('\n');

    // ── Beliefs ──
    out.push_str("── §19.5.A: Beliefs ──\n\n");
    if agent.beliefs.is_empty() {
        out.push_str("  (no beliefs)\n\n");
    } else {
        for belief in &agent.beliefs {
            out.push_str(&format!(
                "  Proposition {}: conf={:.2} charge={:.2} resist={:.2} linkage={:.2}\n",
                belief.proposition_id,
                belief.confidence.to_f64(),
                belief.emotional_charge.to_f64(),
                belief.resistance.to_f64(),
                belief.identity_linkage.to_f64()
            ));
        }
        out.push('\n');
    }

    // ── Intention ──
    out.push_str("── §24.5: Current Intention ──\n\n");
    if let Some(ref intention) = agent.intention {
        out.push_str(&format!("  Goal:      {:?}\n", intention.goal_kind));
        out.push_str(&format!("  Formed:    tick {}\n", intention.formed_tick));
        out.push_str(&format!(
            "  Commit:    {:5.2}\n",
            intention.commitment.to_f64()
        ));
        out.push_str(&format!(
            "  Duration:  {} ticks\n",
            intention.duration_ticks
        ));
        out.push_str(&format!(
            "  Failures:  {}\n",
            intention.consecutive_failures
        ));
        out.push_str(&format!(
            "  Status:    {}\n\n",
            if intention.completed {
                "COMPLETED"
            } else {
                "in progress"
            }
        ));
    } else {
        out.push_str("  (no active intention)\n\n");
    }

    // ── Active Goals ──
    out.push_str("── §3: Active Goals ──\n\n");
    if agent.goals.is_empty() {
        out.push_str("  (no active goals)\n\n");
    } else {
        for goal in agent.goals.iter().take(5) {
            let source_str = match goal.source {
                GoalSource::Need => "need",
                GoalSource::Identity => "identity",
                GoalSource::Emotion => "emotion",
                GoalSource::Command => "command",
            };
            out.push_str(&format!(
                "  {:?}  priority={:.2}  source={}\n",
                goal.kind,
                goal.priority.to_f64(),
                source_str
            ));
        }
        if agent.goals.len() > 5 {
            out.push_str(&format!("  ... and {} more\n", agent.goals.len() - 5));
        }
        out.push('\n');
    }

    // ── Goal History (Iteration 199) ──
    // §3.2: the rejected/completed lists feed goal-generation learning
    // (recent rejections suppress re-forming a kind, completions reinforce);
    // surface the last few of each so the learning is observable in play.
    out.push_str("── §3.2: Goal History (learning) ──\n\n");
    if agent.rejected_goals.is_empty() && agent.completed_goals.is_empty() {
        out.push_str("  (no goal history yet)\n\n");
    } else {
        if !agent.rejected_goals.is_empty() {
            out.push_str(&format!("  rejected ({}): ", agent.rejected_goals.len()));
            let kinds: Vec<String> = agent
                .rejected_goals
                .iter()
                .rev()
                .take(6)
                .map(|g| format!("{:?}", g.kind))
                .collect();
            out.push_str(&kinds.join(", "));
            out.push_str("  ← suppresses re-forming\n");
        }
        if !agent.completed_goals.is_empty() {
            out.push_str(&format!("  completed ({}): ", agent.completed_goals.len()));
            let kinds: Vec<String> = agent
                .completed_goals
                .iter()
                .rev()
                .take(6)
                .map(|g| format!("{:?}", g.kind))
                .collect();
            out.push_str(&kinds.join(", "));
            out.push_str("  → reinforces\n");
        }
        out.push('\n');
    }

    // ── Conflict State ──
    out.push_str("── §19.5.H: Conflict State ──\n\n");
    out.push_str(&format!(
        "  Trauma:           {:5.2}\n",
        agent.conflict.trauma.to_f64()
    ));
    out.push_str(&format!(
        "  Combat fatigue:   {:5.2}\n",
        agent.conflict.combat_fatigue.to_f64()
    ));
    out.push_str(&format!(
        "  Conflicts:        {}\n",
        agent.conflict.conflict_count
    ));
    out.push_str(&format!(
        "  Injuries:         {}\n",
        agent.conflict.injuries_received
    ));
    out.push_str(&format!("  Active feuds:     {}\n\n", agent.feuds.len()));

    // ── Cultural Knowledge ──
    out.push_str("── §19.5.I: Cultural Knowledge ──\n\n");
    out.push_str(&format!(
        "  Knowledge count:  {}\n",
        agent.cultural.knowledge.len()
    ));
    out.push_str(&format!(
        "  Openness:         {:5.2}\n\n",
        agent.cultural.openness.to_f64()
    ));

    // ── Memory ──
    out.push_str("── §22.5: Memory ──\n\n");
    out.push_str(&format!(
        "  Total memories:   {}\n",
        agent.memory.episodes.len()
    ));
    out.push_str(&format!(
        "  Capacity:         {}\n\n",
        agent.memory.capacity
    ));

    out
}

// ── §5 (Iterations 152/153): New-system dashboards ──────────────────

/// §5 (Iteration 152): Render the theology dashboard — the seeded
/// religion, believer tallies, mean conviction, and per-believer state.
pub fn render_theology_dashboard(registry: &TheologyRegistry) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Theology Dashboard                      ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");

    let Some(religion) = &registry.religion else {
        out.push_str("  (no religion seeded — theology dormant)\n");
        return out;
    };
    out.push_str(&format!(
        "  Religion: {} ({})\n",
        religion.deity.name, religion.doctrine.name
    ));
    out.push_str(&format!(
        "  Deity temperament: {:?}\n",
        religion.deity.temperament
    ));
    out.push_str(&format!(
        "  Believers: {}/{}   Converts: {}   Festivals: {}\n",
        registry.believer_count(),
        registry.beliefs.len(),
        registry.converts,
        registry.festivals_held,
    ));
    let beliefs: Vec<&TheologicalBelief> = registry.beliefs.iter().flatten().collect();
    if beliefs.is_empty() {
        out.push_str("  (no believers yet)\n");
        return out;
    }
    let mean = beliefs.iter().map(|b| b.conviction.to_f64()).sum::<f64>() / beliefs.len() as f64;
    out.push_str(&format!("  Mean conviction: {mean:.3}\n"));
    out.push_str("  ── Believers ────────────────────────────────\n");
    for (i, belief) in registry.beliefs.iter().enumerate() {
        if let Some(b) = belief {
            out.push_str(&format!(
                "  Agent {i}: conviction {:.3}, since {}\n",
                b.conviction.to_f64(),
                b.since_tick,
            ));
        }
    }
    out
}

// ── §5 (Iteration 155): Interactive session state ───────────────────

/// §5 (Iteration 153): Render the military dashboard — collective
/// readiness, militia tallies, and the conscription roster.
pub fn render_military_dashboard(registry: &MilitaryRegistry) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Military Dashboard                       ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");

    if registry.is_dormant() {
        out.push_str("  (no barracks — military dormant)\n");
        return out;
    }
    out.push_str(&format!(
        "  Readiness: {:.3}   Militia: {}   Conscripts: {}   Musters: {}   Drills: {}\n",
        registry.readiness.to_f64(),
        registry.militia_size(),
        registry.conscripts,
        registry.musters,
        registry.drills,
    ));
    out.push_str("  ── Roster ──────────────────────────────────\n");
    for (i, member) in registry.roster.iter().enumerate() {
        if let Some(m) = member {
            out.push_str(&format!(
                "  Agent {i}: since {}, dominance {:.3}\n",
                m.enlisted_since,
                m.dominance_at_enlistment.to_f64(),
            ));
        }
    }
    out
}

// ── §13 (Iteration 178): Noosphere/culture inspector ───────────────

/// Render the noosphere/culture inspector — the collective-mind layer that
/// the other dashboards do not surface: the legitimacy field, the meme
/// pool, moral panics, propaganda campaigns, echo-chamber polarization,
/// the rumor pool, and the symbolic noospheric field.
///
/// Pure and read-only: renders whatever state the registries hold. The
/// caller decides what to pass; empty registries render their dormant
/// state explicitly so the output is self-explanatory at any horizon.
pub fn render_noosphere_inspector(
    memes: &MemeRegistry,
    panics: &MoralPanicRegistry,
    propaganda: &PropagandaRegistry,
    legitimacy: &[&LegitimacyField],
    echo: &EchoChamberState,
    rumors: &RumorRegistry,
    noosphere: &NoosphericField,
) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Noosphere / Culture Inspector          ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");

    // ── Legitimacy field (per-agent perception of institutional
    // legitimacy — §11.1) ─────────────────────────────────────────
    out.push_str("  ── Legitimacy (per-agent) ────────────────────\n");
    let mean_legitimacy = if legitimacy.is_empty() {
        0.0
    } else {
        legitimacy.iter().map(|l| l.overall.to_f64()).sum::<f64>() / legitimacy.len() as f64
    };
    out.push_str(&format!(
        "  Agents: {}   Mean overall: {:.3}\n",
        legitimacy.len(),
        mean_legitimacy,
    ));
    for (i, field) in legitimacy.iter().take(10).enumerate() {
        let source_word = if field.sources.len() == 1 {
            "source"
        } else {
            "sources"
        };
        out.push_str(&format!(
            "    Agent {i}: overall {:.3} ({} {source_word}, decay {:.4})\n",
            field.overall.to_f64(),
            field.sources.len(),
            field.base_decay_rate.to_f64(),
        ));
    }
    if legitimacy.len() > 10 {
        out.push_str(&format!(
            "    ... and {} more agents\n",
            legitimacy.len() - 10
        ));
    }

    // ── Meme pool ─────────────────────────────────────────────────
    out.push_str("  ── Meme Pool ───────────────────────────────\n");
    let active_meme_count = memes.memes.iter().filter(|m| m.active).count();
    out.push_str(&format!(
        "  Memes: {} total, {} active\n",
        memes.memes.len(),
        active_meme_count,
    ));
    if memes.memes.is_empty() {
        out.push_str("  (no memes seeded)\n");
    }
    for meme in &memes.memes {
        out.push_str(&format!(
            "    #{} [{:?}] hosts={} virality={:.2} novelty={:.2} sacred={:.2}{}\n",
            meme.id,
            meme.content_type,
            meme.host_count,
            meme.virality.to_f64(),
            meme.novelty.to_f64(),
            meme.sacredness.to_f64(),
            if meme.active { "" } else { " [inactive]" },
        ));
        out.push_str(&format!("      \"{}\"\n", meme.description));
    }

    // ── Moral panics ──────────────────────────────────────────────
    out.push_str("  ── Moral Panics ─────────────────────────────\n");
    if panics.panics.is_empty() {
        out.push_str("  (no moral panics recorded)\n");
    }
    for panic in &panics.panics {
        out.push_str(&format!(
            "    {:?} @tick {} intensity {:.3} fear {:.3} participants {}{}\n",
            panic.trigger,
            panic.start_tick,
            panic.intensity.to_f64(),
            panic.fear_level.to_f64(),
            panic.participants,
            if panic.active {
                " [ACTIVE]"
            } else {
                " [resolved]"
            },
        ));
    }

    // ── Propaganda campaigns ──────────────────────────────────────
    out.push_str("  ── Propaganda Campaigns ──────────────────────\n");
    let active_campaign_count = propaganda.campaigns.iter().filter(|c| c.active).count();
    out.push_str(&format!(
        "  Campaigns: {} total, {} active\n",
        propaganda.campaigns.len(),
        active_campaign_count,
    ));
    if propaganda.campaigns.is_empty() {
        out.push_str("  (no campaigns)\n");
    }
    for campaign in &propaganda.campaigns {
        let channels: Vec<String> = campaign.channels.iter().map(|c| format!("{c:?}")).collect();
        out.push_str(&format!(
            "    #{} sponsor={} intensity={:.2} credibility={:.2} remaining={}/{}{}\n",
            campaign.id,
            campaign.sponsor,
            campaign.intensity.to_f64(),
            campaign.credibility.to_f64(),
            campaign.remaining,
            campaign.duration,
            if campaign.active { "" } else { " [ended]" },
        ));
        out.push_str(&format!(
            "      \"{}\" via {}\n",
            campaign.narrative,
            channels.join("+")
        ));
    }

    // ── Echo chambers ─────────────────────────────────────────────
    out.push_str("  ── Echo Chambers ─────────────────────────────\n");
    out.push_str(&format!(
        "  Polarization: {:.3}   Strength: {:.3}   Cross-cutting ties: {} ({:.3})\n",
        echo.polarization_index.to_f64(),
        echo.echo_chamber_strength.to_f64(),
        echo.total_cross_cutting_ties,
        echo.cross_cutting_ties.to_f64(),
    ));
    if echo.narrative_dominance.is_empty() {
        out.push_str("  (no narrative dominance data)\n");
    }
    for (meme_id, dominance) in &echo.narrative_dominance {
        out.push_str(&format!(
            "    meme #{meme_id}: dominance {:.3}\n",
            dominance.to_f64()
        ));
    }

    // ── Rumors ───────────────────────────────────────────────────
    out.push_str("  ── Rumors ────────────────────────────────────\n");
    if rumors.rumors.is_empty() {
        out.push_str("  (no rumors)\n");
    }
    for rumor in &rumors.rumors {
        let target_desc = match (rumor.target, rumor.institution_target) {
            (Some(agent), _) => format!("agent {agent}"),
            (None, Some(inst)) => format!("inst {inst}"),
            (None, None) => "general".to_string(),
        };
        out.push_str(&format!(
            "    #{} [{target_desc}] prevalence {:.2} evidence {:.2} panic-potential {:.2}\n",
            rumor.id,
            rumor.prevalence.to_f64(),
            rumor.evidence_quality.to_f64(),
            rumor.moral_panic_potential.to_f64(),
        ));
        out.push_str(&format!("      \"{}\"\n", rumor.description));
    }

    // ── Noospheric field ──────────────────────────────────────────
    out.push_str("  ── Noospheric Field ──────────────────────────\n");
    out.push_str(&format!(
        "  Symbolic nodes: {}   Edges: {}\n",
        noosphere.nodes.len(),
        noosphere.edges.len(),
    ));
    if noosphere.nodes.is_empty() {
        out.push_str("  (field empty)\n");
    }
    for node in noosphere.nodes.iter().take(6) {
        out.push_str(&format!(
            "    node #{} activation {:.3}\n",
            node.id,
            node.activation.to_f64(),
        ));
    }
    if noosphere.nodes.len() > 6 {
        out.push_str(&format!(
            "    ... and {} more nodes\n",
            noosphere.nodes.len() - 6
        ));
    }
    out
}
