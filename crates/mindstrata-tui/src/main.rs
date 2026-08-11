//! Mindstrata interactive TUI — the live observation and command console.
//!
//! §5 (Iteration 155): A ratatui event loop over the simulation. Pause/step
//! the world, browse Dashboard / Agents / Inspector / Events / Map views,
//! select an agent, and issue behavior commands (w/e/d/r/s/p) that inject
//! high-priority directive goals through [`Simulation::command_agent`].
//!
//! Controls:
//! - `q` / `Esc` / `Ctrl+C`  quit
//! - `Space`      toggle auto-run (pause/play)
//! - `n`          step one tick while paused
//! - `Tab` / `t`  cycle views
//! - `↑`/`↓`      select agent (also `j`/`k`)
//! - `w`/`e`/`d`/`r`/`s`/`p`  command: Work / Eat / Drink / Rest / Socialize / Worship
//! - `x`          cancel all directives on the selected agent
//!
//! Commands are strong nudges, not mind control: they ride the agent's own
//! goal pipeline (priority-1.0 directive goal → selection bonus), so
//! pressing survival needs can still override them — mirroring the sim's
//! critical-need interruption rule.

use std::io;
use std::ops::ControlFlow;
use std::time::{Duration, Instant};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use mindstrata_sim::institutions::InstitutionKind;
use mindstrata_sim::sim::{SimConfig, Simulation};
use mindstrata_tui::{
    key_to_command, mark_selected_agent_row, render_agent_inspector, render_agent_list,
    render_dashboard, render_event_log, render_world_map, AgentMarker, DashboardConfig, UiState,
    View,
};
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph, Wrap};
use ratatui::{init, restore, Frame};

/// Command-line arguments with defaults, parsed manually (no clap in this crate).
struct Args {
    seed: u64,
    agents: u32,
    delay_ms: u64,
}

impl Default for Args {
    fn default() -> Self {
        Self {
            seed: 42,
            agents: 12,
            delay_ms: 120,
        }
    }
}

fn parse_args() -> Args {
    let mut args = Args::default();
    let mut it = std::env::args().skip(1);
    while let Some(flag) = it.next() {
        let value = it.next();
        let Some(value) = value else {
            eprintln!("Flag {flag} requires a value");
            std::process::exit(2);
        };
        match flag.as_str() {
            "--seed" => args.seed = value.parse().unwrap_or(args.seed),
            "--agents" => args.agents = value.parse().unwrap_or(args.agents),
            "--delay" => args.delay_ms = value.parse().unwrap_or(args.delay_ms),
            other => {
                eprintln!("Unknown flag: {other}");
                std::process::exit(2);
            }
        }
    }
    args
}

fn main() -> io::Result<()> {
    let args = parse_args();
    let mut sim = Simulation::new(SimConfig {
        seed: args.seed,
        max_ticks: u64::MAX, // interactive sessions are unbounded
        world_width: 16,
        world_height: 16,
        num_agents: args.agents,
        snapshot_interval: None,
    });
    sim.populate();

    let mut ui = UiState::new(sim.agents.len());
    let tick_rate = Duration::from_millis(args.delay_ms);

    let mut terminal = init();
    let result = run_loop(&mut terminal, &mut sim, &mut ui, tick_rate);
    restore();
    result
}

fn run_loop(
    terminal: &mut ratatui::DefaultTerminal,
    sim: &mut Simulation,
    ui: &mut UiState,
    tick_rate: Duration,
) -> io::Result<()> {
    let mut last_tick = Instant::now();
    loop {
        terminal.draw(|frame| draw(frame, sim, ui))?;

        let timeout = if ui.auto_play {
            tick_rate.saturating_sub(last_tick.elapsed())
        } else {
            Duration::from_millis(100)
        };
        if event::poll(timeout)? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match handle_key(sim, ui, &key) {
                        ControlFlow::Break(()) => return Ok(()),
                        ControlFlow::Continue(()) => {}
                    }
                }
            }
        }
        if ui.auto_play && last_tick.elapsed() >= tick_rate {
            sim.tick();
            last_tick = Instant::now();
        }
    }
}

