/**
 * Two-clock / relational time.
 *
 * Page-Wootters time is not absolute: it is recovered by conditioning on a
 * chosen clock subsystem. Different clocks define different emergent time
 * coordinates for the same underlying quantum history.
 *
 * We evolve one physical trajectory and read it off with two clocks:
 *
 *   Clock A (uniform): fixed Δt ticks — a "global" discretization.
 *   Clock B (physical): ticks when the disturbance signal at a chosen site
 *     crosses evenly spaced thresholds — a clock *inside* the system that
 *     stalls until the quench reaches it, then runs fast.
 *
 * The map t_A ↔ t_B is generally nonlinear: neither clock is "the" time.
 */

import {
  cloneState,
  expectationZ,
  Hamiltonian,
  normalize,
  type QuantumState,
} from './quantum.ts'
import { analyzeEmergentGeometry } from './geometry.ts'
import {
  evolveInterval,
  type SpacetimeResult,
  type SpacetimeSlice,
} from './spacetime.ts'

export interface TrajectoryPoint {
  t: number
  psi: QuantumState
  ref: QuantumState | null
}

export interface DualClockConfig {
  hamiltonian: Hamiltonian
  initial: QuantumState
  reference?: QuantumState
  /** Physical time step for the stored trajectory. */
  dt: number
  /** Number of trajectory points (uniform clock uses all of them). */
  steps: number
  /** Qubit whose local signal defines the physical clock. */
  clockSite: number
  /** Number of physical-clock slices to extract. */
  physicalSlices: number
  embedDim?: number
  order?: number
}

export interface ClockHistory {
  label: string
  /** Which qubit defines this clock, if any. */
  clockSite?: number
  result: SpacetimeResult
}

export interface DualClockResult {
  sites: number
  /** Uniform Δt clock. */
  uniform: ClockHistory
  /** Event-based clock at `clockSite`. */
  physical: ClockHistory
  /**
   * Relational time map: at physical-clock tick k, what uniform-clock reading
   * index records the same system state? Nonlinearity means the clocks disagree.
   */
  timeMap: { tauUniform: number; tauPhysical: number }[]
  /** R² of linear fit tauUniform = a * tauPhysical + b (1 = synchronized). */
  syncR2: number
  /** Slope of affine fit tauUniform = slope * tauPhysical + intercept. */
  syncSlope: number
  syncIntercept: number
  elapsedMs: number
}

function buildSlice(
  hamiltonian: Hamiltonian,
  psi: QuantumState,
  ref: QuantumState | null,
  baseZ: number[],
  t: number,
  k: number,
  embedDim: number,
): SpacetimeSlice {
  const sites = hamiltonian.n
  const energy = hamiltonian.expectation(psi)
  const report = analyzeEmergentGeometry(psi, 1, Math.max(embedDim, 1))
  let coords: number[][]
  if (embedDim >= 2) {
    coords = report.mds.coords.map((c) => [c[0] ?? 0, c[1] ?? 0])
  } else {
    const raw = report.mds.coords.map((c) => c[0] ?? 0)
    let cov = 0
    const meanI = (sites - 1) / 2
    const meanC = raw.reduce((a, b) => a + b, 0) / sites
    for (let i = 0; i < sites; i++) cov += (i - meanI) * (raw[i] - meanC)
    const oriented = cov < 0 ? raw.map((c) => -c) : raw
    coords = oriented.map((x) => [x])
  }
  const zExpectation = Array.from({ length: sites }, (_, q) => expectationZ(psi, q))
  const refZ = ref
    ? Array.from({ length: sites }, (_, q) => expectationZ(ref, q))
    : baseZ
  const signal = zExpectation.map((z, q) => Math.abs(z - refZ[q]))
  return { k, t, coords, zExpectation, signal, energy }
}

/** Evolve and store the full trajectory at fixed Δt. */
export function evolveTrajectory(config: {
  hamiltonian: Hamiltonian
  initial: QuantumState
  reference?: QuantumState
  dt: number
  steps: number
  order?: number
}): TrajectoryPoint[] {
  const { hamiltonian, initial, reference, dt, steps, order = 6 } = config
  const radius = Math.max(hamiltonian.estimateSpectralRadius(() => 0.5), 1e-6)
  const points: TrajectoryPoint[] = []
  let psi = cloneState(initial)
  let ref = reference ? cloneState(reference) : null
  for (let k = 0; k < steps; k++) {
    points.push({ t: k * dt, psi: cloneState(psi), ref: ref ? cloneState(ref) : null })
    psi = evolveInterval(hamiltonian, psi, dt, radius, order)
    normalize(psi)
    if (ref) {
      ref = evolveInterval(hamiltonian, ref, dt, radius, order)
      normalize(ref)
    }
  }
  return points
}

function signalAtSite(point: TrajectoryPoint, site: number, baseZ: number[]): number {
  const z = expectationZ(point.psi, site)
  const refZ = point.ref ? expectationZ(point.ref, site) : baseZ[site]
  return Math.abs(z - refZ)
}

