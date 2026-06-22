//! Modular-flow dual clocks — port of `src/sim/modular-time.ts`.

use crate::geometry::entropy_of_region;
use crate::quantum::{expectation_z, QuantumState};
use crate::relational_time::{evolve_trajectory, fit_affine, TrajectoryPoint};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModularDualClockConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default = "default_modular_slices")]
    pub modular_slices: usize,
}

fn default_modular_slices() -> usize {
    10
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModularDualClockResult {
    pub n: usize,
    pub center: usize,
    pub region_a: Vec<usize>,
    pub region_b: Vec<usize>,
    pub modular_tau_a: Vec<f64>,
    pub modular_tau_b: Vec<f64>,
    pub tick_a: Vec<usize>,
    pub tick_b: Vec<usize>,
    pub sync_r2_modular: f64,
    pub sync_r2_z: f64,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn cumulative_modular_time(trajectory: &[TrajectoryPoint], region: &[usize]) -> Vec<f64> {
    let mut tau = vec![0.0];
    let mut prev_s = entropy_of_region(&trajectory[0].psi, region);
    for k in 1..trajectory.len() {
        let next_s = entropy_of_region(&trajectory[k].psi, region);
        tau.push(tau[k - 1] + (next_s - prev_s).abs());
        prev_s = next_s;
    }
    tau
}

fn event_ticks_from_cumulative(cumulative: &[f64], num_slices: usize) -> Vec<usize> {
    let max = *cumulative.last().unwrap_or(&0.0);
    if max < 1e-12 {
        return (0..cumulative.len()).collect();
    }
    let targets: Vec<f64> = (0..num_slices)
        .map(|i| max * (i as f64 + 0.5) / num_slices as f64)
        .collect();
    let mut ticks = vec![0usize];
    let mut search_from = 1usize;
    for target in targets {
        let mut found = None;
        for i in search_from..cumulative.len() {
            if cumulative[i] >= target {
                found = Some(i);
                break;
            }
        }
        let Some(idx) = found else { break };
        ticks.push(idx);
        search_from = idx + 1;
    }
    if ticks.len() < 2 && cumulative.len() > 1 {
        ticks.push(cumulative.len() - 1);
    }
    ticks
}

fn z_threshold_ticks(
    trajectory: &[TrajectoryPoint],
    clock_site: usize,
    base_z: &[f64],
    num_slices: usize,
) -> Vec<usize> {
    let signals: Vec<f64> = trajectory
        .iter()
        .map(|p| {
            let z = expectation_z(&p.psi, clock_site);
            let ref_z = p
                .reference
                .as_ref()
                .map(|r| expectation_z(r, clock_site))
                .unwrap_or(base_z[clock_site]);
            (z - ref_z).abs()
        })
        .collect();
    let max_sig = signals.iter().copied().fold(0.0_f64, f64::max).max(1e-9);
    let targets: Vec<f64> = (0..num_slices)
        .map(|i| max_sig * (i as f64 + 0.5) / num_slices as f64)
        .collect();
    let mut ticks = vec![0usize];
    let mut search_from = 1usize;
    for target in targets {
        let mut found = None;
        for i in search_from..signals.len() {
            if signals[i] >= target {
                found = Some(i);
                break;
            }
        }
        let Some(idx) = found else { break };
        ticks.push(idx);
        search_from = idx + 1;
    }
    if ticks.len() < 2 && trajectory.len() > 1 {
        ticks.push(trajectory.len() - 1);
    }
    ticks
}

fn sync_r2_from_ticks(a: &[usize], b: &[usize]) -> f64 {
    let n = a.len().min(b.len());
    if n < 2 {
        return 1.0;
    }
    let xs: Vec<f64> = a[..n].iter().map(|&x| x as f64).collect();
    let ys: Vec<f64> = b[..n].iter().map(|&y| y as f64).collect();
    fit_affine(&xs, &ys).2
}

pub fn build_modular_dual_clock(
    hamiltonian: &crate::quantum::Hamiltonian,
    config: &ModularDualClockConfig,
    initial: QuantumState,
    reference: QuantumState,
) -> ModularDualClockResult {
    let center = config.n / 2;
    let region_a = vec![center];
    let region_b = vec![(center + 2).min(config.n - 1)];

    let base_z: Vec<f64> = (0..config.n)
        .map(|q| expectation_z(&initial, q))
        .collect();

    let trajectory = evolve_trajectory(
        hamiltonian,
        &initial,
        Some(&reference),
        config.dt,
        config.steps,
        6,
    );

    let modular_tau_a = cumulative_modular_time(&trajectory, &region_a);
    let modular_tau_b = cumulative_modular_time(&trajectory, &region_b);
    let tick_a = event_ticks_from_cumulative(&modular_tau_a, config.modular_slices);
    let tick_b = event_ticks_from_cumulative(&modular_tau_b, config.modular_slices);
    let sync_r2_modular = sync_r2_from_ticks(&tick_a, &tick_b);

    let z_a = z_threshold_ticks(&trajectory, region_a[0], &base_z, config.modular_slices);
    let z_b = z_threshold_ticks(&trajectory, region_b[0], &base_z, config.modular_slices);
    let sync_r2_z = sync_r2_from_ticks(&z_a, &z_b);

    ModularDualClockResult {
        n: config.n,
        center,
        region_a,
        region_b,
        modular_tau_a,
        modular_tau_b,
        tick_a,
        tick_b,
        sync_r2_modular,
        sync_r2_z,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
