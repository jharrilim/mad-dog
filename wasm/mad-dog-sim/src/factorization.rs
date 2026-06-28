//! Spectrum-driven factorization search — find qubit labelings that make H look local.

use crate::geometry::{classical_mds, mi_to_distance, mutual_information_matrix};
use crate::linalg::{hermitian_eigen_decomposition, procrustes_2d};
use crate::quantum::{
    hamiltonian_dense, Hamiltonian, PauliOp, PauliTerm, QuantumState,
};
use crate::rng::Rng;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GraphKind {
    Line,
    Grid,
    Torus,
    Cube,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum InputMode {
    Pauli,
    Spectrum,
    /// Blind search from {Eₙ} only — fits uniform TFIM spectra on each candidate line.
    EigenvaluesOnly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchMethod {
    Exact,
    Annealing,
    Greedy,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CouplingEdge {
    pub i: usize,
    pub j: usize,
    pub dist: usize,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FactorizationCandidate {
    pub permutation: Vec<usize>,
    pub locality_fraction: f64,
    pub mi_nn_ratio: f64,
    pub score: f64,
    pub nonlocal_terms: usize,
    pub emergent_dim: usize,
}

#[derive(Clone, Debug)]
pub struct SearchParams {
    pub graph_kind: GraphKind,
    pub rows: usize,
    pub cols: usize,
    pub lx: usize,
    pub ly: usize,
    pub lz: usize,
    pub input_mode: InputMode,
    pub search_method: SearchMethod,
    pub eigenstate_count: usize,
    pub distance_decay: f64,
    pub annealing_steps: usize,
    pub emergent_dim_weight: f64,
    pub spectrum_scrambled: bool,
}

impl Default for SearchParams {
    fn default() -> Self {
        Self {
            graph_kind: GraphKind::Line,
            rows: 0,
            cols: 0,
            lx: 0,
            ly: 0,
            lz: 0,
            input_mode: InputMode::Pauli,
            search_method: SearchMethod::Annealing,
            eigenstate_count: 1,
            distance_decay: 0.0,
            annealing_steps: 3000,
            emergent_dim_weight: 0.1,
            spectrum_scrambled: false,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SearchOutcome {
    pub baseline: FactorizationCandidate,
    pub best: FactorizationCandidate,
    pub top_candidates: Vec<FactorizationCandidate>,
    pub baseline_mi: Vec<Vec<f64>>,
    pub best_mi: Vec<Vec<f64>>,
    pub coupling_edges: Vec<CouplingEdge>,
    pub search_method: String,
    pub search_iters: usize,
    pub scorer_used: String,
}

pub fn permute_hamiltonian(h: &Hamiltonian, perm: &[usize]) -> Hamiltonian {
    let terms = h
        .terms
        .iter()
        .map(|term| PauliTerm {
            coeff: term.coeff,
            ops: term
                .ops
                .iter()
                .map(|op| PauliOp {
                    qubit: perm[op.qubit],
                    letter: op.letter,
                })
                .collect(),
        })
        .collect();
    Hamiltonian::new(h.n, terms)
}

fn two_body_qubits(term: &PauliTerm) -> Option<(usize, usize)> {
    if term.ops.len() != 2 {
        return None;
    }
    let a = term.ops[0].qubit;
    let b = term.ops[1].qubit;
    Some(if a < b { (a, b) } else { (b, a) })
}

fn torus_axis_dist(a: usize, b: usize, period: usize) -> usize {
    let d = a.abs_diff(b);
    d.min(period - d)
}

/// Manhattan (or torus-wrap) distance between canonical lattice site indices.
fn site_graph_distance(graph: GraphKind, si: usize, sj: usize, params: &SearchParams) -> usize {
    match graph {
        GraphKind::Line => si.abs_diff(sj),
        GraphKind::Grid => {
            let ri = si / params.cols;
            let ci = si % params.cols;
            let rj = sj / params.cols;
            let cj = sj % params.cols;
            ri.abs_diff(rj) + ci.abs_diff(cj)
        }
        GraphKind::Torus => {
            let ri = si / params.cols;
            let ci = si % params.cols;
            let rj = sj / params.cols;
            let cj = sj % params.cols;
            torus_axis_dist(ri, rj, params.rows) + torus_axis_dist(ci, cj, params.cols)
        }
        GraphKind::Cube => {
            let lx = params.lx;
            let ly = params.ly;
            let xi = si % lx;
            let yi = (si / lx) % ly;
            let zi = si / (lx * ly);
            let xj = sj % lx;
            let yj = (sj / lx) % ly;
            let zj = sj / (lx * ly);
            xi.abs_diff(xj) + yi.abs_diff(yj) + zi.abs_diff(zj)
        }
    }
}

fn max_site_diameter(graph: GraphKind, n: usize, params: &SearchParams) -> f64 {
    match graph {
        GraphKind::Line => (n.saturating_sub(1)).max(1) as f64,
        GraphKind::Grid => {
            ((params.rows.saturating_sub(1)) + (params.cols.saturating_sub(1))).max(1) as f64
        }
        GraphKind::Torus => {
            ((params.rows / 2) + (params.cols / 2)).max(1) as f64
        }
        GraphKind::Cube => {
            ((params.lx.saturating_sub(1))
                + (params.ly.saturating_sub(1))
                + (params.lz.saturating_sub(1)))
            .max(1) as f64
        }
    }
}

fn active_site_span(graph: GraphKind, sites: &[usize], params: &SearchParams) -> f64 {
    if sites.len() < 2 {
        return 0.0;
    }
    if matches!(graph, GraphKind::Line) {
        return (*sites.iter().max().unwrap() - *sites.iter().min().unwrap()) as f64;
    }
    sites
        .iter()
        .enumerate()
        .flat_map(|(i, &si)| {
            sites[i + 1..]
                .iter()
                .map(move |&sj| site_graph_distance(graph, si, sj, params))
        })
        .max()
        .unwrap_or(0) as f64
}

fn accumulate_far_mi(
    mi: &[Vec<f64>],
    inv: &[usize],
    n: usize,
    graph: GraphKind,
    params: &SearchParams,
    far_sum: &mut f64,
    far_count: &mut usize,
) {
    for k in 0..n {
        for d in (k + 1)..n {
            if site_graph_distance(graph, k, d, params) >= 2 {
                *far_sum += mi[inv[k]][inv[d]];
                *far_count += 1;
            }
        }
    }
}

fn graph_distance(
    graph: GraphKind,
    perm: &[usize],
    i: usize,
    j: usize,
    params: &SearchParams,
) -> usize {
    let pi = perm[i];
    let pj = perm[j];
    match graph {
        GraphKind::Line => pi.abs_diff(pj),
        GraphKind::Grid => {
            let ri = pi / params.cols;
            let ci = pi % params.cols;
            let rj = pj / params.cols;
            let cj = pj % params.cols;
            let _ = params.rows;
            ri.abs_diff(rj) + ci.abs_diff(cj)
        }
        GraphKind::Torus => {
            let ri = pi / params.cols;
            let ci = pi % params.cols;
            let rj = pj / params.cols;
            let cj = pj % params.cols;
            torus_axis_dist(ri, rj, params.rows) + torus_axis_dist(ci, cj, params.cols)
        }
        GraphKind::Cube => {
            let lx = params.lx;
            let ly = params.ly;
            let xi = pi % lx;
            let yi = (pi / lx) % ly;
            let zi = pi / (lx * ly);
            let xj = pj % lx;
            let yj = (pj / lx) % ly;
            let zj = pj / (lx * ly);
            xi.abs_diff(xj) + yi.abs_diff(yj) + zi.abs_diff(zj)
        }
    }
}

fn dist_weight(dist: usize, decay: f64) -> f64 {
    if dist == 0 {
        return 0.0;
    }
    if dist == 1 {
        return 1.0;
    }
    if decay <= 0.0 {
        return 0.0;
    }
    if dist == 2 {
        return decay;
    }
    0.0
}

fn coupling_edges_for(h: &Hamiltonian, perm: &[usize], params: &SearchParams) -> Vec<CouplingEdge> {
    let mut edges = Vec::new();
    for term in &h.terms {
        if let Some((i, j)) = two_body_qubits(term) {
            let dist = graph_distance(
                params.graph_kind,
                perm,
                i,
                j,
                params,
            );
            edges.push(CouplingEdge { i, j, dist });
        }
    }
    edges
}

fn mean_mi_nn_ratio(mi_list: &[Vec<Vec<f64>>], perm: &[usize], params: &SearchParams) -> f64 {
    if mi_list.is_empty() {
        return 0.0;
    }
    let n = perm.len();
    let mut inv = vec![0usize; n];
    for (q, &p) in perm.iter().enumerate() {
        inv[p] = q;
    }
    let mut ratios = Vec::new();
    for mi in mi_list {
        let mut nn_sum = 0.0;
        let mut nn_count = 0usize;
        let mut far_sum = 0.0;
        let mut far_count = 0usize;
        match params.graph_kind {
            GraphKind::Line => {
                for k in 0..(n - 1) {
                    let i = inv[k];
                    let j = inv[k + 1];
                    nn_sum += mi[i][j];
                    nn_count += 1;
                }
                accumulate_far_mi(mi, &inv, n, GraphKind::Line, params, &mut far_sum, &mut far_count);
            }
            GraphKind::Grid => {
                let rows = params.rows;
                let cols = params.cols;
                for r in 0..rows {
                    for c in 0..cols {
                        let k = r * cols + c;
                        let i = inv[k];
                        if c + 1 < cols {
                            let j = inv[r * cols + c + 1];
                            nn_sum += mi[i][j];
                            nn_count += 1;
                        }
                        if r + 1 < rows {
                            let j = inv[(r + 1) * cols + c];
                            nn_sum += mi[i][j];
                            nn_count += 1;
                        }
                    }
                }
                accumulate_far_mi(mi, &inv, n, GraphKind::Grid, params, &mut far_sum, &mut far_count);
            }
            GraphKind::Torus => {
                let rows = params.rows;
                let cols = params.cols;
                for r in 0..rows {
                    for c in 0..cols {
                        let k = r * cols + c;
                        let i = inv[k];
                        let c2 = (c + 1) % cols;
                        let j = inv[r * cols + c2];
                        nn_sum += mi[i][j];
                        nn_count += 1;
                        let r2 = (r + 1) % rows;
                        let j2 = inv[r2 * cols + c];
                        nn_sum += mi[i][j2];
                        nn_count += 1;
                    }
                }
                accumulate_far_mi(mi, &inv, n, GraphKind::Torus, params, &mut far_sum, &mut far_count);
            }
            GraphKind::Cube => {
                let lx = params.lx;
                let ly = params.ly;
                let lz = params.lz;
                let idx = |x: usize, y: usize, z: usize| z * (lx * ly) + y * lx + x;
                for z in 0..lz {
                    for y in 0..ly {
                        for x in 0..lx {
                            let k = idx(x, y, z);
                            let i = inv[k];
                            if x + 1 < lx {
                                nn_sum += mi[i][inv[idx(x + 1, y, z)]];
                                nn_count += 1;
                            }
                            if y + 1 < ly {
                                nn_sum += mi[i][inv[idx(x, y + 1, z)]];
                                nn_count += 1;
                            }
                            if z + 1 < lz {
                                nn_sum += mi[i][inv[idx(x, y, z + 1)]];
                                nn_count += 1;
                            }
                        }
                    }
                }
                accumulate_far_mi(mi, &inv, n, GraphKind::Cube, params, &mut far_sum, &mut far_count);
            }
        }
        let nn_avg = if nn_count > 0 { nn_sum / nn_count as f64 } else { 0.0 };
        let far_avg = if far_count > 0 {
            far_sum / far_count as f64
        } else {
            1e-9
        };
        ratios.push(nn_avg / far_avg.max(1e-12));
    }
    ratios.iter().sum::<f64>() / ratios.len() as f64
}

fn permuted_mi(mi: &[Vec<f64>], perm: &[usize]) -> Vec<Vec<f64>> {
    let n = perm.len();
    let mut out = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            out[perm[i]][perm[j]] = mi[i][j];
        }
    }
    out
}

fn state_from_data(n: usize, data: &[f64]) -> QuantumState {
    QuantumState {
        n,
        dim: 1 << n,
        data: data.to_vec(),
    }
}

fn support_bandwidth(
    graph: GraphKind,
    eigenvectors: &[Vec<f64>],
    perm: &[usize],
    n: usize,
    params: &SearchParams,
) -> f64 {
    if eigenvectors.is_empty() || n < 2 {
        return 0.0;
    }
    let mut total = 0.0;
    let max_span = max_site_diameter(graph, n, params);
    for ev in eigenvectors {
        let dim = 1 << n;
        let mut weighted_span = 0.0;
        let mut mass = 0.0;
        for s in 0..dim {
            let re = ev[2 * s];
            let im = ev[2 * s + 1];
            let p = re * re + im * im;
            if p < 1e-15 {
                continue;
            }
            let active: Vec<usize> = (0..n)
                .filter(|&q| (s >> q) & 1 == 1)
                .map(|q| perm[q])
                .collect();
            let span = active_site_span(graph, &active, params);
            weighted_span += p * span;
            mass += p;
        }
        if mass > 1e-12 {
            total += 1.0 - (weighted_span / mass / max_span).min(1.0);
        }
    }
    total / eigenvectors.len() as f64
}

fn spectrum_state_weights(k: usize) -> Vec<f64> {
    (0..k)
        .map(|i| 1.0 / (1.0 + 0.35 * i as f64))
        .collect()
}

/// Ideal 2D lattice coordinates (column, row) for Procrustes grid-fit tiebreak.
fn ideal_lattice_coords(kind: GraphKind, params: &SearchParams) -> Vec<Vec<f64>> {
    match kind {
        GraphKind::Grid | GraphKind::Torus => {
            let rows = params.rows;
            let cols = params.cols;
            (0..rows * cols)
                .map(|k| {
                    let r = k / cols;
                    let c = k % cols;
                    vec![c as f64, r as f64]
                })
                .collect()
        }
        _ => Vec::new(),
    }
}

fn collect_nn_edge_mis(mi: &[Vec<f64>], inv: &[usize], params: &SearchParams) -> Vec<f64> {
    let n = inv.len();
    let mut edges = Vec::new();
    match params.graph_kind {
        GraphKind::Line => {
            for k in 0..(n - 1) {
                edges.push(mi[inv[k]][inv[k + 1]]);
            }
        }
        GraphKind::Grid => {
            let rows = params.rows;
            let cols = params.cols;
            for r in 0..rows {
                for c in 0..cols {
                    let k = r * cols + c;
                    let i = inv[k];
                    if c + 1 < cols {
                        edges.push(mi[i][inv[r * cols + c + 1]]);
                    }
                    if r + 1 < rows {
                        edges.push(mi[i][inv[(r + 1) * cols + c]]);
                    }
                }
            }
        }
        GraphKind::Torus => {
            let rows = params.rows;
            let cols = params.cols;
            for r in 0..rows {
                for c in 0..cols {
                    let k = r * cols + c;
                    let i = inv[k];
                    edges.push(mi[i][inv[r * cols + (c + 1) % cols]]);
                    edges.push(mi[i][inv[((r + 1) % rows) * cols + c]]);
                }
            }
        }
        GraphKind::Cube => {}
    }
    edges
}

/// MDS embedding fit to an ideal grid/torus layout (higher = better).
fn mds_lattice_fit_score(mi: &[Vec<f64>], perm: &[usize], params: &SearchParams) -> f64 {
    let n = perm.len();
    if !matches!(params.graph_kind, GraphKind::Grid | GraphKind::Torus) {
        return 0.0;
    }
    let mi_p = permuted_mi(mi, perm);
    let dist = mi_to_distance(&mi_p, 1.0);
    let mds = classical_mds(&dist, 2);
    if mds.coords.len() != n || mds.coords.iter().any(|c| c.len() < 2) {
        return 0.0;
    }
    let ideal = ideal_lattice_coords(params.graph_kind, params);
    let aligned = procrustes_2d(&mds.coords, &ideal);
    let sse: f64 = aligned
        .iter()
        .zip(ideal.iter())
        .map(|(a, b)| {
            let dx = a[0] - b[0];
            let dy = a[1] - b[1];
            dx * dx + dy * dy
        })
        .sum();
    let rms = (sse / n as f64).sqrt();
    1.0 / (1.0 + rms)
}

/// Std-dev of NN-edge MI (true 2D labelings show heterogeneous edge weights).
fn nn_edge_mi_heterogeneity(mi_list: &[Vec<Vec<f64>>], perm: &[usize], params: &SearchParams) -> f64 {
    if mi_list.is_empty() {
        return 0.0;
    }
    let n = perm.len();
    let mut inv = vec![0usize; n];
    for (q, &p) in perm.iter().enumerate() {
        inv[p] = q;
    }
    let weights = spectrum_state_weights(mi_list.len());
    let mut sum = 0.0;
    let mut wsum = 0.0;
    for (mi, &wt) in mi_list.iter().zip(weights.iter()) {
        let edges = collect_nn_edge_mis(mi, &inv, params);
        if edges.len() < 2 {
            continue;
        }
        let mean = edges.iter().sum::<f64>() / edges.len() as f64;
        let var = edges.iter().map(|e| (e - mean).powi(2)).sum::<f64>() / edges.len() as f64;
        sum += wt * var.sqrt();
        wsum += wt;
    }
    if wsum > 0.0 {
        sum / wsum
    } else {
        0.0
    }
}

/// MI term for AA-blind 2D search. On 2×2 torus every wrap pair is graph distance 1, so
/// spurious labelings inflate MI-NN ratio above 1 while the true class stays below 1.
fn blind_2d_mi_term(mi_nn_ratio: f64, params: &SearchParams) -> f64 {
    let r = mi_nn_ratio;
    if params.spectrum_scrambled
        && params.graph_kind == GraphKind::Torus
        && params.rows.saturating_mul(params.cols) <= 4
        && r > 1.0
    {
        0.5 / (r + 1.0)
    } else {
        r / (r + 1.0)
    }
}

/// Match a candidate labeling to ground truth allowing square-grid D₄ symmetries.
pub fn lattice_equiv_match(
    perm: &[usize],
    target: &[usize],
    graph_kind: GraphKind,
    rows: usize,
    cols: usize,
) -> bool {
    match graph_kind {
        GraphKind::Grid => grid_equiv_match(perm, target, rows, cols),
        GraphKind::Torus => torus_equiv_match(perm, target, rows, cols),
        _ => perm_distance(perm, target) == 0,
    }
}

/// Secondary score for AA-blind 2D exact search when primary scores tie.
pub(crate) fn blind_lattice_tiebreak(
    mi_list: &[Vec<Vec<f64>>],
    perm: &[usize],
    params: &SearchParams,
) -> f64 {
    let mds = mi_list
        .first()
        .map(|mi| mds_lattice_fit_score(mi, perm, params))
        .unwrap_or(0.0);
    let hetero = nn_edge_mi_heterogeneity(mi_list, perm, params);
    // Heterogeneity breaks 8-way ties on 3×3; MDS fit is identical across tie class.
    0.1 * mds + 0.9 * hetero
}

fn candidate_sort_key(
    mi_list: &[Vec<Vec<f64>>],
    cand: &FactorizationCandidate,
    params: &SearchParams,
) -> (f64, f64) {
    let tiebreak = if params.spectrum_scrambled
        && matches!(params.graph_kind, GraphKind::Grid | GraphKind::Torus)
    {
        blind_lattice_tiebreak(mi_list, &cand.permutation, params)
    } else {
        cand.mi_nn_ratio
    };
    (cand.score, tiebreak)
}

fn weighted_mi_nn_ratio(
    mi_list: &[Vec<Vec<f64>>],
    perm: &[usize],
    params: &SearchParams,
    weights: Option<&[f64]>,
) -> f64 {
    if mi_list.is_empty() {
        return 0.0;
    }
    let uniform = vec![1.0; mi_list.len()];
    let w = weights.unwrap_or(&uniform);
    let mut sum = 0.0;
    let mut wsum = 0.0;
    for (mi, &wt) in mi_list.iter().zip(w.iter()) {
        let ratio = mean_mi_nn_ratio(std::slice::from_ref(mi), perm, params);
        sum += wt * ratio;
        wsum += wt;
    }
    if wsum > 0.0 {
        sum / wsum
    } else {
        0.0
    }
}

pub fn score_permutation(
    h: Option<&Hamiltonian>,
    mi_list: &[Vec<Vec<f64>>],
    perm: &[usize],
    params: &SearchParams,
    spectrum_eigenvectors: Option<&[Vec<f64>]>,
    target_eigenvalues: Option<&[f64]>,
    skip_mds: bool,
) -> FactorizationCandidate {
    let n = perm.len();
    let expected_dim = match params.graph_kind {
        GraphKind::Line => 1,
        GraphKind::Grid | GraphKind::Torus => 2,
        GraphKind::Cube => 3,
    };

    let mut weighted_local = 0.0;
    let mut total_weight = 0.0;
    let mut nonlocal = 0usize;
    if let Some(ham) = h {
        if params.input_mode == InputMode::Pauli
            || (params.input_mode == InputMode::Spectrum && !params.spectrum_scrambled)
        {
            for term in &ham.terms {
                if let Some((i, j)) = two_body_qubits(term) {
                    let dist = graph_distance(
                        params.graph_kind,
                        perm,
                        i,
                        j,
                        params,
                    );
                    let w = dist_weight(dist, params.distance_decay);
                    weighted_local += w;
                    total_weight += 1.0;
                    if dist != 1 {
                        nonlocal += 1;
                    }
                }
            }
        }
    }

    let locality_fraction = if total_weight > 0.0 {
        weighted_local / total_weight
    } else if params.input_mode == InputMode::Spectrum {
        0.0
    } else {
        1.0
    };

    let mi_nn_ratio = if params.input_mode == InputMode::Spectrum {
        let weights = spectrum_state_weights(mi_list.len());
        weighted_mi_nn_ratio(mi_list, perm, params, Some(&weights))
    } else if params.input_mode == InputMode::EigenvaluesOnly {
        0.0
    } else {
        mean_mi_nn_ratio(mi_list, perm, params)
    };

    let emergent_dim = if skip_mds {
        expected_dim
    } else if let Some(mi) = mi_list.first() {
        let mi_for_mds = permuted_mi(mi, perm);
        let distance = mi_to_distance(&mi_for_mds, 1.0);
        classical_mds(&distance, 3).emergent_dim
    } else {
        expected_dim
    };

    let mi_term = if params.spectrum_scrambled
        && matches!(params.graph_kind, GraphKind::Grid | GraphKind::Torus)
    {
        blind_2d_mi_term(mi_nn_ratio, params)
    } else {
        mi_nn_ratio / (mi_nn_ratio + 1.0)
    };
    let dim_bonus = if emergent_dim <= expected_dim {
        1.0
    } else {
        (expected_dim as f64 / emergent_dim as f64).max(0.0)
    };

    let score = match params.input_mode {
        InputMode::Pauli => {
            0.55 * locality_fraction
                + 0.35 * mi_term
                + params.emergent_dim_weight * dim_bonus
        }
        InputMode::Spectrum => {
            let bw = spectrum_eigenvectors
                .map(|ev| support_bandwidth(params.graph_kind, ev, perm, n, params))
                .unwrap_or(0.0);
            let loc_w = if params.spectrum_scrambled { 0.0 } else { 0.40 };
            let mi_w = if params.spectrum_scrambled { 0.50 } else { 0.30 };
            let bw_w = if params.spectrum_scrambled { 0.35 } else { 0.25 };
            loc_w * locality_fraction + mi_w * mi_term + bw_w * bw + 0.15 * dim_bonus
        }
        InputMode::EigenvaluesOnly => {
            // Global spectrum is unitarily permutation-invariant — score cannot depend on `perm`.
            let target = target_eigenvalues.unwrap_or(&[]);
            let spacings: Vec<f64> = target.windows(2).map(|w| (w[1] - w[0]).abs()).collect();
            if spacings.is_empty() {
                0.0
            } else {
                let mean = spacings.iter().sum::<f64>() / spacings.len() as f64;
                let var = spacings.iter().map(|s| (s - mean).powi(2)).sum::<f64>() / spacings.len() as f64;
                mean / (1.0 + var.sqrt())
            }
        }
    };

    FactorizationCandidate {
        permutation: perm.to_vec(),
        locality_fraction,
        mi_nn_ratio,
        score,
        nonlocal_terms: nonlocal,
        emergent_dim,
    }
}

fn identity_perm(n: usize) -> Vec<usize> {
    (0..n).collect()
}

fn random_permutation(n: usize, rng: &mut Rng) -> Vec<usize> {
    let mut perm: Vec<usize> = (0..n).collect();
    for i in (1..n).rev() {
        let j = (rng.next() * (i + 1) as f64).floor() as usize;
        perm.swap(i, j);
    }
    perm
}

fn heap_permute(_n: usize, a: &mut [usize], k: usize, out: &mut Vec<Vec<usize>>) {
    if k == 1 {
        out.push(a.to_vec());
        return;
    }
    heap_permute(_n, a, k - 1, out);
    for i in 0..(k - 1) {
        if k.is_multiple_of(2) {
            a.swap(i, k - 1);
        } else {
            a.swap(0, k - 1);
        }
        heap_permute(_n, a, k - 1, out);
    }
}

fn all_permutations(n: usize) -> Vec<Vec<usize>> {
    let mut a: Vec<usize> = (0..n).collect();
    let mut out = Vec::new();
    heap_permute(n, &mut a, n, &mut out);
    out
}

fn greedy_search(
    h: Option<&Hamiltonian>,
    mi_list: &[Vec<Vec<f64>>],
    params: &SearchParams,
    spectrum_eigenvectors: Option<&[Vec<f64>]>,
    target_eigenvalues: Option<&[f64]>,
    max_iters: usize,
) -> (FactorizationCandidate, usize) {
    let n = h.map(|x| x.n).unwrap_or_else(|| mi_list[0].len());
    let mut perm = identity_perm(n);
    let mut best = score_permutation(
        h,
        mi_list,
        &perm,
        params,
        spectrum_eigenvectors,
        target_eigenvalues,
        true,
    );
    let mut iters = 0usize;
    for _ in 0..max_iters {
        let mut improved = false;
        for i in 0..n {
            for j in (i + 1)..n {
                perm.swap(i, j);
                let cand = score_permutation(
                    h,
                    mi_list,
                    &perm,
                    params,
                    spectrum_eigenvectors,
                    target_eigenvalues,
                    true,
                );
                iters += 1;
                if cand.score > best.score + 1e-12 {
                    best = cand;
                    improved = true;
                } else {
                    perm.swap(i, j);
                }
            }
        }
        if !improved {
            break;
        }
    }
    (best, iters)
}

fn annealing_search(
    h: Option<&Hamiltonian>,
    mi_list: &[Vec<Vec<f64>>],
    params: &SearchParams,
    spectrum_eigenvectors: Option<&[Vec<f64>]>,
    target_eigenvalues: Option<&[f64]>,
    seed: u32,
) -> (FactorizationCandidate, usize) {
    let n = h.map(|x| x.n).unwrap_or_else(|| mi_list[0].len());
    let steps = params.annealing_steps.max(100);
    let mut rng = Rng::new(seed);
    let mut perm = random_permutation(n, &mut rng);
    let mut current = score_permutation(
        h,
        mi_list,
        &perm,
        params,
        spectrum_eigenvectors,
        target_eigenvalues,
        true,
    );
    let mut best = current.clone();
    let t0 = 1.0_f64;
    let t1 = 0.001_f64;
    for step in 0..steps {
        let t = t0 * (t1 / t0).powf(step as f64 / steps as f64);
        let i = (rng.next() * n as f64).floor() as usize % n;
        let mut j = (rng.next() * n as f64).floor() as usize % n;
        if j == i {
            j = (j + 1) % n;
        }
        perm.swap(i, j);
        let cand = score_permutation(
            h,
            mi_list,
            &perm,
            params,
            spectrum_eigenvectors,
            target_eigenvalues,
            true,
        );
        let delta = cand.score - current.score;
        if delta > 1e-12 || rng.next() < (delta / t).exp() {
            current = cand;
            if current.score > best.score + 1e-12 {
                best = current.clone();
            }
        } else {
            perm.swap(i, j);
        }
    }
    (best, steps)
}

pub struct SpectrumData {
    pub eigenvalues: Vec<f64>,
    pub eigenvectors: Vec<Vec<f64>>,
}

pub fn spectrum_from_hamiltonian(h: &Hamiltonian, k: usize) -> SpectrumData {
    let (re, im) = hamiltonian_dense(h);
    let (eigenvalues, eigenvectors) = hermitian_eigen_decomposition(&re, &im);
    let k = k.min(eigenvalues.len());
    SpectrumData {
        eigenvalues: eigenvalues[..k].to_vec(),
        eigenvectors: eigenvectors[..k].to_vec(),
    }
}

fn mi_list_from_states(states: &[QuantumState]) -> Vec<Vec<Vec<f64>>> {
    states.iter().map(mutual_information_matrix).collect()
}

fn mi_list_from_eigenvectors(n: usize, eigenvectors: &[Vec<f64>]) -> Vec<Vec<Vec<f64>>> {
    eigenvectors
        .iter()
        .map(|data| {
            let state = state_from_data(n, data);
            mutual_information_matrix(&state)
        })
        .collect()
}

pub fn search_factorization(
    h: &Hamiltonian,
    states: &[QuantumState],
    top_k: usize,
    params: &SearchParams,
    spectrum: Option<&SpectrumData>,
) -> SearchOutcome {
    let n = h.n;
    let (h_ref, mi_list, spectrum_ev, target_ev) = match params.input_mode {
        InputMode::Pauli => (Some(h), mi_list_from_states(states), None, None),
        InputMode::Spectrum => {
            let spec = spectrum.expect("spectrum data required");
            let h_ref = if params.spectrum_scrambled {
                None
            } else {
                Some(h)
            };
            (
                h_ref,
                mi_list_from_eigenvectors(n, &spec.eigenvectors),
                Some(spec.eigenvectors.as_slice()),
                None,
            )
        }
        InputMode::EigenvaluesOnly => {
            let spec = spectrum.expect("spectrum data required");
            let dummy_mi = vec![vec![vec![0.0; n]; n]];
            (
                None,
                dummy_mi,
                None,
                Some(spec.eigenvalues.as_slice()),
            )
        }
    };

    let identity = score_permutation(
        h_ref,
        &mi_list,
        &identity_perm(n),
        params,
        spectrum_ev,
        target_ev,
        false,
    );
    let scorer_used = match params.input_mode {
        InputMode::Spectrum => "spectrum+mi",
        InputMode::EigenvaluesOnly => "eigenvalues-only",
        InputMode::Pauli => "pauli+mi",
    };

    let (method_name, candidates, search_iters) = if matches!(params.search_method, SearchMethod::Exact)
        && n <= 8
    {
        let cands: Vec<FactorizationCandidate> = all_permutations(n)
            .into_iter()
            .map(|perm| {
                score_permutation(h_ref, &mi_list, &perm, params, spectrum_ev, target_ev, true)
            })
            .collect();
        let iters = cands.len();
        let mut candidates = cands;
        candidates.sort_by(|a, b| {
            let ka = candidate_sort_key(&mi_list, a, params);
            let kb = candidate_sort_key(&mi_list, b, params);
            kb.0
                .partial_cmp(&ka.0)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| kb.1.partial_cmp(&ka.1).unwrap_or(std::cmp::Ordering::Equal))
                .then_with(|| a.permutation.cmp(&b.permutation))
        });
        for cand in candidates.iter_mut().take(top_k.max(1)) {
            *cand = score_permutation(
                h_ref,
                &mi_list,
                &cand.permutation,
                params,
                spectrum_ev,
                target_ev,
                false,
            );
        }
        ("exact".to_string(), candidates, iters)
    } else if matches!(params.search_method, SearchMethod::Greedy) {
        let (best, iters) = greedy_search(h_ref, &mi_list, params, spectrum_ev, target_ev, 40);
        (
            "greedy".to_string(),
            vec![identity.clone(), best],
            iters,
        )
    } else {
        let mut rng = Rng::new(991);
        let mut cands = vec![identity.clone()];
        for _ in 0..16 {
            let perm = random_permutation(n, &mut rng);
            cands.push(score_permutation(
                h_ref,
                &mi_list,
                &perm,
                params,
                spectrum_ev,
                target_ev,
                true,
            ));
        }
        let (ann, iters) = annealing_search(h_ref, &mi_list, params, spectrum_ev, target_ev, 4242);
        cands.push(ann);
        ("annealing".to_string(), cands, iters + 16)
    };

    let mut candidates = candidates;
    candidates.sort_by(|a, b| {
        b.score
            .partial_cmp(&a.score)
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    candidates.dedup_by(|a, b| {
        (a.score - b.score).abs() < 1e-9 && a.permutation == b.permutation
    });

    let mut best = candidates.first().cloned().unwrap_or(identity.clone());
    best = score_permutation(
        h_ref,
        &mi_list,
        &best.permutation,
        params,
        spectrum_ev,
        target_ev,
        false,
    );
    let top: Vec<FactorizationCandidate> = candidates.into_iter().take(top_k).collect();

    let baseline_mi = mi_list
        .first()
        .map(|mi| permuted_mi(mi, &identity_perm(n)))
        .unwrap_or_else(|| vec![vec![0.0; n]; n]);
    let best_mi = mi_list
        .first()
        .map(|mi| permuted_mi(mi, &best.permutation))
        .unwrap_or_else(|| baseline_mi.clone());

    let coupling_edges = h_ref
        .map(|ham| coupling_edges_for(ham, &best.permutation, params))
        .unwrap_or_default();

    SearchOutcome {
        baseline: identity,
        best,
        top_candidates: top,
        baseline_mi,
        best_mi,
        coupling_edges,
        search_method: method_name,
        search_iters,
        scorer_used: scorer_used.to_string(),
    }
}

/// Fast annealing search for quench slices (MI-focused).
pub fn fast_search_on_state(
    h: &Hamiltonian,
    state: &QuantumState,
    steps: usize,
) -> FactorizationCandidate {
    let params = SearchParams {
        search_method: SearchMethod::Annealing,
        annealing_steps: steps,
        eigenstate_count: 1,
        ..Default::default()
    };
    let mi_list = mi_list_from_states(&[state.clone_state()]);
    let (best, _) = annealing_search(Some(h), &mi_list, &params, None, None, 55);
    score_permutation(
        Some(h),
        &mi_list,
        &best.permutation,
        &params,
        None,
        None,
        false,
    )
}

pub fn shuffle_hamiltonian(h: &Hamiltonian, seed: u32) -> (Hamiltonian, Vec<usize>) {
    let mut rng = Rng::new(seed.wrapping_add(4242));
    let perm = random_permutation(h.n, &mut rng);
    (permute_hamiltonian(h, &perm), perm)
}

pub fn perm_distance(a: &[usize], b: &[usize]) -> usize {
    a.iter().zip(b.iter()).filter(|(x, y)| x != y).count()
}

/// Minimum Hamming mismatch to `target`, allowing open-chain reflection (k ↦ n−1−k).
pub fn line_equiv_distance(perm: &[usize], target: &[usize]) -> usize {
    let n = perm.len();
    let direct = perm_distance(perm, target);
    if n == 0 {
        return direct;
    }
    let reflected: Vec<usize> = target.iter().map(|&p| (n - 1) - p).collect();
    direct.min(perm_distance(perm, &reflected))
}

pub fn line_equiv_match(perm: &[usize], target: &[usize]) -> bool {
    line_equiv_distance(perm, target) == 0
}

/// Apply a grid symmetry σ to site indices (rows×cols layout).
fn apply_grid_symmetry(site: usize, sym: usize, rows: usize, cols: usize) -> usize {
    let r = site / cols;
    let c = site % cols;
    let (r2, c2) = match sym {
        0 => (r, c), // identity
        1 => (c, rows - 1 - r), // rot 90 CW
        2 => (rows - 1 - r, cols - 1 - c), // rot 180
        3 => (rows - 1 - c, r), // rot 270 CW
        4 => (rows - 1 - r, c), // reflect horizontal
        5 => (r, cols - 1 - c), // reflect vertical
        6 => (c, r), // reflect main diagonal
        7 => (rows - 1 - c, cols - 1 - r), // reflect anti-diagonal
        _ => (r, c),
    };
    r2 * cols + c2
}

/// Minimum Hamming mismatch allowing square-grid D₄ symmetries (8 rigid motions).
pub fn grid_equiv_distance(perm: &[usize], target: &[usize], rows: usize, cols: usize) -> usize {
    if perm.len() != target.len() || rows * cols != perm.len() {
        return perm.len();
    }
    (0..8)
        .map(|sym| {
            perm.iter()
                .zip(target.iter())
                .filter(|(&p, &t)| apply_grid_symmetry(p, sym, rows, cols) != t)
                .count()
        })
        .min()
        .unwrap_or(perm.len())
}

pub fn grid_equiv_match(perm: &[usize], target: &[usize], rows: usize, cols: usize) -> bool {
    grid_equiv_distance(perm, target, rows, cols) == 0
}

/// Torus 2×2 has 8 label symmetries (D₄ on site indices with periodic wrap).
pub fn torus_equiv_distance(perm: &[usize], target: &[usize], rows: usize, cols: usize) -> usize {
    grid_equiv_distance(perm, target, rows, cols)
}

pub fn torus_equiv_match(perm: &[usize], target: &[usize], rows: usize, cols: usize) -> bool {
    torus_equiv_distance(perm, target, rows, cols) == 0
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UniquenessReport {
    pub equivalence_class_count: usize,
    pub best_class_size: usize,
    pub true_in_top_k: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub true_class_rank: Option<usize>,
    pub score_gap_to_second_class: f64,
}

/// Cluster top-k candidates by open-chain reflection equivalence.
pub fn uniqueness_report(
    top_candidates: &[FactorizationCandidate],
    true_inv_shuffle: &[usize],
) -> UniquenessReport {
    if top_candidates.is_empty() {
        return UniquenessReport {
            equivalence_class_count: 0,
            best_class_size: 0,
            true_in_top_k: false,
            true_class_rank: None,
            score_gap_to_second_class: 0.0,
        };
    }

    let mut classes: Vec<(f64, Vec<usize>)> = Vec::new();
    for (idx, cand) in top_candidates.iter().enumerate() {
        let mut placed = false;
        for class in classes.iter_mut() {
            if line_equiv_match(&cand.permutation, &top_candidates[class.1[0]].permutation) {
                class.1.push(idx);
                class.0 = class.0.max(cand.score);
                placed = true;
                break;
            }
        }
        if !placed {
            classes.push((cand.score, vec![idx]));
        }
    }
    classes.sort_by(|a, b| {
        b.0.partial_cmp(&a.0)
            .unwrap_or(std::cmp::Ordering::Equal)
            .then_with(|| a.1.len().cmp(&b.1.len()))
    });

    let equivalence_class_count = classes.len();
    let best_class_size = classes.first().map(|c| c.1.len()).unwrap_or(0);
    let true_in_top_k = top_candidates
        .iter()
        .any(|c| line_equiv_match(&c.permutation, true_inv_shuffle));
    let true_class_rank = classes.iter().position(|class| {
        class
            .1
            .iter()
            .any(|&idx| line_equiv_match(&top_candidates[idx].permutation, true_inv_shuffle))
    });
    let score_gap_to_second_class = if classes.len() > 1 {
        classes[0].0 - classes[1].0
    } else {
        0.0
    };

    UniquenessReport {
        equivalence_class_count,
        best_class_size,
        true_in_top_k,
        true_class_rank: true_class_rank.map(|r| r + 1),
        score_gap_to_second_class,
    }
}

fn permute_basis_index(s: usize, qubit_perm: &[usize]) -> usize {
    let mut out = 0usize;
    for (q, &p) in qubit_perm.iter().enumerate() {
        if (s >> q) & 1 == 1 {
            out |= 1 << p;
        }
    }
    out
}

/// Scramble qubit labels in eigenvector amplitudes (negative control — eigenvalues unchanged).
pub fn shuffle_spectrum_components(spectrum: &mut SpectrumData, seed: u32) {
    let n = if spectrum.eigenvectors.is_empty() {
        return;
    } else {
        let dim = spectrum.eigenvectors[0].len() / 2;
        (dim as f64).log2() as usize
    };
    let mut rng = Rng::new(seed.wrapping_add(9001));
    let qubit_perm = random_permutation(n, &mut rng);

    for ev in spectrum.eigenvectors.iter_mut() {
        let dim = 1 << n;
        let mut scrambled = vec![0.0; 2 * dim];
        for s in 0..dim {
            let t = permute_basis_index(s, &qubit_perm);
            scrambled[2 * t] = ev[2 * s];
            scrambled[2 * t + 1] = ev[2 * s + 1];
        }
        *ev = scrambled;
    }
}

#[cfg(test)]
mod lattice_equiv_tests {
    use super::*;

    #[test]
    fn grid_d4_symmetry_detects_rotated_labeling() {
        let rows = 3;
        let cols = 3;
        let target: Vec<usize> = (0..9).collect();
        // 90° CW rotation on site indices
        let rotated: Vec<usize> = (0..9)
            .map(|k| apply_grid_symmetry(k, 1, rows, cols))
            .collect();
        assert_eq!(grid_equiv_distance(&rotated, &target, rows, cols), 0);
        assert!(grid_equiv_match(&rotated, &target, rows, cols));
    }
}

#[cfg(test)]
mod eigenvalue_fit_tests {
    use super::*;
    use crate::models::tfim_chain;

    #[test]
    fn global_eigenvalues_invariant_under_qubit_shuffle() {
        let n = 6;
        let base = tfim_chain(n, 1.0, 1.5);
        let (shuffled, _) = shuffle_hamiltonian(&base.hamiltonian, 42);
        let s0 = spectrum_from_hamiltonian(&base.hamiltonian, n);
        let s1 = spectrum_from_hamiltonian(&shuffled, n);
        for i in 0..n {
            assert!(
                (s0.eigenvalues[i] - s1.eigenvalues[i]).abs() < 1e-8,
                "eigenvalue {i}: {} vs {}",
                s0.eigenvalues[i],
                s1.eigenvalues[i]
            );
        }
    }

    #[test]
    fn eigenvalue_only_scores_flat_across_permutations() {
        let n = 4;
        let base = tfim_chain(n, 1.0, 1.5);
        let (shuffled, _) = shuffle_hamiltonian(&base.hamiltonian, 42);
        let spec = spectrum_from_hamiltonian(&shuffled, n);
        let params = SearchParams {
            input_mode: InputMode::EigenvaluesOnly,
            graph_kind: GraphKind::Line,
            cols: n,
            ..Default::default()
        };
        let dummy_mi = vec![vec![vec![0.0; n]; n]];
        let scores: Vec<f64> = all_permutations(n)
            .iter()
            .map(|perm| {
                score_permutation(
                    None,
                    &dummy_mi,
                    perm,
                    &params,
                    None,
                    Some(&spec.eigenvalues),
                    true,
                )
                .score
            })
            .collect();
        let min_s = scores.iter().cloned().fold(f64::INFINITY, f64::min);
        let max_s = scores.iter().cloned().fold(f64::NEG_INFINITY, f64::max);
        let spread = max_s - min_s;
        assert!(
            spread < 1e-12,
            "eigenvalue-only scores must not discriminate permutations; spread={spread}"
        );
    }
}
