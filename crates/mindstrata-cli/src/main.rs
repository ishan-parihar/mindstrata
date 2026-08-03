//! Mindstrata CLI — run headless simulations.

use clap::{Parser, Subcommand};
use mindstrata_sim::{Simulation, sim::SimConfig};
use mindstrata_sim::scenario::Scenario;

/// Mindstrata: a deterministic, emergent human-society simulation.
#[derive(Parser)]
#[command(name = "mindstrata", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run a headless simulation.
    Sim {
        /// RNG seed for determinism.
        #[arg(long, default_value_t = 42)]
        seed: u64,

        /// Number of ticks to simulate.
        #[arg(long, default_value_t = 1000)]
        ticks: u64,

        /// Number of initial agents.
        #[arg(long, default_value_t = 12)]
        agents: u32,

        /// Verbose logging.
        #[arg(short, long)]
        verbose: bool,

        /// Show world map after simulation.
        #[arg(long)]
        map: bool,

        /// Inspect a specific agent by ID after simulation.
        #[arg(long)]
        inspect_agent: Option<usize>,

        /// Show relationship between two agents (format: from,to).
        #[arg(long)]
        show_relationships: Option<String>,

        /// Show market dashboard (prices, inequality, trade volume).
        #[arg(long)]
        market: bool,

        /// Show beliefs for a specific agent by ID.
        #[arg(long)]
        beliefs: Option<usize>,

        /// Show institution/faction dashboard.
        #[arg(long)]
        factions: bool,

        /// Show institutional records for the Council.
        #[arg(long)]
        records: bool,

        /// Show decision traces for a specific agent by ID.
        #[arg(long)]
        decisions: Option<usize>,

        /// Show full psychology pipeline for a specific agent by ID.
        #[arg(long)]
        psychology: Option<usize>,

        /// Show chronological event timeline after simulation.
        #[arg(long)]
        timeline: Option<usize>,

        /// Save a snapshot to disk after simulation (path to .snapshot file).
        #[arg(long)]
        save_snapshot: Option<String>,

        /// Load a snapshot from disk and resume simulation for N more ticks.
        #[arg(long)]
        load_snapshot: Option<String>,

        /// Export metric history to CSV file.
        #[arg(long)]
        export_metrics: Option<String>,
    },
    /// Run a named scenario.
    Scenario {
        /// Scenario name (e.g. "riverford").
        #[arg(default_value = "riverford")]
        name: String,

        /// Verbose logging.
        #[arg(short, long)]
        verbose: bool,

        /// Show world map after simulation.
        #[arg(long)]
        map: bool,
    },
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sim {
            seed,
            ticks,
            agents,
            verbose,
            map,
            inspect_agent,
            show_relationships,
            market,
            beliefs,
            factions,
            records,
            decisions,
            psychology,
            timeline,
            save_snapshot,
            load_snapshot,
            export_metrics,
        } => {
            init_logging(verbose);

            let config = SimConfig {
                seed,
                max_ticks: ticks,
                world_width: 16,
                world_height: 16,
                num_agents: agents,
                snapshot_interval: None,
            };

            println!("╔══════════════════════════════════════════════╗");
            println!("║  Mindstrata v{}", env!("CARGO_PKG_VERSION"));
            println!("║  A Deterministic Emergent Society Simulation");
            println!("╚══════════════════════════════════════════════╝");
            println!();
            println!("  Seed:    {seed}");
            println!("  Ticks:   {ticks}");
            println!("  Agents:  {agents}");
            println!();

            // §16.1: Snapshot loading — restore from disk if requested
            let mut sim = if let Some(ref path) = load_snapshot {
                let snapshot = mindstrata_sim::snapshot::Snapshot::load(std::path::Path::new(path))
                    .unwrap_or_else(|e| {
                        eprintln!("Failed to load snapshot: {e}");
                        std::process::exit(1);
                    });
                println!("  Loaded snapshot from tick {}", snapshot.tick);
                Simulation::from_snapshot(snapshot)
            } else {
                let mut s = Simulation::new(config);
                s.populate();
                s
            };

            println!("Running {ticks} ticks...");
            let start = std::time::Instant::now();
            sim.run(ticks);
            let elapsed = start.elapsed();

            print_results(&sim, elapsed);

            // §16.1: Save snapshot to disk if requested
            if let Some(ref path) = save_snapshot {
                let snapshot = sim.capture_snapshot();
                let save_path = std::path::Path::new(path);
                match snapshot.save(save_path) {
                    Ok(()) => println!("\n  Snapshot saved to: {path}"),
                    Err(e) => eprintln!("\n  Failed to save snapshot: {e}"),
                }
            }

            if map {
                println!();
                let markers: Vec<mindstrata_tui::AgentMarker> = sim.agents.iter().enumerate().map(|(i, a)| {
                    let name_char = a.name.chars().next().unwrap_or('?');
                    mindstrata_tui::AgentMarker {
                        index: i,
                        x: a.position.x,
                        y: a.position.y,
                        name: name_char,
                    }
                }).collect();
                println!("{}", mindstrata_tui::render_world_map(sim.world(), &markers));
            }

            // §17.1: Agent inspector
            if let Some(id) = inspect_agent {
                let summaries = sim.agent_summaries();
                if let Some(summary) = summaries.iter().find(|s| s.index == id) {
                    println!();
                    println!("{}", mindstrata_tui::render_agent_inspector(summary, sim.relationships()));
                } else {
                    eprintln!("Agent {id} not found.");
                }
            }

            // §17.1: Relationship view
            if let Some(ref spec) = show_relationships {
                let parts: Vec<&str> = spec.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(from), Ok(to)) = (parts[0].parse::<usize>(), parts[1].parse::<usize>()) {
                        let summaries = sim.agent_summaries();
                        println!();
                        println!("{}", mindstrata_tui::render_relationship_view(
                            mindstrata_core::id::AgentId::new(from as u64),
                            mindstrata_core::id::AgentId::new(to as u64),
                            sim.relationships(),
                            &summaries,
                        ));
                    } else {
                        eprintln!("Invalid format. Use: --show-relationships from,to");
                    }
                } else {
                    eprintln!("Invalid format. Use: --show-relationships from,to");
                }
            }

            // §17.1: Market dashboard
            if market {
                println!();
                println!("{}", mindstrata_tui::render_market_dashboard(&sim.market));
            }

            // §17.1: Belief inspector
            if let Some(id) = beliefs {
                let summaries = sim.agent_summaries();
                if let Some(summary) = summaries.iter().find(|s| s.index == id) {
                    println!();
                    println!("{}", mindstrata_tui::render_belief_inspector(
                        &summary.name, summary.index,
                        &sim.agents[id].beliefs,
                    ));
                } else {
                    eprintln!("Agent {id} not found.");
                }
            }

            // §17.1: Institution/faction dashboard
            if factions {
                println!();
                println!("{}", mindstrata_tui::render_faction_dashboard(&sim.institutions));
            }

            // §19.5.J: Institutional records
            if records {
                if let Some(council) = sim.institutions.iter().find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Council) {
                    println!();
                    println!("{}", mindstrata_tui::render_institutional_records(
                        &council.name, &council.records, 20,
                    ));
                } else {
                    eprintln!("No Council institution found.");
                }
            }

            // §19.5.J: Decision traces
            if let Some(id) = decisions {
                println!();
                println!("{}", mindstrata_tui::render_decision_traces(
                    sim.provenance(),
                    mindstrata_core::id::AgentId::new(id as u64),
                    20,
                ));
            }

            // §22: Full psychology pipeline inspector
            if let Some(id) = psychology {
                if id < sim.agents.len() {
                    println!();
                    println!("{}", mindstrata_tui::render_psychology_inspector(
                        id,
                        &sim.agents[id].name,
                        &sim.agents[id],
                    ));
                } else {
                    eprintln!("Agent {id} not found.");
                }
            }

            // §6.3: Event timeline view
            if let Some(count) = timeline {
                if count > 0 {
                    println!();
                    let events = sim.recent_events(count);
                    if !events.is_empty() {
                        println!("╔══════════════════════════════════════════════╗");
                        println!("║  Event Timeline (last {count})                       ║");
                        println!("╚══════════════════════════════════════════════╝");
                        println!("{}", mindstrata_tui::render_event_log(events, count));
                    } else {
                        println!("No events recorded.");
                    }
                }
            }

            // §6.5 + §17: Export metric history to CSV using MetricsSnapshot methods
            if let Some(ref path) = export_metrics {
                use mindstrata_sim::sim::MetricsSnapshot;
                let metrics = &sim.metric_history;
                if metrics.is_empty() {
                    eprintln!("No metric history to export.");
                } else {
                    let mut csv = String::from(MetricsSnapshot::csv_header());
                    csv.push('\n');
                    for m in metrics {
                        csv.push_str(&m.to_csv_line());
                        csv.push('\n');
                    }
                    match std::fs::write(path, &csv) {
                        Ok(()) => println!("\n  Metrics exported to: {path} ({} rows)", metrics.len()),
                        Err(e) => eprintln!("\n  Failed to export metrics: {e}"),
                    }
                }
            }
        }

        Commands::Scenario { name, verbose, map } => {
            init_logging(verbose);

            let scenario = match name.as_str() {
                "riverford" => Scenario::riverford(),
                "drought" => Scenario::drought(),
                other => {
                    eprintln!("Unknown scenario: {other}");
                    eprintln!("Available: riverford, drought");
                    std::process::exit(1);
                }
            };

            println!("╔══════════════════════════════════════════════╗");
            println!("║  Mindstrata v{}", env!("CARGO_PKG_VERSION"));
            println!("║  Scenario: {}", scenario.name);
            println!("╚══════════════════════════════════════════════╝");
            println!();
            println!("  {}", scenario.description);
            println!();
            println!("  Seed:    {}", scenario.seed);
            println!("  Ticks:   {}", scenario.ticks);
            println!("  Agents:  {}", scenario.num_agents);
            println!("  Shocks:  {}", scenario.shocks.len());
            println!();

            let mut sim = Simulation::from_scenario(scenario);
            sim.populate();

            println!("Running simulation...");
            let start = std::time::Instant::now();
            sim.run(1000); // run the scenario's ticks
            let elapsed = start.elapsed();

            print_results(&sim, elapsed);

            if map {
                println!();
                let markers: Vec<mindstrata_tui::AgentMarker> = sim.agents.iter().enumerate().map(|(i, a)| {
                    let name_char = a.name.chars().next().unwrap_or('?');
                    mindstrata_tui::AgentMarker {
                        index: i,
                        x: a.position.x,
                        y: a.position.y,
                        name: name_char,
                    }
                }).collect();
                println!("{}", mindstrata_tui::render_world_map(sim.world(), &markers));
            }
        }
    }
}

