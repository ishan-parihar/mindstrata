//! Mindstrata CLI — run headless simulations.

use clap::{Parser, Subcommand};
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::{sim::SimConfig, Simulation};

/// Mindstrata: a deterministic, emergent human-society simulation.
#[derive(Parser)]
#[command(name = "mindstrata", version, about)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
#[expect(
    clippy::large_enum_variant,
    reason = "clap arg container: the Sim variant carries many optional dashboard flags; boxing individual fields would complicate the derive and gain nothing"
)]
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

        /// Load and apply a content pack (mod) from a directory before running.
        #[arg(long, value_name = "DIR")]
        mod_dir: Option<String>,

        /// Verbose logging.
        #[arg(short, long)]
        verbose: bool,

        /// Show world map after simulation.
        #[arg(long)]
        map: bool,

        /// Render the final world state as a PNG map (terrain + sites +
        /// agent sprites) to the given path.
        #[arg(long, value_name = "PATH")]
        render_map: Option<String>,

        /// Render an animated replay GIF (AP2 Phase 5 — replay
        /// visualizations): one frame every `--replay-every` ticks, sampled
        /// live during the run, written to the given path.
        #[arg(long, value_name = "PATH")]
        render_replay: Option<String>,

        /// Cadence for `--render-replay`: sample a frame every N ticks.
        /// Must be ≥ 1; the final tick is always included.
        #[arg(long, default_value_t = 24)]
        replay_every: u64,

        /// Inspect a specific agent by ID after simulation.
        #[arg(long)]
        inspect_agent: Option<usize>,

        /// Print the village chronicle (year-by-year annals) after simulation.
        #[arg(long)]
        chronicle: bool,

        /// Print a dossier for an agent given by index or exact/prefix name.
        #[arg(long, value_name = "IDX_OR_NAME")]
        dossier: Option<String>,

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

        /// Show clan dashboard (members, alliances, enmities, §10.8).
        #[arg(long)]
        clans: bool,

        /// Show patronage dashboard (patron-client relations, §10.9).
        #[arg(long)]
        patronage: bool,

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

        /// Show the noosphere/culture inspector — legitimacy field, meme
        /// pool, moral panics, propaganda campaigns, echo-chamber
        /// polarization, rumors, and the symbolic noospheric field.
        #[arg(long)]
        noosphere: bool,

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
            mod_dir,
            verbose,
            map,
            render_map,
            render_replay,
            replay_every,
            inspect_agent,
            chronicle,
            dossier,
            show_relationships,
            market,
            beliefs,
            factions,
            clans,
            patronage,
            records,
            decisions,
            psychology,
            timeline,
            noosphere,
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

            // §5 (Iteration 156): modding API — apply the content pack (if
            // any) before running, so mod norms/knowledge affect the whole
            // run. Strictly opt-in via --mod; no default world loads a pack.
            if let Some(ref mod_dir) = mod_dir {
                match mindstrata_sim::mods::ContentPack::load(std::path::Path::new(mod_dir)) {
                    Ok(pack) => {
                        println!("  Mod:    {}", pack.describe());
                        match sim.apply_content_pack(&pack) {
                            Ok(applied) => {
                                println!(
                                    "  Applied: {} norms, {} knowledge items added",
                                    applied.norms_added, applied.knowledge_added
                                );
                                if let Some(ref scenario) = pack.scenario {
                                    println!(
                                        "  Pack scenario: '{}' — run it with: scenario {}/scenario.ron",
                                        scenario.name,
                                        mod_dir.trim_end_matches('/')
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!("Failed to apply mod: {e}");
                                std::process::exit(1);
                            }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load mod from {mod_dir}: {e}");
                        std::process::exit(1);
                    }
                }
            }

            println!("Running {ticks} ticks...");
            let start = std::time::Instant::now();

            // §20 (Iteration 171): animated replay — when `--render-replay`
            // is set, step the simulation manually so frames can be sampled
            // at the requested cadence. Sampling reads world state only
            // (no RNG, no mutation), so the tick stream is byte-identical
            // to a plain `sim.run(ticks)` — replay capture cannot perturb
            // the simulation.
            let replay_frames: Option<
                Vec<(
                    mindstrata_sim::world::World,
                    Vec<mindstrata_render::RenderAgent>,
                )>,
            > = if let Some(ref path) = render_replay {
                let every = replay_every.max(1);
                let mut frames = Vec::new();
                let sample = |sim: &Simulation, frames: &mut Vec<_>| {
                    let agents: Vec<mindstrata_render::RenderAgent> = sim
                        .agents
                        .iter()
                        .enumerate()
                        .map(|(i, a)| {
                            mindstrata_render::RenderAgent::new(a.position.x, a.position.y, i as u8)
                        })
                        .collect();
                    frames.push((sim.world().clone(), agents));
                };
                // Frame 0: the populated starting state.
                sample(&sim, &mut frames);
                for i in 0..ticks {
                    sim.tick();
                    if (i + 1) % every == 0 {
                        sample(&sim, &mut frames);
                    }
                }
                println!(
                    "  Replay sampled {} frames (every {every} ticks) → {path}",
                    frames.len()
                );
                Some(frames)
            } else {
                sim.run(ticks);
                None
            };
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
                let markers: Vec<mindstrata_tui::AgentMarker> = sim
                    .agents
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let name_char = a.name.chars().next().unwrap_or('?');
                        mindstrata_tui::AgentMarker {
                            index: i,
                            x: a.position.x,
                            y: a.position.y,
                            name: name_char,
                        }
                    })
                    .collect();
                println!(
                    "{}",
                    mindstrata_tui::render_world_map(sim.world(), &markers)
                );
            }

            // §5 (Iteration 157): Visual rendering — pixel map export.
            // The renderer is a pure function of the world state (no RNG,
            // read-only), so this never perturbs the simulation.
            if let Some(ref path) = render_map {
                let agents: Vec<mindstrata_render::RenderAgent> = sim
                    .agents
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        mindstrata_render::RenderAgent::new(a.position.x, a.position.y, i as u8)
                    })
                    .collect();
                let w = sim.world().width * mindstrata_render::DEFAULT_CELL_PIXELS;
                let h = sim.world().height * mindstrata_render::DEFAULT_CELL_PIXELS;
                match mindstrata_render::render_world_png(
                    sim.world(),
                    &agents,
                    mindstrata_render::DEFAULT_CELL_PIXELS,
                ) {
                    Ok(png) => match std::fs::write(path, &png) {
                        Ok(()) => println!("\n  Rendered world map ({w}x{h}px) to: {path}"),
                        Err(e) => eprintln!("\n  Failed to write rendered map: {e}"),
                    },
                    Err(e) => eprintln!("\n  Failed to encode rendered map: {e}"),
                }
            }

            // §20 (Iteration 171): write the animated replay GIF.
            if let (Some(path), Some(frames)) = (render_replay.as_ref(), replay_frames.as_ref()) {
                let replay: Vec<mindstrata_render::ReplayFrame> = frames
                    .iter()
                    .map(|(world, agents)| mindstrata_render::ReplayFrame { world, agents })
                    .collect();
                match mindstrata_render::render_replay_gif(
                    &replay,
                    mindstrata_render::DEFAULT_CELL_PIXELS,
                    200,
                    image::codecs::gif::Repeat::Infinite,
                ) {
                    Ok(gif) => match std::fs::write(path, &gif) {
                        Ok(()) => {
                            let (fw, fh) = frames.first().map_or((0, 0), |(w, _)| {
                                (
                                    w.width * mindstrata_render::DEFAULT_CELL_PIXELS,
                                    w.height * mindstrata_render::DEFAULT_CELL_PIXELS,
                                )
                            });
                            println!(
                                "\n  Animated replay ({} frames, {fw}x{fh}px) written to: {path}",
                                frames.len()
                            );
                        }
                        Err(e) => eprintln!("\n  Failed to write replay GIF: {e}"),
                    },
                    Err(e) => eprintln!("\n  Failed to encode replay GIF: {e}"),
                }
            }

            // §17.1: Agent inspector
            if let Some(id) = inspect_agent {
                let summaries = sim.agent_summaries();
                if let Some(summary) = summaries.iter().find(|s| s.index == id) {
                    println!();
                    println!(
                        "{}",
                        mindstrata_tui::render_agent_inspector(summary, sim.relationships())
                    );
                } else {
                    eprintln!("Agent {id} not found.");
                }
            }

            // Iteration 259 (Phase 6): village chronicle + agent dossiers.
            if chronicle {
                println!();
                print!("{}", mindstrata_sim::sim::chronicle::render_chronicle(&sim));
            }
            if let Some(ref spec) = dossier {
                // Iteration 261: accept either a numeric index or an agent
                // name (exact match first, then unique prefix).
                let resolved: Option<usize> = spec.parse::<usize>().ok().or_else(|| {
                    let exact = sim.agents.iter().position(|a| a.name == *spec);
                    exact.or_else(|| {
                        let prefix_hits: Vec<usize> = sim
                            .agents
                            .iter()
                            .enumerate()
                            .filter(|(_, a)| a.name.starts_with(spec.as_str()))
                            .map(|(i, _)| i)
                            .collect();
                        if prefix_hits.len() == 1 {
                            Some(prefix_hits[0])
                        } else {
                            None
                        }
                    })
                });
                println!();
                match resolved {
                    Some(idx) => println!(
                        "{}",
                        mindstrata_sim::sim::chronicle::render_dossier(&sim, idx)
                    ),
                    None => eprintln!("No agent matches '{spec}'."),
                }
            }

            // §17.1: Relationship view
            if let Some(ref spec) = show_relationships {
                let parts: Vec<&str> = spec.split(',').collect();
                if parts.len() == 2 {
                    if let (Ok(from), Ok(to)) =
                        (parts[0].parse::<usize>(), parts[1].parse::<usize>())
                    {
                        let summaries = sim.agent_summaries();
                        println!();
                        println!(
                            "{}",
                            mindstrata_tui::render_relationship_view(
                                mindstrata_core::id::AgentId::new(from as u64),
                                mindstrata_core::id::AgentId::new(to as u64),
                                sim.relationships(),
                                &summaries,
                                sim.relationship_v2_between(from, to),
                            )
                        );
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
                    println!(
                        "{}",
                        mindstrata_tui::render_belief_inspector(
                            &summary.name,
                            summary.index,
                            &sim.agents[id].beliefs,
                        )
                    );
                } else {
                    eprintln!("Agent {id} not found.");
                }
            }

            // §17.1: Institution/faction dashboard
            if factions {
                println!();
                println!(
                    "{}",
                    mindstrata_tui::render_faction_dashboard(&sim.institutions)
                );
            }

            // §10.8: Clan dashboard
            if clans {
                println!();
                println!(
                    "{}",
                    mindstrata_tui::render_clan_dashboard(&sim.clan_registry)
                );
            }

            // §10.9: Patronage dashboard
            if patronage {
                println!();
                println!(
                    "{}",
                    mindstrata_tui::render_patronage_dashboard(&sim.patronage_registry)
                );
            }

            // §19.5.J: Institutional records
            if records {
                if let Some(council) = sim
                    .institutions
                    .iter()
                    .find(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Council)
                {
                    println!();
                    println!(
                        "{}",
                        mindstrata_tui::render_institutional_records(
                            &council.name,
                            &council.records,
                            20,
                        )
                    );
                } else {
                    eprintln!("No Council institution found.");
                }
            }

            // §19.5.J: Decision traces
            if let Some(id) = decisions {
                println!();
                println!(
                    "{}",
                    mindstrata_tui::render_decision_traces(
                        sim.provenance(),
                        mindstrata_core::id::AgentId::new(id as u64),
                        20,
                    )
                );
            }

            // §22: Full psychology pipeline inspector
            if let Some(id) = psychology {
                if id < sim.agents.len() {
                    println!();
                    println!(
                        "{}",
                        mindstrata_tui::render_psychology_inspector(
                            id,
                            &sim.agents[id].name,
                            &sim.agents[id],
                        )
                    );
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

            // §13 (Iteration 178): Noosphere/culture inspector
            if noosphere {
                let legitimacy_fields: Vec<_> =
                    sim.agents.iter().map(|a| &a.legitimacy_field).collect();
                println!();
                println!(
                    "{}",
                    mindstrata_tui::render_noosphere_inspector(
                        &sim.meme_registry,
                        &sim.moral_panic_registry,
                        &sim.propaganda_registry,
                        &legitimacy_fields,
                        &sim.echo_chamber,
                        &sim.rumor_registry,
                        &sim.noospheric_field,
                    )
                );
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
                        Ok(()) => {
                            println!("\n  Metrics exported to: {path} ({} rows)", metrics.len());
                        }
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
                "famine" => Scenario::famine(),
                "pestilence" => Scenario::pestilence(),
                "collapse" => Scenario::collapse(),
                // §46 (Iteration 161): the Calm scenario (no shocks) is the
                // control baseline for differential testing — exposed in the
                // CLI so a control run is one command away.
                "calm" => Scenario::calm(),
                // §5 (Iteration 156): modding API — run any .ron scenario file
                // by path (e.g. a mod pack's scenario.ron).
                other => match Scenario::from_file(other) {
                    Ok(s) => s,
                    Err(e) => {
                        eprintln!("Unknown scenario: {other}");
                        eprintln!(
                            "Available: riverford, drought, famine, pestilence, collapse, calm"
                        );
                        eprintln!("Or pass a path to a .ron scenario file: {e}");
                        std::process::exit(1);
                    }
                },
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

            // Run the scenario's declared horizon — not a hardcoded 1000.
            // Previously every scenario was truncated to 1000 ticks, which
            // silently ignored `ticks` (e.g. drought declares 4320) and
            // collapsed the scenarios into near-identical outcomes.
            let scenario_ticks = scenario.ticks;
            let mut sim = Simulation::from_scenario(scenario);
            sim.populate();

            println!("Running {scenario_ticks} ticks...");
            let start = std::time::Instant::now();
            sim.run(scenario_ticks);
            let elapsed = start.elapsed();

            print_results(&sim, elapsed);

            if map {
                println!();
                let markers: Vec<mindstrata_tui::AgentMarker> = sim
                    .agents
                    .iter()
                    .enumerate()
                    .map(|(i, a)| {
                        let name_char = a.name.chars().next().unwrap_or('?');
                        mindstrata_tui::AgentMarker {
                            index: i,
                            x: a.position.x,
                            y: a.position.y,
                            name: name_char,
                        }
                    })
                    .collect();
                println!(
                    "{}",
                    mindstrata_tui::render_world_map(sim.world(), &markers)
                );
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
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| filter.into()),
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

    println!(
        "{}",
        mindstrata_tui::render_agent_list(&sim.agent_summaries())
    );

    println!();
    println!(
        "{}",
        mindstrata_tui::render_dashboard(
            &sim.agent_summaries(),
            sim.event_count(),
            sim.current_tick().as_u64(),
            &mindstrata_tui::DashboardConfig {
                season: sim.season.current.name().to_string(),
                year: sim.season.year,
                grain: sim.total_grain().to_f64(),
                water: sim.total_water().to_f64(),
                institution_count: sim.institutions.len(),
                faction_count: sim
                    .institutions
                    .iter()
                    .filter(|i| i.kind == mindstrata_sim::institutions::InstitutionKind::Faction)
                    .count(),
            },
        )
    );

    // Show last 10 events
    let events = sim.recent_events(10);
    if !events.is_empty() {
        println!("Recent events:");
        println!("{}", mindstrata_tui::render_event_log(events, 10));
    }
}
