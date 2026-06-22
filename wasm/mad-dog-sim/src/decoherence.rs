//! Minimal environment coupling — branch-resolved worldlines after decoherence.

use crate::models::tfim_chain;
use crate::quantum::{
    evolve_interval_inplace, expectation_z_all, signal_from_z, EvolveScratch, Hamiltonian,
    PauliLetter, PauliOp, PauliTerm, QuantumState,
};
use crate::rng::Rng;
use crate::spacetime::{track_single_worldline, WorldlinePoint};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoherenceQuenchConfig {
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

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoherenceSlice {
    pub k: usize,
    pub t: f64,
    pub signal: Vec<f64>,
    pub env_p0: f64,
    pub env_p1: f64,
    pub env_entropy: f64,
    pub env_coherence: f64,
    pub mixed_peak_width: f64,
    pub branch_peak_width: f64,
    pub mixed_entropy: f64,
    pub branch_entropy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BranchTrack {
    pub label: String,
    pub env_bit: u8,
    pub weight: f64,
    pub worldline: Vec<WorldlinePoint>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DecoherenceQuenchResult {
    pub chain_sites: usize,
    pub env_qubit: usize,
    pub couple_step: usize,
    pub coupling: f64,
    pub slices: Vec<DecoherenceSlice>,
    pub mixed_worldline: Vec<WorldlinePoint>,
    pub branches: Vec<BranchTrack>,
    pub sharpen_ratio: f64,
    pub branches_distinguishable: bool,
    pub elapsed_ms: f64,
}

fn chain_plus_env_hamiltonian(n_chain: usize, env_q: usize, chain: &Hamiltonian, g: f64) -> Hamiltonian {
    let mut terms = chain.terms.clone();
    if g.abs() > 1e-12 {
        let mid = n_chain / 2;
        for i in 0..mid {
            terms.push(PauliTerm {
                coeff: -g * 0.5,
                ops: vec![
                    PauliOp {
                        qubit: i,
                        letter: PauliLetter::Z,
                    },
                    PauliOp {
                        qubit: env_q,
                        letter: PauliLetter::X,
                    },
                ],
            });
        }
        for i in (mid + 1)..n_chain {
            terms.push(PauliTerm {
                coeff: g * 0.5,
                ops: vec![
                    PauliOp {
                        qubit: i,
                        letter: PauliLetter::Z,
                    },
                    PauliOp {
                        qubit: env_q,
                        letter: PauliLetter::X,
                    },
                ],
            });
        }
        if mid > 0 {
            terms.push(PauliTerm {
                coeff: -g,
                ops: vec![
                    PauliOp {
                        qubit: mid - 1,
                        letter: PauliLetter::Z,
                    },
                    PauliOp {
                        qubit: env_q,
                        letter: PauliLetter::X,
                    },
                ],
            });
        }
        if mid + 1 < n_chain {
            terms.push(PauliTerm {
                coeff: g,
                ops: vec![
                    PauliOp {
                        qubit: mid + 1,
                        letter: PauliLetter::Z,
                    },
                    PauliOp {
                        qubit: env_q,
                        letter: PauliLetter::X,
                    },
                ],
            });
        }
    }
    Hamiltonian::new(n_chain + 1, terms)
}

/// CNOT: flip target when control is |1⟩.
fn apply_cnot(state: &mut QuantumState, control: usize, target: usize) {
    let dim = state.dim;
    let mut out = vec![0.0; 2 * dim];
    for s in 0..dim {
        let dst = if (s >> control) & 1 == 1 {
            s ^ (1 << target)
        } else {
            s
        };
        out[2 * dst] = state.data[2 * s];
        out[2 * dst + 1] = state.data[2 * s + 1];
    }
    state.data.copy_from_slice(&out);
}

fn defect_initial(n_chain: usize, env_q: usize, center: usize) -> QuantumState {
    let n = n_chain + 1;
    let mut state = QuantumState::zero(n);
    let chain_idx = 1usize << center;
    state.data[2 * chain_idx] = 1.0;
    let _ = env_q;
    state
}

fn reference_initial(n_chain: usize, env_q: usize) -> QuantumState {
    let n = n_chain + 1;
    let mut state = QuantumState::zero(n);
    state.data[0] = 1.0;
    let _ = env_q;
    state
}

fn chain_signal(state: &QuantumState, n_chain: usize, ref_z: &[f64]) -> Vec<f64> {
    let z = expectation_z_all(state);
    let chain_z: Vec<f64> = z[..n_chain].to_vec();
    let ref_chain: Vec<f64> = ref_z[..n_chain].to_vec();
    signal_from_z(&chain_z, &ref_chain)
}

pub(crate) fn env_branch_weights(state: &QuantumState, env_q: usize) -> (f64, f64) {
    let mut p0 = 0.0;
    let mut p1 = 0.0;
    for s in 0..state.dim {
        let re = state.data[2 * s];
        let im = state.data[2 * s + 1];
        let prob = re * re + im * im;
        if (s >> env_q) & 1 == 0 {
            p0 += prob;
        } else {
            p1 += prob;
        }
    }
    (p0, p1)
}

fn env_coherence_proxy(state: &QuantumState, env_q: usize) -> f64 {
    let chain_dim = state.dim / 2;
    let mut re01 = 0.0;
    let mut im01 = 0.0;
    for chain_idx in 0..chain_dim {
        let s0 = chain_idx;
        let s1 = chain_idx | (1 << env_q);
        let a0r = state.data[2 * s0];
        let a0i = state.data[2 * s0 + 1];
        let a1r = state.data[2 * s1];
        let a1i = state.data[2 * s1 + 1];
        re01 += a0r * a1r + a0i * a1i;
        im01 += a0r * a1i - a0i * a1r;
    }
    (re01 * re01 + im01 * im01).sqrt().min(1.0)
}

pub(crate) fn chain_conditional_state(
    state: &QuantumState,
    n_chain: usize,
    env_q: usize,
    env_bit: usize,
) -> QuantumState {
    let chain_dim = 1 << n_chain;
    let mut out = QuantumState::zero(n_chain);
    for chain_idx in 0..chain_dim {
        let full_idx = chain_idx | (env_bit << env_q);
        out.data[2 * chain_idx] = state.data[2 * full_idx];
        out.data[2 * chain_idx + 1] = state.data[2 * full_idx + 1];
    }
    out.normalize();
    out
}

fn signal_entropy(signal: &[f64]) -> f64 {
    let sum: f64 = signal.iter().sum();
    if sum < 1e-12 {
        return 0.0;
    }
    let mut h = 0.0;
    for v in signal {
        let p = v / sum;
        if p > 1e-12 {
            h -= p * p.ln();
        }
    }
    h
}

fn worldline_jitter(worldline: &[WorldlinePoint]) -> f64 {
    if worldline.len() < 2 {
        return 0.0;
    }
    let mut jumps = 0.0;
    for w in worldline.windows(2) {
        jumps += (w[1].site as f64 - w[0].site as f64).abs();
    }
    jumps / (worldline.len() - 1) as f64
}
fn peak_width(signal: &[f64]) -> f64 {
    let max = signal.iter().copied().fold(0.0_f64, f64::max);
    if max < 1e-12 {
        return signal.len() as f64;
    }
    let thresh = 0.5 * max;
    let active: Vec<usize> = signal
        .iter()
        .enumerate()
        .filter(|(_, v)| **v >= thresh)
        .map(|(i, _)| i)
        .collect();
    if active.is_empty() {
        return signal.len() as f64;
    }
    (*active.iter().max().unwrap() - active.iter().min().unwrap() + 1) as f64
}

fn entropy2(p0: f64, p1: f64) -> f64 {
    let mut s = 0.0;
    for p in [p0, p1] {
        if p > 1e-12 {
            s -= p * p.ln();
        }
    }
    s
}

/// Evolve the decoherence quench to the final full (chain + env) state.
pub fn decoherence_final_state(config: &DecoherenceQuenchConfig) -> (QuantumState, usize, usize) {
    let n_chain = config.n.max(4);
    let env_q = n_chain;
    let couple_step = config.couple_step.min(config.steps.saturating_sub(1));
    let center = n_chain / 2;
    let model = tfim_chain(n_chain, 1.0, config.field);
    let g = config.coupling;

    let mut psi = defect_initial(n_chain, env_q, center);
    let mut rng = Rng::new(config.seed);
    let h_free = chain_plus_env_hamiltonian(n_chain, env_q, &model.hamiltonian, 0.0);
    let h_coupled = chain_plus_env_hamiltonian(n_chain, env_q, &model.hamiltonian, g);
    let radius = h_coupled.spectral_radius(&mut rng);
    let mut scratch = EvolveScratch::new(psi.dim);

    for k in 0..config.steps {
        let h = if k >= couple_step { &h_coupled } else { &h_free };
        evolve_interval_inplace(h, &mut psi, config.dt, radius, 6, &mut scratch);
        if k + 1 == couple_step {
            apply_cnot(&mut psi, center, env_q);
        }
    }
    (psi, n_chain, env_q)
}

pub fn run_decoherence_quench(config: &DecoherenceQuenchConfig) -> DecoherenceQuenchResult {
    let n_chain = config.n.max(4);
    let env_q = n_chain;
    let couple_step = config.couple_step.min(config.steps.saturating_sub(1));
    let center = n_chain / 2;
    let model = tfim_chain(n_chain, 1.0, config.field);
    let g = config.coupling;

    let mut psi = defect_initial(n_chain, env_q, center);
    let reference = reference_initial(n_chain, env_q);
    let ref_z = expectation_z_all(&reference);
    let base_z = expectation_z_all(&psi);

    let mut rng = Rng::new(config.seed);
    let h_free = chain_plus_env_hamiltonian(n_chain, env_q, &model.hamiltonian, 0.0);
    let h_coupled = chain_plus_env_hamiltonian(n_chain, env_q, &model.hamiltonian, g);
    let radius = h_coupled.spectral_radius(&mut rng);
    let mut scratch = EvolveScratch::new(psi.dim);

    let mut slices = Vec::with_capacity(config.steps);
    let mut mixed_worldline = Vec::with_capacity(config.steps);
    let mut branch0_wl = Vec::with_capacity(config.steps);
    let mut branch1_wl = Vec::with_capacity(config.steps);

    for k in 0..config.steps {
        let t = k as f64 * config.dt;
        let h = if k >= couple_step { &h_coupled } else { &h_free };

        let mixed_signal = chain_signal(&psi, n_chain, &ref_z);
        let (p0, p1) = env_branch_weights(&psi, env_q);
        let coherence = env_coherence_proxy(&psi, env_q);

        let cond0 = chain_conditional_state(&psi, n_chain, env_q, 0);
        let cond1 = chain_conditional_state(&psi, n_chain, env_q, 1);
        let sig0 = signal_from_z(
            &expectation_z_all(&cond0)[..n_chain],
            &base_z[..n_chain],
        );
        let sig1 = signal_from_z(
            &expectation_z_all(&cond1)[..n_chain],
            &base_z[..n_chain],
        );
        let branch_width = if p0 > 1e-6 && p1 > 1e-6 {
            (peak_width(&sig0) + peak_width(&sig1)) / 2.0
        } else {
            peak_width(&mixed_signal)
        };
        let branch_ent = if p0 > 1e-6 && p1 > 1e-6 {
            (signal_entropy(&sig0) + signal_entropy(&sig1)) / 2.0
        } else {
            signal_entropy(&mixed_signal)
        };

        let mixed_width = peak_width(&mixed_signal);
        let mixed_ent = signal_entropy(&mixed_signal);

        mixed_worldline.push(track_single_worldline(
            &mixed_signal,
            center,
            n_chain,
            1,
            t,
        ));
        branch0_wl.push(track_single_worldline(&sig0, center, n_chain, 1, t));
        branch1_wl.push(track_single_worldline(&sig1, center, n_chain, 1, t));

        slices.push(DecoherenceSlice {
            k,
            t,
            signal: mixed_signal,
            env_p0: p0,
            env_p1: p1,
            env_entropy: entropy2(p0, p1),
            env_coherence: coherence,
            mixed_peak_width: mixed_width,
            branch_peak_width: branch_width,
            mixed_entropy: mixed_ent,
            branch_entropy: branch_ent,
        });

        evolve_interval_inplace(h, &mut psi, config.dt, radius, 6, &mut scratch);
        if k + 1 == couple_step {
            apply_cnot(&mut psi, center, env_q);
        }
    }

    let late = slices.iter().skip(couple_step).collect::<Vec<_>>();
    let late_mixed_ent: f64 = late.iter().map(|s| s.mixed_entropy).sum::<f64>()
        / late.len().max(1) as f64;
    let late_branch_ent: f64 = late.iter().map(|s| s.branch_entropy).sum::<f64>()
        / late.len().max(1) as f64;
    let entropy_ratio = if late_branch_ent > 1e-9 {
        late_mixed_ent / late_branch_ent
    } else {
        1.0
    };
    let mixed_jitter = worldline_jitter(&mixed_worldline[couple_step..]);
    let branch_jitter = (worldline_jitter(&branch0_wl[couple_step..])
        + worldline_jitter(&branch1_wl[couple_step..]))
        / 2.0;
    let jitter_ratio = if branch_jitter > 1e-9 {
        mixed_jitter / branch_jitter
    } else {
        1.0
    };
    let sharpen_ratio = entropy_ratio.max(jitter_ratio);

    let final_p = env_branch_weights(&psi, env_q);
    let late_coherence: f64 = late.iter().map(|s| s.env_coherence).sum::<f64>()
        / late.len().max(1) as f64;
    let branches_distinguishable = final_p.0 > 0.05
        && final_p.1 > 0.05
        && late.iter().any(|s| s.env_entropy > 0.1)
        && (sharpen_ratio > 1.02 || late_coherence < 0.85);

    DecoherenceQuenchResult {
        chain_sites: n_chain,
        env_qubit: env_q,
        couple_step,
        coupling: g,
        slices,
        mixed_worldline,
        branches: vec![
            BranchTrack {
                label: "Branch |0⟩_env".to_string(),
                env_bit: 0,
                weight: final_p.0,
                worldline: branch0_wl,
            },
            BranchTrack {
                label: "Branch |1⟩_env".to_string(),
                env_bit: 1,
                weight: final_p.1,
                worldline: branch1_wl,
            },
        ],
        sharpen_ratio,
        branches_distinguishable,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn decoherence_branches_sharpen() {
        let config = DecoherenceQuenchConfig {
            n: 8,
            field: 1.2,
            dt: 0.2,
            steps: 22,
            couple_step: 7,
            coupling: 0.9,
            seed: 4242,
        };
        let result = run_decoherence_quench(&config);
        assert!(
            result.branches_distinguishable,
            "sharpen={} p0={} p1={}",
            result.sharpen_ratio,
            result.branches[0].weight,
            result.branches[1].weight
        );
        assert!(result.sharpen_ratio >= 1.0);
    }
}
