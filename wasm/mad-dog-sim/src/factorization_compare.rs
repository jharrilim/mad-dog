//! Same |ψ⟩, different n — factorization metrics under embed / truncate (Phase 11 / AG).

use crate::holographic_bound::blind_locality_fraction;
use crate::models::tfim_chain;
use crate::quantum::QuantumState;
use crate::refinement::{
    compute_refinement_baselines, measure_refinement_diagnostics, suggest_split_site,
    RefinementThresholds,
};
use crate::tensor_split::{
    embed_state_at_split, quench_state_at_step, truncate_state_at_split,
};
use serde::{Deserialize, Serialize};

fn state_fidelity(a: &QuantumState, b: &QuantumState) -> f64 {
    assert_eq!(a.n, b.n);
    let mut re = 0.0;
    let mut im = 0.0;
    for i in 0..a.dim {
        re += a.data[2 * i] * b.data[2 * i] + a.data[2 * i + 1] * b.data[2 * i + 1];
        im += a.data[2 * i] * b.data[2 * i + 1] - a.data[2 * i + 1] * b.data[2 * i];
    }
    re * re + im * im
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationCompareCase {
    pub label: String,
    pub n: usize,
    pub locality_fraction: f64,
    pub emergent_dim: usize,
    pub pressure: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationCompareConfig {
    pub n_small: usize,
    pub field: f64,
    pub dt: f64,
    pub step: usize,
    pub seed: u32,
    #[serde(default)]
    pub delta_n: Option<usize>,
    #[serde(default)]
    pub split_site: Option<usize>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationCompareResult {
    pub n_small: usize,
    pub n_large: usize,
    pub delta_n: usize,
    pub split_site: usize,
    pub step: usize,
    pub roundtrip_fidelity: f64,
    pub locality_drift_embed: f64,
    pub locality_drift_roundtrip: f64,
    pub dim_drift_embed: usize,
    pub dim_drift_roundtrip: usize,
    pub embedding_faithful: bool,
    pub cases: Vec<FactorizationCompareCase>,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn case_metrics(
    label: &str,
    state: &QuantumState,
    field: f64,
    seed: u32,
) -> FactorizationCompareCase {
    let model = tfim_chain(state.n, 1.0, field);
    let baselines = compute_refinement_baselines(state.n, field, seed);
    let thresholds = RefinementThresholds::default();
    let diag = measure_refinement_diagnostics(state, 1, &baselines, &thresholds);
    let locality = blind_locality_fraction(&model.hamiltonian, state);
    FactorizationCompareCase {
        label: label.to_string(),
        n: state.n,
        locality_fraction: locality,
        emergent_dim: diag.emergent_dim,
        pressure: diag.pressure,
    }
}

pub fn run_factorization_compare(config: &FactorizationCompareConfig) -> FactorizationCompareResult {
    let delta_n = config.delta_n.unwrap_or(2);
    let n_small = config.n_small;
    let n_large = n_small + delta_n;

    let psi_small =
        quench_state_at_step(n_small, config.field, config.dt, config.step, config.seed);
    let split_site = config
        .split_site
        .unwrap_or_else(|| suggest_split_site(&psi_small));

    let psi_embed = embed_state_at_split(&psi_small, split_site, delta_n);
    let psi_roundtrip = truncate_state_at_split(&psi_embed, split_site, delta_n);
    let psi_control =
        quench_state_at_step(n_large, config.field, config.dt, config.step, config.seed);

    let roundtrip_fidelity = state_fidelity(&psi_small, &psi_roundtrip);

    let native = case_metrics("native_small", &psi_small, config.field, config.seed);
    let embedded = case_metrics("embedded", &psi_embed, config.field, config.seed);
    let roundtrip = case_metrics("roundtrip", &psi_roundtrip, config.field, config.seed);
    let control = case_metrics("control_large_quench", &psi_control, config.field, config.seed);

    let locality_drift_embed = (native.locality_fraction - embedded.locality_fraction).abs();
    let locality_drift_roundtrip = (native.locality_fraction - roundtrip.locality_fraction).abs();
    let dim_drift_embed = native.emergent_dim.abs_diff(embedded.emergent_dim);
    let dim_drift_roundtrip = native.emergent_dim.abs_diff(roundtrip.emergent_dim);
    let embedding_faithful = roundtrip_fidelity > 0.99
        && locality_drift_roundtrip < 0.05
        && dim_drift_roundtrip <= 1;

    FactorizationCompareResult {
        n_small,
        n_large,
        delta_n,
        split_site,
        step: config.step,
        roundtrip_fidelity,
        locality_drift_embed,
        locality_drift_roundtrip,
        dim_drift_embed,
        dim_drift_roundtrip,
        embedding_faithful,
        cases: vec![native, embedded, roundtrip, control],
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embed_roundtrip_near_unit_fidelity() {
        let r = run_factorization_compare(&FactorizationCompareConfig {
            n_small: 8,
            field: 1.5,
            dt: 0.2,
            step: 12,
            seed: 4242,
            delta_n: Some(2),
            split_site: None,
        });
        assert!(r.roundtrip_fidelity > 0.999, "fidelity={}", r.roundtrip_fidelity);
        assert!(r.embedding_faithful);
    }
}
