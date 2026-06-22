//! Emergent simultaneity surfaces runner.

use crate::models::tfim_chain;
use crate::quantum::QuantumState;
use crate::simultaneity::{build_simultaneity_surfaces, SimultaneityConfig, SimultaneityResult};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SimultaneityRunConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub clock_a: Option<usize>,
    #[serde(default)]
    pub clock_b: Option<usize>,
    #[serde(default)]
    pub physical_slices: Option<usize>,
    #[serde(default)]
    pub embed_dim: Option<usize>,
    #[serde(default)]
    pub reference_site: Option<usize>,
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

pub fn run_simultaneity(config: &SimultaneityRunConfig) -> SimultaneityResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let center = config.n / 2;
    let initial = defect_initial(config.n, center);
    let reference = reference_initial(config.n);
    build_simultaneity_surfaces(
        &model.hamiltonian,
        &initial,
        &reference,
        &SimultaneityConfig {
            n: config.n,
            field: config.field,
            dt: config.dt,
            steps: config.steps,
            clock_a: config.clock_a,
            clock_b: config.clock_b,
            physical_slices: config.physical_slices.unwrap_or(15),
            embed_dim: config.embed_dim.unwrap_or(2),
            reference_site: config.reference_site,
        },
    )
}
