//! Mindstrata TUI — debug instrument for observing simulations.
//!
//! Pure render functions live in [`render`], interactive session state
//! ([`View`], [`UiState`]) in [`session`]. Glob re-exports preserve the
//! original single-module API.

mod render;
mod session;

pub use render::{
    render_agent_inspector, render_agent_list, render_belief_inspector, render_chronicle_view,
    render_clan_dashboard, render_dashboard, render_decision_traces, render_dossier_view,
    render_event_log, render_event_log_detailed, render_faction_dashboard,
    render_institutional_records, render_market_dashboard, render_metric_charts,
    render_military_dashboard, render_noosphere_inspector, render_patronage_dashboard,
    render_psychology_inspector, render_relationship_view, render_theology_dashboard,
    render_world_map, AgentMarker, DashboardConfig,
};
pub use session::{key_to_command, mark_selected_agent_row, SearchFailure, UiState, View};

#[cfg(test)]
mod tests {
    //! Mindstrata TUI — debug instrument for observing simulations.
    //!
    //! Provides ASCII world map, agent list, event log, and system dashboard,
    //! plus the interactive session state (view tabs, selection, command keys)
    //! consumed by the `mindstrata-tui` binary's event loop.

    use super::*;
    use crossterm::event::KeyCode;
    use mindstrata_core::fixed::Fixed;
    use mindstrata_sim::culture::{
        EchoChamberState, MemeRegistry, PropagandaRegistry, RumorRegistry,
    };
    use mindstrata_sim::culture::{Meme, MemeContent};
    use mindstrata_sim::military::MilitaryRegistry;
    use mindstrata_sim::military::MilitiaMember;
    use mindstrata_sim::noosphere::{LegitimacyField, MoralPanicRegistry, NoosphericField};
    use mindstrata_sim::person::GoalKind;
    use mindstrata_sim::theology::TheologyRegistry;
    use mindstrata_sim::theology::{Religion, Temperament, TheologicalBelief};

    fn f(x: f64) -> Fixed {
        Fixed::from_f64(x)
    }

    #[test]
    fn theology_dashboard_shows_dormant_state() {
        let reg = TheologyRegistry::new();
        let out = render_theology_dashboard(&reg);
        assert!(out.contains("dormant"), "{out}");
    }

    #[test]
    fn theology_dashboard_shows_believers_and_mean_conviction() {
        let mut reg = TheologyRegistry::new();
        reg.religion = Some(Religion::seeded(
            "The Shepherd",
            Temperament::Benevolent,
            "The Way",
            vec![],
            "The Flock",
        ));
        reg.beliefs = vec![
            Some(TheologicalBelief {
                conviction: f(0.6),
                temperament_held: Temperament::Benevolent,
                since_tick: 4320,
            }),
            None,
        ];
        let out = render_theology_dashboard(&reg);
        assert!(out.contains("The Shepherd"), "{out}");
        assert!(out.contains("Believers: 1/2"), "{out}");
        assert!(out.contains("Mean conviction: 0.600"), "{out}");
        assert!(
            out.contains("Agent 0: conviction 0.600, since 4320"),
            "{out}"
        );
    }

    #[test]
    fn military_dashboard_shows_dormant_state() {
        let reg = MilitaryRegistry::new();
        let out = render_military_dashboard(&reg);
        assert!(out.contains("dormant"), "{out}");
    }

    // ── §5 (Iteration 155): Interactive session-state helpers ──

    #[test]
    fn key_to_command_maps_six_aligned_keys() {
        assert_eq!(key_to_command(KeyCode::Char('w')), Some(GoalKind::Work));
        assert_eq!(key_to_command(KeyCode::Char('e')), Some(GoalKind::Eat));
        assert_eq!(key_to_command(KeyCode::Char('d')), Some(GoalKind::Drink));
        assert_eq!(key_to_command(KeyCode::Char('r')), Some(GoalKind::Rest));
        assert_eq!(
            key_to_command(KeyCode::Char('s')),
            Some(GoalKind::Socialize)
        );
        assert_eq!(key_to_command(KeyCode::Char('p')), Some(GoalKind::Worship));
    }

    #[test]
    fn key_to_command_rejects_non_command_keys() {
        assert_eq!(key_to_command(KeyCode::Char('x')), None);
        assert_eq!(key_to_command(KeyCode::Char('W')), None); // uppercase ≠ command
        assert_eq!(key_to_command(KeyCode::Enter), None);
        assert_eq!(key_to_command(KeyCode::Up), None);
    }

    #[test]
    fn key_to_command_does_not_bind_seek_safety() {
        // SeekSafety is retained by goal generation but has no action-alignment
        // arm in select_action — binding it would silently do nothing.
        assert_eq!(key_to_command(KeyCode::Char('f')), None);
    }

