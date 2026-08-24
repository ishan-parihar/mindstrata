//! Interactive session state: view tabs, selection, key bindings.

//! Mindstrata TUI — debug instrument for observing simulations.
//!
//! Provides ASCII world map, agent list, event log, and system dashboard,
//! plus the interactive session state (view tabs, selection, command keys)
//! consumed by the `mindstrata-tui` binary's event loop.

use crossterm::event::KeyCode;
use mindstrata_sim::person::GoalKind;

/// §5 (Iteration 155): The interactive-TUI view tabs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    /// The system dashboard (resources, averages, institutions).
    Dashboard,
    /// The full agent list table.
    Agents,
    /// The detailed single-agent inspector.
    Inspector,
    /// The recent-event log.
    Events,
    /// The ASCII world map with agent markers.
    Map,
    /// Longitudinal metric trends (Iteration 251).
    Trends,
    /// The village chronicle annals (Iteration 261).
    Chronicle,
    /// The selected agent's full dossier (Iteration 264).
    Dossier,
}

impl View {
    /// Human-readable tab name for the header bar.
    pub fn label(self) -> &'static str {
        match self {
            View::Dashboard => "Dashboard",
            View::Agents => "Agents",
            View::Inspector => "Inspector",
            View::Events => "Events",
            View::Map => "Map",
            View::Trends => "Trends",
            View::Chronicle => "Chronicle",
            View::Dossier => "Dossier",
        }
    }
}

/// §5 (Iteration 155): The interactive-TUI control state — agent
/// selection, view tab, run state, and the last issued command.
#[derive(Debug, Clone)]
pub struct UiState {
    /// Index of the currently selected agent.
    pub selected_agent: usize,
    /// Active view tab.
    pub view: View,
    /// True while auto-advancing (space toggles).
    pub auto_play: bool,
    /// Human-readable description of the last issued command.
    pub last_command: Option<String>,
    /// Ticks stepped manually via the 'n' key.
    pub manual_steps: u64,
    /// Iteration 264: active name-search buffer (`/`). `None` = idle; the
    /// binary's event loop routes printable input here while set.
    pub name_query: Option<String>,
}

impl UiState {
    /// Create a fresh session state for a world with `agent_count` agents.
    pub fn new(agent_count: usize) -> Self {
        let mut state = Self {
            selected_agent: 0,
            view: View::Dashboard,
            auto_play: false,
            last_command: None,
            manual_steps: 0,
            name_query: None,
        };
        state.clamp_selection(agent_count);
        state
    }

    /// Move the selection down, wrapping to the first agent.
    pub fn select_next(&mut self, agent_count: usize) {
        if agent_count == 0 {
            self.selected_agent = 0;
            return;
        }
        self.selected_agent = (self.selected_agent + 1) % agent_count;
    }

    /// Move the selection up, wrapping to the last agent.
    pub fn select_prev(&mut self, agent_count: usize) {
        if agent_count == 0 {
            self.selected_agent = 0;
            return;
        }
        self.selected_agent = (self.selected_agent + agent_count - 1) % agent_count;
    }

    /// Cycle to the next view tab (wraps around).
    pub fn cycle_view(&mut self) {
        self.view = match self.view {
            View::Dashboard => View::Agents,
            View::Agents => View::Inspector,
            View::Inspector => View::Events,
            View::Events => View::Map,
            View::Map => View::Trends,
            View::Trends => View::Chronicle,
            View::Chronicle => View::Dossier,
            View::Dossier => View::Dashboard,
        };
    }

    /// Clamp the selection into range (called after the agent count shrinks).
    pub fn clamp_selection(&mut self, agent_count: usize) {
        if agent_count == 0 {
            self.selected_agent = 0;
        } else {
            self.selected_agent = self.selected_agent.min(agent_count - 1);
        }
    }
}

/// §5 (Iteration 155): Map a command key to the goal it issues.
/// The six bound keys map to `GoalKind`s that `select_action` can steer via
/// the goal-alignment bonus. `SeekSafety` is deliberately NOT bound: it is
/// retained by goal generation but has no action-alignment arm in
/// `select_action` today, so a bound key would silently do nothing.
pub fn key_to_command(key: KeyCode) -> Option<GoalKind> {
    match key {
        KeyCode::Char('w') => Some(GoalKind::Work),
        KeyCode::Char('e') => Some(GoalKind::Eat),
        KeyCode::Char('d') => Some(GoalKind::Drink),
        KeyCode::Char('r') => Some(GoalKind::Rest),
        KeyCode::Char('s') => Some(GoalKind::Socialize),
        KeyCode::Char('p') => Some(GoalKind::Worship),
        _ => None,
    }
}

/// Iteration 264: why a `/` name-search failed to resolve. Surfaced in the
/// footer so the operator knows whether to extend the prefix or pick another
/// name (CLI parity: exact match first, then UNIQUE prefix — ambiguous
/// prefixes deliberately do not guess).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SearchFailure {
    /// No agent name starts with the query.
    NoMatch,
    /// Several agents share the prefix — narrow it.
    Ambiguous,
}

impl UiState {
    /// Enter name-search mode with an empty buffer (`/`).
    pub fn begin_search(&mut self) {
        self.name_query = Some(String::new());
    }

    /// Append one typed character to the active search buffer.
    pub fn search_push(&mut self, c: char) {
        if let Some(q) = &mut self.name_query {
            q.push(c);
        }
    }

    /// Erase the last typed character (Backspace); empty buffer stays valid
    /// so the operator can see they are still in search mode.
    pub fn search_pop(&mut self) {
        if let Some(q) = &mut self.name_query {
            q.pop();
        }
    }

    /// Leave search mode without jumping (Esc).
    pub fn cancel_search(&mut self) {
        self.name_query = None;
    }

    /// Resolve the active query against agent names — numeric buffers jump
    /// by index, otherwise exact match first then unique prefix (the same
    /// contract as the CLI `--dossier NAME` flag). On success the selection
    /// moves to the match and search mode ends; on failure the query stays
    /// on screen so it can be corrected.
    pub fn resolve_search(&mut self, names: &[String]) -> Result<usize, SearchFailure> {
        let Some(query) = &self.name_query else {
            return Err(SearchFailure::NoMatch);
        };
        let resolved = if let Ok(idx) = query.parse::<usize>() {
            if idx < names.len() {
                Some(idx)
            } else {
                None
            }
        } else {
            let exact = names.iter().position(|n| n == query);
            exact.or_else(|| {
                let hits: Vec<usize> = names
                    .iter()
                    .enumerate()
                    .filter(|(_, n)| n.starts_with(query.as_str()))
                    .map(|(i, _)| i)
                    .collect();
                if hits.len() == 1 {
                    Some(hits[0])
                } else {
                    None
                }
            })
        };
        match resolved {
            Some(idx) => {
                self.selected_agent = idx;
                self.name_query = None;
                Ok(idx)
            }
            None if names
                .iter()
                .filter(|n| n.starts_with(query.as_str()))
                .count()
                > 1 =>
            {
                Err(SearchFailure::Ambiguous)
            }
            None => Err(SearchFailure::NoMatch),
        }
    }
}

/// §5 (Iteration 155): Mark the selected agent's row in a rendered agent
/// list with a pointer. Rows 0–1 are the column header and divider, so the
/// agent at index `selected` sits at line `selected + 2`.
pub fn mark_selected_agent_row(list_text: &str, selected: usize) -> String {
    let mut out = String::new();
    for (i, line) in list_text.lines().enumerate() {
        if i >= 2 && i - 2 == selected {
            out.push('▶');
        } else {
            out.push(' ');
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}
