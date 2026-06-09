/**
 * Minimal quantum state-vector engine.
 *
 * A pure state on `n` qubits lives in a 2^n-dimensional Hilbert space. We store
 * amplitudes as a flat Float64Array of length 2 * dim, interleaved as
 * [re0, im0, re1, im1, ...]. This is the correctness-first reference backend;
 * a WebGPU backend can later mirror this same interface.
 */

export type PauliLetter = 'X' | 'Y' | 'Z'

/** A single Pauli operator acting on one qubit. */
export interface PauliOp {
  qubit: number
  letter: PauliLetter
}

/**
 * A term in a Pauli-sum Hamiltonian: coeff * (tensor product of `ops`).
 * Qubits not listed in `ops` are acted on by the identity. At most one op per
 * qubit per term.
 */
export interface PauliTerm {
  coeff: number
  ops: PauliOp[]
}

export interface QuantumState {
  n: number
  dim: number
  /** Interleaved [re, im] amplitudes, length 2*dim. */
  data: Float64Array
}

export function makeZeroState(n: number): QuantumState {
  const dim = 1 << n
  return { n, dim, data: new Float64Array(2 * dim) }
}

export function cloneState(state: QuantumState): QuantumState {
  return { n: state.n, dim: state.dim, data: state.data.slice() }
}

/** Expectation value of the Pauli-Z operator on a single qubit. */
export function expectationZ(state: QuantumState, q: number): number {
  const { data, dim } = state
  let sum = 0
  for (let s = 0; s < dim; s++) {
    const sign = (s >> q) & 1 ? -1 : 1
    const re = data[2 * s]
    const im = data[2 * s + 1]
    sum += sign * (re * re + im * im)
  }
  return sum
}

/** A normalized random complex state (deterministic given the supplied RNG). */
export function makeRandomState(n: number, rng: () => number): QuantumState {
  const state = makeZeroState(n)
  const { data, dim } = state
  for (let i = 0; i < dim; i++) {
    // Box-Muller for Gaussian components -> Haar-ish random direction.
    const u1 = Math.max(rng(), 1e-12)
    const u2 = rng()
    const mag = Math.sqrt(-2 * Math.log(u1))
    data[2 * i] = mag * Math.cos(2 * Math.PI * u2)
    data[2 * i + 1] = mag * Math.sin(2 * Math.PI * u2)
  }
  normalize(state)
  return state
}

export function norm(state: QuantumState): number {
  const { data, dim } = state
  let sum = 0
  for (let i = 0; i < dim; i++) {
    const re = data[2 * i]
    const im = data[2 * i + 1]
    sum += re * re + im * im
  }
  return Math.sqrt(sum)
}

export function normalize(state: QuantumState): void {
  const nrm = norm(state)
  if (nrm < 1e-300) return
  const inv = 1 / nrm
  const { data, dim } = state
  for (let i = 0; i < 2 * dim; i++) data[i] *= inv
}

/**
 * Apply a single Pauli term to `inState`, accumulating into `out`.
 * `out` is NOT cleared; callers accumulate multiple terms into it.
 */
function applyPauliTerm(
  term: PauliTerm,
  inState: QuantumState,
  out: Float64Array,
): void {
  const { data, dim } = inState
  const { coeff, ops } = term

  for (let s = 0; s < dim; s++) {
    let target = s
    // Phase factor (re, im) accumulated across single-qubit ops.
    let pr = 1
    let pi = 0
    for (const op of ops) {
      const bit = (s >> op.qubit) & 1
      switch (op.letter) {
        case 'X':
          target ^= 1 << op.qubit
          break
        case 'Y': {
          target ^= 1 << op.qubit
          // Y|0> = i|1>, Y|1> = -i|0>  => factor +i (bit 0) or -i (bit 1).
          const fr = 0
          const fi = bit === 0 ? 1 : -1
          const npr = pr * fr - pi * fi
          const npi = pr * fi + pi * fr
          pr = npr
          pi = npi
          break
        }
        case 'Z':
          if (bit === 1) {
            pr = -pr
            pi = -pi
          }
          break
      }
    }
    const ar = data[2 * s]
    const ai = data[2 * s + 1]
    // (coeff * phase) * amplitude
    const cr = coeff * pr
    const ci = coeff * pi
    out[2 * target] += cr * ar - ci * ai
    out[2 * target + 1] += cr * ai + ci * ar
  }
}

/** Hamiltonian as a list of Pauli terms, with cached operator-norm estimate. */
export class Hamiltonian {
  readonly n: number
  readonly terms: PauliTerm[]

  constructor(n: number, terms: PauliTerm[]) {
    this.n = n
    this.terms = terms
  }

  /** result = H |state>. Allocates a fresh state. */
  apply(state: QuantumState): QuantumState {
    const result = makeZeroState(this.n)
    for (const term of this.terms) {
      applyPauliTerm(term, state, result.data)
    }
    return result
  }

  /** Expectation value <state|H|state> (real for Hermitian H). */
  expectation(state: QuantumState): number {
    const hPsi = this.apply(state)
    const { data, dim } = state
    let re = 0
    for (let i = 0; i < dim; i++) {
      // Re(<state| hPsi>) = sum conj(state) * hPsi.
      const sr = data[2 * i]
      const si = data[2 * i + 1]
      const hr = hPsi.data[2 * i]
      const hi = hPsi.data[2 * i + 1]
      re += sr * hr + si * hi
    }
    return re
  }

  /** Estimate the largest |eigenvalue| via power iteration on |H|. */
  estimateSpectralRadius(rng: () => number, iters = 30): number {
    let v = makeRandomState(this.n, rng)
    let lambda = 0
    for (let k = 0; k < iters; k++) {
      const hv = this.apply(v)
      lambda = norm(hv)
      if (lambda < 1e-300) break
      normalize(hv)
      v = hv
    }
    return lambda
  }
}

/**
 * Find a low-energy (approximate ground) state via imaginary-time evolution:
 * repeatedly apply (1 - dt*H) and renormalize. The step `dt` is chosen from the
 * estimated spectral radius so every multiplier (1 - dt*E) stays positive, and
 * the most negative energy (the ground state) is amplified the most.
 */
export function groundState(
  h: Hamiltonian,
  rng: () => number,
  options: { maxIters?: number; tol?: number } = {},
): { state: QuantumState; energy: number; iters: number } {
  const { maxIters = 4000, tol = 1e-9 } = options
  const radius = Math.max(h.estimateSpectralRadius(rng), 1e-6)
  const dt = 0.5 / radius

  const state = makeRandomState(h.n, rng)
  let prevEnergy = Infinity
  let energy = h.expectation(state)
  let iters = 0

  for (; iters < maxIters; iters++) {
    const hPsi = h.apply(state)
    const { data, dim } = state
    for (let i = 0; i < dim; i++) {
      data[2 * i] -= dt * hPsi.data[2 * i]
      data[2 * i + 1] -= dt * hPsi.data[2 * i + 1]
    }
    normalize(state)
    energy = h.expectation(state)
    if (Math.abs(prevEnergy - energy) < tol) {
      iters++
      break
    }
    prevEnergy = energy
  }

  return { state, energy, iters }
}

/** Deterministic mulberry32 PRNG for reproducible runs. */
export function makeRng(seed: number): () => number {
  let a = seed >>> 0
  return function () {
    a |= 0
    a = (a + 0x6d2b79f5) | 0
    let t = Math.imul(a ^ (a >>> 15), 1 | a)
    t = (t + Math.imul(t ^ (t >>> 7), 61 | t)) ^ t
    return ((t ^ (t >>> 14)) >>> 0) / 4294967296
  }
}
