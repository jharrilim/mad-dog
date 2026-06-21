/**
 * RT relation analysis and "mass" deformation tests.
 *
 * In the vacuum (ground state), S_A ≈ (1/2) Σ I(a:b) with slope ≈ 1.
 * Injecting a local excitation — our stand-in for mass — concentrates
 * entanglement and deforms the slope, hinting at an emergent metric response.
 */

import { entropyOfRegion, mutualInformationMatrix } from './geometry.ts'
import {
  cloneState,
  kickX,
  normalize,
  type Hamiltonian,
  type QuantumState,
} from './quantum.ts'
import { evolveInterval } from './spacetime.ts'

export interface AreaLawPoint {
  size: number
  sGround: number
  sRandom: number
}

export interface RtPoint {
  entropy: number
  cut: number
  size: number
  start: number
  /** Minimum distance from region to the mass site (for coloring). */
  distFromMass?: number
}

export interface RtFit {
  rtPoints: RtPoint[]
  rtSlope: number
  rtR2: number
}

export interface HolographyReport {
  n: number
  areaLaw: AreaLawPoint[]
  rtPoints: RtPoint[]
  rtSlope: number
  rtR2: number
}

export interface RtMassReport {
  n: number
  massSite: number
  strength: number
  vacuum: RtFit
  mass: RtFit
  /** RT slope vs. mass strength (vacuum at 0, then increasing quench time). */
  sweep: { strength: number; slope: number; r2: number }[]
}

function interval(start: number, length: number): number[] {
  return Array.from({ length }, (_, i) => start + i)
}

function fitRtThroughOrigin(points: { entropy: number; cut: number }[]): {
  rtSlope: number
  rtR2: number
} {
  let sxy = 0
  let sxx = 0
  let sy = 0
  for (const p of points) {
    sxy += p.cut * p.entropy
    sxx += p.cut * p.cut
    sy += p.entropy
  }
  const rtSlope = sxx > 0 ? sxy / sxx : 0
  const yBar = points.length > 0 ? sy / points.length : 0
  let ssRes = 0
  let ssTot = 0
  for (const p of points) {
    const pred = rtSlope * p.cut
    ssRes += (p.entropy - pred) ** 2
    ssTot += (p.entropy - yBar) ** 2
  }
  const rtR2 = ssTot > 0 ? 1 - ssRes / ssTot : 0
  return { rtSlope, rtR2 }
}

function regionDistFromMass(region: number[], massSite: number): number {
  let min = Infinity
  for (const q of region) min = Math.min(min, Math.abs(q - massSite))
  return min
}

/** Compute the discrete RT relation S_A vs. boundary MI cut for every interval. */
export function analyzeRtRelation(
  state: QuantumState,
  massSite?: number,
): RtFit {
  const n = state.n
  const mi = mutualInformationMatrix(state)
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
        entropy: entropyOfRegion(state, region),
        cut,
        size: L,
        start,
        distFromMass:
          massSite !== undefined ? regionDistFromMass(region, massSite) : undefined,
      })
    }
  }

  const { rtSlope, rtR2 } = fitRtThroughOrigin(rtPoints)
  return { rtPoints, rtSlope, rtR2 }
}

export function analyzeHolography(
  ground: QuantumState,
  random: QuantumState,
): HolographyReport {
  const n = ground.n
  const { rtPoints, rtSlope, rtR2 } = analyzeRtRelation(ground)

  const areaLaw: AreaLawPoint[] = []
  for (let L = 1; L < n; L++) {
    const region = interval(0, L)
    areaLaw.push({
      size: L,
      sGround: entropyOfRegion(ground, region),
      sRandom: entropyOfRegion(random, region),
    })
  }

  return { n, areaLaw, rtPoints, rtSlope, rtR2 }
}

/**
 * Inject "mass" at `massSite`: flip the local spin (X kick) and evolve briefly
 * under H so entanglement concentrates around the excitation. `strength` is the
 * evolution time (0 = vacuum ground state).
 */
export function injectMass(
  h: Hamiltonian,
  ground: QuantumState,
  massSite: number,
  strength: number,
): QuantumState {
  if (strength <= 0) return cloneState(ground)
  const radius = Math.max(h.estimateSpectralRadius(() => 0.5), 1e-6)
  let psi = kickX(ground, massSite)
  normalize(psi)
  psi = evolveInterval(h, psi, strength, radius)
  normalize(psi)
  return psi
}

/** Vacuum vs. mass-deformed RT relation, plus a slope sweep over mass strength. */
export function analyzeRtMassDeformation(
  h: Hamiltonian,
  ground: QuantumState,
  massSite: number,
  strength: number,
  sweepSteps = 9,
  maxStrength = 2,
): RtMassReport {
  const n = ground.n
  const vacuum = analyzeRtRelation(ground, massSite)
  const massState = injectMass(h, ground, massSite, strength)
  const mass = analyzeRtRelation(massState, massSite)

  const sweep: RtMassReport['sweep'] = [
    { strength: 0, slope: vacuum.rtSlope, r2: vacuum.rtR2 },
  ]
  for (let i = 1; i < sweepSteps; i++) {
    const s = (maxStrength * i) / (sweepSteps - 1)
    const st = injectMass(h, ground, massSite, s)
    const fit = analyzeRtRelation(st)
    sweep.push({ strength: s, slope: fit.rtSlope, r2: fit.rtR2 })
  }

  return { n, massSite, strength, vacuum, mass, sweep }
}
