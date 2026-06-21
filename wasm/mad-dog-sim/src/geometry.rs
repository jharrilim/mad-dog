//! Emergent geometry — port of `src/sim/geometry.ts`.

use crate::linalg::{entropy_from_eigenvalues, hermitian_eigenvalues, jacobi_eigen_symmetric};
use crate::quantum::QuantumState;
use serde::{Deserialize, Serialize};

pub fn entropy_one_site(state: &QuantumState, q: usize) -> f64 {
    let dim = state.dim;
    let data = &state.data;
    let mut r00 = 0.0;
    let mut r11 = 0.0;
    let mut r01re = 0.0;
    let mut r01im = 0.0;

    for s in 0..dim {
        if (s >> q) & 1 != 0 {
            continue;
        }
        let s1 = s | (1 << q);
        let a0r = data[2 * s];
        let a0i = data[2 * s + 1];
        let a1r = data[2 * s1];
        let a1i = data[2 * s1 + 1];
        r00 += a0r * a0r + a0i * a0i;
        r11 += a1r * a1r + a1i * a1i;
        r01re += a0r * a1r + a0i * a1i;
        r01im += a0i * a1r - a0r * a1i;
    }

    let re = vec![vec![r00, r01re], vec![r01re, r11]];
    let im = vec![vec![0.0, r01im], vec![-r01im, 0.0]];
    entropy_from_eigenvalues(&hermitian_eigenvalues(&re, &im))
}

pub fn entropy_two_site(state: &QuantumState, i: usize, j: usize) -> f64 {
    let dim = state.dim;
    let data = &state.data;
    let mut re = vec![vec![0.0; 4]; 4];
    let mut im = vec![vec![0.0; 4]; 4];
    let bi = 1 << i;
    let bj = 1 << j;

    for s in 0..dim {
        if (s & bi) != 0 || (s & bj) != 0 {
            continue;
        }
        let idx = [s, s | bj, s | bi, s | bi | bj];
        for a in 0..4 {
            let ar = data[2 * idx[a]];
            let ai = data[2 * idx[a] + 1];
            for b in 0..4 {
                let br = data[2 * idx[b]];
                let bim = data[2 * idx[b] + 1];
                re[a][b] += ar * br + ai * bim;
                im[a][b] += ai * br - ar * bim;
            }
        }
    }
    entropy_from_eigenvalues(&hermitian_eigenvalues(&re, &im))
}

pub fn region_eigenvalues(state: &QuantumState, region: &[usize]) -> Vec<f64> {
    let n = state.n;
    let dim = state.dim;
    let data = &state.data;

    let mut in_region = vec![false; n];
    for &q in region {
        in_region[q] = true;
    }
    let mut a = Vec::new();
    let mut env = Vec::new();
    for q in 0..n {
        if in_region[q] {
            a.push(q);
        } else {
            env.push(q);
        }
    }
    let (keep, trace) = if a.len() <= env.len() { (a, env) } else { (env, a) };

    let k = keep.len();
    if k == 0 {
        return vec![1.0];
    }
    let dim_a = 1usize << k;
    let dim_e = 1usize << trace.len();

    let mut re = vec![0.0; dim_e * dim_a];
    let mut im = vec![0.0; dim_e * dim_a];
    for s in 0..dim {
        let mut ia = 0usize;
        for (j, &q) in keep.iter().enumerate() {
            ia |= ((s >> q) & 1) << j;
        }
        let mut ie = 0usize;
        for (j, &q) in trace.iter().enumerate() {
            ie |= ((s >> q) & 1) << j;
        }
        re[ie * dim_a + ia] = data[2 * s];
        im[ie * dim_a + ia] = data[2 * s + 1];
    }

    let mut rho_re = vec![vec![0.0; dim_a]; dim_a];
    let mut rho_im = vec![vec![0.0; dim_a]; dim_a];
    for e in 0..dim_e {
        let base = e * dim_a;
        for i in 0..dim_a {
            let ar = re[base + i];
            let ai = im[base + i];
            if ar == 0.0 && ai == 0.0 {
                continue;
            }
            for j in 0..dim_a {
                let br = re[base + j];
                let bi = im[base + j];
                rho_re[i][j] += ar * br + ai * bi;
                rho_im[i][j] += ai * br - ar * bi;
            }
        }
    }
    hermitian_eigenvalues(&rho_re, &rho_im)
}

pub fn entropy_of_region(state: &QuantumState, region: &[usize]) -> f64 {
    entropy_from_eigenvalues(&region_eigenvalues(state, region))
}

/** Full mutual-information matrix between all single-qubit factors. */
pub fn mutual_information_matrix(state: &QuantumState) -> Vec<Vec<f64>> {
    let n = state.n;
    let s1: Vec<f64> = (0..n).map(|q| entropy_one_site(state, q)).collect();
    let mut mi = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in (i + 1)..n {
            let sij = entropy_two_site(state, i, j);
            let value = (s1[i] + s1[j] - sij).max(0.0);
            mi[i][j] = value;
            mi[j][i] = value;
        }
    }
    mi
}

