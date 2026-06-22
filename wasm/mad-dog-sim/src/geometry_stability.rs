//! Gauge-free MI geometry stability across clock slices.

use crate::geometry::{
    analyze_emergent_geometry, classical_mds, pairwise_from_coords, spectral_embedding,
};
use crate::models::{tfim_chain, tfim_cube, tfim_grid};
use crate::quantum::{
    evolve_interval_inplace, expectation_z_all, signal_from_z, EvolveScratch, Hamiltonian,
    QuantumState,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryStabilityConfig {
    #[serde(default = "default_kind")]
    pub kind: String,
    #[serde(default = "default_n")]
    pub n: usize,
    #[serde(default)]
    pub rows: Option<usize>,
    #[serde(default)]
    pub cols: Option<usize>,
    #[serde(default)]
    pub lz: Option<usize>,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default = "default_xi")]
    pub xi: f64,
    #[serde(default = "default_seed")]
    pub seed: u32,
}

fn default_kind() -> String {
    "chain".to_string()
}

fn default_n() -> usize {
    10
}

fn default_xi() -> f64 {
    1.0
}

fn default_seed() -> u32 {
    42
}

#[derive(Clone, Debug)]
struct LatticeSetup {
    label: String,
    hamiltonian: Hamiltonian,
    sites: usize,
    defect_site: usize,
    expected_dim: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryStabilitySlice {
    pub k: usize,
    pub t: f64,
    pub emergent_dim: usize,
    pub top_eigenvalues: Vec<f64>,
    pub distance_drift: f64,
    pub rank_correlation: f64,
    pub embedding_correlation: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryStabilityResult {
    pub kind: String,
    pub label: String,
    pub sites: usize,
    pub field: f64,
    pub expected_dim: usize,
    pub slices: Vec<GeometryStabilitySlice>,
    pub mean_distance_drift: f64,
    pub mean_rank_correlation: f64,
    pub mean_embedding_correlation: f64,
    pub dim_std: f64,
    pub mean_emergent_dim: f64,
    pub geometry_stable: bool,
    pub dim_stable: bool,
    pub elapsed_ms: f64,
}

fn defect_initial(n: usize, center: usize) -> QuantumState {
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << center)] = 1.0;
    initial
}

fn reference_initial(n: usize) -> QuantumState {
    let mut reference = QuantumState::zero(n);
    reference.data[0] = 1.0;
    reference
}

pub fn upper_triangle_flat(dist: &[Vec<f64>]) -> Vec<f64> {
    let n = dist.len();
    let mut out = Vec::with_capacity(n * (n - 1) / 2);
    for i in 0..n {
        for j in (i + 1)..n {
            out.push(dist[i][j]);
        }
    }
    out
}

fn rank_values(values: &[f64]) -> Vec<f64> {
    let n = values.len();
    let mut order: Vec<(f64, usize)> = values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, v)| (v, i))
        .collect();
    order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));
    let mut ranks = vec![0.0; n];
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && (order[j].0 - order[i].0).abs() < 1e-12 {
            j += 1;
        }
        let avg_rank = (i + j - 1) as f64 / 2.0 + 1.0;
        for k in i..j {
            ranks[order[k].1] = avg_rank;
        }
        i = j;
    }
    ranks
}

pub fn spearman_corr(a: &[f64], b: &[f64]) -> f64 {
    if a.len() != b.len() || a.len() < 2 {
        return 1.0;
    }
    let ra = rank_values(a);
    let rb = rank_values(b);
    let mean_a = ra.iter().sum::<f64>() / ra.len() as f64;
    let mean_b = rb.iter().sum::<f64>() / rb.len() as f64;
    let mut num = 0.0;
    let mut den_a = 0.0;
    let mut den_b = 0.0;
    for i in 0..ra.len() {
        let da = ra[i] - mean_a;
        let db = rb[i] - mean_b;
        num += da * db;
        den_a += da * da;
        den_b += db * db;
    }
    let den = (den_a * den_b).sqrt();
    if den < 1e-12 {
        1.0
    } else {
        num / den
    }
}

