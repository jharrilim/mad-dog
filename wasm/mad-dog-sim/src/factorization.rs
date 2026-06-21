//! Spectrum-driven factorization search — find qubit labelings that make H look local.

use crate::geometry::mutual_information_matrix;
use crate::quantum::{ground_state, Hamiltonian, PauliOp, PauliTerm, QuantumState};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationCandidate {
    /// `perm[original_qubit] = line_position`.
    pub permutation: Vec<usize>,
    pub locality_fraction: f64,
    pub mi_nn_ratio: f64,
    pub score: f64,
    pub nonlocal_terms: usize,
}

/// Remap qubit labels: `perm[old_qubit] = new_label`.
pub fn permute_hamiltonian(h: &Hamiltonian, perm: &[usize]) -> Hamiltonian {
    let terms = h
        .terms
        .iter()
        .map(|term| PauliTerm {
            coeff: term.coeff,
            ops: term
                .ops
                .iter()
                .map(|op| PauliOp {
                    qubit: perm[op.qubit],
                    letter: op.letter,
                })
                .collect(),
        })
        .collect();
    Hamiltonian::new(h.n, terms)
}

fn two_body_qubits(term: &PauliTerm) -> Option<(usize, usize)> {
    if term.ops.len() != 2 {
        return None;
    }
    let a = term.ops[0].qubit;
    let b = term.ops[1].qubit;
    Some(if a < b { (a, b) } else { (b, a) })
}

pub fn score_permutation(
    h: &Hamiltonian,
    perm: &[usize],
    mi: &[Vec<f64>],
) -> FactorizationCandidate {
    let n = h.n;
    let mut inv = vec![0usize; n];
    for (q, &p) in perm.iter().enumerate() {
        inv[p] = q;
    }

    let mut local = 0usize;
    let mut nonlocal = 0usize;
    for term in &h.terms {
        if let Some((i, j)) = two_body_qubits(term) {
            let dist = (perm[i] as i32 - perm[j] as i32).unsigned_abs();
            if dist == 1 {
                local += 1;
            } else {
                nonlocal += 1;
            }
        }
    }
    let total_2body = local + nonlocal;
    let locality_fraction = if total_2body > 0 {
        local as f64 / total_2body as f64
    } else {
        1.0
    };

    let mut nn_sum = 0.0;
    let mut nn_count = 0usize;
    let mut far_sum = 0.0;
    let mut far_count = 0usize;
    for k in 0..(n - 1) {
        let i = inv[k];
        let j = inv[k + 1];
        nn_sum += mi[i][j];
        nn_count += 1;
    }
    for k in 0..n {
        for d in 2..n {
            if k + d < n {
                let i = inv[k];
                let j = inv[k + d];
                far_sum += mi[i][j];
                far_count += 1;
            }
        }
    }
    let nn_avg = if nn_count > 0 {
        nn_sum / nn_count as f64
    } else {
        0.0
    };
    let far_avg = if far_count > 0 {
        far_sum / far_count as f64
    } else {
        1e-9
    };
    let mi_nn_ratio = nn_avg / far_avg.max(1e-12);

    let score = 0.6 * locality_fraction + 0.4 * (mi_nn_ratio / (mi_nn_ratio + 1.0));

    FactorizationCandidate {
        permutation: perm.to_vec(),
        locality_fraction,
        mi_nn_ratio,
        score,
        nonlocal_terms: nonlocal,
    }
}

fn identity_perm(n: usize) -> Vec<usize> {
    (0..n).collect()
}

fn random_permutation(n: usize, rng: &mut Rng) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rng.next() * (i + 1) as f64).floor() as usize;
        perm.swap(i, j);
    }
    perm
}

fn heap_permute(n: usize, a: &mut [usize], k: usize, out: &mut Vec<Vec<usize>>) {
    if k == 1 {
        out.push(a.to_vec());
        return;
    }
    heap_permute(n, a, k - 1, out);
    for i in 0..(k - 1) {
        if k % 2 == 0 {
            a.swap(i, k - 1);
        } else {
            a.swap(0, k - 1);
        }
        heap_permute(n, a, k - 1, out);
    }
}

fn all_permutations(n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<usize> = (0..n).collect();
    let mut out = Vec::new();
    heap_permute(n, &mut a, n, &mut out);
    out
}

fn greedy_search(h: &Hamiltonian, mi: &[Vec<f64>], max_iters: usize) -> FactorizationCandidate {
    let n = h.n;
    let mut perm = identity_perm(n);
    let mut best = score_permutation(h, &perm, mi);
    for _ in 0..max_iters {
        let mut improved = false;
        for i in 0..n {
            for j in (i + 1)..n {
                perm.swap(i, j);
                let cand = score_permutation(h, &perm, mi);
                if cand.score > best.score + 1e-12 {
                    best = cand;
                    improved = true;
                } else {
                    perm.swap(i, j);
                }
            }
        }
        if !improved {
            break;
        }
    }
    best
}

pub fn search_factorization(
    h: &Hamiltonian,
    state: &QuantumState,
    top_k: usize,
) -> (FactorizationCandidate, FactorizationCandidate, Vec<FactorizationCandidate>) {
    let n = h.n;
    let mi = mutual_information_matrix(state);
    let identity = score_permutation(h, &identity_perm(n), &mi);

    let mut candidates: Vec<FactorizationCandidate> = if n <= 8 {
        all_permutations(n)
            .into_iter()
            .map(|perm| score_permutation(h, &perm, &mi))
            .collect()
    } else {
        let mut rng = Rng::new(991);
        let mut out = vec![identity.clone()];
        for _ in 0..32 {
            let perm = random_permutation(n, &mut rng);
            out.push(score_permutation(h, &perm, &mi));
        }
        out.push(greedy_search(h, &mi, 40));
        out
    };

    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.dedup_by(|a, b| {
        (a.score - b.score).abs() < 1e-9 && a.permutation == b.permutation
    });

    let best = candidates.first().cloned().unwrap_or(identity.clone());
    let top: Vec<FactorizationCandidate> = candidates.into_iter().take(top_k).collect();
    (identity, best, top)
}

pub fn ground_state_for(h: &Hamiltonian, seed: u32) -> (QuantumState, f64) {
    let mut rng = Rng::new(seed);
    let (state, energy, _) = ground_state(h, &mut rng, 4000, 1e-8);
    (state, energy)
}

pub fn shuffle_hamiltonian(h: &Hamiltonian, seed: u32) -> (Hamiltonian, Vec<usize>) {
    let mut rng = Rng::new(seed.wrapping_add(4242));
    let perm = random_permutation(h.n, &mut rng);
    (permute_hamiltonian(h, &perm), perm)
}
