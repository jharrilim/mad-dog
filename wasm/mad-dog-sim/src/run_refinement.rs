//! Refinement runners for WASM.

use crate::models::tfim_chain;
use crate::quantum::{evolve_interval, QuantumState};
use crate::refinement::{
    compute_refinement_baselines, defect_initial_state, first_split_trigger_from_diag,
    measure_refinement_diagnostics, refinement_decoupling_lag, suggest_split_site,
    RefinementBaselines, RefinementDiagnostics, RefinementThresholds, SplitEvent,
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

fn state_after_quench_with_site(
    chain_n: usize,
    field: f64,
    dt: f64,
    quench_step: usize,
    seed: u32,
) -> (QuantumState, RefinementDiagnostics) {
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
    let diag = measure_refinement_diagnostics(&state, 1, &baselines, &thresholds);
    (state, diag)
}

fn state_after_quench(
    chain_n: usize,
    field: f64,
    dt: f64,
    quench_step: usize,
    seed: u32,
) -> RefinementDiagnostics {
    state_after_quench_with_site(chain_n, field, dt, quench_step, seed).1
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

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveRefinementConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub seed: u32,
    #[serde(default)]
    pub delta_n: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AdaptiveRefinementResult {
    pub n: usize,
    pub delta_n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub baselines: RefinementBaselines,
    pub slices: Vec<RefinementQuenchSlice>,
    pub decoupling_lag: usize,
    pub peak_step: usize,
    pub peak_pressure: f64,
    pub split_event: Option<SplitEvent>,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_adaptive_refinement(config: &AdaptiveRefinementConfig) -> AdaptiveRefinementResult {
    let quench = run_refinement_quench(&RefinementQuenchConfig {
        n: config.n,
        field: config.field,
        dt: config.dt,
        steps: config.steps,
        seed: config.seed,
    });
    let delta_n = config.delta_n.unwrap_or(2);
    let diag_steps: Vec<(usize, RefinementDiagnostics)> = quench
        .slices
        .iter()
        .map(|s| (s.step, s.diagnostics.clone()))
        .collect();

    let (peak_step, peak_pressure) = diag_steps
        .iter()
        .fold((0usize, 0.0_f64), |acc, (step, d)| {
            if d.pressure > acc.1 {
                (*step, d.pressure)
            } else {
                acc
            }
        });

    let thresholds = RefinementThresholds::default();
    let split_event = first_split_trigger_from_diag(&diag_steps, &thresholds).map(|trigger_step| {
        let trigger_t = trigger_step as f64 * config.dt;
        let (state, pre) = state_after_quench_with_site(
            config.n,
            config.field,
            config.dt,
            trigger_step,
            config.seed,
        );
        let split_site = suggest_split_site(&state);
        let post = state_after_quench(
            config.n + delta_n,
            config.field,
            config.dt,
            trigger_step,
            config.seed,
        );
        let pressure_delta = pre.pressure - post.pressure;
        let accepted = post.pressure < pre.pressure - 1e-6;
        SplitEvent {
            trigger_step,
            trigger_t,
            pre_n: config.n,
            post_n: config.n + delta_n,
            suggested_split_site: split_site,
            pre,
            post,
            accepted,
            pressure_delta,
        }
    });

    AdaptiveRefinementResult {
        n: config.n,
        delta_n,
        field: config.field,
        dt: config.dt,
        steps: config.steps,
        baselines: quench.baselines,
        slices: quench.slices,
        decoupling_lag: quench.decoupling_lag,
        peak_step,
        peak_pressure,
        split_event,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
