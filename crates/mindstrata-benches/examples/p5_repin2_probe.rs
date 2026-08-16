//! P5 re-pin sweep — quantify honest anchors for the 4 remaining failures:
//! 1. neural_like belief-fold differential (scarcity > abundant + 0.02)
//! 2. taboo_shame directionals (seeded violence < stripped AND stripped shame > seeded)
//! 3. tenderness help multiplier (rising-segment lift at seed 42)
//! 4. revolution peak council absorption (>= 5 after coup)
use mindstrata_sim::sim::{SimConfig, Simulation};

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let which = args.get(1).map(|s| s.as_str()).unwrap_or("all");

    if which == "neural" || which == "all" {
        neural_sweep();
    }
    if which == "taboo" || which == "all" {
        taboo_sweep();
    }
    if which == "tenderness" || which == "all" {
        tenderness_sweep();
    }
    if which == "revolution" || which == "all" {
        revolution_sweep();
    }
}

fn neural_sweep() {
    println!("=== NEURAL FOLD: scarcity vs abundant belief confidence @5000 ===");
    for seed in [42u64, 1, 7, 13, 46, 55, 99, 3, 44, 2] {
        let mut abundant = Simulation::new(SimConfig {
            seed,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        abundant.populate();
        for site in &mut abundant.world.sites {
            for stock in &mut site.inventory {
                if stock.resource_id == 1 {
                    stock.quantity = mindstrata_core::fixed::Fixed::from_f64(500.0);
                }
            }
        }
        abundant.run(5000);
        let abundant_conf: f64 = abundant
            .agents
            .iter()
            .filter(|a| !a.beliefs.is_empty())
            .map(|a| a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>() / a.beliefs.len() as f64)
            .sum::<f64>()
            / abundant.agents.iter().filter(|a| !a.beliefs.is_empty()).count().max(1) as f64;

        let mut scarcity = Simulation::new(SimConfig {
            seed,
            max_ticks: 5000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        scarcity.populate();
        for site in &mut scarcity.world.sites {
            for stock in &mut site.inventory {
                if stock.resource_id == 1 {
                    stock.quantity = mindstrata_core::fixed::Fixed::ZERO;
                }
            }
        }
        scarcity.run(5000);
        let scarcity_conf: f64 = scarcity
            .agents
            .iter()
            .filter(|a| !a.beliefs.is_empty())
            .map(|a| a.beliefs.iter().map(|b| b.confidence.to_f64()).sum::<f64>() / a.beliefs.len() as f64)
            .sum::<f64>()
            / scarcity.agents.iter().filter(|a| !a.beliefs.is_empty()).count().max(1) as f64;

        let delta = scarcity_conf - abundant_conf;
        let marker = if delta > 0.02 { "HOLDS" } else { "fails" };
        println!("seed {seed:>2}: abundant {abundant_conf:.3} scarcity {scarcity_conf:.3} delta {delta:+.3} {marker}");
    }
}

fn taboo_sweep() {
    println!("=== TABOO-SHAME: seeded vs stripped @2000 ===");
    use mindstrata_core::conflict::ConflictKind;
    use mindstrata_core::event::SimEvent;
    use mindstrata_core::fixed::Fixed;
    use mindstrata_sim::Simulation;

    let run_world = |seed: u64, strip_taboos: bool| -> (f64, usize) {
        let config = SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        if strip_taboos {
            for a in &mut sim.agents {
                a.cultural_cognition.taboos.clear();
            }
        }
        sim.run(2000);
        let mut shame_sum = Fixed::ZERO;
        for a in &sim.agents {
            shame_sum += a.emotions.shame;
        }
        let violence = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::ConflictOccurred {
                        kind: ConflictKind::Violence,
                        ..
                    }
                )
            })
            .count();
        (shame_sum.to_f64(), violence)
    };

    for seed in [7u64, 42, 99, 55, 2, 50, 13, 1, 3, 44, 46, 5] {
        let (seeded_shame, seeded_violence) = run_world(seed, false);
        let (stripped_shame, stripped_violence) = run_world(seed, true);
        let v_ok = seeded_violence < stripped_violence;
        let s_ok = stripped_shame > seeded_shame;
        let marker = if v_ok && s_ok { "HOLDS-BOTH" } else if v_ok { "v-only" } else if s_ok { "s-only" } else { "neither" };
        println!(
            "seed {seed:>2}: seeded {seeded_violence:>3} acts / shame {seeded_shame:.3} | stripped {stripped_violence:>3} acts / shame {stripped_shame:.3} {marker}"
        );
    }
}

fn tenderness_sweep() {
    println!("=== TENDERNESS: help count across multipliers @2000 seed 42 ===");
    use mindstrata_core::event::SimEvent;
    use mindstrata_core::fixed::Fixed;
    use mindstrata_sim::Simulation;

    let count_help = |seed: u64, mult: f64| -> u64 {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 2000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.populate();
        sim.params.social_tenderness_help_multiplier = Fixed::from_f64(mult);
        sim.run(2000);
        sim.recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::InteractionOccurred {
                        kind: mindstrata_core::event::InteractionKind::Help,
                        ..
                    }
                )
            })
            .count() as u64
    };

    // Multiplier curve at seed 42
    for mult in [0.0, 0.1, 0.2, 0.3, 0.5, 0.8, 1.0] {
        println!("seed 42 mult {mult:.1} -> {} help", count_help(42, mult));
    }
    // Try other seeds at the 0.2 vs 0.5 rising-segment anchors
    for seed in [1u64, 7, 13, 46, 55, 99, 3, 44, 2, 5] {
        let cold = count_help(seed, 0.2);
        let warm = count_help(seed, 0.5);
        let marker = if warm > cold * 11 / 10 { "HOLDS" } else { "fails" };
        println!("seed {seed:>2}: cold(0.2) {cold} warm(0.5) {warm} lift {} {marker}", warm as i64 - cold as i64);
    }
}

