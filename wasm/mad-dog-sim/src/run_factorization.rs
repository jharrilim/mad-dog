//! Factorization search runner for WASM.

use crate::factorization::{
    line_equiv_distance, line_equiv_match, perm_distance, score_permutation, search_factorization,
    shuffle_hamiltonian, shuffle_spectrum_components, spectrum_from_hamiltonian,
    uniqueness_report, FactorizationCandidate, GraphKind, InputMode, SearchMethod, SearchParams,
    UniquenessReport,
};
use crate::geometry::mutual_information_matrix;
use crate::models::{
    heisenberg_chain, random_nonlocal, sparse_local_chain, tfim_chain, tfim_cube, tfim_grid,
    tfim_torus, xx_chain,
};
use crate::quantum::{Hamiltonian, low_energy_states};
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
    pub lx: Option<usize>,
    #[serde(default)]
    pub ly: Option<usize>,
    #[serde(default)]
    pub lz: Option<usize>,
    #[serde(default)]
    pub distance_decay: Option<f64>,
    #[serde(default)]
    pub annealing_steps: Option<usize>,
    #[serde(default)]
    pub spectrum_scramble: Option<bool>,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uniqueness: Option<UniquenessReport>,
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
        Some("torus") => GraphKind::Torus,
        Some("cube") => GraphKind::Cube,
        _ => GraphKind::Line,
    }
}

fn build_params(config: &FactorizationSearchConfig) -> SearchParams {
    let graph_kind = parse_graph_kind(config.graph_kind.as_ref());
    let rows = config.rows.unwrap_or(0);
    let cols = config.cols.unwrap_or(0);
    let lx = config.lx.unwrap_or(0);
    let ly = config.ly.unwrap_or(0);
    let lz = config.lz.unwrap_or(0);
    SearchParams {
        graph_kind,
        rows,
        cols,
        lx,
        ly,
        lz,
        input_mode: parse_input_mode(config.input_mode.as_ref()),
        search_method: parse_search_method(config.search_method.as_ref(), config.n),
        eigenstate_count: config.eigenstate_count.unwrap_or(1).clamp(1, 4),
        distance_decay: config.distance_decay.unwrap_or(0.0),
        annealing_steps: config.annealing_steps.unwrap_or(3000),
        emergent_dim_weight: 0.1,
        spectrum_scrambled: config.spectrum_scramble == Some(true),
    }
}

fn inverse_shuffle(shuffle: &[usize]) -> Vec<usize> {
    let mut inv = vec![0usize; shuffle.len()];
    for (q, &p) in shuffle.iter().enumerate() {
        inv[p] = q;
    }
    inv
}

fn shuffled_local_model(
    kind: &str,
    n: usize,
    field: f64,
    seed: u32,
) -> (String, Hamiltonian, Vec<usize>) {
    match kind {
        "shuffled_chain" => {
            let model = tfim_chain(n, 1.0, field);
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, seed);
            (format!("Shuffled TFIM chain (n={n})"), shuffled, shuffle_perm)
        }
        "shuffled_xx_chain" => {
            let model = xx_chain(n, 1.0, field);
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, seed);
            (format!("Shuffled XX chain (n={n})"), shuffled, shuffle_perm)
        }
        "shuffled_heisenberg_chain" => {
            let model = heisenberg_chain(n, 1.0, field);
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, seed);
            (
                format!("Shuffled Heisenberg chain (n={n})"),
                shuffled,
                shuffle_perm,
            )
        }
        "shuffled_sparse_chain" => {
            let model = sparse_local_chain(n, seed.wrapping_add(17));
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, seed);
            (
                format!("Shuffled sparse local chain (n={n})"),
                shuffled,
                shuffle_perm,
            )
        }
        other => panic!("unknown shuffled local kind: {other}"),
    }
}

