/**
 * High-level driver that the UI calls: pick a model, find its low-energy state,
 * and run the emergent-geometry pipeline. Kept separate from the React layer so
 * it can later move into a Web Worker / WebGPU backend without UI changes.
 */

import {
  groundState,
  makeRandomState,
  makeRng,
  makeZeroState,
} from './quantum.ts'
import {
  randomNonlocal,
  tfimChain,
  tfimGrid,
  type BuiltModel,
} from './models.ts'
import { analyzeEmergentGeometry, type EmergenceReport } from './geometry.ts'
import { buildSpacetime, type SpacetimeResult } from './spacetime.ts'
import { analyzeHolography, type HolographyReport } from './holography.ts'

export type ModelKind = 'chain' | 'grid' | 'random'

export interface RunConfig {
  kind: ModelKind
  /** qubit count for chain/random */
  n: number
  /** grid dimensions */
  rows: number
  cols: number
  /** transverse field strength (h > 1 => paramagnetic, correlations decay) */
  field: number
  seed: number
}

export interface RunResult {
  label: string
  qubits: number
  energy: number
  iters: number
  expectedDim: number
  report: EmergenceReport
  truePositions: { x: number; y: number }[]
  elapsedMs: number
}

function buildModel(config: RunConfig): BuiltModel {
  switch (config.kind) {
    case 'chain':
      return tfimChain(config.n, 1, config.field)
    case 'grid':
      return tfimGrid(config.rows, config.cols, 1, config.field)
    case 'random':
      return randomNonlocal(config.n, makeRng(config.seed + 991))
  }
}

export function runEmergence(config: RunConfig): RunResult {
  const start = performance.now()
  const model = buildModel(config)
  const rng = makeRng(config.seed)
  const { state, energy, iters } = groundState(model.hamiltonian, rng, {
    maxIters: 4000,
  })
  const report = analyzeEmergentGeometry(state)
  return {
    label: model.label,
    qubits: model.hamiltonian.n,
    energy,
    iters,
    expectedDim: model.layout.expectedDim,
    report,
    truePositions: model.layout.truePositions,
    elapsedMs: performance.now() - start,
  }
}

export interface SpacetimeRunConfig {
  /** chain length */
  n: number
  /** transverse field (paramagnetic > 1 for clean geometry) */
  field: number
  /** clock-reading time step */
  dt: number
  /** number of clock readings */
  steps: number
  seed: number
}

/**
 * Emergent-spacetime demo: start from a polarized product state with a single
 * flipped ("defect") spin at the centre of a local chain, then build the
 * Page-Wootters history and its clock-conditioned emergent geometry. Under the
 * local Hamiltonian the defect spreads ballistically, tracing a light cone in
 * the emergent spacetime, while entanglement (hence the spatial geometry)
 * grows over emergent time.
 */
export function runSpacetime(config: SpacetimeRunConfig): SpacetimeResult {
  const model = tfimChain(config.n, 1, config.field)
  const center = Math.floor(config.n / 2)
  // |up ... up (down at center) ... up> = basis state with only the center bit set.
  const initial = makeZeroState(config.n)
  initial.data[2 * (1 << center)] = 1
  // Defect-free reference |up ... up> to isolate the light-cone signal.
  const reference = makeZeroState(config.n)
  reference.data[0] = 1
  return buildSpacetime({
    hamiltonian: model.hamiltonian,
    initial,
    reference,
    dt: config.dt,
    steps: config.steps,
  })
}

export interface Spacetime2DConfig {
  rows: number
  cols: number
  field: number
  dt: number
  steps: number
}

/**
 * 2D emergent-geometry-in-time demo: a local Ising grid with a central defect.
 * The disturbance spreads radially and the emergent 2D MDS geometry (aligned to
 * the grid to remove rotation/reflection gauge freedom) evolves across clock
 * readings: an emergent curved 2D spacetime slice sequence.
 */
export function runSpacetime2D(config: Spacetime2DConfig): SpacetimeResult {
  const model = tfimGrid(config.rows, config.cols, 1, config.field)
  const n = config.rows * config.cols
  const center = Math.floor(config.rows / 2) * config.cols + Math.floor(config.cols / 2)
  const initial = makeZeroState(n)
  initial.data[2 * (1 << center)] = 1
  const reference = makeZeroState(n)
  reference.data[0] = 1
  return buildSpacetime({
    hamiltonian: model.hamiltonian,
    initial,
    reference,
    dt: config.dt,
    steps: config.steps,
    embedDim: 2,
    alignTo: model.layout.truePositions,
  })
}

export interface HolographyRunConfig {
  /** chain length */
  n: number
  /** transverse field (h > 1 gapped paramagnet -> clean area law) */
  field: number
  seed: number
}

export interface HolographyRunResult {
  label: string
  energy: number
  report: HolographyReport
  elapsedMs: number
}

/**
 * Baby Ryu-Takayanagi test: take the gapped ground state of a local Ising chain
 * (area-law entanglement) and a Haar-random state (volume-law), then compare
 * how region entropy scales and whether the discrete "entropy = boundary area"
 * relation holds for the ground state.
 */
export function runHolography(config: HolographyRunConfig): HolographyRunResult {
  const start = performance.now()
  const model = tfimChain(config.n, 1, config.field)
  const rng = makeRng(config.seed)
  const { state, energy } = groundState(model.hamiltonian, rng, {
    maxIters: 4000,
  })
  const random = makeRandomState(config.n, makeRng(config.seed + 4242))
  const report = analyzeHolography(state, random)
  return {
    label: model.label,
    energy,
    report,
    elapsedMs: performance.now() - start,
  }
}
