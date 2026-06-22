//! Gauge-free MI geometry stability across clock slices.

use crate::geometry::analyze_emergent_geometry;
use crate::models::tfim_chain;
use crate::quantum::{
    evolve_interval_inplace, expectation_z_all, signal_from_z, EvolveScratch, QuantumState,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryStabilityConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default = "default_xi")]
    pub xi: f64,
    #[serde(default = "default_seed")]
    pub seed: u32,
}

fn default_xi() -> f64 {
    1.0
}

fn default_seed() -> u32 {
    42
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
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeometryStabilityResult {
    pub sites: usize,
    pub field: f64,
    pub slices: Vec<GeometryStabilitySlice>,
    pub mean_distance_drift: f64,
    pub mean_rank_correlation: f64,
    pub dim_std: f64,
    pub geometry_stable: bool,
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

fn upper_triangle_flat(dist: &[Vec<f64>]) -> Vec<f64> {
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

fn spearman_corr(a: &[f64], b: &[f64]) -> f64 {
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

fn analyze_slice(
    state: &QuantumState,
    xi: f64,
    prev_distance: Option<&[Vec<f64>]>,
) -> (Vec<Vec<f64>>, usize, Vec<f64>, f64, f64) {
    let report = analyze_emergent_geometry(state, xi, 3);
    let dim = report.mds.emergent_dim;
    let top: Vec<f64> = report
        .mds
        .eigenvalues
        .iter()
        .copied()
        .filter(|v| *v > 1e-9)
        .take(3)
        .collect();
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
    (report.distance, dim, top, drift, rank_corr)
}

pub fn run_geometry_stability(config: &GeometryStabilityConfig) -> GeometryStabilityResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let center = config.n / 2;
    let initial = defect_initial(config.n, center);
    let reference = reference_initial(config.n);
    let ref_z = expectation_z_all(&reference);

    let mut rng = Rng::new(config.seed);
    let radius = model.hamiltonian.spectral_radius(&mut rng);
    let mut psi = initial;
    let mut scratch = EvolveScratch::new(psi.dim);

    let mut slices = Vec::with_capacity(config.steps);
    let mut prev_distance: Option<Vec<Vec<f64>>> = None;
    let mut drift_sum = 0.0;
    let mut corr_sum = 0.0;
    let mut drift_count = 0usize;
    let mut dims = Vec::with_capacity(config.steps);

    for k in 0..config.steps {
        let t = k as f64 * config.dt;
        let (distance, dim, top_eigenvalues, distance_drift, rank_correlation) =
            analyze_slice(&psi, config.xi, prev_distance.as_deref());
        dims.push(dim as f64);

        if k > 0 {
            drift_sum += distance_drift;
            corr_sum += rank_correlation;
            drift_count += 1;
        }

        slices.push(GeometryStabilitySlice {
            k,
            t,
            emergent_dim: dim,
            top_eigenvalues,
            distance_drift,
            rank_correlation,
        });

        prev_distance = Some(distance);

        let _ = signal_from_z(&expectation_z_all(&psi), &ref_z);
        evolve_interval_inplace(
            &model.hamiltonian,
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
    let dim_mean = dims.iter().sum::<f64>() / dims.len().max(1) as f64;
    let dim_std = (dims
        .iter()
        .map(|d| (d - dim_mean).powi(2))
        .sum::<f64>()
        / dims.len().max(1) as f64)
        .sqrt();

    let geometry_stable = mean_rank_correlation > 0.85;

    GeometryStabilityResult {
        sites: config.n,
        field: config.field,
        slices,
        mean_distance_drift,
        mean_rank_correlation,
        dim_std,
        geometry_stable,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ordered_phase_geometry_stable() {
        let config = GeometryStabilityConfig {
            n: 10,
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
    fn paramagnetic_geometry_less_stable() {
        let ordered = run_geometry_stability(&GeometryStabilityConfig {
            n: 10,
            field: 1.2,
            dt: 0.2,
            steps: 20,
            xi: 1.0,
            seed: 42,
        });
        let paramag = run_geometry_stability(&GeometryStabilityConfig {
            n: 10,
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
