//! Mindstrata TUI — debug instrument for observing simulations.
//!
//! Provides ASCII world map, agent list, event log, and system dashboard.

use mindstrata_core::event::SimEvent;
use mindstrata_core::fixed::Fixed;
use mindstrata_core::id::AgentId;
use mindstrata_sim::world::{World, Terrain, SiteKind};
use mindstrata_sim::sim::AgentSummary;
use mindstrata_sim::person::{Belief, Relationship};
use mindstrata_sim::market::MarketState;
use mindstrata_sim::institutions::Institution;

/// Render the ASCII world map.
pub fn render_world_map(world: &World) -> String {
    let mut out = String::new();
    out.push_str("  ");
    for x in 0..world.width {
        out.push_str(&format!("{:1}", x % 10));
    }
    out.push('\n');

    for y in 0..world.height {
        out.push_str(&format!("{:1} ", y % 10));
        for x in 0..world.width {
            if let Some(tile) = world.tile(x as i32, y as i32) {
                let ch = if tile.site.is_some() {
                    // Find site kind
                    world.sites.iter()
                        .find(|s| tile.site == Some(s.id))
                        .map(|s| match s.kind {
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
                        .unwrap_or('?')
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
        summary.name, summary.index,
        summary.health.to_f64(), summary.energy.to_f64(),
        summary.hunger.to_f64(), summary.thirst.to_f64(),
        summary.fatigue.to_f64(),
        summary.valence.to_f64(), summary.joy.to_f64(),
        summary.fear.to_f64(), summary.anger.to_f64(),
        summary.current_action,
        if summary.has_intention { "active" } else { "none" },
        summary.attention_budget.to_f64(),
    ));

    // Show relationships for this agent
    let rels: Vec<&Relationship> = relationships.iter()
        .filter(|r| r.from == agent_id)
        .collect();
    if !rels.is_empty() {
        out.push_str("\n  ── Relationships ──\n\n");
        for r in rels.iter().take(5) {
            out.push_str(&format!(
                "  → Agent {} trust={:.2} affection={:.2}\n",
                r.to.as_u64(), r.trust.to_f64(), r.affection.to_f64()
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
) -> String {
    let from_name = agents.get(from_id.as_u64() as usize)
        .map(|s| s.name.as_str()).unwrap_or("?");
    let to_name = agents.get(to_id.as_u64() as usize)
        .map(|s| s.name.as_str()).unwrap_or("?");

    let forward = relationships.iter().find(|r| r.from == from_id && r.to == to_id);
    let backward = relationships.iter().find(|r| r.from == to_id && r.to == from_id);

    let mut out = String::new();
    out.push_str(&format!(
        "╔══════════════════════════════════════════╗\n\
         ║  Relationship View                      ║\n\
         ╚══════════════════════════════════════════╝\n\
         {} → {}\n",
        from_name, to_name
    ));

    if let Some(r) = forward {
        out.push_str(&format!(
            "  trust:      {:.2}\n\
             affection:  {:.2}\n\
             respect:    {:.2}\n\
             obligation: {:.2}\n",
            r.trust.to_f64(), r.affection.to_f64(),
            r.respect.to_f64(), r.obligation.to_f64(),
        ));
    } else {
        out.push_str("  (no relationship)\n");
    }

    out.push_str(&format!("\n  {} → {}\n", to_name, from_name));
    if let Some(r) = backward {
        out.push_str(&format!(
            "  trust:      {:.2}\n\
             affection:  {:.2}\n\
             respect:    {:.2}\n\
             obligation: {:.2}\n",
            r.trust.to_f64(), r.affection.to_f64(),
            r.respect.to_f64(), r.obligation.to_f64(),
        ));
    } else {
        out.push_str("  (no relationship)\n");
    }

    out
}

/// Dashboard configuration for the render_dashboard function.
pub struct DashboardConfig {
    pub season: String,
    pub year: u64,
    pub grain: f64,
    pub water: f64,
    pub institution_count: usize,
    pub faction_count: usize,
}

/// §17.1: Enhanced system dashboard with season, institutions, factions.
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
        config.season, config.year,
        agents.len(),
        config.institution_count, config.faction_count,
        config.grain, config.water,
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
         Trade Volume:      {:.1}\n",
        market.inequality.to_f64(),
        market.avg_wealth.to_f64(),
        market.median_wealth.to_f64(),
        market.volume_this_tick.to_f64(),
    ));

    out
}

// ── §17.1: Belief Inspector ────────────────────────────────────────────

/// §17.1: Render an agent's beliefs.
pub fn render_belief_inspector(
    agent_name: &str,
    agent_id: usize,
    beliefs: &[Belief],
) -> String {
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════╗\n");
    out.push_str("║  Belief Inspector                        ║\n");
    out.push_str("╚══════════════════════════════════════════╝\n\n");
    out.push_str(&format!("Agent: {} (ID {})\n\n", agent_name, agent_id));

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
        out.push_str(&format!(
            "  {:<5} {:<8} {:>8.3} {:>8.3} {:>8.3}\n",
            b.proposition_id,
            "prop_{}",
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
            inst.name, kind_str,
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
                out.push_str(&format!("  [{tick:>5}] 💧 Agent {} drank\n", agent.as_u64()));
            }
            SimEvent::AgentRested { agent, .. } => {
                out.push_str(&format!("  [{tick:>5}] 😴 Agent {} rested\n", agent.as_u64()));
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
                    icon, from.as_u64(), to.as_u64()
                ));
            }
            SimEvent::RelationshipChanged { from, to, trust_delta, affection_delta, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 📊 {}→{} trust {:+.3} affection {:+.3}\n",
                    from.as_u64(), to.as_u64(),
                    trust_delta.to_f64(), affection_delta.to_f64()
                ));
            }
            SimEvent::RumorSpread { source, target, content_hash, distortion, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 🗣️ Rumor {}→{} prop={} distortion={:.3}\n",
                    source.as_u64(), target.as_u64(),
                    content_hash, distortion.to_f64()
                ));
            }
            _ => {
                out.push_str(&format!("  [{tick:>5}] ❓ {:?}\n", ev));
            }
        }
    }
    out
}