fn revolution_sweep() {
    println!("=== REVOLUTION: peak council membership + coup count @70K seed 42 ===");
    use mindstrata_core::conflict::ConflictKind;
    use mindstrata_core::event::SimEvent;
    use mindstrata_sim::institutions::InstitutionKind;
    use mindstrata_sim::Simulation;

    let mut sim = Simulation::new(SimConfig {
        seed: 42,
        max_ticks: 70000,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    });
    sim.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
    sim.populate();
    let mut peak_council = 0usize;
    let mut peak_at = 0u64;
    for chunk in 0..140 {
        sim.run(500);
        let council = sim
            .institutions
            .iter()
            .find(|i| i.kind == InstitutionKind::Council)
            .expect("council should exist");
        if council.members.len() > peak_council {
            peak_council = council.members.len();
            peak_at = chunk * 500;
        }
    }
    let rev_count = sim
        .recent_events(10_000_000)
        .iter()
        .filter(|e| {
            matches!(
                e,
                SimEvent::ConflictOccurred {
                    kind: ConflictKind::Revolution,
                    ..
                }
            )
        })
        .count();
    println!("seed 42: peak council {peak_council} @tick {peak_at}, revolutions {rev_count}");
    // Also sample a few other seeds briefly to see if any absorb >= 5
    for seed in [1u64, 7, 13, 46, 55, 99] {
        let mut sim = Simulation::new(SimConfig {
            seed,
            max_ticks: 70000,
            world_width: 16,
            world_height: 16,
            num_agents: 12,
            snapshot_interval: None,
        });
        sim.params.meme_mutation_rate_base = mindstrata_core::fixed::Fixed::ZERO;
        sim.populate();
        let mut peak_council = 0usize;
        for _ in 0..140 {
            sim.run(500);
            let council = sim
                .institutions
                .iter()
                .find(|i| i.kind == InstitutionKind::Council)
                .expect("council should exist");
            peak_council = peak_council.max(council.members.len());
        }
        let rev_count = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| {
                matches!(
                    e,
                    SimEvent::ConflictOccurred {
                        kind: ConflictKind::Revolution,
                        ..
                    }
                )
            })
            .count();
        println!("seed {seed:>2}: peak council {peak_council}, revolutions {rev_count}");
    }
}
