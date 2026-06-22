//! Independent EFT DOF estimators for U′ (Phase 9 circularity hardening).
//!
//! `measured_dof_per_site` uses branch effective rank; `predicted_dof_per_site` uses
//! stabilizer code rate. This module supplies a third estimate from the entanglement
//! spectrum participation ratio — no Pauli stabilizer search.

use crate::geometry::region_eigenvalues;
use crate::quantum::QuantumState;

/// Participation ratio of a normalized density-matrix spectrum: 1 / Σ λ².
pub fn participation_ratio(eigenvalues: &[f64]) -> f64 {
    let mut sum_sq = 0.0;
    for &v in eigenvalues {
        if v > 1e-12 {
            sum_sq += v * v;
        }
    }
    if sum_sq <= 1e-12 {
        return 1.0;
    }
    1.0 / sum_sq
}

/// Average branch participation ratio on `window`, normalized per site.
pub fn independent_dof_per_site(
    branch0: &QuantumState,
    branch1: &QuantumState,
    window: &[usize],
) -> f64 {
    let window_len = window.len().max(1);
    let pr0 = participation_ratio(&region_eigenvalues(branch0, window));
    let pr1 = participation_ratio(&region_eigenvalues(branch1, window));
    (pr0 + pr1) / (2.0 * window_len as f64)
}

/// Whether two DOF-per-site estimates agree within relative / absolute tolerance.
pub fn dof_pair_agrees(a: f64, b: f64, rel_tol: f64, abs_tol: f64) -> bool {
    if b > 1e-6 {
        ((a - b) / b).abs() < rel_tol || (a - b).abs() < abs_tol
    } else {
        a.abs() < abs_tol
    }
}

/// U′ pass: at least two of {measured, predicted, independent} agree.
pub fn two_of_three_dof_agreement(
    measured: f64,
    predicted: f64,
    independent: f64,
    rel_tol: f64,
    abs_tol: f64,
) -> (bool, bool, bool, bool) {
    let mp = dof_pair_agrees(measured, predicted, rel_tol, abs_tol);
    let mi = dof_pair_agrees(measured, independent, rel_tol, abs_tol);
    let pi = dof_pair_agrees(predicted, independent, rel_tol, abs_tol);
    let passes = [mp, mi, pi].iter().filter(|&&x| x).count() >= 2;
    (passes, mp, mi, pi)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn participation_ratio_uniform_and_single() {
        let uniform = vec![0.25; 4];
        assert!((participation_ratio(&uniform) - 4.0).abs() < 1e-9);
        let single = vec![1.0];
        assert!((participation_ratio(&single) - 1.0).abs() < 1e-9);
    }

    #[test]
    fn two_of_three_requires_two_pairs() {
        let (pass, mp, mi, pi) = two_of_three_dof_agreement(1.0, 1.05, 2.0, 0.1, 0.05);
        assert!(mp);
        assert!(!mi);
        assert!(!pi);
        assert!(!pass, "only one pair agrees");

        let (pass, _, _, _) = two_of_three_dof_agreement(1.0, 1.02, 1.04, 0.1, 0.05);
        assert!(pass, "all three close enough for two pairs");

        let (pass, _, _, _) = two_of_three_dof_agreement(1.0, 2.0, 3.0, 0.1, 0.05);
        assert!(!pass);
    }
}
