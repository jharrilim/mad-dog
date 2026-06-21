//! Two-clock / relational time — port of `src/sim/relational-time.ts`.

use crate::geometry::analyze_emergent_geometry;
use crate::quantum::{evolve_interval, expectation_z, Hamiltonian, QuantumState};
use crate::rng::Rng;
use crate::spacetime::{SpacetimeResult, SpacetimeSlice};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug)]
pub struct TrajectoryPoint {
    pub t: f64,
    pub psi: QuantumState,
    pub reference: Option<QuantumState>,
}

#[derive(Clone, Debug)]
pub struct DualClockConfig<'a> {
    pub hamiltonian: &'a Hamiltonian,
    pub initial: QuantumState,
    pub reference: Option<QuantumState>,
    pub dt: f64,
    pub steps: usize,
    pub clock_site: usize,
    pub physical_slices: usize,
    pub embed_dim: usize,
    pub order: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ClockHistory {
    pub label: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub clock_site: Option<usize>,
    pub result: SpacetimeResult,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TimeMapPoint {
    pub tau_uniform: f64,
    pub tau_physical: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DualClockResult {
    pub sites: usize,
    pub uniform: ClockHistory,
    pub physical: ClockHistory,
    pub time_map: Vec<TimeMapPoint>,
    pub sync_r2: f64,
    pub sync_slope: f64,
    pub sync_intercept: f64,
    pub elapsed_ms: f64,
}

fn orient_coords(coords: &[f64]) -> Vec<f64> {
    let n = coords.len();
    if n < 2 {
        return coords.to_vec();
    }
    let mean_i = (n - 1) as f64 / 2.0;
    let mean_c = coords.iter().sum::<f64>() / n as f64;
    let mut cov = 0.0;
    for (i, &c) in coords.iter().enumerate() {
        cov += (i as f64 - mean_i) * (c - mean_c);
    }
    if cov < 0.0 {
        coords.iter().map(|c| -c).collect()
    } else {
        coords.to_vec()
    }
}

fn build_slice(
    hamiltonian: &Hamiltonian,
    psi: &QuantumState,
    reference: Option<&QuantumState>,
    base_z: &[f64],
    t: f64,
    k: usize,
    embed_dim: usize,
) -> SpacetimeSlice {
    let sites = hamiltonian.n;
    let energy = hamiltonian.expectation(psi);
    let report = analyze_emergent_geometry(psi, 1.0, embed_dim.max(1));
    let coords = if embed_dim >= 2 {
        report
            .mds
            .coords
            .iter()
            .map(|c| vec![c.first().copied().unwrap_or(0.0), c.get(1).copied().unwrap_or(0.0)])
            .collect()
    } else {
        let raw: Vec<f64> = report
            .mds
            .coords
            .iter()
            .map(|c| c.first().copied().unwrap_or(0.0))
            .collect();
        orient_coords(&raw).into_iter().map(|x| vec![x]).collect()
    };
    let z_expectation: Vec<f64> = (0..sites).map(|q| expectation_z(psi, q)).collect();
    let ref_z: Vec<f64> = if let Some(r) = reference {
        (0..sites).map(|q| expectation_z(r, q)).collect()
    } else {
        base_z.to_vec()
    };
    let signal: Vec<f64> = z_expectation
        .iter()
        .zip(ref_z.iter())
        .map(|(z, rz)| (z - rz).abs())
        .collect();
    SpacetimeSlice {
        k,
        t,
        coords,
        z_expectation,
        signal,
        energy,
    }
}

pub fn evolve_trajectory(
    hamiltonian: &Hamiltonian,
    initial: &QuantumState,
    reference: Option<&QuantumState>,
    dt: f64,
    steps: usize,
    order: usize,
) -> Vec<TrajectoryPoint> {
    let mut rng = Rng::new(42);
    let radius = hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let mut points = Vec::with_capacity(steps);
    let mut psi = initial.clone_state();
    let mut ref_state = reference.cloned();
    for k in 0..steps {
        points.push(TrajectoryPoint {
            t: k as f64 * dt,
            psi: psi.clone_state(),
            reference: ref_state.as_ref().map(|r| r.clone_state()),
        });
        psi = evolve_interval(hamiltonian, &psi, dt, radius, order);
        psi.normalize();
        if let Some(ref mut r) = ref_state {
            *r = evolve_interval(hamiltonian, r, dt, radius, order);
            r.normalize();
        }
    }
    points
}

fn signal_at_site(point: &TrajectoryPoint, site: usize, base_z: &[f64]) -> f64 {
    let z = expectation_z(&point.psi, site);
    let ref_z = point
        .reference
        .as_ref()
        .map(|r| expectation_z(r, site))
        .unwrap_or(base_z[site]);
    (z - ref_z).abs()
}

fn slices_from_indices(
    hamiltonian: &Hamiltonian,
    trajectory: &[TrajectoryPoint],
    indices: &[usize],
    times: &[f64],
    initial: &QuantumState,
    embed_dim: usize,
) -> Vec<SpacetimeSlice> {
    let base_z: Vec<f64> = (0..hamiltonian.n)
        .map(|q| expectation_z(initial, q))
        .collect();
    indices
        .iter()
        .zip(times.iter())
        .enumerate()
        .map(|(k, (&idx, &t))| {
            let point = &trajectory[idx];
            build_slice(
                hamiltonian,
                &point.psi,
                point.reference.as_ref(),
                &base_z,
                t,
                k,
                embed_dim,
            )
        })
        .collect()
}

fn physical_clock_indices(
    trajectory: &[TrajectoryPoint],
    clock_site: usize,
    base_z: &[f64],
    num_slices: usize,
) -> (Vec<usize>, Vec<f64>) {
    let signals: Vec<f64> = trajectory
        .iter()
        .map(|p| signal_at_site(p, clock_site, base_z))
        .collect();
    let max_sig = signals.iter().copied().fold(1e-9_f64, f64::max);
    let targets: Vec<f64> = (0..num_slices)
        .map(|i| max_sig * (i as f64 + 0.5) / num_slices as f64)
        .collect();

    let mut indices = vec![0];
    let mut times = vec![0.0];
    let mut search_from = 1;
    for target in targets {
        let mut found = None;
        for i in search_from..trajectory.len() {
            if signals[i] >= target {
                found = Some(i);
                break;
            }
        }
        let Some(i) = found else { break };
        indices.push(i);
        times.push(trajectory[i].t);
        search_from = i + 1;
    }

    if indices.len() < 2 && trajectory.len() > 1 {
        let last = trajectory.len() - 1;
        indices.push(last);
        times.push(trajectory[last].t);
    }
    (indices, times)
}

pub fn fit_affine(xs: &[f64], ys: &[f64]) -> (f64, f64, f64) {
    if xs.len() < 2 {
        return (1.0, 0.0, 1.0);
    }
    let n = xs.len() as f64;
    let mut sxx = 0.0;
    let mut sxy = 0.0;
    let mut sx = 0.0;
    let mut sy = 0.0;
    for i in 0..xs.len() {
        sxx += xs[i] * xs[i];
        sxy += xs[i] * ys[i];
        sx += xs[i];
        sy += ys[i];
    }
    let denom = n * sxx - sx * sx;
    let slope = if denom != 0.0 {
        (n * sxy - sx * sy) / denom
    } else {
        1.0
    };
    let intercept = (sy - slope * sx) / n;
    let y_bar = sy / n;
    let mut ss_res = 0.0;
    let mut ss_tot = 0.0;
    for i in 0..xs.len() {
        let pred = slope * xs[i] + intercept;
        ss_res += (ys[i] - pred).powi(2);
        ss_tot += (ys[i] - y_bar).powi(2);
    }
    let r2 = if ss_tot > 0.0 { 1.0 - ss_res / ss_tot } else { 1.0 };
    (slope, intercept, r2)
}

pub fn build_dual_clock(config: DualClockConfig<'_>) -> DualClockResult {
    let hamiltonian = config.hamiltonian;
    let sites = hamiltonian.n;
    let embed_dim = config.embed_dim;
    let order = config.order;

    let trajectory = evolve_trajectory(
        hamiltonian,
        &config.initial,
        config.reference.as_ref(),
        config.dt,
        config.steps,
        order,
    );

    let base_z: Vec<f64> = (0..sites)
        .map(|q| expectation_z(&config.initial, q))
        .collect();
    let energy0 = hamiltonian.expectation(&config.initial);

    let uniform_indices: Vec<usize> = (0..trajectory.len()).collect();
    let uniform_times: Vec<f64> = trajectory.iter().map(|p| p.t).collect();
    let uniform_slice_list = slices_from_indices(
        hamiltonian,
        &trajectory,
        &uniform_indices,
        &uniform_times,
        &config.initial,
        embed_dim,
    );
    let mut energy_drift = 0.0_f64;
    for s in &uniform_slice_list {
        energy_drift = energy_drift.max((s.energy - energy0).abs());
    }

    let (phys_indices, _) =
        physical_clock_indices(&trajectory, config.clock_site, &base_z, config.physical_slices);
    let physical_times: Vec<f64> = (0..phys_indices.len()).map(|k| k as f64).collect();
    let physical_slice_list = slices_from_indices(
        hamiltonian,
        &trajectory,
        &phys_indices,
        &physical_times,
        &config.initial,
        embed_dim,
    );
    for s in &physical_slice_list {
        energy_drift = energy_drift.max((s.energy - energy0).abs());
    }

    let time_map: Vec<TimeMapPoint> = phys_indices
        .iter()
        .enumerate()
        .map(|(k_physical, &uniform_idx)| TimeMapPoint {
            tau_uniform: uniform_idx as f64,
            tau_physical: k_physical as f64,
        })
        .collect();

    let xs: Vec<f64> = time_map.iter().map(|p| p.tau_physical).collect();
    let ys: Vec<f64> = time_map.iter().map(|p| p.tau_uniform).collect();
    let (sync_slope, sync_intercept, sync_r2) = fit_affine(&xs, &ys);

    DualClockResult {
        sites,
        uniform: ClockHistory {
            label: "Uniform clock (fixed Δt)".to_string(),
            clock_site: None,
            result: SpacetimeResult {
                sites,
                slices: uniform_slice_list,
                energy_drift,
                light_cone: None,
                elapsed_ms: 0.0,
            },
        },
        physical: ClockHistory {
            label: format!("Physical clock (site {})", config.clock_site),
            clock_site: Some(config.clock_site),
            result: SpacetimeResult {
                sites,
                slices: physical_slice_list,
                energy_drift,
                light_cone: None,
                elapsed_ms: 0.0,
            },
        },
        time_map,
        sync_r2,
        sync_slope,
        sync_intercept,
        elapsed_ms: 0.0,
    }
}
