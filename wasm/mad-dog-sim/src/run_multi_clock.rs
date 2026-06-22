//! Multi-clock consistency runner for WASM.

use crate::models::tfim_chain;
use crate::multi_clock::{build_multi_clock, MultiClockConfig, MultiClockResult};
use crate::quantum::QuantumState;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MultiClockRunConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub clock_sites: Option<Vec<usize>>,
    #[serde(default)]
    pub physical_slices: Option<usize>,
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

pub fn run_multi_clock(config: &MultiClockRunConfig) -> MultiClockResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let center = config.n / 2;
    let initial = defect_initial(config.n, center);
    let reference = reference_initial(config.n);
    build_multi_clock(
        &model.hamiltonian,
        initial,
        reference,
        &MultiClockConfig {
            n: config.n,
            dt: config.dt,
            steps: config.steps,
            clock_sites: config.clock_sites.clone(),
            physical_slices: config.physical_slices.unwrap_or(15),
        },
    )
}
