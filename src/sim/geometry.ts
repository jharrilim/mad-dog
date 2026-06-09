/**
 * Emergent geometry from entanglement.
 *
 * Pipeline (following Cao-Carroll-Michalakis, arXiv:1606.08444, as referenced
 * by the Mad-Dog Everettianism essay):
 *   1. reduced density matrices of each factor and each pair of factors
 *   2. mutual information  I(a:b) = S_a + S_b - S_ab
 *   3. distance            d(a,b) = -xi * ln( I(a:b) / I_max )
 *   4. classical MDS to embed the distance matrix into R^k
 *   5. detector: the eigenvalue spectrum of the MDS Gram matrix tells us the
 *      emergent spatial dimension and how cleanly space emerged.
 */

import {
  entropyFromEigenvalues,
  hermitianEigenvalues,
  jacobiEigenSymmetric,
} from './linalg.ts'
import type { QuantumState } from './quantum.ts'

/** One-qubit reduced density matrix (2x2 Hermitian) -> von Neumann entropy. */
export function entropyOneSite(state: QuantumState, q: number): number {
  const { data, dim } = state
  let r00 = 0
  let r11 = 0
  let r01re = 0
  let r01im = 0
  for (let s = 0; s < dim; s++) {
    if ((s >> q) & 1) continue
    const s1 = s | (1 << q)
    const a0r = data[2 * s]
    const a0i = data[2 * s + 1]
    const a1r = data[2 * s1]
    const a1i = data[2 * s1 + 1]
    r00 += a0r * a0r + a0i * a0i
    r11 += a1r * a1r + a1i * a1i
    // rho01 = sum a0 * conj(a1)
    r01re += a0r * a1r + a0i * a1i
    r01im += a0i * a1r - a0r * a1i
  }
  const re = [
    [r00, r01re],
    [r01re, r11],
  ]
  const im = [
    [0, r01im],
    [-r01im, 0],
  ]
  return entropyFromEigenvalues(hermitianEigenvalues(re, im))
}

/** Two-qubit reduced density matrix (4x4 Hermitian) -> von Neumann entropy. */
export function entropyTwoSite(
  state: QuantumState,
  i: number,
  j: number,
): number {
  const { data, dim } = state
  const re: number[][] = Array.from({ length: 4 }, () => new Array<number>(4).fill(0))
  const im: number[][] = Array.from({ length: 4 }, () => new Array<number>(4).fill(0))

  const bi = 1 << i
  const bj = 1 << j
  for (let s = 0; s < dim; s++) {
    if ((s & bi) !== 0 || (s & bj) !== 0) continue
    // Gather the 4 amplitudes for this "rest" configuration.
    const idx = [s, s | bj, s | bi, s | bi | bj] // local index = bit_i*2 + bit_j
    for (let a = 0; a < 4; a++) {
      const ar = data[2 * idx[a]]
      const ai = data[2 * idx[a] + 1]
      for (let b = 0; b < 4; b++) {
        const br = data[2 * idx[b]]
        const bibm = data[2 * idx[b] + 1]
        // rho[a][b] += amp_a * conj(amp_b)
        re[a][b] += ar * br + ai * bibm
        im[a][b] += ai * br - ar * bibm
      }
    }
  }
  return entropyFromEigenvalues(hermitianEigenvalues(re, im))
}

/** Full mutual-information matrix between all single-qubit factors. */
export function mutualInformationMatrix(state: QuantumState): number[][] {
  const n = state.n
  const s1 = Array.from({ length: n }, (_, q) => entropyOneSite(state, q))
  const mi: number[][] = Array.from({ length: n }, () =>
    new Array<number>(n).fill(0),
  )
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      const sij = entropyTwoSite(state, i, j)
      const value = Math.max(0, s1[i] + s1[j] - sij)
      mi[i][j] = value
      mi[j][i] = value
    }
  }
  return mi
}

/**
 * Convert a mutual-information matrix into a distance matrix.
 * Large MI -> short distance. Pairs with negligible MI are clamped to a
 * maximum distance so the embedding stays finite.
 */
