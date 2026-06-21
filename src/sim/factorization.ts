/**
 * Spectrum-driven factorization search — types and TS reference fallback.
 */

import { groundState, Hamiltonian, makeRng, type QuantumState } from './quantum.ts'
import { mutualInformationMatrix } from './geometry.ts'
import { randomNonlocal, tfimChain } from './models.ts'

export interface FactorizationCandidate {
  /** perm[original_qubit] = line_position */
  permutation: number[]
  localityFraction: number
  miNnRatio: number
  score: number
  nonlocalTerms: number
}

export type FactorizationKind = 'shuffled_chain' | 'random'

export interface FactorizationSearchConfig {
  kind: FactorizationKind
  n: number
  field: number
  seed: number
  topK?: number
}

export interface FactorizationSearchResult {
  label: string
  n: number
  energy: number
  baseline: FactorizationCandidate
  best: FactorizationCandidate
  topCandidates: FactorizationCandidate[]
  recoveredIdentity: boolean
  trueShuffle?: number[]
  elapsedMs: number
}

function remapHamiltonian(h: Hamiltonian, perm: number[]): Hamiltonian {
  return new Hamiltonian(
    h.n,
    h.terms.map((term) => ({
      coeff: term.coeff,
      ops: term.ops.map((op) => ({ ...op, qubit: perm[op.qubit] })),
    })),
  )
}

function twoBodyQubits(term: { ops: { qubit: number }[] }): [number, number] | null {
  if (term.ops.length !== 2) return null
  const a = term.ops[0].qubit
  const b = term.ops[1].qubit
  return a < b ? [a, b] : [b, a]
}

function scorePermutation(
  h: Hamiltonian,
  perm: number[],
  mi: number[][],
): FactorizationCandidate {
  const n = h.n
  const inv = Array.from({ length: n }, (_, pos) => {
    for (let q = 0; q < n; q++) if (perm[q] === pos) return q
    return 0
  })

  let local = 0
  let nonlocal = 0
  for (const term of h.terms) {
    if (twoBodyQubits(term)) {
      const i = term.ops[0].qubit
      const j = term.ops[1].qubit
      const dist = Math.abs(perm[i] - perm[j])
      if (dist === 1) local++
      else nonlocal++
    }
  }
  const total = local + nonlocal
  const localityFraction = total > 0 ? local / total : 1

  let nnSum = 0
  let nnCount = 0
  let farSum = 0
  let farCount = 0
  for (let k = 0; k < n - 1; k++) {
    nnSum += mi[inv[k]][inv[k + 1]]
    nnCount++
  }
  for (let k = 0; k < n; k++) {
    for (let d = 2; k + d < n; d++) {
      farSum += mi[inv[k]][inv[k + d]]
      farCount++
    }
  }
  const nnAvg = nnCount > 0 ? nnSum / nnCount : 0
  const farAvg = farCount > 0 ? farSum / farCount : 1e-9
  const miNnRatio = nnAvg / Math.max(farAvg, 1e-12)
  const score = 0.6 * localityFraction + 0.4 * (miNnRatio / (miNnRatio + 1))

  return {
    permutation: perm.slice(),
    localityFraction,
    miNnRatio,
    score,
    nonlocalTerms: nonlocal,
  }
}

function identityPerm(n: number): number[] {
  return Array.from({ length: n }, (_, i) => i)
}

function randomPermutation(n: number, rng: () => number): number[] {
  const perm = identityPerm(n)
  for (let i = n - 1; i > 0; i--) {
    const j = Math.floor(rng() * (i + 1))
    ;[perm[i], perm[j]] = [perm[j], perm[i]]
  }
  return perm
}

function heapPermute(n: number, a: number[], k: number, out: number[][]): void {
  if (k === 1) {
    out.push(a.slice())
    return
  }
  heapPermute(n, a, k - 1, out)
  for (let i = 0; i < k - 1; i++) {
    if (k % 2 === 0) {
      ;[a[i], a[k - 1]] = [a[k - 1], a[i]]
    } else {
      ;[a[0], a[k - 1]] = [a[k - 1], a[0]]
    }
    heapPermute(n, a, k - 1, out)
  }
}