    #[test]
    fn ui_state_cycles_through_all_views() {
        let mut ui = UiState::new(4);
        assert_eq!(ui.view, View::Dashboard);
        ui.cycle_view();
        assert_eq!(ui.view, View::Agents);
        ui.cycle_view();
        assert_eq!(ui.view, View::Inspector);
        ui.cycle_view();
        assert_eq!(ui.view, View::Events);
        ui.cycle_view();
        assert_eq!(ui.view, View::Map);
        ui.cycle_view();
        // Iteration 251: the Trends view joins the cycle before wrap.
        assert_eq!(ui.view, View::Trends);
        ui.cycle_view();
        // Iteration 261: the chronicle annals close the cycle.
        assert_eq!(ui.view, View::Chronicle);
        ui.cycle_view();
        // Iteration 264: the dossier pane closes the cycle.
        assert_eq!(ui.view, View::Dossier);
        ui.cycle_view();
        assert_eq!(ui.view, View::Dashboard);
    }

    #[test]
    fn name_search_resolves_exact_unique_prefix_and_index() {
        // Iteration 264: `/` search shares the CLI --dossier contract —
        // numeric index in range, exact name first, then UNIQUE prefix;
        // ambiguous prefixes and misses leave the query on screen.
        let names: Vec<String> = vec!["Anna".into(), "Bran".into(), "Beatrice".into()];
        let mut ui = UiState::new(names.len());
        ui.begin_search();
        for c in "Bran".chars() {
            ui.search_push(c);
        }
        assert_eq!(ui.resolve_search(&names), Ok(1));
        assert_eq!(ui.selected_agent, 1);
        assert!(ui.name_query.is_none(), "success ends search mode");

        // Exact beats unique-prefix semantics: 'Anna' is exact at 0.
        ui.begin_search();
        for c in "Anna".chars() {
            ui.search_push(c);
        }
        assert_eq!(ui.resolve_search(&names), Ok(0));

        // Numeric buffer jumps by index; out-of-range is a miss.
        ui.begin_search();
        ui.search_push('2');
        assert_eq!(ui.resolve_search(&names), Ok(2));
        ui.begin_search();
        ui.search_push('9');
        assert_eq!(ui.resolve_search(&names), Err(SearchFailure::NoMatch));
        assert!(ui.name_query.is_some(), "failure keeps the query visible");

        // Ambiguous prefix ('B' hits Bran + Beatrice) fails without guessing.
        ui.begin_search();
        ui.search_push('B');
        assert_eq!(ui.resolve_search(&names), Err(SearchFailure::Ambiguous));
        ui.cancel_search();

        // Backspace erases one character at a time and never underflows.
        ui.begin_search();
        for c in "Annx".chars() {
            ui.search_push(c);
        }
        ui.search_pop();
        assert_eq!(ui.name_query.as_deref(), Some("Ann"));
        for _ in 0..5 {
            ui.search_pop();
        }
        assert_eq!(ui.name_query.as_deref(), Some(""));
    }

    #[test]
    fn trends_view_renders_history_and_empty_state() {
        use mindstrata_sim::sim::MetricsSnapshot;
        let empty = render_metric_charts(&[]);
        assert!(empty.contains("No metric history yet"));
        let mut history = Vec::new();
        for t in 0..10u64 {
            let mut m = MetricsSnapshot::default();
            m.tick = t * 100;
            m.avg_stress = t as f64 / 10.0;
            history.push(m);
        }
        let rendered = render_metric_charts(&history);
        assert!(rendered.contains("Village Trends"));
        assert!(rendered.contains("stress"));
        assert!(rendered.contains("families"));
        assert!(rendered.contains("samples 10"));
    }

    #[test]
    fn psychology_inspector_shows_goal_history_learning_section() {
        // Iteration 199: the rejected/completed goal lists were write-only
        // in the sim; the psychology inspector now surfaces them as the
        // observable "goal history (learning)" record. Run a short sim so
        // the lists are populated (completions accrue from the first
        // completed action), then render and assert the section exists.
        let config = mindstrata_sim::sim::SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = mindstrata_sim::Simulation::new(config);
        sim.populate();
        sim.run(500);
        let agent = &sim.agents[0];
        let out = render_psychology_inspector(0, "anna", agent);
        assert!(out.contains("Goal History (learning)"), "{out}");
        assert!(out.contains("completed ("), "{out}");
        // The completion list must have entries after 500 ticks (the
        // golden-window probe shows ~600 completions/12 agents/1000 ticks).
        assert!(
            !agent.completed_goals.is_empty(),
            "completed_goals should be populated after 500 ticks"
        );
    }