fn init_logging(verbose: bool) {
    let filter = if verbose {
        "mindstrata_core=debug,mindstrata_sim=debug"
    } else {
        "mindstrata_sim=warn"
    };
    let _ = tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| filter.into()),
        )
        .try_init();
}

fn print_results(sim: &Simulation, elapsed: std::time::Duration) {
    let metrics = sim.metrics_snapshot();

    println!();
    println!("╔══════════════════════════════════════════════╗");
    println!("║  Final State                                 ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("  Ticks completed: {}", metrics.tick);
    println!("  Agents:          {}", metrics.agent_count);
    println!("  Events:          {}", metrics.event_count);
    println!("  Journal entries: {}", metrics.journal_len);
    println!("  Time elapsed:    {elapsed:.2?}");
    println!();

    println!("╔══════════════════════════════════════════════╗");
    println!("║  Metrics Summary                             ║");
    println!("╚══════════════════════════════════════════════╝");
    println!("  Avg Hunger:    {:.3}", metrics.avg_hunger);
    println!("  Avg Thirst:    {:.3}", metrics.avg_thirst);
    println!("  Avg Fatigue:   {:.3}", metrics.avg_fatigue);
    println!("  Avg Valence:   {:.3}", metrics.avg_valence);
    println!("  Avg Joy:       {:.3}", metrics.avg_joy);
    println!("  Avg Fear:      {:.3}", metrics.avg_fear);
    println!("  Total Grain:   {:.1}", metrics.total_grain);
    println!("  Total Water:   {:.1}", metrics.total_water);
    println!();

    println!("{}", mindstrata_tui::render_agent_list(&sim.agent_summaries()));

    println!();
    println!("{}", mindstrata_tui::render_dashboard(
        &sim.agent_summaries(),
        sim.event_count(),
        sim.current_tick().as_u64(),
        &mindstrata_tui::DashboardConfig {
            season: sim.season.current.name().to_string(),
            year: sim.season.year,
            grain: sim.total_grain().to_f64(),
            water: sim.total_water().to_f64(),
            institution_count: sim.institutions.len(),
            faction_count: sim.institutions.iter().filter(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Faction).count(),
        },
    ));

    // Show last 10 events
    let events = sim.recent_events(10);
    if !events.is_empty() {
        println!("Recent events:");
        println!("{}", mindstrata_tui::render_event_log(events, 10));
    }
}
