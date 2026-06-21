//! High-level drivers exposed to WASM.

use crate::geometry::{analyze_emergent_geometry, EmergenceReport};
use crate::models::{random_nonlocal, tfim_chain, tfim_grid, BuiltModel, TruePosition};
use crate::quantum::ground_state;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RunConfig {
    pub kind: String,
    pub n: usize,
    pub rows: usize,
    pub cols: usize,
    pub field: f64,
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RunResult {
    pub label: String,
    pub qubits: usize,
    pub energy: f64,
    pub iters: usize,
    pub expected_dim: usize,
    pub report: EmergenceReport,
    pub true_positions: Vec<TruePosition>,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn build_model(config: &RunConfig) -> BuiltModel {
    match config.kind.as_str() {
        "chain" => tfim_chain(config.n, 1.0, config.field),
        "grid" => tfim_grid(config.rows, config.cols, 1.0, config.field),
        _ => {
            let mut rng = Rng::new(config.seed.wrapping_add(991));
            random_nonlocal(config.n, &mut rng)
        }
    }
}

pub fn run_emergence(config: &RunConfig) -> RunResult {
    let model = build_model(config);
    let mut rng = Rng::new(config.seed);
    let (state, energy, iters) = ground_state(&model.hamiltonian, &mut rng, 4000, 1e-9);
    let report = analyze_emergent_geometry(&state, 1.0, 3);
    RunResult {
        label: model.label.clone(),
        qubits: model.hamiltonian.n,
        energy,
        iters,
        expected_dim: model.layout.expected_dim,
        true_positions: model.layout.true_positions.clone(),
        report,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
