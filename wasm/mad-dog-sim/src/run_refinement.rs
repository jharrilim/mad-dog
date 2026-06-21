//! Refinement runners for WASM.

use crate::models::tfim_chain;
use crate::quantum::evolve_interval;
use crate::refinement::{
    compute_refinement_baselines, defect_initial_state, measure_refinement_diagnostics,
    refinement_decoupling_lag, RefinementBaselines, RefinementDiagnostics, RefinementThresholds,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementQuenchConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementQuenchSlice {
    pub t: f64,
    pub step: usize,
    pub diagnostics: RefinementDiagnostics,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementQuenchResult {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub baselines: RefinementBaselines,
    pub slices: Vec<RefinementQuenchSlice>,
    pub decoupling_lag: usize,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementNCompareConfig {
    pub n: usize,
    #[serde(default)]
    pub delta_n: Option<usize>,
    pub field: f64,
    pub dt: f64,
    pub quench_step: usize,
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RefinementNCompareResult {
    pub n: usize,
    pub n_large: usize,
    pub field: f64,
    pub quench_step: usize,
    pub dt: f64,
    pub small: RefinementDiagnostics,
    pub large: RefinementDiagnostics,
    pub larger_relieves: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn state_after_quench(
    chain_n: usize,
    field: f64,
    dt: f64,
    quench_step: usize,
    seed: u32,
) -> RefinementDiagnostics {
    let model = tfim_chain(chain_n, 1.0, field);
    let mut rng = Rng::new(42);
    let radius = model
        .hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let baselines = compute_refinement_baselines(chain_n, field, seed);
    let thresholds = RefinementThresholds::default();
    let mut state = defect_initial_state(chain_n);
    for _ in 0..quench_step {
        state = evolve_interval(&model.hamiltonian, &state, dt, radius, 6);
    }
    measure_refinement_diagnostics(&state, 1, &baselines, &thresholds)
}

pub fn run_refinement_quench(config: &RefinementQuenchConfig) -> RefinementQuenchResult {
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
        slices.push(RefinementQuenchSlice {
            t: step as f64 * config.dt,
            step,
            diagnostics: measure_refinement_diagnostics(&state, 1, &baselines, &thresholds),
        });
        if step < config.steps {
            state = evolve_interval(&model.hamiltonian, &state, config.dt, radius, 6);
        }
    }

    let decoupling_lag = refinement_decoupling_lag(
        &slices
            .iter()
            .map(|s| (s.step, s.diagnostics.clone()))
            .collect::<Vec<_>>(),
    );

    RefinementQuenchResult {
        n: config.n,
        field: config.field,
        dt: config.dt,
        baselines,
        slices,
        decoupling_lag,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

pub fn run_refinement_n_compare(config: &RefinementNCompareConfig) -> RefinementNCompareResult {
    let delta_n = config.delta_n.unwrap_or(2);
    let n_large = config.n + delta_n;
    let small = state_after_quench(config.n, config.field, config.dt, config.quench_step, config.seed);
    let large = state_after_quench(
        n_large,
        config.field,
        config.dt,
        config.quench_step,
        config.seed,
    );
    let larger_relieves = large.pressure < small.pressure - 1e-6;

    RefinementNCompareResult {
        n: config.n,
        n_large,
        field: config.field,
        quench_step: config.quench_step,
        dt: config.dt,
        small,
        large,
        larger_relieves,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