/// Handle one key press — returns `Break` to quit the loop.
fn handle_key(sim: &mut Simulation, ui: &mut UiState, key: &KeyEvent) -> ControlFlow<()> {
    let agent_count = sim.agents.len();
    match key.code {
        // Raw mode disables ISIG, so Ctrl+C arrives as a key event — treat it
        // as quit alongside q/Esc.
        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
            return ControlFlow::Break(());
        }
        KeyCode::Char('q') | KeyCode::Esc => return ControlFlow::Break(()),
        KeyCode::Char(' ') => ui.auto_play = !ui.auto_play,
        KeyCode::Char('n') => {
            sim.tick();
            ui.manual_steps += 1;
        }
        KeyCode::Tab | KeyCode::Char('t') => ui.cycle_view(),
        KeyCode::Down | KeyCode::Char('j') => ui.select_next(agent_count),
        KeyCode::Up | KeyCode::Char('k') => ui.select_prev(agent_count),
        KeyCode::Char('x') => {
            if sim.clear_commands(ui.selected_agent) {
                ui.last_command = Some("directives cleared".into());
            }
        }
        other => {
            if let Some(kind) = key_to_command(other) {
                let name = sim
                    .agents
                    .get(ui.selected_agent)
                    .map_or_else(|| "?".into(), |a| a.name.clone());
                if sim.command_agent(ui.selected_agent, kind) {
                    ui.last_command = Some(format!("{name} ← {kind:?}"));
                }
            }
        }
    }
    ControlFlow::Continue(())
}

fn draw(frame: &mut Frame, sim: &Simulation, ui: &UiState) {
    let chunks = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(1),
        Constraint::Length(3),
    ])
    .split(frame.area());

    let run_state = if ui.auto_play {
        "▶ RUNNING"
    } else {
        "⏸ PAUSED"
    };
    let header = format!(
        " Mindstrata Interactive  |  Tick {:>6}  |  {}  |  view: {}  |  [q] quit",
        sim.current_tick().as_u64(),
        run_state,
        ui.view.label(),
    );
    frame.render_widget(
        Paragraph::new(Line::from(header)).block(Block::bordered()),
        chunks[0],
    );

    let body = render_view(sim, ui);
    let overflow = body
        .lines()
        .count()
        .saturating_sub(chunks[1].height as usize);
    frame.render_widget(
        Paragraph::new(body)
            .wrap(Wrap { trim: false })
            .scroll((overflow as u16, 0)),
        chunks[1],
    );

    let selected = ui.selected_agent.min(sim.agents.len().saturating_sub(1));
    let sel_name = sim
        .agents
        .get(selected)
        .map_or_else(|| "?".into(), |a| a.name.clone());
    let command = ui.last_command.as_deref().unwrap_or("—");
    let footer = format!(
        " selected: {selected} ({sel_name})  |  steps: {}  |  ↑↓ select · space run · n step · \
         t view · w/e/d/r/s/p command · x clear  |  last: {command}",
        ui.manual_steps,
    );
    frame.render_widget(
        Paragraph::new(Line::from(footer)).block(Block::bordered()),
        chunks[2],
    );
}

fn render_view(sim: &Simulation, ui: &UiState) -> String {
    match ui.view {
        View::Dashboard => {
            let config = DashboardConfig {
                season: sim.season.current.name().to_string(),
                year: sim.season.year,
                grain: sim.total_grain().to_f64(),
                water: sim.total_water().to_f64(),
                institution_count: sim.institutions.len(),
                faction_count: sim
                    .institutions
                    .iter()
                    .filter(|i| i.kind == InstitutionKind::Faction)
                    .count(),
            };
            render_dashboard(
                &sim.agent_summaries(),
                sim.event_count(),
                sim.current_tick().as_u64(),
                &config,
            )
        }
        View::Agents => mark_selected_agent_row(
            &render_agent_list(&sim.agent_summaries()),
            ui.selected_agent,
        ),
        View::Inspector => {
            let summaries = sim.agent_summaries();
            let idx = ui.selected_agent.min(summaries.len().saturating_sub(1));
            match summaries.get(idx) {
                Some(summary) => render_agent_inspector(summary, sim.relationships()),
                None => "(no agents)".into(),
            }
        }
        View::Events => render_event_log(sim.recent_events(40), 40),
        View::Map => {
            let markers: Vec<AgentMarker> = sim
                .agents
                .iter()
                .enumerate()
                .map(|(i, a)| AgentMarker {
                    index: i,
                    x: a.position.x,
                    y: a.position.y,
                    name: a.name.chars().next().unwrap_or('?'),
                })
                .collect();
            render_world_map(sim.world(), &markers)
        }
    }
}
