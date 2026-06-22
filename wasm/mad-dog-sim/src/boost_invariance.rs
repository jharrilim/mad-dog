//! Weak boost invariance — light-cone speed stable under clock-subset observers.

use crate::lorentz::grid_light_cone_velocity;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostInvarianceConfig {
    pub rows: usize,
    pub cols: usize,
    pub field: f64,
    pub dt: f64,
    pub steps: usize,
    #[serde(default)]
    pub edge_site: Option<usize>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BoostInvarianceResult {
    pub rows: usize,
    pub cols: usize,
    pub velocity_uniform: f64,
    pub velocity_edge_clock: f64,
    pub relative_delta: f64,
    pub shape_invariant: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

pub fn run_boost_invariance(config: &BoostInvarianceConfig) -> BoostInvarianceResult {
    let rows = config.rows;
    let cols = config.cols;
    let edge = config.edge_site.unwrap_or(0);
    let v_uniform = grid_light_cone_velocity(
        rows,
        cols,
        config.field,
        config.dt,
        config.steps,
        None,
    );
    let v_edge = grid_light_cone_velocity(
        rows,
        cols,
        config.field,
        config.dt,
        config.steps,
        Some(edge),
    );
    let denom = v_uniform.abs().max(v_edge.abs()).max(1e-9);
    let relative_delta = (v_uniform - v_edge).abs() / denom;
    let shape_invariant = relative_delta < 0.2;
    BoostInvarianceResult {
        rows,
        cols,
        velocity_uniform: v_uniform,
        velocity_edge_clock: v_edge,
        relative_delta,
        shape_invariant,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boost_invariance_within_epsilon() {
        let result = run_boost_invariance(&BoostInvarianceConfig {
            rows: 3,
            cols: 3,
            field: 1.2,
            dt: 0.2,
            steps: 28,
            edge_site: Some(0),
        });
        assert!(result.shape_invariant);
    }
}
