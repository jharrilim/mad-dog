//! Joint factorization + refinement quench study.

use crate::factorization::{fast_search_on_state, perm_distance};
use crate::models::tfim_chain;
use crate::quantum::evolve_interval;
use crate::refinement::{
    compute_refinement_baselines, defect_initial_state, measure_refinement_diagnostics,
    RefinementDiagnostics, RefinementThresholds,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationRefinementConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub seed: u32,
    #[serde(default = "default_anneal_steps")]
    pub annealing_steps: usize,
}

fn default_anneal_steps() -> usize {
    800
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationRefinementSlice {
    pub t: f64,
    pub step: usize,
    pub refinement_pressure: f64,
    pub factorization_score: f64,
    pub locality_fraction: f64,
    pub permutation: Vec<usize>,
    pub perm_drift: usize,
    pub diagnostics: RefinementDiagnostics,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationRefinementResult {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub slices: Vec<FactorizationRefinementSlice>,
    pub initial_permutation: Vec<usize>,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_factorization_refinement_study(
    config: &FactorizationRefinementConfig,
) -> FactorizationRefinementResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let mut rng = Rng::new(42);
    let radius = model
        .hamiltonian
        .estimate_spectral_radius(&mut rng, 30)
        .max(1e-6);
    let baselines = compute_refinement_baselines(config.n, config.field, config.seed);
    let thresholds = RefinementThresholds::default();

    let mut state = defect_initial_state(config.n);
    let initial_search = fast_search_on_state(
        &model.hamiltonian,
        &state,
        config.annealing_steps,
    );
    let initial_perm = initial_search.permutation.clone();

    let mut slices = Vec::new();
    for step in 0..=config.steps {
        let diag = measure_refinement_diagnostics(&state, 1, &baselines, &thresholds);
        let fac = fast_search_on_state(
            &model.hamiltonian,
            &state,
            config.annealing_steps,
        );
        slices.push(FactorizationRefinementSlice {
            t: step as f64 * config.dt,
            step,
            refinement_pressure: diag.pressure,
            factorization_score: fac.score,
            locality_fraction: fac.locality_fraction,
            permutation: fac.permutation.clone(),
            perm_drift: perm_distance(&fac.permutation, &initial_perm),
            diagnostics: diag,
        });
        if step < config.steps {
            state = evolve_interval(&model.hamiltonian, &state, config.dt, radius, 6);
        }
    }

    FactorizationRefinementResult {
        n: config.n,
        field: config.field,
        dt: config.dt,
        slices,
        initial_permutation: initial_perm,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
