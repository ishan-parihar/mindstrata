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
        .and_then(|s| s.parse().ok())
        .unwrap_or(51);
    let horizon: u64 = std::env::var("I242_HORIZON")
        .ok()
        .and_then(|s| s.parse().ok())
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

    let mut t = 0u64;
    while t < horizon {
        sim.run(144);
        t += 144;
        let coarse = std::env::var("I242_COARSE").is_ok();
        let period = if coarse {
            20000
        } else if std::env::var("I242_KIN").is_ok() {
            250
        } else {
            720
        };
        if t % period != 0 && t != horizon {
            continue;
        }
        let n = sim.agents.len();
        let mut eligible = 0usize;
        let mut enemy_pairs = 0usize;
        let mut best_trust = 0.0f64;
        let mut best_affection = 0.0f64;
        let mut best_health = 0.0f64;
        let mut best_compat = 0.0f64;
        let married_count = sim.agents.iter().filter(|a| a.partner.is_some()).count() / 2;
        for i in 0..n {
            if sim.agents[i].partner.is_some() || sim.agents[i].age < Fixed::from_f64(18.0) {
                continue;
            }
            for j in (i + 1)..n {
                if sim.agents[j].partner.is_some() || sim.agents[j].age < Fixed::from_f64(18.0) {
                    continue;
                }
                let age_diff = (sim.agents[i].age - sim.agents[j].age).abs().to_f64();
                if age_diff > 15.0 {
                    continue;
                }
                eligible += 1;
                let rel = sim
                    .relationships
                    .iter()
                    .find(|r| r.from.as_u64() == i as u64 && r.to.as_u64() == j as u64);
                let trust = rel.map_or(0.0, |r| r.trust.to_f64());
                let aff = rel.map_or(0.0, |r| r.affection.to_f64());
                let h =
                    (sim.agents[i].body.health.to_f64() + sim.agents[j].body.health.to_f64()) * 0.5;
                let compat = 1.0
                    - (sim.agents[i].personality.agreeableness.to_f64()
                        - sim.agents[j].personality.agreeableness.to_f64())
                    .abs();
                if trust > best_trust {
                    best_trust = trust;
                }
                if aff > best_affection {
                    best_affection = aff;
                }
                if trust > 0.02 && h > best_health {
                    best_health = h;
                    best_compat = compat;
                }
            }
        }
        let births = sim
            .recent_events(10_000_000)
            .iter()
            .filter(|e| matches!(e, mindstrata_core::event::SimEvent::ChildBorn { .. }))
            .count();
        let preg = 0usize;
        println!(
            "t={t:>6}: married={married_count} births={births} concept={preg} eligible={eligible} \
             best(trust={best_trust:.3} aff={best_affection:.3} h={best_health:.2} compat={best_compat:.2})",
        );
        // Worst-health agent dump.
        if let Some(w) = sim
            .agents
            .iter()
            .enumerate()
            .min_by_key(|(_, a)| a.body.health)
            .map(|(i, _)| i)
        {
            let ag = &sim.agents[w];
            let e = &ag.embodied;
            println!(
                "    worst[{}]: body.h={:.4} base={:.4} inj={:.3} sick={:.3} stress={:.3} \
pain={:.3} shock={:.3} skel_i={:.3} thirst={:.3} hunger={:.3} fatigue={:.3} tier={:?}",
                w,
                ag.body.health.to_f64(),
                e.health.to_f64(),
                e.injury.to_f64(),
                e.immune.sickness_level().to_f64(),
                e.endocrine.stress.level.to_f64(),
                e.nervous.pain.effective_pain().to_f64(),
                e.cardiovascular.shock_risk.to_f64(),
                e.skeletal.structural_integrity.to_f64(),
                ag.needs.thirst.to_f64(),
                ag.needs.hunger.to_f64(),
                ag.needs.fatigue.to_f64(),
                ag.agent_tier.tier,
            );
            println!(
                "      derived_now={:.4} energy_b={:.3} e_f={:.3}",
                e.derived_health().to_f64(),
                ag.body.energy.to_f64(),
                e.energy.to_f64()
            );
        }

        // First couple's conception-factor readout.
        // First couple's conception-factor readout.
        if let Some(i) = sim.agents.iter().position(|a| a.partner.is_some()) {
            let p = sim.agents[i].partner.unwrap();
            let f = sim.agents[i]
                .embodied
                .reproductive
                .fertility
                .min(sim.agents[p].embodied.reproductive.fertility)
                .to_f64();
            let l = (sim.agents[i].embodied.reproductive.libido
                + sim.agents[p].embodied.reproductive.libido)
                .to_f64()
                * 0.5;
            let nut = sim.agents[i]
                .embodied
                .digestive
                .gut_health
                .min(sim.agents[p].embodied.digestive.gut_health)
                .to_f64();
            let pd = (sim.agents[i].embodied.reproductive.parental_drive
                + sim.agents[p].embodied.reproductive.parental_drive)
                .to_f64()
                * 0.5;
            let hmin = sim.agents[i]
                .body
                .health
                .min(sim.agents[p].body.health)
                .to_f64();
            let age_y = sim.agents[i].age.to_f64().min(sim.agents[p].age.to_f64());
            let v2pos = {
                let mut pos = None;
                let (a, b) = if i < p { (i, p) } else { (p, i) };
                // targets ordered 0..n excluding self; index of b in a's list
                pos = Some(if b < a { b } else { b - 1 });
                pos.unwrap()
            };
            let q = sim.agents[i]
                .relationship_v2s
                .get(v2pos)
                .map(|r| r.quality().to_f64());
            let preg0 = format!(
                "{:?}",
                sim.agents[i].embodied.reproductive.pregnancy.is_some()
            );
            let preg1 = format!(
                "{:?}",
                sim.agents[p].embodied.reproductive.pregnancy.is_some()
            );
            let sex0 = format!("{:?}", sim.agents[i].embodied.reproductive.sex);
            let sex1 = format!("{:?}", sim.agents[p].embodied.reproductive.sex);
            let gavg = (sim.agents[i].embodied.digestive.gut_health.to_f64()
                + sim.agents[p].embodied.digestive.gut_health.to_f64())
                * 0.5;
            let gp = sim
                .agents
                .iter()
                .filter_map(|a| a.embodied.reproductive.pregnancy.as_ref())
                .map(|p| p.gestation_progress.to_f64())
                .collect::<Vec<_>>();
            println!(
                "    couple({i},{p}): fert={f:.3} libido={l:.3} gutAVG={gavg:.3} pdrive={pd:.3} \
                 hmin={hmin:.3} ageY={age_y:.0} sex={sex0}/{sex1} preg={preg0}/{preg1} \
                 gest={gp:?}"
            );
        }
    }
}
// probe variant: seed 51 with elevated birth rate, print conception-relevant
// state every 500 ticks (run with I242_KIN=1)
