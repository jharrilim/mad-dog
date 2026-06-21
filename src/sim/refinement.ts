/**
 * Adaptive holographic refinement diagnostics.
 *
 * Speculative criterion: a fixed factor count n is "adequate" when |ψ⟩ under a
 * local Hamiltonian simultaneously supports (i) a low-dimensional emergent MI
 * geometry, (ii) an area-law-like entropy profile, and (iii) a discrete RT
 * relation S_A ≈ slope × (boundary cut). When those fail under quench growth of
 * entanglement, refinement pressure rises — a toy signal that a larger n might
 * be required to restore holographic scaling.
 */

import { analyzeEmergentGeometry, entropyOfRegion } from './geometry.ts'
import { analyzeRtRelation } from './holography.ts'
import {
  groundState,
  makeRandomState,
  makeRng,
  makeZeroState,
  type QuantumState,
} from './quantum.ts'
import { evolveInterval } from './spacetime.ts'
import { tfimChain } from './models.ts'

export interface RefinementBaselines {
  /** S(n/2)/S(1) for the gapped ground state at this n. */
  groundAreaGrowth: number
  /** Same ratio for a Haar-random state (volume-law proxy). */
  randomAreaGrowth: number
}

export interface RefinementThresholds {
  /** Composite pressure above which we flag "needs refinement". */
  pressure: number
  /** Minimum RT fit quality. */
  minRtR2: number
  /** |slope − 1| tolerance for the discrete RT relation. */
  maxRtSlopeDev: number
}

export const DEFAULT_REFINEMENT_THRESHOLDS: RefinementThresholds = {
  pressure: 0.42,
  minRtR2: 0.82,
  maxRtSlopeDev: 0.35,
}

export interface RefinementReasons {
  rtFit: boolean
  rtSlope: boolean
  areaLaw: boolean
  emergentDim: boolean
}

export interface RefinementDiagnostics {
  n: number
  rtSlope: number
  rtR2: number
  emergentDim: number
  /** Edge-anchored S(n/2) / S(1); rises toward volume-law entanglement. */
  areaGrowth: number
  /** Normalized 0..1: 0 ≈ ground-like, 1 ≈ random-like area growth. */
  areaPressure: number
  /** Weighted composite 0..1; higher ⇒ holographic consistency breaking down. */
  pressure: number
  needsRefinement: boolean
  reasons: RefinementReasons
}

function edgeAnchoredEntropies(state: QuantumState): number[] {
  const n = state.n
  const out: number[] = []
  for (let L = 1; L < n; L++) {
    out.push(entropyOfRegion(state, Array.from({ length: L }, (_, i) => i)))
  }
  return out
}

/** S(L_mid)/S(1) for edge-anchored intervals; used as an area-vs-volume proxy. */
export function areaGrowthRatio(state: QuantumState): number {
  const ent = edgeAnchoredEntropies(state)
  if (ent.length < 1) return 0
  const s1 = ent[0]
  const mid = ent[Math.floor(ent.length / 2)]
  return mid / (s1 + 1e-9)
}

export function computeRefinementBaselines(
  n: number,
  field: number,
  seed: number,
): RefinementBaselines {
  const model = tfimChain(n, 1, field)
  const { state: ground } = groundState(model.hamiltonian, makeRng(seed), {
    maxIters: 4000,
  })
  const random = makeRandomState(n, makeRng(seed + 4242))
  return {
    groundAreaGrowth: areaGrowthRatio(ground),
    randomAreaGrowth: areaGrowthRatio(random),
  }
}

function clamp01(x: number): number {
  return Math.max(0, Math.min(1, x))
}

