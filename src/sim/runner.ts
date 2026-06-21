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
  normalize,
  type QuantumState,
} from './quantum.ts'
import {
  cubeEdges,
  randomNonlocal,
  tfimChain,
  tfimCube,
  tfimGrid,
  type BuiltModel,
} from './models.ts'
import { analyzeEmergentGeometry, type EmergenceReport } from './geometry.ts'
import {
  buildSpacetime,
  evolveInterval,
  measureLightCone,
  type LightCone,
  type SpacetimeResult,
} from './spacetime.ts'
import { analyzeHolography, analyzeRtMassDeformation, type HolographyReport, type RtMassReport } from './holography.ts'
import { buildDualClock, type DualClockResult } from './relational-time.ts'

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

export interface RtMassRunConfig {
  n: number
  field: number
  seed: number
  /** Qubit where the excitation ("mass") is injected. */
  massSite?: number
  /** Quench evolution time — mass strength. */
  strength: number
}

export interface RtMassRunResult {
  label: string
  report: RtMassReport
  elapsedMs: number
}

/**
 * Deform the RT slope by concentrating entanglement at a local excitation.
 * Compare vacuum (ground state) vs. mass-perturbed RT relation.
 */
export function runRtMass(config: RtMassRunConfig): RtMassRunResult {
  const start = performance.now()
  const model = tfimChain(config.n, 1, config.field)
  const rng = makeRng(config.seed)
  const { state } = groundState(model.hamiltonian, rng, { maxIters: 4000 })
  const massSite = config.massSite ?? Math.floor(config.n / 2)
  const report = analyzeRtMassDeformation(
    model.hamiltonian,
    state,
    massSite,
    config.strength,
  )
  return {
    label: model.label,
    report,
    elapsedMs: performance.now() - start,
  }
}

export interface RelationalTimeConfig {
  n: number
  field: number
  dt: number
  steps: number
  /** Qubit whose local disturbance defines the physical clock. */
  clockSite: number
  physicalSlices: number
}

/**
 * Two-clock demo: same quench, same Hamiltonian, but time read off a uniform
 * clock vs. a physical clock at `clockSite`. The relational time map shows
 * how emergent histories diverge when clocks disagree.
 */
export function runRelationalTime(
  config: RelationalTimeConfig,
): DualClockResult {
  const model = tfimChain(config.n, 1, config.field)
  const center = Math.floor(config.n / 2)
  const initial = makeZeroState(config.n)
  initial.data[2 * (1 << center)] = 1
  const reference = makeZeroState(config.n)
  reference.data[0] = 1
  return buildDualClock({
    hamiltonian: model.hamiltonian,
    initial,
    reference,
    dt: config.dt,
    steps: config.steps,
    clockSite: config.clockSite,
    physicalSlices: config.physicalSlices,
  })
}

export interface Universe3DConfig {
  lx: number
  ly: number
  lz: number
  field: number
  dt: number
  steps: number
}

export interface Universe3DResult {
  model: BuiltModel
  spacetime: SpacetimeResult
  lightCone: LightCone
  defectSite: number
  edges: [number, number][]
  /** Manhattan distance from defect on the true lattice. */
  siteDistances: number[]
  elapsedMs: number
}

function cubeDefectSite(lx: number, ly: number, lz: number): number {
  const cx = Math.floor(lx / 2)
  const cy = Math.floor(ly / 2)
  const cz = Math.floor(lz / 2)
  return cz * (lx * ly) + cy * lx + cx
}

function latticeDistances(
  positions: { x: number; y: number; z?: number }[],
  center: number,
): number[] {
  const c = positions[center]
  const cz = c.z ?? 0
  return positions.map((p) =>
    Math.abs(p.x - c.x) + Math.abs(p.y - c.y) + Math.abs((p.z ?? 0) - cz),
  )
}

/**
 * 3+1 universe lab: TFIM cube quench with emergent 3D MDS geometry per clock
 * slice (Page-Wootters emergent time).
 */
export function runUniverse3D(config: Universe3DConfig): Universe3DResult {
  const start = performance.now()
  const { lx, ly, lz, field, dt, steps } = config
  const model = tfimCube(lx, ly, lz, 1, field)
  const n = model.hamiltonian.n
  const defectSite = cubeDefectSite(lx, ly, lz)
  const initial = makeZeroState(n)
  initial.data[2 * (1 << defectSite)] = 1
  const reference = makeZeroState(n)
  reference.data[0] = 1
  const siteDistances = latticeDistances(model.layout.truePositions, defectSite)
  const spacetime = buildSpacetime({
    hamiltonian: model.hamiltonian,
    initial,
    reference,
    dt,
    steps,
    embedDim: 3,
    alignTo: model.layout.truePositions,
  })
  const lightCone = measureLightCone(
    spacetime,
    0.12,
    defectSite,
    siteDistances,
  )
  return {
    model,
    spacetime,
    lightCone,
    defectSite,
    edges: cubeEdges(lx, ly, lz),
    siteDistances,
    elapsedMs: performance.now() - start,
  }
}

/** Re-evolve the quenched initial state to clock reading k (for on-demand MI). */
export function stateAtUniverseSlice(
  config: Universe3DConfig,
  k: number,
): { state: QuantumState; defectSite: number } {
  const { lx, ly, lz, field, dt } = config
  const model = tfimCube(lx, ly, lz, 1, field)
  const defectSite = cubeDefectSite(lx, ly, lz)
  const n = model.hamiltonian.n
  const initial = makeZeroState(n)
  initial.data[2 * (1 << defectSite)] = 1
  const radius = Math.max(
    model.hamiltonian.estimateSpectralRadius(() => 0.5),
    1e-6,
  )
  let psi = initial
  for (let i = 0; i < k; i++) {
    psi = evolveInterval(model.hamiltonian, psi, dt, radius)
    normalize(psi)
  }
  return { state: psi, defectSite }
}
