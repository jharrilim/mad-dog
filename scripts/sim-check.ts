/**
 * Numerical sanity check for the simulator engine.
 * Run with:  node scripts/sim-check.ts
 *
 * Expectations:
 *  - Bell pair: S_one_site = ln 2, MI = 2 ln 2.
 *  - TFIM chain: emergent dimension ~= 1, nearest neighbours most correlated.
 *  - TFIM grid: emergent dimension ~= 2.
 *  - Random non-local: high emergent dimension (no clean low-D space).
 */

import { groundState, makeRng, makeZeroState, normalize } from '../src/sim/quantum.ts'
import { tfimChain, tfimGrid, randomNonlocal } from '../src/sim/models.ts'
import {
  analyzeEmergentGeometry,
  entropyOneSite,
  mutualInformationMatrix,
} from '../src/sim/geometry.ts'
import { buildSpacetime } from '../src/sim/spacetime.ts'

function approx(a: number, b: number, tol = 1e-6): string {
  return Math.abs(a - b) < tol ? 'OK' : `MISMATCH (got ${a}, want ${b})`
}

// --- Bell state |00> + |11> ---
{
  const bell = makeZeroState(2)
  bell.data[2 * 0] = 1 // |00>
  bell.data[2 * 3] = 1 // |11>
  normalize(bell)
  const s = entropyOneSite(bell, 0)
  const mi = mutualInformationMatrix(bell)
  console.log('Bell pair:')
  console.log('  S(one site) =', s.toFixed(6), '  expect ln2 =', Math.LN2.toFixed(6), approx(s, Math.LN2, 1e-6))
  console.log('  I(0:1)      =', mi[0][1].toFixed(6), '  expect 2ln2 =', (2 * Math.LN2).toFixed(6), approx(mi[0][1], 2 * Math.LN2, 1e-6))
}

function summarize(label: string, report: ReturnType<typeof analyzeEmergentGeometry>) {
  const eig = report.mds.eigenvalues.slice(0, 6).map((v) => v.toFixed(3))
  console.log(`\n${label}`)
  console.log('  emergent dimension :', report.mds.emergentDim)
  console.log('  top eigenvalues    :', eig.join(', '))
  console.log('  explained var (d=1,2,3):',
    report.mds.explainedVariance.slice(0, 3).map((v) => v.toFixed(3)).join(', '))
}

const rng = makeRng(12345)

// --- TFIM chain (paramagnetic phase: correlations decay with distance) ---
{
  const model = tfimChain(8, 1, 1.5)
  const { state, energy, iters } = groundState(model.hamiltonian, rng, { maxIters: 3000 })
  console.log(`\n${model.label}: energy=${energy.toFixed(4)} (iters=${iters})`)
  const report = analyzeEmergentGeometry(state)
  summarize(model.label, report)
  // Nearest-neighbour vs far MI.
  const nn = report.mi[0][1]
  const far = report.mi[0][7]
  console.log('  MI(0:1) =', nn.toFixed(4), ' MI(0:7) =', far.toFixed(4),
    nn > far ? 'OK (neighbours more correlated)' : 'UNEXPECTED')
}

// --- TFIM grid 3x3 (paramagnetic phase) ---
{
  const model = tfimGrid(3, 3, 1, 1.5)
  const { state, energy } = groundState(model.hamiltonian, rng, { maxIters: 3000 })
  console.log(`\n${model.label}: energy=${energy.toFixed(4)}`)
  summarize(model.label, analyzeEmergentGeometry(state))
}

// --- Random non-local ---
{
  const model = randomNonlocal(8, makeRng(777))
  const { state, energy } = groundState(model.hamiltonian, rng, { maxIters: 3000 })
  console.log(`\n${model.label}: energy=${energy.toFixed(4)}`)
  summarize(model.label, analyzeEmergentGeometry(state))
}

// --- Page-Wootters emergent spacetime ---
{
  const n = 9
  const center = 4
  const model = tfimChain(n, 1, 1.0)
  // Polarized product state with a flipped defect at the centre, plus a
  // defect-free reference to isolate the light-cone signal.
  const initial = makeZeroState(n)
  initial.data[2 * (1 << center)] = 1
  const reference = makeZeroState(n)
  reference.data[0] = 1
  const result = buildSpacetime({
    hamiltonian: model.hamiltonian,
    initial,
    reference,
    dt: 0.2,
    steps: 20,
  })
  console.log('\nPage-Wootters emergent spacetime (9-site chain, kick at site 4):')
  console.log('  energy drift across clock readings:', result.energyDrift.toExponential(2),
    result.energyDrift < 1e-3 ? 'OK (evolution ~unitary)' : 'HIGH')

  // Light cone: connected <Z_center Z_i> perturbation should spread with a
  // finite front. We track, per clock reading, the farthest site whose |<Z_i>|
  // deviates noticeably from its initial value.
  console.log('  light-cone signal |<Z_i>_defect - <Z_i>_ref| (rows = clock time):')
  result.slices.forEach((slice, k) => {
    if (k % 2 !== 0) return
    const row = slice.signal
      .map((d) => (d > 0.3 ? '#' : d > 0.1 ? '+' : d > 0.05 ? ':' : '·'))
      .join(' ')
    console.log(`    t=${slice.t.toFixed(2)}  ${row}`)
  })

  // Causality test: the signal should ARRIVE later at sites farther from the
  // defect (finite Lieb-Robinson velocity = emergent "speed of light").
  const arrival = Array.from({ length: n }, () => Infinity)
  result.slices.forEach((slice) => {
    slice.signal.forEach((d, i) => {
      if (d > 0.05 && slice.t < arrival[i]) arrival[i] = slice.t
    })
  })
  let causal = true
  for (let i = center + 1; i < n; i++) {
    if (arrival[i] < arrival[i - 1] - 1e-9) causal = false
  }
  for (let i = center - 1; i >= 0; i--) {
    if (arrival[i] < arrival[i + 1] - 1e-9) causal = false
  }
  console.log('  arrival times by site:', arrival.map((a) => a.toFixed(1)).join(' '))
  console.log('  arrival delayed with distance (finite speed):', causal ? 'OK' : 'NO')
}

console.log('\nDone.')
