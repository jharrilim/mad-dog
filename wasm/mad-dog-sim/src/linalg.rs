//! Small dense linear algebra — port of `src/sim/linalg.ts`.

pub struct EigenResult {
    pub values: Vec<f64>,
    pub vectors: Vec<Vec<f64>>,
}

pub fn jacobi_eigen_symmetric(input: &[Vec<f64>]) -> EigenResult {
    let n = input.len();
    let mut a: Vec<Vec<f64>> = input.iter().map(|row| row.clone()).collect();
    let mut v: Vec<Vec<f64>> = (0..n)
        .map(|i| (0..n).map(|j| if i == j { 1.0 } else { 0.0 }).collect())
        .collect();

    for _ in 0..100 {
        let mut off = 0.0;
        for p in 0..n {
            for q in (p + 1)..n {
                off += a[p][q] * a[p][q];
            }
        }
        if off < 1e-28 {
            break;
        }

        for p in 0..n {
            for q in (p + 1)..n {
                let apq = a[p][q];
                if apq.abs() < 1e-300 {
                    continue;
                }
                let app = a[p][p];
                let aqq = a[q][q];
                let phi = 0.5 * (2.0 * apq).atan2(aqq - app);
                let c = phi.cos();
                let s = phi.sin();

                for k in 0..n {
                    let akp = a[k][p];
                    let akq = a[k][q];
                    a[k][p] = c * akp - s * akq;
                    a[k][q] = s * akp + c * akq;
                }
                for k in 0..n {
                    let apk = a[p][k];
                    let aqk = a[q][k];
                    a[p][k] = c * apk - s * aqk;
                    a[q][k] = s * apk + c * aqk;
                }
                for k in 0..n {
                    let vkp = v[k][p];
                    let vkq = v[k][q];
                    v[k][p] = c * vkp - s * vkq;
                    v[k][q] = s * vkp + c * vkq;
                }
            }
        }
    }

    let values: Vec<f64> = (0..n).map(|i| a[i][i]).collect();
    let mut order: Vec<(f64, usize)> = values.iter().copied().enumerate().map(|(i, val)| (val, i)).collect();
    order.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap());

    let sorted_values: Vec<f64> = order.iter().map(|o| o.0).collect();
    let mut sorted_vectors: Vec<Vec<f64>> = vec![vec![0.0; n]; n];
    for (new_idx, o) in order.iter().enumerate() {
        for row in 0..n {
            sorted_vectors[row][new_idx] = v[row][o.1];
        }
    }

    EigenResult {
        values: sorted_values,
        vectors: sorted_vectors,
    }
}

pub fn hermitian_eigenvalues(re: &[Vec<f64>], im: &[Vec<f64>]) -> Vec<f64> {
    let n = re.len();
    let mut m = vec![vec![0.0; 2 * n]; 2 * n];
    for i in 0..n {
        for j in 0..n {
            m[i][j] = re[i][j];
            m[i + n][j + n] = re[i][j];
            m[i][j + n] = -im[i][j];
            m[i + n][j] = im[i][j];
        }
    }
    let result = jacobi_eigen_symmetric(&m);
    let mut eigs = Vec::with_capacity(n);
    for i in (0..2 * n).step_by(2) {
        eigs.push(result.values[i]);
    }
    eigs
}

pub fn entropy_from_eigenvalues(eigs: &[f64]) -> f64 {
    let mut s = 0.0;
    for &lambda in eigs {
        if lambda > 1e-12 {
            s -= lambda * lambda.ln();
        }
    }
    s
}

/// 2D Procrustes alignment — port of `procrustes2D` in `src/sim/linalg.ts`.
pub fn procrustes_2d(x: &[Vec<f64>], y: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = x.len();
    if n == 0 {
        return x.to_vec();
    }
    let mut cx = [0.0; 2];
    let mut cy = [0.0; 2];
    for i in 0..n {
        cx[0] += x[i][0];
        cx[1] += x[i][1];
        cy[0] += y[i][0];
        cy[1] += y[i][1];
    }
    cx[0] /= n as f64;
    cx[1] /= n as f64;
    cy[0] /= n as f64;
    cy[1] /= n as f64;

    let xc: Vec<[f64; 2]> = x
        .iter()
        .map(|p| [p[0] - cx[0], p[1] - cx[1]])
        .collect();
    let yc: Vec<[f64; 2]> = y
        .iter()
        .map(|p| [p[0] - cy[0], p[1] - cy[1]])
        .collect();

    let mut a = 0.0;
    let mut b = 0.0;
    let mut c = 0.0;
    let mut d = 0.0;
    let mut norm_x = 0.0;
    for i in 0..n {
        a += xc[i][0] * yc[i][0];
        b += xc[i][0] * yc[i][1];
        c += xc[i][1] * yc[i][0];
        d += xc[i][1] * yc[i][1];
        norm_x += xc[i][0] * xc[i][0] + xc[i][1] * xc[i][1];
    }

    let e = (a + d) / 2.0;
    let f = (a - d) / 2.0;
    let g = (c + b) / 2.0;
    let h = (c - b) / 2.0;
    let q = (e * e + h * h).sqrt();
    let r = (f * f + g * g).sqrt();
    let sx = q + r;
    let sy = q - r;
    let a1 = g.atan2(f);
    let a2 = h.atan2(e);
    let theta = (a2 - a1) / 2.0;
    let phi = (a2 + a1) / 2.0;

    let cu = theta.cos();
    let su = theta.sin();
    let cv = phi.cos();
    let sv = phi.sin();
    let rmat = [
        [cu * cv + su * sv, cu * -sv + su * cv],
        [su * cv - cu * sv, su * -sv - cu * cv],
    ];
    let scale = if norm_x > 1e-12 { (sx + sy) / norm_x } else { 1.0 };

    xc.iter()
        .map(|p| {
            let rx = p[0] * rmat[0][0] + p[1] * rmat[1][0];
            let ry = p[0] * rmat[0][1] + p[1] * rmat[1][1];
            vec![scale * rx + cy[0], scale * ry + cy[1]]
        })
        .collect()
}
