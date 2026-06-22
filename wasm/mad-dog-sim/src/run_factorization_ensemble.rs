//! Phase-1 factorization ensemble — multi-model blind recovery battery.

use crate::run_factorization::{run_factorization_search, FactorizationSearchConfig, FactorizationSearchResult};
use serde::Serialize;

const RECOVERY_RATE_MIN: f64 = 0.80;

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnsembleCaseResult {
    pub label: String,
    pub kind: String,
    pub n: usize,
    pub seed: u32,
    pub recovered_identity: bool,
    pub score: f64,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationEnsembleResult {
    pub cases: Vec<EnsembleCaseResult>,
    pub recovered: usize,
    pub total: usize,
    pub recovery_rate: f64,
    pub passed: bool,
    pub elapsed_ms: f64,
    pub backend: &'static str,
}

fn case(
    label: &str,
    kind: &str,
    n: usize,
    field: f64,
    seed: u32,
    eigenstate_count: usize,
    input_mode: &str,
) -> EnsembleCaseResult {
    let result: FactorizationSearchResult = run_factorization_search(&FactorizationSearchConfig {
        kind: kind.to_string(),
        n,
        field,
        seed,
        top_k: 3,
        input_mode: Some(input_mode.to_string()),
        search_method: Some("exact".to_string()),
        eigenstate_count: Some(eigenstate_count),
        graph_kind: None,
        rows: None,
        cols: None,
        distance_decay: None,
        annealing_steps: None,
        spectrum_scramble: None,
    });
    EnsembleCaseResult {
        label: label.to_string(),
        kind: kind.to_string(),
        n,
        seed,
        recovered_identity: result.recovered_identity,
        score: result.best.score,
    }
}

pub fn run_factorization_ensemble() -> FactorizationEnsembleResult {
    let mut cases = Vec::new();

    for seed in [100_u32, 101, 102, 103, 104] {
        cases.push(case(
            &format!("TFIM n=6 seed={seed}"),
            "shuffled_chain",
            6,
            1.5,
            seed,
            3,
            "spectrum",
        ));
    }
    for seed in [100_u32, 101, 102] {
        cases.push(case(
            &format!("TFIM n=8 seed={seed}"),
            "shuffled_chain",
            8,
            1.5,
            seed,
            3,
            "spectrum",
        ));
    }
    for seed in [200_u32, 201, 202, 203, 204] {
        cases.push(case(
            &format!("XX n=6 seed={seed}"),
            "shuffled_xx_chain",
            6,
            1.5,
            seed,
            3,
            "spectrum",
        ));
    }
    for seed in [300_u32, 301, 302, 303, 304] {
        cases.push(case(
            &format!("Heisenberg n=6 seed={seed}"),
            "shuffled_heisenberg_chain",
            6,
            1.5,
            seed,
            3,
            "pauli",
        ));
    }
    for seed in [400_u32, 401, 402, 403, 404] {
        cases.push(case(
            &format!("Sparse n=6 seed={seed}"),
            "shuffled_sparse_chain",
            6,
            1.5,
            seed,
            3,
            "pauli",
        ));
    }

    let recovered = cases.iter().filter(|c| c.recovered_identity).count();
    let total = cases.len();
    let recovery_rate = if total > 0 {
        recovered as f64 / total as f64
    } else {
        0.0
    };

    FactorizationEnsembleResult {
        cases,
        recovered,
        total,
        recovery_rate,
        passed: recovery_rate >= RECOVERY_RATE_MIN,
        elapsed_ms: 0.0,
        backend: "wasm",
    }
}
