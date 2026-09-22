//! i374 diag — reproduce the EXACT golden path (Scenario::collapse via
//! from_scenario, no param override) and print the metric hash inputs.
use mindstrata_sim::scenario::Scenario;
use mindstrata_sim::Simulation;

fn run() {
    let mut sc = Scenario::collapse();
    sc.seed = 42;
    sc.ticks = 4320;
    let mut sim = Simulation::from_scenario(sc);
    sim.populate();
    sim.run(4320);
    let ms = sim.metrics_snapshot();
    println!("hash-inputs: hunger={:.10} thirst={:.10} fatigue={:.10} valence={:.10} joy={:.10} fear={:.10}",
        ms.avg_hunger, ms.avg_thirst, ms.avg_fatigue, ms.avg_valence, ms.avg_joy, ms.avg_fear);
    println!(
        "  grain={:.6} water={:.6} events={} journal={} agents={}",
        ms.total_grain, ms.total_water, ms.event_count, ms.journal_len, ms.agent_count
    );
}

fn main() {
    run();
}
