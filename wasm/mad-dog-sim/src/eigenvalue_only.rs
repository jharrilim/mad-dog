//! Eigenvalue-only blind inference — impossibility battery.
//!
//! The full Hamiltonian spectrum {Eₙ} is invariant under qubit permutations, so
//! no labeling can be inferred from eigenvalues alone. This module documents that
//! limit with automated checks and contrasts spectrum+ψ recovery.

use crate::factorization::{
    line_equiv_match, search_factorization, shuffle_hamiltonian, spectrum_from_hamiltonian,
    GraphKind, InputMode, SearchMethod, SearchParams,
};
use crate::models::tfim_chain;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EigenvalueOnlyCase {
    pub n: usize,
    pub seed: u32,
    pub recovered: bool,
    pub spectrum_recovered: bool,
    pub score_spread: f64,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EigenvalueOnlyResult {
    pub total: usize,
    pub eigenvalue_recovered: usize,
    pub spectrum_recovered: usize,
    pub pass: bool,
    pub cases: Vec<EigenvalueOnlyCase>,
    pub elapsed_ms: f64,
}

fn inverse_perm(perm: &[usize]) -> Vec<usize> {
    let mut inv = vec![0usize; perm.len()];
    for (q, &p) in perm.iter().enumerate() {
        inv[p] = q;
    }
    inv
}

fn eigenvalue_only_params(n: usize) -> SearchParams {
    SearchParams {
        input_mode: InputMode::EigenvaluesOnly,
        search_method: SearchMethod::Exact,
        graph_kind: GraphKind::Line,
        cols: n,
        eigenstate_count: n.min(8),
        ..Default::default()
    }
}

fn spectrum_blind_params(n: usize) -> SearchParams {
    SearchParams {
        input_mode: InputMode::Spectrum,
        spectrum_scrambled: true,
        search_method: SearchMethod::Exact,
        graph_kind: GraphKind::Line,
        cols: n,
        eigenstate_count: 3,
        ..Default::default()
    }
}

fn score_spread(outcome: &crate::factorization::SearchOutcome) -> f64 {
    if outcome.top_candidates.is_empty() {
        return 0.0;
    }
    let scores: Vec<f64> = outcome
        .top_candidates
        .iter()
        .map(|c| c.score)
        .collect();
    let min = scores.iter().cloned().fold(f64::INFINITY, f64::min);
    let max = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
    max - min
}

fn run_case(n: usize, seed: u32) -> EigenvalueOnlyCase {
    let base = tfim_chain(n, 1.0, 1.5);
    let (shuffled, shuffle) = shuffle_hamiltonian(&base.hamiltonian, seed);
    let inv = inverse_perm(&shuffle);
    let spec = spectrum_from_hamiltonian(&shuffled, n.min(8));

    let eig_out = search_factorization(&shuffled, &[], 5, &eigenvalue_only_params(n), Some(&spec));
    let spec_out = search_factorization(&shuffled, &[], 1, &spectrum_blind_params(n), Some(&spec));

    EigenvalueOnlyCase {
        n,
        seed,
        recovered: line_equiv_match(&eig_out.best.permutation, &inv),
        spectrum_recovered: line_equiv_match(&spec_out.best.permutation, &inv),
        score_spread: score_spread(&eig_out),
    }
}

/// Battery: {Eₙ}-only must **not** recover labeling; spectrum+ψ control **should**.
pub fn run_eigenvalue_only_battery() -> EigenvalueOnlyResult {
    let cases = vec![
        run_case(4, 42),
        run_case(4, 43),
        run_case(5, 42),
        run_case(5, 43),
    ];
    let total = cases.len();
    let eigenvalue_recovered = cases.iter().filter(|c| c.recovered).count();
    let spectrum_recovered = cases.iter().filter(|c| c.spectrum_recovered).count();
    let pass = eigenvalue_recovered == 0
        && spectrum_recovered >= total.saturating_sub(1)
        && cases.iter().all(|c| c.score_spread < 1e-9);

    EigenvalueOnlyResult {
        total,
        eigenvalue_recovered,
        spectrum_recovered,
        pass,
        cases,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn battery_documents_impossibility() {
        let r = run_eigenvalue_only_battery();
        assert!(r.pass, "{:?}", r);
        assert_eq!(r.eigenvalue_recovered, 0);
    }
}
