/**
 * Small dense linear algebra for the simulator.
 *
 * These routines operate on tiny matrices (reduced density matrices are 2x2 or
 * 4x4; the MDS Gram matrix is N x N for N factors, typically <= ~24), so a
 * simple cyclic Jacobi eigensolver is more than adequate and avoids extra deps.
 */

export interface EigenResult {
  /** Eigenvalues, ascending. */
  values: number[]
  /** Eigenvectors as columns: vectors[i][k] is component i of eigenvector k. */
  vectors: number[][]
}

/**
 * Cyclic Jacobi eigensolver for a real symmetric matrix.
 * Returns eigenvalues in ascending order with matching eigenvectors.
 */
export function jacobiEigenSymmetric(input: number[][]): EigenResult {
  const n = input.length
  // Work on a copy.
  const a = input.map((row) => row.slice())
  // Eigenvector accumulator, initialized to identity.
  const v: number[][] = Array.from({ length: n }, (_, i) =>
    Array.from({ length: n }, (_, j) => (i === j ? 1 : 0)),
  )

  const maxSweeps = 100
  for (let sweep = 0; sweep < maxSweeps; sweep++) {
    // Sum of squares of off-diagonal elements.
    let off = 0
    for (let p = 0; p < n; p++) {
      for (let q = p + 1; q < n; q++) off += a[p][q] * a[p][q]
    }
    if (off < 1e-28) break

    for (let p = 0; p < n; p++) {
      for (let q = p + 1; q < n; q++) {
        const apq = a[p][q]
        if (Math.abs(apq) < 1e-300) continue
        const app = a[p][p]
        const aqq = a[q][q]
        const phi = 0.5 * Math.atan2(2 * apq, aqq - app)
        const c = Math.cos(phi)
        const s = Math.sin(phi)

        // Apply rotation J^T A J.
        for (let k = 0; k < n; k++) {
          const akp = a[k][p]
          const akq = a[k][q]
          a[k][p] = c * akp - s * akq
          a[k][q] = s * akp + c * akq
        }
        for (let k = 0; k < n; k++) {
          const apk = a[p][k]
          const aqk = a[q][k]
          a[p][k] = c * apk - s * aqk
          a[q][k] = s * apk + c * aqk
        }
        // Accumulate eigenvectors.
        for (let k = 0; k < n; k++) {
          const vkp = v[k][p]
          const vkq = v[k][q]
          v[k][p] = c * vkp - s * vkq
          v[k][q] = s * vkp + c * vkq
        }
      }
    }
  }

  const values = a.map((row, i) => row[i])
  const order = values
    .map((val, i) => ({ val, i }))
    .sort((x, y) => x.val - y.val)

  const sortedValues = order.map((o) => o.val)
  const sortedVectors: number[][] = Array.from({ length: n }, () =>
    new Array<number>(n).fill(0),
  )
  order.forEach((o, newIdx) => {
    for (let row = 0; row < n; row++) sortedVectors[row][newIdx] = v[row][o.i]
  })

  return { values: sortedValues, vectors: sortedVectors }
}

/**
 * Eigenvalues of a Hermitian matrix H = re + i*im.
 *
 * Uses the standard real embedding M = [[re, -im], [im, re]], a (2n x 2n) real
 * symmetric matrix whose eigenvalues are exactly those of H, each appearing
 * twice. We return the n distinct eigenvalues (ascending).
 */
export function hermitianEigenvalues(re: number[][], im: number[][]): number[] {
  const n = re.length
  const m: number[][] = Array.from({ length: 2 * n }, () =>
    new Array<number>(2 * n).fill(0),
  )
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      m[i][j] = re[i][j]
      m[i + n][j + n] = re[i][j]
      m[i][j + n] = -im[i][j]
      m[i + n][j] = im[i][j]
    }
  }
  const { values } = jacobiEigenSymmetric(m)
  // Eigenvalues come in degenerate pairs; take every other (ascending).
  const result: number[] = []
  for (let i = 0; i < 2 * n; i += 2) result.push(values[i])
  return result
}

/**
 * 2D Procrustes alignment: rotate/reflect/scale/translate `x` (N x 2) to best
 * match `y` (N x 2). Used to remove the rotation/reflection gauge freedom of
 * MDS so an evolving 2D embedding does not spin between frames.
 */
export function procrustes2D(x: number[][], y: number[][]): number[][] {
  const n = x.length
  if (n === 0) return x
  const cx = [0, 0]
  const cy = [0, 0]
  for (let i = 0; i < n; i++) {
    cx[0] += x[i][0]
    cx[1] += x[i][1]
    cy[0] += y[i][0]
    cy[1] += y[i][1]
  }
  cx[0] /= n
  cx[1] /= n
  cy[0] /= n
  cy[1] /= n

  const xc = x.map((p) => [p[0] - cx[0], p[1] - cx[1]])
  const yc = y.map((p) => [p[0] - cy[0], p[1] - cy[1]])

  // M = Xc^T Yc (2x2)
  let a = 0
  let b = 0
  let c = 0
  let d = 0
  let normX = 0
  for (let i = 0; i < n; i++) {
    a += xc[i][0] * yc[i][0]
    b += xc[i][0] * yc[i][1]
    c += xc[i][1] * yc[i][0]
    d += xc[i][1] * yc[i][1]
    normX += xc[i][0] * xc[i][0] + xc[i][1] * xc[i][1]
  }

  // Analytic 2x2 SVD of M = U S V^T.
  const e = (a + d) / 2
  const f = (a - d) / 2
  const g = (c + b) / 2
  const h = (c - b) / 2
  const q = Math.hypot(e, h)
  const r = Math.hypot(f, g)
  const sx = q + r
  const sy = q - r
  const a1 = Math.atan2(g, f)
  const a2 = Math.atan2(h, e)
  const theta = (a2 - a1) / 2
  const phi = (a2 + a1) / 2

  // R = U V^T (rotation, possibly with reflection).
  const cu = Math.cos(theta)
  const su = Math.sin(theta)
  const cv = Math.cos(phi)
  const sv = Math.sin(phi)
  const u = [
    [cu, -su],
    [su, cu],
  ]
  const v = [
    [cv, -sv],
    [sv, cv],
  ]
  // R = U * V^T
  const rmat = [
    [u[0][0] * v[0][0] + u[0][1] * v[0][1], u[0][0] * v[1][0] + u[0][1] * v[1][1]],
    [u[1][0] * v[0][0] + u[1][1] * v[0][1], u[1][0] * v[1][0] + u[1][1] * v[1][1]],
  ]
  const scale = normX > 1e-12 ? (sx + sy) / normX : 1

  return xc.map((p) => {
    const rx = p[0] * rmat[0][0] + p[1] * rmat[1][0]
    const ry = p[0] * rmat[0][1] + p[1] * rmat[1][1]
    return [scale * rx + cy[0], scale * ry + cy[1]]
  })
}

/** Shannon/von Neumann entropy (in nats) from a list of probabilities/eigenvalues. */
export function entropyFromEigenvalues(eigs: number[]): number {
  let s = 0
  for (const lambda of eigs) {
    if (lambda > 1e-12) s -= lambda * Math.log(lambda)
  }
  return s
}
