//! Locality-from-spectrum blind inference battery.
//!
//! Tests whether MI + bandwidth alone (no Ĥ knowledge) can recover the correct
//! 1D factorization of shuffled quantum chains from eigenvectors only.
//!
//! When `spectrum_scrambled=true`, `search_factorization` sets h_ref=None and
//! uses weights: 50% MI nearest-neighbor ratio, 35% graph bandwidth, 15% emergent-dim
//! bonus — no Hamiltonian term structure is consulted. Far MI pairs and bandwidth
//! are graph-native for Grid/Torus (Manhattan / wrap-aware), not chain-linear.

use crate::factorization::{
    line_equiv_match, search_factorization, shuffle_hamiltonian, spectrum_from_hamiltonian,
    GraphKind, InputMode, SearchMethod, SearchParams, SpectrumData,
};
use crate::models::{tfim_chain, tfim_grid, tfim_torus, xx_chain};
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

fn spectrum_only_params(graph_kind: GraphKind, rows: usize, cols: usize, n: usize) -> SearchParams {
    SearchParams {
        input_mode: InputMode::Spectrum,
        spectrum_scrambled: true, // h_ref=None; use 50%MI+35%BW+15%dim
        search_method: SearchMethod::Exact,
        graph_kind,
        rows,
        cols,
        eigenstate_count: EIGENSTATE_COUNT.min(n).max(2),
        ..Default::default()
    }
}

pub(crate) fn blind_case(model_kind: &str, n: usize, field: f64, seed: u32) -> LocalitySpectrumCase {
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
    let params = spectrum_only_params(GraphKind::Line, 1, n, n);
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

/// Blind MI+bandwidth case on a 2D lattice (for Phase 12 scaling research).
pub fn blind_lattice_case(
    lattice: &str,
    rows: usize,
    cols: usize,
    field: f64,
    seed: u32,
) -> LocalitySpectrumCase {
    let n = rows * cols;
    let base_h = match lattice {
        "grid" => tfim_grid(rows, cols, 1.0, field).hamiltonian,
        "torus" => tfim_torus(rows, cols, 1.0, field).hamiltonian,
        other => panic!("unknown lattice: {other}"),
    };
    let (shuffled_h, true_shuffle) = shuffle_hamiltonian(&base_h, seed);
    let k = EIGENSTATE_COUNT.min(n).max(2);
    let spectrum = spectrum_from_hamiltonian(&shuffled_h, k);
    let graph_kind = if lattice == "torus" {
        GraphKind::Torus
    } else {
        GraphKind::Grid
    };
    let params = spectrum_only_params(graph_kind, rows, cols, n);
    let outcome = search_factorization(&shuffled_h, &[], 1, &params, Some(&spectrum));
    let inv = inverse_perm(&true_shuffle);
    let recovered = crate::factorization::lattice_equiv_match(
        &outcome.best.permutation,
        &inv,
        graph_kind,
        rows,
        cols,
    );
    LocalitySpectrumCase {
        model: format!("{lattice} {rows}x{cols} h={field:.2}"),
        n,
        field,
        seed,
        recovered,
        score: outcome.best.score,
        mi_nn_ratio: outcome.best.mi_nn_ratio,
    }
}


/// Phase 12 falsification AL: AA-blind recovers chain control and 2D grid/torus (mod D₄).
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AaBlind2dBoundaryResult {
    pub pass: bool,
    pub chain_recovered: bool,
    pub grid_recovered: bool,
    pub torus_recovered: bool,
    pub cases: Vec<LocalitySpectrumCase>,
}

pub fn run_aa_blind_2d_boundary() -> AaBlind2dBoundaryResult {
    let chain = blind_case("tfim", 6, 1.5, 4242);
    let grid = blind_lattice_case("grid", 3, 3, 1.5, 4242);
    let torus = blind_lattice_case("torus", 2, 2, 1.5, 4242);
    let pass = chain.recovered && grid.recovered && torus.recovered;
    AaBlind2dBoundaryResult {
        pass,
        chain_recovered: chain.recovered,
        grid_recovered: grid.recovered,
        torus_recovered: torus.recovered,
        cases: vec![chain, grid, torus],
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

    /// Phase 12 research: AA-blind scaling on chains and 2D lattices.
    #[test]
    fn scaling_probe_blind_spectrum() {
        let mut report = Vec::new();
        for n in [4usize, 5, 6, 7, 8] {
            let c = blind_case("tfim", n, 1.5, 4242);
            report.push(format!(
                "chain n={n} recovered={} score={:.3} mi_nn={:.3}",
                c.recovered, c.score, c.mi_nn_ratio
            ));
        }
        let grid = blind_lattice_case("grid", 3, 3, 1.5, 4242);
        report.push(format!(
            "grid 3x3 recovered={} score={:.3} mi_nn={:.3}",
            grid.recovered, grid.score, grid.mi_nn_ratio
        ));
        let torus = blind_lattice_case("torus", 2, 2, 1.5, 4242);
        report.push(format!(
            "torus 2x2 recovered={} score={:.3} mi_nn={:.3}",
            torus.recovered, torus.score, torus.mi_nn_ratio
        ));
        eprintln!("scaling_probe_blind_spectrum:\n{}", report.join("\n"));
        // Document-only probe: assert chain n=6 (known AA case), not full battery
        assert!(blind_case("tfim", 6, 1.5, 42).recovered);
    }

    #[test]
    fn aa_blind_2d_boundary() {
        let b = run_aa_blind_2d_boundary();
        assert!(
            b.pass,
            "AL boundary: chain={} grid={} torus={}",
            b.chain_recovered,
            b.grid_recovered,
            b.torus_recovered
        );
    }

}
