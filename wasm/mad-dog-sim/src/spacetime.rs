//! Emergent spacetime — port of `src/sim/spacetime.ts`.

use crate::geometry::analyze_emergent_geometry;
use crate::linalg::procrustes_2d;
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
    let use_2d = embed_dim >= 2;

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

        let report = analyze_emergent_geometry(&psi, 1.0, embed_dim.max(1));
        let coords = if use_2d {
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
        elapsed_ms: 0.0,
    }
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
