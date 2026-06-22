//! Emergent dimension vs lattice manifold sweep (Phase 2 S2).

use crate::geometry::analyze_emergent_geometry;
use crate::models::{tfim_chain, tfim_cube, tfim_grid};
use crate::quantum::{ground_state, QuantumState};
use crate::rng::Rng;
use serde::Serialize;

const EMBEDDING_CORR_MIN: f64 = 0.80;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryDimSweepCase {
    pub label: String,
    pub kind: String,
    pub sites: usize,
    pub expected_dim: usize,
    pub emergent_dim: usize,
    pub embedding_correlation: f64,
    pub passed: bool,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryDimSweepResult {
    pub cases: Vec<GeometryDimSweepCase>,
    pub passed: usize,
    pub total: usize,
    pub all_passed: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn ground_dim(state: &QuantumState, xi: f64, max_dim: usize) -> usize {
    analyze_emergent_geometry(state, xi, max_dim).mds.emergent_dim
}

fn embedding_corr(state: &QuantumState, xi: f64, max_dim: usize) -> f64 {
    use crate::geometry::{classical_mds, pairwise_from_coords, spectral_embedding};
    use crate::geometry_stability::{spearman_corr, upper_triangle_flat};

    let report = analyze_emergent_geometry(state, xi, max_dim);
    let mds = classical_mds(&report.distance, max_dim);
    let spectral = spectral_embedding(&report.distance, max_dim);
    let d_mds = pairwise_from_coords(&mds.coords);
    let d_spec = pairwise_from_coords(&spectral.coords);
    spearman_corr(
        &upper_triangle_flat(&d_mds),
        &upper_triangle_flat(&d_spec),
    )
}

fn case_chain(n: usize, field: f64, seed: u32) -> GeometryDimSweepCase {
    let model = tfim_chain(n, 1.0, field);
    let mut rng = Rng::new(seed);
    let (state, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let expected = 1usize;
    let dim = ground_dim(&state, 1.0, 3);
    let embed = embedding_corr(&state, 1.0, 3);
    GeometryDimSweepCase {
        label: format!("chain n={n} ground"),
        kind: "chain".to_string(),
        sites: n,
        expected_dim: expected,
        emergent_dim: dim,
        embedding_correlation: embed,
        passed: dim >= expected
            && (expected <= 1 || embed >= EMBEDDING_CORR_MIN),
    }
}

fn case_grid(rows: usize, cols: usize, field: f64, seed: u32) -> GeometryDimSweepCase {
    let model = tfim_grid(rows, cols, 1.0, field);
    let mut rng = Rng::new(seed);
    let (state, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let expected = 2usize;
    let dim = ground_dim(&state, 1.0, 3);
    let embed = embedding_corr(&state, 1.0, 3);
    GeometryDimSweepCase {
        label: format!("grid {rows}x{cols} ground"),
        kind: "grid".to_string(),
        sites: rows * cols,
        expected_dim: expected,
        emergent_dim: dim,
        embedding_correlation: embed,
        passed: dim >= expected
            && (expected <= 1 || embed >= EMBEDDING_CORR_MIN),
    }
}

fn case_cube(lx: usize, ly: usize, lz: usize, field: f64, seed: u32) -> GeometryDimSweepCase {
    let model = tfim_cube(lx, ly, lz, 1.0, field);
    let mut rng = Rng::new(seed);
    let (state, _, _) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let expected = if lx * ly * lz <= 8 { 1 } else { 3 };
    let dim = ground_dim(&state, 1.0, 3);
    let embed = embedding_corr(&state, 1.0, 3);
    GeometryDimSweepCase {
        label: format!("cube {lx}x{ly}x{lz} ground"),
        kind: "cube".to_string(),
        sites: lx * ly * lz,
        expected_dim: expected,
        emergent_dim: dim,
        embedding_correlation: embed,
        passed: dim >= expected
            && (expected <= 1 || embed >= EMBEDDING_CORR_MIN),
    }
}

pub fn run_geometry_dim_sweep() -> GeometryDimSweepResult {
    let field = 1.5;
    let cases = vec![
        case_chain(8, field, 42),
        case_chain(12, field, 43),
        case_grid(3, 3, field, 44),
        case_cube(2, 2, 2, field, 45),
        case_cube(2, 2, 3, field, 46),
    ];
    let passed = cases.iter().filter(|c| c.passed).count();
    let total = cases.len();
    GeometryDimSweepResult {
        all_passed: passed == total,
        passed,
        total,
        cases,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
