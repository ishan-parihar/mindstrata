//! Mindstrata CLI — run headless simulations.

use anyhow::Result;
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

fn main() -> Result<()> {
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

            let mut sim = Simulation::new(config);
            sim.populate();

            println!("Running {ticks} ticks...");
            let start = std::time::Instant::now();
            sim.run(ticks);
            let elapsed = start.elapsed();

            print_results(&sim, elapsed);

            if map {
                println!();
                println!("{}", mindstrata_tui::render_world_map(sim.world()));
            }

            // §17.1: Agent inspector
            if let Some(id) = inspect_agent {
                let summaries = sim.agent_summaries();
                if let Some(summary) = summaries.iter().find(|s| s.index == id) {
                    println!();
                    println!("{}", mindstrata_tui::render_agent_inspector(summary, sim.relationships()));
                } else {
                    eprintln!("Agent {} not found.", id);
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
        }

        Commands::Scenario { name, verbose, map } => {
            init_logging(verbose);

            let scenario = match name.as_str() {
                "riverford" => Scenario::riverford(),
                _ => {
                    eprintln!("Unknown scenario: {name}");
                    eprintln!("Available: riverford");
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
                println!("{}", mindstrata_tui::render_world_map(sim.world()));
            }
        }
    }

    Ok(())
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
    println!("  Time elapsed:    {:.2?}", elapsed);
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
