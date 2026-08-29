//! Probe: differentiation matrix — founder line profiles yield ≥2 behavioral clusters (SIM 5.17).
//!
//! Founder trait draws are U(0,1) at N=12 (H5 budget); this probe shows they
//! produce at least two distinct village trajectories at 2K ticks across the
//! canonical 12-seed family (CA-5). Vectors are village-level behavioral
//! metrics; clustering is 2-means on the 5-D space [avg_stress, gini,
//! polarization_index, fear_p90, avg_best_skill] (all in [0,1] except gini).
//! The floor is inter-centroid Euclidean distance >0.05 and both clusters
//! non-empty — the UM-1 criterion 2 evidence (AP4 04-cycle-plan SIM 14).
//!
//! Run: cargo run --release -p mindstrata-benches --example i272_differentiation_matrix

use mindstrata_sim::sim::{MetricsSnapshot, SimConfig, Simulation};

const FAMILY: [u64; 12] = [1, 2, 7, 13, 21, 42, 46, 55, 77, 99, 123, 12345];
const HORIZON: u64 = 2000;

fn vectorize(m: &MetricsSnapshot) -> [f64; 5] {
    [
        m.avg_stress,
        m.gini,
        m.polarization_index,
        m.fear_p90,
        m.avg_best_skill,
    ]
}

fn euclid(a: &[f64; 5], b: &[f64; 5]) -> f64 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| (x - y) * (x - y))
        .sum::<f64>()
        .sqrt()
}

fn main() {
    let mut snapshots: Vec<(u64, MetricsSnapshot)> = Vec::new();
    for seed in FAMILY {
        let config = SimConfig {
            seed,
            max_ticks: HORIZON,
            num_agents: 12,
            world_width: 16,
            world_height: 16,
            snapshot_interval: None,
        };
        let mut sim = Simulation::new(config);
        sim.populate();
        sim.run(HORIZON);
        let snap = sim.metrics_snapshot();
        println!(
            "seed={:5} trait_var={:.4} stress={:.3} gini={:.3} polar={:.3} fear_p90={:.3} skill={:.3} kinship={:.3}",
            seed, snap.trait_variance, snap.avg_stress, snap.gini, snap.polarization_index, snap.fear_p90, snap.avg_best_skill, snap.mean_kinship
        );
        snapshots.push((seed, snap));
    }

    let vectors: Vec<[f64; 5]> = snapshots.iter().map(|(_, m)| vectorize(m)).collect();

    // 2-means: init centroids as min-vector and max-vector per dimension.
    let mut c0 = [f64::INFINITY; 5];
    let mut c1 = [f64::NEG_INFINITY; 5];
    for v in &vectors {
        for i in 0..5 {
            c0[i] = c0[i].min(v[i]);
            c1[i] = c1[i].max(v[i]);
        }
    }
    let mut assignments = vec![0usize; vectors.len()];
    for _iter in 0..20 {
        // Assign
        for (idx, v) in vectors.iter().enumerate() {
            let d0 = euclid(v, &c0);
            let d1 = euclid(v, &c1);
            assignments[idx] = if d0 <= d1 { 0 } else { 1 };
        }
        // Recompute
        let mut sum0 = [0.0; 5];
        let mut sum1 = [0.0; 5];
        let mut cnt0 = 0usize;
        let mut cnt1 = 0usize;
        for (v, &a) in vectors.iter().zip(&assignments) {
            if a == 0 {
                for i in 0..5 {
                    sum0[i] += v[i];
                }
                cnt0 += 1;
            } else {
                for i in 0..5 {
                    sum1[i] += v[i];
                }
                cnt1 += 1;
            }
        }
        // Handle empty cluster: keep old centroid.
        if cnt0 > 0 {
            for i in 0..5 {
                c0[i] = sum0[i] / cnt0 as f64;
            }
        }
        if cnt1 > 0 {
            for i in 0..5 {
                c1[i] = sum1[i] / cnt1 as f64;
            }
        }
        // Early break if stable? (not needed for 20 iters, negligible cost)
    }

    let cnt0 = assignments.iter().filter(|&&a| a == 0).count();
    let cnt1 = assignments.len() - cnt0;
    let inter = euclid(&c0, &c1);
    println!(
        "clusters: c0_size={} c1_size={} inter_centroid={:.4}",
        cnt0, cnt1, inter
    );
    println!("centroid c0: {:?}", c0);
    println!("centroid c1: {:?}", c1);
    for (idx, (seed, _)) in snapshots.iter().enumerate() {
        println!(" seed {} -> cluster {}", seed, assignments[idx]);
    }

    // UM-1 criterion 2 floor: both clusters populated and separation >0.05.
    let floor = 0.05;
    let pass = cnt0 > 0 && cnt1 > 0 && inter > floor;
    println!(
        "differentiation {} (floor {:.2}, inter {:.4}, sizes {}/{})",
        if pass { "PASS" } else { "FAIL" },
        floor,
        inter,
        cnt0,
        cnt1
    );

    // Also report trait variance spread as the founder-budget health check.
    let trait_vars: Vec<f64> = snapshots.iter().map(|(_, m)| m.trait_variance).collect();
    let tv_min = trait_vars.iter().cloned().fold(f64::INFINITY, f64::min);
    let tv_max = trait_vars.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    let tv_mean = trait_vars.iter().sum::<f64>() / trait_vars.len() as f64;
    println!(
        "trait_variance spread: min={:.4} max={:.4} mean={:.4} range={:.4}",
        tv_min,
        tv_max,
        tv_mean,
        tv_max - tv_min
    );
}