export function miToDistance(mi: number[][], xi = 1): number[][] {
  const n = mi.length
  let iMax = 0
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) iMax = Math.max(iMax, mi[i][j])
  }
  if (iMax <= 0) {
    // No correlations at all: everything equidistant.
    return Array.from({ length: n }, (_, i) =>
      Array.from({ length: n }, (_, j) => (i === j ? 0 : 1)),
    )
  }

  const dist: number[][] = Array.from({ length: n }, () =>
    new Array<number>(n).fill(0),
  )
  const floor = iMax * 1e-4
  let maxFinite = 0
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      const value = mi[i][j]
      if (value > floor) {
        const d = -xi * Math.log(value / iMax)
        dist[i][j] = d
        dist[j][i] = d
        maxFinite = Math.max(maxFinite, d)
      } else {
        dist[i][j] = -1 // sentinel, filled below
        dist[j][i] = -1
      }
    }
  }
  const cap = (maxFinite > 0 ? maxFinite : 1) * 1.3
  for (let i = 0; i < n; i++) {
    for (let j = i + 1; j < n; j++) {
      if (dist[i][j] < 0) {
        dist[i][j] = cap
        dist[j][i] = cap
      }
    }
  }
  return dist
}

export interface MdsResult {
  /** Embedding coordinates: coords[i] is the position of factor i (length = dim used). */
  coords: number[][]
  /** Eigenvalues of the Gram matrix, descending (the "scree" spectrum). */
  eigenvalues: number[]
  /** Number of dominant positive eigenvalues = detected emergent dimension. */
  emergentDim: number
  /** Fraction of positive eigenvalue mass captured by the top-k dimensions. */
  explainedVariance: number[]
}

/**
 * Classical (Torgerson) multidimensional scaling.
 * Returns up to `maxDim` coordinates plus the eigenvalue spectrum used to
 * detect the emergent dimension.
 */
export function classicalMds(distance: number[][], maxDim = 3): MdsResult {
  const n = distance.length
  // Squared distances.
  const d2 = distance.map((row) => row.map((d) => d * d))
  // Double centering: B = -1/2 J D2 J.
  const rowMean = d2.map((row) => row.reduce((a, b) => a + b, 0) / n)
  const grand = rowMean.reduce((a, b) => a + b, 0) / n
  const b: number[][] = Array.from({ length: n }, () =>
    new Array<number>(n).fill(0),
  )
  for (let i = 0; i < n; i++) {
    for (let j = 0; j < n; j++) {
      b[i][j] = -0.5 * (d2[i][j] - rowMean[i] - rowMean[j] + grand)
    }
  }

  const { values, vectors } = jacobiEigenSymmetric(b)
  // Descending order.
  const order = values
    .map((val, i) => ({ val, i }))
    .sort((x, y) => y.val - x.val)

  const eigenvalues = order.map((o) => o.val)
  const positive = eigenvalues.filter((v) => v > 1e-9)
  const totalPositive = positive.reduce((a, v) => a + v, 0) || 1

  const explainedVariance: number[] = []
  let running = 0
  for (let k = 0; k < eigenvalues.length; k++) {
    if (eigenvalues[k] > 1e-9) running += eigenvalues[k]
    explainedVariance.push(running / totalPositive)
  }

  // Detect emergent dimension via the largest relative gap in the eigenvalue
  // spectrum, restricted to eigenvalues that carry non-trivial weight (> 1% of
  // the largest). A degenerate pair of leading eigenvalues (e.g. an isotropic
  // 2D plane) is preserved because we cut at the largest *ratio* drop.
  const maxEig = eigenvalues[0] > 0 ? eigenvalues[0] : 1
  const significant = eigenvalues.filter((v) => v > 0.01 * maxEig)
  let emergentDim = significant.length
  let bestRatio = 1
  for (let k = 1; k < significant.length; k++) {
    const ratio = significant[k - 1] / significant[k]
    if (ratio > bestRatio) {
      bestRatio = ratio
      emergentDim = k
    }
  }
  if (significant.length <= 1) emergentDim = significant.length

  const dims = Math.min(maxDim, n)
  const coords: number[][] = Array.from({ length: n }, () =>
    new Array<number>(dims).fill(0),
  )
  for (let k = 0; k < dims; k++) {
    const lambda = order[k].val
    if (lambda <= 0) continue
    const scale = Math.sqrt(lambda)
    const col = order[k].i
    for (let i = 0; i < n; i++) coords[i][k] = vectors[i][col] * scale
  }

  return { coords, eigenvalues, emergentDim, explainedVariance }
}

export interface EmergenceReport {
  mi: number[][]
  distance: number[][]
  mds: MdsResult
}

/** Run the full state -> emergent-geometry pipeline. */
export function analyzeEmergentGeometry(
  state: QuantumState,
  xi = 1,
  maxDim = 3,
): EmergenceReport {
  const mi = mutualInformationMatrix(state)
  const distance = miToDistance(mi, xi)
  const mds = classicalMds(distance, maxDim)
  return { mi, distance, mds }
}