pub fn mi_to_distance(mi: &[Vec<f64>], xi: f64) -> Vec<Vec<f64>> {
    let n = mi.len();
    let mut i_max = 0.0_f64;
    for i in 0..n {
        for j in (i + 1)..n {
            i_max = i_max.max(mi[i][j]);
        }
    }
    if i_max <= 0.0 {
        return (0..n)
            .map(|i| (0..n).map(|j| if i == j { 0.0 } else { 1.0 }).collect())
            .collect();
    }

    let mut dist = vec![vec![0.0; n]; n];
    let floor = i_max * 1e-4;
    let mut max_finite = 0.0_f64;
    for i in 0..n {
        for j in (i + 1)..n {
            let value = mi[i][j];
            if value > floor {
                let d = -xi * (value / i_max).ln();
                dist[i][j] = d;
                dist[j][i] = d;
                max_finite = max_finite.max(d);
            } else {
                dist[i][j] = -1.0;
                dist[j][i] = -1.0;
            }
        }
    }
    let cap = if max_finite > 0.0 { max_finite } else { 1.0 } * 1.3;
    for i in 0..n {
        for j in (i + 1)..n {
            if dist[i][j] < 0.0 {
                dist[i][j] = cap;
                dist[j][i] = cap;
            }
        }
    }
    dist
}

pub fn mi_distances_from_state(state: &QuantumState, center: usize, xi: f64) -> Vec<f64> {
    let dist = mi_to_distance(&mutual_information_matrix(state), xi);
    dist[center].clone()
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MdsResult {
    pub coords: Vec<Vec<f64>>,
    pub eigenvalues: Vec<f64>,
    pub emergent_dim: usize,
    pub explained_variance: Vec<f64>,
}

pub fn classical_mds(distance: &[Vec<f64>], max_dim: usize) -> MdsResult {
    let n = distance.len();
    let d2: Vec<Vec<f64>> = distance
        .iter()
        .map(|row| row.iter().map(|d| d * d).collect())
        .collect();
    let row_mean: Vec<f64> = d2
        .iter()
        .map(|row| row.iter().sum::<f64>() / n as f64)
        .collect();
    let grand: f64 = row_mean.iter().sum::<f64>() / n as f64;
    let mut b = vec![vec![0.0; n]; n];
    for i in 0..n {
        for j in 0..n {
            b[i][j] = -0.5 * (d2[i][j] - row_mean[i] - row_mean[j] + grand);
        }
    }

    let EigenWrap { values, vectors } = {
        let r = jacobi_eigen_symmetric(&b);
        EigenWrap {
            values: r.values,
            vectors: r.vectors,
        }
    };

    let mut order: Vec<(f64, usize)> = values.iter().copied().enumerate().map(|(i, val)| (val, i)).collect();
    order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let eigenvalues: Vec<f64> = order.iter().map(|o| o.0).collect();
    let positive: Vec<f64> = eigenvalues.iter().copied().filter(|v| *v > 1e-9).collect();
    let total_positive: f64 = positive.iter().sum::<f64>().max(1.0);

    let mut explained_variance = Vec::new();
    let mut running = 0.0;
    for &ev in &eigenvalues {
        if ev > 1e-9 {
            running += ev;
        }
        explained_variance.push(running / total_positive);
    }

    let max_eig = if eigenvalues[0] > 0.0 {
        eigenvalues[0]
    } else {
        1.0
    };
    let significant: Vec<f64> = eigenvalues
        .iter()
        .copied()
        .filter(|v| *v > 0.01 * max_eig)
        .collect();
    let mut emergent_dim = significant.len();
    let mut best_ratio = 1.0;
    for k in 1..significant.len() {
        let ratio = significant[k - 1] / significant[k];
        if ratio > best_ratio {
            best_ratio = ratio;
            emergent_dim = k;
        }
    }
    if significant.len() <= 1 {
        emergent_dim = significant.len();
    }

    let dims = max_dim.min(n);
    let mut coords = vec![vec![0.0; dims]; n];
    for k in 0..dims {
        let lambda = order[k].0;
        if lambda <= 0.0 {
            continue;
        }
        let scale = lambda.sqrt();
        let col = order[k].1;
        for i in 0..n {
            coords[i][k] = vectors[i][col] * scale;
        }
    }

    MdsResult {
        coords,
        eigenvalues,
        emergent_dim,
        explained_variance,
    }
}

struct EigenWrap {
    values: Vec<f64>,
    vectors: Vec<Vec<f64>>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct EmergenceReport {
    pub mi: Vec<Vec<f64>>,
    pub distance: Vec<Vec<f64>>,
    pub mds: MdsResult,
}

pub fn analyze_emergent_geometry(state: &QuantumState, xi: f64, max_dim: usize) -> EmergenceReport {
    let mi = mutual_information_matrix(state);
    let distance = mi_to_distance(&mi, xi);
    let mds = classical_mds(&distance, max_dim);
    EmergenceReport {
        mi,
        distance,
        mds,
    }
}
