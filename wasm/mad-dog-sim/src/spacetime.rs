//! Emergent spacetime — port of `src/sim/spacetime.ts`.

use crate::geometry::analyze_emergent_geometry;
use crate::linalg::{procrustes_2d, procrustes_3d};
use crate::models::TruePosition;
use crate::quantum::{evolve_interval, expectation_z, Hamiltonian, QuantumState};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacetimeSlice {
    pub k: usize,
    pub t: f64,
    pub coords: Vec<Vec<f64>>,
    pub z_expectation: Vec<f64>,
    pub signal: Vec<f64>,
    pub energy: f64,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacetimeResult {
    pub sites: usize,
    pub slices: Vec<SpacetimeSlice>,
    pub energy_drift: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub light_cone: Option<LightCone>,
    pub elapsed_ms: f64,
}

#[derive(Clone, Debug)]
pub struct SpacetimeConfig<'a> {
    pub hamiltonian: &'a Hamiltonian,
    pub initial: QuantumState,
    pub reference: Option<QuantumState>,
    pub dt: f64,
    pub steps: usize,
    pub embed_dim: usize,
    pub align_to: Option<&'a [TruePosition]>,
    pub order: usize,
    /// When false, skip MI/MDS per slice (light-cone signal only).
    pub include_geometry: bool,
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

pub fn build_spacetime(config: SpacetimeConfig<'_>) -> SpacetimeResult {
    let hamiltonian = config.hamiltonian;
    let sites = hamiltonian.n;
    let embed_dim = config.embed_dim.max(1);
    let use_3d = embed_dim >= 3;
    let use_2d = embed_dim >= 2 && !use_3d;

    let align_target_3: Option<Vec<Vec<f64>>> = config.align_to.map(|pts| {
        pts.iter()
            .map(|p| vec![p.x, p.y, p.z.unwrap_or(0.0)])
            .collect()
    });
    let align_target_2: Option<Vec<Vec<f64>>> = config.align_to.map(|pts| {
        pts.iter().map(|p| vec![p.x, p.y]).collect()
    });

    let energy0 = hamiltonian.expectation(&config.initial);
    let mut energy_drift = 0.0_f64;
    let mut rng = Rng::new(42);
    let radius = hamiltonian.estimate_spectral_radius(&mut rng, 30).max(1e-6);

    let mut slices = Vec::new();
    let mut psi = config.initial.clone_state();
    let mut ref_state = config.reference.clone();
    let base_z: Vec<f64> = (0..sites).map(|q| expectation_z(&config.initial, q)).collect();

    for k in 0..config.steps {
        let t = k as f64 * config.dt;
        let energy = hamiltonian.expectation(&psi);
        energy_drift = energy_drift.max((energy - energy0).abs());

        let coords = if config.include_geometry {
            let report = analyze_emergent_geometry(&psi, 1.0, embed_dim.max(1));
            if use_3d {
                let raw: Vec<Vec<f64>> = report
                    .mds
                    .coords
                    .iter()
                    .map(|c| {
                        vec![
                            c.first().copied().unwrap_or(0.0),
                            c.get(1).copied().unwrap_or(0.0),
                            c.get(2).copied().unwrap_or(0.0),
                        ]
                    })
                    .collect();
                if let Some(ref target) = align_target_3 {
                    procrustes_3d(&raw, target)
                } else {
                    raw
                }
            } else if use_2d {
                let raw: Vec<Vec<f64>> = report
                    .mds
                    .coords
                    .iter()
                    .map(|c| vec![c.first().copied().unwrap_or(0.0), c.get(1).copied().unwrap_or(0.0)])
                    .collect();
                if let Some(ref target) = align_target_2 {
                    procrustes_2d(&raw, target)
                } else {
                    raw
                }
            } else {
                let oriented = orient_coords(
                    &report
                        .mds
                        .coords
                        .iter()
                        .map(|c| c.first().copied().unwrap_or(0.0))
                        .collect::<Vec<_>>(),
                );
                oriented.into_iter().map(|x| vec![x]).collect()
            }
        } else {
            vec![Vec::new(); sites]
        };

        let z_expectation: Vec<f64> = (0..sites).map(|q| expectation_z(&psi, q)).collect();
        let ref_z: Vec<f64> = if let Some(ref r) = ref_state {
            (0..sites).map(|q| expectation_z(r, q)).collect()
        } else {
            base_z.clone()
        };
        let signal: Vec<f64> = z_expectation
            .iter()
            .zip(ref_z.iter())
            .map(|(z, rz)| (z - rz).abs())
            .collect();

        slices.push(SpacetimeSlice {
            k,
            t,
            coords,
            z_expectation,
            signal,
            energy,
        });

        psi = evolve_interval(hamiltonian, &psi, config.dt, radius, config.order);
        psi.normalize();
        if let Some(ref mut r) = ref_state {
            *r = evolve_interval(hamiltonian, r, config.dt, radius, config.order);
            r.normalize();
        }
    }

    SpacetimeResult {
        sites,
        slices,
        energy_drift,
        light_cone: None,
        elapsed_ms: 0.0,
    }
}

/// Light-cone trajectory without per-slice MI/MDS (scattering, sweeps).
pub fn build_light_cone_trajectory(mut config: SpacetimeConfig<'_>) -> SpacetimeResult {
    config.include_geometry = false;
    build_spacetime(config)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightCone {
    pub velocity: f64,
    pub arrivals: Vec<f64>,
    pub center: usize,
    pub dt: f64,
}

pub fn measure_light_cone(
    result: &SpacetimeResult,
    threshold_fraction: f64,
    center_site: Option<usize>,
    site_distances: Option<&[f64]>,
) -> LightCone {
    let n = result.sites;
    let center = center_site.unwrap_or(n / 2);
    let mut max = 0.0_f64;
    for s in &result.slices {
        for &v in &s.signal {
            max = max.max(v);
        }
    }
    let thr = (max * threshold_fraction).max(1e-6);

    let mut arrivals = vec![f64::INFINITY; n];
    for slice in &result.slices {
        for (i, &d) in slice.signal.iter().enumerate() {
            if d > thr && slice.t < arrivals[i] {
                arrivals[i] = slice.t;
            }
        }
    }

    let mut num = 0.0;
    let mut den = 0.0;
    for i in 0..n {
        if i == center {
            continue;
        }
        let t = arrivals[i];
        if !t.is_finite() || t <= 0.0 {
            continue;
        }
        let d = site_distances
            .map(|sd| sd[i])
            .unwrap_or(((i as i32) - (center as i32)).abs() as f64);
        num += d * t;
        den += t * t;
    }
    let velocity = if den > 0.0 { num / den } else { 0.0 };
    let dt = if result.slices.len() > 1 {
        result.slices[1].t - result.slices[0].t
    } else {
        1.0
    };
    LightCone {
        velocity,
        arrivals,
        center,
        dt,
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightConeComparison {
    pub lattice: LightCone,
    pub mi: LightCone,
    pub mi_time_avg: LightCone,
    pub velocity_ratio: f64,
}

fn chain_distances(n: usize, center: usize) -> Vec<f64> {
    (0..n).map(|i| ((i as i32) - (center as i32)).abs() as f64).collect()
}

pub fn average_mi_distances_from_trajectory(
    hamiltonian: &crate::quantum::Hamiltonian,
    initial: &crate::quantum::QuantumState,
    center: usize,
    dt: f64,
    steps: usize,
    order: usize,
) -> Vec<f64> {
    use crate::geometry::mi_distances_from_state;
    use crate::quantum::evolve_interval;
    use crate::rng::Rng;

    let n = hamiltonian.n;
    let mut sums = vec![0.0; n];
    let mut psi = initial.clone();
    let mut rng = Rng::new(42);
    let radius = hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let count = steps.max(1) as f64;
    for k in 0..steps {
        let d = mi_distances_from_state(&psi, center, 1.0);
        for i in 0..n {
            sums[i] += d[i];
        }
        if k + 1 < steps {
            psi = evolve_interval(hamiltonian, &psi, dt, radius, order);
            psi.normalize();
        }
    }
    sums.iter().map(|s| s / count).collect()
}

pub fn compare_light_cone_velocities(
    result: &SpacetimeResult,
    threshold_fraction: f64,
    center_site: Option<usize>,
    mi_distances: Option<&[f64]>,
    mi_distances_time_avg: Option<&[f64]>,
) -> LightConeComparison {
    let center = center_site.unwrap_or(result.sites / 2);
    let lattice_d = chain_distances(result.sites, center);
    let lattice = measure_light_cone(
        result,
        threshold_fraction,
        Some(center),
        Some(&lattice_d),
    );
    let mi_d = mi_distances.unwrap_or(&lattice_d);
    let mi_time_d = mi_distances_time_avg.unwrap_or(mi_d);
    let mi = measure_light_cone(result, threshold_fraction, Some(center), Some(mi_d));
    let mi_time_avg =
        measure_light_cone(result, threshold_fraction, Some(center), Some(mi_time_d));
    let velocity_ratio = if lattice.velocity > 1e-9 {
        mi.velocity / lattice.velocity
    } else {
        0.0
    };
    LightConeComparison {
        lattice,
        mi,
        mi_time_avg,
        velocity_ratio,
    }
}