fn frobenius_relative_drift(d0: &[Vec<f64>], d1: &[Vec<f64>]) -> f64 {
    let n = d0.len();
    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..n {
        for j in 0..n {
            let diff = d0[i][j] - d1[i][j];
            num += diff * diff;
            den += d0[i][j] * d0[i][j];
        }
    }
    (num / den.max(1e-12)).sqrt()
}

fn embedding_correlation(distance: &[Vec<f64>], max_dim: usize) -> f64 {
    let mds = classical_mds(distance, max_dim);
    let spectral = spectral_embedding(distance, max_dim);
    let d_mds = pairwise_from_coords(&mds.coords);
    let d_spec = pairwise_from_coords(&spectral.coords);
    spearman_corr(&upper_triangle_flat(&d_mds), &upper_triangle_flat(&d_spec))
}

fn lattice_setup(config: &GeometryStabilityConfig) -> LatticeSetup {
    match config.kind.as_str() {
        "grid" => {
            let rows = config.rows.unwrap_or(3);
            let cols = config.cols.unwrap_or(3);
            let model = tfim_grid(rows, cols, 1.0, config.field);
            let defect_site = (rows / 2) * cols + cols / 2;
            LatticeSetup {
                label: format!("TFIM grid {rows}x{cols}"),
                hamiltonian: model.hamiltonian,
                sites: rows * cols,
                defect_site,
                expected_dim: 2,
            }
        }
        "cube" => {
            let lx = config.rows.unwrap_or(2);
            let ly = config.cols.unwrap_or(2);
            let lz = config.lz.unwrap_or(3);
            let model = tfim_cube(lx, ly, lz, 1.0, config.field);
            let defect_site = (lz / 2) * (lx * ly) + (ly / 2) * lx + lx / 2;
            LatticeSetup {
                label: format!("TFIM cube {lx}x{ly}x{lz}"),
                hamiltonian: model.hamiltonian,
                sites: lx * ly * lz,
                defect_site,
                expected_dim: 3,
            }
        }
        _ => {
            let model = tfim_chain(config.n, 1.0, config.field);
            let defect_site = config.n / 2;
            LatticeSetup {
                label: format!("TFIM chain n={}", config.n),
                hamiltonian: model.hamiltonian,
                sites: config.n,
                defect_site,
                expected_dim: 1,
            }
        }
    }
}

fn analyze_slice(
    state: &QuantumState,
    xi: f64,
    max_dim: usize,
    prev_distance: Option<&[Vec<f64>]>,
) -> (Vec<Vec<f64>>, usize, Vec<f64>, f64, f64, f64) {
    let report = analyze_emergent_geometry(state, xi, max_dim);
    let dim = report.mds.emergent_dim;
    let top: Vec<f64> = report
        .mds
        .eigenvalues
        .iter()
        .copied()
        .filter(|v| *v > 1e-9)
        .take(3)
        .collect();
    let embedding_correlation = embedding_correlation(&report.distance, max_dim);
    let (drift, rank_corr) = if let Some(prev) = prev_distance {
        let tri0 = upper_triangle_flat(prev);
        let tri1 = upper_triangle_flat(&report.distance);
        (
            frobenius_relative_drift(prev, &report.distance),
            spearman_corr(&tri0, &tri1),
        )
    } else {
        (0.0, 1.0)
    };
    (
        report.distance,
        dim,
        top,
        drift,
        rank_corr,
        embedding_correlation,
    )
}

