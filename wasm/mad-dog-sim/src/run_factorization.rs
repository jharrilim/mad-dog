//! Factorization search runner for WASM.

use crate::factorization::{
    line_equiv_distance, line_equiv_match, perm_distance, score_permutation, search_factorization,
    shuffle_hamiltonian, spectrum_from_hamiltonian, FactorizationCandidate, GraphKind,
    InputMode, SearchMethod, SearchParams,
};
use crate::geometry::mutual_information_matrix;
use crate::models::{random_nonlocal, tfim_chain, tfim_grid};
use crate::quantum::low_energy_states;
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationSearchConfig {
    pub kind: String,
    pub n: usize,
    pub field: f64,
    pub seed: u32,
    #[serde(default = "default_top_k")]
    pub top_k: usize,
    #[serde(default)]
    pub input_mode: Option<String>,
    #[serde(default)]
    pub search_method: Option<String>,
    #[serde(default)]
    pub eigenstate_count: Option<usize>,
    #[serde(default)]
    pub graph_kind: Option<String>,
    #[serde(default)]
    pub rows: Option<usize>,
    #[serde(default)]
    pub cols: Option<usize>,
    #[serde(default)]
    pub distance_decay: Option<f64>,
    #[serde(default)]
    pub annealing_steps: Option<usize>,
}

