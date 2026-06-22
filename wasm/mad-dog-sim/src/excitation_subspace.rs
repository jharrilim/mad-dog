//! Branch-resolved excitation subspace probe — QECC / EFT diagnostic.

use crate::decoherence::{
    chain_conditional_state, decoherence_final_state, env_branch_weights, DecoherenceQuenchConfig,
};
use crate::geometry::region_eigenvalues;
use crate::linalg::hermitian_eigenvalues;
use crate::quantum::{expectation_x, expectation_z, QuantumState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcitationSubspaceConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default = "default_couple_step")]
    pub couple_step: usize,
    #[serde(default = "default_coupling")]
    pub coupling: f64,
    #[serde(default = "default_seed")]
    pub seed: u32,
    #[serde(default = "default_window_radius")]
    pub window_radius: usize,
}

fn default_couple_step() -> usize {
    8
}

fn default_coupling() -> f64 {
    0.85
}

fn default_seed() -> u32 {
    4242
}

fn default_window_radius() -> usize {
    2
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PauliSiteDiagnostic {
    pub site: usize,
    pub mixed_z: f64,
    pub branch0_z: f64,
    pub branch1_z: f64,
    pub mixed_x: f64,
    pub branch0_x: f64,
    pub branch1_x: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExcitationSubspaceResult {
    pub chain_sites: usize,
    pub excitation_peak: usize,
    pub window_sites: Vec<usize>,
    pub env_p0: f64,
    pub env_p1: f64,
    pub mixed_sharpness: f64,
    pub branch0_sharpness: f64,
    pub branch1_sharpness: f64,
    pub sharpness_gain: f64,
    pub mixed_effective_rank: f64,
    pub branch0_effective_rank: f64,
    pub branch1_effective_rank: f64,
    pub rank_reduction: f64,
    pub branch_overlap: f64,
    pub site_diagnostics: Vec<PauliSiteDiagnostic>,
    pub code_like: bool,
    pub elapsed_ms: f64,
}

fn pauli_sharpness(state: &QuantumState, sites: &[usize]) -> f64 {
    if sites.is_empty() {
        return 0.0;
    }
    let mut sum = 0.0;
    for &q in sites {
        sum += expectation_z(state, q).abs();
        sum += expectation_x(state, q).abs();
    }
    sum / (2.0 * sites.len() as f64)
}

pub(crate) fn mixed_expectation_z(psi: &QuantumState, n_chain: usize, env_q: usize, q: usize) -> f64 {
    let (p0, p1) = env_branch_weights(psi, env_q);
    let c0 = chain_conditional_state(psi, n_chain, env_q, 0);
    let c1 = chain_conditional_state(psi, n_chain, env_q, 1);
    p0 * expectation_z(&c0, q) + p1 * expectation_z(&c1, q)
}

pub(crate) fn mixed_expectation_x(psi: &QuantumState, n_chain: usize, env_q: usize, q: usize) -> f64 {
    let (p0, p1) = env_branch_weights(psi, env_q);
    let c0 = chain_conditional_state(psi, n_chain, env_q, 0);
    let c1 = chain_conditional_state(psi, n_chain, env_q, 1);
    p0 * expectation_x(&c0, q) + p1 * expectation_x(&c1, q)
}

fn effective_rank(eigenvalues: &[f64]) -> f64 {
    let mut probs: Vec<f64> = eigenvalues.iter().copied().filter(|&v| v > 1e-12).collect();
    if probs.is_empty() {
        return 1.0;
    }
    let sum: f64 = probs.iter().sum();
    for p in &mut probs {
        *p /= sum;
    }
    let entropy: f64 = probs
        .iter()
        .filter(|&&p| p > 1e-12)
        .map(|p| -p * p.ln())
        .sum();
    entropy.exp()
}

fn chain_marginal_eigenvalues(psi: &QuantumState, n_chain: usize, env_q: usize) -> Vec<f64> {
    let dim = 1 << n_chain;
    let mut re = vec![vec![0.0; dim]; dim];
    let mut im = vec![vec![0.0; dim]; dim];
    for i in 0..dim {
        for j in 0..dim {
            for env in 0..2 {
                let si = i | (env << env_q);
                let sj = j | (env << env_q);
                let ar = psi.data[2 * si];
                let ai = psi.data[2 * si + 1];
                let br = psi.data[2 * sj];
                let bi = psi.data[2 * sj + 1];
                re[i][j] += ar * br + ai * bi;
                im[i][j] += ai * br - ar * bi;
            }
        }
    }
    hermitian_eigenvalues(&re, &im)
}

fn window_marginal_eigenvalues(
    psi: &QuantumState,
    n_chain: usize,
    env_q: usize,
    window: &[usize],
) -> Vec<f64> {
    if window.len() == n_chain {
        return chain_marginal_eigenvalues(psi, n_chain, env_q);
    }
    let mut in_window = vec![false; n_chain];
    for &q in window {
        in_window[q] = true;
    }
    let mut keep = Vec::new();
    let mut trace = Vec::new();
    for q in 0..n_chain {
        if in_window[q] {
            keep.push(q);
        } else {
            trace.push(q);
        }
    }
    let dim_keep = 1 << keep.len();
    let dim_trace = 1 << trace.len();
    let chain_dim = 1 << n_chain;
    let mut rho_re = vec![vec![0.0; chain_dim]; chain_dim];
    let mut rho_im = vec![vec![0.0; chain_dim]; chain_dim];
    for i in 0..chain_dim {
        for j in 0..chain_dim {
            for env in 0..2 {
                let si = i | (env << env_q);
                let sj = j | (env << env_q);
                let ar = psi.data[2 * si];
                let ai = psi.data[2 * si + 1];
                let br = psi.data[2 * sj];
                let bi = psi.data[2 * sj + 1];
                rho_re[i][j] += ar * br + ai * bi;
                rho_im[i][j] += ai * br - ar * bi;
            }
        }
    }
    let mut re = vec![vec![0.0; dim_keep]; dim_keep];
    let mut im = vec![vec![0.0; dim_keep]; dim_keep];
    for ia in 0..dim_keep {
        for ja in 0..dim_keep {
            for ie in 0..dim_trace {
                let mut i_full = 0usize;
                for (idx, &q) in keep.iter().enumerate() {
                    i_full |= ((ia >> idx) & 1) << q;
                }
                for (idx, &q) in trace.iter().enumerate() {
                    i_full |= ((ie >> idx) & 1) << q;
                }
                let mut j_full = 0usize;
                for (idx, &q) in keep.iter().enumerate() {
                    j_full |= ((ja >> idx) & 1) << q;
                }
                for (idx, &q) in trace.iter().enumerate() {
                    j_full |= ((ie >> idx) & 1) << q;
                }
                re[ia][ja] += rho_re[i_full][j_full];
                im[ia][ja] += rho_im[i_full][j_full];
            }
        }
    }
    hermitian_eigenvalues(&re, &im)
}

pub(crate) fn branch_overlap(c0: &QuantumState, c1: &QuantumState) -> f64 {
    let mut re = 0.0;
    let mut im = 0.0;
    for i in 0..c0.dim {
        re += c0.data[2 * i] * c1.data[2 * i] + c0.data[2 * i + 1] * c1.data[2 * i + 1];
        im += c0.data[2 * i + 1] * c1.data[2 * i] - c0.data[2 * i] * c1.data[2 * i + 1];
    }
    re * re + im * im
}

fn excitation_peak_from_mixed(psi: &QuantumState, n_chain: usize, env_q: usize) -> usize {
    let mut best = 0usize;
    let mut best_val = -1.0_f64;
    for q in 0..n_chain {
        let v = mixed_expectation_z(psi, n_chain, env_q, q).abs();
        if v > best_val {
            best_val = v;
            best = q;
        }
    }
    best
}

fn window_sites(peak: usize, radius: usize, n_chain: usize) -> Vec<usize> {
    let lo = peak.saturating_sub(radius);
    let hi = (peak + radius).min(n_chain - 1);
    (lo..=hi).collect()
}

pub fn run_excitation_subspace_probe(config: &ExcitationSubspaceConfig) -> ExcitationSubspaceResult {
    let deco_config = DecoherenceQuenchConfig {
        n: config.n,
        field: config.field,
        dt: config.dt,
        steps: config.steps,
        couple_step: config.couple_step,
        coupling: config.coupling,
        seed: config.seed,
    };
    let (psi, n_chain, env_q) = decoherence_final_state(&deco_config);
    let (p0, p1) = env_branch_weights(&psi, env_q);

    let branch0 = chain_conditional_state(&psi, n_chain, env_q, 0);
    let branch1 = chain_conditional_state(&psi, n_chain, env_q, 1);

    let peak = excitation_peak_from_mixed(&psi, n_chain, env_q);
    let window = window_sites(peak, config.window_radius, n_chain);

    let mixed_sharpness = {
        let mut sum = 0.0;
        for &q in &window {
            sum += mixed_expectation_z(&psi, n_chain, env_q, q).abs();
            sum += mixed_expectation_x(&psi, n_chain, env_q, q).abs();
        }
        sum / (2.0 * window.len().max(1) as f64)
    };
    let branch0_sharpness = pauli_sharpness(&branch0, &window);
    let branch1_sharpness = pauli_sharpness(&branch1, &window);
    let branch_avg = (branch0_sharpness + branch1_sharpness) / 2.0;
    let sharpness_gain = if mixed_sharpness > 1e-9 {
        branch_avg / mixed_sharpness
    } else {
        1.0
    };

    let mixed_effective_rank =
        effective_rank(&window_marginal_eigenvalues(&psi, n_chain, env_q, &window));
    let branch0_effective_rank =
        effective_rank(&region_eigenvalues(&branch0, &window));
    let branch1_effective_rank =
        effective_rank(&region_eigenvalues(&branch1, &window));
    let branch_rank_avg = (branch0_effective_rank + branch1_effective_rank) / 2.0;
    let rank_reduction = if mixed_effective_rank > 1e-9 {
        branch_rank_avg / mixed_effective_rank
    } else {
        1.0
    };

    let overlap = branch_overlap(&branch0, &branch1);

    let site_diagnostics: Vec<PauliSiteDiagnostic> = (0..n_chain)
        .map(|q| PauliSiteDiagnostic {
            site: q,
            mixed_z: mixed_expectation_z(&psi, n_chain, env_q, q),
            branch0_z: expectation_z(&branch0, q),
            branch1_z: expectation_z(&branch1, q),
            mixed_x: mixed_expectation_x(&psi, n_chain, env_q, q),
            branch0_x: expectation_x(&branch0, q),
            branch1_x: expectation_x(&branch1, q),
        })
        .collect();

    let code_like = p0 > 0.05
        && p1 > 0.05
        && sharpness_gain > 1.03
        && rank_reduction < 0.98
        && overlap < 0.95;

    ExcitationSubspaceResult {
        chain_sites: n_chain,
        excitation_peak: peak,
        window_sites: window,
        env_p0: p0,
        env_p1: p1,
        mixed_sharpness,
        branch0_sharpness,
        branch1_sharpness,
        sharpness_gain,
        mixed_effective_rank,
        branch0_effective_rank,
        branch1_effective_rank,
        rank_reduction,
        branch_overlap: overlap,
        site_diagnostics,
        code_like,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn excitation_subspace_code_like_after_decoherence() {
        let config = ExcitationSubspaceConfig {
            n: 8,
            field: 1.2,
            dt: 0.2,
            steps: 22,
            couple_step: 7,
            coupling: 0.9,
            seed: 4242,
            window_radius: 2,
        };
        let r = run_excitation_subspace_probe(&config);
        assert!(
            r.code_like,
            "gain={:.3} rankRed={:.3} overlap={:.3}",
            r.sharpness_gain,
            r.rank_reduction,
            r.branch_overlap
        );
    }
}
