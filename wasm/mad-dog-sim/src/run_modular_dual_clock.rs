//! Modular dual-clock runner.

use crate::models::tfim_chain;
use crate::modular_time::{build_modular_dual_clock, ModularDualClockConfig, ModularDualClockResult};
use crate::quantum::QuantumState;

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

pub fn run_modular_dual_clock(config: &ModularDualClockConfig) -> ModularDualClockResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let center = config.n / 2;
    let initial = defect_initial(config.n, center);
    let reference = reference_initial(config.n);
    build_modular_dual_clock(&model.hamiltonian, config, initial, reference)
}