function slicesFromIndices(
  hamiltonian: Hamiltonian,
  trajectory: TrajectoryPoint[],
  indices: number[],
  times: number[],
  initial: QuantumState,
  embedDim: number,
): SpacetimeSlice[] {
  const baseZ = Array.from({ length: hamiltonian.n }, (_, q) =>
    expectationZ(initial, q),
  )
  return indices.map((idx, k) =>
    buildSlice(
      hamiltonian,
      trajectory[idx].psi,
      trajectory[idx].ref,
      baseZ,
      times[k],
      k,
      embedDim,
    ),
  )
}

/** Pick physical-clock ticks when the signal at `clockSite` crosses thresholds. */
function physicalClockIndices(
  trajectory: TrajectoryPoint[],
  clockSite: number,
  baseZ: number[],
  numSlices: number,
): { indices: number[]; times: number[] } {
  const signals = trajectory.map((p) => signalAtSite(p, clockSite, baseZ))
  const maxSig = Math.max(...signals, 1e-9)
  const targets = Array.from({ length: numSlices }, (_, i) =>
    (maxSig * (i + 0.5)) / numSlices,
  )

  const indices: number[] = [0]
  const times: number[] = [0]
  let searchFrom = 1
  for (const target of targets) {
    let found = -1
    for (let i = searchFrom; i < trajectory.length; i++) {
      if (signals[i] >= target) {
        found = i
        break
      }
    }
    if (found < 0) break
    indices.push(found)
    times.push(trajectory[found].t)
    searchFrom = found + 1
  }

  // Ensure at least two slices for plotting.
  if (indices.length < 2 && trajectory.length > 1) {
    indices.push(trajectory.length - 1)
    times.push(trajectory[trajectory.length - 1].t)
  }
  return { indices, times }
}

function fitAffine(xs: number[], ys: number[]): { slope: number; intercept: number; r2: number } {
  if (xs.length < 2) return { slope: 1, intercept: 0, r2: 1 }
  const n = xs.length
  let sxx = 0
  let sxy = 0
  let sx = 0
  let sy = 0
  for (let i = 0; i < n; i++) {
    sxx += xs[i] * xs[i]
    sxy += xs[i] * ys[i]
    sx += xs[i]
    sy += ys[i]
  }
  const denom = n * sxx - sx * sx
  const slope = denom !== 0 ? (n * sxy - sx * sy) / denom : 1
  const intercept = (sy - slope * sx) / n
  const yBar = sy / n
  let ssRes = 0
  let ssTot = 0
  for (let i = 0; i < n; i++) {
    const pred = slope * xs[i] + intercept
    ssRes += (ys[i] - pred) ** 2
    ssTot += (ys[i] - yBar) ** 2
  }
  const r2 = ssTot > 0 ? 1 - ssRes / ssTot : 1
  return { slope, intercept, r2 }
}

/** Build two emergent histories from one shared quantum trajectory. */
export function buildDualClock(config: DualClockConfig): DualClockResult {
  const start = performance.now()
  const {
    hamiltonian,
    initial,
    reference,
    dt,
    steps,
    clockSite,
    physicalSlices,
    embedDim = 1,
    order = 6,
  } = config
  const sites = hamiltonian.n

  const trajectory = evolveTrajectory({
    hamiltonian,
    initial,
    reference,
    dt,
    steps,
    order,
  })

  const baseZ = Array.from({ length: sites }, (_, q) => expectationZ(initial, q))
  const energy0 = hamiltonian.expectation(initial)

  // Uniform clock: every trajectory point, t = k * dt.
  const uniformIndices = trajectory.map((_, k) => k)
  const uniformTimes = trajectory.map((p) => p.t)
  const uniformSliceList = slicesFromIndices(
    hamiltonian,
    trajectory,
    uniformIndices,
    uniformTimes,
    initial,
    embedDim,
  )
  let energyDrift = 0
  for (const s of uniformSliceList) {
    energyDrift = Math.max(energyDrift, Math.abs(s.energy - energy0))
  }

  // Physical clock: event-based at clockSite.
  const { indices: physIndices } = physicalClockIndices(
    trajectory,
    clockSite,
    baseZ,
    physicalSlices,
  )
  const physicalSliceList = slicesFromIndices(
    hamiltonian,
    trajectory,
    physIndices,
    // Physical clock's own relational time coordinate = tick index, not Schrödinger t.
    physIndices.map((_, k) => k),
    initial,
    embedDim,
  )
  for (const s of physicalSliceList) {
    energyDrift = Math.max(energyDrift, Math.abs(s.energy - energy0))
  }

  // Relational time map: uniform reading index vs physical tick index at same state.
  const timeMap = physIndices.map((uniformIdx, kPhysical) => ({
    tauUniform: uniformIdx,
    tauPhysical: kPhysical,
  }))

  const { slope: syncSlope, intercept: syncIntercept, r2: syncR2 } = fitAffine(
    timeMap.map((p) => p.tauPhysical),
    timeMap.map((p) => p.tauUniform),
  )

  const elapsedMs = performance.now() - start

  return {
    sites,
    uniform: {
      label: 'Uniform clock (fixed Δt)',
      result: {
        sites,
        slices: uniformSliceList,
        energyDrift,
        elapsedMs,
      },
    },
    physical: {
      label: `Physical clock (site ${clockSite})`,
      clockSite,
      result: {
        sites,
        slices: physicalSliceList,
        energyDrift,
        elapsedMs,
      },
    },
    timeMap,
    syncR2,
    syncSlope,
    syncIntercept,
    elapsedMs,
  }
}