fn default_top_k() -> usize {
    5
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationSearchResult {
    pub label: String,
    pub n: usize,
    pub energy: f64,
    pub baseline: FactorizationCandidate,
    pub best: FactorizationCandidate,
    pub top_candidates: Vec<FactorizationCandidate>,
    pub recovered_identity: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub perm_match_distance: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub true_shuffle: Option<Vec<usize>>,
    pub baseline_mi: Vec<Vec<f64>>,
    pub best_mi: Vec<Vec<f64>>,
    pub coupling_edges: Vec<crate::factorization::CouplingEdge>,
    pub input_mode: String,
    pub scorer_used: String,
    pub search_method: String,
    pub search_iters: usize,
    pub elapsed_ms: f64,
}

fn parse_input_mode(s: Option<&String>) -> InputMode {
    match s.map(|x| x.as_str()) {
        Some("spectrum") => InputMode::Spectrum,
        _ => InputMode::Pauli,
    }
}

fn parse_search_method(s: Option<&String>, n: usize) -> SearchMethod {
    match s.map(|x| x.as_str()) {
        Some("exact") => SearchMethod::Exact,
        Some("greedy") => SearchMethod::Greedy,
        Some("annealing") => SearchMethod::Annealing,
        _ => {
            if n <= 8 {
                SearchMethod::Exact
            } else {
                SearchMethod::Annealing
            }
        }
    }
}

fn parse_graph_kind(s: Option<&String>) -> GraphKind {
    match s.map(|x| x.as_str()) {
        Some("grid") => GraphKind::Grid,
        _ => GraphKind::Line,
    }
}

fn build_params(config: &FactorizationSearchConfig) -> SearchParams {
    let graph_kind = parse_graph_kind(config.graph_kind.as_ref());
    let rows = config.rows.unwrap_or(0);
    let cols = config.cols.unwrap_or(0);
    SearchParams {
        graph_kind,
        rows,
        cols,
        input_mode: parse_input_mode(config.input_mode.as_ref()),
        search_method: parse_search_method(config.search_method.as_ref(), config.n),
        eigenstate_count: config.eigenstate_count.unwrap_or(1).clamp(1, 4),
        distance_decay: config.distance_decay.unwrap_or(0.0),
        annealing_steps: config.annealing_steps.unwrap_or(3000),
        emergent_dim_weight: 0.1,
    }
}

fn inverse_shuffle(shuffle: &[usize]) -> Vec<usize> {
    let mut inv = vec![0usize; shuffle.len()];
    for (q, &p) in shuffle.iter().enumerate() {
        inv[p] = q;
    }
    inv
}

pub fn run_factorization_search(config: &FactorizationSearchConfig) -> FactorizationSearchResult {
    let top_k = config.top_k.clamp(1, 20);
    let mut params = build_params(config);

    let (label, hamiltonian, true_shuffle) = match config.kind.as_str() {
        "shuffled_chain" => {
            let model = tfim_chain(config.n, 1.0, config.field);
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, config.seed);
            (
                format!("Shuffled TFIM chain (n={})", config.n),
                shuffled,
                Some(shuffle_perm),
            )
        }
        "shuffled_grid" => {
            let rows = config.rows.unwrap_or(3);
            let cols = config.cols.unwrap_or(3);
            params.graph_kind = GraphKind::Grid;
            params.rows = rows;
            params.cols = cols;
            let model = tfim_grid(rows, cols, 1.0, config.field);
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, config.seed);
            (
                format!("Shuffled TFIM grid ({rows}x{cols})"),
                shuffled,
                Some(shuffle_perm),
            )
        }
        _ => {
            let mut rng = Rng::new(config.seed);
            let model = random_nonlocal(config.n, &mut rng);
            (model.label.clone(), model.hamiltonian, None)
        }
    };

    let k = params.eigenstate_count;
    let states: Vec<_> = low_energy_states(&hamiltonian, k, config.seed)
        .into_iter()
        .map(|(s, _)| s)
        .collect();
    let energy = hamiltonian.expectation(&states[0]);

    let spectrum = if params.input_mode == InputMode::Spectrum {
        Some(spectrum_from_hamiltonian(&hamiltonian, k))
    } else {
        None
    };

    let outcome = search_factorization(
        &hamiltonian,
        &states,
        top_k,
        &params,
        spectrum.as_ref(),
    );

    let perm_match_distance = true_shuffle.as_ref().map(|shuffle| {
        line_equiv_distance(&outcome.best.permutation, &inverse_shuffle(shuffle))
    });

    let recovered_identity = if let Some(ref shuffle) = true_shuffle {
        let inv = inverse_shuffle(shuffle);
        let perm_ok = if params.input_mode == InputMode::Spectrum {
            line_equiv_match(&outcome.best.permutation, &inv)
        } else {
            perm_distance(&outcome.best.permutation, &inv) == 0
        };
        if params.input_mode == InputMode::Spectrum {
            perm_ok
        } else {
            let mi = mutual_information_matrix(&states[0]);
            let recovery = score_permutation(
                Some(&hamiltonian),
                &[mi],
                &inv,
                &params,
                None,
                false,
            );
            perm_ok
                || (recovery.locality_fraction > 0.99 && recovery.nonlocal_terms == 0)
        }
    } else {
        outcome.best.score > outcome.baseline.score + 0.05
    };

    let input_mode_str = match params.input_mode {
        InputMode::Pauli => "pauli",
        InputMode::Spectrum => "spectrum",
    }
    .to_string();

    FactorizationSearchResult {
        label,
        n: config.n,
        energy,
        baseline: outcome.baseline,
        best: outcome.best,
        top_candidates: outcome.top_candidates,
        recovered_identity,
        perm_match_distance,
        true_shuffle,
        baseline_mi: outcome.baseline_mi,
        best_mi: outcome.best_mi,
        coupling_edges: outcome.coupling_edges,
        input_mode: input_mode_str,
        scorer_used: outcome.scorer_used,
        search_method: outcome.search_method,
        search_iters: outcome.search_iters,
        elapsed_ms: 0.0,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::factorization::line_equiv_distance;

    #[test]
    fn spectrum_recovers_shuffled_chain_n6() {
        let config = FactorizationSearchConfig {
            kind: "shuffled_chain".to_string(),
            n: 6,
            field: 1.5,
            seed: 4242,
            top_k: 3,
            input_mode: Some("spectrum".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(3),
            graph_kind: None,
            rows: None,
            cols: None,
            distance_decay: None,
            annealing_steps: None,
        };
        let result = run_factorization_search(&config);
        assert!(
            result.recovered_identity,
            "score={} perm={:?} dist={:?}",
            result.best.score,
            result.best.permutation,
            result.perm_match_distance
        );
        assert_eq!(result.perm_match_distance, Some(0));
    }

    #[test]
    fn line_equiv_distance_accepts_reflection() {
        let a = vec![0, 3, 4, 1, 2, 5];
        let b = vec![5, 2, 1, 4, 3, 0];
        assert_eq!(line_equiv_distance(&a, &b), 0);
    }
}