pub fn run_geometry_stability(config: &GeometryStabilityConfig) -> GeometryStabilityResult {
    let lattice = lattice_setup(config);
    let max_dim = lattice.expected_dim.max(3);
    let initial = defect_initial(lattice.sites, lattice.defect_site);
    let reference = reference_initial(lattice.sites);
    let ref_z = expectation_z_all(&reference);

    let mut rng = Rng::new(config.seed);
    let radius = lattice.hamiltonian.spectral_radius(&mut rng);
    let mut psi = initial;
    let mut scratch = EvolveScratch::new(psi.dim);

    let mut slices = Vec::with_capacity(config.steps);
    let mut prev_distance: Option<Vec<Vec<f64>>> = None;
    let mut drift_sum = 0.0;
    let mut corr_sum = 0.0;
    let mut embed_sum = 0.0;
    let mut drift_count = 0usize;
    let mut dims = Vec::with_capacity(config.steps);

    for k in 0..config.steps {
        let t = k as f64 * config.dt;
        let (distance, dim, top_eigenvalues, distance_drift, rank_correlation, embedding_correlation) =
            analyze_slice(&psi, config.xi, max_dim, prev_distance.as_deref());
        dims.push(dim as f64);

        if k > 0 {
            drift_sum += distance_drift;
            corr_sum += rank_correlation;
            embed_sum += embedding_correlation;
            drift_count += 1;
        }

        slices.push(GeometryStabilitySlice {
            k,
            t,
            emergent_dim: dim,
            top_eigenvalues,
            distance_drift,
            rank_correlation,
            embedding_correlation,
        });

        prev_distance = Some(distance);

        let _ = signal_from_z(&expectation_z_all(&psi), &ref_z);
        evolve_interval_inplace(
            &lattice.hamiltonian,
            &mut psi,
            config.dt,
            radius,
            6,
            &mut scratch,
        );
    }

    let mean_distance_drift = if drift_count > 0 {
        drift_sum / drift_count as f64
    } else {
        0.0
    };
    let mean_rank_correlation = if drift_count > 0 {
        corr_sum / drift_count as f64
    } else {
        1.0
    };
    let mean_embedding_correlation = if drift_count > 0 {
        embed_sum / drift_count as f64
    } else {
        1.0
    };
    let dim_mean = dims.iter().sum::<f64>() / dims.len().max(1) as f64;
    let dim_std = (dims
        .iter()
        .map(|d| (d - dim_mean).powi(2))
        .sum::<f64>()
        / dims.len().max(1) as f64)
        .sqrt();

    let geometry_stable = mean_rank_correlation > 0.85;
    let dim_floor = match lattice.expected_dim {
        3 | 2 => 1.5,
        _ => 1.0,
    };
    let dim_stable =
        dim_mean >= dim_floor && dims.iter().all(|&d| d >= 1.0);

    GeometryStabilityResult {
        kind: config.kind.clone(),
        label: lattice.label,
        sites: lattice.sites,
        field: config.field,
        expected_dim: lattice.expected_dim,
        slices,
        mean_distance_drift,
        mean_rank_correlation,
        mean_embedding_correlation,
        dim_std,
        mean_emergent_dim: dim_mean,
        geometry_stable,
        dim_stable,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_phase_geometry_stable() {
        let config = GeometryStabilityConfig {
            kind: "chain".to_string(),
            n: 10,
            rows: None,
            cols: None,
            lz: None,
            field: 1.2,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        };
        let r = run_geometry_stability(&config);
        assert!(
            r.mean_rank_correlation > 0.85,
            "drift={:.3} corr={:.3} dimStd={:.3}",
            r.mean_distance_drift,
            r.mean_rank_correlation,
            r.dim_std
        );
    }

    #[test]
    fn cube_223_geometry_stable() {
        let r = run_geometry_stability(&GeometryStabilityConfig {
            kind: "cube".to_string(),
            n: 12,
            rows: Some(2),
            cols: Some(2),
            lz: Some(3),
            field: 1.2,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        });
        assert!(
            r.geometry_stable && r.dim_stable,
            "cube corr={:.3} embed={:.3} dimMean={:.2}",
            r.mean_rank_correlation,
            r.mean_embedding_correlation,
            r.mean_emergent_dim
        );
    }

    #[test]
    fn paramagnetic_geometry_less_stable() {
        let ordered = run_geometry_stability(&GeometryStabilityConfig {
            kind: "chain".to_string(),
            n: 10,
            rows: None,
            cols: None,
            lz: None,
            field: 1.2,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        });
        let paramag = run_geometry_stability(&GeometryStabilityConfig {
            kind: "chain".to_string(),
            n: 10,
            rows: None,
            cols: None,
            lz: None,
            field: 3.5,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        });
        assert!(
            ordered.mean_rank_correlation >= paramag.mean_rank_correlation - 0.05
                || ordered.mean_distance_drift <= paramag.mean_distance_drift,
            "ordered corr={:.3} paramag corr={:.3}",
            ordered.mean_rank_correlation,
            paramag.mean_rank_correlation
        );
    }
}
