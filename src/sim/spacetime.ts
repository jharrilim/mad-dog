/**
 * Emergent time via the Page-Wootters mechanism, coupled to the emergent-space
 * pipeline to produce an emergent *spacetime*.
 *
 * The idea (Page & Wootters 1983; the essay's "problem of time" section):
 * the universe is described by a single timeless global state |Psi> on
 * H_clock (x) H_system, annihilated by a constraint C = H_clock + H_system.
 * There is no external time. Yet "conditioning" the global state on a clock
 * reading,  |psi(t_k)> ~ <k|_clock |Psi>,  recovers ordinary Schrodinger
 * evolution of the system under H_system. Time is a correlation with the clock.
 *
 * Concretely we build the "history state"
 *     |Psi> = (1/sqrt(N)) sum_k |k>_clock (x) e^{-i H_system t_k} |psi_0>_system
 * which satisfies the discretized constraint, and read time off the clock. For
 * each clock reading we run the mutual-information -> MDS pipeline, so we watch
 * a spatial geometry evolve across clock readings: an emergent spacetime.
 */

import {
  cloneState,
  expectationZ,
  Hamiltonian,
  normalize,
  type QuantumState,
} from './quantum.ts'
import { analyzeEmergentGeometry } from './geometry.ts'
import { procrustes2D } from './linalg.ts'

/**
 * One micro-step of real-time Schrodinger evolution, |psi> -> e^{-i H dt}|psi>,
 * via a truncated Taylor series. Accurate only when ||H|| * dt is small; callers
 * use `evolveInterval` which sub-divides the step accordingly.
 */
export function evolveStep(
  h: Hamiltonian,
  state: QuantumState,
  dt: number,
  order = 6,
): QuantumState {
  const out = cloneState(state)
  let term = cloneState(state)
  for (let m = 1; m <= order; m++) {
    const hTerm = h.apply(term)
    // term_m = (-i*dt/m) * H * term_{m-1};  (-i)(a+bi) = b - a i.
    const f = dt / m
    const next = hTerm
    for (let i = 0; i < state.dim; i++) {
      const a = hTerm.data[2 * i]
      const b = hTerm.data[2 * i + 1]
      next.data[2 * i] = f * b
      next.data[2 * i + 1] = -f * a
    }
    for (let i = 0; i < 2 * state.dim; i++) out.data[i] += next.data[i]
    term = next
  }
  return out
}

/**
 * Evolve for a time `dt`, sub-dividing into micro-steps so that the Taylor
 * series stays accurate (||H|| * microStep <= ~0.2). This keeps the evolution
 * essentially unitary even for large dt.
 */
export function evolveInterval(
  h: Hamiltonian,
  state: QuantumState,
  dt: number,
  radius: number,
  order = 6,
): QuantumState {
  const sub = Math.max(1, Math.ceil((Math.abs(dt) * radius) / 0.2))
  const micro = dt / sub
  let psi = state
  for (let i = 0; i < sub; i++) psi = evolveStep(h, psi, micro, order)
  return psi
}

export interface SpacetimeSlice {
  /** Clock reading index k. */
  k: number
  /** Emergent "time" t_k. */
  t: number
  /**
   * Emergent spatial coordinates of each qubit (1D or 2D), oriented
   * consistently across clock readings.
   */
  coords: number[][]
  /** Single-site <Z_i> observable at this clock reading. */
  zExpectation: number[]
  /**
   * Lieb-Robinson light-cone signal: |<Z_i>_perturbed - <Z_i>_reference|.
   * Isolates information spreading from the defect (vs. uniform on-site
   * precession that affects every site identically).
   */
  signal: number[]
  /** Total energy of the conditional state (should be conserved). */
  energy: number
}

export interface SpacetimeResult {
  sites: number
  slices: SpacetimeSlice[]
  /** Max |energy - energy_0| across slices: a unitarity / accuracy check. */
  energyDrift: number
  elapsedMs: number
}

export interface SpacetimeConfig {
  /** System Hamiltonian to evolve under. */
  hamiltonian: Hamiltonian
  /** Non-stationary initial state (e.g. a product state with a local defect). */
  initial: QuantumState
  /**
   * Defect-free reference state evolved in lockstep, used to isolate the
   * light-cone signal. If omitted, the signal is measured against t=0.
   */
  reference?: QuantumState
  /** Time step between clock readings. */
  dt: number
  /** Number of clock readings (slices). */
  steps: number
  /** Embedding dimension for the emergent geometry (1 or 2). */
  embedDim?: number
  /**
   * Reference layout (e.g. the true grid positions) used to Procrustes-align
   * 2D embeddings across clock readings so the sheet does not spin/flip.
   */
  alignTo?: { x: number; y: number }[]
  /** Taylor order for each step. */
  order?: number
}

/**
 * Build the sequence of clock-conditioned slices and their emergent geometry.
 * Each slice is <k|Psi> of the timeless global history state.
 */