export function measureRefinementDiagnostics(
  state: QuantumState,
  expectedDim: number,
  baselines: RefinementBaselines,
  thresholds: RefinementThresholds = DEFAULT_REFINEMENT_THRESHOLDS,
): RefinementDiagnostics {
  const { rtSlope, rtR2 } = analyzeRtRelation(state)
  const { mds } = analyzeEmergentGeometry(state)
  const emergentDim = mds.emergentDim
  const areaGrowth = areaGrowthRatio(state)

  const span =
    baselines.randomAreaGrowth - baselines.groundAreaGrowth + 1e-9
  const areaPressure = clamp01(
    (areaGrowth - baselines.groundAreaGrowth) / span,
  )

  const unentangled = areaGrowth < 1e-4 && rtR2 < 1e-4
  if (unentangled) {
    return {
      n: state.n,
      rtSlope: 1,
      rtR2: 1,
      emergentDim,
      areaGrowth,
      areaPressure: 0,
      pressure: 0,
      needsRefinement: false,
      reasons: {
        rtFit: false,
        rtSlope: false,
        areaLaw: false,
        emergentDim: false,
      },
    }
  }

  const rtSlopeSafe = Number.isFinite(rtSlope) ? rtSlope : 1
  const rtR2Safe = Number.isFinite(rtR2) ? rtR2 : 0

  const rtDeficit = clamp01(1 - rtR2Safe)
  const slopeDeficit = clamp01(
    Math.abs(rtSlopeSafe - 1) / thresholds.maxRtSlopeDev,
  )
  const dimPressure = clamp01(
    (emergentDim - expectedDim) / Math.max(1, expectedDim + 0.5),
  )

  const pressure = clamp01(
    0.35 * rtDeficit +
      0.25 * slopeDeficit +
      0.25 * areaPressure +
      0.15 * dimPressure,
  )

  const reasons: RefinementReasons = {
    rtFit: rtR2Safe < thresholds.minRtR2,
    rtSlope: Math.abs(rtSlopeSafe - 1) > thresholds.maxRtSlopeDev,
    areaLaw: areaPressure > 0.55,
    emergentDim: emergentDim > expectedDim + 0.5,
  }

  const needsRefinement =
    pressure >= thresholds.pressure ||
    (reasons.rtFit && reasons.areaLaw) ||
    (reasons.rtSlope && reasons.emergentDim)

  return {
    n: state.n,
    rtSlope: rtSlopeSafe,
    rtR2: rtR2Safe,
    emergentDim,
    areaGrowth,
    areaPressure,
    pressure,
    needsRefinement,
    reasons,
  }
}

export function defectInitialState(n: number): QuantumState {
  const center = Math.floor(n / 2)
  const initial = makeZeroState(n)
  initial.data[2 * (1 << center)] = 1
  return initial
}

export interface RefinementQuenchSlice {
  t: number
  step: number
  diagnostics: RefinementDiagnostics
}

export interface RefinementQuenchResult {
  n: number
  field: number
  dt: number
  baselines: RefinementBaselines
  slices: RefinementQuenchSlice[]
  elapsedMs: number
}

export interface RefinementQuenchConfig {
  n: number
  field: number
  dt: number
  steps: number
  seed: number
}

/** Evolve a central defect quench and track refinement pressure over clock time. */
export function runRefinementQuench(
  config: RefinementQuenchConfig,
): RefinementQuenchResult {
  const start = performance.now()
  const model = tfimChain(config.n, 1, config.field)
  const radius = Math.max(
    model.hamiltonian.estimateSpectralRadius(() => 0.5),
    1e-6,
  )
  const baselines = computeRefinementBaselines(
    config.n,
    config.field,
    config.seed,
  )
  let state = defectInitialState(config.n)
  const slices: RefinementQuenchSlice[] = []

  for (let step = 0; step <= config.steps; step++) {
    slices.push({
      t: step * config.dt,
      step,
      diagnostics: measureRefinementDiagnostics(state, 1, baselines),
    })
    if (step < config.steps) {
      state = evolveInterval(model.hamiltonian, state, config.dt, radius)
    }
  }

  return {
    n: config.n,
    field: config.field,
    dt: config.dt,
    baselines,
    slices,
    elapsedMs: performance.now() - start,
  }
}

export interface RefinementNCompareResult {
  n: number
  nLarge: number
  field: number
  quenchStep: number
  dt: number
  small: RefinementDiagnostics
  large: RefinementDiagnostics
  /** True when the larger chain has strictly lower pressure after the same quench. */
  largerRelieves: boolean
  elapsedMs: number
}

export interface RefinementNCompareConfig {
  n: number
  /** Added to n for the comparison chain (default 2 keeps even lengths). */
  deltaN?: number
  field: number
  dt: number
  quenchStep: number
  seed: number
}

/**
 * Same defect quench evolved for a fixed number of steps at n and n+Δn.
 * Tests whether "adding factors" lowers holographic refinement pressure.
 */
export function runRefinementNCompare(
  config: RefinementNCompareConfig,
): RefinementNCompareResult {
  const start = performance.now()
  const deltaN = config.deltaN ?? 2
  const nLarge = config.n + deltaN

  function stateAfterQuench(chainN: number): RefinementDiagnostics {
    const model = tfimChain(chainN, 1, config.field)
    const radius = Math.max(
      model.hamiltonian.estimateSpectralRadius(() => 0.5),
      1e-6,
    )
    const baselines = computeRefinementBaselines(
      chainN,
      config.field,
      config.seed,
    )
    let state = defectInitialState(chainN)
    for (let k = 0; k < config.quenchStep; k++) {
      state = evolveInterval(model.hamiltonian, state, config.dt, radius)
    }
    return measureRefinementDiagnostics(state, 1, baselines)
  }

  const small = stateAfterQuench(config.n)
  const large = stateAfterQuench(nLarge)

  return {
    n: config.n,
    nLarge,
    field: config.field,
    quenchStep: config.quenchStep,
    dt: config.dt,
    small,
    large,
    largerRelieves: large.pressure < small.pressure - 1e-6,
    elapsedMs: performance.now() - start,
  }
}