pub fn run_factorization_search(config: &FactorizationSearchConfig) -> FactorizationSearchResult {
    let top_k = config.top_k.clamp(1, 20);
    let mut params = build_params(config);

    let (label, hamiltonian, true_shuffle) = match config.kind.as_str() {
        "shuffled_chain"
        | "shuffled_xx_chain"
        | "shuffled_heisenberg_chain"
        | "shuffled_sparse_chain" => {
            let (label, h, shuffle_perm) =
                shuffled_local_model(&config.kind, config.n, config.field, config.seed);
            (label, h, Some(shuffle_perm))
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
        "shuffled_torus" => {
            let rows = config.rows.unwrap_or(2);
            let cols = config.cols.unwrap_or(2);
            params.graph_kind = GraphKind::Torus;
            params.rows = rows;
            params.cols = cols;
            let model = tfim_torus(rows, cols, 1.0, config.field);
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, config.seed);
            (
                format!("Shuffled TFIM torus ({rows}x{cols})"),
                shuffled,
                Some(shuffle_perm),
            )
        }
        "shuffled_cube" => {
            let lx = config.lx.unwrap_or(2);
            let ly = config.ly.unwrap_or(2);
            let lz = config.lz.unwrap_or(2);
            params.graph_kind = GraphKind::Cube;
            params.lx = lx;
            params.ly = ly;
            params.lz = lz;
            let model = tfim_cube(lx, ly, lz, 1.0, config.field);
            let (shuffled, shuffle_perm) = shuffle_hamiltonian(&model.hamiltonian, config.seed);
            (
                format!("Shuffled TFIM cube ({lx}x{ly}x{lz})"),
                shuffled,
                Some(shuffle_perm),
            )
        }
        "random" => {
            let mut rng = Rng::new(config.seed);
            let model = random_nonlocal(config.n, &mut rng);
            (model.label.clone(), model.hamiltonian, None)
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

    let mut spectrum = if params.input_mode == InputMode::Spectrum {
        Some(spectrum_from_hamiltonian(&hamiltonian, k))
    } else {
        None
    };
    if config.spectrum_scramble == Some(true) {
        if let Some(ref mut spec) = spectrum {
            shuffle_spectrum_components(spec, config.seed);
        }
    }

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
        false
    };

    let uniqueness = true_shuffle.as_ref().map(|shuffle| {
        uniqueness_report(&outcome.top_candidates, &inverse_shuffle(shuffle))
    });

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
        uniqueness,
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

    fn spectrum_config(kind: &str, n: usize, seed: u32) -> FactorizationSearchConfig {
        let eigenstate_count = if kind == "shuffled_heisenberg_chain" {
            n.min(6)
        } else if kind == "shuffled_sparse_chain" {
            4
        } else {
            3
        };
        FactorizationSearchConfig {
            kind: kind.to_string(),
            n,
            field: 1.5,
            seed,
            top_k: 5,
            input_mode: Some("spectrum".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(eigenstate_count),
            graph_kind: None,
            rows: None,
            cols: None,
            lx: None,
            ly: None,
            lz: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        }
    }

    #[test]
    fn spectrum_recovers_shuffled_chain_n6() {
        let result = run_factorization_search(&spectrum_config("shuffled_chain", 6, 4242));
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
    fn spectrum_recovers_shuffled_xx_chain_n6() {
        let result = run_factorization_search(&spectrum_config("shuffled_xx_chain", 6, 4242));
        assert!(result.recovered_identity, "xx chain failed");
    }

    #[test]
    fn pauli_recovers_shuffled_heisenberg_chain_n6() {
        let mut config = spectrum_config("shuffled_heisenberg_chain", 6, 4242);
        config.input_mode = Some("pauli".to_string());
        config.eigenstate_count = Some(3);
        let result = run_factorization_search(&config);
        assert!(
            result.recovered_identity,
            "pauli heisenberg failed: dist={:?}",
            result.perm_match_distance
        );
    }

    #[test]
    fn spectrum_recovers_shuffled_sparse_chain_n6() {
        let result =
            run_factorization_search(&spectrum_config("shuffled_sparse_chain", 6, 4242));
        assert!(
            result.recovered_identity,
            "sparse chain failed: dist={:?}",
            result.perm_match_distance
        );
    }

    #[test]
    fn spectrum_scramble_prevents_recovery() {
        let mut config = spectrum_config("shuffled_chain", 6, 4242);
        config.spectrum_scramble = Some(true);
        let result = run_factorization_search(&config);
        assert!(
            !result.recovered_identity,
            "scrambled spectrum should not recover"
        );
    }

    #[test]
    fn line_equiv_distance_accepts_reflection() {
        let a = vec![0, 3, 4, 1, 2, 5];
        let b = vec![5, 2, 1, 4, 3, 0];
        assert_eq!(line_equiv_distance(&a, &b), 0);
    }

    #[test]
    fn pauli_recovers_shuffled_cube_222() {
        let result = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_cube".to_string(),
            n: 8,
            field: 1.5,
            seed: 4242,
            top_k: 5,
            input_mode: Some("pauli".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(1),
            graph_kind: Some("cube".to_string()),
            rows: None,
            cols: None,
            lx: Some(2),
            ly: Some(2),
            lz: Some(2),
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        assert!(
            result.recovered_identity,
            "cube failed: dist={:?} locality={:.3}",
            result.perm_match_distance,
            result.best.locality_fraction
        );
    }

    #[test]
    fn pauli_recovers_shuffled_torus_22() {
        let result = run_factorization_search(&FactorizationSearchConfig {
            kind: "shuffled_torus".to_string(),
            n: 4,
            field: 1.5,
            seed: 4242,
            top_k: 5,
            input_mode: Some("pauli".to_string()),
            search_method: Some("exact".to_string()),
            eigenstate_count: Some(1),
            graph_kind: Some("torus".to_string()),
            rows: Some(2),
            cols: Some(2),
            lx: None,
            ly: None,
            lz: None,
            distance_decay: None,
            annealing_steps: None,
            spectrum_scramble: None,
        });
        assert!(
            result.recovered_identity,
            "torus failed: dist={:?} locality={:.3}",
            result.perm_match_distance,
            result.best.locality_fraction
        );
    }
}
