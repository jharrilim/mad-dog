/**
 * A "baby Ryu-Takayanagi" test for the emergent geometry.
 *
 * Two honest checks, run on a ground state vs. a Haar-random state:
 *
 *  1. Area law vs. volume law. For nested regions A of growing size, how does
 *     the entanglement entropy S(A) scale? A gapped local ground state should
 *     obey an AREA law: S(A) tracks the *boundary* of A (it saturates). A
 *     random state obeys a VOLUME law: S(A) grows with the *size* of A (a Page
 *     curve peaking at half the system). Entropy ~ boundary, not bulk, is the
 *     defining holographic signature.
 *
 *  2. The discrete Ryu-Takayanagi / "entropy = area" relation. The essay
 *     (Carroll-Singh, building on Cao-Carroll-Michalakis) notes that for
 *     redundancy-constrained states the region entropy equals a boundary sum of
 *     mutual informations:
 *
 *         S_A  ~  (1/2) * sum_{a in A, b not in A} I(a:b).
 *
 *     The RHS is literally the total entanglement crossing the boundary of A:
 *     the discrete analog of a minimal-surface "area". We compute exact S_A and
 *     this boundary cut for every contiguous interval and ask whether they line
 *     up (best-fit slope and R^2 through the origin).
 */

import { entropyOfRegion, mutualInformationMatrix } from './geometry.ts'
import type { QuantumState } from './quantum.ts'

export interface AreaLawPoint {
  /** size |A| of the (edge-anchored) region */
  size: number
  /** entropy of the gapped ground state */
  sGround: number
  /** entropy of the Haar-random state (volume-law reference) */
  sRandom: number
}

export interface RtPoint {
  /** exact von Neumann entropy S_A */
  entropy: number
  /** boundary cut (1/2) sum_{a in A, b not in A} I(a:b) */
  cut: number
  size: number
  start: number
}

export interface HolographyReport {
  n: number
  areaLaw: AreaLawPoint[]
  rtPoints: RtPoint[]
  /** best-fit slope of S_A vs. boundary cut, forced through the origin */
  rtSlope: number
  /** coefficient of determination of that fit */
  rtR2: number
}

function interval(start: number, length: number): number[] {
  return Array.from({ length }, (_, i) => start + i)
}

export function analyzeHolography(
  ground: QuantumState,
  random: QuantumState,
): HolographyReport {
  const n = ground.n
  const mi = mutualInformationMatrix(ground)

  // --- Area law: nested edge-anchored regions A = {0, 1, ..., L-1} ---
  const areaLaw: AreaLawPoint[] = []
  for (let L = 1; L < n; L++) {
    const region = interval(0, L)
    areaLaw.push({
      size: L,
      sGround: entropyOfRegion(ground, region),
      sRandom: entropyOfRegion(random, region),
    })
  }

  // --- RT relation: every contiguous interval, exact S_A vs. boundary MI cut ---
  const rtPoints: RtPoint[] = []
  for (let start = 0; start < n; start++) {
    for (let L = 1; start + L < n; L++) {
      const region = interval(start, L)
      const inRegion = new Array<boolean>(n).fill(false)
      for (const q of region) inRegion[q] = true
      let cut = 0
      for (const a of region) {
        for (let b = 0; b < n; b++) if (!inRegion[b]) cut += mi[a][b]
      }
      cut *= 0.5
      rtPoints.push({
        entropy: entropyOfRegion(ground, region),
        cut,
        size: L,
        start,
      })
    }
  }

  // Least-squares fit S = slope * cut (through the origin) + R^2.
  let sxy = 0
  let sxx = 0
  let sy = 0
  for (const p of rtPoints) {
    sxy += p.cut * p.entropy
    sxx += p.cut * p.cut
    sy += p.entropy
  }
  const rtSlope = sxx > 0 ? sxy / sxx : 0
  const yBar = rtPoints.length > 0 ? sy / rtPoints.length : 0
  let ssRes = 0
  let ssTot = 0
  for (const p of rtPoints) {
    const pred = rtSlope * p.cut
    ssRes += (p.entropy - pred) ** 2
    ssTot += (p.entropy - yBar) ** 2
  }
  const rtR2 = ssTot > 0 ? 1 - ssRes / ssTot : 0

  return { n, areaLaw, rtPoints, rtSlope, rtR2 }
}
