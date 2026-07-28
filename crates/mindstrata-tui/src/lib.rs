//! Mindstrata TUI — debug instrument for observing simulations.
//!
//! Provides ASCII world map, agent list, event log, and system dashboard.

use mindstrata_core::event::SimEvent;
use mindstrata_sim::world::{World, Terrain, SiteKind};
use mindstrata_sim::sim::AgentSummary;

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
        "{:<3} {:<8} {:>5} {:>5} {:>5} {:>5} {:>6} {:>6} {:>6} {:<12}\n",
        "ID", "Name", "H", "T", "F", "HP", "Val", "Joy", "Fear", "Action"
    ));
    out.push_str(&format!("{}\n", "─".repeat(72)));

    for s in agents {
        out.push_str(&format!(
            "{:<3} {:<8} {:>5.2} {:>5.2} {:>5.2} {:>5.2} {:>+5.2} {:>5.2} {:>5.2} {:<12}\n",
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
        ));
    }
    out
}

/// Render the event log (last n events).
pub fn render_event_log(events: &[SimEvent], n: usize) -> String {
    let mut out = String::new();
    let start = events.len().saturating_sub(n);
    for (i, ev) in events[start..].iter().enumerate() {
        let tick = start + i;
        match ev {
            SimEvent::AgentAte { agent, .. } => {
                out.push_str(&format!("[{tick}] Agent {} ate\n", agent.as_u64()));
            }
            SimEvent::AgentDrank { agent, .. } => {
                out.push_str(&format!("[{tick}] Agent {} drank\n", agent.as_u64()));
            }
            SimEvent::AgentRested { agent, .. } => {
                out.push_str(&format!("[{tick}] Agent {} rested\n", agent.as_u64()));
            }
            SimEvent::InteractionOccurred { from, to, kind, .. } => {
                out.push_str(&format!(
                    "[{tick}] Agent {} {:?} Agent {}\n",
                    from.as_u64(), kind, to.as_u64()
                ));
            }
            SimEvent::RelationshipChanged { from, to, trust_delta, .. } => {
                out.push_str(&format!(
                    "[{tick}] {}→{} trust {:+.3}\n",
                    from.as_u64(), to.as_u64(), trust_delta.to_f64()
                ));
            }
            _ => {
                out.push_str(&format!("[{tick}] Event: {:?}\n", ev));
            }
        }
    }
    out
}

/// Render system dashboard.
pub fn render_dashboard(agents: &[AgentSummary], events_count: usize, tick: u64) -> String {
    let n = agents.len() as f64;
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
         Agents:      {}\n\
         Events:      {events_count}\n\
         ── Population Averages ──\n\
         Hunger:      {avg_hunger:.3}\n\
         Thirst:      {avg_thirst:.3}\n\
         Fatigue:     {avg_fatigue:.3}\n\
         Health:      {avg_health:.3}\n\
         Valence:     {avg_valence:+.3}\n\
         Joy:         {avg_joy:.3}\n\
         Fear:        {avg_fear:.3}\n",
        agents.len()
    )
}
