//! Dispersion relation — wavepacket propagation on TFIM chain.

use crate::models::tfim_chain;
use crate::quantum::{expectation_z, Hamiltonian, QuantumState};
use crate::relational_time::{evolve_trajectory, fit_affine, TrajectoryPoint};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispersionConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default = "default_modes")]
    pub modes: usize,
}

fn default_modes() -> usize {
    3
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispersionMode {
    pub k: f64,
    pub group_velocity: f64,
    pub omega: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DispersionResult {
    pub n: usize,
    pub field: f64,
    pub modes: Vec<DispersionMode>,
    pub omega_slope: f64,
    pub omega_intercept: f64,
    pub linear_r2: f64,
    pub linear_at_small_k: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn wavepacket_initial(n: usize, k: f64) -> QuantumState {
    let mut psi = QuantumState::zero(n);
    for j in 0..n {
        let amp = (k * j as f64).cos();
        if amp.abs() > 1e-9 {
            psi.data[2 * (1 << j)] += amp;
        }
    }
    psi.normalize();
    psi
}

fn signal_com(signal: &[f64]) -> f64 {
    let sum: f64 = signal.iter().sum();
    if sum < 1e-9 {
        return 0.0;
    }
    signal
        .iter()
        .enumerate()
        .map(|(j, &s)| j as f64 * s)
        .sum::<f64>()
        / sum
}

fn com_velocity(
    trajectory: &[TrajectoryPoint],
    base_z: &[f64],
) -> f64 {
    let mut ts = Vec::new();
    let mut coms = Vec::new();
    for point in trajectory {
        let z: Vec<f64> = (0..point.psi.n)
            .map(|q| expectation_z(&point.psi, q))
            .collect();
        let ref_z: Vec<f64> = point
            .reference
            .as_ref()
            .map(|r| (0..r.n).map(|q| expectation_z(r, q)).collect())
            .unwrap_or_else(|| base_z.to_vec());
        let signal: Vec<f64> = z
            .iter()
            .zip(ref_z.iter())
            .map(|(z, rz)| (z - rz).abs())
            .collect();
        ts.push(point.t);
        coms.push(signal_com(&signal));
    }
    if ts.len() < 2 {
        return 0.0;
    }
    fit_affine(&ts, &coms).0
}

pub fn build_dispersion(
    hamiltonian: &Hamiltonian,
    config: &DispersionConfig,
) -> DispersionResult {
    let n = config.n;
    let mut reference = QuantumState::zero(n);
    reference.data[0] = 1.0;
    let base_z: Vec<f64> = (0..n).map(|q| expectation_z(&reference, q)).collect();

    let mut modes = Vec::new();
    for m in 1..=config.modes {
        let k = std::f64::consts::TAU * m as f64 / n as f64;
        let initial = wavepacket_initial(n, k);
        let trajectory = evolve_trajectory(
            hamiltonian,
            &initial,
            Some(&reference),
            config.dt,
            config.steps,
            6,
        );
        let v_g = com_velocity(&trajectory, &base_z);
        let omega = v_g.abs() * k;
        modes.push(DispersionMode {
            k,
            group_velocity: v_g.abs(),
            omega,
        });
    }

    let ks: Vec<f64> = modes.iter().map(|m| m.k).collect();
    let omegas: Vec<f64> = modes.iter().map(|m| m.omega).collect();
    let (slope, intercept, r2) = fit_affine(&ks, &omegas);
    let v_mean = modes.iter().map(|m| m.group_velocity).sum::<f64>() / modes.len().max(1) as f64;
    let v_cov = if modes.len() >= 2 && v_mean > 1e-9 {
        let var = modes
            .iter()
            .map(|m| (m.group_velocity - v_mean).powi(2))
            .sum::<f64>()
            / modes.len() as f64;
        var.sqrt() / v_mean
    } else {
        f64::INFINITY
    };
    let linear_at_small_k =
        v_mean > 0.01 && v_cov < 0.35 && slope > 0.0 && r2 > 0.85 && intercept.abs() < slope * 0.35;

    DispersionResult {
        n,
        field: config.field,
        modes,
        omega_slope: slope,
        omega_intercept: intercept,
        linear_r2: r2,
        linear_at_small_k,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

pub fn run_dispersion(config: &DispersionConfig) -> DispersionResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    build_dispersion(&model.hamiltonian, config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dispersion_produces_modes() {
        let result = run_dispersion(&DispersionConfig {
            n: 10,
            field: 1.0,
            dt: 0.2,
            steps: 24,
            modes: 2,
        });
        assert_eq!(result.modes.len(), 2);
        assert!(result.omega_slope.is_finite());
    }
}
