//! Small dense linear algebra — port of `src/sim/linalg.ts`.

pub struct EigenResult {
    pub values: Vec<f64>,
    pub vectors: Vec<Vec<f64>>,
}

pub fn jacobi_eigen_symmetric(input: &[Vec<f64>]) -> EigenResult {
    let n = input.len();
    let mut a: Vec<Vec<f64>> = input.to_vec();
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

/// Full Hermitian eigendecomposition; eigenvectors as interleaved amplitude data.
pub fn hermitian_eigen_decomposition(
    re: &[Vec<f64>],
    im: &[Vec<f64>],
) -> (Vec<f64>, Vec<Vec<f64>>) {
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
    let mut states = Vec::with_capacity(n);
    for k in 0..n {
        eigs.push(result.values[2 * k]);
        let mut data = vec![0.0; 2 * n];
        for i in 0..n {
            data[2 * i] = result.vectors[i][2 * k];
            data[2 * i + 1] = result.vectors[i + n][2 * k];
        }
        let mut norm = 0.0;
        for i in 0..n {
            norm += data[2 * i] * data[2 * i] + data[2 * i + 1] * data[2 * i + 1];
        }
        norm = norm.sqrt().max(1e-300);
        for v in &mut data {
            *v /= norm;
        }
        states.push(data);
    }
    (eigs, states)
}

pub fn hermitian_eigenvalues(re: &[Vec<f64>], im: &[Vec<f64>]) -> Vec<f64> {
    hermitian_eigen_decomposition(re, im).0
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

fn mat_mul_3(a: &[[f64; 3]; 3], b: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    let mut c = [[0.0; 3]; 3];
    for i in 0..3 {
        for j in 0..3 {
            for k in 0..3 {
                c[i][j] += a[i][k] * b[k][j];
            }
        }
    }
    c
}

fn transpose_3(m: &[[f64; 3]; 3]) -> [[f64; 3]; 3] {
    [
        [m[0][0], m[1][0], m[2][0]],
        [m[0][1], m[1][1], m[2][1]],
        [m[0][2], m[1][2], m[2][2]],
    ]
}

fn det_3(m: &[[f64; 3]; 3]) -> f64 {
    m[0][0] * (m[1][1] * m[2][2] - m[1][2] * m[2][1])
        - m[0][1] * (m[1][0] * m[2][2] - m[1][2] * m[2][0])
        + m[0][2] * (m[1][0] * m[2][1] - m[1][1] * m[2][0])
}

/// 3D Procrustes (Kabsch) alignment — port of `procrustes3D` in `src/sim/linalg.ts`.
pub fn procrustes_3d(x: &[Vec<f64>], y: &[Vec<f64>]) -> Vec<Vec<f64>> {
    let n = x.len();
    if n == 0 {
        return x.to_vec();
    }
    let mut cx = [0.0; 3];
    let mut cy = [0.0; 3];
    for i in 0..n {
        for d in 0..3 {
            cx[d] += x[i].get(d).copied().unwrap_or(0.0);
            cy[d] += y[i].get(d).copied().unwrap_or(0.0);
        }
    }
    for d in 0..3 {
        cx[d] /= n as f64;
        cy[d] /= n as f64;
    }

    let xc: Vec<[f64; 3]> = x
        .iter()
        .map(|p| {
            [
                p.first().copied().unwrap_or(0.0) - cx[0],
                p.get(1).copied().unwrap_or(0.0) - cx[1],
                p.get(2).copied().unwrap_or(0.0) - cx[2],
            ]
        })
        .collect();
    let yc: Vec<[f64; 3]> = y
        .iter()
        .map(|p| {
            [
                p.first().copied().unwrap_or(0.0) - cy[0],
                p.get(1).copied().unwrap_or(0.0) - cy[1],
                p.get(2).copied().unwrap_or(0.0) - cy[2],
            ]
        })
        .collect();

    let mut h = [[0.0; 3]; 3];
    let mut norm_x = 0.0;
    for i in 0..n {
        norm_x += xc[i][0] * xc[i][0] + xc[i][1] * xc[i][1] + xc[i][2] * xc[i][2];
        for j in 0..3 {
            for k in 0..3 {
                h[j][k] += xc[i][j] * yc[i][k];
            }
        }
    }

    let ht = transpose_3(&h);
    let ht_h = mat_mul_3(&ht, &h);
    let ht_h_vec: Vec<Vec<f64>> = ht_h.iter().map(|row| row.to_vec()).collect();
    let eig = jacobi_eigen_symmetric(&ht_h_vec);
    let mut order: Vec<(f64, usize)> = eig
        .values
        .iter()
        .copied()
        .enumerate()
        .map(|(i, val)| (val, i))
        .collect();
    order.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap());

    let sing: Vec<f64> = order.iter().map(|o| o.0.max(0.0).sqrt()).collect();
    let total_sing: f64 = sing.iter().sum::<f64>().max(1.0);
    let scale = if norm_x > 1e-12 {
        total_sing / norm_x
    } else {
        1.0
    };

    let mut v = [[0.0; 3]; 3];
    for k in 0..3 {
        let col = order[k].1;
        for i in 0..3 {
            v[i][k] = eig.vectors[i][col];
        }
    }

    let mut u = mat_mul_3(&h, &v);
    for k in 0..3 {
        let s = if sing[k] > 1e-12 { sing[k] } else { 1.0 };
        for i in 0..3 {
            u[i][k] /= s;
        }
    }

    let mut rmat = mat_mul_3(&u, &transpose_3(&v));
    if det_3(&rmat) < 0.0 {
        for i in 0..3 {
            u[i][2] *= -1.0;
        }
        rmat = mat_mul_3(&u, &transpose_3(&v));
    }

    xc.iter()
        .map(|p| {
            let rx = p[0] * rmat[0][0] + p[1] * rmat[0][1] + p[2] * rmat[0][2];
            let ry = p[0] * rmat[1][0] + p[1] * rmat[1][1] + p[2] * rmat[1][2];
            let rz = p[0] * rmat[2][0] + p[1] * rmat[2][1] + p[2] * rmat[2][2];
            vec![
                scale * rx + cy[0],
                scale * ry + cy[1],
                scale * rz + cy[2],
            ]
        })
        .collect()
}
