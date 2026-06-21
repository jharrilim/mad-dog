//! Spacetime runners for WASM.

use crate::models::{tfim_chain, tfim_grid};
use crate::quantum::QuantumState;
use crate::spacetime::{build_spacetime, SpacetimeConfig, SpacetimeResult};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpacetimeRunConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub seed: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Spacetime2DConfig {
    pub rows: usize,
    pub cols: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
}

fn defect_initial(n: usize, center: usize) -> QuantumState {
    let mut initial = QuantumState::zero(n);
    initial.data[2 * (1 << center)] = 1.0;
    initial
}

fn reference_initial(n: usize) -> QuantumState {
    let mut reference = QuantumState::zero(n);
    reference.data[0] = 1.0;
    reference
}

pub fn run_spacetime(config: &SpacetimeRunConfig) -> SpacetimeResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let center = config.n / 2;
    let initial = defect_initial(config.n, center);
    let reference = reference_initial(config.n);
    build_spacetime(SpacetimeConfig {
        hamiltonian: &model.hamiltonian,
        initial,
        reference: Some(reference),
        dt: config.dt,
        steps: config.steps,
        embed_dim: 1,
        align_to: None,
        order: 6,
    })
}

pub fn run_spacetime_2d(config: &Spacetime2DConfig) -> SpacetimeResult {
    let model = tfim_grid(config.rows, config.cols, 1.0, config.field);
    let n = config.rows * config.cols;
    let center = (config.rows / 2) * config.cols + config.cols / 2;
    let initial = defect_initial(n, center);
    let reference = reference_initial(n);
    build_spacetime(SpacetimeConfig {
        hamiltonian: &model.hamiltonian,
        initial,
        reference: Some(reference),
        dt: config.dt,
        steps: config.steps,
        embed_dim: 2,
        align_to: Some(&model.layout.true_positions),
        order: 6,
    })
}
