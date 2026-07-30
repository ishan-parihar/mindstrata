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
use mindstrata_sim::institutions::{Institution, InstitutionalRecord};
use mindstrata_sim::provenance::CausalProvenance;

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
            let agent_at_pos: Option<&AgentMarker> = agent_markers.iter()
                .find(|m| m.x == x as i32 && m.y == y as i32);

            if let Some(marker) = agent_at_pos {
                out.push(marker.name);
            } else if let Some(tile) = world.tile(x as i32, y as i32) {
                let ch = if tile.site.is_some() {
                    // Find site kind
                    world.sites.iter()
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
        .map_or("?", |s| s.name.as_str());
    let to_name = agents.get(to_id.as_u64() as usize)
        .map_or("?", |s| s.name.as_str());

    let forward = relationships.iter().find(|r| r.from == from_id && r.to == to_id);
    let backward = relationships.iter().find(|r| r.from == to_id && r.to == from_id);

    let mut out = String::new();
    out.push_str(&format!(
        "╔══════════════════════════════════════════╗\n\
         ║  Relationship View                      ║\n\
         ╚══════════════════════════════════════════╝\n\
         {from_name} → {to_name}\n"
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

    out.push_str(&format!("\n  {to_name} → {from_name}\n"));
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
            r.tick, status, r.action, r.affected.len()
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
        let interrupted = if trace.interrupted_by_critical_needs { " [interrupted]" } else { "" };
        out.push_str(&format!(
            "  [{:>5}] {}{}{}\n",
            trace.tick, trace.action_name, routine, interrupted
        ));
        for factor in &trace.factors {
            out.push_str(&format!(
                "        {}: {:.3} — {}\n",
                factor.kind, factor.magnitude.to_f64(), factor.description
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
            SimEvent::KnowledgeTransferred { source, target, knowledge_id, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 📚 Knowledge {}→{} id={}\n",
                    source.as_u64(), target.as_u64(), knowledge_id
                ));
            }
            SimEvent::ConflictOccurred { aggressor, target, kind, injury, fear_induced, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] ⚔️ Conflict {}→{} {} injury={:.3} fear={:.3}\n",
                    aggressor.as_u64(), target.as_u64(), kind,
                    injury.to_f64(), fear_induced.to_f64()
                ));
            }
            SimEvent::FeudFormed { party_a, party_b, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 🔥 Feud formed {}↔{}\n",
                    party_a.as_u64(), party_b.as_u64()
                ));
            }
            SimEvent::MarriageFormed { spouse_a, spouse_b, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 💍 Marriage {}↔{}\n",
                    spouse_a.as_u64(), spouse_b.as_u64()
                ));
            }
            SimEvent::ChildBorn { child, parent_a, parent_b, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 👶 Child {} born ({} + {})\n",
                    child.as_u64(), parent_a.as_u64(), parent_b.as_u64()
                ));
            }
            SimEvent::NormViolated { agent, norm_id, witnesses, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] ⚖️ Agent {} violated norm {} ({} witnesses)\n",
                    agent.as_u64(), norm_id, witnesses.len()
                ));
            }
            SimEvent::AgentSpawned { agent, .. } => {
                out.push_str(&format!("  [{tick:>5}] 🌱 Agent {} spawned\n", agent.as_u64()));
            }
            SimEvent::AgentDied { agent, cause, .. } => {
                out.push_str(&format!("  [{tick:>5}] 💀 Agent {} died ({:?})\n", agent.as_u64(), cause));
            }
            SimEvent::TradeOccurred { buyer, seller, good, quantity, price, .. } => {
                out.push_str(&format!(
                    "  [{tick:>5}] 💰 Trade {}→{} good={} qty={:.2} price={:.1}\n",
                    buyer.as_u64(), seller.as_u64(), good,
                    quantity.to_f64(), price.to_f64()
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
    use mindstrata_sim::person::{IdentityKind, GoalSource};
    let mut out = String::new();
    out.push_str("╔══════════════════════════════════════════════════════════╗\n");
    out.push_str("║  Psychology Inspector — Full Cognitive Pipeline          ║\n");
    out.push_str("╚══════════════════════════════════════════════════════════╝\n\n");
    out.push_str(&format!("Agent: {agent_name} (ID {agent_index})\n\n"));

    // ── Body ──
    out.push_str("── §22.1: Body State ──\n\n");
    out.push_str(&format!("  Health:      {:5.2}   Energy:    {:5.2}\n", agent.body.health.to_f64(), agent.body.energy.to_f64()));
    out.push_str(&format!("  Age:         {:5.1} years\n\n", agent.age.to_f64()));

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
    out.push_str(&format!("  Valence:   {:+5.2}   (positive=joy, negative=sadness/fear)\n", agent.affect.valence.to_f64()));
    out.push_str(&format!("  Arousal:    {:5.2}   (high=activated, low=calm)\n", agent.affect.arousal.to_f64()));
    out.push_str(&format!("  Control:    {:5.2}   (high=in control, low=helpless)\n\n", agent.affect.control.to_f64()));

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
    out.push_str(&format!("  Attention Cap:  {:5.2}\n", agent.cognitive.attention_capacity.to_f64()));
    out.push_str(&format!("  Executive Cap:  {:5.2}\n", agent.cognitive.executive_capacity.to_f64()));
    out.push_str(&format!("  Stress:         {:5.2}\n", agent.cognitive.stress.to_f64()));
    out.push_str(&format!("  Planning Horiz: {} ticks\n", agent.cognitive.planning_horizon));
    out.push_str(&format!("  Heuristic Bias: {:5.2}   (high=uses shortcuts)\n\n", agent.cognitive.heuristic_bias.to_f64()));

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
        out.push_str(&format!("  {:<12} {:5.2} [{}]\n", kind_str, identity.strength.to_f64(), bar));
    }
    out.push('\n');

    // ── Derived Mental States ──
    out.push_str("── §22: Derived Mental States ──\n\n");
    out.push_str(&format!("  Trauma Risk:  {:5.2}\n", agent.derived.trauma_risk.to_f64()));
    out.push_str(&format!("  Depress Risk: {:5.2}\n", agent.derived.depression_risk.to_f64()));
    out.push_str(&format!("  Resilience:   {:5.2}\n", agent.derived.resilience.to_f64()));
    out.push_str(&format!("  Ambition:     {:5.2}\n", agent.derived.ambition.to_f64()));
    out.push_str(&format!("  Resentment:   {:5.2}\n\n", agent.derived.resentment.to_f64()));

    // ── Status ──
    out.push_str("── §19.5.G: Status ──\n\n");
    out.push_str(&format!("  Wealth:       {:5.2}  (coins: {})\n", agent.status.wealth_status.to_f64(), agent.wealth.coin.to_f64()));
    out.push_str(&format!("  Social:       {:5.2}\n", agent.status.social_status.to_f64()));
    out.push_str(&format!("  Role:         {:5.2}\n\n", agent.status.role_status.to_f64()));

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
            out.push_str(&format!("  Proposition {}: conf={:.2} charge={:.2} resist={:.2} linkage={:.2}\n",
                belief.proposition_id, belief.confidence.to_f64(),
                belief.emotional_charge.to_f64(), belief.resistance.to_f64(),
                belief.identity_linkage.to_f64()));
        }
        out.push('\n');
    }

    // ── Intention ──
    out.push_str("── §24.5: Current Intention ──\n\n");
    if let Some(ref intention) = agent.intention {
        out.push_str(&format!("  Goal:      {:?}\n", intention.goal_kind));
        out.push_str(&format!("  Formed:    tick {}\n", intention.formed_tick));
        out.push_str(&format!("  Commit:    {:5.2}\n", intention.commitment.to_f64()));
        out.push_str(&format!("  Duration:  {} ticks\n", intention.duration_ticks));
        out.push_str(&format!("  Failures:  {}\n", intention.consecutive_failures));
        out.push_str(&format!("  Status:    {}\n\n", if intention.completed { "COMPLETED" } else { "in progress" }));
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
            };
            out.push_str(&format!("  {:?}  priority={:.2}  source={}\n", goal.kind, goal.priority.to_f64(), source_str));
        }
        if agent.goals.len() > 5 {
            out.push_str(&format!("  ... and {} more\n", agent.goals.len() - 5));
        }
        out.push('\n');
    }

    // ── Conflict State ──
    out.push_str("── §19.5.H: Conflict State ──\n\n");
    out.push_str(&format!("  Trauma:           {:5.2}\n", agent.conflict.trauma.to_f64()));
    out.push_str(&format!("  Combat fatigue:   {:5.2}\n", agent.conflict.combat_fatigue.to_f64()));
    out.push_str(&format!("  Conflicts:        {}\n", agent.conflict.conflict_count));
    out.push_str(&format!("  Injuries:         {}\n", agent.conflict.injuries_received));
    out.push_str(&format!("  Active feuds:     {}\n\n", agent.feuds.len()));

    // ── Cultural Knowledge ──
    out.push_str("── §19.5.I: Cultural Knowledge ──\n\n");
    out.push_str(&format!("  Knowledge count:  {}\n", agent.cultural.knowledge.len()));
    out.push_str(&format!("  Openness:         {:5.2}\n\n", agent.cultural.openness.to_f64()));

    // ── Memory ──
    out.push_str("── §22.5: Memory ──\n\n");
    out.push_str(&format!("  Total memories:   {}\n", agent.memory.episodes.len()));
    out.push_str(&format!("  Capacity:         {}\n\n", agent.memory.capacity));

    out
}