    #[test]
    fn psychology_inspector_goal_history_handles_empty_lists() {
        // A freshly populated agent (no ticks run) has empty lists — the
        // section must degrade to the "no goal history yet" placeholder.
        let config = mindstrata_sim::sim::SimConfig {
            seed: 42,
            max_ticks: 0,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = mindstrata_sim::Simulation::new(config);
        sim.populate();
        let agent = &sim.agents[0];
        let out = render_psychology_inspector(0, "anna", agent);
        assert!(out.contains("(no goal history yet)"), "{out}");
    }

    #[test]
    fn relationship_view_shows_v2_stage_and_progress() {
        // Iteration 201: `RelationshipV2.stage_progress` was a dead field
        // (never produced, never consumed); the daily pass now writes the
        // continuous progress toward the next §10.3 stage, and the
        // relationship view surfaces it. Run a short sim (the ladder is
        // dense even at 500 ticks), render with the v2 edge, and assert
        // the stage + progress line exists.
        let config = mindstrata_sim::sim::SimConfig {
            seed: 42,
            max_ticks: 500,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = mindstrata_sim::Simulation::new(config);
        sim.populate();
        sim.run(500);
        let summaries = sim.agent_summaries();
        let v2 = sim.relationship_v2_between(0, 1);
        assert!(
            v2.is_some(),
            "a populated 12-agent world must hold the 0→1 v2 edge"
        );
        let out = render_relationship_view(
            mindstrata_core::id::AgentId::new(0),
            mindstrata_core::id::AgentId::new(1),
            sim.relationships(),
            &summaries,
            v2,
        );
        assert!(out.contains("stage:"), "{out}");
        assert!(out.contains("progress"), "{out}");
        // The write-side production must have populated the field (non-zero
        // on a dense ladder — the golden probe shows 113/132 edges past
        // Unnoticed at 1000 ticks).
        assert!(
            v2.unwrap().stage_progress > Fixed::ZERO,
            "stage_progress must be produced by the daily pass"
        );
    }

    #[test]
    fn relationship_view_handles_missing_edge() {
        let config = mindstrata_sim::sim::SimConfig {
            seed: 42,
            max_ticks: 0,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = mindstrata_sim::Simulation::new(config);
        sim.populate();
        let summaries = sim.agent_summaries();
        // Agent 0 → 0 is not a valid edge (self), so v2_edge is None.
        let out = render_relationship_view(
            mindstrata_core::id::AgentId::new(0),
            mindstrata_core::id::AgentId::new(1),
            sim.relationships(),
            &summaries,
            None,
        );
        assert!(out.contains("Relationship View"), "{out}");
        assert!(!out.contains("stage:"), "{out}");
    }

    #[test]
    fn ui_state_selection_wraps_and_clamps() {
        let mut ui = UiState::new(4);
        ui.select_next(4);
        assert_eq!(ui.selected_agent, 1);
        ui.select_prev(4);
        assert_eq!(ui.selected_agent, 0);
        ui.select_prev(4);
        assert_eq!(ui.selected_agent, 3); // wraps to last
        ui.select_next(4);
        assert_eq!(ui.selected_agent, 0);
        ui.selected_agent = 10;
        ui.clamp_selection(4);
        assert_eq!(ui.selected_agent, 3);
        ui.clamp_selection(0);
        assert_eq!(ui.selected_agent, 0);
    }

    #[test]
    fn mark_selected_agent_row_points_at_selected_row() {
        let text = "header\n────\nanna\nbran\ncara\n";
        let out = mark_selected_agent_row(text, 1);
        let lines: Vec<&str> = out.lines().collect();
        assert!(lines[0].starts_with(' '), "header unmarked: {lines:?}");
        assert!(lines[1].starts_with(' '), "divider unmarked: {lines:?}");
        assert!(lines[2].starts_with(' '), "agent 0 unmarked: {lines:?}");
        assert!(lines[3].starts_with('▶'), "selected row marked: {lines:?}");
        assert!(lines[4].starts_with(' '), "agent 2 unmarked: {lines:?}");
    }

    #[test]
    fn mark_selected_agent_row_ignores_out_of_range_selection() {
        let text = "header\n────\nanna\n";
        let out = mark_selected_agent_row(text, 9);
        assert!(
            out.lines().all(|l| l.starts_with(' ')),
            "no row marked: {out}"
        );
    }

    #[test]
    fn military_dashboard_shows_readiness_and_roster() {
        let mut reg = MilitaryRegistry::new();
        reg.readiness = f(0.45);
        reg.conscripts = 2;
        reg.musters = 1;
        reg.drills = 2;
        reg.roster = vec![
            Some(MilitiaMember {
                enlisted_since: 4320,
                dominance_at_enlistment: f(0.7),
            }),
            None,
            Some(MilitiaMember {
                enlisted_since: 8640,
                dominance_at_enlistment: f(0.5),
            }),
        ];
        let out = render_military_dashboard(&reg);
        assert!(out.contains("Readiness: 0.450"), "{out}");
        assert!(out.contains("Militia: 2"), "{out}");
        assert!(
            out.contains("Agent 0: since 4320, dominance 0.700"),
            "{out}"
        );
    }

    #[test]
    fn noosphere_inspector_shows_empty_registries_as_dormant() {
        let memes = MemeRegistry::default();
        let panics = MoralPanicRegistry::new();
        let propaganda = PropagandaRegistry::default();
        let legitimacy = LegitimacyField::new(f(0.5));
        let legitimacy_refs = [&legitimacy];
        let echo = EchoChamberState::default();
        let rumors = RumorRegistry::default();
        let noosphere = NoosphericField::new();
        let out = render_noosphere_inspector(
            &memes,
            &panics,
            &propaganda,
            &legitimacy_refs,
            &echo,
            &rumors,
            &noosphere,
        );
        assert!(out.contains("Noosphere / Culture Inspector"), "{out}");
        assert!(out.contains("Agents: 1   Mean overall: 0.500"), "{out}");
        assert!(out.contains("no memes seeded"), "{out}");
        assert!(out.contains("no moral panics recorded"), "{out}");
        assert!(out.contains("no campaigns"), "{out}");
        assert!(out.contains("no rumors"), "{out}");
        assert!(out.contains("field empty"), "{out}");
    }

    #[test]
    fn noosphere_inspector_shows_live_state_across_all_sections() {
        let mut memes = MemeRegistry::default();
        memes.register(Meme::new(
            0,
            "The temple hoards grain".to_string(),
            MemeContent::Rumor,
            f(0.8),
            f(0.6),
            100,
            f(0.8),
            f(0.05),
        ));

        let mut panics = MoralPanicRegistry::new();
        let mut panic = mindstrata_sim::noosphere::MoralPanic::new(
            mindstrata_sim::noosphere::PanicTrigger::InstitutionalCorruption,
            Some(3),
            900,
        );
        panic.participants = 12;
        panic.active = false;
        panics.register(panic);

        let mut propaganda = PropagandaRegistry::default();
        propaganda.register(mindstrata_sim::culture::PropagandaCampaign::new(
            0,
            0,
            vec![0, 1, 2, 3],
            "Order preserves survival".to_string(),
            f(0.7),
            vec![mindstrata_sim::culture::PropagandaChannel::Edict],
            2000,
            500,
        ));

        let mut legitimacy = LegitimacyField::new(f(0.6));
        legitimacy
            .sources
            .push(mindstrata_sim::noosphere::LegitimacySource {
                name: "divine mandate".to_string(),
                strength: f(0.4),
                decay_rate: f(0.001),
                requires_ritual: true,
            });
        let legitimacy_refs = [&legitimacy];

        let mut echo = EchoChamberState::default();
        echo.polarization_index = f(0.35);
        echo.echo_chamber_strength = f(0.42);
        echo.narrative_dominance.insert(0, f(0.61));

        let mut rumors = RumorRegistry::default();
        rumors.register(mindstrata_sim::culture::RumorV2::new(
            0,
            "The well is poisoned".to_string(),
            Some(5),
            f(0.6),
            f(0.4),
            f(0.7),
            700,
        ));

        let mut noosphere = NoosphericField::new();
        noosphere
            .nodes
            .push(mindstrata_sim::noosphere::SymbolicNode::new(
                0,
                Default::default(),
            ));

        let out = render_noosphere_inspector(
            &memes,
            &panics,
            &propaganda,
            &legitimacy_refs,
            &echo,
            &rumors,
            &noosphere,
        );
        assert!(out.contains("Memes: 1 total, 1 active"), "{out}");
        assert!(out.contains("\"The temple hoards grain\""), "{out}");
        assert!(out.contains("InstitutionalCorruption"), "{out}");
        assert!(out.contains("participants 12 [resolved]"), "{out}");
        assert!(
            out.contains("\"Order preserves survival\" via Edict"),
            "{out}"
        );
        assert!(
            out.contains("Agent 0: overall 0.600 (1 source, decay 0.0001)"),
            "{out}"
        );
        assert!(out.contains("Polarization: 0.350"), "{out}");
        assert!(out.contains("meme #0: dominance 0.610"), "{out}");
        assert!(out.contains("#0 [agent 5]"), "{out}");
        assert!(out.contains("\"The well is poisoned\""), "{out}");
        assert!(out.contains("Symbolic nodes: 1   Edges: 0"), "{out}");
        assert!(out.contains("node #0 activation 0.300"), "{out}");
    }
}