export function buildSpacetime(config: SpacetimeConfig): SpacetimeResult {
  const start = performance.now()
  const {
    hamiltonian,
    initial,
    reference,
    dt,
    steps,
    embedDim = 1,
    alignTo,
    order = 6,
  } = config
  const sites = hamiltonian.n
  const alignTarget = alignTo?.map((p) => [p.x, p.y])

  const energy0 = hamiltonian.expectation(initial)
  let energyDrift = 0
  const radius = Math.max(hamiltonian.estimateSpectralRadius(() => 0.5), 1e-6)

  const slices: SpacetimeSlice[] = []
  let psi = cloneState(initial)
  let ref = reference ? cloneState(reference) : null
  const baseZ = Array.from({ length: sites }, (_, q) => expectationZ(initial, q))

  for (let k = 0; k < steps; k++) {
    const t = k * dt
    const energy = hamiltonian.expectation(psi)
    energyDrift = Math.max(energyDrift, Math.abs(energy - energy0))

    const report = analyzeEmergentGeometry(psi, 1, Math.max(embedDim, 1))
    let coords: number[][]
    if (embedDim >= 2) {
      const raw = report.mds.coords.map((c) => [c[0] ?? 0, c[1] ?? 0])
      coords = alignTarget ? procrustes2D(raw, alignTarget) : raw
    } else {
      coords = orientCoords(report.mds.coords.map((c) => c[0] ?? 0)).map((x) => [
        x,
      ])
    }
    const zExpectation = Array.from({ length: sites }, (_, q) =>
      expectationZ(psi, q),
    )
    const refZ = ref
      ? Array.from({ length: sites }, (_, q) => expectationZ(ref!, q))
      : baseZ
    const signal = zExpectation.map((z, q) => Math.abs(z - refZ[q]))

    slices.push({ k, t, coords, zExpectation, signal, energy })

    // Advance the clock (= advance the conditional state one Schrodinger step).
    psi = evolveInterval(hamiltonian, psi, dt, radius, order)
    normalize(psi)
    if (ref) {
      ref = evolveInterval(hamiltonian, ref, dt, radius, order)
      normalize(ref)
    }
  }

  return {
    sites,
    slices,
    energyDrift,
    elapsedMs: performance.now() - start,
  }
}

/**
 * Fix the sign/orientation ambiguity of the MDS axis so the emergent line does
 * not randomly flip between consecutive clock readings: orient so the
 * coordinate increases with qubit index.
 */
function orientCoords(coords: number[]): number[] {
  const n = coords.length
  if (n < 2) return coords
  // Correlate coordinate with index; flip if negative.
  let cov = 0
  const meanI = (n - 1) / 2
  const meanC = coords.reduce((a, b) => a + b, 0) / n
  for (let i = 0; i < n; i++) cov += (i - meanI) * (coords[i] - meanC)
  return cov < 0 ? coords.map((c) => -c) : coords.slice()
}

export interface LightCone {
  /** Emergent Lieb-Robinson velocity (sites per unit emergent time). */
  velocity: number
  /** First arrival time of the signal at each site (Infinity if never). */
  arrivals: number[]
  /** Index of the defect (signal source). */
  center: number
  /** Time step between clock readings. */
  dt: number
}

/**
 * Measure the emergent "speed of light" from a spacetime run: the front of the
 * light-cone signal. We find each site's arrival time (first crossing of a
 * threshold) and least-squares fit distance = velocity * time through the
 * origin. The slope is the emergent Lieb-Robinson velocity.
 */
export function measureLightCone(
  result: SpacetimeResult,
  thresholdFraction = 0.12,
): LightCone {
  const n = result.sites
  const center = Math.floor(n / 2)
  let max = 0
  for (const s of result.slices) for (const v of s.signal) max = Math.max(max, v)
  const thr = Math.max(max * thresholdFraction, 1e-6)

  const arrivals = new Array<number>(n).fill(Infinity)
  for (const slice of result.slices) {
    slice.signal.forEach((d, i) => {
      if (d > thr && slice.t < arrivals[i]) arrivals[i] = slice.t
    })
  }

  let num = 0
  let den = 0
  for (let i = 0; i < n; i++) {
    if (i === center) continue
    const t = arrivals[i]
    if (!isFinite(t) || t <= 0) continue
    const d = Math.abs(i - center)
    num += d * t
    den += t * t
  }
  const velocity = den > 0 ? num / den : 0
  const dt = result.slices.length > 1 ? result.slices[1].t - result.slices[0].t : 1
  return { velocity, arrivals, center, dt }
}

/** Apply a single Pauli-Z "kick" to a qubit, returning a new state. */
export function kickZ(state: QuantumState, qubit: number): QuantumState {
  const kicked = cloneState(state)
  const { dim } = state
  for (let s = 0; s < dim; s++) {
    if ((s >> qubit) & 1) {
      kicked.data[2 * s] = -kicked.data[2 * s]
      kicked.data[2 * s + 1] = -kicked.data[2 * s + 1]
    }
  }
  return kicked
}
