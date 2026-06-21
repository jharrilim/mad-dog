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

import {
  groundState,
  makeRandomState,
  makeRng,
  makeZeroState,
  normalize,
} from '../src/sim/quantum.ts'
import { tfimChain, tfimGrid, tfimCube, randomNonlocal } from '../src/sim/models.ts'
import {
  analyzeEmergentGeometry,
  entropyOfRegion,
  entropyOneSite,
  entropyTwoSite,
  mutualInformationMatrix,
} from '../src/sim/geometry.ts'
import { buildSpacetime } from '../src/sim/spacetime.ts'
import { analyzeHolography, analyzeRtMassDeformation } from '../src/sim/holography.ts'
import { runRelationalTime, runUniverse3D } from '../src/sim/runner.ts'

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

// --- entropyOfRegion agrees with the dedicated one/two-site routines ---
{
  const model = tfimChain(8, 1, 1.2)
  const { state } = groundState(model.hamiltonian, makeRng(2024), { maxIters: 3000 })
  const s1a = entropyOneSite(state, 3)
  const s1b = entropyOfRegion(state, [3])
  const s2a = entropyTwoSite(state, 2, 5)
  const s2b = entropyOfRegion(state, [2, 5])
  // Pure-state symmetry: S_A = S_complement.
  const sLeft = entropyOfRegion(state, [0, 1, 2, 3])
  const sRight = entropyOfRegion(state, [4, 5, 6, 7])
  console.log('\nentropyOfRegion consistency:')
  console.log('  vs one-site :', approx(s1a, s1b, 1e-9))
  console.log('  vs two-site :', approx(s2a, s2b, 1e-9))
  console.log('  S_A = S_comp:', approx(sLeft, sRight, 1e-9))
}

// --- Baby Ryu-Takayanagi: area law vs volume law + entropy = boundary area ---
{
  const n = 10
  const model = tfimChain(n, 1, 1.5)
  const { state } = groundState(model.hamiltonian, makeRng(31337), { maxIters: 4000 })
  const random = makeRandomState(n, makeRng(99))
  const report = analyzeHolography(state, random)

  console.log('\nBaby Ryu-Takayanagi (10-site chain, h=1.5):')
  console.log('  S(A) vs region size |A| (edge-anchored):')
  console.log('    |A|      :', report.areaLaw.map((p) => p.size.toString().padStart(5)).join(''))
  console.log('    ground   :', report.areaLaw.map((p) => p.sGround.toFixed(2).padStart(5)).join(''))
  console.log('    random   :', report.areaLaw.map((p) => p.sRandom.toFixed(2).padStart(5)).join(''))

  // Area law: ground-state entropy of the bulk-spanning region stays well below
  // the random (volume-law) value at the same size.
  const mid = report.areaLaw[Math.floor(report.areaLaw.length / 2)]
  console.log(
    `  at |A|=${mid.size}: ground=${mid.sGround.toFixed(3)} << random=${mid.sRandom.toFixed(3)}`,
    mid.sGround < 0.6 * mid.sRandom ? 'OK (area << volume)' : 'UNEXPECTED',
  )

  // Random state should grow with size up to the half-chain (volume law).
  const grows = report.areaLaw
    .slice(0, Math.floor(n / 2))
    .every((p, i, arr) => i === 0 || p.sRandom > arr[i - 1].sRandom - 1e-9)
  console.log('  random S grows with |A| (volume law):', grows ? 'OK' : 'NO')

  console.log(
    `  RT fit  S_A = ${report.rtSlope.toFixed(3)} * (boundary MI cut),  R^2 = ${report.rtR2.toFixed(4)}`,
    report.rtR2 > 0.9 ? 'OK (entropy tracks boundary area)' : 'WEAK',
  )
}

