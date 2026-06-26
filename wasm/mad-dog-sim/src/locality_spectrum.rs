//! Locality-from-spectrum blind inference battery.
//!
//! Tests whether MI + bandwidth alone (no Ĥ knowledge) can recover the correct
//! 1D factorization of shuffled quantum chains from eigenvectors only.
//!
//! When `spectrum_scrambled=true`, `search_factorization` sets h_ref=None and
//! uses weights: 50% MI nearest-neighbor ratio, 35% bandwidth, 15% emergent-dim
//! bonus — no Hamiltonian term structure is consulted.

use crate::factorization::{
    line_equiv_match, search_factorization, shuffle_hamiltonian, spectrum_from_hamiltonian,
    GraphKind, InputMode, SearchMethod, SearchParams, SpectrumData,
};
use crate::models::{tfim_chain, xx_chain};
use serde::Serialize;

const EIGENSTATE_COUNT: usize = 3;
const PASS_RATE: f64 = 0.67; // ≥67% of cases must recover correct factorization

/// Result for one blind inference trial.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalitySpectrumCase {
    pub model: String,
    pub n: usize,
    pub field: f64,
    pub seed: u32,
    pub recovered: bool,
    pub score: f64,
    pub mi_nn_ratio: f64,
}

/// Aggregate result of the locality-from-spectrum battery.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LocalitySpectrumResult {
    pub total: usize,
    pub recovered: usize,
    pub recovery_rate: f64,
    pub pass: bool,
    pub cases: Vec<LocalitySpectrumCase>,
    pub elapsed_ms: f64,
}

fn inverse_perm(perm: &[usize]) -> Vec<usize> {
    let mut inv = vec![0usize; perm.len()];
    for (q, &p) in perm.iter().enumerate() {
        inv[p] = q;
    }
    inv
}

fn spectrum_only_params(n: usize) -> SearchParams {
    SearchParams {
        input_mode: InputMode::Spectrum,
        spectrum_scrambled: true, // h_ref=None; use 50%MI+35%BW+15%dim
        search_method: SearchMethod::Exact,
        graph_kind: GraphKind::Line,
        rows: 1,
        cols: n,
        eigenstate_count: EIGENSTATE_COUNT,
        distance_decay: 0.0,
        annealing_steps: 3000,
        emergent_dim_weight: 0.1,
    }
}

fn blind_case(model_kind: &str, n: usize, field: f64, seed: u32) -> LocalitySpectrumCase {
    // Build base Hamiltonian, then shuffle qubit labels
    let base_h = match model_kind {
        "tfim" => tfim_chain(n, 1.0, field).hamiltonian,
        "xx" => xx_chain(n, 1.0, field).hamiltonian,
        other => panic!("unknown model kind: {other}"),
    };
    let (shuffled_h, true_shuffle) = shuffle_hamiltonian(&base_h, seed);

    // Compute REAL eigenvectors of the shuffled Hamiltonian (no scrambling)
    let spectrum: SpectrumData = spectrum_from_hamiltonian(&shuffled_h, EIGENSTATE_COUNT);

    // Search using MI + bandwidth only (spectrum_scrambled=true disables Ĥ term)
    let params = spectrum_only_params(n);
    let outcome = search_factorization(&shuffled_h, &[], 1, &params, Some(&spectrum));

    // Recovery: does the best permutation match the inverse of the applied shuffle?
    let inv = inverse_perm(&true_shuffle);
    let recovered = line_equiv_match(&outcome.best.permutation, &inv);

    LocalitySpectrumCase {
        model: format!("{model_kind} n={n} h={field:.2}"),
        n,
        field,
        seed,
        recovered,
        score: outcome.best.score,
        mi_nn_ratio: outcome.best.mi_nn_ratio,
    }
}

/// Run the full locality-from-spectrum battery.
///
/// 9 cases across 3 TFIM phases × 3 seeds each:
/// - TFIM ordered phase (h=0.5), seeds 42/43/44
/// - TFIM near-critical (h=1.0), seeds 42/43/44
/// - TFIM paramagnet (h=1.5), seeds 42/43/44
///
/// Note: XX chain in the gapless phase (h/J < 1) has algebraically-decaying MI,
/// making all chain orderings look identical to the scorer — a known limitation
/// of pure MI+bandwidth inference without Ĥ.
///
/// Pass if ≥67% recover the correct spatial factorization.
pub fn run_locality_spectrum_battery() -> LocalitySpectrumResult {
    let mut cases = Vec::new();

    // TFIM ordered phase: strong ferromagnetic order, short-range MI → clear chain locality
    for seed in [42u32, 43, 44] {
        cases.push(blind_case("tfim", 6, 0.5, seed));
    }
    // TFIM near-critical: entanglement structure changes, power-law correlations, harder test
    for seed in [42u32, 43, 44] {
        cases.push(blind_case("tfim", 6, 1.0, seed));
    }
    // TFIM paramagnet: transverse-field dominated, gapped, different MI pattern
    for seed in [42u32, 43, 44] {
        cases.push(blind_case("tfim", 6, 1.5, seed));
    }

    let total = cases.len();
    let recovered = cases.iter().filter(|c| c.recovered).count();
    let recovery_rate = recovered as f64 / total as f64;

    LocalitySpectrumResult {
        total,
        recovered,
        recovery_rate,
        pass: recovery_rate >= PASS_RATE,
        cases,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn blind_inference_recovers_tfim_ordered() {
        // Single case: TFIM ordered phase should recover easily
        let case = blind_case("tfim", 6, 0.5, 42);
        assert!(
            case.recovered,
            "MI+bandwidth blind inference should recover TFIM ordered-phase chain (seed 42); \
             got score={:.3} mi_nn={:.3}",
            case.score, case.mi_nn_ratio
        );
    }

    #[test]
    fn battery_passes_threshold() {
        let result = run_locality_spectrum_battery();
        assert!(
            result.pass,
            "locality-from-spectrum battery: {}/{} recovered ({:.0}% < 67% threshold). Cases:\n{}",
            result.recovered,
            result.total,
            result.recovery_rate * 100.0,
            result
                .cases
                .iter()
                .map(|c| format!("  {} seed={} recovered={} score={:.3}", c.model, c.seed, c.recovered, c.score))
                .collect::<Vec<_>>()
                .join("\n")
        );
    }

    #[test]
    fn tfim_paramagnet_recovers() {
        // TFIM paramagnet (h=1.5, transverse-field dominated): should still show chain-local MI
        let case = blind_case("tfim", 6, 1.5, 42);
        assert!(
            case.recovered,
            "TFIM paramagnet blind inference should recover chain; \
             score={:.3} mi_nn={:.3}",
            case.score, case.mi_nn_ratio
        );
    }
}
