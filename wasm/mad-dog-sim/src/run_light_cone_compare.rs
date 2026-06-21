//! Lieb–Robinson velocity: lattice distance vs MI emergent distance.

use crate::geometry::mi_distances_from_state;
use crate::models::tfim_chain;
use crate::quantum::QuantumState;
use crate::spacetime::{
    average_mi_distances_from_trajectory, build_spacetime, compare_light_cone_velocities,
    LightConeComparison, SpacetimeConfig, SpacetimeResult,
};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LightConeCompareConfig {
    pub n: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub seed: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LightConeCompareResult {
    pub spacetime: SpacetimeResult,
    pub comparison: LightConeComparison,
    pub elapsed_ms: f64,
    pub backend: &'static str,
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

pub fn run_light_cone_compare(config: &LightConeCompareConfig) -> LightConeCompareResult {
    let model = tfim_chain(config.n, 1.0, config.field);
    let center = config.n / 2;
    let initial = defect_initial(config.n, center);
    let reference = reference_initial(config.n);
    let spacetime = build_spacetime(SpacetimeConfig {
        hamiltonian: &model.hamiltonian,
        initial: initial.clone(),
        reference: Some(reference),
        dt: config.dt,
        steps: config.steps,
        embed_dim: 1,
        align_to: None,
        order: 6,
        include_geometry: true,
    });
    let mi_dist = mi_distances_from_state(&initial, center, 1.0);
    let mi_time_avg = average_mi_distances_from_trajectory(
        &model.hamiltonian,
        &initial,
        center,
        config.dt,
        config.steps,
        6,
    );
    let comparison = compare_light_cone_velocities(
        &spacetime,
        0.12,
        Some(center),
        Some(&mi_dist),
        Some(&mi_time_avg),
    );
    LightConeCompareResult {
        spacetime,
        comparison,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
