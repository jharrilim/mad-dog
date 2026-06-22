//! Emergent simultaneity surfaces — foliation bend between physical clocks.

use crate::geometry::analyze_emergent_geometry;
use crate::quantum::{expectation_z, Hamiltonian, QuantumState};
use crate::relational_time::{
    evolve_trajectory, fit_affine, physical_clock_indices, physical_time_at_uniform,
    TrajectoryPoint,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimultaneityConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub clock_a: Option<usize>,
    #[serde(default)]
    pub clock_b: Option<usize>,
    #[serde(default = "default_physical_slices")]
    pub physical_slices: usize,
    #[serde(default = "default_embed_dim")]
    pub embed_dim: usize,
    #[serde(default)]
    pub reference_site: Option<usize>,
}

fn default_physical_slices() -> usize {
    15
}

fn default_embed_dim() -> usize {
    2
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimultaneitySlice {
    pub k: usize,
    pub tau_a: f64,
    pub tau_b: f64,
    pub tau_skew: f64,
    pub emergent_x: f64,
    pub emergent_y: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimultaneityResult {
    pub sites: usize,
    pub clock_a: usize,
    pub clock_b: usize,
    pub reference_site: usize,
    pub mean_tau_skew: f64,
    pub max_tau_skew: f64,
    pub slope_a: f64,
    pub slope_b: f64,
    pub slope_delta: f64,
    pub bend_detected: bool,
    pub slices: Vec<SimultaneitySlice>,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn physical_track(trajectory: &[TrajectoryPoint], site: usize, base_z: &[f64], slices: usize) -> Vec<f64> {
    let (indices, _) = physical_clock_indices(trajectory, site, base_z, slices);
    (0..trajectory.len())
        .map(|k| physical_time_at_uniform(&indices, k))
        .collect()
}

pub fn build_simultaneity_surfaces(
    hamiltonian: &Hamiltonian,
    initial: &QuantumState,
    reference: &QuantumState,
    config: &SimultaneityConfig,
) -> SimultaneityResult {
    let sites = config.n;
    let defect_site = sites / 2;
    let clock_a = config.clock_a.unwrap_or(defect_site);
    let clock_b = config.clock_b.unwrap_or(0);
    let reference_site = config.reference_site.unwrap_or(0);
    let embed_dim = config.embed_dim;

    let trajectory = evolve_trajectory(
        hamiltonian,
        initial,
        Some(reference),
        config.dt,
        config.steps,
        6,
    );

    let base_z: Vec<f64> = (0..sites)
        .map(|q| expectation_z(initial, q))
        .collect();

    let track_a = physical_track(&trajectory, clock_a, &base_z, config.physical_slices);
    let track_b = physical_track(&trajectory, clock_b, &base_z, config.physical_slices);

    let mut slices = Vec::with_capacity(trajectory.len());
    let mut tau_skews = Vec::with_capacity(trajectory.len());
    let mut xs = Vec::with_capacity(trajectory.len());
    let mut ys = Vec::with_capacity(trajectory.len());

    for (k, point) in trajectory.iter().enumerate() {
        let report = analyze_emergent_geometry(&point.psi, 1.0, embed_dim.max(1));
        let ref_idx = reference_site.min(report.mds.coords.len().saturating_sub(1));
        let emergent_x = report
            .mds
            .coords
            .get(ref_idx)
            .and_then(|c| c.first().copied())
            .unwrap_or(0.0);
        let emergent_y = if embed_dim >= 2 {
            report
                .mds
                .coords
                .get(ref_idx)
                .and_then(|c| c.get(1).copied())
                .unwrap_or(0.0)
        } else {
            0.0
        };

        let tau_a = track_a[k];
        let tau_b = track_b[k];
        let tau_skew = (tau_a - tau_b).abs();
        tau_skews.push(tau_skew);
        xs.push(emergent_x);
        ys.push(emergent_y);
        slices.push(SimultaneitySlice {
            k,
            tau_a,
            tau_b,
            tau_skew,
            emergent_x,
            emergent_y,
        });
    }

    let max_skew = tau_skews.iter().copied().fold(0.0_f64, f64::max).max(1e-9);
    let mean_tau_skew = tau_skews.iter().sum::<f64>() / tau_skews.len() as f64 / max_skew;

    let (slope_a, _, _) = fit_affine(&track_a, &xs);
    let (slope_b, _, _) = fit_affine(&track_b, &xs);
    let slope_delta = (slope_a - slope_b).abs()
        / slope_a.abs().max(slope_b.abs()).max(1e-9);

    let bend_detected = mean_tau_skew > 0.06 && slope_delta > 0.12;

    SimultaneityResult {
        sites,
        clock_a,
        clock_b,
        reference_site,
        mean_tau_skew,
        max_tau_skew: max_skew,
        slope_a,
        slope_b,
        slope_delta,
        bend_detected,
        slices,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tfim_chain;
    use crate::quantum::QuantumState;

    #[test]
    fn simultaneity_surfaces_bend_between_defect_and_edge() {
        let n = 9;
        let model = tfim_chain(n, 1.0, 1.0);
        let mut initial = QuantumState::zero(n);
        initial.data[2 * (1 << (n / 2))] = 1.0;
        let mut reference = QuantumState::zero(n);
        reference.data[0] = 1.0;
        let result = build_simultaneity_surfaces(
            &model.hamiltonian,
            &initial,
            &reference,
            &SimultaneityConfig {
                n,
                field: 1.0,
                dt: 0.2,
                steps: 40,
                clock_a: Some(n / 2),
                clock_b: Some(0),
                physical_slices: 15,
                embed_dim: 2,
                reference_site: Some(0),
            },
        );
        assert!(result.mean_tau_skew > 0.05);
        assert!(result.bend_detected);
    }
}