function allPermutations(n: number): number[][] {
  const a = identityPerm(n)
  const out: number[][] = []
  heapPermute(n, a, n, out)
  return out
}

function greedySearch(
  h: Hamiltonian,
  mi: number[][],
  maxIters: number,
): FactorizationCandidate {
  const n = h.n
  const perm = identityPerm(n)
  let best = scorePermutation(h, perm, mi)
  for (let iter = 0; iter < maxIters; iter++) {
    let improved = false
    for (let i = 0; i < n; i++) {
      for (let j = i + 1; j < n; j++) {
        ;[perm[i], perm[j]] = [perm[j], perm[i]]
        const cand = scorePermutation(h, perm, mi)
        if (cand.score > best.score + 1e-12) {
          best = cand
          improved = true
        } else {
          ;[perm[i], perm[j]] = [perm[j], perm[i]]
        }
      }
    }
    if (!improved) break
  }
  return best
}

function searchFactorization(
  h: Hamiltonian,
  state: QuantumState,
  topK: number,
): [FactorizationCandidate, FactorizationCandidate, FactorizationCandidate[]] {
  const n = h.n
  const mi = mutualInformationMatrix(state)
  const baseline = scorePermutation(h, identityPerm(n), mi)

  let candidates: FactorizationCandidate[]
  if (n <= 8) {
    candidates = allPermutations(n).map((perm) => scorePermutation(h, perm, mi))
  } else {
    const rng = makeRng(991)
    candidates = [baseline]
    for (let i = 0; i < 32; i++) {
      candidates.push(scorePermutation(h, randomPermutation(n, rng), mi))
    }
    candidates.push(greedySearch(h, mi, 40))
  }

  candidates.sort((a, b) => b.score - a.score)
  const best = candidates[0] ?? baseline
  const top = candidates.slice(0, topK)
  return [baseline, best, top]
}

function shuffleHamiltonian(
  h: Hamiltonian,
  seed: number,
): [Hamiltonian, number[]] {
  const rng = makeRng(seed + 4242)
  const perm = randomPermutation(h.n, rng)
  return [remapHamiltonian(h, perm), perm]
}

/** TypeScript reference implementation (fallback when WASM unavailable). */
export function runFactorizationSearch(
  config: FactorizationSearchConfig,
): FactorizationSearchResult {
  const start = performance.now()
  const topK = Math.min(20, Math.max(1, config.topK ?? 5))

  let label: string
  let hamiltonian: Hamiltonian
  let trueShuffle: number[] | undefined

  if (config.kind === 'shuffled_chain') {
    const model = tfimChain(config.n, 1, config.field)
    const [shuffled, shufflePerm] = shuffleHamiltonian(model.hamiltonian, config.seed)
    label = `Shuffled TFIM chain (n=${config.n})`
    hamiltonian = shuffled
    trueShuffle = shufflePerm
  } else {
    const model = randomNonlocal(config.n, makeRng(config.seed))
    label = model.label
    hamiltonian = model.hamiltonian
  }

  const rng = makeRng(config.seed)
  const { state, energy } = groundState(hamiltonian, rng, { maxIters: 4000 })
  const [baseline, best, topCandidates] = searchFactorization(
    hamiltonian,
    state,
    topK,
  )

  let recoveredIdentity = false
  if (trueShuffle) {
    const n = config.n
    const invShuffle = Array.from({ length: n }, () => 0)
    trueShuffle.forEach((p, q) => {
      invShuffle[p] = q
    })
    const recovery = scorePermutation(
      hamiltonian,
      invShuffle,
      mutualInformationMatrix(state),
    )
    recoveredIdentity =
      recovery.localityFraction > 0.99 && recovery.nonlocalTerms === 0
  } else {
    recoveredIdentity = best.score > baseline.score + 0.05
  }

  return {
    label,
    n: config.n,
    energy,
    baseline,
    best,
    topCandidates,
    recoveredIdentity,
    trueShuffle,
    elapsedMs: performance.now() - start,
  }
}
