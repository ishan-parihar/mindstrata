//! Per-agent lifetime trace harness (run explicitly with `--ignored`).
//!
//! Traces agent 0 ("Anna") tick-by-tick for a full scheduler year (51,840
//! ticks) and dumps her psychological state + relevant world events to a CSV
//! so we can test whether emotions/stress ever react at seasonal or yearly
//! timescales.
//!
//! Run: `cargo test --release -p mindstrata-tests anna_year_trace -- --ignored --nocapture`

use mindstrata_core::conflict::ConflictKind;
use mindstrata_core::event::SimEvent;
use mindstrata_core::id::AgentId;
use mindstrata_sim::person::Belief;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

/// Scheduler year: 51,840 ticks (144-tick days × 360-day year).
const YEAR_TICKS: u64 = 51_840;
/// Season tracker cadence: 8,760 ticks/season (4 seasons → 35,040/year).
const SEASON_TICKS: u64 = 8_760;

#[test]
#[ignore = "long-running trace harness; run explicitly with --ignored"]
fn anna_year_trace() {
    let config = SimConfig {
        seed: 42,
        max_ticks: YEAR_TICKS,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();

    // Anna is names[0] → agent index 0.
    assert_eq!(sim.agents[0].name, "Anna", "agent 0 should be Anna");

    let mut out = String::with_capacity(YEAR_TICKS as usize * 80);
    out.push_str("tick,season,season_year,season_tick,name,age,partner,");
    out.push_str("hunger,thirst,fatigue,safety,health,wealth,");
    out.push_str("fear,anger,joy,sadness,valence,arousal,");
    out.push_str("cognitive_stress,endocrine_stress,");
    out.push_str("b0_conf,b0_charge,b1_conf,b1_charge,");
    out.push_str("events_this_tick,anna_events,moral_panic,revolution\n");

    let mut prev_events = 0usize;
    let mut prev_season = sim.season.current;
    let mut panics = 0u64;
    let mut revolutions = 0u64;
    let mut anna_conflicts = 0u64;
    let mut anna_died = false;

    for tick in 0..YEAR_TICKS {
        sim.tick();

        // Detect season boundary from the tracker (advances every 8760 ticks).
        let season_boundary = sim.season.current != prev_season;
        prev_season = sim.season.current;
        let _ = season_boundary;

        let new_events = sim.event_count();
        let events_this_tick = new_events.saturating_sub(prev_events);

        // Scan ONLY the events pushed this tick (delta window) — O(events) total.
        let mut anna_events = String::new();
        let mut this_panic = false;
        let mut this_rev = false;
        if events_this_tick > 0 {
            let all = sim.recent_events(new_events);
            for ev in &all[prev_events..new_events] {
                match ev {
                    SimEvent::ConflictOccurred {
                        aggressor,
                        target,
                        kind,
                        fear_induced,
                        ..
                    } => {
                        if matches!(kind, ConflictKind::MoralPanic) {
                            this_panic = true;
                        }
                        if matches!(kind, ConflictKind::Revolution) {
                            this_rev = true;
                        }
                        let hits_anna = *aggressor == AgentId::new(0) || *target == AgentId::new(0);
                        if hits_anna
                            && !matches!(kind, ConflictKind::MoralPanic | ConflictKind::Revolution)
                        {
                            anna_conflicts += 1;
                            if !anna_events.is_empty() {
                                anna_events.push('|');
                            }
                            let role = if *aggressor == AgentId::new(0) {
                                "agg"
                            } else {
                                "tgt"
                            };
                            anna_events.push_str(&format!(
                                "{role}:{:?}:{:.2}",
                                kind,
                                fear_induced.to_f64()
                            ));
                        }
                    }
                    SimEvent::AgentDied { agent, .. } => {
                        if *agent == AgentId::new(0) {
                            anna_died = true;
                            if !anna_events.is_empty() {
                                anna_events.push('|');
                            }
                            anna_events.push_str("DIED");
                        }
                    }
                    SimEvent::MarriageFormed {
                        spouse_a, spouse_b, ..
                    } if (*spouse_a == AgentId::new(0) || *spouse_b == AgentId::new(0)) => {
                        if !anna_events.is_empty() {
                            anna_events.push('|');
                        }
                        anna_events.push_str("MARRIED");
                    }
                    _ => {}
                }
            }
        }
        prev_events = new_events;
        if this_panic {
            panics += 1;
        }
        if this_rev {
            revolutions += 1;
        }

        let a = &sim.agents[0];
        let b0 = a.beliefs.iter().find(|b| b.proposition_id == 0);
        let b1 = a.beliefs.iter().find(|b| b.proposition_id == 1);
        let f = |x: Option<&Belief>| -> (String, String) {
            match x {
                Some(b) => (
                    format!("{:.4}", b.confidence.to_f64()),
                    format!("{:.4}", b.emotional_charge.to_f64()),
                ),
                None => ("-".into(), "-".into()),
            }
        };
        let (b0c, b0e) = f(b0);
        let (b1c, b1e) = f(b1);

        out.push_str(&format!(
            "{tick},{},{},{},{},{:.2},{},",
            sim.season.current.name(),
            sim.season.year,
            sim.season.tick_in_season,
            a.name,
            a.age.to_f64(),
            a.partner.is_some()
        ));
        out.push_str(&format!(
            "{:.3},{:.3},{:.3},{:.3},{:.3},{:.3},",
            a.needs.hunger.to_f64(),
            a.needs.thirst.to_f64(),
            a.needs.fatigue.to_f64(),
            a.needs.safety.to_f64(),
            a.body.health.to_f64(),
            a.wealth.coin.to_f64()
        ));
        out.push_str(&format!(
            "{:.4},{:.4},{:.4},{:.4},{:.4},{:.4},",
            a.emotions.fear.to_f64(),
            a.emotions.anger.to_f64(),
            a.emotions.joy.to_f64(),
            a.emotions.sadness.to_f64(),
            a.affect.valence.to_f64(),
            a.affect.arousal.to_f64()
        ));
        out.push_str(&format!(
            "{:.4},{:.4},",
            a.cognitive.stress.to_f64(),
            a.embodied.endocrine.stress.level.to_f64()
        ));
        out.push_str(&format!("{b0c},{b0e},{b1c},{b1e},"));
        out.push_str(&format!(
            "{events_this_tick},{},{},{}",
            if anna_events.is_empty() {
                "-"
            } else {
                &anna_events
            },
            if this_panic { "1" } else { "0" },
            if this_rev { "1" } else { "0" }
        ));
        out.push('\n');
    }

    let path = "/tmp/anna_trace_51840.csv";
    std::fs::write(path, &out).expect("write trace csv");
    println!("wrote {path} ({YEAR_TICKS} rows)");
    println!(
        "summary: anna_conflicts={anna_conflicts} moral_panics={panics} revolutions={revolutions} anna_died={anna_died} season_boundaries_in_window={}",
        (YEAR_TICKS / SEASON_TICKS) + if !YEAR_TICKS.is_multiple_of(SEASON_TICKS) { 1 } else { 0 }
    );

    // Minimal sanity: trace must have covered the full year.
    assert_eq!(sim.current_tick().as_u64(), YEAR_TICKS);
    assert!(std::path::Path::new(path).exists());
}
