//! RT quench time-series runner (Phase 5 / S5).

use crate::models::tfim_chain;
use crate::quantum::{evolve_interval, QuantumState};
use crate::refinement::{
    compute_refinement_baselines, defect_initial_state, measure_refinement_diagnostics,
    RefinementBaselines, RefinementThresholds,
};
use crate::rng::Rng;
use crate::rt_quench::{
    rt_deviation_density_correlation, rt_quench_slice_from_state, structured_rt_deviation,
    RtQuenchSlice,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RtQuenchConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RtQuenchResult {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub baselines: RefinementBaselines,
    pub slices: Vec<RtQuenchSlice>,
    pub deviation_density_corr: f64,
    pub structured_deviation: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_rt_quench(config: &RtQuenchConfig) -> RtQuenchResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let mut rng = Rng::new(42);
    let radius = model
        .hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let baselines = compute_refinement_baselines(config.n, config.field, config.seed);
    let thresholds = RefinementThresholds::default();
    let mut state = defect_initial_state(config.n);
    let mut slices = Vec::new();

    for step in 0..=config.steps {
        let diag = measure_refinement_diagnostics(&state, 1, &baselines, &thresholds);
        slices.push(rt_quench_slice_from_state(
            &state,
            step,
            step as f64 * config.dt,
            diag.area_pressure,
            thresholds.max_rt_slope_dev,
        ));
        if step < config.steps {
            state = evolve_interval(&model.hamiltonian, &state, config.dt, radius, 6);
        }
    }

    let deviation_density_corr = rt_deviation_density_correlation(&slices);
    let structured_deviation = structured_rt_deviation(&slices, 0.35);

    RtQuenchResult {
        n: config.n,
        field: config.field,
        dt: config.dt,
        baselines,
        slices,
        deviation_density_corr,
        structured_deviation,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvatureProxyQuenchConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub seed: u32,
    #[serde(default = "default_xi")]
    pub xi: f64,
}

fn default_xi() -> f64 {
    1.0
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CurvatureProxyQuenchResult {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub xi: f64,
    pub slices: Vec<crate::curvature_proxy::CurvatureProxySlice>,
    pub proxy_correlation: f64,
    pub geo_density_corr: f64,
    pub internally_consistent: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_curvature_proxy_quench(config: &CurvatureProxyQuenchConfig) -> CurvatureProxyQuenchResult {
use crate::curvature_proxy::{
    geodesic_deviation_from_state, geodesic_density_correlation, proxy_suite_consistent,
    proxy_suite_correlation, CurvatureProxySlice,
};

    let model = tfim_chain(config.n, 1.0, config.field);
    let mut rng = Rng::new(42);
    let radius = model
        .hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let baselines = compute_refinement_baselines(config.n, config.field, config.seed);
    let thresholds = RefinementThresholds::default();
    let mut state: QuantumState = defect_initial_state(config.n);
    let mut slices = Vec::new();

    for step in 0..=config.steps {
        let diag = measure_refinement_diagnostics(&state, 1, &baselines, &thresholds);
        let rt = rt_quench_slice_from_state(
            &state,
            step,
            step as f64 * config.dt,
            diag.area_pressure,
            thresholds.max_rt_slope_dev,
        );
        slices.push(CurvatureProxySlice {
            t: rt.t,
            step: rt.step,
            area_pressure: rt.area_pressure,
            geodesic_deviation: geodesic_deviation_from_state(&state, config.xi),
            rt_slope_deviation: rt.slope_deviation,
        });
        if step < config.steps {
            state = evolve_interval(&model.hamiltonian, &state, config.dt, radius, 6);
        }
    }

    let proxy_correlation = proxy_suite_correlation(&slices);
    let geo_density_corr = geodesic_density_correlation(&slices);
    let internally_consistent = proxy_suite_consistent(&slices, 0.25);

    CurvatureProxyQuenchResult {
        n: config.n,
        field: config.field,
        dt: config.dt,
        xi: config.xi,
        slices,
        proxy_correlation,
        geo_density_corr,
        internally_consistent,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
