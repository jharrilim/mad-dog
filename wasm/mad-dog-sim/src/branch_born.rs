//! Branch Born-weight vs overlap correlation (Phase 6 / S7).

use crate::decoherence::{
    chain_conditional_state, decoherence_final_state, env_branch_weights, DecoherenceQuenchConfig,
};
use crate::excitation_subspace::branch_overlap;
use crate::geometry_stability::spearman_corr;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchBornPoint {
    pub coupling: f64,
    pub env_p0: f64,
    pub env_p1: f64,
    pub weight_imbalance: f64,
    pub branch_overlap: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchBornResult {
    pub n: usize,
    pub field: f64,
    pub points: Vec<BranchBornPoint>,
    pub imbalance_overlap_corr: f64,
    pub entropy_overlap_corr: f64,
    pub born_consistent: bool,
}

pub fn run_branch_born_study(n: usize, field: f64, dt: f64, steps: usize, couple_step: usize) -> BranchBornResult {
    let couplings = [0.35, 0.55, 0.7, 0.85, 0.95];
    let mut points = Vec::new();
    for &coupling in &couplings {
        let config = DecoherenceQuenchConfig {
            n,
            field,
            dt,
            steps,
            couple_step,
            coupling,
            seed: 4242,
        };
        let (psi, n_chain, env_q) = decoherence_final_state(&config);
        let (p0, p1) = env_branch_weights(&psi, env_q);
        let b0 = chain_conditional_state(&psi, n_chain, env_q, 0);
        let b1 = chain_conditional_state(&psi, n_chain, env_q, 1);
        let overlap = branch_overlap(&b0, &b1);
        points.push(BranchBornPoint {
            coupling,
            env_p0: p0,
            env_p1: p1,
            weight_imbalance: (p0 - p1).abs(),
            branch_overlap: overlap,
        });
    }

    let imbalances: Vec<f64> = points.iter().map(|p| p.weight_imbalance).collect();
    let overlaps: Vec<f64> = points.iter().map(|p| p.branch_overlap).collect();
    let entropies: Vec<f64> = points
        .iter()
        .map(|p| {
            let mut s = 0.0;
            for pi in [p.env_p0, p.env_p1] {
                if pi > 1e-9 {
                    s -= pi * pi.ln();
                }
            }
            s
        })
        .collect();
    let imbalance_overlap_corr = spearman_corr(&imbalances, &overlaps);
    let entropy_overlap_corr = spearman_corr(&entropies, &overlaps);
    let born_consistent =
        entropy_overlap_corr < -0.35 || imbalance_overlap_corr.abs() > 0.55;

    BranchBornResult {
        n,
        field,
        points,
        imbalance_overlap_corr,
        entropy_overlap_corr,
        born_consistent,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn born_weights_correlate_with_branch_distinguishability() {
        let r = run_branch_born_study(8, 1.2, 0.2, 22, 7);
        assert!(
            r.born_consistent,
            "entropyCorr env={:.3} imb={:.3}",
            r.imbalance_overlap_corr,
            r.imbalance_overlap_corr
        );
    }
}
