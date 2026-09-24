//! Iteration 242 — natural marriage-gate diagnosis.
//!
//! Runs an untouched world (default params) and samples, every 100 days:
//! unpartnered adults, eligible pairs (age-compatible, neither partnered),
//! their max trust / affection / mean-health / attraction-proxy, how many
//! eligible pairs are cross-clan-enemy (clan_factor 0), and cumulative
//! marriages. Locates the starving factor in the
//! chance = attraction x health x trust x rate x clan_factor product.

use mindstrata_core::fixed::Fixed;
use mindstrata_sim::sim::SimConfig;
use mindstrata_sim::Simulation;

fn main() {
    let seed: u64 = std::env::var("I242_SEED")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(51);
    let horizon: u64 = std::env::var("I242_HORIZON")
        .ok()
        .and_then(|raw| raw.parse().ok())
        .unwrap_or(20000);
    let config = SimConfig {
        seed,
        max_ticks: horizon,
        world_width: 16,
        world_height: 16,
        num_agents: 12,
        snapshot_interval: None,
    };
    let mut sim = Simulation::new(config);
    sim.populate();
    if std::env::var("I242_KIN").is_ok() {
        sim.demography_config.birth_rate = Fixed::from_f64(12.0);
    }

    let mut tick = 0u64;
    while tick < horizon {
        sim.run(144);
        tick += 144;
        let coarse = std::env::var("I242_COARSE").is_ok();
        let period = if coarse {
            20000
        } else if std::env::var("I242_KIN").is_ok() {
            250
        } else {
            720
        };
        if !tick.is_multiple_of(period) && tick != horizon {
            continue;
        }
        let agent_count = sim.agents.len();
        let mut eligible = 0usize;
        let mut best_trust = 0.0f64;
        let mut best_affection = 0.0f64;
        let mut best_health = 0.0f64;
        let mut best_compat = 0.0f64;
        let married_count = sim
            .agents
            .iter()
            .filter(|agent| agent.partner.is_some())
            .count()
            / 2;
        for first in 0..agent_count {
            if sim.agents[first].partner.is_some() || sim.agents[first].age < Fixed::from_f64(18.0)
            {
                continue;
            }
            for second in (first + 1)..agent_count {
                if sim.agents[second].partner.is_some()
                    || sim.agents[second].age < Fixed::from_f64(18.0)
                {
                    continue;
                }
                let age_diff = (sim.agents[first].age - sim.agents[second].age)
                    .abs()
                    .to_f64();
                if age_diff > 15.0 {
                    continue;
                }
                eligible += 1;
                let rel = sim.relationships.iter().find(|bond| {
                    bond.from.as_u64() == first as u64 && bond.to.as_u64() == second as u64
                });
                let trust = rel.map_or(0.0, |bond| bond.trust.to_f64());
                let aff = rel.map_or(0.0, |bond| bond.affection.to_f64());
                let mid_health = f64::midpoint(
                    sim.agents[first].body.health.to_f64(),
                    sim.agents[second].body.health.to_f64(),
                );
                let compat = 1.0
                    - (sim.agents[first].personality.agreeableness.to_f64()
                        - sim.agents[second].personality.agreeableness.to_f64())
                    .abs();
                if trust > best_trust {
                    best_trust = trust;
                }
                if aff > best_affection {
                    best_affection = aff;
                }
                if trust > 0.02 && mid_health > best_health {
                    best_health = mid_health;
                    best_compat = compat;
                }
            }
        }
        let births = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|evt| matches!(evt, mindstrata_core::event::SimEvent::ChildBorn { .. }))
            .count();
        let preg = 0usize;
        println!(
            "t={tick:>6}: married={married_count} births={births} concept={preg} eligible={eligible} \
             best(trust={best_trust:.3} aff={best_affection:.3} h={best_health:.2} compat={best_compat:.2})",
        );
        // Worst-health agent dump.
        if let Some(worst_idx) = sim
            .agents
            .iter()
            .enumerate()
            .min_by_key(|(_, agent)| agent.body.health)
            .map(|(idx, _)| idx)
        {
            let ag = &sim.agents[worst_idx];
            let embodied = &ag.embodied;
            println!(
                "    worst[{}]: body.h={:.4} base={:.4} inj={:.3} sick={:.3} stress={:.3} \
pain={:.3} shock={:.3} skel_i={:.3} thirst={:.3} hunger={:.3} fatigue={:.3} tier={:?}",
                worst_idx,
                ag.body.health.to_f64(),
                embodied.health.to_f64(),
                embodied.injury.to_f64(),
                embodied.immune.sickness_level().to_f64(),
                embodied.endocrine.stress.level.to_f64(),
                embodied.nervous.pain.effective_pain().to_f64(),
                embodied.cardiovascular.shock_risk.to_f64(),
                embodied.skeletal.structural_integrity.to_f64(),
                ag.needs.thirst.to_f64(),
                ag.needs.hunger.to_f64(),
                ag.needs.fatigue.to_f64(),
                ag.agent_tier.tier,
            );
            println!(
                "      derived_now={:.4} energy_b={:.3} e_f={:.3}",
                embodied.derived_health().to_f64(),
                ag.body.energy.to_f64(),
                embodied.energy.to_f64()
            );
        }

        // First couple's conception-factor readout.
        // First couple's conception-factor readout.
        if let Some(first_idx) = sim.agents.iter().position(|agent| agent.partner.is_some()) {
            let mate_idx = sim.agents[first_idx].partner.unwrap();
            let fert = sim.agents[first_idx]
                .embodied
                .reproductive
                .fertility
                .min(sim.agents[mate_idx].embodied.reproductive.fertility)
                .to_f64();
            let libido = (sim.agents[first_idx].embodied.reproductive.libido
                + sim.agents[mate_idx].embodied.reproductive.libido)
                .to_f64()
                * 0.5;
            let _nut = sim.agents[first_idx]
                .embodied
                .digestive
                .gut_health
                .min(sim.agents[mate_idx].embodied.digestive.gut_health)
                .to_f64();
            let pd = (sim.agents[first_idx].embodied.reproductive.parental_drive
                + sim.agents[mate_idx].embodied.reproductive.parental_drive)
                .to_f64()
                * 0.5;
            let hmin = sim.agents[first_idx]
                .body
                .health
                .min(sim.agents[mate_idx].body.health)
                .to_f64();
            let age_y = sim.agents[first_idx]
                .age
                .to_f64()
                .min(sim.agents[mate_idx].age.to_f64());
            let v2pos = {
                let (low, high) = if first_idx < mate_idx {
                    (first_idx, mate_idx)
                } else {
                    (mate_idx, first_idx)
                };
                // targets ordered 0..n excluding self; index of high in low's list
                if high < low {
                    high
                } else {
                    high - 1
                }
            };
            let _q = sim.agents[first_idx]
                .relationship_v2s
                .get(v2pos)
                .map(|bond| bond.quality().to_f64());
            let preg0 = format!(
                "{:?}",
                sim.agents[first_idx]
                    .embodied
                    .reproductive
                    .pregnancy
                    .is_some()
            );
            let preg1 = format!(
                "{:?}",
                sim.agents[mate_idx]
                    .embodied
                    .reproductive
                    .pregnancy
                    .is_some()
            );
            let sex0 = format!("{:?}", sim.agents[first_idx].embodied.reproductive.sex);
            let sex1 = format!("{:?}", sim.agents[mate_idx].embodied.reproductive.sex);
            let gavg = f64::midpoint(
                sim.agents[first_idx].embodied.digestive.gut_health.to_f64(),
                sim.agents[mate_idx].embodied.digestive.gut_health.to_f64(),
            );
            let gp = sim
                .agents
                .iter()
                .filter_map(|mother| mother.embodied.reproductive.pregnancy.as_ref())
                .map(|pregnancy| pregnancy.gestation_progress.to_f64())
                .collect::<Vec<_>>();
            println!(
                "    couple({first_idx},{mate_idx}): fert={fert:.3} libido={libido:.3} gutAVG={gavg:.3} pdrive={pd:.3} \
                 hmin={hmin:.3} ageY={age_y:.0} sex={sex0}/{sex1} preg={preg0}/{preg1} \
                 gest={gp:?}"
            );
        }
    }
}
// probe variant: seed 51 with elevated birth rate, print conception-relevant
// state every 500 ticks (run with I242_KIN=1)
