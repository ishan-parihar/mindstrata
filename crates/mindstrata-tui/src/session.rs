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
            View::Chronicle => View::Dashboard,
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
///
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
