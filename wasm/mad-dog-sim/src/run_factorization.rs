//! Factorization search runner for WASM.

use crate::factorization::{
    ground_state_for, score_permutation, search_factorization, shuffle_hamiltonian,
    FactorizationCandidate,
};
use crate::models::{random_nonlocal, tfim_chain};
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
    pub true_shuffle: Option<Vec<usize>>,
    pub elapsed_ms: f64,
}

pub fn run_factorization_search(config: &FactorizationSearchConfig) -> FactorizationSearchResult {
    let top_k = config.top_k.max(1).min(20);

    let (label, hamiltonian, true_shuffle) = match config.kind.as_str() {
        "shuffled_chain" => {
            let model = tfim_chain(config.n, 1.0, config.field);
            let (shuffled, shuffle_perm) =
                shuffle_hamiltonian(&model.hamiltonian, config.seed);
            (
                format!("Shuffled TFIM chain (n={})", config.n),
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

    let (state, energy) = ground_state_for(&hamiltonian, config.seed);
    let (baseline, best, top_candidates) =
        search_factorization(&hamiltonian, &state, top_k);

    let recovered_identity = if let Some(ref shuffle) = true_shuffle {
        let n = config.n;
        let mut inv_shuffle = vec![0usize; n];
        for (q, &p) in shuffle.iter().enumerate() {
            inv_shuffle[p] = q;
        }
        let recovery = score_permutation(&hamiltonian, &inv_shuffle, &crate::geometry::mutual_information_matrix(&state));
        recovery.locality_fraction > 0.99 && recovery.nonlocal_terms == 0
    } else {
        best.score > baseline.score + 0.05
    };

    FactorizationSearchResult {
        label,
        n: config.n,
        energy,
        baseline,
        best,
        top_candidates,
        recovered_identity,
        true_shuffle,
        elapsed_ms: 0.0,
    }
}