// --- Two-clock relational time ---
{
  const n = 9
  const center = Math.floor(n / 2)
  const edge = runRelationalTime({
    n,
    field: 1,
    dt: 0.2,
    steps: 40,
    clockSite: 0,
    physicalSlices: 15,
  })
  const atDefect = runRelationalTime({
    n,
    field: 1,
    dt: 0.2,
    steps: 40,
    clockSite: center,
    physicalSlices: 15,
  })
  console.log('\nTwo-clock relational time (9-site chain, defect at center):')
  console.log(
    `  edge clock (site 0): sync R^2 = ${edge.syncR2.toFixed(4)}`,
    edge.syncR2 < 0.95 ? 'OK (clocks disagree)' : 'UNEXPECTED (too synchronized)',
  )
  console.log(
    `  defect clock (site ${center}): sync R^2 = ${atDefect.syncR2.toFixed(4)}`,
    atDefect.syncR2 > 0.99 ? 'OK (clock at defect stays in sync)' : 'UNEXPECTED',
  )
}

// --- RT slope deformation under local "mass" ---
{
  const n = 10
  const model = tfimChain(n, 1, 1.5)
  const { state } = groundState(model.hamiltonian, makeRng(7), { maxIters: 4000 })
  const center = Math.floor(n / 2)
  const vacuum = analyzeRtMassDeformation(model.hamiltonian, state, center, 0)
  const heavy = analyzeRtMassDeformation(model.hamiltonian, state, center, 1.0)
  const delta = heavy.mass.rtSlope - vacuum.vacuum.rtSlope
  console.log('\nRT slope under mass (10-site chain, h=1.5):')
  console.log(
    `  vacuum slope = ${vacuum.vacuum.rtSlope.toFixed(3)}`,
    Math.abs(vacuum.vacuum.rtSlope - 1) < 0.1 ? 'OK (~1)' : 'UNEXPECTED',
  )
  console.log(
    `  mass slope   = ${heavy.mass.rtSlope.toFixed(3)}  (Δ = ${delta.toFixed(3)})`,
    delta > 0.2 ? 'OK (slope deformed upward)' : 'WEAK',
  )
  console.log(
    `  mass R^2     = ${heavy.mass.rtR2.toFixed(3)}`,
    heavy.mass.rtR2 > 0.85 ? 'OK (relation still holds roughly)' : 'BROKEN',
  )
}

// --- TFIM cube 2x2x2 (emergent 3D) ---
{
  const model = tfimCube(2, 2, 2, 1, 1.5)
  const { state, energy } = groundState(model.hamiltonian, makeRng(42), {
    maxIters: 4000,
  })
  console.log(`\n${model.label}: energy=${energy.toFixed(4)}`)
  const report = analyzeEmergentGeometry(state, 1, 3)
  summarize(model.label, report)
  const mi = report.mi
  const nn = mi[0][1]
  let far = 0
  for (let j = 0; j < 8; j++) if (j !== 0) far = Math.max(far, mi[0][j])
  console.log(
    '  MI(neighbour) vs MI(far):',
    nn.toFixed(4),
    far.toFixed(4),
    nn >= far * 0.5 ? 'OK (local structure)' : 'WEAK',
  )
}

// --- 3+1 universe lab (cube quench, 3D coords) ---
{
  const uni = runUniverse3D({
    lx: 2,
    ly: 2,
    lz: 2,
    field: 1,
    dt: 0.25,
    steps: 12,
  })
  const slice = uni.spacetime.slices[0]
  const dim3 = slice.coords.every((c) => c.length >= 3)
  console.log('\n3+1 universe (2x2x2 cube quench):')
  console.log(
    '  3D coords per site:',
    dim3 ? 'OK' : 'NO',
    'sample:',
    slice.coords[0].map((v) => v.toFixed(2)).join(','),
  )
  console.log(
    '  energy drift:',
    uni.spacetime.energyDrift.toExponential(2),
    uni.spacetime.energyDrift < 1e-2 ? 'OK' : 'HIGH',
  )
  console.log(
    '  LR velocity:',
    uni.lightCone.velocity.toFixed(3),
    uni.lightCone.velocity > 0 ? 'OK (finite)' : 'WEAK',
  )
}

console.log('\nDone.')
