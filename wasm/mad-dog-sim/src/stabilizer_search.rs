//! Pauli-group stabilizer search on branch-resolved excitation windows (Phase 6 / I′).

use crate::excitation_subspace::{mixed_expectation_x, mixed_expectation_z};
use crate::quantum::{pauli_expectation, PauliLetter, QuantumState};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StabilizerGenerator {
    pub label: String,
    pub weight: usize,
    pub mixed_expectation: f64,
    pub branch0_expectation: f64,
    pub branch1_expectation: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StabilizerSearchReport {
    pub window_sites: Vec<usize>,
    pub generators: Vec<StabilizerGenerator>,
    pub generator_count: usize,
    pub code_distance: usize,
    pub code_rate: f64,
    pub stabilizer_found: bool,
}

fn pauli_anticommute_count(
    a: &[(usize, PauliLetter)],
    b: &[(usize, PauliLetter)],
) -> usize {
    let mut count = 0usize;
    for &(qa, la) in a {
        for &(qb, lb) in b {
            if qa != qb {
                continue;
            }
            let anticom = matches!(
                (la, lb),
                (PauliLetter::X, PauliLetter::Z)
                    | (PauliLetter::Z, PauliLetter::X)
                    | (PauliLetter::X, PauliLetter::Y)
                    | (PauliLetter::Y, PauliLetter::X)
                    | (PauliLetter::Y, PauliLetter::Z)
                    | (PauliLetter::Z, PauliLetter::Y)
            );
            if anticom {
                count += 1;
            }
        }
    }
    count
}

fn pauli_commute(a: &[(usize, PauliLetter)], b: &[(usize, PauliLetter)]) -> bool {
    pauli_anticommute_count(a, b) % 2 == 0
}

fn mixed_pauli_expectation(
    psi: &QuantumState,
    n_chain: usize,
    env_q: usize,
    ops: &[(usize, PauliLetter)],
) -> f64 {
    match ops.len() {
        0 => 1.0,
        1 => match ops[0].1 {
            PauliLetter::X => mixed_expectation_x(psi, n_chain, env_q, ops[0].0),
            PauliLetter::Z => mixed_expectation_z(psi, n_chain, env_q, ops[0].0),
            PauliLetter::Y => 0.0,
        },
        2 if ops[0].1 == ops[1].1 => {
            let e0 = mixed_pauli_expectation(psi, n_chain, env_q, &[ops[0]]);
            let e1 = mixed_pauli_expectation(psi, n_chain, env_q, &[ops[1]]);
            e0 * e1
        }
        // Weight-3 same-letter: product approximation (valid when sites decorrelate in mixed state)
        3 if ops[0].1 == ops[1].1 && ops[1].1 == ops[2].1 => {
            let e0 = mixed_pauli_expectation(psi, n_chain, env_q, &[ops[0]]);
            let e1 = mixed_pauli_expectation(psi, n_chain, env_q, &[ops[1]]);
            let e2 = mixed_pauli_expectation(psi, n_chain, env_q, &[ops[2]]);
            e0 * e1 * e2
        }
        _ => 0.0,
    }
}

fn branch_stabilizer_candidate(
    psi: &QuantumState,
    n_chain: usize,
    env_q: usize,
    branch0: &QuantumState,
    branch1: &QuantumState,
    ops: &[(usize, PauliLetter)],
) -> (bool, f64, f64, f64) {
    let em = mixed_pauli_expectation(psi, n_chain, env_q, ops);
    let e0 = pauli_expectation(branch0, ops);
    let e1 = pauli_expectation(branch1, ops);
    let sharpens = e0.abs() > em.abs() + 0.02 && e1.abs() > em.abs() + 0.02;
    let agrees = (e0 - e1).abs() < 0.25;
    let strong = e0.abs() > 0.07 && e1.abs() > 0.07;
    (sharpens && agrees && strong, em, e0, e1)
}

fn candidate_strings(window: &[usize]) -> Vec<(String, Vec<(usize, PauliLetter)>)> {
    let mut out = Vec::new();
    for &q in window {
        out.push((format!("X{q}"), vec![(q, PauliLetter::X)]));
        out.push((format!("Z{q}"), vec![(q, PauliLetter::Z)]));
    }
    for w in window.windows(2) {
        let (a, b) = (w[0], w[1]);
        out.push((format!("Z{a}Z{b}"), vec![(a, PauliLetter::Z), (b, PauliLetter::Z)]));
        out.push((format!("X{a}X{b}"), vec![(a, PauliLetter::X), (b, PauliLetter::X)]));
    }
    for w in window.windows(3) {
        let (a, b, c) = (w[0], w[1], w[2]);
        out.push((format!("Z{a}Z{b}Z{c}"), vec![(a, PauliLetter::Z), (b, PauliLetter::Z), (c, PauliLetter::Z)]));
        out.push((format!("X{a}X{b}X{c}"), vec![(a, PauliLetter::X), (b, PauliLetter::X), (c, PauliLetter::X)]));
    }
    out
}

fn pauli_weight(ops: &[(usize, PauliLetter)]) -> usize {
    ops.len()
}

/// Scan Pauli candidates sharpened on branches vs mixed state.
pub fn search_stabilizers(
    psi: &QuantumState,
    n_chain: usize,
    env_q: usize,
    branch0: &QuantumState,
    branch1: &QuantumState,
    window: &[usize],
) -> StabilizerSearchReport {
    if window.is_empty() {
        return StabilizerSearchReport {
            window_sites: window.to_vec(),
            generators: Vec::new(),
            generator_count: 0,
            code_distance: 0,
            code_rate: 0.0,
            stabilizer_found: false,
        };
    }

    let mut generators: Vec<(String, Vec<(usize, PauliLetter)>, f64, f64, f64)> = Vec::new();
    for (label, ops) in candidate_strings(window) {
        let (ok, em, e0, e1) =
            branch_stabilizer_candidate(psi, n_chain, env_q, branch0, branch1, &ops);
        if !ok {
            continue;
        }
        if generators
            .iter()
            .all(|(_, gops, _, _, _)| pauli_commute(gops, &ops))
        {
            generators.push((label, ops, em, e0, e1));
        }
    }

    let generator_count = generators.len();
    let w = window.len();
    let logical_qubits = w.saturating_sub(generator_count);
    let code_rate = if w > 0 {
        logical_qubits as f64 / w as f64
    } else {
        0.0
    };

    let mut code_distance = w.max(1);
    for (label, ops) in candidate_strings(window) {
        if generators.iter().any(|(glabel, _, _, _, _)| glabel == &label) {
            continue;
        }
        if !generators
            .iter()
            .all(|(_, gops, _, _, _)| pauli_commute(gops, &ops))
        {
            continue;
        }
        let (_, _, e0, e1) =
            branch_stabilizer_candidate(psi, n_chain, env_q, branch0, branch1, &ops);
        if e0.abs() < 0.55 && e1.abs() < 0.55 {
            code_distance = code_distance.min(pauli_weight(&ops));
        }
    }
    if generator_count == 0 {
        code_distance = 0;
    }

    let gen_out: Vec<StabilizerGenerator> = generators
        .into_iter()
        .map(|(label, ops, em, e0, e1)| StabilizerGenerator {
            label,
            weight: pauli_weight(&ops),
            mixed_expectation: em,
            branch0_expectation: e0,
            branch1_expectation: e1,
        })
        .collect();

    StabilizerSearchReport {
        window_sites: window.to_vec(),
        generator_count,
        code_distance,
        code_rate,
        stabilizer_found: generator_count >= 1 && code_distance >= 1,
        generators: gen_out,
    }
}

/// Larger window should not shrink code distance (scaling sanity).
pub fn distance_scales_with_window(
    psi: &QuantumState,
    n_chain: usize,
    env_q: usize,
    branch0: &QuantumState,
    branch1: &QuantumState,
    peak: usize,
    radius_small: usize,
    radius_large: usize,
    n_chain_sites: usize,
) -> bool {
    let lo_s = peak.saturating_sub(radius_small);
    let hi_s = (peak + radius_small).min(n_chain_sites - 1);
    let window_s: Vec<usize> = (lo_s..=hi_s).collect();
    let lo_l = peak.saturating_sub(radius_large);
    let hi_l = (peak + radius_large).min(n_chain_sites - 1);
    let window_l: Vec<usize> = (lo_l..=hi_l).collect();
    let small = search_stabilizers(psi, n_chain, env_q, branch0, branch1, &window_s);
    let large = search_stabilizers(psi, n_chain, env_q, branch0, branch1, &window_l);
    large.code_distance >= small.code_distance
        && large.generator_count >= small.generator_count.saturating_sub(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::decoherence::{chain_conditional_state, decoherence_final_state};
    use crate::excitation_subspace::{
        deco_config_from_excitation, run_excitation_subspace_probe, ExcitationSubspaceConfig,
    };

    fn branch_states() -> (QuantumState, usize, usize, QuantumState, QuantumState, Vec<usize>) {
        let excitation = ExcitationSubspaceConfig {
            n: 8,
            field: 1.2,
            dt: 0.2,
            steps: 22,
            couple_step: 7,
            coupling: 0.9,
            seed: 4242,
            window_radius: 2,
            model: None,
        };
        let probe = run_excitation_subspace_probe(&excitation);
        let deco_config = deco_config_from_excitation(&excitation);
        let (psi, n_chain, env_q) = decoherence_final_state(&deco_config);
        let b0 = chain_conditional_state(&psi, n_chain, env_q, 0);
        let b1 = chain_conditional_state(&psi, n_chain, env_q, 1);
        (psi, n_chain, env_q, b0, b1, probe.window_sites)
    }

    #[test]
    fn finds_stabilizer_generators_on_branch_window() {
        let (psi, n_chain, env_q, b0, b1, window) = branch_states();
        let r = search_stabilizers(&psi, n_chain, env_q, &b0, &b1, &window);
        assert!(
            r.stabilizer_found,
            "generators={} distance={}",
            r.generator_count,
            r.code_distance
        );
    }

    #[test]
    fn candidate_strings_includes_weight_3_triples() {
        let window = vec![2usize, 3, 4, 5];
        let strings = candidate_strings(&window);
        let labels: Vec<&str> = strings.iter().map(|(l, _)| l.as_str()).collect();
        assert!(labels.contains(&"Z2Z3Z4"), "ZZZ triple missing");
        assert!(labels.contains(&"X2X3X4"), "XXX triple missing");
        assert!(labels.contains(&"Z3Z4Z5"), "ZZZ triple at offset missing");
        // Weight-3 triples should have exactly 3 ops
        for (label, ops) in &strings {
            if label.starts_with('Z') && label.len() > 4 {
                assert!(ops.len() <= 3, "unexpected op count for {label}");
            }
        }
    }

    #[test]
    fn weight_3_candidates_evaluated_without_panic() {
        let (psi, n_chain, env_q, b0, b1, window) = branch_states();
        // Smoke test: run with weight-3 candidates present (window ≥ 3 sites)
        assert!(window.len() >= 3, "window too small for weight-3 test");
        let r = search_stabilizers(&psi, n_chain, env_q, &b0, &b1, &window);
        // Check if any weight-3 generator was found
        let w3 = r.generators.iter().filter(|g| g.weight == 3).count();
        println!("weight-3 generators found: {w3} / total: {}", r.generator_count);
        // The search should still find at least one generator (weight-2 still present)
        assert!(r.stabilizer_found, "lost stabilizer after adding weight-3 candidates");
    }
}
