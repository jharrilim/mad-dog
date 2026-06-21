//! Relational-time runner for WASM.

use crate::models::tfim_chain;
use crate::quantum::QuantumState;
use crate::relational_time::{build_dual_clock, DualClockConfig, DualClockResult};
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RelationalTimeConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    pub clock_site: usize,
    pub physical_slices: usize,
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

pub fn run_relational_time(config: &RelationalTimeConfig) -> DualClockResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let center = config.n / 2;
    let initial = defect_initial(config.n, center);
    let reference = reference_initial(config.n);
    build_dual_clock(DualClockConfig {
        hamiltonian: &model.hamiltonian,
        initial,
        reference: Some(reference),
        dt: config.dt,
        steps: config.steps,
        clock_site: config.clock_site,
        physical_slices: config.physical_slices,
        embed_dim: 1,
        order: 6,
    })
}
