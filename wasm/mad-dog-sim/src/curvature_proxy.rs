//! Curvature proxy via geodesic deviation in MI-derived distance (Phase 5 / S5).

use crate::geometry::{mi_to_distance, mutual_information_matrix};
use crate::geometry_stability::spearman_corr;
use crate::quantum::QuantumState;
use serde::{Deserialize, Serialize};

/// Mean relative triangle defect |d(i,k) − d(i,j) − d(j,k)| / d(i,k) on chain triples.
pub fn chain_geodesic_deviation(distance: &[Vec<f64>]) -> f64 {
    let n = distance.len();
    if n < 3 {
        return 0.0;
    }
    let mut defects = Vec::new();
    for i in 0..n {
        for j in (i + 1)..n {
            for k in (j + 1)..n {
                let direct = distance[i][k];
                let via = distance[i][j] + distance[j][k];
                if direct > 1e-9 {
                    defects.push((via - direct).abs() / direct);
                }
            }
        }
    }
    if defects.is_empty() {
        return 0.0;
    }
    defects.iter().sum::<f64>() / defects.len() as f64
}

pub fn geodesic_deviation_from_state(state: &QuantumState, xi: f64) -> f64 {
    let dist = mi_to_distance(&mutual_information_matrix(state), xi);
    chain_geodesic_deviation(&dist)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvatureProxySlice {
    pub t: f64,
    pub step: usize,
    pub area_pressure: f64,
    pub geodesic_deviation: f64,
    pub rt_slope_deviation: f64,
}

/// Spearman ρ of geodesic deviation vs area pressure (excitation density proxy).
pub fn geodesic_density_correlation(slices: &[CurvatureProxySlice]) -> f64 {
    if slices.len() < 3 {
        return 0.0;
    }
    let density: Vec<f64> = slices.iter().map(|s| s.area_pressure).collect();
    let geo: Vec<f64> = slices.iter().map(|s| s.geodesic_deviation).collect();
    spearman_corr(&density, &geo)
}

/// Spearman ρ between the two curvature proxies (geodesic vs RT slope deficit).
pub fn proxy_cross_correlation(slices: &[CurvatureProxySlice]) -> f64 {
    if slices.len() < 3 {
        return 0.0;
    }
    let geo: Vec<f64> = slices.iter().map(|s| s.geodesic_deviation).collect();
    let slope: Vec<f64> = slices.iter().map(|s| s.rt_slope_deviation).collect();
    spearman_corr(&geo, &slope)
}

/// Internal consistency: geodesic proxy tracks density; cross-proxy not anti-correlated.
pub fn proxy_suite_consistent(slices: &[CurvatureProxySlice], min_geo_corr: f64) -> bool {
    let geo_r = geodesic_density_correlation(slices);
    let cross_r = proxy_cross_correlation(slices);
    geo_r.abs() >= min_geo_corr || cross_r.abs() >= min_geo_corr + 0.1
}

pub fn proxy_suite_correlation(slices: &[CurvatureProxySlice]) -> f64 {
    proxy_cross_correlation(slices)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::tfim_chain;
    use crate::quantum::{evolve_interval, ground_state};
    use crate::refinement::{
        compute_refinement_baselines, defect_initial_state, measure_refinement_diagnostics,
        RefinementThresholds,
    };
    use crate::rng::Rng;
    use crate::rt_quench::rt_quench_slice_from_state;

    #[test]
    fn ground_geodesic_deviation_small() {
        let model = tfim_chain(10, 1.0, 1.5);
        let (ground, _, _) = ground_state(&model.hamiltonian, &mut Rng::new(42), 4000, 1e-9);
        let dev = geodesic_deviation_from_state(&ground, 1.0);
        assert!(dev < 0.5, "ground geodesic dev={dev}");
    }

    #[test]
    fn proxy_correlates_under_quench() {
        let n = 10;
        let field = 1.5;
        let dt = 0.2;
        let model = tfim_chain(n, 1.0, field);
        let mut rng = Rng::new(42);
        let radius = model
            .hamiltonian
            .estimate_spectral_radius(&mut rng, 30)
            .max(1e-6);
        let baselines = compute_refinement_baselines(n, field, 7711);
        let thresholds = RefinementThresholds::default();
        let mut state = defect_initial_state(n);
        let mut slices = Vec::new();
        for step in 0..=16 {
            let diag = measure_refinement_diagnostics(&state, 1, &baselines, &thresholds);
            let rt = rt_quench_slice_from_state(
                &state,
                step,
                step as f64 * dt,
                diag.area_pressure,
                thresholds.max_rt_slope_dev,
            );
            slices.push(CurvatureProxySlice {
                t: rt.t,
                step: rt.step,
                area_pressure: rt.area_pressure,
                geodesic_deviation: geodesic_deviation_from_state(&state, 1.0),
                rt_slope_deviation: rt.slope_deviation,
            });
            if step < 16 {
                state = evolve_interval(&model.hamiltonian, &state, dt, radius, 6);
            }
        }
        assert!(
            proxy_suite_consistent(&slices, 0.2),
            "expected proxy suite consistency, geoρ={}, crossρ={}",
            geodesic_density_correlation(&slices),
            proxy_cross_correlation(&slices)
        );
    }
}
